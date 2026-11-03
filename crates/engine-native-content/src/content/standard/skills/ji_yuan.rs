use super::*;

// 急援（伊籍，P2/N08 标准缺失将第四批 2026-10-07）：一名角色进入濒死
// 状态时，可令其摸一张牌（登记差异：仅濒死臂）。
#[derive(Clone)]
pub(in super::super) struct JiYuanBehavior;

impl SkillBehavior for JiYuanBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Dying { .. }) && ctx.state.player(owner).alive
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JiYuanMachine { owner })
    }
}

#[derive(Clone)]
pub(in super::super) struct JiYuanMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for JiYuanMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JiYuan { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 濒死者为事件目标（任一角色濒死时其摸 1）。
        if let EventData::Dying { player, .. } = ctx.res.stack.last().expect("dying frame").data {
            if ctx.core.player(player).dying {
                ctx.start_draw(player, 1);
            }
        }
        Flow::Done
    }
}
