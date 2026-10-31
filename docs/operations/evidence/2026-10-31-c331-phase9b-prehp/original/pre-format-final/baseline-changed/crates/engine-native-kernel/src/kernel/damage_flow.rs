//! One intrinsic damage continuation; settlement/progress belongs only to its root.
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
        if d.prevented && d.settlement.outcome == O::Running {
            d.settlement.outcome = O::Cancelled;
            d.settlement.progress = P::Stopped;
        }
        if d.amount <= 0 && d.settlement.outcome == O::Running {
            d.settlement.outcome = O::Zero;
            d.settlement.progress = P::ZeroPending;
        }
        let mut child = None;
        match d.settlement.progress.clone() {
            P::Before => {
                d.settlement.progress = P::Stopped;
            }
            P::ZeroPending => {
                d.settlement.progress = P::Stopped;
                child = Some(EventData::DamageWindow {
                    root: self.root,
                    window: W::Zero,
                });
            }
            P::Stopped => {
                ctx.res.frame_mut(self.root).unwrap().data = EventData::Damage(d);
                return Flow::Done;
            }
            P::HpCommitted | P::TargetDamage => {
                d.settlement.progress = P::DyingPending;
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
            P::DyingPending => {
                d.settlement.progress = P::DyingDone;
            }
            P::DyingDone => {
                d.settlement.progress = P::Source;
                if !d.settlement.notrigger {
                    child = Some(EventData::DamageWindow {
                        root: self.root,
                        window: W::Source,
                    });
                }
            }
            P::Source => {
                d.settlement.progress = P::End;
                if !d.settlement.notrigger {
                    child = Some(EventData::DamageWindow {
                        root: self.root,
                        window: W::End,
                    });
                }
            }
            P::End => {
                d.settlement.progress = P::After;
                if !d.settlement.notrigger {
                    child = Some(EventData::DamageWindow {
                        root: self.root,
                        window: W::After,
                    });
                }
            }
            P::After => {
                let mut remaining = Vec::new();
                if d.nature != crate::event::DamageNature::Normal
                    && !d.from_chain
                    && ctx.core.player(d.target).linked
                {
                    remaining = ctx
                        .core
                        .seat_order_from(d.target)
                        .into_iter()
                        .filter(|p| *p != d.target && ctx.core.player(*p).linked)
                        .collect();
                    for p in ctx
                        .core
                        .alive_players()
                        .filter(|p| ctx.core.player(*p).linked)
                        .collect::<Vec<_>>()
                    {
                        ctx.core.player_mut(p).linked = false;
                        ctx.log(LogEvent::Linked {
                            player: p,
                            linked: false,
                        });
                    }
                }
                d.settlement.progress = P::Chain { remaining };
            }
            P::Chain { mut remaining } => {
                if remaining.is_empty() {
                    d.settlement.progress = P::Done;
                    d.settlement.outcome = O::Completed;
                } else {
                    let target = remaining.remove(0);
                    d.settlement.progress = P::Chain { remaining };
                    let mut chained =
                        damage_data_with(d.source, target, d.amount, d.nature, d.card);
                    chained.from_chain = true;
                    child = Some(EventData::Damage(chained));
                }
            }
            P::Done => {
                ctx.res.frame_mut(self.root).unwrap().data = EventData::Damage(d);
                return Flow::Done;
            }
        }
        ctx.res.frame_mut(self.root).unwrap().data = EventData::Damage(d);
        if let Some(child) = child {
            ctx.emit(child);
        }
        Flow::Yield
    }
}
