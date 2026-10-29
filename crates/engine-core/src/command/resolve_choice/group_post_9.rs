//! Choice-continuation handlers (post 9).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_choose_spxizhan_discard_or_lose;
pub(crate) use resolve_choose_spxizhan_discard_or_lose::*;

mod resolve_dcjue_choice;
pub(crate) use resolve_dcjue_choice::*;
mod resolve_houfeng_choice;
pub(crate) use resolve_houfeng_choice::*;
mod resolve_tongduo_choice;
pub(crate) use resolve_tongduo_choice::*;
