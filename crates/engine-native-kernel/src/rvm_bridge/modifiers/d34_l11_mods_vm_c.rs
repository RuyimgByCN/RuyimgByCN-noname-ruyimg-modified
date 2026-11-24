//! Modifier & combat-rule helpers for Track `P3-E3` (`HostBehavior::L11VmFixC`).

use crate::catalog::{CardKind, Catalog};
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

use super::d34_flags::flag_val;
use super::d34_l11_mods_npc_b::turn_stamp;

/// Check whether `player` holds `skill_name` as an unsealed `HostBehavior::L11VmFixC` skill
/// (honoring `chanyuan`/`txhj_feilong_skill_blank`, plus `tieji`/`xj_fengyin`/`fangzhu`/`rewansha` when `!locked`).
pub fn has_unsealed_vm_c_skill(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    skill_name: &str,
    locked: bool,
) -> bool {
    let Some(def) = catalog.skill_by_name(skill_name) else {
        return false;
    };
    if catalog.skills[def as usize].behavior.as_host_behavior()
        != Some(crate::vocabulary::host::HostBehavior::L11VmFixC)
    {
        return false;
    }
    if !state.player(player).alive
        || !state.has_skill(player, def)
        || catalog.is_rechanyuan_sealed_def(state, player, def)
        || catalog
            .skill_by_name("txhj_feilong_skill_blank")
            .is_some_and(|b| state.has_skill(player, b))
    {
        return false;
    }
    if !locked
        && (catalog.is_tieji_sealed(state, player)
            || catalog.is_xj_fengyin_sealed(state, player)
            || crate::kernel::mbcmfangzhu_baiban(state, player))
    {
        return false;
    }
    true
}

/// Hand-limit modifiers for `L11VmFixC`:
/// - `juejing` (`extra.js:10746`, locked): `+2`
/// - `nzry_jieying` (`extra.js:11753`, locked global non-stacking): linked players `+2`
/// - `rekuangcai` (`huicui.js:8058`, `rekuangcai_change` cumulative delta `pos - neg`)
/// - `rejizhi` (`refresh.js:12293`, `+l11c_rejizhi_hand_bonus` until turn end)
pub(super) fn apply_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut limit: i32,
) -> i32 {
    if !state.player(player).alive {
        return limit;
    }
    if has_unsealed_vm_c_skill(state, catalog, player, "juejing", true) {
        limit += 2;
    }
    if state.player(player).linked
        && state
            .alive_players()
            .any(|p| has_unsealed_vm_c_skill(state, catalog, p, "nzry_jieying", true))
    {
        limit += 2;
    }
    let rk_pos = flag_val(state, player, "l11c_rekuangcai_pos") as i32;
    let rk_neg = flag_val(state, player, "l11c_rekuangcai_neg") as i32;
    if rk_pos != rk_neg {
        limit += rk_pos - rk_neg;
    }
    let tstamp = turn_stamp(state);
    if flag_val(state, player, "l11c_rejizhi_hand_turn") == tstamp {
        limit += flag_val(state, player, "l11c_rejizhi_hand_bonus") as i32;
    }
    limit
}

/// Distance modifiers for `L11VmFixC`:
/// - `zhuikong_distance` (`yijiang.js:11005`): during `a`'s turn if `a` has temp `zhuikong_distance` pointing at `b`, distance is `1`
pub(super) fn apply_distance_total(
    state: &GameStateCore,
    _catalog: &Catalog,
    a: PlayerId,
    b: PlayerId,
    mut cur: i32,
) -> i32 {
    let tstamp = turn_stamp(state);
    if flag_val(state, a, "l11c_zhuikong_dist_turn") == tstamp
        && flag_val(state, a, "l11c_zhuikong_dist_target") == u32::from(b) + 1
    {
        cur = 1;
    }
    cur
}

/// Play-phase `Sha` use-limit modifiers for `L11VmFixC`:
/// - `rekuangcai` (`huicui.js:8058`): unlimited during owner's turn
/// - `new_repaoxiao` (`refresh.js:11676`): unlimited
/// - `jintao` (`tw.js:13171`): `+1`
/// - `xiandeng` (`sp.js:18707`): `+1`
/// - `twjiange` (`tw.js:8859`) / `xinniluan`: exempted via `sha_no_use_count`
pub(super) fn apply_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
    mut cur: i32,
    mut unlimited: bool,
) -> (i32, bool) {
    if has_unsealed_vm_c_skill(state, catalog, player, "new_repaoxiao", true) {
        unlimited = true;
    }
    if state.turn.current == player
        && has_unsealed_vm_c_skill(state, catalog, player, "rekuangcai", true)
    {
        unlimited = true;
    }
    if has_unsealed_vm_c_skill(state, catalog, player, "jintao", true) {
        cur += 1;
    }
    if has_unsealed_vm_c_skill(state, catalog, player, "xiandeng", true) {
        cur += 1;
    }
    if sha_no_use_count(state, player, card, CardKind::Sha) {
        unlimited = true;
    }
    (cur, unlimited)
}

