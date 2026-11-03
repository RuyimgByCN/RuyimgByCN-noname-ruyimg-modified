use super::*;

// 枭姬（孙尚香）：失去装备区里的一张牌（替换/弃置）时摸两张牌。
#[derive(Clone)]
pub(in super::super) struct XiaoJiBehavior;

impl SkillBehavior for XiaoJiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Equipped {
            player,
            replaced: Some(_),
            ..
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(XiaoJiMachine { owner })
    }
}

#[derive(Clone)]
pub(in super::super) struct XiaoJiMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for XiaoJiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::XiaoJi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.start_draw(self.owner, 2);
        Flow::Done
    }
}
