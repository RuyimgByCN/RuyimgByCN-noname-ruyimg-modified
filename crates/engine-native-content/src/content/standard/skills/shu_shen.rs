use super::*;

// 淑慎（甘夫人，P2/N08 标准缺失将第二批 2026-10-07）：回复体力后，
// 可令一名其他角色摸牌——有手牌摸 1，无手牌摸 2。
#[derive(Clone)]
pub(in super::super) struct ShuShenBehavior;

impl SkillBehavior for ShuShenBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Recover { player, amount, .. }
            if *player == owner && *amount > 0 && ctx.state.player(owner).alive)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ShuShenMachine {
            owner,
            target: None,
            stage: ShuShenStage::Ask,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum ShuShenStage {
    /// 目标选择（挂起 = shushen_target）。
    Ask,
    /// 恢复读应答：目标摸 1（有手牌）或 2（无手牌）。
    Read,
}

#[derive(Clone)]
pub(in super::super) struct ShuShenMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) target: Option<PlayerId>,
    pub(in super::super) stage: ShuShenStage,
}

impl SkillMachine for ShuShenMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::ShuShen {
            owner: self.owner,
            target: self.target,
            stage: match self.stage {
                ShuShenStage::Ask => ShuShenStageSnap::Ask,
                ShuShenStage::Read => ShuShenStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            ShuShenStage::Ask => {
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = ShuShenStage::Read;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectPlayers,
                        prompt: "shushen_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            ShuShenStage::Read => {
                if let ChoiceOption::Player(target) = ctx.answer_first() {
                    self.target = Some(target);
                    if ctx.core.player(target).alive {
                        let n = if ctx.core.player(target).hand.is_empty() {
                            2
                        } else {
                            1
                        };
                        ctx.start_draw(target, n as u32);
                    }
                }
                Flow::Done
            }
        }
    }
}
