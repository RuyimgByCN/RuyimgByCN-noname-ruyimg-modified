//! Effect primitive implementations — bingshi 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

mod resolve_potlvyi;
pub(crate) use resolve_potlvyi::*;

mod potcaoshuang_timing;
pub(crate) use potcaoshuang_timing::*;

mod resolve_potyuji;
pub(crate) use resolve_potyuji::*;

mod choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank;
pub(crate) use choose_discard_one_for_sha_refund_and_draw_if_red_restore_if_equal_rank::*;
mod first_shared_pile_card_by_kind;
pub(crate) use first_shared_pile_card_by_kind::*;
mod choose_give_one_hand_equip_or_recover;
pub(crate) use choose_give_one_hand_equip_or_recover::*;
