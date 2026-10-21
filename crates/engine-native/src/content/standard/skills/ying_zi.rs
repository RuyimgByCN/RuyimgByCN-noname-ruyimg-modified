use super::*;

// 英姿（周瑜）：摸牌阶段摸牌时额外摸一张牌。
#[derive(Clone)]
pub(in super::super) struct YingZiBehavior;

impl SkillBehavior for YingZiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::Draw,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(YingZiMachine { owner })
    }
}

#[derive(Clone)]
pub(in super::super) struct YingZiMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for YingZiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::YingZi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.draw_cards(self.owner, 1);
        Flow::Done
    }
}
