use super::*;

// 离间（貂蝉）：出牌阶段限一次——弃一张手牌置于桌面作为载体，令两名
// 其他角色决斗（登记差异：不限男性角色）；借牌直结算（内嵌决斗机器）。
#[derive(Clone)]
pub(in super::super) struct LiJianBehavior;

impl SkillBehavior for LiJianBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LiJianMachine {
            owner,
            stage: LiJianStage::DiscardAsk,
            card: None,
            inner: None,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.is_empty() {
            return None;
        }
        if ctx.state.alive_players().count() < 3 {
            return None;
        }
        Some(Box::new(LiJianMachine {
            owner,
            stage: LiJianStage::DiscardAsk,
            card: None,
            inner: None,
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum LiJianStage {
    /// 挂起询问 = lijian_discard（弃一张手牌作为离间载体；取消 = 放弃）。
    DiscardAsk,
    /// 消费 lijian_discard 并挂起 lijian_targets（两名其他角色）。
    TargetsAsk,
    /// 消费 lijian_targets：载体牌置桌并内嵌决斗直结算。
    Launch,
}

pub(in super::super) struct LiJianMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: LiJianStage,
    /// 载体牌（弃置的手牌，置于桌面参与决斗结算）。
    pub(in super::super) card: Option<CardId>,
    pub(in super::super) inner: Option<Box<dyn SkillMachine>>,
}

impl SkillMachine for LiJianMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(Self {
            owner: self.owner,
            stage: self.stage,
            card: self.card,
            inner: self.inner.as_ref().map(|m| m.fork()),
        })
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::LiJian {
            owner: self.owner,
            stage: match self.stage {
                LiJianStage::DiscardAsk => LiJianStageSnap::DiscardAsk,
                LiJianStage::TargetsAsk => LiJianStageSnap::TargetsAsk,
                LiJianStage::Launch => LiJianStageSnap::Launch,
            },
            card: self.card,
            inner: self.inner.as_ref().map(|m| Box::new(m.snapshot())),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 内嵌决斗存在即全权委托
        if let Some(inner) = self.inner.as_mut() {
            let flow = inner.step(ctx);
            if matches!(flow, Flow::Done) {
                self.inner = None;
                return Flow::Done;
            }
            return flow;
        }
        match self.stage {
            LiJianStage::DiscardAsk => {
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
                self.stage = LiJianStage::TargetsAsk;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "lijian_discard",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            LiJianStage::TargetsAsk => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                self.card = Some(c);
                // 性别门控：离间限男性角色
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner && seat_is_male(ctx, *p))
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.len() < 2 {
                    return Flow::Done;
                }
                self.stage = LiJianStage::Launch;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "lijian_targets",
                        options: candidates,
                        min: 2,
                        max: 2,
                        allow_cancel: true,
                    },
                }
            }
            LiJianStage::Launch => {
                let Some(card) = self.card else {
                    return Flow::Done;
                };
                let picks: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if picks.len() < 2 {
                    // 弃权：载体牌离手入弃牌堆
                    if ctx.core.player(self.owner).hand.contains(&card) {
                        ctx.discard_from_hand(self.owner, card);
                    }
                    return Flow::Done;
                }
                // 载体牌从手牌置于桌面，作为决斗结算的牌
                ctx.core.player_mut(self.owner).hand.retain(|x| *x != card);
                ctx.core.table.push(card);
                let duel =
                    crate::kernel::DuelMachine::new(card, picks[0], picks[1], true, false, false);
                self.inner = Some(Box::new(duel));
                let flow = self.inner.as_mut().expect("just set").step(ctx);
                if matches!(flow, Flow::Done) {
                    self.inner = None;
                    return Flow::Done;
                }
                flow
            }
        }
    }
}
