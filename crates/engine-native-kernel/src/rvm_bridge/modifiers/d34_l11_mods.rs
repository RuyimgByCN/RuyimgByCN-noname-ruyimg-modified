//! Explicit per-skill `L11 太虚幻境` modifier evaluations (`d34_l11_mods.rs`), extracted from
//! `archive/upstream/noname-liuli/noname/extension/太虚幻境/{extension.js,extension_skill.js,extension_buff.js}`.

use crate::catalog::{CardKind, Catalog, EquipSlot};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

use super::flag_val;
use super::has_ext_skill;

fn has_armor(state: &GameStateCore, catalog: &Catalog, seat: PlayerId) -> bool {
    state
        .player(seat)
        .equips
        .iter()
        .any(|&cid| catalog.card_slot_of(state, cid) == Some(EquipSlot::Armor))
}

pub(super) fn apply_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut limit: i32,
) -> i32 {
    // extension.js:3441 (`xinmingjian2`: unlimited maxHandcard)
    if flag_val(state, player, "l11_mod:xinmingjian2") > 0
        || has_ext_skill(state, catalog, player, "xinmingjian2")
    {
        limit += 99;
    }
    // extension.js:4988 (`txkeji_add`: +4 maxHandcard)
    if flag_val(state, player, "l11_mod:txkeji_add") > 0
        || has_ext_skill(state, catalog, player, "txkeji_add")
    {
        limit += 4;
    }
    // extension.js:1393 (`txhj_renwang`: +expansion count)
    limit += flag_val(state, player, "l11_mod:txhj_renwang") as i32;
    // extension_buff.js:365 (`buff_txhj_wuweizhile`: target hand limit -1)
    limit -= flag_val(state, player, "l11_wuweizhile_minus1") as i32;
    limit
}

pub(super) fn apply_distance_total(
    state: &GameStateCore,
    catalog: &Catalog,
    from: PlayerId,
    to: PlayerId,
    mut dist: i32,
) -> i32 {
    // extension.js:3460 (`txmazhan`: globalFrom - 2, globalTo + 1)
    if flag_val(state, from, "l11_mod:txmazhan") > 0
        || has_ext_skill(state, catalog, from, "txmazhan")
    {
        dist -= 2;
    }
    if flag_val(state, to, "l11_mod:txmazhan") > 0
        || has_ext_skill(state, catalog, to, "txmazhan")
    {
        dist += 1;
    }
    dist
}

pub(super) fn apply_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
    mut unlimited: bool,
) -> (i32, bool) {
    // extension.js:3441 (`xinmingjian2`: unlimited Sha)
    if flag_val(state, player, "l11_mod:xinmingjian2") > 0
        || has_ext_skill(state, catalog, player, "xinmingjian2")
    {
        unlimited = true;
    }
    // extension.js:6078 (`txhj_oldhuxiao`) & extension_skill.js:1071 (`pyzhuren_diamond`)
    if flag_val(state, player, "l11_mod:txhj_oldhuxiao") > 0 {
        cur += 1;
    }
    if flag_val(state, player, "l11_mod:pyzhuren_diamond") > 0
        || has_ext_skill(state, catalog, player, "pyzhuren_diamond")
    {
        cur += 1;
    }
    (cur, unlimited)
}

pub(super) fn apply_card_distance_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    kind: CardKind,
    mut unlimited: bool,
) -> bool {
    // extension.js:4854 (`txshouyi`: targetInRange => true)
    if flag_val(state, player, "l11_mod:txshouyi") > 0
        || has_ext_skill(state, catalog, player, "txshouyi")
    {
        unlimited = true;
    }
    // extension_buff.js:376 (`buff_txhj_yuanduandilue`: Bingliang unlimited distance)
    if kind == CardKind::Bing
        && (flag_val(state, player, "l11_yuanduandilue_bing_no_dist") > 0
            || has_ext_skill(state, catalog, player, "buff_txhj_yuanduandilue")
            || has_ext_skill(state, catalog, player, "txhj_yuanduandilue"))
    {
        unlimited = true;
    }
    unlimited
}

pub(super) fn apply_card_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    kind: CardKind,
    mut canceled: bool,
) -> bool {
    let is_trick = !kind.is_basic_kind() && kind.equip_slot().is_none();
    // extension.js:3593 (`txshixin`: immune to others' tricks/delayed tricks)
    if actor != target
        && is_trick
        && (flag_val(state, target, "l11_mod:txshixin") > 0
            || has_ext_skill(state, catalog, target, "txshixin"))
    {
        canceled = true;
    }
    // extension.js:658 (`txhj_guimei`: immune to delayed tricks)
    if matches!(
        kind,
        CardKind::Le | CardKind::Bing | CardKind::ShanDian | CardKind::HongShui
    ) && (flag_val(state, target, "l11_mod:txhj_guimei") > 0
        || has_ext_skill(state, catalog, target, "txhj_guimei"))
    {
        canceled = true;
    }
    // extension.js:5303 (`zglkongcheng`: empty hand immune to Sha/Duel)
    if state.player(target).hand.is_empty()
        && (kind.is_sha_like() || kind == CardKind::Duel)
        && (flag_val(state, target, "l11_mod:zglkongcheng") > 0
            || has_ext_skill(state, catalog, target, "zglkongcheng"))
    {
        canceled = true;
    }
    // extension.js:5704 (`txmanzhen`: immune to SavageAttack)
    if kind == CardKind::SavageAttack
        && (flag_val(state, target, "l11_mod:txmanzhen") > 0
            || has_ext_skill(state, catalog, target, "txmanzhen"))
    {
        canceled = true;
    }
    canceled
}

