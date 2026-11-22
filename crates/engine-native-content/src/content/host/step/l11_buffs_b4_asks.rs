//! L11（太虚幻境）Batch L11-4 Buffs 询问构建（`l11_buffs_b4_asks.rs`）。
//!
//! 承载：
//! - `txhj_luanjifengshi:discard_card`（`extension_skill.js:2532`，`HostStage::SelectCard`）：【万箭齐发】目标弃置 1 张牌抵消直击；
//! - `txhj_suikuiliejia:discard_armor`（`extension_skill.js:2754`，`HostStage::TargetChoice`）：持武器【杀】造成伤害后弃置目标防具。

use super::l11_common::*;
use super::*;
use crate::catalog::{Catalog, EquipSlot};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::ids::PlayerId;
use crate::snap::ExpectedAsk;
use crate::state::GameStateCore;

pub(super) fn build_l11_buffs_c_ask(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    let t = target?;
    if !core.player(t).alive {
        return None;
    }
    match stage {
        HostStage::SelectCard => {
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
                    prompt: "txhj_luanjifengshi:discard_card",
                    options,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        HostStage::TargetChoice => {
            let options: Vec<ChoiceOption> = core
                .player(t)
                .equips
                .iter()
                .copied()
                .filter(|&c| catalog.card_slot_of(core, c) == Some(EquipSlot::Armor))
                .map(ChoiceOption::Card)
                .collect();
            if options.is_empty() {
                return None;
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "txhj_suikuiliejia:discard_armor",
                    options,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::content::host) fn ask_l11_buffs_c(
    stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    _card: Option<u32>,
    _cards: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    let (observer, spec) = build_l11_buffs_c_ask(stage, owner, target, core, catalog)?;
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
