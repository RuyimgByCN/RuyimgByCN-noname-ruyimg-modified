//! L11（太虚幻境）Batch L11-5 系统、宝物扩展与神器询问构建（`l11_system_asks.rs`）。

use super::l11_common::*;
use super::*;
use crate::catalog::Catalog;
use crate::choice::ChoiceSpec;
use crate::ids::{CardId, PlayerId};
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) const ASK_TWSIDAO_CHOOSE: u32 = 1;

pub(super) fn build_l11_system_ask(
    ask_id: u32,
    owner: PlayerId,
    _target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    _catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    if !core.player(owner).alive {
        return None;
    }
    match ask_id {
        ASK_TWSIDAO_CHOOSE => tag_ask(owner, "twsidao:choose_treasure", 0..=2, false),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l11_system_hooks(
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
        build_l11_system_ask(ask_id, owner, target, targets, card, cards, core, catalog)?;
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
