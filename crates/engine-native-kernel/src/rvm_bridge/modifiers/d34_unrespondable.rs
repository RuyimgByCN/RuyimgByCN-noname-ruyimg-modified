//! Decision 34 不可响应（`cardUnrespondable`）扩展修饰器求值（自 `d34_flags` 拆出以保持单文件有效行数上限）。

use super::{
    card_suit, d34_l11_mods, d34_u05_mods, flag_val, has_ext_skill, has_slot_equipped, has_weapon,
};
use crate::catalog::{CardKind, Catalog, EquipSlot, Suit};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

pub fn apply_card_unrespondable(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
    mut unrespondable: bool,
) -> bool {
    if kind.is_sha_like() {
        if flag_val(state, source, "l11_taiji_unrespondable") > 0
            || flag_val(state, source, "l11_lulu_huwei_boost") > 0
            || has_ext_skill(state, catalog, source, "ssr_wushuang")
            || flag_val(state, source, "l25_wushuang_unrespondable") > 0
            || flag_val(state, source, "u05_cross_mod:wushuang") > 0
            || flag_val(state, source, "yxs_nvquan_double_response") > 0
            || flag_val(state, source, "l20_liegong_direct_hit") > 0
            || flag_val(state, source, "l27_potzhuangshi_unblockable") > 0
            || flag_val(state, source, "l27_twxiafeng_spent") > 0
        {
            unrespondable = true;
        }
        if has_weapon(state, catalog, source)
            && (flag_val(state, source, "l11_weapon_dunjiazhoufa") > 0
                || has_ext_skill(state, catalog, source, "buff_txhj_dunjiazhoufa"))
        {
            unrespondable = true;
        }
        if flag_val(state, source, "yxs_hanbei_unblockable") > 0
            || (has_ext_skill(state, catalog, source, "hanbei")
                && has_slot_equipped(state, catalog, source, |s| {
                    matches!(s, EquipSlot::HorsePlus | EquipSlot::HorseMinus)
                }))
        {
            unrespondable = true;
        }
        if (has_ext_skill(state, catalog, source, "sheshu")
            || flag_val(state, source, "yxs_sheshu_pierce") > 0)
            && state.player(target).hp >= 3
        {
            unrespondable = true;
        }
        if matches!(
            card_suit(catalog, Some(card)),
            Some(Suit::Heart | Suit::Diamond)
        ) && flag_val(state, source, "l27_sbzhaxiang_red_sha") > 0
        {
            unrespondable = true;
        }
    }
    if kind == CardKind::Duel
        && (has_ext_skill(state, catalog, source, "ssr_wushuang")
            || flag_val(state, source, "u05_cross_mod:wushuang") > 0
            || flag_val(state, source, "yxs_nvquan_double_response") > 0)
    {
        unrespondable = true;
    }
    if !kind.is_basic_kind()
        && !kind.is_equip()
        && (flag_val(state, source, "l11_buff_trick_unwuxie") > 0
            || has_ext_skill(state, catalog, source, "buff_txhj_wenhezhenlue"))
    {
        unrespondable = true;
    }
    let unrespondable =
        d34_u05_mods::apply_card_unrespondable(state, source, target, kind, unrespondable);
    d34_l11_mods::apply_card_unrespondable(state, catalog, source, kind, unrespondable)
}
