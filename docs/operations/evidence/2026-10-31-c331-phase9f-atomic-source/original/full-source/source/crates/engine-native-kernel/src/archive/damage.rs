//! Private106 damage settlement codec; old machine field layouts are unchanged.
use super::*;
use crate::event::{DamageOutcome as O, DamageProgress as P, DamageSettlement};
pub(super) fn enc_settlement(w: &mut W, s: &DamageSettlement) {
    enc_opt_u32(w, s.committed_damage);
    w.u32(s.hp_loss);
    w.u8(match s.outcome {
        O::Running => 0,
        O::Cancelled => 1,
        O::ReplacedByLoseHp => 2,
        O::Zero => 3,
        O::Completed => 4,
    });
    w.boolean(s.notrigger);
    w.u8(s.progress.tag());
    if let Some(eligible) = s.progress.chain_eligible() {
        w.boolean(eligible);
    }
    if let P::Chain { remaining } = &s.progress {
        w.len(remaining.len());
        for p in remaining {
            w.u8(*p);
        }
    }
}
pub(super) fn dec_settlement(r: &mut R<'_>) -> Result<DamageSettlement, ArchiveError> {
    let committed_damage = dec_opt_u32(r)?;
    let hp_loss = r.u32()?;
    let outcome = match r.u8()? {
        0 => O::Running,
        1 => O::Cancelled,
        2 => O::ReplacedByLoseHp,
        3 => O::Zero,
        4 => O::Completed,
        t => return Err(ArchiveError::BadTag("damage_outcome", t)),
    };
    let notrigger = r.boolean()?;
    let progress = match r.u8()? {
        0 => P::Before,
        1 => P::HpCommitted {
            chain_eligible: r.boolean()?,
        },
        2 => P::TargetDamage {
            chain_eligible: r.boolean()?,
        },
        3 => P::DyingPending {
            chain_eligible: r.boolean()?,
        },
        4 => P::DyingDone {
            chain_eligible: r.boolean()?,
        },
        5 => P::Source {
            chain_eligible: r.boolean()?,
        },
        6 => P::End {
            chain_eligible: r.boolean()?,
        },
        7 => P::After {
            chain_eligible: r.boolean()?,
        },
        8 => P::Chain {
            remaining: r.vec(|r| r.u8())?,
        },
        9 => P::Done,
        10 => P::ZeroPending,
        11 => P::Stopped,
        12 => P::Start,
        13 => P::Begin,
        14 => P::Begin1,
        15 => P::Begin2,
        16 => P::Begin3,
        17 => P::Begin4,
        18 => P::SubmitHp,
        19 => P::CancelledPending,
        t => return Err(ArchiveError::BadTag("damage_progress", t)),
    };
    Ok(DamageSettlement {
        committed_damage,
        hp_loss,
        outcome,
        notrigger,
        progress,
    })
}

pub(super) fn enc_native_damage_skill(
    w: &mut W,
    owner: u8,
    def: u16,
    window: crate::event::DamageWindowKind,
) {
    w.u8(85);
    w.u8(owner);
    w.u16(def);
    w.u8(window.tag());
}
