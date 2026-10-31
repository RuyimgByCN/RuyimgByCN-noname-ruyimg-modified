//! Genuine existing native declarations use the ordinary source-window competition.
use super::*;
use crate::event::NativeDamageEffect;
pub(super) struct NativeDamageBehavior {
    pub name: &'static str,
    pub effect: NativeDamageEffect,
}
impl SkillBehavior for NativeDamageBehavior {
    fn native_damage_effect(&self) -> Option<NativeDamageEffect> {
        Some(self.effect)
    }
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event,EventData::Damage(d)if self.effect.matches(owner,d,ctx.catalog))
    }
    fn start(&self, _owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        NoopBehavior.start(_owner, _event)
    }
    fn start_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        _event: &EventData,
        kind: EventKind,
    ) -> Box<dyn SkillMachine> {
        assert_eq!(kind, self.effect.window().event_kind());
        Box::new(crate::kernel::NativeDamageSkill {
            owner,
            def: ctx
                .catalog
                .skill_by_name(self.name)
                .expect("immutable declaration"),
            window: self.effect.window(),
        })
    }
}
