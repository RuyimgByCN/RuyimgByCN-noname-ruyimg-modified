//! U05-7d 杀海拾遗 swd 补充技能触发过滤表（`filtering/swd_u05_7d.rs`）。

use super::super::*;

impl HostSkillBehavior {
    pub(super) fn filter_swd_u05_7d(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
    ) -> bool {
        match self.behavior {
            HostBehavior::SwdDuijueOut => matches!(
                event,
                EventData::Death { player, .. }
                if ctx
                    .state
                    .vm_aux
                    .flags
                    .get(&(0, "duijue_duelist_a".to_string()))
                    .copied()
                    .and_then(|v| v.checked_sub(1))
                    == Some(u32::from(*player))
                    || ctx
                        .state
                        .vm_aux
                        .flags
                        .get(&(0, "duijue_duelist_b".to_string()))
                        .copied()
                        .and_then(|v| v.checked_sub(1))
                        == Some(u32::from(*player))
            ),
            HostBehavior::SwdDuijueTrigger => matches!(
                event,
                EventData::TurnEndWindow { player, after: true } if *player == owner
            ),
            HostBehavior::SwdTanlinTarget | HostBehavior::SwdTanlinOwner => matches!(
                event,
                EventData::TurnEndWindow { after: true, .. }
            ),
            HostBehavior::SwdTanlinDefence => matches!(
                event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && !ctx.state.player(owner).hand.is_empty()
                    && d.source.is_some_and(|s| {
                        ctx.state.player(s).alive && !ctx.state.player(s).hand.is_empty()
                    })
            ),
            HostBehavior::SwdHuanling2 => matches!(
                event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && ctx.state.player(owner).face_down
            ),
            HostBehavior::SwdLexue2 => matches!(
                event,
                EventData::TurnEndWindow { player, after: false } if *player == owner
            ),
            HostBehavior::SwdTianshuRemove => matches!(
                event,
                EventData::Death { player, .. }
                if ctx.state.player(owner).swdtianshu_from == Some(*player)
            ),
            HostBehavior::SwdMoyu => matches!(
                event,
                EventData::Death { killer, .. }
                if *killer == Some(owner)
                    && ctx.state.player(owner).alive
                    && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
            ),
            _ => false,
        }
    }
}
