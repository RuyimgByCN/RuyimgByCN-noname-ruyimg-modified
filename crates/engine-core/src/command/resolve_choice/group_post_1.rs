//! Choice-continuation handlers (post 1).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_force_others_use_sha_on_nearest_or_lose_hp;
pub(crate) use resolve_force_others_use_sha_on_nearest_or_lose_hp::*;

mod resolve_choose_add_current_card_target;
pub(crate) use resolve_choose_add_current_card_target::*;
mod resolve_choose_gain_sha_for_opening_extra_play;
pub(crate) use resolve_choose_gain_sha_for_opening_extra_play::*;
mod resolve_choose_gain_resolved_sha_or_shan_then_optional_use_sha;
pub(crate) use resolve_choose_gain_resolved_sha_or_shan_then_optional_use_sha::*;
mod resolve_give_selected_then_continue_distinct_recover_at_two;
pub(crate) use resolve_give_selected_then_continue_distinct_recover_at_two::*;
