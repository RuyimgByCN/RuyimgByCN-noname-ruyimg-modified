//! Track `P3-E2`（`HostBehavior::L11VmFixB`）静态修饰规则与限次/距离/目标判定
//! （`jiangchi2`、`rejiangchi3`、`reweimu`、`rexianzhen2`/`3`/`4`、`sbyangwei_effect`、
//! `tianyi2`/`3`、`xinliegong`）。

use crate::catalog::{CardKind, Catalog, Suit};
use crate::ids::{CardId, PlayerId};
use crate::rvm_bridge::modifiers::eval_ext::{bound_target_seat, card_effective_rank};
use crate::state::GameStateCore;
use super::{flag_val, has_ext_skill};

#[inline]
fn has_vm_b_skill(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    name: &str,
    locked: bool,
) -> bool {
    if !has_ext_skill(state, catalog, player, name) {
        return false;
    }
    if !locked
        && (catalog.is_tieji_sealed(state, player) || catalog.is_xj_fengyin_sealed(state, player))
    {
        return false;
    }
    true
}

/// `rexianzhen4`（`refresh.js:9624`）：【杀】不计入手牌上限且弃牌阶段不能弃置【杀】。
pub(super) fn is_card_ignored_in_discard_phase(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) -> bool {
    catalog.card_kind(card).is_sha_like() && has_vm_b_skill(state, catalog, player, "rexianzhen4", false)
}

/// `jiangchi2`（+1 杀）、`sbyangwei_effect`（+1 杀）、`tianyi2`（+1 杀）。
pub(super) fn apply_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut cur: i32,
    unlimited: bool,
) -> (i32, bool) {
    if has_vm_b_skill(state, catalog, player, "jiangchi2", true) {
        cur += 1;
    }
    if has_vm_b_skill(state, catalog, player, "sbyangwei_effect", true) {
        cur += 1;
    }
    if has_vm_b_skill(state, catalog, player, "tianyi2", true) {
        cur += 1;
    }
    (cur, unlimited)
}

/// `tianyi2`（`shenhua.js:6030`）：使用【杀】额外指定 1 个目标（仅非无限目标时 +1）。
pub(super) fn apply_sha_extra_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    kind: CardKind,
    mut extra: i32,
) -> i32 {
    if kind.is_sha_like() && has_vm_b_skill(state, catalog, player, "tianyi2", true) {
        extra += 1;
    }
    extra
}

/// `jiangchi2` / `sbyangwei_effect` / `tianyi2`（【杀】无距离限制）、`rexianzhen2`（对绑定目标牌无距离限制）、
/// `xinliegong`（`distance(player, target) <= rank(sha)`）。
pub(super) fn apply_card_distance_unlimited_to(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    target: Option<PlayerId>,
    card: CardId,
    kind: CardKind,
    mut unlimited: bool,
) -> bool {
    if kind.is_sha_like()
        && (has_vm_b_skill(state, catalog, player, "jiangchi2", true)
            || has_vm_b_skill(state, catalog, player, "sbyangwei_effect", true)
            || has_vm_b_skill(state, catalog, player, "tianyi2", true))
    {
        unlimited = true;
    }
    if target.is_some()
        && target == bound_target_seat(state, player)
        && has_vm_b_skill(state, catalog, player, "rexianzhen2", true)
    {
        unlimited = true;
    }
    if kind.is_sha_like()
        && card != CardId::MAX
        && has_vm_b_skill(state, catalog, player, "xinliegong", false)
    {
        let rank = card_effective_rank(state, catalog, card);
        if rank > 0
            && target.is_some_and(|t| state.computed_distance(catalog, player, t) <= u32::from(rank))
        {
            unlimited = true;
        }
    }
    unlimited
}

/// `rexianzhen2`：对绑定目标使用【杀】无次数限制。
pub(in crate::rvm_bridge::modifiers) fn eval_sha_unlimited_for_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    targets: &[PlayerId],
) -> bool {
    if targets.is_empty() {
        return false;
    }
    let Some(bound) = bound_target_seat(state, player) else {
        return false;
    };
    targets.iter().all(|&t| t == bound)
        && (has_vm_b_skill(state, catalog, player, "rexianzhen2", true)
            || flag_val(state, player, "l11_vm_b:rexianzhen2") > 0)
}

/// `reweimu`（`refresh.js:5136`）：黑色锦囊牌不能指定持有者为目标。
pub(super) fn apply_card_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    mut canceled: bool,
) -> bool {
    if kind.is_trick_kind()
        && card.is_some_and(|c| {
            matches!(
                catalog.effective_card_suit(state, actor, c),
                Suit::Spade | Suit::Club
            )
        })
        && has_vm_b_skill(state, catalog, target, "reweimu", true)
    {
        canceled = true;
    }
    canceled
}

/// `rejiangchi3` / `rexianzhen3` / `tianyi3`：不能使用【杀】。
pub(super) fn apply_card_use_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    kind: CardKind,
    mut canceled: bool,
) -> bool {
    if kind.is_sha_like()
        && (has_vm_b_skill(state, catalog, actor, "rejiangchi3", true)
            || has_vm_b_skill(state, catalog, actor, "rexianzhen3", true)
            || has_vm_b_skill(state, catalog, actor, "tianyi3", true))
    {
        canceled = true;
    }
    canceled
}

/// `rexianzhen2`（对绑定目标无视防具）、`sbyangwei_effect`（攻击方无视目标防具）。
pub(super) fn apply_ignore_armor(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    mut ignore: bool,
) -> bool {
    if (Some(target) == bound_target_seat(state, source)
        && has_vm_b_skill(state, catalog, source, "rexianzhen2", true))
        || has_vm_b_skill(state, catalog, source, "sbyangwei_effect", true)
    {
        ignore = true;
    }
    ignore
}
