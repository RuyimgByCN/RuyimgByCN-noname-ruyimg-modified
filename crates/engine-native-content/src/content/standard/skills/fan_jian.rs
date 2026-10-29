use super::*;

// 反间（周瑜）：出牌阶段限一次——一名其他角色先选一种花色（v203 对齐
// 上游 chooseControl：♥/♦/♣/♠），再从周瑜手牌中任选一张获得；花色与所
// 选不符时受到周瑜造成的 1 点伤害（上游条件 damage，花色相符无事）。
#[derive(Clone)]
pub(in super::super) struct FanJianBehavior;

impl SkillBehavior for FanJianBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(FanJianMachine {
            owner,
            stage: FanJianStage::TargetAsk,
            target: None,
            chosen_suit: None,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.is_empty() {
            return None;
        }
        if !ctx.state.alive_players().any(|t| t != owner) {
            return None;
        }
        Some(Box::new(FanJianMachine {
            owner,
            stage: FanJianStage::TargetAsk,
            target: None,
            chosen_suit: None,
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum FanJianStage {
    /// 挂起询问 = fanjian_target（选择一名其他角色；取消 = 放弃）。
    TargetAsk,
    /// 挂起询问 = fanjian_suit（目标选一种花色；v203 对齐上游）。
    SuitAsk,
    /// 挂起询问 = fanjian_pick（目标选周瑜一张手牌获得）。
    PickAsk,
    /// 消费 fanjian_pick：交牌并按花色比对条件伤害。
    Consume,
}

#[derive(Clone)]
pub(in super::super) struct FanJianMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: FanJianStage,
    pub(in super::super) target: Option<PlayerId>,
    /// 目标所选花色 tag（0=♥ 1=♦ 2=♣ 3=♠，上游 controls 序）。
    pub(in super::super) chosen_suit: Option<u8>,
}

/// 反间花色 tag：0=♥ 1=♦ 2=♣ 3=♠（上游 chooseControl 序）。
fn fanjian_suit_tag(s: crate::catalog::Suit) -> u8 {
    match s {
        crate::catalog::Suit::Heart => 0,
        crate::catalog::Suit::Diamond => 1,
        crate::catalog::Suit::Club => 2,
        crate::catalog::Suit::Spade => 3,
    }
}

impl SkillMachine for FanJianMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::FanJian {
            owner: self.owner,
            stage: match self.stage {
                FanJianStage::TargetAsk => FanJianStageSnap::TargetAsk,
                FanJianStage::SuitAsk => FanJianStageSnap::SuitAsk,
                FanJianStage::PickAsk => FanJianStageSnap::PickAsk,
                FanJianStage::Consume => FanJianStageSnap::Consume,
            },
            target: self.target,
            chosen_suit: self.chosen_suit,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            FanJianStage::TargetAsk => {
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|t| *t != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = FanJianStage::SuitAsk;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "fanjian_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            FanJianStage::SuitAsk => {
                // 消费 fanjian_target 应答
                let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(target).alive {
                    return Flow::Done;
                }
                self.target = Some(target);
                self.stage = FanJianStage::PickAsk;
                Flow::WaitChoice {
                    observer: target,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: "fanjian_suit",
                        options: (0..=3).map(ChoiceOption::Tag).collect(),
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            FanJianStage::PickAsk => {
                let Some(ChoiceOption::Tag(suit)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                self.chosen_suit = Some(suit);
                let Some(target) = self.target else {
                    return Flow::Done;
                };
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                self.stage = FanJianStage::Consume;
                Flow::WaitChoice {
                    observer: target,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "fanjian_pick",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            FanJianStage::Consume => {
                let Some(target) = self.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    let suit_matches = self
                        .chosen_suit
                        .is_some_and(|t| fanjian_suit_tag(ctx.catalog.cards[c as usize].suit) == t);
                    ctx.give_hand_card(self.owner, target, c);
                    // v203 对齐上游：花色相符无事；不符才受 1 点伤害
                    if !suit_matches {
                        ctx.emit(EventData::Damage(crate::machine::damage_data(
                            Some(self.owner),
                            target,
                            1,
                        )));
                    }
                }
                Flow::Done
            }
        }
    }
}
