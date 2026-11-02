//! The root owns progression; each ordinary window uses the existing child frame queue.
use super::*;
use crate::event::{DamageOutcome as O, DamageProgress as P, DamageWindowKind as W};

#[derive(Clone)]
pub struct DamageFlow {
    pub root: EventId,
}
impl crate::catalog::SkillMachine for DamageFlow {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::DamageFlow { root: self.root }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(frame) = ctx.res.frame(self.root) else {
            return Flow::Done;
        };
        let EventData::Damage(mut d) = frame.data.clone() else {
            return Flow::Done;
        };
        if d.source.is_some_and(|s| !ctx.core.player(s).alive) {
            d.source = None;
        }
        if d.settlement.outcome == O::Running {
            if d.prevented {
                ctx.log(LogEvent::DamagePrevented { target: d.target });
                d.settlement.outcome = O::Cancelled;
                d.settlement.progress = P::CancelledPending;
            } else if d.amount <= 0 {
                d.settlement.outcome = O::Zero;
                d.settlement.progress = P::ZeroPending;
            }
        }
        if d.settlement.outcome == O::Cancelled
            && !matches!(d.settlement.progress, P::Stopped | P::CancelledPending)
        {
            d.settlement.progress = P::CancelledPending;
        }
        if d.settlement.outcome == O::ReplacedByLoseHp {
            d.settlement.progress = P::Stopped;
        }
        let mut child = None;
        match d.settlement.progress.clone() {
            P::Start => d.settlement.progress = P::Before,
            P::Before => d.settlement.progress = P::Begin,
            P::Begin => d.settlement.progress = P::Begin1,
            P::Begin1 => d.settlement.progress = P::Begin2,
            P::Begin2 => d.settlement.progress = P::Begin3,
            P::Begin3 => d.settlement.progress = P::Begin4,
            P::Begin4 => d.settlement.progress = P::SubmitHp,
            P::SubmitHp => {
                ctx.res.frame_mut(self.root).unwrap().data = EventData::Damage(d.clone());
                commit_damage(ctx, self.root, d);
                return Flow::Yield;
            }
            P::ZeroPending | P::CancelledPending => {
                let window = if d.settlement.progress == P::ZeroPending {
                    W::Zero
                } else {
                    W::Cancelled
                };
                d.settlement.progress = P::Stopped;
                if !d.settlement.notrigger {
                    child = Some(EventData::DamageWindow {
                        root: self.root,
                        window,
                    });
                }
            }
            P::Stopped | P::Done => {
                ctx.res.frame_mut(self.root).unwrap().data = EventData::Damage(d);
                return Flow::Done;
            }
            P::HpCommitted { chain_eligible } => {
                d.settlement.progress = P::TargetDamage { chain_eligible }
            }
            P::TargetDamage { chain_eligible } => {
                d.settlement.progress = P::DyingPending { chain_eligible };
                if ctx.core.player(d.target).alive
                    && ctx.core.player(d.target).hp <= 0
                    && !ctx.core.player(d.target).dying
                {
                    ctx.core.player_mut(d.target).dying = true;
                    ctx.log(LogEvent::DyingEntered { player: d.target });
                    child = Some(EventData::Dying {
                        player: d.target,
                        source: d.source,
                    });
                }
            }
            P::DyingPending { chain_eligible } => {
                d.settlement.progress = P::DyingDone { chain_eligible }
            }
            P::DyingDone { chain_eligible } => d.settlement.progress = P::Source { chain_eligible },
            P::Source { chain_eligible } => d.settlement.progress = P::End { chain_eligible },
            P::End { chain_eligible } => d.settlement.progress = P::After { chain_eligible },
            P::After { chain_eligible } => {
                let remaining = if chain_eligible {
                    ctx.core
                        .seat_order_from(ctx.core.turn.current)
                        .into_iter()
                        .filter(|p| *p != d.target && ctx.core.player(*p).linked)
                        .collect()
                } else {
                    Vec::new()
                };
                d.settlement.progress = P::Chain { remaining };
            }
            P::Chain { mut remaining } => {
                if remaining.is_empty() {
                    d.settlement.progress = P::Done;
                    d.settlement.outcome = O::Completed;
                } else {
                    let target = remaining.remove(0);
                    d.settlement.progress = P::Chain { remaining };
                    if ctx.core.player(target).alive && ctx.core.player(target).linked {
                        let mut chained =
                            damage_data_with(d.source, target, d.amount, d.nature, d.card);
                        chained.from_chain = true;
                        child = Some(EventData::Damage(chained));
                    }
                }
            }
        }
        if child.is_none() && !d.settlement.notrigger {
            if let Some(window) = d.settlement.progress.window() {
                child = Some(EventData::DamageWindow {
                    root: self.root,
                    window,
                });
            }
        }
        ctx.res.frame_mut(self.root).unwrap().data = EventData::Damage(d);
        if let Some(child) = child {
            ctx.emit(child);
        }
        Flow::Yield
    }
}
