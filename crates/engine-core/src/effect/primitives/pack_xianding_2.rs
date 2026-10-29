//! Effect primitive implementations — xianding 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

/// 安忍 counter mark id (标王沈 stdanran ported with mark 221181016).
pub(crate) const STDANRAN_COUNTER_MARK: u32 = 221_181_016;

pub(crate) const DCDANJI_MASHU_DEF: u32 = 2;
pub(crate) const DCDANJI_DCNUCHEN_DEF: u32 = 2_960_451_206;

pub(crate) const CILV_STATE_TAG: &[u8; 4] = b"CLV1";

pub(crate) const DCSBYINGMOU_STATE_TAG: [u8; 4] = *b"DYM1";

#[derive(Clone, Copy)]
pub(crate) struct DcsbyingmouSequence {
    target: PlayerId,
    source: PlayerId,
    card: CardId,
    any_damage: bool,
    had_damage_cards: bool,
    draw_branch: bool,
}

pub(crate) const DCSBMENGMOU_STATE_TAG: [u8; 4] = *b"DMM1";

#[derive(Clone, Copy)]
pub(crate) struct DcsbmengmouSequence {
    pub(crate) target: PlayerId,
    pub(crate) card: CardId,
    pub(crate) remaining: u32,
    pub(crate) yin: bool,
}

mod resolve_stdgaobian;
pub(crate) use resolve_stdgaobian::*;

mod bizu_groups;
pub(crate) use bizu_groups::*;
mod resolve_dc_sb_zhangxiu;
pub(crate) use resolve_dc_sb_zhangxiu::*;
mod continue_dcsbyingmou_spree;
pub(crate) use continue_dcsbyingmou_spree::*;
