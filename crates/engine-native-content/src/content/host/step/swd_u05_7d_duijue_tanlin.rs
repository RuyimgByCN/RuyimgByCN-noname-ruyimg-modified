//! U05-7d 对决（`duijue*`）与探林（`tanlin*` / `tanlin_defence`）主机步骤实现。

use super::super::*;
use super::swd_u05_7d::refund_once_used;

pub(super) fn step_duijue_tanlin(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.behavior {
        HostBehavior::SwdDuijue => match m.stage {
            HostStage::Act => {
                if ctx
                    .core
                    .vm_aux
                    .flags
                    .get(&(m.owner, "duijue_used".to_string()))
                    .copied()
                    .unwrap_or(0)
                    != 0
                {
                    return Flow::Done;
                }
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != m.owner
                            && ctx.core.player(*p).hp > 1
                            && !ctx.catalog.has_undist(ctx.core, *p)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "duijue_target",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive || t == m.owner || ctx.core.player(t).hp <= 1 {
                    return Flow::Done;
                }
                ctx.core
                    .vm_aux
                    .flags
                    .insert((m.owner, "duijue_used".to_string()), 1);
                ctx.core
                    .vm_aux
                    .flags
                    .insert((m.owner, "duijue_pending_target".to_string()), u32::from(t) + 1);
                if let Some(d3_def) = ctx.catalog.skill_by_behavior(HostBehavior::SwdDuijueTrigger) {
                    ctx.insert_skill(
                        (m.owner, d3_def),
                        crate::state::SkillInstance {
                            owner: m.owner,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "对决",
                    owner: m.owner,
                    presentation: None,
                });
                ctx.core.turn.skip_play_phase = true;
                if ctx.core.turn.phase == crate::event::Phase::Play {
                    ctx.emit(EventData::Phase {
                        player: ctx.core.turn.current,
                        phase: crate::event::Phase::Play,
                        begin: false,
                    });
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdDuijueTrigger => {
            if let Some(d3_def) = ctx.catalog.skill_by_behavior(HostBehavior::SwdDuijueTrigger) {
                ctx.core.skills.remove(&(m.owner, d3_def));
            }
            let Some(t) = ctx
                .core
                .vm_aux
                .flags
                .remove(&(m.owner, "duijue_pending_target".to_string()))
                .and_then(|v| v.checked_sub(1))
                .map(|v| v as PlayerId)
            else {
                return Flow::Done;
            };
            if !ctx.core.player(t).alive || !ctx.core.player(m.owner).alive {
                return Flow::Done;
            }
            let d2_def = ctx.catalog.skill_by_behavior(HostBehavior::SwdDuijueOut);
            let alive: Vec<PlayerId> = ctx.core.alive_players().collect();
            for p in alive {
                if p != m.owner && p != t {
                    if let Some(def) = d2_def {
                        ctx.insert_skill(
                            (p, def),
                            crate::state::SkillInstance {
                                owner: p,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: false,
                                rounds: 0,
                            },
                        );
                    }
                    ctx.core
                        .vm_aux
                        .flags
                        .insert((p, "duijue_out_active".to_string()), 1);
                }
            }
            ctx.core
                .vm_aux
                .flags
                .insert((0, "duijue_loop_remaining".to_string()), 6);
            ctx.core
                .vm_aux
                .flags
                .insert((0, "duijue_in_duel_turn".to_string()), 0);
            ctx.core
                .vm_aux
                .flags
                .insert((0, "duijue_duelist_a".to_string()), u32::from(t) + 1);
            ctx.core
                .vm_aux
                .flags
                .insert((0, "duijue_duelist_b".to_string()), u32::from(m.owner) + 1);
            ctx.core
                .vm_aux
                .flags
                .insert((0, "duijue_return_after".to_string()), u32::from(m.owner) + 1);
            Flow::Done
        }
        HostBehavior::SwdDuijueOut => {
            if let Some(d2_def) = ctx.catalog.skill_by_behavior(HostBehavior::SwdDuijueOut) {
                ctx.core.skills.remove(&(m.owner, d2_def));
            }
            ctx.core
                .vm_aux
                .flags
                .remove(&(m.owner, "duijue_out_active".to_string()));
            Flow::Done
        }
        HostBehavior::SwdTanlin => match m.stage {
            HostStage::Act => {
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != m.owner
                            && !ctx.core.player(*p).hand.is_empty()
                            && !ctx.catalog.has_undist(ctx.core, *p)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if options.is_empty() || ctx.core.player(m.owner).hand.is_empty() {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.stage = HostStage::TargetChoice;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "tanlin_target",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::TargetChoice => {
                let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive
                    || ctx.core.player(t).hand.is_empty()
                    || ctx.core.player(m.owner).hand.is_empty()
                {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.target = Some(t);
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "tanlin_pindian",
                        options: ctx
                            .core
                            .player(m.owner)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect(),
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let Some(ChoiceOption::Card(c_owner)) = ctx.answer().first().copied() else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                let Some(t) = m.target else {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                };
                if !ctx.core.player(m.owner).hand.contains(&c_owner)
                    || ctx.core.player(t).hand.is_empty()
                {
                    refund_once_used(ctx, m.owner, &m.def_key);
                    return Flow::Done;
                }
                m.card = Some(c_owner);
                m.stage = HostStage::CardsAsk;
                Flow::WaitChoice {
                    observer: t,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "tanlin_target_pindian",
                        options: ctx
                            .core
                            .player(t)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect(),
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            HostStage::CardsAsk => {
                let (Some(t), Some(c_owner)) = (m.target, m.card) else {
                    return Flow::Done;
                };
                let Some(ChoiceOption::Card(c_target)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(m.owner).hand.contains(&c_owner)
                    || !ctx.core.player(t).hand.contains(&c_target)
                {
                    return Flow::Done;
                }
                ctx.take_hand_card(m.owner, c_owner);
                ctx.take_hand_card(t, c_target);
                let r_owner = ctx.catalog.cards[c_owner as usize].rank;
                let r_target = ctx.catalog.cards[c_target as usize].rank;
                ctx.log(LogEvent::SkillTriggered {
                    skill: "探林",
                    owner: m.owner,
                    presentation: None,
                });
                if r_owner > r_target {
                    let already_marked = ctx.catalog.has_host_behavior(
                        ctx.core,
                        t,
                        HostBehavior::SwdTanlinTarget,
                    );
                    if !already_marked {
                        if let Some(t2_def) =
                            ctx.catalog.skill_by_behavior(HostBehavior::SwdTanlinTarget)
                        {
                            ctx.insert_skill(
                                (t, t2_def),
                                crate::state::SkillInstance {
                                    owner: t,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                        if let Some(t3_def) =
                            ctx.catalog.skill_by_behavior(HostBehavior::SwdTanlinOwner)
                        {
                            ctx.insert_skill(
                                (m.owner, t3_def),
                                crate::state::SkillInstance {
                                    owner: m.owner,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                        ctx.note_cards_lost(m.owner, &[c_owner]);
                        ctx.emit(EventData::CardsLost {
                            player: m.owner,
                            cards: vec![c_owner],
                            origins: vec![crate::event::LostOrigin::Hand],
                        });
                        ctx.note_cards_lost(t, &[c_target]);
                        ctx.emit(EventData::CardsLost {
                            player: t,
                            cards: vec![c_target],
                            origins: vec![crate::event::LostOrigin::Hand],
                        });
                        ctx.core.player_mut(m.owner).hand.push(c_owner);
                        ctx.core.player_mut(m.owner).hand.push(c_target);
                        ctx.log(LogEvent::CardsGiven {
                            from: t,
                            to: m.owner,
                            cards: vec![c_target],
                        });
                    } else {
                        ctx.to_discard(c_owner);
                        ctx.to_discard(c_target);
                    }
                } else {
                    ctx.to_discard(c_owner);
                    ctx.to_discard(c_target);
                    if ctx.core.player(m.owner).alive && ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(t),
                            m.owner,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        HostBehavior::SwdTanlinTarget => {
            if let Some(def) = ctx.catalog.skill_by_behavior(HostBehavior::SwdTanlinTarget) {
                ctx.core.skills.remove(&(m.owner, def));
            }
            Flow::Done
        }
        HostBehavior::SwdTanlinOwner => {
            if let Some(def) = ctx.catalog.skill_by_behavior(HostBehavior::SwdTanlinOwner) {
                ctx.core.skills.remove(&(m.owner, def));
            }
            Flow::Done
        }
        HostBehavior::SwdTanlinDefence => match m.stage {
            HostStage::Act => {
                let src = ctx
                    .res
                    .frame(ctx.current_event_frame_id())
                    .map(|f| &f.data)
                    .and_then(|d| match d {
                        EventData::Damage(dmg) => dmg.source,
                        _ => None,
                    });
                let Some(src) = src else {
                    return Flow::Done;
                };
                if !ctx.core.player(m.owner).alive || !ctx.core.player(src).alive {
                    return Flow::Done;
                }
                let src_hand = ctx.core.player(src).hand.len();
                let my_hand = ctx.core.player(m.owner).hand.len();
                let max = src_hand.min(my_hand);
                if max == 0 {
                    return Flow::Done;
                }
                m.target = Some(src);
                m.stage = HostStage::SelectCard;
                Flow::WaitChoice {
                    observer: m.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "tanlin_defence_discard",
                        options: ctx
                            .core
                            .player(m.owner)
                            .hand
                            .iter()
                            .copied()
                            .map(ChoiceOption::Card)
                            .collect(),
                        min: 1,
                        max,
                        allow_cancel: true,
                    },
                }
            }
            HostStage::SelectCard => {
                let picks: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picks.is_empty() {
                    return Flow::Done;
                }
                let Some(src) = m.target else {
                    return Flow::Done;
                };
                ctx.log(LogEvent::SkillTriggered {
                    skill: "探林",
                    owner: m.owner,
                    presentation: None,
                });
                let mut count = 0usize;
                for c in picks {
                    if ctx.core.player(m.owner).hand.contains(&c) {
                        ctx.discard_from_hand(m.owner, c);
                        count += 1;
                    }
                }
                if count == 0 {
                    return Flow::Done;
                }
                if ctx.core.player(src).alive {
                    let mut pool: Vec<CardId> = ctx.core.player(src).hand.iter().copied().collect();
                    let take = count.min(pool.len());
                    for _ in 0..take {
                        let idx = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let c = pool.swap_remove(idx);
                        if ctx.core.player(src).hand.contains(&c) {
                            ctx.discard_from_hand(src, c);
                        }
                    }
                }
                if ctx.core.player(m.owner).alive {
                    ctx.start_draw(m.owner, 1);
                }
                Flow::Done
            }
            _ => Flow::Done,
        },
        _ => Flow::Done,
    }
}
