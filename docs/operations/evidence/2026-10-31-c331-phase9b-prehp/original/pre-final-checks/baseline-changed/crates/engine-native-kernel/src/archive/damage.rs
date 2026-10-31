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
    let (tag, remaining) = match &s.progress {
        P::Before => (0, None),
        P::HpCommitted => (1, None),
        P::TargetDamage => (2, None),
        P::DyingPending => (3, None),
        P::DyingDone => (4, None),
        P::Source => (5, None),
        P::End => (6, None),
        P::After => (7, None),
        P::Chain { remaining } => (8, Some(remaining)),
        P::Done => (9, None),
        P::ZeroPending => (10, None),
        P::Stopped => (11, None),
    };
    w.u8(tag);
    if let Some(remaining) = remaining {
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
        1 => P::HpCommitted,
        2 => P::TargetDamage,
        3 => P::DyingPending,
        4 => P::DyingDone,
        5 => P::Source,
        6 => P::End,
        7 => P::After,
        8 => P::Chain {
            remaining: r.vec(|r| r.u8())?,
        },
        9 => P::Done,
        10 => P::ZeroPending,
        11 => P::Stopped,
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
