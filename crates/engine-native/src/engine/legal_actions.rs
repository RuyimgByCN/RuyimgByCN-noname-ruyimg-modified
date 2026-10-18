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
/// 候选探测触发内核不变量 panic 的累计次数（NC12 长程防护；供测试/诊断读取）。
static PROBE_FAULTS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// 探测故障上下文环形缓冲（NC12 根因诊断；不打印，仅内存记录，测试可读）。
static PROBE_FAULT_LOG: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// 记录一次探测故障上下文（内部使用；最多 32 条）。
pub(crate) fn note_probe_fault_context(context: String) {
    if let Ok(mut log) = PROBE_FAULT_LOG.lock() {
        if log.len() >= 32 {
            log.remove(0);
        }
        log.push(context);
    }
}

/// 读取探测故障上下文（诊断用；按发生顺序，最多 32 条）。
pub fn probe_fault_log() -> Vec<String> {
    PROBE_FAULT_LOG.lock().map(|l| l.clone()).unwrap_or_default()
}

/// 清空探测故障上下文（测试用）。
pub fn clear_probe_fault_log() {
    if let Ok(mut log) = PROBE_FAULT_LOG.lock() {
        log.clear();
    }
}

/// 记录一次探测故障（内部使用）。
pub(crate) fn note_probe_fault() {
    PROBE_FAULTS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// 读取探测故障累计数（诊断用；0 = 未发生）。
pub fn probe_faults() -> u64 {
    PROBE_FAULTS.load(std::sync::atomic::Ordering::Relaxed)
}

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
    // NC12 长程防护（2026-10-19）：候选探测同样在**克隆**上真实 `submit`，若内核在
    // 该探测状态上触发不变量 panic（实测 `machine.rs:408 card N not in hand of P`，
    // 回溯 legal_actions::push_action → submit → finish_segment → commit_frame），
    // **不得杀死进程**——捕获后视为「该动作不合法」并计数。
    // 真实提交路径不捕获（内核不变量在真实引擎仍硬失败）。
    let probed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        engine.probe_action(seat, &action)
    }));
    let legal = match probed {
        Ok(r) => r.is_ok(),
        Err(_) => {
            crate::engine::legal_actions::note_probe_fault();
            crate::engine::legal_actions::note_probe_fault_context(format!(
                "seat={seat} action={action:?} rev={} phase={:?} table={:?}",
                engine.revision(),
                engine.turn().phase,
                engine.state_core().table
            ));
            false
        }
    };
    if legal {
        out.push(action);
    }
}