pub(super) fn apply_card_use_canceled(
    state: &GameStateCore,
    actor: PlayerId,
    in_hand: bool,
    mut canceled: bool,
) -> bool {
    // extension.js:5131 (`mdqianxi72`: hand cards locked this turn)
    if in_hand && flag_val(state, actor, "l11_mod:mdqianxi72") > 0 {
        canceled = true;
    }
    canceled
}

pub(super) fn apply_damage_amount(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
    mut cur: i32,
) -> i32 {
    if cur <= 0 {
        return 0;
    }
    let ckind = card.map(|c| catalog.card_kind(c));
    let is_sha = ckind.is_some_and(CardKind::is_sha_like);
    let is_trick = ckind.is_some_and(|k| !k.is_basic_kind() && k.equip_slot().is_none());

    // extension.js:3723 (`txhuangfu`: immune to thunder damage)
    if nature == DamageNature::Thunder
        && (flag_val(state, target, "l11_mod:txhuangfu") > 0
            || has_ext_skill(state, catalog, target, "txhuangfu"))
    {
        return 0;
    }
    // extension.js:5851 (`txyongguan`: prevents damage when ward active)
    if flag_val(state, target, "l11_mod:txyongguan_ward") > 0 {
        return 0;
    }
    // extension.js:5657-5681 (`txmanjia` / `txmanjia1` / `txmanjia2`: unarmored Vine Armor)
    let has_txmanjia = flag_val(state, target, "l11_mod:txmanjia") > 0
        || has_ext_skill(state, catalog, target, "txmanjia")
        || has_ext_skill(state, catalog, target, "txmanjia1")
        || has_ext_skill(state, catalog, target, "txmanjia2");
    if has_txmanjia && !has_armor(state, catalog, target) {
        if matches!(ckind, Some(CardKind::SavageAttack | CardKind::ArcheryAttack))
            || (matches!(ckind, Some(CardKind::Sha)) && nature == DamageNature::Normal)
        {
            return 0;
        }
        if nature == DamageNature::Fire {
            cur += 1;
        }
    }

    if let Some(src) = source {
        // extension.js:554, 572, 699, 1344, 3355, 5809 & extension_skill.js:1071
        if flag_val(state, src, "l11_mod:damage_plus1") > 0
            || has_ext_skill(state, catalog, src, "txhj_shiyuan")
        {
            cur += 1;
        }
        if is_sha
            && (flag_val(state, src, "l11_mod:txfusha") > 0
                || has_ext_skill(state, catalog, src, "txfusha")
                || has_ext_skill(state, catalog, src, "txhj_zuijiu")
                || flag_val(state, src, "l11_mowang_jiu_persist") > 0)
        {
            cur += 1;
        }
        // extension_buff.js:321 (`buff_txhj_taipingzhoufa`: elemental Sha +1 damage)
        if is_sha
            && matches!(nature, DamageNature::Fire | DamageNature::Thunder)
            && (flag_val(state, src, "l11_taipingzhoufa") > 0
                || has_ext_skill(state, catalog, src, "buff_txhj_taipingzhoufa"))
        {
            cur += 1;
        }
        // extension_buff.js:673 (`buff_txhj_tianxiawushuang`: normal Sha +1 damage)
        if is_sha
            && nature == DamageNature::Normal
            && (flag_val(state, src, "l11_tianxiawushuang_normal_sha_plus1") > 0
                || has_ext_skill(state, catalog, src, "buff_txhj_tianxiawushuang"))
        {
            cur += 1;
        }
    }

    // extension.js:3760 (`txxiebi`: damage > 1 -> -1)
    if cur > 1
        && (flag_val(state, target, "l11_mod:txxiebi") > 0
            || has_ext_skill(state, catalog, target, "txxiebi"))
    {
        cur -= 1;
    }
    // extension_buff.js:288, 299, 651 (`xihuozhikai`, `dingleizhikai`, `cebeibuyu` with armor)
    if has_armor(state, catalog, target) {
        if nature == DamageNature::Fire
            && (flag_val(state, target, "l11_xihuozhikai") > 0
                || has_ext_skill(state, catalog, target, "buff_txhj_xihuozhikai"))
        {
            cur = (cur - 1).max(0);
        }
        if nature == DamageNature::Thunder
            && (flag_val(state, target, "l11_dingleizhikai") > 0
                || has_ext_skill(state, catalog, target, "buff_txhj_dingleizhikai"))
        {
            cur = (cur - 1).max(0);
        }
        if is_trick
            && (flag_val(state, target, "l11_cebeibuyu_trick_shield") > 0
                || has_ext_skill(state, catalog, target, "buff_txhj_cebeibuyu"))
        {
            cur = (cur - 1).max(0);
        }
    }
    cur
}
