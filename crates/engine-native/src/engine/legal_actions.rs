//! NC03 权威动作查询：由内核给出「该座位此刻可提交的候选动作」。
//!
//! 修复前：前端按 `hand.slice(0, 8)` 拼接 `PlayCard`、AI 在 `ai.rs` 里镜像
//! 规则枚举候选（`enumerate_candidates_variant`，注释即「非法由模拟拒绝」），
//! 规则合法性被复制到内核之外——客户端/AI 各自写一套，可能给出非法或遗漏
//! 合法动作。
//!
//! 本查询**以内核校验为准**：候选先由视图粗筛，再在克隆引擎上走与正式提交
//! 相同的 `submit` 校验，仅保留被接受的候选；因此「合法动作」的定义只有一处。
//!
//! 边界（登记差异/待办）：
//! - 候选空间有上限（手牌数 × 目标组合），非穷尽；超大局面需分页/分步接口；
//! - 克隆校验的代价随状态规模增长（NC12 基线测量；后续可换成非变异校验入口）；
//! - 主动技候选仅枚举「已授予且启用」的技能定义，未展开技能内部选择。

use crate::engine::{Engine, GameAction, GameCommand, RejectReason};

/// 单次查询的候选上限（防止组合爆炸；超限时返回已达上限的子集）。
pub const MAX_LEGAL_ACTIONS: usize = 256;

/// 每张手牌尝试的目标组合上限。
const MAX_TARGET_COMBOS: usize = 8;

impl Engine {
    /// 权威动作查询：`seat` 当前可提交的候选动作（已通过内核校验）。
    ///
    /// 顺序稳定（先应答、后主动技、再出牌、最后阶段推进），便于回放与测试。
    pub fn legal_actions(&self, seat: crate::ids::PlayerId) -> Vec<GameAction> {
        let mut out: Vec<GameAction> = Vec::new();
        let mut dedup = std::collections::BTreeSet::new();

        // ① 开放选择：枚举该观察者的合法应答（单选逐项、多选取前 min 项、取消）
        if let Some(choice) = self.open_choice() {
            if choice.observer == seat {
                for opt in choice.spec.options.iter().take(MAX_LEGAL_ACTIONS) {
                    push_action(
                        self,
                        seat,
                        GameAction::Answer {
                            choice_id: choice.id,
                            selection: vec![*opt],
                        },
                        &mut out,
                        &mut dedup,
                    );
                }
                if choice.spec.min >= 2 {
                    let sel: Vec<crate::choice::ChoiceOption> = choice
                        .spec
                        .options
                        .iter()
                        .take(choice.spec.min as usize)
                        .copied()
                        .collect();
                    if !sel.is_empty() {
                        push_action(
                            self,
                            seat,
                            GameAction::Answer {
                                choice_id: choice.id,
                                selection: sel,
                            },
                            &mut out,
                            &mut dedup,
                        );
                    }
                }
                if choice.spec.allow_cancel {
                    push_action(
                        self,
                        seat,
                        GameAction::Answer {
                            choice_id: choice.id,
                            selection: Vec::new(),
                        },
                        &mut out,
                        &mut dedup,
                    );
                }
                return out;
            }
            // 有开放选择时，非观察者无合法动作（内核会以 ChoicePending 拒绝）
            return out;
        }

        // ② 主动技（仅自己回合的出牌阶段；合法性由内核校验）
        if self.turn().current == seat && self.turn().phase == crate::event::Phase::Play {
            // 主动技候选：已授予、启用、且（每回合限一次者）本回合未用尽；
            // 具体合法性仍由内核校验（probe_action）。
            let defs: Vec<crate::ids::SkillDefId> = self
                .state_core()
                .skills
                .iter()
                .filter(|((owner, _), inst)| {
                    *owner == seat && inst.enabled && !(inst.once_used)
                })
                .map(|((_, def), _)| *def)
                .collect();
            for def in defs {
                push_action(
                    self,
                    seat,
                    GameAction::ActivateSkill { skill: def },
                    &mut out,
                    &mut dedup,
                );
                if out.len() >= MAX_LEGAL_ACTIONS {
                    return out;
                }
            }

            // ③ 出牌：逐手牌 × 目标组合（自目标/无目标 + 单目标 + 双目标）
            let hand: Vec<crate::ids::CardId> = self.debug_hand(seat);
            let others: Vec<crate::ids::PlayerId> = (0..self.state_core().players.len() as u8)
                .filter(|p| *p != seat && self.state_core().player(*p).alive)
                .collect();
            'cards: for card in hand {
                let mut combos: Vec<Vec<crate::ids::PlayerId>> = vec![Vec::new()];
                for t in others.iter().take(MAX_TARGET_COMBOS) {
                    combos.push(vec![*t]);
                }
                if others.len() >= 2 {
                    combos.push(vec![others[0], others[1]]);
                }
                for targets in combos {
                    push_action(
                        self,
                        seat,
                        GameAction::PlayCard { card, targets },
                        &mut out,
                        &mut dedup,
                    );
                    if out.len() >= MAX_LEGAL_ACTIONS {
                        break 'cards;
                    }
                }
            }
        }

        // ④ 阶段推进（内核校验轮次/阶段/暂停状态）
        push_action(self, seat, GameAction::NextPhase, &mut out, &mut dedup);
        out
    }

    /// 单个动作是否可提交（内核校验；不改变本引擎状态）。
    pub fn is_legal_action(&self, seat: crate::ids::PlayerId, action: &GameAction) -> bool {
        self.probe_action(seat, action).is_ok()
    }

    /// 非变异校验探针：在克隆上走与正式提交相同的校验路径。
    pub fn probe_action(
        &self,
        seat: crate::ids::PlayerId,
        action: &GameAction,
    ) -> Result<(), RejectReason> {
        let mut probe = self.clone();
        match probe.submit(GameCommand {
            actor: seat,
            expected_revision: self.revision(),
            // 去重高水位耗尽（u64::MAX）时用饱和值：探针会以 DuplicateCommand 被拒，
            // 表现为「无候选」而不是算术溢出 panic。
            dedup_id: self.last_dedup.saturating_add(1),
            action: action.clone(),
        }) {
            crate::engine::CommandOutcome::Rejected(reason) => Err(reason),
            _ => Ok(()),
        }
    }
}

/// 候选入列：内核校验通过且未重复才加入。
fn push_action(
    engine: &Engine,
    seat: crate::ids::PlayerId,
    action: GameAction,
    out: &mut Vec<GameAction>,
    dedup: &mut std::collections::BTreeSet<String>,
) {
    if out.len() >= MAX_LEGAL_ACTIONS {
        return;
    }
    let key = format!("{action:?}");
    if !dedup.insert(key) {
        return;
    }
    if engine.probe_action(seat, &action).is_ok() {
        out.push(action);
    }
}
