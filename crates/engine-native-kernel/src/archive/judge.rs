use super::*;
pub(super) fn enc_judge_machine(w: &mut W, m: &MachineSnapshot) {
    match m {
        MachineSnapshot::JudgeContinuation {
            owner,
            queue,
            current,
            ordinary,
            started,
            payment,
        } => {
            w.u8(87);
            w.u8(*owner);
            w.len(queue.len());
            for c in queue {
                w.u32(*c)
            }
            enc_opt_u32(w, *ordinary);
            w.boolean(*started);
            w.boolean(current.is_some());
            if let Some(c) = current {
                enc_judgement_progress(w, c)
            };
            w.boolean(payment.is_some());
            if let Some(p) = payment {
                w.u8(p.player);
                w.u32(p.old);
                w.boolean(p.discard_old);
                w.u16(p.provider);
            }
        }
        MachineSnapshot::JudgePhase {
            owner,
            queue,
            current,
        } => {
            w.u8(4);
            enc_pid(w, owner);
            w.len(queue.len());
            for c in queue {
                w.u32(*c);
            }
            match current {
                Some(c) => {
                    w.u8(1);
                    enc_judgement_progress(w, c);
                }
                None => w.u8(0),
            }
        }
        _ => unreachable!("judgement encoder caller"),
    }
}

pub(super) fn dec_judge_continuation(r: &mut R<'_>) -> Result<MachineSnapshot, ArchiveError> {
    Ok(MachineSnapshot::JudgeContinuation {
        owner: r.u8()?,
        queue: r.vec(|r| r.u32())?,
        ordinary: dec_opt_u32(r)?,
        started: r.boolean()?,
        current: if r.boolean()? {
            Some(Box::new(dec_judgement_progress(r)?))
        } else {
            None
        },
        payment: if r.boolean()? {
            Some(crate::snap::JudgeReplacementSnap {
                player: r.u8()?,
                old: r.u32()?,
                discard_old: r.boolean()?,
                provider: r.u16()?,
            })
        } else {
            None
        },
    })
}
