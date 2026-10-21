//! Effect primitive implementations — huicui cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

/// 魔狱 add-on gain mark (set when the previous use did not damage the owner).
pub(crate) const DCMOYU_ADD_MARK: u32 = 6_0000_0004;

/// 宽释 marks: watch flag on the target, once-per-turn flag on the owner.
pub(crate) const REKUANSHI_WATCH_MARK: u32 = 6_0000_0002;
pub(crate) const REKUANSHI_USED_MARK: u32 = 6_0000_0003;

/// Pending-duel mark on the owner; count holds the initiator's raw id. ABI 477.
pub(crate) const DCKUIZHEN_PENDING_MARK: u32 = 6_0000_0001;

/// 蛮智 branch-once-per-turn mark ids (give = 1, gain = 2).
pub(crate) const DCMANZHI_GIVE_MARK: u32 = 5_0000_0001;
pub(crate) const DCMANZHI_GAIN_MARK: u32 = 5_0000_0002;

mod potjixi_targets;
pub(crate) use potjixi_targets::*;
mod resolve_dcchiying;
pub(crate) use resolve_dcchiying::*;
mod choose_hand_compare_between;
pub(crate) use choose_hand_compare_between::*;
