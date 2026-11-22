//! Trigger metadata, limits, filtering, activation, machine init, and dispatch for `u01_fixes_2`.

use super::*;

pub(crate) fn trigger_meta_u01_fix2(name: &str) -> Option<TriggerMeta> {
    if matches!(
        name,
        "dddqianlong"
            | "dddlingyong"
            | "dddxuyu"
            | "dddxuyu_old"
            | "dddshijian_old"
            | "dddtuoji"
            | "dddquche"
            | "dddzhijie"
            | "dddfusi"
            | "dddfusi_global"
    ) {
        return None;
    }
    let meta = |event: EventKind, extra_events: Vec<EventKind>| TriggerMeta {
        event,
        timing: Timing::Normal,
        priority: 1,
        order_selectable: false,
        optional: false,
        extra_events,
    };
    match name {
        "dddzhengjun" => Some(meta(
            EventKind::Damage,
            vec![
                EventKind::Recover,
                EventKind::LoseHp,
                EventKind::CardsDrawn,
                EventKind::CardsDiscarded,
                EventKind::CardsLost,
                EventKind::CardsMoved,
                EventKind::Equipped,
            ],
        )),
        "dddxianxi" => Some(meta(EventKind::CardUsed, Vec::new())),
        "dddxuxiao" => Some(meta(EventKind::CardsDiscarded, vec![EventKind::CardsLost])),
        "dddshijian" => Some(meta(EventKind::CardUsed, Vec::new())),
        "dddtaisi" => Some(meta(EventKind::TurnPhaseEnd, Vec::new())),
        "ddddongcha" => Some(meta(EventKind::PhaseBegin, vec![EventKind::TurnPhaseEnd])),
        "dddqiahua" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "dddxunxun" => Some(meta(EventKind::PhaseBegin, Vec::new())),
        "dddchashi" => Some(meta(
            EventKind::PhaseBegin,
            vec![EventKind::CardUsed, EventKind::Equipped],
        )),
        "dddmiaoxing" => Some(meta(EventKind::PhaseBegin, vec![EventKind::DrawPhaseAfter])),
        "dddfenji" => Some(meta(EventKind::DrawPhaseBegin1, vec![EventKind::PhaseBegin])),
        _ => None,
    }
}

pub(crate) fn limits_u01_fix2(name: &str) -> Option<SkillLimits> {
    is_u01_fix_2_name(name).then(SkillLimits::default)
}

