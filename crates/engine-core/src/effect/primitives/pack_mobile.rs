//! Effect primitive implementations — mobile cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum VirtualCardTracking {
    None,
    SourceFree,
    Dcctjiuxian,
}

mod resolve_jiaohua;
pub(crate) use resolve_jiaohua::*;
mod give_cards;
pub use give_cards::*;
mod commit_damage;
pub(crate) use commit_damage::*;
