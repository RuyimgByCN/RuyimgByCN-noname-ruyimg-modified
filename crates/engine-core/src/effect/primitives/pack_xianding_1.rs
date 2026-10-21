//! Effect primitive implementations — xianding 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) const SPMIAOLVE_CUSTOM_CHOICE: u32 = 0x5350_5391;
pub(crate) const SPYINGJIA_CUSTOM_CHOICE: u32 = 0x5359_5391;
pub(crate) const MANTIANGUOHAI_CARD_DEF: u32 = 3_442_665_819;

mod set_potfaen_pending;
pub(crate) use set_potfaen_pending::*;
mod resolve_zhengbi;
pub(crate) use resolve_zhengbi::*;
mod resolve_sibian;
pub(crate) use resolve_sibian::*;
