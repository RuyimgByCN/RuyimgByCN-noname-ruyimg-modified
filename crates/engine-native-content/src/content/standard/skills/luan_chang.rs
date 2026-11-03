use super::*;

/// 乱常（Boss 梼杌，规则 v218；A2 S13）：自己准备阶段开始（上游子技
/// player phaseZhunbeiBegin；原锚判定阶段开始）视为使用【南蛮入侵】
/// （虚拟牌，目标=全部其他存活角色）；结束阶段视为用【万箭齐发】
/// （kernel Phase::End 臂查询式）。触发计划层过滤 disabled，
/// 激活门控=SkillInstance.enabled 翻转。
pub(in super::super) struct LuanChangBehavior;

impl SkillBehavior for LuanChangBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Prepare, begin: true } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LuanChangMachine { owner })
    }
}

/// 乱常机器：无挂起，启动即虚拟南蛮（use_virtual_card 先例：虎煞
/// 虚拟南蛮同款，目标=全部其他存活角色）。
#[derive(Clone)]
pub(in super::super) struct LuanChangMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for LuanChangMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossLuanchang { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let targets: Vec<PlayerId> = ctx
            .core
            .seat_order_from(self.owner)
            .into_iter()
            .filter(|p| *p != self.owner)
            .collect();
        ctx.use_virtual_card(self.owner, CardKind::SavageAttack, targets);
        Flow::Done
    }
}
