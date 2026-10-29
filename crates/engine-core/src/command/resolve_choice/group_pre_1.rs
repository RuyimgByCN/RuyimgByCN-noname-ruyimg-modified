//! Choice-continuation handlers (pre 1).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

use super::*;

mod resolve_choose_other_yinghun;
pub(crate) use resolve_choose_other_yinghun::*;

mod resolve_oldimeng;
pub(crate) use resolve_oldimeng::*;
mod resolve_chegu;
pub(crate) use resolve_chegu::*;
mod resolve_youdi;
pub(crate) use resolve_youdi::*;
mod resolve_choose_category_then_target_give_matching_or_sha_unrespondable;
pub(crate) use resolve_choose_category_then_target_give_matching_or_sha_unrespondable::*;
