//! Choice-continuation handlers (post 6).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_resolve_dckuizhen;
pub(crate) use resolve_resolve_dckuizhen::*;

mod resolve_resolve_xinxingluan;
pub(crate) use resolve_resolve_xinxingluan::*;

mod resolve_resolve_dcmanzhi;
pub(crate) use resolve_resolve_dcmanzhi::*;
mod resolve_jiebing;
pub(crate) use resolve_jiebing::*;
mod resolve_mbduanyang;
pub(crate) use resolve_mbduanyang::*;
mod resolve_potqingshi_choice;
pub(crate) use resolve_potqingshi_choice::*;
