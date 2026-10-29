use super::*;

// ---------------------------------------------------------------------------
// 授艺：当你受到伤害后，你获得「移伤」直到回合结束。
// ---------------------------------------------------------------------------

pub(in super::super) struct ShouYiBehavior;

impl SkillBehavior for ShouYiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ShouYiMachine { owner })
    }
}

#[derive(Clone)]
pub(in super::super) struct ShouYiMachine {
    pub(in super::super) owner: PlayerId,
}

impl SkillMachine for ShouYiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::ShouYi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let yishang = ctx
            .catalog
            .skill_by_name(YI_SHANG)
            .expect("移伤 in catalog");
        ctx.add_skill_until_turn_end(self.owner, yishang);
        Flow::Done
    }
}
