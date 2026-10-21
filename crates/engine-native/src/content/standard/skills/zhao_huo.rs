use super::*;

// ---------------------------------------------------------------------------
// 移伤：当你即将受到伤害时，你可以将其转移给另一名存活玩家。
// ---------------------------------------------------------------------------

/// 招祸（Boss 毕方，规则 v227）：DamageBefore 一步直改——持有者造成的
/// 伤害视为火属性（nature=Fire）。
pub(in super::super) struct ZhaoHuoBehavior;

impl SkillBehavior for ZhaoHuoBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.source == Some(owner) && d.target != owner && !d.prevented)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ZhaoHuoMachine { owner })
    }
}

/// 招祸机器：无挂起，启动即改当前伤害帧 nature。
#[derive(Clone)]
pub(in super::super) struct ZhaoHuoMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for ZhaoHuoMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossZhaohuo { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let d = ctx.current_damage_mut();
        d.nature = crate::event::DamageNature::Fire;
        Flow::Done
    }
}
