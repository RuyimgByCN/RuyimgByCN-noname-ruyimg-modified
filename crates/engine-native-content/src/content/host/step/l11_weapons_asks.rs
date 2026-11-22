//! L11（太虚幻境）Batch L11-5 武器覆写技询问构建（`l11_weapons_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceSpec;
use crate::ids::{CardId, PlayerId};
use crate::rvm_bridge::modifiers::{has_fuchendangmo, has_shangjiangdelvli};
use crate::state::GameStateCore;
use crate::snap::ExpectedAsk;

pub(super) const ASK_HANBING_CONFIRM: u32 = 1;
pub(super) const ASK_HANBING_PICK: u32 = 2;
pub(super) const ASK_CIXIONG_CONFIRM: u32 = 3;
pub(super) const ASK_CIXIONG_TARGET_DISCARD: u32 = 4;
pub(super) const ASK_GUANSHI_DISCARD: u32 = 5;
pub(super) const ASK_SANJIAN_DISCARD: u32 = 6;
pub(super) const ASK_SANJIAN_TARGET: u32 = 7;
pub(super) const ASK_YITIANJIAN_DISCARD: u32 = 8;
pub(super) const ASK_PYZHUREN_DISCARD: u32 = 9;

pub(super) fn is_equipped_weapon_named(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
    c: CardId,
    name: &str,
    kind: Option<CardKind>,
) -> bool {
    core.player(p).equips.contains(&c)
        && (catalog.cards.get(c as usize).is_some_and(|cd| cd.name == name)
            || kind.is_some_and(|k| catalog.card_kind(c) == k))
}

pub(super) fn guanshi_discard_pool(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| !is_equipped_weapon_named(core, catalog, owner, c, "guanshi", Some(CardKind::Guanshi)))
        .collect()
}

pub(super) fn pyzhuren_discard_pool(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| !is_equipped_weapon_named(core, catalog, owner, c, "pyzhuren_diamond", None))
        .collect()
}

pub(super) fn sanjian_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    damaged: PlayerId,
) -> Vec<PlayerId> {
    alive_all(core, owner)
        .into_iter()
        .filter(|&p| p != damaged && core.computed_distance(catalog, damaged, p) <= 1)
        .collect()
}

pub(super) fn build_l11_weapons_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        ASK_HANBING_CONFIRM => {
            let t = target.filter(|&p| core.player(p).alive && !he_cards(core, p).is_empty())?;
            let _ = t;
            confirm_ask(owner, "hanbing_skill:confirm")
        }
        ASK_HANBING_PICK => {
            let t = target.filter(|&p| core.player(p).alive)?;
            let pool = he_cards(core, t);
            if pool.is_empty() {
                return None;
            }
            card_ask(owner, "hanbing_skill:pick_card", pool, 1, 1, false)
        }
        ASK_CIXIONG_CONFIRM => {
            let _t = target.filter(|&p| core.player(p).alive)?;
            confirm_ask(owner, "cixiong_skill:confirm")
        }
        ASK_CIXIONG_TARGET_DISCARD => {
            let t = target.filter(|&p| core.player(p).alive)?;
            let need = if has_fuchendangmo(core, catalog, owner) {
                2
            } else {
                1
            };
            let hand = core.player(t).hand.clone();
            if hand.len() < need {
                return None;
            }
            card_ask(t, "cixiong_skill:target_discard", hand, need, need, true)
        }
        ASK_GUANSHI_DISCARD => {
            let need = if has_shangjiangdelvli(core, catalog, owner) {
                1
            } else {
                2
            };
            let pool = guanshi_discard_pool(core, catalog, owner);
            if pool.len() < need {
                return None;
            }
            card_ask(owner, "guanshi_skill:discard", pool, need, need, true)
        }
        ASK_SANJIAN_DISCARD => {
            let hand = core.player(owner).hand.clone();
            if hand.is_empty() {
                return None;
            }
            card_ask(owner, "sanjian_skill:discard", hand, 0, 1, true)
        }
        ASK_SANJIAN_TARGET => {
            let damaged = target.filter(|&p| core.player(p).alive)?;
            let tgts = sanjian_targets(core, catalog, owner, damaged);
            if tgts.is_empty() {
                return None;
            }
            player_ask(owner, "sanjian_skill:target", tgts, 0, 1, true)
        }
        ASK_YITIANJIAN_DISCARD => {
            let hand = core.player(owner).hand.clone();
            if hand.is_empty() {
                return None;
            }
            card_ask(owner, "yitianjian:discard", hand, 0, 1, true)
        }
        ASK_PYZHUREN_DISCARD => {
            let pool = pyzhuren_discard_pool(core, catalog, owner);
            if pool.is_empty() {
                return None;
            }
            card_ask(owner, "pyzhuren_diamond:discard", pool, 0, 1, true)
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l11_weapons(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    if stage == HostStage::Act || stage == HostStage::DrawSettle {
        return None;
    }
    let ask_id = card?;
    let (observer, spec) =
        build_l11_weapons_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
    Some(crate::snap::ask(
        observer,
        spec.prompt,
        spec.kind,
        spec.options,
        spec.min,
        spec.max,
        spec.allow_cancel,
    ))
}
