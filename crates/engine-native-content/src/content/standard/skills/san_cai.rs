use super::*;

// ---------------------------------------------------------------------------
// 散财：受到伤害后，可以摸两张牌，然后将一张手牌交给另一名玩家。
// ---------------------------------------------------------------------------

pub(in super::super) struct SanCaiBehavior;

impl SkillBehavior for SanCaiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(SanCaiMachine {
            owner,
            stage: SanCaiStage::Draw,
            target: None,
        })
    }
}

#[derive(Clone)]
pub(in super::super) enum SanCaiStage {
    /// 摸两张（子事件结算后继续）。
    Draw,
    /// 询问交给谁（恢复后读目标）。
    Target,
    /// 已知目标，询问交哪张牌（恢复后读牌）。
    GiveCard,
    /// 已知目标与牌，执行给予。
    Finish,
}

#[derive(Clone)]
pub(in super::super) struct SanCaiMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: SanCaiStage,
    pub(in super::super) target: Option<PlayerId>,
}

impl SkillMachine for SanCaiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::SanCai {
            owner: self.owner,
            stage: match self.stage {
                SanCaiStage::Draw => SanCaiStageSnap::Draw,
                SanCaiStage::Target => SanCaiStageSnap::Target,
                SanCaiStage::GiveCard => SanCaiStageSnap::GiveCard,
                SanCaiStage::Finish => SanCaiStageSnap::Finish,
            },
            target: self.target,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            SanCaiStage::Draw => {
                ctx.start_draw(self.owner, 2);
                self.stage = SanCaiStage::Target;
                Flow::Yield
            }
            SanCaiStage::Target => {
                let others: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if others.is_empty() || ctx.core.player(self.owner).hand.is_empty() {
                    return Flow::Done;
                }
                self.stage = SanCaiStage::GiveCard;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "sancai_target",
                        options: others,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            SanCaiStage::GiveCard => {
                match ctx.answer_first() {
                    ChoiceOption::Player(p) => self.target = Some(p),
                    _ => return Flow::Done,
                }
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
                self.stage = SanCaiStage::Finish;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "sancai_card",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            SanCaiStage::Finish => {
                if let (Some(target), ChoiceOption::Card(c)) = (self.target, ctx.answer_first()) {
                    ctx.give_hand_card(self.owner, target, c);
                }
                Flow::Done
            }
        }
    }
}
