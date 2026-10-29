//! Choice-continuation handlers (post 5).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_zhizheng_commit;
pub(crate) use resolve_zhizheng_commit::*;

mod resolve_twjuxiang_card;
pub(crate) use resolve_twjuxiang_card::*;
mod resolve_qiaobian_discard;
pub(crate) use resolve_qiaobian_discard::*;
mod resolve_resolve_xinkuangfu;
pub(crate) use resolve_resolve_xinkuangfu::*;
mod resolve_resolve_nsdingzhou;
pub(crate) use resolve_resolve_nsdingzhou::*;
