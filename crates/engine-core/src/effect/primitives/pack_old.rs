//! Effect primitive implementations — old cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

mod oldxuanfeng_distance_one;
pub(crate) use oldxuanfeng_distance_one::*;
mod resolve_beizhu;
pub(crate) use resolve_beizhu::*;
mod draw_then_choose_give_or_discard_hand;
pub(crate) use draw_then_choose_give_or_discard_hand::*;
