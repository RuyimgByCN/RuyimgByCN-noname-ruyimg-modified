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
