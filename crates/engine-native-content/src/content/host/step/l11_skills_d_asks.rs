//! L11（太虚幻境）武将技能 Group D（`L11GeneralSkillsD` tag 730）的询问构建（`l11_skills_d_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_OLDFUMAN_CARD: u32 = 1;
pub(super) const ASK_OLDFUMAN_TARGET: u32 = 2;
pub(super) const ASK_HGKUROU_CARD: u32 = 3;
pub(super) const ASK_GDSANCHEN_TARGET: u32 = 4;
pub(super) const ASK_GDSANCHEN_DISCARD: u32 = 5;
pub(super) const ASK_TXLVEMING_TARGET: u32 = 6;
pub(super) const ASK_TXLVEMING_RANK: u32 = 7;
pub(super) const ASK_TXLANGXI_TARGET: u32 = 8;
pub(super) const ASK_TXKUANGYI_CONFIRM: u32 = 9;
pub(super) const ASK_TXTANBEI_TARGET: u32 = 10;
pub(super) const ASK_TXTANBEI_MODE: u32 = 11;
pub(super) const ASK_TXYICHENG_CONFIRM: u32 = 12;
pub(super) const ASK_TXYICHENG_DISCARD: u32 = 13;

pub(super) fn build_l11_skills_d_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tgt = target.unwrap_or(owner);
    match ask_id {
        ASK_OLDFUMAN_CARD => {
            let shas: Vec<CardId> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_sha_like())
                .collect();
            card_ask(owner, "oldfuman:choose_card", shas, 1, 1, true)
        }
        ASK_OLDFUMAN_TARGET => {
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| !has_named_skill(core, catalog, p, "oldfuman2"))
                .collect();
            player_ask(owner, "oldfuman:choose_target", tgts, 1, 1, true)
        }
        ASK_HGKUROU_CARD => card_ask(
            owner,
            "hgkurou:choose_card",
            he_cards(core, owner),
            1,
            1,
            true,
        ),
        ASK_GDSANCHEN_TARGET => {
            let tgts: Vec<PlayerId> = alive_all(core, owner)
                .into_iter()
                .filter(|&p| flag(core, owner, &format!("skill_turn:l11_sanchen_done:{p}")) == 0)
                .collect();
            player_ask(owner, "gdsanchen:choose_target", tgts, 1, 1, true)
        }
        ASK_GDSANCHEN_DISCARD => {
            let pool = he_cards(core, tgt);
            let need = pool.len().min(3);
            card_ask(tgt, "gdsanchen:discard_cards", pool, need, need, false)
        }
        ASK_TXLVEMING_TARGET => {
            let my_eq = core.player(owner).equips.len();
            let tgts: Vec<PlayerId> = alive_others(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).equips.len() > my_eq)
                .collect();
            player_ask(owner, "txlveming:choose_target", tgts, 1, 1, true)
        }
        ASK_TXLVEMING_RANK => tag_ask(tgt, "txlveming:choose_rank", 0..=12, false),
        ASK_TXLANGXI_TARGET => {
            player_ask(owner, "txlangxi:choose_target", alive_others(core, owner), 1, 1, true)
        }
        ASK_TXKUANGYI_CONFIRM => confirm_ask(owner, "txkuangyi:confirm"),
        ASK_TXTANBEI_TARGET => {
            player_ask(owner, "txtanbei:choose_target", alive_others(core, owner), 1, 1, true)
        }
        ASK_TXTANBEI_MODE => tag_ask(tgt, "txtanbei:choose_mode", 0..=1, false),
        ASK_TXYICHENG_CONFIRM => confirm_ask(owner, "txyicheng:confirm"),
        ASK_TXYICHENG_DISCARD => {
            let pool = he_cards(core, tgt);
            spec_ask(
                tgt,
                ChoiceKind::SelectCards,
                "txyicheng:discard_card",
                pool.into_iter().map(ChoiceOption::Card).collect(),
                1,
                1,
                false,
            )
        }
        _ => None,
    }
}

pub(in crate::content::host) fn ask_l11_skills_d(
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
        build_l11_skills_d_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
