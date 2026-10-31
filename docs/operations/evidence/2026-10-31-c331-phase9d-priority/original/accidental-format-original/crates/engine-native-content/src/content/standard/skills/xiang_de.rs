use super::*;

/// 相德（Boss 混沌，规则 v210）：DamageBefore 一步直改——来源装备区
/// 有武器且非持有者本人时伤害 +1。
pub(in super::super) struct XiangDeBehavior;

impl SkillBehavior for XiangDeBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner
                && !d.prevented
                && d.source.is_some()
                && d.source != Some(owner)
                && ctx
                    .state
                    .player(d.source.expect("checked"))
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.cards[*c as usize].kind == crate::catalog::CardKind::Weapon))
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(XiangDeMachine { owner })
    }
}

/// 相德机器：无挂起，启动即直改当前伤害帧。
#[derive(Clone)]
pub(in super::super) struct XiangDeMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for XiangDeMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossXiangde { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let d = ctx.current_damage_mut();
        if let Some(amount) = d.amount.checked_add(1) {
            d.amount = amount;
        } else {
            ctx.fail_damage_amount();
            return Flow::Done;
        }
        Flow::Done
    }
}
