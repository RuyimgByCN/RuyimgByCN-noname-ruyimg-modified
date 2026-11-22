//! L11（太虚幻境）武将技能 Group E（`L11GeneralSkillsE` tag 731）的询问构建（`l11_skills_e_asks.rs`）。

use super::l11_common::*;
use super::l11_common_b2::*;
use super::*;
use crate::catalog::{CardKind, Catalog};
use crate::choice::ChoiceSpec;
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_TXYINLI_CARDS: u32 = 1;
pub(super) const ASK_TXNANGCE_CONFIRM: u32 = 2;
pub(super) const ASK_TXDUANZUI_CONFIRM: u32 = 3;
pub(super) const ASK_MDQIANXI7_CONFIRM: u32 = 4;
pub(super) const ASK_MDQIANXI7_DISCARD: u32 = 5;
pub(super) const ASK_MDQIANXI7_TARGET: u32 = 6;
pub(super) const ASK_MDQIANXI_CONFIRM: u32 = 7;
pub(super) const ASK_MDQIANXI_TARGET: u32 = 8;
pub(super) const ASK_LONGDAN_RECOVER: u32 = 9;
pub(super) const ASK_LONGDAN_DAMAGE: u32 = 10;
pub(super) const ASK_TXJUXIANG_TARGET: u32 = 11;
pub(super) const ASK_TXDAOJI_DISCARD: u32 = 12;
pub(super) const ASK_TXDAOJI_TARGET: u32 = 13;
pub(super) const ASK_TXDAOJI_HE: u32 = 14;

pub(super) fn build_l11_skills_e_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tgt = target.unwrap_or(owner);
    match ask_id {
        ASK_TXYINLI_CARDS => {
            let actor_tag = u32::from(tgt) + 1;
            let mut pool = Vec::new();
            for &c in &core.discard {
                if flag(core, owner, &format!("l11_yinli_eq:{c}")) == actor_tag && !pool.contains(&c)
                {
                    pool.push(c);
                }
            }
            card_ask(owner, "txyinli:choose_cards", pool, 1, usize::MAX, true)
        }
        ASK_TXNANGCE_CONFIRM => confirm_ask(owner, "txnangce:confirm"),
        ASK_TXDUANZUI_CONFIRM => confirm_ask(owner, "txduanzui:confirm"),
        ASK_MDQIANXI7_CONFIRM => confirm_ask(owner, "mdqianxi7:confirm"),
        ASK_MDQIANXI7_DISCARD => card_ask(
            owner,
            "mdqianxi7:discard_card",
            he_cards(core, owner),
            1,
            1,
            false,
        ),
        ASK_MDQIANXI7_TARGET => {
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.computed_distance(catalog, owner, p) <= 1)
                .collect();
            player_ask(owner, "mdqianxi7:choose_target", tgts, 1, 1, false)
        }
        ASK_MDQIANXI_CONFIRM => confirm_ask(owner, "mdqianxi:confirm"),
        ASK_MDQIANXI_TARGET => {
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.computed_distance(catalog, owner, p) <= 1)
                .collect();
            player_ask(owner, "mdqianxi:choose_target", tgts, 1, 1, false)
        }
        ASK_LONGDAN_RECOVER => {
            let excluded = cards.get(1).copied().map(|v| v as PlayerId);
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| Some(p) != excluded && core.player(p).hp < core.player(p).max_hp as i32)
                .collect();
            player_ask(owner, "zylongdan:recover_target", tgts, 1, 1, true)
        }
        ASK_LONGDAN_DAMAGE => {
            let excluded = cards.get(1).copied().map(|v| v as PlayerId);
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| Some(p) != excluded)
                .collect();
            player_ask(owner, "zylongdan:damage_target", tgts, 1, 1, true)
        }
        ASK_TXJUXIANG_TARGET => player_ask(
            owner,
            "txjuxiang:choose_target",
            alive_all(core, owner),
            1,
            1,
            true,
        ),
        ASK_TXDAOJI_DISCARD => {
            let pool: Vec<CardId> = he_cards(core, owner)
                .into_iter()
                .filter(|&c| !catalog.card_kind(c).is_basic_kind())
                .collect();
            card_ask(owner, "txdaoji:discard_card", pool, 1, 1, true)
        }
        ASK_TXDAOJI_TARGET => {
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !he_cards(core, p).is_empty())
                .collect();
            player_ask(owner, "txdaoji:choose_target", tgts, 1, 1, true)
        }
        ASK_TXDAOJI_HE => card_ask(owner, "txdaoji:choose_he", he_cards(core, tgt), 1, 1, false),
        _ => None,
    }
}

pub(super) fn find_txjuxiang_pool_card(
    catalog: &Catalog,
    pool: &[CardId],
    used_mask: u32,
) -> Option<(usize, u32)> {
    if used_mask == 3 {
        return None;
    }
    if let Some((idx, bit)) = pool.iter().enumerate().find_map(|(idx, &cid)| {
        let cd = catalog.cards.get(cid as usize)?;
        if cd.kind != CardKind::SavageAttack {
            return None;
        }
        if cd.suit == crate::catalog::Suit::Spade && cd.rank == 7 && (used_mask & 1) == 0 {
            Some((idx, 1))
        } else if cd.suit == crate::catalog::Suit::Club && cd.rank == 9 && (used_mask & 2) == 0 {
            Some((idx, 2))
        } else {
            None
        }
    }) {
        return Some((idx, bit));
    }
    pool.iter().position(|&cid| {
        catalog
            .cards
            .get(cid as usize)
            .is_some_and(|cd| cd.kind == CardKind::SavageAttack)
    })
    .map(|idx| (idx, if (used_mask & 1) == 0 { 1 } else { 2 }))
}

pub(super) fn find_red_or_black_color_val(
    catalog: &Catalog,
    core: &GameStateCore,
    owner: PlayerId,
    cid: CardId,
) -> u32 {
    if suit_is_red(catalog, core, owner, cid) {
        1
    } else {
        2
    }
}

pub(in crate::content::host) fn ask_l11_skills_e(
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
        build_l11_skills_e_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
