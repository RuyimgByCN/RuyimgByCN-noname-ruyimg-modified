//! Effect primitive implementations — bingshi 4 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

#[derive(Clone, Copy)]
pub(crate) struct PotfuanState {
    recorded: u32,
    used_round: u32,
    used: u32,
    only: u32,
    pending_card: CardId,
    pending_target: PlayerId,
    pending_first: u32,
}

impl Default for PotfuanState {
    fn default() -> Self {
        PotfuanState {
            recorded: 0,
            used_round: 0,
            used: 0,
            only: 0,
            pending_card: CardId::NONE,
            pending_target: PlayerId::NONE,
            pending_first: 0,
        }
    }
}

#[derive(Clone)]
pub(crate) struct PotlulianPending {
    card: CardId,
    kind: u8,
    targets: Vec<PlayerId>,
}

mod resolve_mbfeijing;
pub(crate) use resolve_mbfeijing::*;
mod suit_code;
pub(crate) use suit_code::*;
mod resolve_potlulian;
pub(crate) use resolve_potlulian::*;