pub(crate) fn filter_u01_fix2(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
    event: &EventData,
) -> Option<bool> {
    if !is_u01_fix_2_name(&skill.name) {
        return None;
    }
    if !ctx.state.player(owner).alive || ctx.catalog.is_tieji_sealed(ctx.state, owner) {
        return Some(false);
    }
    let ok = match skill.name.as_str() {
        "dddzhengjun" => {
            if ctx.state.turn.current != owner || ctx.state.turn.phase != Phase::Play {
                return Some(false);
            }
            let seq = flag(ctx.state, 0, "play_phase_seq").max(1);
            match event {
                EventData::Damage(d) if !d.prevented && d.amount > 0 => {
                    flag(ctx.state, owner, "dddzhengjun_used_b1") != seq
                        && !dddzhengjun_match_targets(ctx.state, owner, d.target, &[], 1)
                            .is_empty()
                }
                EventData::Recover { player, amount, .. } if *amount > 0 => {
                    flag(ctx.state, owner, "dddzhengjun_used_b1") != seq
                        && !dddzhengjun_match_targets(ctx.state, owner, *player, &[], 1)
                            .is_empty()
                }
                EventData::LoseHp { player, amount } if *amount > 0 => {
                    flag(ctx.state, owner, "dddzhengjun_used_b1") != seq
                        && !dddzhengjun_match_targets(ctx.state, owner, *player, &[], 1)
                            .is_empty()
                }
                EventData::CardsDrawn { player, cards } if !cards.is_empty() => {
                    flag(ctx.state, owner, "dddzhengjun_used_b2") != seq
                        && !dddzhengjun_match_targets(ctx.state, owner, *player, &[], 2)
                            .is_empty()
                }
                EventData::CardsDiscarded { player, cards } if !cards.is_empty() => {
                    flag(ctx.state, owner, "dddzhengjun_used_b2") != seq
                        && !dddzhengjun_match_targets(ctx.state, owner, *player, &[], 2)
                            .is_empty()
                }
                EventData::CardsLost {
                    player,
                    cards,
                    origins,
                } if !cards.is_empty() => {
                    let b2 = origins.contains(&LostOrigin::Hand)
                        && flag(ctx.state, owner, "dddzhengjun_used_b2") != seq
                        && !dddzhengjun_match_targets(ctx.state, owner, *player, &[], 2)
                            .is_empty();
                    let b3 = origins.contains(&LostOrigin::Equip)
                        && flag(ctx.state, owner, "dddzhengjun_used_b3") != seq
                        && !dddzhengjun_match_targets(ctx.state, owner, *player, &[], 3)
                            .is_empty();
                    b2 || b3
                }
                EventData::CardsMoved { from, to, cards }
                    if from != to
                        && !cards.is_empty()
                        && !cards.iter().all(|c| {
                            ctx.state.player(*to).equips.contains(c)
                                || ctx.state.player(*to).judge.contains(c)
                        }) =>
                {
                    flag(ctx.state, owner, "dddzhengjun_used_b2") != seq
                        && !dddzhengjun_match_targets(ctx.state, owner, *to, &[*from, *to], 2)
                            .is_empty()
                }
                EventData::Equipped { player, .. } => {
                    flag(ctx.state, owner, "dddzhengjun_used_b3") != seq
                        && !dddzhengjun_match_targets(ctx.state, owner, *player, &[], 3)
                            .is_empty()
                }
                _ => false,
            }
        }
        "dddxianxi" => matches!(
            event,
            EventData::CardUsed(u)
            if u.player == owner
                && !u.cancelled
                && ctx.state.turn.current == owner
                && ctx.state.turn.phase == Phase::Play
                && u.effective_kind(ctx.catalog).is_sha_like()
                && !dddxianxi_eligible_targets(ctx.state, ctx.catalog, owner, &u.targets, u.nature).is_empty()
        ),
        "dddxuxiao" => match event {
            EventData::CardsDiscarded { cards, .. } => {
                !dddxuxiao_valid_cards(ctx.state, ctx.catalog, cards).is_empty()
            }
            EventData::CardsLost { player, cards, .. } => {
                if flag(ctx.state, *player, "in_discard_to_limit") != 0 {
                    false
                } else {
                    let disc: Vec<CardId> = cards
                        .iter()
                        .copied()
                        .filter(|&c| flag(ctx.state, *player, &format!("u01_discard_reason:{c}")) != 0)
                        .collect();
                    !dddxuxiao_valid_cards(ctx.state, ctx.catalog, &disc).is_empty()
                }
            }
            _ => false,
        },
        "dddshijian" => matches!(
            event,
            EventData::CardUsed(u)
            if !u.cancelled
                && flag(ctx.state, owner, "dddshijian_extra_target") == u32::from(u.player) + 1
                && flag(ctx.state, owner, "dddshijian_extra_seq")
                    == flag(ctx.state, 0, "play_phase_seq").max(1)
        ),
        "dddtaisi" => matches!(
            event,
            EventData::TurnEndWindow { after: false, .. }
            if flag(ctx.state, owner, "turn_change_hp") != 0
                && !dddtaisi_discard_cards(ctx.state).is_empty()
        ),
        "ddddongcha" => match event {
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } => {
                *player != owner
                    && ctx.state.player(*player).alive
                    && !ctx.state.player(owner).hand.is_empty()
                    && !ctx.state.player(*player).hand.is_empty()
            }
            EventData::TurnEndWindow {
                player,
                after: false,
            } => {
                flag(ctx.state, owner, "ddddongcha_active_turn") == u32::from(*player) + 1
                    && !ddd_jian_cards(ctx.state, owner).is_empty()
            }
            _ => false,
        },
        "dddqiahua" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Prepare,
                begin: true,
            } if *player != owner
                && ctx.state.player(*player).alive
                && ctx.state.player(*player).hp > 0
                && dddfusi_unshown_hand(ctx.state, owner).len()
                    >= ctx.state.player(*player).hp as usize
        ),
        "dddxunxun" => matches!(
            event,
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } if *player == owner && (!ctx.state.deck.is_empty() || !ctx.state.discard.is_empty())
        ),
        "dddchashi" => match event {
            EventData::Phase {
                player,
                phase: Phase::Play,
                begin: true,
            } => *player != owner && !he_cards(ctx.state, owner).is_empty(),
            EventData::CardUsed(u) => {
                !u.cancelled && chashi_card_matches(ctx.state, ctx.catalog, owner, u)
            }
            EventData::Equipped { player, card, .. } => {
                flag(ctx.state, owner, "dddchashi_armed") != 0
                    && flag(ctx.state, owner, "dddchashi_target") == u32::from(*player) + 1
                    && (card_kind_group(ctx.catalog.card_kind(*card))
                        == flag(ctx.state, owner, "dddchashi_kind")
                        || effective_suit_bit(ctx.state, ctx.catalog, *player, *card)
                            == flag(ctx.state, owner, "dddchashi_suit"))
            }
            _ => false,
        },
        "dddmiaoxing" => match event {
            EventData::Phase {
                player,
                phase: Phase::Draw,
                begin: true,
            } => {
                *player == owner
                    && flag(ctx.state, owner, "dddmiaoxing_init_done") == 0
                    && ctx.state.player(owner).hand.len() >= 3
            }
            EventData::DrawPhase {
                player,
                progress: crate::event::DrawPhaseProgress::After,
                ..
            } => {
                *player == owner
                    && flag(ctx.state, owner, "dddmiaoxing_init_done") != 0
                    && !dddmiaoxing_mismatch_tags(ctx.state, owner).is_empty()
            }
            _ => false,
        },
        "dddfenji" => match event {
            EventData::DrawPhase {
                player,
                progress: crate::event::DrawPhaseProgress::Begin2,
                ..
            } => {
                *player == owner
                    && !dddfenji_usable_tricks(ctx.state, ctx.catalog, owner).is_empty()
            }
            EventData::Phase {
                player,
                phase: Phase::Discard,
                begin: true,
            } => {
                *player == owner
                    && !dddfenji_usable_tricks(ctx.state, ctx.catalog, owner).is_empty()
            }
            _ => false,
        },
        _ => false,
    };
    Some(ok)
}

