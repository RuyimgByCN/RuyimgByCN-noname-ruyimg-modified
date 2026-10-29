use super::*;

/// 覆泰（Boss 穷奇，规则 v212）：准备阶段（判定阶段开始锚点，神智同
/// 口径）——所有受伤角色按座位序各回复 1 点体力。
pub(in super::super) struct FutaiBehavior;

impl SkillBehavior for FutaiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Judge, begin: true } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(FutaiMachine { owner })
    }
}

/// 覆泰机器：无挂起，启动即按座位序回血。
#[derive(Clone)]
pub(in super::super) struct FutaiMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for FutaiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossFutai { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        for p in ctx.core.seat_order_from(self.owner) {
            let pl = ctx.core.player(p);
            if pl.alive && pl.hp < pl.max_hp as i32 {
                ctx.recover(p, 1, None);
            }
        }
        Flow::Done
    }
}
