//! Finite native bodies run inside an actual named invocation in the ordinary stack.
use super::*;
use crate::event::{
    InvocationData, InvocationPhase as PhaseOfSkill, NativeDamageEffect as Effect,
    NativeInvocationProgress as P,
};
#[derive(Clone)]
pub struct NativeInvocation {
    pub owner: PlayerId,
    pub def: crate::ids::SkillDefId,
    pub window: crate::event::DamageWindowKind,
    pub progress: P,
}
pub fn advance_invocation(frame: &mut crate::event::Frame) {
    let EventData::SkillInvocation(data) = &mut frame.data else {
        return;
    };
    let next = match data.phase {
        PhaseOfSkill::Before if data.cancelled => Some(PhaseOfSkill::Omitted),
        PhaseOfSkill::Before => Some(PhaseOfSkill::Begin),
        PhaseOfSkill::Begin if data.cancelled => Some(PhaseOfSkill::End),
        PhaseOfSkill::Begin => Some(PhaseOfSkill::Body),
        PhaseOfSkill::Body => Some(PhaseOfSkill::End),
        PhaseOfSkill::End => Some(PhaseOfSkill::After),
        PhaseOfSkill::After | PhaseOfSkill::Omitted => None,
    };
    if let Some(phase) = next {
        data.phase = phase;
        frame.kind = data.phase.kind();
        frame.stage = if phase == PhaseOfSkill::Body {
            crate::event::Stage::Intrinsic
        } else {
            crate::event::Stage::Commit
        };
        frame.planned = false;
        frame.intrinsic_done = false;
    } else {
        frame.stage = crate::event::Stage::Done;
    }
}
impl NativeInvocation {
    fn effect(&self, ctx: &Ctx<'_>) -> Effect {
        ctx.catalog.skills[self.def as usize]
            .behavior
            .native_damage_effect(self.window.event_kind())
            .expect("validated admitted definition")
    }
    fn emit_invocation(&mut self, ctx: &mut Ctx<'_>, context: InvocationData) -> Flow {
        self.progress = P::AwaitInvocation {
            child: ctx.res.next_event_id,
        };
        ctx.emit(EventData::SkillInvocation(context));
        Flow::Yield
    }
    fn log_child(&self, ctx: &mut Ctx<'_>, begin: bool) {
        ctx.emit(EventData::SkillLog {
            owner: self.owner,
            def: self.def,
            begin,
        });
    }
    fn he(ctx: &Ctx<'_>, player: PlayerId) -> Vec<crate::ids::CardId> {
        let p = ctx.core.player(player);
        p.hand.iter().chain(p.equips.iter()).copied().collect()
    }
    fn choice(player: PlayerId, cards: &[crate::ids::CardId], count: Option<u32>) -> Flow {
        Flow::WaitChoice {
            observer: player,
            spec: ChoiceSpec {
                kind: ChoiceKind::SelectCards,
                prompt: if count.is_some() {
                    "c331_zhangwu_source"
                } else {
                    "c331_zhangwu_holder"
                },
                options: cards.iter().copied().map(ChoiceOption::Card).collect(),
                min: count.map(|n| n as usize).unwrap_or(1),
                max: count.map(|n| n as usize).unwrap_or(cards.len()),
                allow_cancel: true,
            },
        }
    }
    fn selected(ctx: &Ctx<'_>, candidates: &[crate::ids::CardId]) -> Vec<crate::ids::CardId> {
        ctx.answer()
            .iter()
            .filter_map(|a| match a {
                ChoiceOption::Card(c) if candidates.contains(c) => Some(*c),
                _ => None,
            })
            .collect()
    }
    fn discard(ctx: &mut Ctx<'_>, player: PlayerId, cards: &[crate::ids::CardId]) {
        let mut discarded = Vec::new();
        for &card in cards {
            let p = ctx.core.player(player);
            if p.hand.contains(&card) || p.equips.contains(&card) {
                ctx.take_owned_card(player, card);
                ctx.to_discard(card);
                discarded.push(card);
            }
        }
        if !discarded.is_empty() {
            ctx.log(LogEvent::CardDiscarded {
                player,
                cards: discarded.clone(),
            });
            ctx.emit(EventData::CardsDiscarded {
                player,
                cards: discarded,
            });
        }
    }
    fn start_judge(
        &mut self,
        ctx: &mut Ctx<'_>,
        target: PlayerId,
        purpose: crate::event::JudgePurpose,
        log: bool,
    ) -> Flow {
        let parent = ctx.current_frame_id();
        let child = ctx.res.next_event_id;
        self.progress = P::AwaitJudge {
            child,
            target,
            purpose,
        };
        ctx.emit(EventData::OrdinaryJudge {
            owner: target,
            invocation: parent,
        });
        if log {
            let judge = ctx.res.stack.pop().expect("just emitted judge");
            ctx.log(LogEvent::SkillTriggered {
                skill: ctx.catalog.skill_name(self.def),
                owner: self.owner,
            });
            self.log_child(ctx, true);
            ctx.res.stack.push(judge);
            if let EventData::SkillInvocation(d) = &mut ctx.res.frame_mut(parent).unwrap().data {
                d.deferred_log = true;
            }
        }
        Flow::Yield
    }
    fn body(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let id = ctx.current_frame_id();
        let EventData::SkillInvocation(context) =
            ctx.res.frame(id).expect("invocation").data.clone()
        else {
            return Flow::Done;
        };
        match self.progress.clone() {
            P::Body => {
                if !ctx.core.player(self.owner).alive {
                    return Flow::Done;
                }
                match self.effect(ctx) {
                    Effect::Zhangwu => {
                        let cards = Self::he(ctx, self.owner);
                        if cards.is_empty() {
                            return Flow::Done;
                        }
                        self.progress = P::HolderAsk {
                            candidates: cards.clone(),
                        };
                        Self::choice(self.owner, &cards, None)
                    }
                    Effect::Jiding => {
                        let Some(source) = context.source.filter(|p| ctx.core.player(*p).alive)
                        else {
                            return Flow::Done;
                        };
                        if ctx.core.deck.is_empty() {
                            ctx.res.fault = Some(crate::machine::VmFault {
                                skill: ctx.catalog.skill_name(self.def).to_owned(),
                                owner: self.owner,
                                event: "SkillInvocation".into(),
                                kind: "NativeVirtualCarrierUnavailable".into(),
                                detail: "existing virtual-card carrier contract has no deck card"
                                    .into(),
                            });
                            return Flow::Done;
                        }
                        let child = ctx.res.next_event_id;
                        if let EventData::SkillInvocation(d) =
                            &mut ctx.res.frame_mut(id).unwrap().data
                        {
                            d.jiding_card = Some(child);
                        }
                        self.progress = P::JidingCard { child };
                        ctx.use_virtual_card(self.owner, CardKind::LeiSha, vec![source]);
                        Flow::Yield
                    }
                    Effect::Yuance => {
                        let _ = ctx.core.rng.gen_range(2);
                        self.progress = P::YuanceAsk;
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::Control,
                                prompt: "c331_yuance",
                                options: vec![
                                    ChoiceOption::Control(crate::event::YuanceOption::LoseHp),
                                    ChoiceOption::Control(crate::event::YuanceOption::Recover),
                                ],
                                min: 1,
                                max: 1,
                                allow_cancel: true,
                            },
                        }
                    }
                    Effect::Jidian => {
                        let targets: Vec<_> = ctx
                            .core
                            .alive_players()
                            .filter(|p| {
                                *p != context.target
                                    && ctx.core.computed_distance(ctx.catalog, context.target, *p)
                                        <= 1
                            })
                            .collect();
                        if targets.is_empty() {
                            return Flow::Done;
                        }
                        self.progress = P::JidianAsk {
                            targets: targets.clone(),
                        };
                        Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: "c331_jidian_target",
                                options: targets.into_iter().map(ChoiceOption::Player).collect(),
                                min: 1,
                                max: 1,
                                allow_cancel: true,
                            },
                        }
                    }
                    _ => unreachable!("only finite implemented invocation bodies"),
                }
            }
            P::HolderAsk { candidates } => {
                let cards = Self::selected(ctx, &candidates);
                if cards.is_empty() {
                    return Flow::Done;
                }
                self.progress = P::HolderPay { cards };
                ctx.log(LogEvent::SkillTriggered {
                    skill: ctx.catalog.skill_name(self.def),
                    owner: self.owner,
                });
                self.log_child(ctx, true);
                Flow::Yield
            }
            P::HolderPay { cards } => {
                self.progress = P::HolderLoss {
                    count: cards.len() as u32,
                };
                if ctx.core.player(self.owner).alive {
                    Self::discard(ctx, self.owner, &cards);
                }
                Flow::Yield
            }
            P::HolderLoss { count } => {
                self.progress = P::HolderAfterLog { count };
                self.log_child(ctx, false);
                Flow::Yield
            }
            P::HolderAfterLog { count } => {
                if !ctx.core.player(self.owner).alive {
                    return Flow::Done;
                }
                let Some(source) = context.source.filter(|p| ctx.core.player(*p).alive) else {
                    return Flow::Done;
                };
                let cards = Self::he(ctx, source);
                if cards.len() < count as usize {
                    self.progress = P::SourceLoss;
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(self.owner),
                        source,
                        i32::try_from(count).expect("validated bounded card payment"),
                    )));
                    Flow::Yield
                } else {
                    self.progress = P::SourceAsk {
                        player: source,
                        count,
                        candidates: cards.clone(),
                    };
                    Self::choice(source, &cards, Some(count))
                }
            }
            P::SourceAsk {
                count, candidates, ..
            } => {
                let Some(source) = context.source else {
                    return Flow::Done;
                };
                self.progress = P::SourceLoss;
                if !ctx.core.player(self.owner).alive || !ctx.core.player(source).alive {
                    return Flow::Done;
                }
                let cards = Self::selected(ctx, &candidates);
                if cards.is_empty() {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(self.owner),
                        source,
                        i32::try_from(count).expect("validated bounded card payment"),
                    )));
                } else {
                    Self::discard(ctx, source, &cards);
                }
                Flow::Yield
            }
            P::YuanceAsk => {
                let Some(ChoiceOption::Control(option)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                self.start_judge(
                    ctx,
                    self.owner,
                    crate::event::JudgePurpose::Yuance(option),
                    false,
                )
            }
            P::JidianAsk { targets } => {
                let Some(ChoiceOption::Player(target)) = ctx
                    .answer()
                    .first()
                    .copied()
                    .filter(|p| matches!(p,ChoiceOption::Player(t)if targets.contains(t)))
                else {
                    return Flow::Done;
                };
                self.start_judge(ctx, target, crate::event::JudgePurpose::Jidian, true)
            }
            P::AwaitJudge {
                target, purpose, ..
            } => {
                self.progress = P::BodyDone;
                let colour = context.judgement;
                if let EventData::SkillInvocation(d) = &mut ctx.res.frame_mut(id).unwrap().data {
                    d.judgement = None;
                }
                if !ctx.core.player(self.owner).alive {
                    return Flow::Done;
                }
                let Some(suit) = colour else {
                    return Flow::Done;
                };
                let black = matches!(
                    suit,
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                );
                match purpose {
                    crate::event::JudgePurpose::Jidian => {
                        if black && ctx.core.player(target).alive {
                            let mut d = crate::machine::damage_data(Some(self.owner), target, 1);
                            d.nature = crate::event::DamageNature::Thunder;
                            ctx.emit(EventData::Damage(d));
                        }
                    }
                    crate::event::JudgePurpose::Yuance(option) => {
                        let Some(source) = context.source else {
                            return Flow::Done;
                        };
                        let actor = if matches!(option, crate::event::YuanceOption::LoseHp) == black
                        {
                            context.target
                        } else {
                            source
                        };
                        match option {
                            crate::event::YuanceOption::LoseHp => ctx.emit(EventData::LoseHp {
                                player: actor,
                                amount: 1,
                            }),
                            crate::event::YuanceOption::Recover => {
                                ctx.recover(actor, 1, None);
                            }
                        }
                    }
                }
                Flow::Yield
            }
            P::BodyDone | P::SourceLoss => Flow::Done,
            P::JidingCard { .. } => {
                self.progress = P::SourceLoss;
                if ctx.core.player(self.owner).alive {
                    if ctx.core.skills.remove(&(self.owner, self.def)).is_some() {
                        ctx.log(LogEvent::SkillToggled {
                            player: self.owner,
                            skill: ctx.catalog.skill_name(self.def),
                            enabled: false,
                        });
                        ctx.emit(EventData::SkillToggled {
                            player: self.owner,
                            skill: self.def,
                            enabled: false,
                            temporary: false,
                        });
                    }
                }
                Flow::Yield
            }
            _ => Flow::Done,
        }
    }
}
impl SkillMachine for NativeInvocation {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::NativeInvocation {
            owner: self.owner,
            def: self.def,
            window: self.window,
            progress: self.progress.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.progress.clone() {
            P::Ready => {
                let EventData::DamageWindow { root, .. } = ctx
                    .res
                    .frame(ctx.current_frame_id())
                    .expect("trigger window")
                    .data
                else {
                    return Flow::Done;
                };
                let d = ctx.res.frame(root).expect("damage root").damage();
                let context = InvocationData {
                    owner: self.owner,
                    def: self.def,
                    window: ctx.current_frame_id(),
                    source: d.source,
                    target: d.target,
                    phase: PhaseOfSkill::Before,
                    cancelled: false,
                    jiding_card: None,
                    judgement: None,
                    deferred_log: false,
                };
                if self.effect(ctx) == Effect::Jiding {
                    self.progress = P::BeforeInvocation { context };
                    self.log_child(ctx, true);
                    Flow::Yield
                } else {
                    self.emit_invocation(ctx, context)
                }
            }
            P::BeforeInvocation { context } => self.emit_invocation(ctx, context),
            P::AwaitInvocation { .. } => {
                self.progress = P::Finished;
                if self.effect(ctx) == Effect::Jiding {
                    self.log_child(ctx, false);
                    Flow::Yield
                } else {
                    Flow::Done
                }
            }
            P::Finished => Flow::Done,
            _ => self.body(ctx),
        }
    }
}

pub fn jiding_recovery_origin(
    frames: &[crate::event::Frame],
    root: &crate::event::Frame,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
) -> bool {
    let Some(card_frame) = root
        .parent
        .and_then(|id| frames.iter().find(|f| f.id == id))
    else {
        return false;
    };
    let EventData::CardUsed(card) = &card_frame.data else {
        return false;
    };
    let Some(inv) = card_frame
        .parent
        .and_then(|id| frames.iter().find(|f| f.id == id))
    else {
        return false;
    };
    let EventData::SkillInvocation(context) = &inv.data else {
        return false;
    };
    context.owner == owner
        && card.player == owner
        && card.use_as == Some(CardKind::LeiSha)
        && context.phase == PhaseOfSkill::Body
        && context.jiding_card == Some(card_frame.id)
        && catalog.skills.get(context.def as usize).and_then(|s| {
            s.behavior
                .native_damage_effect(crate::event::EventKind::DamageEnd)
        }) == Some(Effect::Jiding)
}