pub(crate) fn activate_u01_fix2(
    skill: &HostSkillBehavior,
    ctx: &QueryCtx<'_>,
    owner: PlayerId,
) -> Option<Option<Box<dyn crate::catalog::SkillMachine>>> {
    if !is_u01_fix_2_name(&skill.name) {
        return None;
    }
    if !ctx.state.player(owner).alive
        || ctx.state.turn.phase != Phase::Play
        || ctx.state.turn.current != owner
    {
        return Some(None);
    }
    let ok = match skill.name.as_str() {
        "dddquche" => {
            !ctx.catalog.is_tieji_sealed(ctx.state, owner)
                && !is_scope4_used(ctx.state, owner, "dddquche")
                && !dddquche_red_cards(ctx.state, ctx.catalog, owner).is_empty()
                && ctx.state.alive_players().any(|p| p != owner)
        }
        "dddfusi_global" => {
            flag(ctx.state, owner, "dddfusi_refused") == 0
                && flag(ctx.state, owner, "dddfusi_window_refused") == 0
                && !dddfusi_usable_play_cards(ctx.state, ctx.catalog, owner).is_empty()
        }
        _ => false,
    };
    Some(ok.then(|| {
        Box::new(HostMachine {
            def_key: skill.name.clone(),
            behavior: skill.behavior,
            owner,
            target: None,
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }) as Box<dyn crate::catalog::SkillMachine>
    }))
}

