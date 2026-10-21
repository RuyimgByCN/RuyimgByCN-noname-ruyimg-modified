//! Effect primitive implementations — extra 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) const DILI_DEF: u32 = 3_732_080_996;
pub(crate) const YUHENG_DEF: u32 = 3_006_389_674;
pub(crate) const DILI_LINES: [u32; 6] = [
    3_511_235_243,
    3_520_220_242,
    4_057_699_019,
    2_682_708_493,
    3_616_039_425,
    2_521_447_965,
];
pub(crate) const YUHENG_POOL: [u32; 18] = [
    2_790_748_306,
    3_138_095_197,
    2_238_473_770,
    3_285_976_176,
    3_859_813_014,
    3_075_552_814,
    4_211_343_786,
    4_068_014_439,
    2_231_312_450,
    2_455_708_599,
    2_662_349_862,
    2_398_589_569,
    2_873_874_515,
    3_295_441_821,
    2_509_094_453,
    3_040_931_439,
    3_834_102_895,
    3_473_961_755,
];
pub(crate) const YUHENG_ALL_MASK: u32 = (1 << YUHENG_POOL.len()) - 1;
pub(crate) const CHANGANDAJIAN_DESTROY_DEF: u32 = 4_232_406_220;
pub(crate) const CHANGANDAJIAN_EQUIP2_DEF: u32 = 4_130_114_000;
pub(crate) const CHANGANDAJIAN_DEFS: [u32; 6] = [
    3_238_845_718,
    4_130_114_000,
    2_828_308_711,
    3_888_754_104,
    4_174_045_670,
    3_540_539_807,
];

#[derive(Clone, Copy, Default)]
pub(crate) struct YuhengState {
    history: u32,
    full_remaining: u32,
    current_remaining: u32,
}

mod draw_then_optionally_distribute_hand_cards;
pub(crate) use draw_then_optionally_distribute_hand_cards::*;
mod replace_draw_with_distinct_suit_selection;
pub(crate) use replace_draw_with_distinct_suit_selection::*;
mod resolve_nzry_longnu;
pub(crate) use resolve_nzry_longnu::*;
