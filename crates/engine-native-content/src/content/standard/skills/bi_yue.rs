use super::*;

// 闭月（貂蝉）：结束阶段开始时摸一张牌。
#[derive(Clone)]
pub(in super::super) struct BiYueBehavior;

impl SkillBehavior for BiYueBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::End,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(BiYueMachine { owner })
    }
}

#[derive(Clone)]
pub(in super::super) struct BiYueMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for BiYueMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BiYue { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.start_draw(self.owner, 1);
        Flow::Done
    }
}
