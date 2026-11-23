use super::*;

impl HostSkillBehavior {
    pub(super) fn filter_u05_gujian(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
    ) -> bool {
        super::step::u05_b2_gujian::filter_u05_gujian(self, ctx, owner, event)
    }
}
