use super::*;

// 结姻（孙尚香）：出牌阶段限一次——弃两张手牌，与一名**受伤的**
// 其他男性角色各回复 1 点体力（v202 对齐上游 filterTarget：male +
// hp<maxHp + 非自己；上游无「已婚」「装备区有牌」条件，旧登记撤销）。
#[derive(Clone)]
pub(in super::super) struct JieYinBehavior;

impl SkillBehavior for JieYinBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JieYinMachine {
            owner,
            stage: JieYinStage::CardsAsk,
            cards: Vec::new(),
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.len() < 2 {
            return None;
        }
        if !ctx.state.alive_players().any(|t| {
            t != owner
                && slots_is_male(&ctx.state.player(t).general_slots)
                && ctx.state.player(t).hp < ctx.state.player(t).max_hp as i32
        }) {
            return None;
        }
        Some(Box::new(JieYinMachine {
            owner,
            stage: JieYinStage::CardsAsk,
            cards: Vec::new(),
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum JieYinStage {
    /// 挂起询问 = jieyin_cards（弃两张手牌；不足两枚或取消 = 放弃）。
    CardsAsk,
    /// 消费 jieyin_cards 并挂起 jieyin_target 询问。
    TargetAsk,
    /// 消费 jieyin_target：双方各回复 1 点体力。
    TargetRead,
}

#[derive(Clone)]
pub(in super::super) struct JieYinMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: JieYinStage,
    pub(in super::super) cards: Vec<CardId>,
}

impl SkillMachine for JieYinMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JieYin {
            owner: self.owner,
            stage: match self.stage {
                JieYinStage::CardsAsk => JieYinStageSnap::CardsAsk,
                JieYinStage::TargetAsk => JieYinStageSnap::TargetAsk,
                JieYinStage::TargetRead => JieYinStageSnap::TargetRead,
            },
            cards: self.cards.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            JieYinStage::CardsAsk => {
                let hand = ctx.core.player(self.owner).hand.clone();
                if hand.len() < 2 {
                    return Flow::Done;
                }
                let options: Vec<ChoiceOption> =
                    hand.iter().copied().map(ChoiceOption::Card).collect();
                self.stage = JieYinStage::TargetAsk;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "jieyin_cards",
                        options,
                        min: 2,
                        max: 2,
                        allow_cancel: true,
                    },
                }
            }
            JieYinStage::TargetAsk => {
                self.cards = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if self.cards.len() < 2 {
                    return Flow::Done;
                }
                // 目标门控（v202 对齐上游 filterTarget）：其他受伤男性
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|t| {
                        *t != self.owner
                            && seat_is_male(ctx, *t)
                            && ctx.core.player(*t).hp < ctx.core.player(*t).max_hp as i32
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = JieYinStage::TargetRead;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "jieyin_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            JieYinStage::TargetRead => {
                let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(target).alive {
                    return Flow::Done;
                }
                for c in &self.cards {
                    ctx.discard_from_hand(self.owner, *c);
                }
                ctx.recover(self.owner, 1, None);
                ctx.recover(target, 1, None);
                Flow::Done
            }
        }
    }
}
