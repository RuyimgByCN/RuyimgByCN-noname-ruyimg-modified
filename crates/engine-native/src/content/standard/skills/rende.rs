use super::*;

// 仁德（刘备）：出牌阶段限一次的主动技——将任意张手牌交给一名
// 其他角色；给出的牌不少于两张时回复 1 点体力（体力已满则无效）。
#[derive(Clone)]
pub(in super::super) struct RendeBehavior;

impl SkillBehavior for RendeBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false // 主动技：不随事件触发
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(RendeMachine {
            owner,
            stage: RendeStage::SelectCardsAsk,
            cards: Vec::new(),
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.is_empty() {
            return None; // 无手牌可给：无法发动
        }
        Some(Box::new(RendeMachine {
            owner,
            stage: RendeStage::SelectCardsAsk,
            cards: Vec::new(),
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum RendeStage {
    /// 挂起询问 = rende_cards（选择给出的手牌，空应答 = 放弃）。
    SelectCardsAsk,
    /// 消费 rende_cards 应答并挂起 rende_target 询问。
    TargetAsk,
    /// 消费 rende_target 应答并交付（≥2 张则回复 1 点体力）。
    TargetRead,
}

#[derive(Clone)]
pub(in super::super) struct RendeMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: RendeStage,
    /// SelectCards 应答选中的牌（待交付）。
    pub(in super::super) cards: Vec<CardId>,
}

impl SkillMachine for RendeMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Rende {
            owner: self.owner,
            stage: match self.stage {
                RendeStage::SelectCardsAsk => RendeStageSnap::SelectCardsAsk,
                RendeStage::TargetAsk => RendeStageSnap::TargetAsk,
                RendeStage::TargetRead => RendeStageSnap::TargetRead,
            },
            cards: self.cards.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            RendeStage::SelectCardsAsk => {
                if ctx.core.player(self.owner).hand.is_empty() {
                    return Flow::Done;
                }
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                let max_cards = options.len();
                self.stage = RendeStage::TargetAsk;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "rende_cards",
                        options,
                        min: 0,
                        max: max_cards,
                        allow_cancel: true,
                    },
                }
            }
            RendeStage::TargetAsk => {
                self.cards = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if self.cards.is_empty() {
                    return Flow::Done; // 空应答：放弃发动
                }
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = RendeStage::TargetRead;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "rende_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            RendeStage::TargetRead => {
                let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                    return Flow::Done; // 弃权：不交付
                };
                if !ctx.core.player(target).alive {
                    return Flow::Done;
                }
                // 仁德累计（ADR 0776 对齐上游 rende1/content）：按本
                // 出牌阶段累计给出数判定——此前 <2 且累计后 ≥2 时回复
                // 1 点（每次发动都可给牌，无每回合一次限制）。
                let num_before = ctx.core.turn.rende_given;
                for c in &self.cards {
                    ctx.give_hand_card(self.owner, target, *c);
                }
                let given = self.cards.len() as u8;
                ctx.core.turn.rende_given = num_before.saturating_add(given);
                if num_before < 2 && num_before + given > 1 {
                    ctx.recover(self.owner, 1, None);
                }
                Flow::Done
            }
        }
    }
}
