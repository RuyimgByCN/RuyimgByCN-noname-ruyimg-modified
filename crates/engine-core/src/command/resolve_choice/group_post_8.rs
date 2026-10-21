//! Choice-continuation handlers (post 8).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod open_equipment_pair_choice;
pub(crate) use open_equipment_pair_choice::*;
mod resolve_resolve_olzhijie;
pub(crate) use resolve_resolve_olzhijie::*;
mod resolve_choose_other_give_one_hand_equip;
pub(crate) use resolve_choose_other_give_one_hand_equip::*;
mod complete_view_as_response;
pub(crate) use complete_view_as_response::*;
