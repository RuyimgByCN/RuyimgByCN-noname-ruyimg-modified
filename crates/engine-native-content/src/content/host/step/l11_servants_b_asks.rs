//! L11（太虚幻境）侍从技能 Group B（`L11ServantSkillsB` tag 729）的询问构建（`l11_servants_b_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::Catalog;
use crate::choice::ChoiceSpec;
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_YAN3_CHOOSE_CARD: u32 = 1;
pub(super) const ASK_MANMAN2_CHOOSE_CARD: u32 = 2;
pub(super) const ASK_XIAOXIAO1_CONFIRM: u32 = 3;

pub(super) fn build_l11_servants_b_ask(
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
        ASK_YAN3_CHOOSE_CARD => card_ask(
            owner,
            "txhj_yanSkill3:choose_card",
            he_cards(core, tgt),
            1,
            1,
            false,
        ),
        ASK_MANMAN2_CHOOSE_CARD => card_ask(
            owner,
            "txhj_manmanSkill2:choose_card",
            he_cards(core, tgt),
            1,
            1,
            false,
        ),
        ASK_XIAOXIAO1_CONFIRM => confirm_ask(owner, "txhj_xiaoxiaoSkill1:confirm"),
        _ => None,
    }
}

pub(in crate::content::host) fn ask_l11_servants_b(
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
        build_l11_servants_b_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
