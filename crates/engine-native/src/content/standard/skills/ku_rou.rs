use super::*;

// 苦肉（黄盖）：出牌阶段主动技——失去 1 点体力，摸两张牌（每阶段限一次）。
#[derive(Clone)]
pub(in super::super) struct KuRouBehavior;

impl SkillBehavior for KuRouBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false // 主动技：不随事件触发
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(KuRouMachine {
            owner,
            stage: KuRouStage::Lose,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.dying {
            return None;
        }
        Some(Box::new(KuRouMachine {
            owner,
            stage: KuRouStage::Lose,
        }))
    }
}

#[derive(Clone)]
pub(in super::super) enum KuRouStage {
    /// 失去 1 点体力（含濒死内联检查）。
    Lose,
    /// 摸两张牌。
    Draw,
}

#[derive(Clone)]
pub(in super::super) struct KuRouMachine {
    pub(in super::super) owner: PlayerId,
    pub(in super::super) stage: KuRouStage,
}

impl SkillMachine for KuRouMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::KuRou {
            owner: self.owner,
            stage: match self.stage {
                KuRouStage::Lose => KuRouStageSnap::Lose,
                KuRouStage::Draw => KuRouStageSnap::Draw,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            KuRouStage::Lose => {
                let p = ctx.core.player_mut(self.owner);
                p.hp -= 1;
                let entering = p.hp <= 0 && !p.dying;
                if entering {
                    p.dying = true;
                    ctx.log(LogEvent::DyingEntered { player: self.owner });
                    ctx.emit(EventData::Dying {
                        player: self.owner,
                        source: None,
                    });
                }
                self.stage = KuRouStage::Draw;
                Flow::Yield
            }
            KuRouStage::Draw => {
                ctx.draw_cards(self.owner, 2);
                Flow::Done
            }
        }
    }
}
