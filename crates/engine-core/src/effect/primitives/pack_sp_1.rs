//! Effect primitive implementations — sp 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

mod choose_move_field_card_to;
pub(crate) use choose_move_field_card_to::*;

mod discard_from_hand;
pub(crate) use discard_from_hand::*;
mod choose_other_bizheng;
pub(crate) use choose_other_bizheng::*;
mod rejieyin_can_equip;
pub(crate) use rejieyin_can_equip::*;
mod resolve_oldxuanfeng;
pub(crate) use resolve_oldxuanfeng::*;
