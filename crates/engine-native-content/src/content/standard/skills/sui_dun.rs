use super::*;

// 碎盾：每回合限一次，当你即将受到伤害时，防止之。
// ---------------------------------------------------------------------------

pub(in super::super) struct SuiDunBehavior;

impl SkillBehavior for SuiDunBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented)
    }

    fn start(&self, _owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(SuiDunMachine)
    }
}

#[derive(Clone)]
pub(in super::super) struct SuiDunMachine;

impl SkillMachine for SuiDunMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::SuiDun
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.current_damage_mut().prevented = true;
        Flow::Done
    }
}
