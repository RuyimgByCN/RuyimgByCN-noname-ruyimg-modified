//! Effect primitive implementations — tw 3 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) const TWCHENGPU_CUSTOM_CHOICE: u32 = 0x5457_8901;

pub(crate) const TWCAOANG_CUSTOM_CHOICE: u32 = 0x5457_9001;
pub(crate) const TWNIUJIN_CUSTOM_CHOICE: u32 = 0x5457_9101;
pub(crate) const TWMOUKUI_CUSTOM_CHOICE: u32 = 0x5457_9301;
pub(crate) const TWCHENXUN_CUSTOM_CHOICE: u32 = 0x5457_9701;
pub(crate) const TWFENXIAN_CUSTOM_CHOICE: u32 = 0x5457_9901;

/// Custom-continuation marker for ABI 419 追锋选目标（payload[0]）。
pub(crate) const RESOLVE_DBZHUIFENG_TARGET: u32 = 0xDCDA_4190;

mod resolve_twtianyu;
pub(crate) use resolve_twtianyu::*;
mod resolve_twchenxun;
pub(crate) use resolve_twchenxun::*;
