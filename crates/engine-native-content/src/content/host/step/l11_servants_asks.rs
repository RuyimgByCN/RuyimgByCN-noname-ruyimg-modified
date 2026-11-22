//! L11（太虚幻境）待援（S01–S04）与侍灵技能（S05–S28）的询问构建（`l11_servants_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceSpec;
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_DAIYUAN_MODE: u32 = 1;
pub(super) const ASK_DAIYUAN_CARDS: u32 = 2;
pub(super) const ASK_DAIYUAN_TARGET: u32 = 3;
pub(super) const ASK_QILIN2_CARD: u32 = 4;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CardType2 {
    Basic,
    Trick,
    Equip,
}

pub(super) fn card_type2(catalog: &Catalog, card: CardId) -> CardType2 {
    let k = catalog.card_kind(card);
    if k.is_basic_kind() {
        CardType2::Basic
    } else if k.equip_slot().is_some() {
        CardType2::Equip
    } else {
        CardType2::Trick
    }
}

pub(super) fn has_diff_color_pair(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    let he = he_cards(core, owner);
    if he.len() < 2 {
        return false;
    }
    let first_red = effective_color_is_red(core, catalog, owner, he[0]);
    he.iter()
        .skip(1)
        .any(|&c| effective_color_is_red(core, catalog, owner, c) != first_red)
}

pub(super) fn has_diff_type2_pair(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> bool {
    let he = he_cards(core, owner);
    if he.len() < 2 {
        return false;
    }
    let first_t = card_type2(catalog, he[0]);
    he.iter().skip(1).any(|&c| card_type2(catalog, c) != first_t)
}

pub(super) fn damaged_daiyuan_holders(
    core: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
) -> Vec<PlayerId> {
    alive_all(core, actor)
        .into_iter()
        .filter(|&p| {
            p != actor
                && has_named_skill(core, catalog, p, "txhj_daiyuan")
                && core.player(p).hp < core.player(p).max_hp as i32
        })
        .collect()
}

pub(super) fn build_l11_servant_ask(
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
        ASK_DAIYUAN_MODE => {
            let mut tags = Vec::new();
            if !has_named_skill(core, catalog, owner, "txhj_daiyuanA")
                && has_diff_color_pair(core, catalog, owner)
            {
                tags.push(0);
            }
            if !has_named_skill(core, catalog, owner, "txhj_daiyuanB")
                && has_diff_type2_pair(core, catalog, owner)
            {
                tags.push(1);
            }
            tag_ask(owner, "txhj_daiyuan:choose_mode", tags, true)
        }
        ASK_DAIYUAN_CARDS => {
            card_ask(owner, "txhj_daiyuan:choose_cards", he_cards(core, owner), 2, 2, true)
        }
        ASK_DAIYUAN_TARGET => player_ask(
            owner,
            "txhj_daiyuan:choose_target",
            damaged_daiyuan_holders(core, catalog, owner),
            1,
            1,
            false,
        ),
        ASK_QILIN2_CARD => {
            let t = target?;
            card_ask(owner, "txhj_qilinSkill2:choose_card", he_cards(core, t), 1, 1, false)
        }
        _ => None,
    }
}

pub(in crate::content::host) fn ask_l11_servants(
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
        build_l11_servant_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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

#[allow(dead_code)]
fn _unused_card_kind_anchor(_: CardKind) {}
