//! 流离（大乔）：成为杀目标时弃一张牌把此杀转移给攻击范围内的另一名角色的三段询问；进行中返回本步结果。

use super::*;

impl ShaResolveMachine {
    /// 流离（大乔）：成为杀目标时弃一张牌把此杀转移给攻击范围内的另一名角色的三段询问；进行中返回本步结果。
    pub(super) fn step_liuli(&mut self, ctx: &mut Ctx<'_>) -> Option<Flow> {
    // 流离（大乔；逐技对照批二 v76 对齐上游 chooseCardTarget）：成为
    // 杀目标时可自选弃置一张牌（手牌+装备）并将此杀转移给攻击范围内
    // 的一名其他角色（非此杀使用者、非自己、且为该杀合法目标——空城
    // 不可）。整体可取消：弃牌询问空应答=不发动；先选牌后选目标，
    // 转移生效后弃置暂存牌（上游 content 顺序）。
    if !self.liuli_done {
        self.liuli_done = true;
        let liuli = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::LIU_LI)
            .map(|d| ctx.core.has_skill(self.target, d))
            .unwrap_or(false);
        let has_he = !ctx.core.player(self.target).hand.is_empty()
            || !ctx.core.player(self.target).equips.is_empty();
        if liuli && has_he && !self.liuli_candidates(ctx).is_empty() {
            let mut options: Vec<ChoiceOption> = ctx
                .core
                .player(self.target)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            options.extend(
                ctx.core
                    .player(self.target)
                    .equips
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card),
            );
            self.liuli_stage = 1;
            return Some(Flow::WaitChoice {
                observer: self.target,
                spec: ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "liuli_discard_choose",
                    options,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            });
        }
    } else if self.liuli_stage == 1 {
        self.liuli_stage = 0;
        let pick = ctx.answer().first().and_then(|o| match o {
            ChoiceOption::Card(c) => Some(*c),
            _ => None,
        });
        if let Some(c) = pick.filter(|c| {
            ctx.core.player(self.target).hand.contains(c)
                || ctx.core.player(self.target).equips.contains(c)
        }) {
            self.liuli_card = Some(c);
            let candidates = self.liuli_candidates(ctx);
            if !candidates.is_empty() {
                self.liuli_stage = 2;
                return Some(Flow::WaitChoice {
                    observer: self.target,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "liuli_transfer",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                });
            }
            self.liuli_card = None;
        }
    } else if self.liuli_stage == 2 {
        self.liuli_stage = 0;
        let from = self.target;
        if let Some(ChoiceOption::Player(new_target)) = ctx.answer().first().copied() {
            if self.liuli_candidate_ok(ctx, new_target) {
                self.target = new_target;
                // 新目标重新判定八卦/铁骑；护驾窗口对新目标重开
                self.armor_judged = false;
                self.tieji_red = None;
                self.asked = false;
                self.hujia_queue = None;
                self.hujia_holder = None;
            }
        }
        // 上游 content：转移生效后弃置所选牌（手牌或装备；牌在
        // 被转移者 from 的区域——self.target 已指向新目标）。
        if let Some(c) = self.liuli_card.take() {
            let in_hand = ctx.core.player(from).hand.contains(&c);
            if in_hand {
                ctx.discard_from_hand(from, c);
            } else if ctx.core.player(from).equips.contains(&c) {
                ctx.core.player_mut(from).equips.retain(|x| *x != c);
                crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, from, c);
                ctx.to_discard(c);
            }
            ctx.log(LogEvent::CardDiscarded {
                player: from,
                cards: vec![c],
            });
        }
    }
        None
    }
}
