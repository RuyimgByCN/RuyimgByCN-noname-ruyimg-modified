//! Choice-continuation handlers (pre 3).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_new_yajiao;
pub(crate) use resolve_new_yajiao::*;

mod resolve_choose_gain_card_from_each_other_then_turn_over;
pub(crate) use resolve_choose_gain_card_from_each_other_then_turn_over::*;
mod resolve_choose_inspected_heart_destination;
pub(crate) use resolve_choose_inspected_heart_destination::*;
mod resolve_choose_qixing_weather_cards;
pub(crate) use resolve_choose_qixing_weather_cards::*;
mod append_pothaoshi_refills;
pub(crate) use append_pothaoshi_refills::*;
