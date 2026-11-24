//! Q4 (`wansha` / `rewansha` / `完杀`) rescue-blocking queries on [`Catalog`].

use super::Catalog;

impl Catalog {
    /// Whether `player` currently holds an active `wansha` / `rewansha` / `完杀` skill.
    pub fn has_active_wansha(
        &self,
        core: &crate::state::GameStateCore,
        player: crate::ids::PlayerId,
    ) -> bool {
        (player as usize) < core.players.len()
            && core.player(player).alive
            && ["wansha", "rewansha", "完杀"]
                .iter()
                .any(|n| self.skill_by_name(n).is_some_and(|def| core.has_skill(player, def)))
    }

    /// Upstream `wansha` / `rewansha` dying `Tao` suppression: during a holder's turn,
    /// other non-dying players cannot use or save with `Tao`.
    pub fn wansha_blocks_rescue(
        &self,
        core: &crate::state::GameStateCore,
        actor: crate::ids::PlayerId,
        dying: crate::ids::PlayerId,
    ) -> bool {
        let cur = core.turn.current;
        actor != dying
            && actor != cur
            && (cur as usize) < core.players.len()
            && self.has_active_wansha(core, cur)
    }
}
