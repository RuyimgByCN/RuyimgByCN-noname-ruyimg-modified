//! L11（太虚幻境）武将技能 Batch A（S29–S43）的询问构建（`l11_skills_a_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::Catalog;
use crate::choice::ChoiceSpec;
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_MOYANY_TARGET: u32 = 1;
pub(super) const ASK_YIRANG_TARGET: u32 = 2;
pub(super) const ASK_BOJI_CONFIRM: u32 = 3;
pub(super) const ASK_BOJI_DISCARD: u32 = 4;
pub(super) const ASK_DQTIANZI_TARGET: u32 = 5;
pub(super) const ASK_DQTIANZI_CARDS: u32 = 6;
pub(super) const ASK_MINGJIAN_CARDS: u32 = 7;
pub(super) const ASK_MINGJIAN_TARGET: u32 = 8;
pub(super) const ASK_MOJIAN_CONFIRM: u32 = 9;
pub(super) const ASK_YUSHOU_CONFIRM: u32 = 10;

pub(super) fn yirang_nonbasic_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<CardId> {
    he_cards(core, owner)
        .into_iter()
        .filter(|&c| !catalog.card_kind(c).is_basic_kind())
        .collect()
}

pub(super) fn yirang_targets(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    let my_max = core.player(owner).max_hp;
    alive_others(core, owner)
        .into_iter()
        .filter(|&p| core.player(p).max_hp > my_max)
        .collect()
}

pub(super) fn build_l11_skills_a_ask(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    _catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let tgt = target.unwrap_or(owner);
    match ask_id {
        ASK_MOYANY_TARGET => player_ask(
            owner,
            "tx_moyany:choose_target",
            alive_others(core, owner),
            1,
            1,
            false,
        ),
        ASK_YIRANG_TARGET => player_ask(
            owner,
            "txyirang:choose_target",
            yirang_targets(core, owner),
            1,
            1,
            true,
        ),
        ASK_BOJI_CONFIRM => confirm_ask(owner, "txayboji:confirm"),
        ASK_BOJI_DISCARD => card_ask(owner, "txayboji:discard_card", he_cards(core, owner), 1, 1, false),
        ASK_DQTIANZI_TARGET => player_ask(
            owner,
            "tx_dqtianzi:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_DQTIANZI_CARDS => {
            let h = core.player(tgt).hand.clone();
            let max = h.len();
            card_ask(owner, "tx_dqtianzi:choose_cards", h, 0, max, true)
        }
        ASK_MINGJIAN_CARDS => {
            let h = core.player(owner).hand.clone();
            let max = h.len();
            card_ask(owner, "xinmingjian:choose_cards", h, 1, max, true)
        }
        ASK_MINGJIAN_TARGET => player_ask(
            owner,
            "xinmingjian:choose_target",
            alive_others(core, owner),
            1,
            1,
            true,
        ),
        ASK_MOJIAN_CONFIRM => confirm_ask(owner, "tx_mojian:confirm"),
        ASK_YUSHOU_CONFIRM => confirm_ask(owner, "tx_yushou:confirm"),
        _ => None,
    }
}

pub(in crate::content::host) fn ask_l11_skills_a(
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
        build_l11_skills_a_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
