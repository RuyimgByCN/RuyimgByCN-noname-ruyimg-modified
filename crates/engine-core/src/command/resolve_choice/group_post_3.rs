//! Choice-continuation handlers (post 3).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_draw_give_or_discard_hand_discard;
pub(crate) use resolve_draw_give_or_discard_hand_discard::*;
mod resolve_choose_give_two_different_colors_or_recover;
pub(crate) use resolve_choose_give_two_different_colors_or_recover::*;
mod resolve_resolve_lulve;
pub(crate) use resolve_resolve_lulve::*;
