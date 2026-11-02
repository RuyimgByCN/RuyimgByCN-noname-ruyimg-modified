//! Internal invocation receipts; existing machine85 bytes are unchanged.
use super::*;
use crate::event::{InvocationData, InvocationPhase, NativeInvocationProgress as P};
pub(super) fn enc_context(w: &mut W, d: &InvocationData) {
    w.u8(d.owner);
    w.u16(d.def);
    w.u32(d.window);
    enc_opt_pid(w, &d.source);
    w.u8(d.target);
    w.u8(d.phase.tag());
    w.boolean(d.cancelled);
    enc_opt_u32(w, d.jiding_card);
    w.len(d.jiding_changes.len());
    for c in &d.jiding_changes {
        w.u32(c.card); w.u16(c.def); w.u8(c.actor); w.u8(c.before_player); w.u8(c.before_target);
        match &c.cost {
            crate::event::JidingCardCost::Zhenlie { lose_hp } => { w.u8(0); w.u32(*lose_hp); }
            crate::event::JidingCardCost::TongJi { card, loss } => { w.u8(1); w.u32(*card); enc_opt_u32(w, *loss); }
            crate::event::JidingCardCost::Yihua { cards, losses } => {
                w.u8(2); w.len(cards.len()); for card in cards { w.u32(*card); }
                w.len(losses.len()); for child in losses { w.u32(*child); }
            }
        }
    }
    enc_opt_u32(w, d.judgement.map(|s| s as u32));
    w.boolean(d.deferred_log);
    w.boolean(d.payment.is_some());
    if let Some(p) = &d.payment {
        w.len(p.selected.len());
        for c in &p.selected {
            w.u32(*c)
        }
        w.u32(p.log_begin);
        w.boolean(p.committed);
        w.len(p.loss_children.len());
        for id in &p.loss_children {
            w.u32(*id)
        }
        enc_opt_u32(w, p.after_log);
    }
    w.boolean(d.effect_applied);
    w.boolean(d.judge_receipt.is_some());
    if let Some(p) = &d.judge_receipt {
        w.u32(p.child);
        w.u8(p.target);
        w.u8(match p.purpose {
            crate::event::JudgePurpose::Yuance(o) => o.tag(),
            crate::event::JudgePurpose::Jidian => 2,
        });
    }
}
pub(super) fn dec_context(r: &mut R<'_>) -> Result<InvocationData, ArchiveError> {
    let mut data = InvocationData {
        owner: r.u8()?,
        def: r.u16()?,
        window: r.u32()?,
        source: dec_opt_pid(r)?,
        target: r.u8()?,
        phase: InvocationPhase::from_tag(r.u8()?)
            .ok_or(ArchiveError::BadTag("invocation_phase", 0))?,
        cancelled: r.boolean()?,
        jiding_card: dec_opt_u32(r)?,
        jiding_changes: r.vec(|r| {
            let card=r.u32()?; let def=r.u16()?; let actor=r.u8()?; let before_player=r.u8()?; let before_target=r.u8()?;
            let cost=match r.u8()? {
                0 => crate::event::JidingCardCost::Zhenlie { lose_hp:r.u32()? },
                1 => crate::event::JidingCardCost::TongJi { card:r.u32()?, loss:dec_opt_u32(r)? },
                2 => crate::event::JidingCardCost::Yihua { cards:r.vec(|r|r.u32())?, losses:r.vec(|r|r.u32())? },
                t => return Err(ArchiveError::BadTag("jiding_card_cost",t)),
            };
            Ok(crate::event::JidingCardChange { card,def,actor,before_player,before_target,cost })
        })?,
        judgement: match dec_opt_u32(r)? {
            None => None,
            Some(0) => Some(crate::catalog::Suit::Spade),
            Some(1) => Some(crate::catalog::Suit::Heart),
            Some(2) => Some(crate::catalog::Suit::Club),
            Some(3) => Some(crate::catalog::Suit::Diamond),
            Some(t) => return Err(ArchiveError::BadTag("judge_suit", t as u8)),
        },
        deferred_log: r.boolean()?,
        effect_applied: false,
        judge_receipt: None,
        payment: if r.boolean()? {
            Some(crate::event::InvocationPayment {
                selected: r.vec(|r| r.u32())?,
                log_begin: r.u32()?,
                committed: r.boolean()?,
                loss_children: r.vec(|r| r.u32())?,
                after_log: dec_opt_u32(r)?,
            })
        } else {
            None
        },
    };
    data.effect_applied = r.boolean()?;
    if r.boolean()? {
        let child = r.u32()?;
        let target = r.u8()?;
        let purpose = match r.u8()? {
            0 => crate::event::JudgePurpose::Yuance(crate::event::YuanceOption::LoseHp),
            1 => crate::event::JudgePurpose::Yuance(crate::event::YuanceOption::Recover),
            2 => crate::event::JudgePurpose::Jidian,
            t => return Err(ArchiveError::BadTag("judge_purpose", t)),
        };
        data.judge_receipt = Some(crate::event::InvocationJudgeReceipt {
            child,
            target,
            purpose,
        });
    }
    Ok(data)
}
pub(super) fn enc_progress(w: &mut W, p: &P) {
    match p {
        P::YuanceAsk => w.u8(12),
        P::JidianAsk { targets } => {
            w.u8(13);
            enc_pids(w, targets);
        }
        P::AwaitJudge {
            child,
            target,
            purpose,
        } => {
            w.u8(14);
            w.u32(*child);
            w.u8(*target);
            w.u8(match purpose {
                crate::event::JudgePurpose::Yuance(crate::event::YuanceOption::LoseHp) => 0,
                crate::event::JudgePurpose::Yuance(crate::event::YuanceOption::Recover) => 1,
                crate::event::JudgePurpose::Jidian => 2,
            });
        }
        P::BodyDone => w.u8(15),
        P::JidianAfterLog { target } => {
            w.u8(16);
            w.u8(*target);
        }
        P::QinglongAsk {
            target,
            hand,
            equip,
        } => {
            w.u8(17);
            w.u8(*target);
            w.len(hand.len());
            for c in hand {
                w.u32(*c)
            }
            w.len(equip.len());
            for c in equip {
                w.u32(*c)
            }
        }
        P::QinglongLog {
            target,
            card,
            public,
        } => {
            w.u8(18);
            w.u8(*target);
            w.u32(*card);
            w.boolean(*public);
        }
        P::QinglongLoss {
            target,
            card,
            public,
        } => {
            w.u8(19);
            w.u8(*target);
            w.u32(*card);
            w.boolean(*public);
        }
        P::Ready => w.u8(0),
        P::BeforeInvocation { context } => {
            w.u8(1);
            enc_context(w, context)
        }
        P::AwaitInvocation { child, admission } => {
            w.u8(2);
            w.u32(*child);
            enc_context(w, admission);
        }
        P::Finished => w.u8(3),
        P::Body => w.u8(4),
        P::HolderAsk { candidates } => {
            w.u8(5);
            w.len(candidates.len());
            for c in candidates {
                w.u32(*c)
            }
        }
        P::HolderPay { cards } => {
            w.u8(6);
            w.len(cards.len());
            for c in cards {
                w.u32(*c)
            }
        }
        P::HolderLoss { count } => {
            w.u8(7);
            w.u32(*count)
        }
        P::HolderAfterLog { count } => {
            w.u8(8);
            w.u32(*count)
        }
        P::SourceAsk {
            player,
            count,
            candidates,
        } => {
            w.u8(9);
            w.u8(*player);
            w.u32(*count);
            w.len(candidates.len());
            for c in candidates {
                w.u32(*c)
            }
        }
        P::SourceLoss => w.u8(10),
        P::JidingCard { child } => {
            w.u8(11);
            w.u32(*child)
        }
    }
}
pub(super) fn dec_progress(r: &mut R<'_>) -> Result<P, ArchiveError> {
    Ok(match r.u8()? {
        12 => P::YuanceAsk,
        13 => P::JidianAsk {
            targets: dec_pids(r)?,
        },
        14 => P::AwaitJudge {
            child: r.u32()?,
            target: r.u8()?,
            purpose: match r.u8()? {
                0 => crate::event::JudgePurpose::Yuance(crate::event::YuanceOption::LoseHp),
                1 => crate::event::JudgePurpose::Yuance(crate::event::YuanceOption::Recover),
                2 => crate::event::JudgePurpose::Jidian,
                t => return Err(ArchiveError::BadTag("judge_purpose", t)),
            },
        },
        15 => P::BodyDone,
        16 => P::JidianAfterLog { target: r.u8()? },
        17 => P::QinglongAsk {
            target: r.u8()?,
            hand: r.vec(|r| r.u32())?,
            equip: r.vec(|r| r.u32())?,
        },
        18 => P::QinglongLog {
            target: r.u8()?,
            card: r.u32()?,
            public: r.boolean()?,
        },
        19 => P::QinglongLoss {
            target: r.u8()?,
            card: r.u32()?,
            public: r.boolean()?,
        },
        0 => P::Ready,
        1 => P::BeforeInvocation {
            context: dec_context(r)?,
        },
        2 => P::AwaitInvocation {
            child: r.u32()?,
            admission: dec_context(r)?,
        },
        3 => P::Finished,
        4 => P::Body,
        5 => P::HolderAsk {
            candidates: r.vec(|r| r.u32())?,
        },
        6 => P::HolderPay {
            cards: r.vec(|r| r.u32())?,
        },
        7 => P::HolderLoss { count: r.u32()? },
        8 => P::HolderAfterLog { count: r.u32()? },
        9 => P::SourceAsk {
            player: r.u8()?,
            count: r.u32()?,
            candidates: r.vec(|r| r.u32())?,
        },
        10 => P::SourceLoss,
        11 => P::JidingCard { child: r.u32()? },
        t => return Err(ArchiveError::BadTag("native_invocation_progress", t)),
    })
}