/// `CardUseUnlimited` modifiers for `L11VmFixC` (covers `Jiu` etc. under `rekuangcai`):
pub(super) fn apply_card_use_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    unlimited: bool,
) -> bool {
    if unlimited {
        return true;
    }
    if state.turn.current == player
        && has_unsealed_vm_c_skill(state, catalog, player, "rekuangcai", true)
    {
        return true;
    }
    sha_no_use_count(state, player, card, kind)
}

/// `Sha` / `Jiu` / general `ShaNoUseCount` exemption flags for `twjiange` and `xinniluan`.
pub fn sha_no_use_count(
    state: &GameStateCore,
    player: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
) -> bool {
    if !kind.is_sha_like() {
        return false;
    }
    flag_val(state, player, "l11c_twjiange_sha_active") > 0
        || card.is_some_and(|c| {
            flag_val(state, player, &format!("l11c_twjiange_card:{c}")) > 0
                || flag_val(state, player, &format!("l11c_xinniluan_card:{c}")) > 0
        })
}

/// Unlimited-distance modifiers for `L11VmFixC`:
/// - `rekuangcai`: during owner's turn, all cards ignore distance
/// - `jintao`: `Sha` always ignores distance
/// - `new_repaoxiao2` / `new_repaoxiao` after 1st Sha in Play: `Sha` ignores distance until turn end
/// - `xiandeng`: `Sha` ignores distance when `sha_used_count_turn == 0`
/// - `twjiange` / `xinniluan`: converted `Sha` ignores distance
pub(super) fn apply_card_distance_unlimited_to(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    _target: Option<PlayerId>,
    card: CardId,
    kind: CardKind,
    unlimited: bool,
) -> bool {
    if unlimited {
        return true;
    }
    if state.turn.current == player
        && has_unsealed_vm_c_skill(state, catalog, player, "rekuangcai", true)
    {
        return true;
    }
    if kind.is_sha_like() {
        let tstamp = turn_stamp(state);
        if has_unsealed_vm_c_skill(state, catalog, player, "jintao", true) {
            return true;
        }
        if has_unsealed_vm_c_skill(state, catalog, player, "new_repaoxiao2", true)
            || flag_val(state, player, "l11c_new_repaoxiao2_turn") == tstamp
        {
            return true;
        }
        if has_unsealed_vm_c_skill(state, catalog, player, "xiandeng", true)
            && flag_val(state, player, "l11c_turn_sha_used_stamp") != tstamp
        {
            return true;
        }
        if sha_no_use_count(state, player, Some(card), kind)
            || flag_val(state, player, "l11c_xinniluan_dist_unlimited") > 0
        {
            return true;
        }
    }
    false
}

/// `qiaoshui2` (`yijiang.js:10090`): losing Pindian forbids using any trick (`get.type(card,'trick')=='trick'`) this turn.
pub(super) fn apply_card_use_canceled(
    state: &GameStateCore,
    _catalog: &Catalog,
    actor: PlayerId,
    kind: CardKind,
    canceled: bool,
) -> bool {
    if canceled {
        return true;
    }
    let tstamp = turn_stamp(state);
    kind.is_trick_kind() && flag_val(state, actor, "l11c_qiaoshui2_turn") == tstamp
}

/// `drlt_qianjie` (`shenhua.js:1012`): immune to gaining link (`!player.isLinked()` -> `linkBegin` cancel).
pub fn drlt_qianjie_blocks_link(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    !state.player(player).linked && has_unsealed_vm_c_skill(state, catalog, player, "drlt_qianjie", true)
}

/// `drlt_qianjie` (`shenhua.js:1043`): cannot be targeted by `canCompare` Pindian.
pub fn can_compare_target(state: &GameStateCore, catalog: &Catalog, target: PlayerId) -> bool {
    !has_unsealed_vm_c_skill(state, catalog, target, "drlt_qianjie", true)
}

/// `nzry_jieying` (`extra.js:11753`): locked-linked (`linkBefore` cancel when currently linked).
pub fn nzry_jieying_blocks_unlink(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    state.player(player).linked && has_unsealed_vm_c_skill(state, catalog, player, "nzry_jieying", true)
}

/// `xswuyan` (`yijiang.js:11724`): ordinary tricks (`get.type(card) == 'trick'`) between `actor != target`
/// are canceled in `useCardToBefore` when either holds unsealed `xswuyan`.
pub fn xswuyan_excludes_target(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    kind: CardKind,
) -> bool {
    actor != target
        && kind.is_ordinary_trick_kind()
        && (has_unsealed_vm_c_skill(state, catalog, actor, "xswuyan", true)
            || has_unsealed_vm_c_skill(state, catalog, target, "xswuyan", true))
}

/// `nzry_cunmu` (`shenhua.js:2307`): locked — ALL `Draw` machines draw from the bottom of deck.
pub fn nzry_cunmu_draws_from_bottom(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    has_unsealed_vm_c_skill(state, catalog, player, "nzry_cunmu", true)
}
