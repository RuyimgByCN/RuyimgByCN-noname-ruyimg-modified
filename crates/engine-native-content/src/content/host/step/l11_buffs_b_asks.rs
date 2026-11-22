//! L11（太虚幻境）Batch L11-3 Buffs Group B 询问构建（`l11_buffs_b_asks.rs`）。
//!
//! 承载 `_buff_txhj_juxiangzhifeng` 目标弃置询问（`txhj_juxiangzhifeng:discard_card`）。

use super::l11_common::*;
use super::*;
use crate::catalog::Catalog;
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::PlayerId;
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) fn build_l11_buffs_b_ask(
    stage: HostStage,
    _owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<u32>,
    _cards: &[u32],
    core: &GameStateCore,
    _catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    if stage != HostStage::SelectCard {
        return None;
    }
    let t = target?;
    if !core.player(t).alive {
        return None;
    }
    let options: Vec<ChoiceOption> = he_cards(core, t)
        .into_iter()
        .map(ChoiceOption::Card)
        .collect();
    if options.is_empty() {
        return None;
    }
    Some((
        t,
        ChoiceSpec {
            kind: ChoiceKind::SelectCards,
            prompt: "txhj_juxiangzhifeng:discard_card",
            options,
            min: 0,
            max: 1,
            allow_cancel: true,
        },
    ))
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l11_buffs_b(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let (observer, spec) =
        build_l11_buffs_b_ask(stage, owner, target, targets, card, cards, core, catalog)?;
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
