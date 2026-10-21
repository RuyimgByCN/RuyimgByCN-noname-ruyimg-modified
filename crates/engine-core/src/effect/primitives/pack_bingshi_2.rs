//! Effect primitive implementations — bingshi 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

mod resolve_potlvyi;
pub(crate) use resolve_potlvyi::*;
mod first_shared_pile_card_by_kind;
pub(crate) use first_shared_pile_card_by_kind::*;
