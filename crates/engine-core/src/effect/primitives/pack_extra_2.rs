//! Effect primitive implementations — extra 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BeigeOutcomePolicy {
    Classic,
    Configured {
        heart_recover_amount: u32,
        diamond_draw_count: u32,
    },
}

pub(crate) const DAMAGE_DISCARD_WEAPON_RANGE: u32 = 0;
pub(crate) const DAMAGE_DISCARD_DISTANCE: u32 = 1;

mod choose_link_other;
pub(crate) use choose_link_other::*;

mod beige_stage_effect;
pub(crate) use beige_stage_effect::*;
mod reveal_uniform_hand_draw_self_choose_others;
pub(crate) use reveal_uniform_hand_draw_self_choose_others::*;
mod choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both;
pub(crate) use choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both::*;
mod reduce_bound_damage_by_one;
pub(crate) use reduce_bound_damage_by_one::*;
