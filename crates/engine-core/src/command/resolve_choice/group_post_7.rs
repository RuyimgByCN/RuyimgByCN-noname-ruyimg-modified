//! Choice-continuation handlers (post 7).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_potweiyan_choice;
pub(crate) use resolve_potweiyan_choice::*;
mod resolve_reveal_category_distribution;
pub(crate) use resolve_reveal_category_distribution::*;
mod resolve_discard_equipment_pair_with_armor;
pub(crate) use resolve_discard_equipment_pair_with_armor::*;
