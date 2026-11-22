//! U01 `3D精选` 手牌明置与手牌封锁判定（自 `d34_flags.rs` 拆出）。

use super::flag_val;
use crate::catalog::Catalog;
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

pub fn is_u01_shown_card(state: &GameStateCore, player: PlayerId, card: CardId) -> bool {
    flag_val(state, player, &format!("u01_shown_card:{card}")) > 0
        || flag_val(state, player, &format!("dddshilie_shown_card:{card}")) > 0
}

pub fn is_dddtongyu_hand_locked(state: &GameStateCore, player: PlayerId) -> bool {
    let cur = super::super::d34_l28_mods::l28_turn_stamp(state);
    cur != 0 && flag_val(state, player, "dddtongyu_hand_locked_stamp") == cur
}

pub fn is_dddfusi_hand_locked(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    let cur = state.turn.current;
    cur != player
        && state.player(cur).alive
        && catalog
            .skill_by_name("dddfusi")
            .is_some_and(|def| state.has_skill(cur, def))
        && !state.player(cur).hand.is_empty()
        && state
            .player(cur)
            .hand
            .iter()
            .all(|c| is_u01_shown_card(state, cur, *c))
}

pub fn is_u01_hand_locked(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    is_dddtongyu_hand_locked(state, player) || is_dddfusi_hand_locked(state, catalog, player)
}
