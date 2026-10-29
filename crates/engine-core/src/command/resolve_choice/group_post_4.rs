//! Choice-continuation handlers (post 4).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_resolve_dctingxian;
pub(crate) use resolve_resolve_dctingxian::*;

mod resolve_resolve_zhiba;
pub(crate) use resolve_resolve_zhiba::*;
mod settle_releiji_judgment;
pub(crate) use settle_releiji_judgment::*;
mod resolve_qizhi_target;
pub(crate) use resolve_qizhi_target::*;
mod resolve_kuizhu_damage;
pub(crate) use resolve_kuizhu_damage::*;
