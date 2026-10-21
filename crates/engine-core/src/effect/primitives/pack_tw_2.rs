//! Effect primitive implementations — tw 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) const RESOLVE_DDD_LIANGXI_CHOICE: u32 = 0xDCDD_6400;
pub(crate) const DDD_TONGYU_DEF: u32 = 2_302_761_270;
pub(crate) const DDD_TONGYU_BLOCKER_DEF: u32 = 3_861_572_400;

mod ddd_bailei_damage;
pub(crate) use ddd_bailei_damage::*;
mod resolve_zhugeruoxue_choice;
pub(crate) use resolve_zhugeruoxue_choice::*;
mod resolve_twzhengjian;
pub(crate) use resolve_twzhengjian::*;
