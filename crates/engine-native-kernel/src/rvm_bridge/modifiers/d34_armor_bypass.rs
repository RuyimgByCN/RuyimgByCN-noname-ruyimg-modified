//! Armor bypass (`unequip2` / `ignore_armor`) rules shared across Decision 34 packs.

use super::{d34_l11_mods, d34_u05_mods, flag_val, has_ext_skill, has_weapon};
use crate::catalog::Catalog;
use crate::event::Phase;
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

pub fn is_target_unequip2(state: &GameStateCore, target: PlayerId) -> bool {
    flag_val(state, target, "l11_xingtian_armor_null") > 0
        || flag_val(state, target, "l11_noda_axe2_turn") > 0
        || flag_val(state, target, "l25_yijue_locked") > 0
        || flag_val(state, target, "l25_pojia_broken") > 0
        || flag_val(state, target, "l25_hubu_unequip2") > 0
        || flag_val(state, target, "chixueqingfeng2_count") > 0
}

pub fn apply_ignore_armor(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    ignore: bool,
) -> bool {
    apply_ignore_armor_with_card(state, catalog, source, target, None, ignore)
}

pub fn apply_ignore_armor_with_card(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    mut ignore: bool,
) -> bool {
    if is_target_unequip2(state, target)
        || has_ext_skill(state, catalog, source, "ssr_pojia")
        || has_ext_skill(state, catalog, source, "ssr_hubu")
        || flag_val(state, source, "l25_pojia_ignore_armor") > 0
        || (has_ext_skill(state, catalog, source, "fz_gzpaoxiao")
            && (has_ext_skill(state, catalog, source, "shouyue")
                || flag_val(state, source, "shouyue_active") > 0))
        || flag_val(state, source, "l27_twxiafeng_spent") > 0
        || flag_val(state, source, "u05_swd_jianji_unequip") > 0
    {
        ignore = true;
    }
    if state.turn.current == source
        && state.turn.phase == Phase::Play
        && !state.is_friend_of(catalog, source, target)
        && (has_weapon(state, catalog, source)
            || d34_l11_mods::has_equipped_named(state, catalog, source, "txhj_feilong"))
        && (flag_val(state, source, "l11_weapon_pozhenzhifeng") > 0
            || has_ext_skill(state, catalog, source, "txhj_pozhenzhifeng")
            || has_ext_skill(state, catalog, source, "buff_txhj_pozhenzhifeng")
            || has_ext_skill(state, catalog, source, "_buff_txhj_pozhenzhifeng"))
    {
        ignore = true;
    }
    let ignore = d34_u05_mods::apply_ignore_armor(state, source, ignore);
    let ignore = d34_l11_mods::apply_ignore_armor(state, catalog, source, target, card, ignore);
    super::super::d34_l28_mods::apply_ignore_armor(state, catalog, target, ignore)
}
