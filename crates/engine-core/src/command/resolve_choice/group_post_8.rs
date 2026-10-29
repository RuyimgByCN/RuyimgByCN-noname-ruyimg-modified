//! Choice-continuation handlers (post 8).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod open_equipment_pair_choice;
pub(crate) use open_equipment_pair_choice::*;

mod resolve_resolve_mbbeiming;
pub(crate) use resolve_resolve_mbbeiming::*;
mod resolve_resolve_olzhijie;
pub(crate) use resolve_resolve_olzhijie::*;
mod resolve_resolve_biaozhao3;
pub(crate) use resolve_resolve_biaozhao3::*;
mod resolve_choose_other_give_one_hand_equip;
pub(crate) use resolve_choose_other_give_one_hand_equip::*;
mod resolve_retieji;
pub(crate) use resolve_retieji::*;
mod resolve_reanguo;
pub(crate) use resolve_reanguo::*;
mod complete_view_as_response;
pub(crate) use complete_view_as_response::*;
mod run_response_effects;
pub(crate) use run_response_effects::*;