pub(crate) fn machine_u01_fix2(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    event: &EventData,
) -> Option<HostMachine> {
    if !is_u01_fix_2_name(&skill.name) {
        return None;
    }
    let mut target = None;
    let mut card = None;
    let mut cards = Vec::new();
    let mut targets = Vec::new();
    match (skill.name.as_str(), event) {
        ("dddzhengjun", EventData::Damage(d)) => {
            card = Some(1);
            target = Some(d.target);
        }
        ("dddzhengjun", EventData::Recover { player, .. } | EventData::LoseHp { player, .. }) => {
            card = Some(1);
            target = Some(*player);
        }
        ("dddzhengjun", EventData::CardsDrawn { player, .. } | EventData::CardsDiscarded { player, .. }) => {
            card = Some(2);
            target = Some(*player);
        }
        ("dddzhengjun", EventData::CardsLost { player, origins, .. }) => {
            card = Some(if origins.contains(&LostOrigin::Equip) { 3 } else { 2 });
            target = Some(*player);
        }
        ("dddzhengjun", EventData::CardsMoved { from, to, .. }) => {
            card = Some(2);
            target = Some(*to);
            targets = vec![*from, *to];
        }
        ("dddzhengjun", EventData::Equipped { player, .. }) => {
            card = Some(3);
            target = Some(*player);
        }
        ("dddxianxi", EventData::CardUsed(u)) => {
            targets = u.targets.clone();
        }
        ("dddxuxiao", EventData::CardsLost { cards: cs, .. } | EventData::CardsDiscarded { cards: cs, .. }) => {
            cards = cs.clone();
        }
        ("dddshijian", EventData::CardUsed(_)) => {
            card = Some(2);
        }
        ("ddddongcha", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            card = Some(0);
            target = Some(*player);
        }
        ("ddddongcha", EventData::TurnEndWindow { player, after: false }) => {
            card = Some(1);
            target = Some(*player);
        }
        ("dddqiahua", EventData::Phase { player, phase: Phase::Prepare, begin: true }) => {
            target = Some(*player);
        }
        ("dddchashi", EventData::Phase { player, phase: Phase::Play, begin: true }) => {
            card = Some(0);
            target = Some(*player);
        }
        ("dddchashi", EventData::CardUsed(u)) => {
            card = Some(1);
            target = Some(u.player);
        }
        ("dddchashi", EventData::Equipped { player, .. }) => {
            card = Some(1);
            target = Some(*player);
        }
        ("dddmiaoxing", EventData::Phase { phase: Phase::Draw, begin: true, .. }) => {
            card = Some(10);
        }
        ("dddmiaoxing", EventData::DrawPhase { .. }) => {
            card = Some(20);
        }
        ("dddfenji", EventData::DrawPhase { .. }) => {
            card = Some(1);
        }
        ("dddfenji", EventData::Phase { phase: Phase::Discard, begin: true, .. }) => {
            card = Some(2);
        }
        _ => {}
    }
    Some(HostMachine {
        def_key: skill.name.clone(),
        behavior: skill.behavior,
        owner,
        target,
        card,
        cards,
        targets,
        stage: HostStage::Act,
    })
}

pub(crate) fn step_u01_fix2(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Option<Flow> {
    if !is_u01_fix_2_name(&machine.def_key) {
        return None;
    }
    step_b1::step_b1(machine, ctx)
        .or_else(|| step_b2::step_b2(machine, ctx))
        .or_else(|| step_c1::step_c1(machine, ctx))
        .or_else(|| step_c2::step_c2(machine, ctx))
}
