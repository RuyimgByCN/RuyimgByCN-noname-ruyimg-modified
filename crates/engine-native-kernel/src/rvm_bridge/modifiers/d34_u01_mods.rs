//! U01 `3D精选` 手牌明置与手牌封锁判定（自 `d34_flags.rs` 拆出）。

use super::{flag_val, has_ext_skill};
use crate::catalog::{Catalog, Faction};
use crate::ids::{CardId, PlayerId};
use crate::state::{GameMode, GameStateCore, Role};

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

pub fn is_u01_real_lord(state: &GameStateCore, p: PlayerId) -> bool {
    match &state.mode {
        GameMode::Identity { roles } => roles.get(p as usize) == Some(&Role::Lord),
        _ => flag_val(state, p, "is_zhu") == 1,
    }
}

pub fn u01_xiaoxing_bonus(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> i32 {
    if has_ext_skill(state, catalog, player, "dddxiaoxing")
        && !catalog.is_tieji_sealed(state, player)
    {
        3
    } else {
        0
    }
}

pub fn u01_fuyi_sha_bonus(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> i32 {
    if state.effective_faction(catalog, player) != Some(Faction::Qun) {
        return 0;
    }
    let lords = state
        .alive_players()
        .filter(|&lord| {
            is_u01_real_lord(state, lord) && has_ext_skill(state, catalog, lord, "dddfuyi")
        })
        .count();
    lords as i32
}

pub fn is_dddlianer_forbidden(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    card: CardId,
) -> bool {
    if card == 0 || (card as usize) >= catalog.cards.len() {
        return false;
    }
    let cap = flag_val(state, actor, "dddlianer_rank_cap");
    if cap == 0 {
        return false;
    }
    let rank = u32::from(catalog.cards[card as usize].rank);
    rank == 0 || rank >= cap
}

pub fn has_ddddanggu_negative(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    state.player(player).linked
        && state.alive_players().any(|owner| {
            owner != player
                && has_ext_skill(state, catalog, owner, "ddddanggu")
                && !catalog.is_tieji_sealed(state, owner)
        })
}
