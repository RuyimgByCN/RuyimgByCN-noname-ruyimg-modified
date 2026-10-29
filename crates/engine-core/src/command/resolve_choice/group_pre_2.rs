//! Choice-continuation handlers (pre 2).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_twruilian;
pub(crate) use resolve_twruilian::*;

mod resolve_twyuyan;
pub(crate) use resolve_twyuyan::*;
mod resolve_twchenzhen;
pub(crate) use resolve_twchenzhen::*;
mod resolve_twshanghe;
pub(crate) use resolve_twshanghe::*;
mod resolve_twniujin_custom;
pub(crate) use resolve_twniujin_custom::*;
mod resolve_guguoanbang_choice;
pub(crate) use resolve_guguoanbang_choice::*;
