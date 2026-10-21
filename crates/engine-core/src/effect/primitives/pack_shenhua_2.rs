//! Effect primitive implementations — shenhua 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

/// 踞襄（张绣）禁手牌：本回合不能使用或打出手牌。
pub(crate) const DR_LT_XIONGLUAN_BAN: engine_api::SkillDefId =
    engine_api::SkillDefId::new(1234567905);

mod qizhi_count;
pub(crate) use qizhi_count::*;
mod resolve_liangyin;
pub(crate) use resolve_liangyin::*;
mod resolve_pottuntian;
pub(crate) use resolve_pottuntian::*;
