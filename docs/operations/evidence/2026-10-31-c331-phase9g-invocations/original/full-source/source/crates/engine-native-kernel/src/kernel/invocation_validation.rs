//! Bind admitted continuations to their real trigger and invocation, independently of provider renewal.
use super::*;
use crate::event::{InvocationPhase, NativeInvocationProgress as P};
use crate::resolution::Slot;
use crate::snap::{ExpectedAsk, MachineSnapshot as M, ResolutionSnapshot, SnapshotError};
impl NativeInvocation {
    pub fn expected_ask(
        owner: PlayerId,
        progress: &P,
        _core: &crate::state::GameStateCore,
    ) -> Option<ExpectedAsk> {
        if matches!(progress, P::YuanceAsk) {
            return Some(crate::snap::ask(
                owner,
                "c331_yuance",
                ChoiceKind::Control,
                vec![
                    ChoiceOption::Control(crate::event::YuanceOption::LoseHp),
                    ChoiceOption::Control(crate::event::YuanceOption::Recover),
                ],
                1,
                1,
                true,
            ));
        }
        if let P::JidianAsk { targets } = progress {
            return Some(crate::snap::ask(
                owner,
                "c331_jidian_target",
                ChoiceKind::SelectPlayers,
                targets.iter().copied().map(ChoiceOption::Player).collect(),
                1,
                1,
                true,
            ));
        }
        let (actor, prompt, candidates, min, max) = match progress {
            P::HolderAsk { candidates } => (
                owner,
                "c331_zhangwu_holder",
                candidates,
                1,
                candidates.len(),
            ),
            P::SourceAsk {
                player,
                count,
                candidates,
            } => (
                *player,
                "c331_zhangwu_source",
                candidates,
                *count as usize,
                *count as usize,
            ),
            _ => return None,
        };
        Some(crate::snap::ask(
            actor,
            prompt,
            ChoiceKind::SelectCards,
            candidates.iter().copied().map(ChoiceOption::Card).collect(),
            min,
            max,
            true,
        ))
    }
    pub fn validate_bindings(
        core: &crate::state::GameStateCore,
        catalog: &crate::catalog::Catalog,
        res: &ResolutionSnapshot,
    ) -> Result<(), SnapshotError> {
        let bad = || SnapshotError::IdOutOfBounds {
            what: "native_invocation_binding",
            got: 0,
        };
        for park in &res.parks {
            if let M::JudgeContinuation {
                owner,
                queue,
                current,
                ordinary,
                started,
                payment,
            } = &park.machine
            {
                if let Some(pay) = payment {
                    if usize::from(pay.player) >= core.players.len()
                        || pay.old as usize >= catalog.cards.len()
                        || current
                            .as_ref()
                            .is_none_or(|c| c.judge_card == pay.old || c.finalized || c.asked)
                    {
                        return Err(bad());
                    }
                }
                if let Some(invocation) = ordinary {
                    let Slot::Intrinsic { frame } = park.slot else {
                        return Err(bad());
                    };
                    let judge = res.stack.iter().find(|f| f.id == frame).ok_or_else(bad)?;
                    if !matches!(judge.data,EventData::OrdinaryJudge{owner:p,invocation:i}if p==*owner&&i==*invocation)||judge.parent!=Some(*invocation)||!queue.is_empty()||current.as_ref().is_some_and(|c|c.trick.is_some()||!*started)||!res.parks.iter().any(|p|p.slot==Slot::Intrinsic{frame:*invocation}&&matches!(&p.machine,M::NativeInvocation{progress:P::AwaitJudge{child,target,..},..}if *child==frame&&*target==*owner)){return Err(bad());}
                } else if current.as_ref().is_some_and(|c| c.trick.is_none()) {
                    return Err(bad());
                }
            }
        }
        for f in &res.stack {
            if let EventData::SkillInvocation(context) = &f.data {
                if context.phase == InvocationPhase::Before
                    && (context.judgement.is_some()
                        || context.jiding_card.is_some()
                        || context.deferred_log)
                {
                    return Err(bad());
                }
                if let Some(card) = context.jiding_card {
                    if res.stack.iter().any(|child|child.id==card)&&!res.parks.iter().any(|p|p.slot==Slot::Intrinsic{frame:f.id}&&matches!(&p.machine,M::NativeInvocation{progress:P::JidingCard{child},owner,def,..}if *child==card&&*owner==context.owner&&*def==context.def)){return Err(bad());}
                }

                let index = res
                    .stack
                    .iter()
                    .position(|g| g.id == f.id)
                    .ok_or_else(bad)?;
                let parent = index
                    .checked_sub(1)
                    .and_then(|n| res.stack.get(n))
                    .ok_or_else(bad)?;
                if f.parent!=Some(context.window)||parent.id!=context.window||!matches!(parent.data,EventData::DamageWindow{..})||!res.parks.iter().any(|p|matches!(&p.machine,M::NativeInvocation{owner,def,progress:P::AwaitInvocation{child},..}if *child==f.id&&*owner==context.owner&&*def==context.def)&&p.slot==Slot::Trigger{frame:context.window}){return Err(bad());}
            }
        }
        for park in &res.parks {
            let M::NativeInvocation {
                owner,
                def,
                window,
                progress,
            } = &park.machine
            else {
                continue;
            };
            let effect = catalog
                .skills
                .get(*def as usize)
                .and_then(|s| s.behavior.native_damage_effect(window.event_kind()))
                .ok_or_else(bad)?;
            if effect.window() != *window || !crate::event::uses_native_invocation(effect) {
                return Err(bad());
            }
            let frame_id = match park.slot {
                Slot::Trigger { frame } | Slot::Intrinsic { frame } => frame,
            };
            let index = res
                .stack
                .iter()
                .position(|f| f.id == frame_id)
                .ok_or_else(bad)?;
            let f = &res.stack[index];
            if park.resume_depth != index + 1 {
                return Err(bad());
            }
            if let Slot::Trigger { .. } = park.slot {
                if !matches!(f.data,EventData::DamageWindow{window:w,..}if w==*window)||f.stage!=crate::event::Stage::AfterTriggers||!res.parks.iter().any(|p|matches!(&p.machine,M::TriggerOrder{frame,done,..}if *frame==f.id&&done.contains(&(*owner,*def)))){return Err(bad());}
                match progress {
                    P::Ready => {
                        if !core.player(*owner).alive {
                            return Err(bad());
                        }
                    }
                    P::BeforeInvocation { context } => {
                        if context.owner != *owner
                            || context.def != *def
                            || context.window != f.id
                            || context.phase != InvocationPhase::Before
                            || context.cancelled
                        {
                            return Err(bad());
                        }
                    }
                    P::AwaitInvocation { child } => {
                        if *child <= f.id || *child >= res.next_event_id {
                            return Err(bad());
                        }
                        if let Some(child) = res.stack.iter().find(|g| g.id == *child) {
                            if !matches!(&child.data,EventData::SkillInvocation(c)if c.owner==*owner&&c.def==*def&&c.window==f.id)
                            {
                                return Err(bad());
                            }
                        }
                    }
                    P::Finished => {}
                    _ => return Err(bad()),
                }
            } else {
                let EventData::SkillInvocation(context) = &f.data else {
                    return Err(bad());
                };
                if context.owner != *owner
                    || context.def != *def
                    || context.phase != InvocationPhase::Body
                    || f.stage != crate::event::Stage::Intrinsic
                {
                    return Err(bad());
                }
                let (actor, cards) = match progress {
                    P::HolderAsk { candidates } => (*owner, Some(candidates)),
                    P::SourceAsk {
                        player,
                        count,
                        candidates,
                    } => {
                        if Some(*player) != context.source
                            || *count == 0
                            || *count as usize > candidates.len()
                        {
                            return Err(bad());
                        }
                        (*player, Some(candidates))
                    }
                    P::HolderPay { cards } => {
                        if cards.is_empty() {
                            return Err(bad());
                        }
                        (*owner, None)
                    }
                    P::Body
                    | P::HolderLoss { .. }
                    | P::HolderAfterLog { .. }
                    | P::SourceLoss
                    | P::JidingCard { .. }
                    | P::BodyDone
                    | P::YuanceAsk
                    | P::JidianAsk { .. }
                    | P::AwaitJudge { .. } => (*owner, None),
                    _ => return Err(bad()),
                };
                if let Some(cards) = cards {
                    let p = core.player(actor);
                    let actual: Vec<_> = p.hand.iter().chain(p.equips.iter()).copied().collect();
                    if actual != *cards || !core.player(actor).alive {
                        return Err(bad());
                    }
                }
                if let P::JidingCard { child } = progress {
                    if effect != crate::event::NativeDamageEffect::Jiding
                        || *child <= f.id
                        || *child >= res.next_event_id
                    {
                        return Err(bad());
                    }
                    if let Some(c) = res.stack.iter().find(|g| g.id == *child) {
                        if c.parent != Some(f.id)
                            || !matches!(&c.data,EventData::CardUsed(u)if u.player==*owner&&u.use_as==Some(CardKind::LeiSha))
                        {
                            return Err(bad());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
