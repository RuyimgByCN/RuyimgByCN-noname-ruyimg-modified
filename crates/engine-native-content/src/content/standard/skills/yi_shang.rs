use super::*;

pub(in super::super) struct YiShangBehavior;

impl SkillBehavior for YiShangBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(YiShangMachine {
            owner,
            stage: YiShangStage::Target,
        })
    }
}

#[derive(Clone)]
pub(in super::super) enum YiShangStage {
    Target,
    Apply,
}

#[derive(Clone)]
pub(in super::super) struct YiShangMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: YiShangStage,
}

impl SkillMachine for YiShangMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::YiShang {
            owner: self.owner,
            stage: match self.stage {
                YiShangStage::Target => YiShangStageSnap::Target,
                YiShangStage::Apply => YiShangStageSnap::Apply,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            YiShangStage::Target => {
                let others: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if others.is_empty() {
                    return Flow::Done;
                }
                self.stage = YiShangStage::Apply;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "yishang_target",
                        options: others,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            YiShangStage::Apply => {
                if let ChoiceOption::Player(p) = ctx.answer_first() {
                    let d = ctx.current_damage_mut();
                    d.redirect_chain.push(d.target);
                    d.target = p;
                }
                Flow::Done
            }
        }
    }
}
