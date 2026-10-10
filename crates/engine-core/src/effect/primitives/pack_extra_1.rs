//! Effect primitive implementations — extra 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_then_optionally_distribute_hand_cards(
    tx: &mut Transaction,
    player: PlayerId,
    draw_count: u32,
    give_cap: u32,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
    finish: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if give_cap == 0 || give_cap > u8::MAX as u32 {
        return Err(EffectError::ContentFault(
            "optional hand-distribution cap must be in 1..=255".into(),
        ));
    }
    if cards.len() != targets.len() || cards.len() > give_cap as usize {
        return Err(EffectError::EngineFault(
            "invalid optional hand-distribution state".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if !ready_to_choose {
        if finish || !cards.is_empty() {
            return Err(EffectError::EngineFault(
                "optional hand distribution started with internal state".into(),
            ));
        }
        let events = if draw_count == 0 {
            Vec::new()
        } else {
            draw_cards(tx, player, draw_count)?
        };
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DrawThenOptionallyDistributeHandCards {
                player,
                draw_count,
                give_cap,
                cards,
                targets,
                skill,
                ready_to_choose: true,
                finish: false,
            },
        );
        return Ok(events);
    }
    if finish {
        return finish_optional_hand_distribution(tx, player, give_cap, cards, targets);
    }

    let planned: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    if planned.len() != cards.len()
        || cards
            .iter()
            .any(|card| !hand_cards_sorted(tx, player).contains(card))
    {
        return Err(EffectError::Illegal(
            "optional hand-distribution plan became stale".into(),
        ));
    }
    let options: Vec<_> = hand_cards_sorted(tx, player)
        .into_iter()
        .filter(|card| !planned.contains(card))
        .collect();
    let has_recipient = tx
        .state()
        .players
        .iter_live()
        .any(|target| target.alive && target.id != player);
    if cards.len() == give_cap as usize || options.is_empty() || !has_recipient {
        return finish_optional_hand_distribution(tx, player, give_cap, cards, targets);
    }

    let mut payload = vec![0, player.raw(), give_cap, skill.raw(), cards.len() as u32];
    for (card, target) in cards.iter().zip(&targets) {
        payload.push(card.raw());
        payload.push(target.raw());
    }
    open_choice(
        tx,
        player,
        1,
        (give_cap as usize - cards.len()).min(options.len()) as u8,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DrawThenOptionallyDistributeHandCards,
        payload,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn give_selected_then_continue_distinct_recover_at_two(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    mut used_targets: Vec<PlayerId>,
    total_given: u32,
    skill: engine_api::SkillId,
    continue_only: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let player_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    if !player_alive || tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
        return Ok(Vec::new());
    }
    let mut distinct_targets = std::collections::BTreeSet::new();
    if used_targets
        .iter()
        .any(|used| *used == player || !distinct_targets.insert(used.raw()))
    {
        return Err(EffectError::EngineFault(
            "invalid distinct phase-gift target history".into(),
        ));
    }

    if continue_only {
        if target.is_some() || !cards.is_empty() {
            return Err(EffectError::EngineFault(
                "distinct phase-gift continuation contains a batch".into(),
            ));
        }
        let options = hand_cards_sorted(tx, player);
        let has_target = tx.state().players.iter_live().any(|candidate| {
            candidate.alive
                && candidate.id != player
                && !distinct_targets.contains(&candidate.id.raw())
        });
        if options.is_empty() || !has_target {
            return Ok(Vec::new());
        }
        let mut payload = vec![
            0,
            player.raw(),
            total_given,
            skill.raw(),
            used_targets.len() as u32,
        ];
        payload.extend(used_targets.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            player,
            1,
            options.len().min(u8::MAX as usize) as u8,
            true,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::GiveSelectedThenContinueDistinctRecoverAtTwo,
            payload,
        );
    }

    let initial = total_given == 0 && used_targets.is_empty();
    if (initial && !originating_skill_is_enabled(tx, player, skill))
        || target == player
        || used_targets.contains(&target)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || cards.is_empty()
    {
        return Err(EffectError::Illegal(
            "invalid distinct phase-gift batch context".into(),
        ));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("phase-gift hand zone missing".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("phase-gift target hand missing".into()))?;
    let mut distinct_cards = std::collections::BTreeSet::new();
    if cards.iter().any(|card| {
        !distinct_cards.insert(card.raw())
            || !tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player && state.zone == from)
    }) {
        return Err(EffectError::Illegal(
            "distinct phase-gift cards are duplicated or stale".into(),
        ));
    }

    let current = total_given.saturating_add(cards.len() as u32);
    let events = give_cards_as(tx, player, cards, to, from)?;
    used_targets.push(target);
    if total_given < 2 && current >= 2 {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::RecoverHp {
                source: player,
                player,
                amount: 1,
            });
    }
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
            player,
            target: PlayerId::NONE,
            cards: Vec::new(),
            used_targets,
            total_given: current,
            skill,
            continue_only: true,
        },
    );
    Ok(events)
}

/// ABI 679 (batch 172): active selection — give the chosen cards to `target`, then the
/// **chooser** recovers `amount` HP (jun_renwang 交牌回自己族).
pub(crate) fn give_selected_then_recover_self(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || amount == 0
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
        || cards.is_empty()
    {
        return Err(EffectError::Illegal(
            "invalid selected gift and self-recovery context".into(),
        ));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("gift owner has no hand".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("gift target has no hand".into()))?;
    let mut seen = std::collections::BTreeSet::new();
    if cards.iter().any(|card| {
        !seen.insert(card.raw())
            || !tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player && state.zone == from)
    }) {
        return Err(EffectError::Illegal(
            "gift cards must be distinct current hand cards".into(),
        ));
    }
    let events = give_cards_as(tx, player, cards, to, from)?;
    let mut all = events;
    all.extend(super::run_effects(
        tx,
        vec![EffectRequest::RecoverHp {
            source: player,
            player,
            amount,
        }],
    )?);
    Ok(all)
}

pub(crate) fn give_selected_then_recover_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    amount: u32,
    skill: engine_api::SkillId,
    ready_to_recover: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ready_to_recover {
        return if tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            recover_hp(tx, player, target, amount)
        } else {
            Ok(Vec::new())
        };
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || amount == 0
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
        || cards.is_empty()
    {
        return Err(EffectError::Illegal(
            "invalid selected gift and recovery context".into(),
        ));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("gift owner has no hand".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("gift target has no hand".into()))?;
    let mut seen = std::collections::BTreeSet::new();
    if cards.iter().any(|card| {
        !seen.insert(card.raw())
            || !tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player && state.zone == from)
    }) {
        return Err(EffectError::Illegal(
            "gift cards must be distinct current hand cards".into(),
        ));
    }
    let events = give_cards_as(tx, player, cards, to, from)?;
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::GiveSelectedThenRecoverTarget {
            player,
            target,
            cards: Vec::new(),
            amount,
            skill,
            ready_to_recover: true,
        },
    );
    Ok(events)
}

pub(crate) fn resolve_dingpan(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || ![player, target].into_iter().all(|participant| {
                    tx.state()
                        .players
                        .get(participant)
                        .is_some_and(|state| state.alive)
                })
                || equipped_cards(tx, target).is_empty()
            {
                return Err(EffectError::Illegal("invalid Dingpan context".into()));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDingpan {
                    player,
                    target,
                    stage: 1,
                    skill,
                });
            super::run_effects(
                tx,
                vec![EffectRequest::DrawCardsFromSkill {
                    player: target,
                    count: 1,
                    skill,
                }],
            )
            .map(|_| Vec::new())
        }
        1 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::ChooseDingpanBranch,
                vec![player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let source = if tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                player
            } else {
                PlayerId::NONE
            };
            super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| Vec::new())
        }
        _ => Err(EffectError::EngineFault("bad Dingpan stage".into())),
    }
}

pub(crate) fn resolve_tanlu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    amount: u32,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let state = tx.state();
            let phase_begin = state.turn.current == target
                && (state.turn.phase == Phase::Idle
                    || (state.turn.phase == Phase::Prepare
                        && state.turn.flag(turn_flag::EXPLICIT_TURN_BEGIN_EVENT) == 0));
            if amount != 0
                || !phase_begin
                || player == target
                || !originating_skill_is_enabled(tx, player, skill)
                || ![player, target].into_iter().all(|participant| {
                    state
                        .players
                        .get(participant)
                        .is_some_and(|participant| participant.alive)
                })
            {
                return Err(EffectError::Illegal("invalid Tanlu context".into()));
            }
            let frozen = state
                .players
                .get(player)
                .expect("validated Tanlu owner")
                .hp
                .abs_diff(
                    state
                        .players
                        .get(target)
                        .expect("validated Tanlu target")
                        .hp,
                );
            if frozen == 0 || frozen > state.hand_count(target) {
                return super::run_effects(
                    tx,
                    vec![EffectRequest::ResolveTanlu {
                        player,
                        target,
                        amount: frozen,
                        stage: 1,
                        skill,
                    }],
                )
                .map(|_| Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::ResolveTanlu,
                vec![0, player.raw(), target.raw(), frozen, skill.raw()],
            )
        }
        1 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTanlu {
                    player,
                    target,
                    amount,
                    stage: 2,
                    skill,
                });
            super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| Vec::new())
        }
        2 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_target_cards(
                tx.state(),
                target,
                player,
                rule_api::TargetCardScope::Hand,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTanlu,
                vec![2, player.raw(), target.raw(), amount, skill.raw()],
            )
        }
        _ => Err(EffectError::EngineFault("bad Tanlu stage".into())),
    }
}

pub(crate) fn choose_max_mark_other_judge_kill_unless_tao_or_taoyuan(
    tx: &mut Transaction,
    player: PlayerId,
    mark: engine_api::MarkDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if mark.is_none() || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mark_count = |candidate: &Player| {
        candidate
            .marks
            .iter()
            .find(|(definition, _)| *definition == mark.raw())
            .map_or(0, |(_, count)| *count)
    };
    let max = tx
        .state()
        .players
        .iter_live()
        .filter(|candidate| candidate.alive && candidate.id != player)
        .map(mark_count)
        .max()
        .unwrap_or(0);
    if max <= 0 {
        return Ok(Vec::new());
    }
    let options = tx
        .state()
        .players
        .iter_live()
        .filter(|candidate| {
            candidate.alive && candidate.id != player && mark_count(candidate) == max
        })
        .map(|candidate| candidate.id.raw())
        .collect();
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan,
        vec![player.raw(), mark.raw(), skill.raw()],
    )
}

pub(crate) fn choose_zone_priority_gain_random_card_from_each_other_then_turn_over(
    tx: &mut Transaction,
    player: PlayerId,
    priority: u8,
    mut remaining_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if priority == u8::MAX {
        if !originating_skill_is_enabled(tx, player, skill)
            || !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
        {
            return Ok(Vec::new());
        }
        let has_cards = tx.state().players.iter_live().any(|target| {
            target.alive
                && target.id != player
                && [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                    .into_iter()
                    .any(|kind| {
                        tx.state()
                            .zones
                            .find(kind, target.id)
                            .and_then(|zone| tx.state().zones.get(zone))
                            .is_some_and(|zone| !zone.cards.is_empty())
                    })
        });
        if !has_cards {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1, 2],
            ChoiceKind::Custom,
            ContinuationKind::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver,
            vec![player.raw(), skill.raw()],
        );
    }
    if priority > 2
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let Some(target) = remaining_targets.first().copied() else {
        return turn_over(tx, player);
    };
    remaining_targets.remove(0);
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
            player,
            priority,
            remaining_targets,
            skill,
        },
    );
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let order = match priority {
        0 => [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge],
        1 => [ZoneKind::Equip, ZoneKind::Hand, ZoneKind::Judge],
        2 => [ZoneKind::Judge, ZoneKind::Hand, ZoneKind::Equip],
        _ => unreachable!("priority validated"),
    };
    let Some((from, cards)) = order.into_iter().find_map(|kind| {
        let zone = tx.state().zones.find(kind, target)?;
        let cards = tx.state().zones.get(zone)?.cards.clone();
        (!cards.is_empty()).then_some((zone, cards))
    }) else {
        return Ok(Vec::new());
    };
    let max = u32::try_from(cards.len())
        .map_err(|_| EffectError::EngineFault("zone exceeds deterministic RNG range".into()))?;
    let mut rng = RuleRng::from_state(tx.state().rng);
    let card = cards[rng.gen_range_u32(max) as usize];
    tx.state_mut().rng = rng.into_state();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("Guixin owner hand zone missing".into()))?;
    move_cards(tx, vec![card], hand, Some(from))
}

pub(crate) fn choose_gain_card_from_each_other_then_turn_over(
    tx: &mut Transaction,
    player: PlayerId,
    initialized: bool,
    remaining_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_gain_card_from_each_other(tx, player, initialized, remaining_targets, skill, true)
}

pub(crate) fn choose_gain_card_from_each_other_replace_draw(
    tx: &mut Transaction,
    player: PlayerId,
    initialized: bool,
    remaining_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_gain_card_from_each_other(tx, player, initialized, remaining_targets, skill, false)
}

pub(crate) fn choose_gain_card_from_each_other(
    tx: &mut Transaction,
    player: PlayerId,
    mut initialized: bool,
    mut remaining_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
    turn_over_at_end: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if !initialized && !turn_over_at_end {
        if tx.state().turn.draw_phase_is_fixed() {
            return Ok(Vec::new());
        }
        tx.state_mut().turn.set_draw_phase_fixed_count(0);
    }
    if !initialized {
        let order = tx.state().players.alive_by_seat();
        remaining_targets = order
            .iter()
            .position(|&target| target == player)
            .map(|start| {
                (1..order.len())
                    .map(|offset| order[(start + offset) % order.len()])
                    .collect()
            })
            .unwrap_or_default();
        initialized = true;
    }
    let Some(target) = remaining_targets.first().copied() else {
        return if turn_over_at_end {
            turn_over(tx, player)
        } else {
            Ok(Vec::new())
        };
    };
    remaining_targets.remove(0);
    let next = if turn_over_at_end {
        EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
            player,
            initialized,
            remaining_targets,
            skill,
        }
    } else {
        EffectRequest::ChooseGainCardFromEachOtherReplaceDraw {
            player,
            initialized,
            remaining_targets,
            skill,
        }
    };
    tx.state_mut().resolution.deferred_tail_effects.push(next);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options = crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        TargetCardScope::HandEquipJudge,
    );
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        player.raw(),
        target.raw(),
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver,
        payload,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mass_damage_discard_turn_over(
    tx: &mut Transaction,
    player: PlayerId,
    mark: engine_api::MarkDefId,
    mark_count: u32,
    damage_amount: u32,
    hand_count: u32,
    mut stage: u8,
    mut index: u32,
    mut targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
    hand_equip: bool,
    turn_owner_over: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if mark.is_none() || mark_count == 0 || damage_amount == 0 || hand_count == 0 {
        return Err(EffectError::ContentFault(
            "mass damage/discard requires non-zero parameters".into(),
        ));
    }
    if stage == 0 {
        let state = tx.state();
        let instance = state
            .skills
            .get(skill)
            .ok_or_else(|| EffectError::ContentFault("mass damage active skill missing".into()))?;
        let contract = state
            .content
            .first_active_for_skill(instance.def)
            .ok_or_else(|| {
                EffectError::ContentFault("mass damage active contract missing".into())
            })?;
        let active_limit_matches = if hand_equip {
            contract.once_per_game
        } else {
            contract.uses_per_turn == Some(1)
        };
        if instance.owner != player
            || !state.skill_effectively_enabled(instance)
            || state.turn.current != player
            || state.turn.phase != Phase::Play
            || !active_limit_matches
            || contract.mark_at_least != Some((mark.raw(), mark_count))
        {
            return Err(EffectError::ContentFault(
                "mass damage/discard requires its declared active contract".into(),
            ));
        }
        let available = state
            .players
            .get(player)
            .and_then(|owner| {
                owner
                    .marks
                    .iter()
                    .find(|(definition, _)| *definition == mark.raw())
                    .map(|(_, count)| (*count).max(0) as u32)
            })
            .unwrap_or(0);
        if available < mark_count {
            return Err(EffectError::Illegal(
                "not enough marks for mass damage/discard".into(),
            ));
        }
        targets = state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != player)
            .collect();
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(if hand_equip {
                EffectRequest::ResolveMassDamageDiscardHandEquip {
                    player,
                    mark,
                    mark_count,
                    damage_amount,
                    discard_count: hand_count,
                    stage: 1,
                    index: 0,
                    targets,
                    skill,
                }
            } else {
                EffectRequest::ResolveMassDamageDiscardTurnOver {
                    player,
                    mark,
                    mark_count,
                    damage_amount,
                    hand_count,
                    stage: 1,
                    index: 0,
                    targets,
                    skill,
                }
            });
        return remove_mark(tx, player, mark.raw(), mark_count);
    }

    loop {
        if stage == if hand_equip { 3 } else { 4 } {
            return if turn_owner_over {
                turn_over(tx, player)
            } else {
                Ok(Vec::new())
            };
        }
        if stage > if hand_equip { 3 } else { 4 } {
            return Err(EffectError::ContentFault(
                "unknown mass damage/discard stage".into(),
            ));
        }
        let Some(target) = targets.get(index as usize).copied() else {
            stage += 1;
            index = 0;
            continue;
        };
        index = index.saturating_add(1);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let next = if hand_equip {
            EffectRequest::ResolveMassDamageDiscardHandEquip {
                player,
                mark,
                mark_count,
                damage_amount,
                discard_count: hand_count,
                stage,
                index,
                targets: targets.clone(),
                skill,
            }
        } else {
            EffectRequest::ResolveMassDamageDiscardTurnOver {
                player,
                mark,
                mark_count,
                damage_amount,
                hand_count,
                stage,
                index,
                targets: targets.clone(),
                skill,
            }
        };
        match stage {
            1 => {
                tx.state_mut().resolution.deferred_tail_effects.push(next);
                return deal_damage(
                    tx,
                    player,
                    target,
                    damage_amount,
                    false,
                    rule_api::damage_nature::NORMAL,
                    false,
                    CardId::NONE,
                    0,
                    0,
                );
            }
            2 if !hand_equip => {
                let Some(from) = tx.state().zones.find(ZoneKind::Equip, target) else {
                    continue;
                };
                let cards = tx
                    .state()
                    .zones
                    .get(from)
                    .map(|zone| zone.cards.clone())
                    .unwrap_or_default();
                if cards.is_empty() {
                    continue;
                }
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                tx.state_mut().resolution.deferred_tail_effects.push(next);
                return move_cards_with_reason(
                    tx,
                    cards,
                    discard,
                    Some(from),
                    CardMoveReason::Discard,
                );
            }
            2 | 3 => {
                let mut options = crate::query::discardable_hand_equip_cards(tx.state(), target);
                if !hand_equip {
                    let hand_zone = tx.state().zones.find(ZoneKind::Hand, target);
                    options.retain(|card| {
                        hand_zone.is_some_and(|zone| {
                            tx.state()
                                .cards
                                .get(*card)
                                .is_some_and(|card| card.zone == zone)
                        })
                    });
                }
                let required = usize::try_from(hand_count)
                    .unwrap_or(usize::MAX)
                    .min(options.len());
                if required == 0 {
                    continue;
                }
                tx.state_mut().resolution.deferred_tail_effects.push(next);
                if required == options.len() {
                    return discard_cards(tx, target, options);
                }
                let required = u8::try_from(required).map_err(|_| {
                    EffectError::ContentFault("mass hand discard count exceeds 255".into())
                })?;
                let mut payload = vec![
                    target.raw(),
                    u32::from(required),
                    options.len().min(u32::MAX as usize) as u32,
                ];
                if hand_equip {
                    payload.push(1);
                }
                payload.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    target,
                    required,
                    required,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::DiscardHandForMassDamage,
                    payload,
                );
            }
            _ => unreachable!(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mark_parity_control(
    tx: &mut Transaction,
    player: PlayerId,
    mark: engine_api::MarkDefId,
    mass_threshold: u32,
    damage_amount: u32,
    stage: u8,
    mut index: u32,
    target: PlayerId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if mark.is_none() || mass_threshold == 0 || damage_amount == 0 {
        return Err(EffectError::ContentFault(
            "mark parity control requires non-zero parameters".into(),
        ));
    }
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill) {
            return Err(EffectError::ContentFault(
                "mark parity control requires its originating skill".into(),
            ));
        }
        let options = tx.state().players.alive_by_seat();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            options.iter().map(|target| target.raw()).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::ChooseMarkParityControlTarget,
            vec![
                player.raw(),
                mark.raw(),
                mass_threshold,
                damage_amount,
                skill.raw(),
            ],
        );
    }
    if stage == 1 {
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            return Ok(Vec::new());
        }
        let marks = tx
            .state()
            .players
            .get(player)
            .and_then(|owner| {
                owner
                    .marks
                    .iter()
                    .find(|(definition, _)| *definition == mark.raw())
                    .map(|(_, count)| (*count).max(0) as u32)
            })
            .unwrap_or(0);
        let next = EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage: if marks % 2 == 1 { 3 } else { 2 },
            index: 0,
            target,
            targets: Vec::new(),
            skill,
        };
        tx.state_mut().resolution.deferred_tail_effects.push(next);
        if marks % 2 == 1 {
            return deal_damage(
                tx,
                player,
                target,
                damage_amount,
                false,
                rule_api::damage_nature::NORMAL,
                false,
                CardId::NONE,
                0,
                0,
            );
        }
        let linked = tx
            .state()
            .players
            .get(target)
            .unwrap()
            .marks
            .iter()
            .any(|(definition, count)| *definition == crate::content::MARK_CHAIN && *count > 0);
        return if linked {
            Ok(Vec::new())
        } else {
            add_mark(tx, target, crate::content::MARK_CHAIN, 1)
        };
    }
    if stage == 2 {
        let options = crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            TargetCardScope::HandEquipJudge,
        );
        if options.is_empty() {
            return resolve_mark_parity_control(
                tx,
                player,
                mark,
                mass_threshold,
                damage_amount,
                3,
                0,
                target,
                Vec::new(),
                skill,
            );
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::DiscardMarkParityControlTargetCard,
            vec![
                player.raw(),
                target.raw(),
                mark.raw(),
                mass_threshold,
                damage_amount,
                skill.raw(),
            ],
        );
    }
    if stage == 3 {
        let marks = tx
            .state()
            .players
            .get(player)
            .and_then(|owner| {
                owner
                    .marks
                    .iter()
                    .find(|(definition, _)| *definition == mark.raw())
                    .map(|(_, count)| (*count).max(0) as u32)
            })
            .unwrap_or(0);
        if marks <= mass_threshold
            || !tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.alive)
        {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ConfirmMarkParityMassDamage,
            vec![
                player.raw(),
                mark.raw(),
                mass_threshold,
                damage_amount,
                skill.raw(),
            ],
        );
    }
    if stage != 4 {
        return Err(EffectError::ContentFault(
            "unknown mark parity control stage".into(),
        ));
    }
    let Some(target) = targets.get(index as usize).copied() else {
        return Ok(Vec::new());
    };
    index = index.saturating_add(1);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return resolve_mark_parity_control(
            tx,
            player,
            mark,
            mass_threshold,
            damage_amount,
            4,
            index,
            PlayerId::NONE,
            targets,
            skill,
        );
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold,
            damage_amount,
            stage: 4,
            index,
            target: PlayerId::NONE,
            targets,
            skill,
        });
    deal_damage(
        tx,
        player,
        target,
        damage_amount,
        false,
        rule_api::damage_nature::NORMAL,
        false,
        CardId::NONE,
        0,
        0,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_selected_equipment_discard_then_choose_damage(
    tx: &mut Transaction,
    player: PlayerId,
    damage_amount: u32,
    nature: u8,
    mut stage: u8,
    mut index: u32,
    mut targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if damage_amount == 0 || nature > rule_api::damage_nature::THUNDER {
        return Err(EffectError::ContentFault(
            "selected equipment discard/damage has invalid parameters".into(),
        ));
    }
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill) || targets.is_empty() {
            return Err(EffectError::ContentFault(
                "selected equipment discard/damage requires active targets".into(),
            ));
        }
        let order = tx.state().players.alive_by_seat();
        let mut seen = std::collections::BTreeSet::new();
        if targets.iter().any(|&target| {
            !order.contains(&target)
                || !seen.insert(target.raw())
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
        }) {
            return Err(EffectError::Illegal(
                "selected equipment discard/damage targets are invalid".into(),
            ));
        }
        targets.sort_by_key(|target| {
            order
                .iter()
                .position(|candidate| candidate == target)
                .expect("validated selected target")
        });
        stage = 1;
        index = 0;
    }
    if stage == 1 {
        while let Some(target) = targets.get(index as usize).copied() {
            index = index.saturating_add(1);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                continue;
            }
            let Some(from) = tx.state().zones.find(ZoneKind::Equip, target) else {
                continue;
            };
            let cards = tx
                .state()
                .zones
                .get(from)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default();
            if cards.is_empty() {
                continue;
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveSelectedEquipmentDiscardThenChooseDamage {
                    player,
                    damage_amount,
                    nature,
                    stage: 1,
                    index,
                    targets,
                    skill,
                },
            );
            return move_cards_with_reason(tx, cards, discard, Some(from), CardMoveReason::Discard);
        }
        stage = 2;
    }
    if stage != 2 {
        return Err(EffectError::ContentFault(
            "unknown selected equipment discard/damage stage".into(),
        ));
    }
    let options: Vec<_> = targets
        .iter()
        .copied()
        .filter(|target| {
            tx.state()
                .players
                .get(*target)
                .is_some_and(|participant| participant.alive)
        })
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        player.raw(),
        damage_amount,
        u32::from(nature),
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseSelectedTargetDamage,
        payload,
    )
}

pub(crate) fn replace_draw_with_distinct_suit_selection(
    tx: &mut Transaction,
    player: PlayerId,
    count: u8,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 {
        return Err(EffectError::ContentFault(
            "distinct-suit draw replacement requires a positive count".into(),
        ));
    }
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || tx.state().turn.current != player
            || tx.state().turn.phase != Phase::Draw
            || tx.state().turn.draw_phase_is_fixed()
        {
            return Err(EffectError::Illegal(
                "invalid distinct-suit draw replacement context".into(),
            ));
        }
        tx.state_mut().turn.set_draw_phase_fixed_count(0);
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
        if tx
            .state()
            .zones
            .get(draw)
            .map_or(0, |zone| zone.cards.len())
            < usize::from(count)
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let zone = tx
            .state()
            .zones
            .get(draw)
            .ok_or_else(|| EffectError::EngineFault("draw pile disappeared".into()))?;
        let take = usize::from(count).min(zone.cards.len());
        if take == 0 {
            return Ok(Vec::new());
        }
        let cards = zone.cards[zone.cards.len() - take..].to_vec();
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ReplaceDrawWithDistinctSuitSelection {
                player,
                count,
                stage: 1,
                cards: cards.clone(),
                skill,
            },
        );
        let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
        events.push(DomainEvent::CardsRevealed {
            cards,
            to: Vec::new(),
        });
        return Ok(events);
    }
    if stage != 1 || cards.is_empty() {
        return Err(EffectError::ContentFault(
            "bad distinct-suit draw replacement stage".into(),
        ));
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    if cards.iter().any(|card| {
        !tx.state()
            .cards
            .get(*card)
            .is_some_and(|card| card.zone == processing)
    }) {
        return Err(EffectError::Illegal(
            "distinct-suit revealed cards left processing".into(),
        ));
    }
    let suits: std::collections::BTreeSet<_> = cards
        .iter()
        .filter_map(|card| tx.state().cards.get(*card))
        .map(|card| {
            crate::command::judge::suit_tag(crate::query::effective_card_suit(
                tx.state(),
                player,
                card,
            ))
        })
        .collect();
    let required = u8::try_from(suits.len())
        .map_err(|_| EffectError::EngineFault("distinct suit count exceeds 255".into()))?;
    let mut payload = vec![player.raw(), skill.raw(), cards.len() as u32];
    payload.extend(cards.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        required,
        required,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDistinctSuitCards,
        payload,
    )
}

pub(crate) fn inspect_target_hand_choose_heart_discard_or_top(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || player == target
            || ![player, target].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|state| state.alive)
            })
        {
            return Err(EffectError::Illegal(
                "invalid target-hand heart inspection context".into(),
            ));
        }
        let hand = tx.state().hand_cards(target);
        if hand.is_empty() {
            return Err(EffectError::Illegal(
                "target-hand heart inspection requires a nonempty hand".into(),
            ));
        }
        let hearts: Vec<_> = hand
            .iter()
            .copied()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_suit(tx.state(), target, card) == Suit::Heart
                })
            })
            .collect();
        let events = vec![DomainEvent::CardsRevealed {
            cards: hand,
            to: vec![player],
        }];
        if hearts.is_empty() {
            return Ok(events);
        }
        let mut events = events;
        events.extend(open_choice(
            tx,
            player,
            1,
            1,
            true,
            hearts.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseInspectedHeartCard,
            vec![player.raw(), target.raw(), skill.raw()],
        )?);
        return Ok(events);
    }
    if stage != 1 {
        return Err(EffectError::ContentFault(
            "bad target-hand heart inspection stage".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("target hand missing".into()))?;
    if !tx.state().cards.get(card).is_some_and(|state| {
        state.zone == hand
            && crate::query::effective_card_suit(tx.state(), target, state) == Suit::Heart
    }) {
        return Err(EffectError::Illegal(
            "inspected heart card is no longer in hand".into(),
        ));
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![0, 1],
        ChoiceKind::Custom,
        ContinuationKind::ChooseInspectedHeartDestination,
        vec![player.raw(), target.raw(), card.raw(), skill.raw()],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_yeyan(
    tx: &mut Transaction,
    player: PlayerId,
    mode: u8,
    stage: u8,
    mut index: u32,
    mut targets: Vec<PlayerId>,
    data: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 {
        let state = tx.state();
        let instance = state
            .skills
            .get(skill)
            .ok_or_else(|| EffectError::ContentFault("Yeyan active skill missing".into()))?;
        let contract = state
            .content
            .first_active_for_skill(instance.def)
            .ok_or_else(|| EffectError::ContentFault("Yeyan active contract missing".into()))?;
        if mode != u8::MAX
            || instance.owner != player
            || !state.skill_effectively_enabled(instance)
            || state.turn.current != player
            || state.turn.phase != Phase::Play
            || !contract.once_per_game
            || contract.targets.is_some()
            || contract.selection.is_some()
        {
            return Err(EffectError::ContentFault(
                "Yeyan requires its targetless once-per-game active contract".into(),
            ));
        }
        let mut suits = std::collections::BTreeSet::new();
        for card in crate::query::yeyan_hand_cards(state, player) {
            let card = state
                .cards
                .get(card)
                .ok_or_else(|| EffectError::EngineFault("Yeyan hand card missing".into()))?;
            suits.insert(crate::command::judge::suit_tag(
                crate::query::effective_card_suit(state, player, card),
            ));
        }
        let mut options = vec![0];
        if suits.len() >= 4 {
            options.push(1);
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::Custom,
            ContinuationKind::ChooseYeyanMode,
            vec![player.raw(), skill.raw()],
        );
    }

    if stage == 1 {
        if mode == 0 {
            let options = tx.state().players.alive_by_seat();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            return open_choice(
                tx,
                player,
                1,
                options.len().min(3) as u8,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseYeyanTargets,
                vec![player.raw(), 0, skill.raw()],
            );
        }
        if mode != 1 {
            return Err(EffectError::ContentFault("unknown Yeyan mode".into()));
        }
        let options = crate::query::yeyan_hand_cards(tx.state(), player);
        let suits: std::collections::BTreeSet<_> = options
            .iter()
            .filter_map(|card| tx.state().cards.get(*card))
            .map(|card| {
                crate::command::judge::suit_tag(crate::query::effective_card_suit(
                    tx.state(),
                    player,
                    card,
                ))
            })
            .collect();
        if suits.len() < 4 {
            return Err(EffectError::Illegal(
                "Yeyan no longer has four distinct hand-card suits".into(),
            ));
        }
        let mut payload = vec![player.raw(), skill.raw(), options.len() as u32];
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            4,
            4,
            false,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseYeyanBigCards,
            payload,
        );
    }

    if stage == 2 {
        if mode != 1 {
            return Err(EffectError::ContentFault(
                "only big Yeyan chooses targets after its cost".into(),
            ));
        }
        let options = tx.state().players.alive_by_seat();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            options.len().min(2) as u8,
            false,
            options.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::ChooseYeyanTargets,
            vec![player.raw(), 1, skill.raw()],
        );
    }

    if stage == 3 {
        if mode != 1 || !matches!(targets.len(), 1 | 2) {
            return Err(EffectError::ContentFault(
                "bad big-Yeyan allocation state".into(),
            ));
        }
        if targets.len() == 1 {
            return open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![2, 3],
                ChoiceKind::Custom,
                ContinuationKind::ChooseYeyanSingleDamage,
                vec![player.raw(), skill.raw(), targets[0].raw()],
            );
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets.iter().map(|target| target.raw()).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::ChooseYeyanDoubleTarget,
            vec![
                player.raw(),
                skill.raw(),
                targets[0].raw(),
                targets[1].raw(),
            ],
        );
    }

    if stage != 4 || !matches!(mode, 0 | 1) {
        return Err(EffectError::ContentFault("bad Yeyan stage".into()));
    }
    targets.sort_by_key(|target| {
        tx.state()
            .players
            .get(*target)
            .map(|participant| participant.seat.raw())
            .unwrap_or(u8::MAX)
    });
    loop {
        let Some(target) = targets.get(index as usize).copied() else {
            return Ok(Vec::new());
        };
        index = index.saturating_add(1);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let amount = if mode == 0 {
            1
        } else if targets.len() == 1 {
            if !matches!(data, 2 | 3) {
                return Err(EffectError::ContentFault(
                    "single-target big Yeyan requires two or three damage".into(),
                ));
            }
            data
        } else if targets.len() == 2 {
            if !targets.iter().any(|target| target.raw() == data) {
                return Err(EffectError::ContentFault(
                    "two-target big Yeyan boost target missing".into(),
                ));
            }
            u32::from(target.raw() == data) + 1
        } else {
            return Err(EffectError::ContentFault(
                "invalid Yeyan target count".into(),
            ));
        };
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveYeyan {
                player,
                mode,
                stage: 4,
                index,
                targets,
                data,
                skill,
            });
        return deal_damage(
            tx,
            player,
            target,
            amount,
            false,
            rule_api::damage_nature::FIRE,
            false,
            CardId::NONE,
            0,
            0,
        );
    }
}

pub(crate) fn choose_all_lose_hp_or_recover(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || tx.state().turn.flag(turn_flag::DISCARD_PHASE_DISCARDED) < 2
    {
        return Err(EffectError::Illegal(
            "invalid Qinyin discard-phase context".into(),
        ));
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        vec![0, 1],
        ChoiceKind::Custom,
        ContinuationKind::ChooseAllLoseHpOrRecover,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_qixing(
    tx: &mut Transaction,
    player: PlayerId,
    initialize: bool,
    stage: u8,
    hand_cards: Vec<CardId>,
    pile_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if stage == 0 && initialize && tx.state().skill_pile_cards(player, skill_def).is_empty() {
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
        let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
        let mut events = Vec::new();
        for _ in 0..7 {
            if tx
                .state()
                .zones
                .get(draw)
                .is_none_or(|zone| zone.cards.is_empty())
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let Some(card) = tx
                .state()
                .zones
                .get(draw)
                .and_then(|zone| zone.cards.last())
                .copied()
            else {
                break;
            };
            events.extend(move_cards(tx, vec![card], pile, Some(draw))?);
        }
        if !events.is_empty() {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveQixing {
                    player,
                    initialize,
                    stage: 1,
                    hand_cards: Vec::new(),
                    pile_cards: Vec::new(),
                    skill,
                });
            return Ok(events);
        }
    }
    if stage <= 1 {
        let hand = tx.state().hand_cards(player);
        let pile = tx.state().skill_pile_cards(player, skill_def);
        let max = hand.len().min(pile.len()).min(u8::MAX as usize) as u8;
        if max == 0 {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            max,
            true,
            hand.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseQixingHandCards,
            vec![player.raw(), skill.raw()],
        );
    }
    if stage != 2 || hand_cards.len() != pile_cards.len() || hand_cards.is_empty() {
        return Err(EffectError::Illegal("invalid Qixing exchange state".into()));
    }
    let hand_zone = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile_zone = ensure_skill_pile_zone(tx, player, skill_def)?;
    let current_hand: std::collections::BTreeSet<_> =
        tx.state().hand_cards(player).into_iter().collect();
    let current_pile: std::collections::BTreeSet<_> = tx
        .state()
        .skill_pile_cards(player, skill_def)
        .into_iter()
        .collect();
    let selected_hand: std::collections::BTreeSet<_> = hand_cards.iter().copied().collect();
    let selected_pile: std::collections::BTreeSet<_> = pile_cards.iter().copied().collect();
    if selected_hand.len() != hand_cards.len()
        || selected_pile.len() != pile_cards.len()
        || !selected_hand.is_subset(&current_hand)
        || !selected_pile.is_subset(&current_pile)
    {
        return Err(EffectError::Illegal(
            "Qixing exchange cards are no longer legal".into(),
        ));
    }
    let mut events = move_cards(tx, hand_cards, pile_zone, Some(hand_zone))?;
    events.extend(move_cards(tx, pile_cards, hand_zone, Some(pile_zone))?);
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_hidden_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    pile_skill_def: engine_api::SkillDefId,
    action: u8,
    card: CardId,
    stage: u8,
    hand_cards: Vec<CardId>,
    pile_cards: Vec<CardId>,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if pile_skill_def.is_none()
        || !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let pile_def = pile_skill_def.raw();
    match (action, stage) {
        (0, 0) => {
            if !tx.state().skill_pile_cards(player, pile_def).is_empty() {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            let pile = ensure_skill_pile_zone(tx, player, pile_def)?;
            let mut events = Vec::new();
            for _ in 0..2 {
                if tx
                    .state()
                    .zones
                    .get(draw)
                    .is_none_or(|zone| zone.cards.is_empty())
                {
                    reshuffle_discard_into_draw(tx)?;
                }
                let Some(next) = tx
                    .state()
                    .zones
                    .get(draw)
                    .and_then(|zone| zone.cards.last())
                    .copied()
                else {
                    break;
                };
                events.extend(move_cards(tx, vec![next], pile, Some(draw))?);
                tx.state_mut()
                    .cards
                    .get_mut(next)
                    .expect("moved card remains live")
                    .face_up = false;
            }
            Ok(events)
        }
        (1 | 3, 0) => {
            let hidden = tx.state().hidden_skill_pile_cards(player, pile_def);
            if hidden.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                hidden.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseHiddenSkillPileCards,
                vec![
                    player.raw(),
                    pile_def,
                    u32::from(action),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        (1 | 3, 1) => {
            let [selected] = pile_cards.as_slice() else {
                return Err(EffectError::Illegal(
                    "hidden skill-pile reveal requires one card".into(),
                ));
            };
            if !tx
                .state()
                .hidden_skill_pile_cards(player, pile_def)
                .contains(selected)
            {
                return Err(EffectError::Illegal(
                    "hidden skill-pile card is no longer hidden".into(),
                ));
            }
            tx.state_mut()
                .cards
                .get_mut(*selected)
                .expect("validated hidden card")
                .face_up = true;
            let mut events = vec![DomainEvent::RuleNote {
                code: 238,
                payload: vec![player.raw(), pile_def, selected.raw()],
            }];
            if action == 1 {
                crate::command::play::exclude_saved_card_target(tx, card, player)?;
            } else {
                let count = tx
                    .state()
                    .revealed_skill_pile_cards(player, pile_def)
                    .len()
                    .min(u32::MAX as usize) as u32;
                events.extend(draw_cards(tx, player, count)?);
            }
            Ok(events)
        }
        (2, 0) => {
            let hand = tx.state().hand_cards(player);
            let max = hand
                .len()
                .min(tx.state().hidden_skill_pile_cards(player, pile_def).len())
                .min(u8::MAX as usize) as u8;
            if max == 0 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                max,
                true,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseHiddenSkillPileHandCards,
                vec![
                    player.raw(),
                    pile_def,
                    u32::from(action),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        (2, 1) => {
            if hand_cards.is_empty() || hand_cards.len() != pile_cards.len() {
                return Err(EffectError::Illegal(
                    "hidden skill-pile exchange counts differ".into(),
                ));
            }
            let current_hand: std::collections::BTreeSet<_> =
                tx.state().hand_cards(player).into_iter().collect();
            let current_hidden: std::collections::BTreeSet<_> = tx
                .state()
                .hidden_skill_pile_cards(player, pile_def)
                .into_iter()
                .collect();
            let chosen_hand: std::collections::BTreeSet<_> = hand_cards.iter().copied().collect();
            let chosen_pile: std::collections::BTreeSet<_> = pile_cards.iter().copied().collect();
            if chosen_hand.len() != hand_cards.len()
                || chosen_pile.len() != pile_cards.len()
                || !chosen_hand.is_subset(&current_hand)
                || !chosen_pile.is_subset(&current_hidden)
            {
                return Err(EffectError::Illegal(
                    "hidden skill-pile exchange is no longer legal".into(),
                ));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("owner hand zone missing".into()))?;
            let pile = ensure_skill_pile_zone(tx, player, pile_def)?;
            let moved_to_pile = hand_cards.clone();
            let mut events = move_cards(tx, hand_cards, pile, Some(hand))?;
            for card in moved_to_pile {
                tx.state_mut()
                    .cards
                    .get_mut(card)
                    .expect("moved card remains live")
                    .face_up = false;
            }
            events.extend(move_cards(tx, pile_cards, hand, Some(pile))?);
            Ok(events)
        }
        (4, 0) => {
            let suits: Vec<_> = tx
                .state()
                .revealed_skill_pile_cards(player, pile_def)
                .into_iter()
                .filter_map(|card| tx.state().cards.get(card).map(|card| card.suit))
                .collect();
            let count = event_cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| suits.contains(&card.suit))
                })
                .count()
                .min(u32::MAX as usize) as u32;
            draw_cards(tx, player, count)
        }
        _ => Err(EffectError::ContentFault(format!(
            "invalid hidden skill-pile action {action} stage {stage}"
        ))),
    }
}

pub(crate) fn set_non_locked_skill_suppression(
    tx: &mut Transaction,
    player: PlayerId,
    enabled: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || tx.state().turn.current != player
    {
        return Ok(Vec::new());
    }
    if enabled {
        tx.state_mut().turn.non_locked_skill_suppression = skill;
    } else {
        if tx.state().players.iter_live().any(|player| player.dying)
            || tx.state().turn.non_locked_skill_suppression != skill
        {
            return Ok(Vec::new());
        }
        tx.state_mut().turn.non_locked_skill_suppression = engine_api::SkillId::NONE;
    }
    tx.state_mut().rebuild_derived_indexes();
    Ok(vec![DomainEvent::RuleNote {
        code: 239,
        payload: vec![player.raw(), skill.raw(), u32::from(enabled)],
    }])
}

pub(crate) fn suppress_non_locked_skills_until_turn_end(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().turn.current.is_none()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || tx
            .state()
            .turn
            .non_locked_skill_suppressed_players
            .contains(&player)
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .turn
        .non_locked_skill_suppressed_players
        .push(player);
    tx.state_mut()
        .turn
        .non_locked_skill_suppressed_players
        .sort_by_key(|player| player.raw());
    tx.state_mut().rebuild_derived_indexes();
    Ok(vec![DomainEvent::RuleNote {
        code: 356,
        payload: vec![player.raw()],
    }])
}

pub(crate) const DILI_DEF: u32 = 3_732_080_996;
pub(crate) const YUHENG_DEF: u32 = 3_006_389_674;
pub(crate) const DILI_LINES: [u32; 6] = [
    3_511_235_243,
    3_520_220_242,
    4_057_699_019,
    2_682_708_493,
    3_616_039_425,
    2_521_447_965,
];
pub(crate) const YUHENG_POOL: [u32; 18] = [
    2_790_748_306,
    3_138_095_197,
    2_238_473_770,
    3_285_976_176,
    3_859_813_014,
    3_075_552_814,
    4_211_343_786,
    4_068_014_439,
    2_231_312_450,
    2_455_708_599,
    2_662_349_862,
    2_398_589_569,
    2_873_874_515,
    3_295_441_821,
    2_509_094_453,
    3_040_931_439,
    3_834_102_895,
    3_473_961_755,
];
pub(crate) const YUHENG_ALL_MASK: u32 = (1 << YUHENG_POOL.len()) - 1;
pub(crate) const CHANGANDAJIAN_DESTROY_DEF: u32 = 4_232_406_220;
pub(crate) const CHANGANDAJIAN_EQUIP2_DEF: u32 = 4_130_114_000;
pub(crate) const CHANGANDAJIAN_DEFS: [u32; 6] = [
    3_238_845_718,
    4_130_114_000,
    2_828_308_711,
    3_888_754_104,
    4_174_045_670,
    3_540_539_807,
];

#[derive(Clone, Copy, Default)]
pub(crate) struct YuhengState {
    history: u32,
    full_remaining: u32,
    current_remaining: u32,
}

pub(crate) fn decode_yuheng_state(blob: &[u8]) -> YuhengState {
    if blob.len() != 16 || &blob[..4] != b"YHG1" {
        return YuhengState::default();
    }
    YuhengState {
        history: u32::from_le_bytes(blob[4..8].try_into().unwrap()),
        full_remaining: u32::from_le_bytes(blob[8..12].try_into().unwrap()),
        current_remaining: u32::from_le_bytes(blob[12..16].try_into().unwrap()),
    }
}

pub(crate) fn encode_yuheng_state(state: YuhengState) -> Vec<u8> {
    let mut blob = b"YHG1".to_vec();
    blob.extend_from_slice(&state.history.to_le_bytes());
    blob.extend_from_slice(&state.full_remaining.to_le_bytes());
    blob.extend_from_slice(&state.current_remaining.to_le_bytes());
    blob
}

pub(crate) fn player_skill_by_def(
    tx: &Transaction,
    player: PlayerId,
    skill_def: u32,
) -> Option<engine_api::SkillId> {
    tx.state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def.raw() == skill_def)
        })
        .copied()
}

pub(crate) fn current_yuheng_mask(tx: &Transaction, player: PlayerId) -> u32 {
    DILI_LINES
        .iter()
        .enumerate()
        .filter(|(_, line)| player_skill_by_def(tx, player, **line).is_some())
        .fold(0, |mask, (index, _)| mask | (0b111 << (index * 3)))
}

pub(crate) fn yuheng_state(tx: &Transaction, player: PlayerId) -> Option<(engine_api::SkillId, YuhengState)> {
    let skill = player_skill_by_def(tx, player, YUHENG_DEF)?;
    Some((
        skill,
        decode_yuheng_state(&tx.state().skills.get(skill)?.state_blob),
    ))
}

pub(crate) fn store_yuheng_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    state: YuhengState,
) -> Result<(), EffectError> {
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("Yuheng skill disappeared".into()))?;
    instance.state_blob = encode_yuheng_state(state);
    Ok(())
}

pub(crate) fn refresh_dili_line_enablement(tx: &mut Transaction, player: PlayerId, history: u32) {
    for (index, line) in DILI_LINES.iter().enumerate() {
        let Some(skill) = player_skill_by_def(tx, player, *line) else {
            continue;
        };
        let group = 0b111 << (index * 3);
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.enabled = history & group == group;
        }
    }
    tx.state_mut().rebuild_derived_indexes();
}

pub(crate) fn take_random_mask_bit(tx: &mut Transaction, mask: u32) -> Option<(usize, u32)> {
    let choices: Vec<_> = (0..YUHENG_POOL.len())
        .filter(|index| mask & (1 << index) != 0)
        .collect();
    if choices.is_empty() {
        return None;
    }
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let choice = choices[rng.gen_range_u32(choices.len() as u32) as usize];
    tx.state_mut().rng = rng.into_state();
    Some((choice, mask & !(1 << choice)))
}

pub(crate) fn add_dili_line(
    tx: &mut Transaction,
    player: PlayerId,
    line_index: usize,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = add_skill(
        tx,
        player,
        engine_api::SkillDefId::new(DILI_LINES[line_index]),
    )?;
    if let Some((yuheng, mut state)) = yuheng_state(tx, player) {
        state.current_remaining |= 0b111 << (line_index * 3);
        store_yuheng_state(tx, yuheng, state)?;
        refresh_dili_line_enablement(tx, player, state.history);
    }
    events.push(DomainEvent::RuleNote {
        code: 240,
        payload: vec![player.raw(), DILI_LINES[line_index]],
    });
    Ok(events)
}

pub(crate) fn open_changandajian_target_choice(
    tx: &mut Transaction,
    player: PlayerId,
    remaining: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if remaining == 0 {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            [ZoneKind::Equip, ZoneKind::Judge].into_iter().any(|kind| {
                tx.state()
                    .zones
                    .find(kind, target)
                    .and_then(|zone| tx.state().zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty())
            })
        })
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseChangandajianTarget,
        vec![player.raw(), remaining, skill.raw()],
    )
}

pub(crate) fn finalize_tiaojiyanmei(
    tx: &mut Transaction,
    actor: PlayerId,
    left: PlayerId,
    right: PlayerId,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive)
        || tx.state().hand_count(left) != tx.state().hand_count(right)
    {
        return Ok(Vec::new());
    }
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card| Some(card.zone) == discard)
    {
        return Ok(Vec::new());
    }
    let targets = tx.state().players.alive_by_seat();
    open_choice(
        tx,
        actor,
        1,
        1,
        true,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTiaojiyanmeiGainTarget,
        vec![actor.raw(), card.raw()],
    )
}

pub(crate) fn begin_tiaojiyanmei_discard(
    tx: &mut Transaction,
    actor: PlayerId,
    left: PlayerId,
    right: PlayerId,
    higher: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), higher);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        higher,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DiscardTiaojiyanmeiCard,
        vec![actor.raw(), left.raw(), right.raw(), higher.raw()],
    )
}

pub(crate) fn resolve_dili_yuheng(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    card: CardId,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(origin) = tx.state().skills.get(skill).cloned() else {
        return Ok(Vec::new());
    };
    if origin.owner != player || !tx.state().skill_effectively_enabled(&origin) {
        return Ok(Vec::new());
    }
    match action {
        0 if origin.def.raw() == DILI_DEF => {
            if DILI_LINES
                .iter()
                .any(|line| player_skill_by_def(tx, player, *line).is_some())
            {
                return Ok(Vec::new());
            }
            let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
            let line = rng.gen_range_u32(DILI_LINES.len() as u32) as usize;
            tx.state_mut().rng = rng.into_state();
            add_dili_line(tx, player, line)
        }
        1 if origin.def.raw() == YUHENG_DEF => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let used_other_non_locked = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|player| player.skills.iter())
                .filter_map(|skill| tx.state().skills.get(*skill))
                .any(|instance| {
                    instance.id != skill
                        && instance.play_phase_use_count > 0
                        && !tx
                            .state()
                            .content
                            .skill_metadata(instance.def)
                            .is_some_and(|metadata| metadata.locked)
                });
            let removable: Vec<_> = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|player| player.skills.iter())
                .filter(|candidate| **candidate != skill)
                .filter(|candidate| {
                    tx.state().skills.get(**candidate).is_some_and(|instance| {
                        !tx.state()
                            .content
                            .skill_metadata(instance.def)
                            .is_some_and(|metadata| metadata.locked)
                    })
                })
                .copied()
                .collect();
            let mut events = Vec::new();
            for candidate in removable {
                events.extend(remove_skill(tx, candidate)?);
            }

            let mut state = decode_yuheng_state(&origin.state_blob);
            let full = if state.full_remaining == 0 {
                YUHENG_ALL_MASK
            } else {
                state.full_remaining
            };
            let current = if state.current_remaining == 0 {
                current_yuheng_mask(tx, player)
            } else {
                state.current_remaining
            };
            let (full_index, full_remaining) = take_random_mask_bit(tx, full)
                .ok_or_else(|| EffectError::EngineFault("Yuheng full pool is empty".into()))?;
            let mut selected = vec![full_index];
            let mut current_remaining = current;
            if !used_other_non_locked && current != 0 {
                let distinct = current & !(1 << full_index);
                let selection_mask = if distinct == 0 { current } else { distinct };
                if let Some((current_index, _)) = take_random_mask_bit(tx, selection_mask) {
                    selected.push(current_index);
                    current_remaining &= !(1 << current_index);
                }
            }
            for index in selected {
                let selected_def = YUHENG_POOL[index];
                state.history |= 1 << index;
                events.extend(add_skill(
                    tx,
                    player,
                    engine_api::SkillDefId::new(selected_def),
                )?);
            }
            state.full_remaining = full_remaining;
            state.current_remaining = current_remaining;
            store_yuheng_state(tx, skill, state)?;
            refresh_dili_line_enablement(tx, player, state.history);

            let qionglan = player_skill_by_def(tx, player, DILI_LINES[2]);
            if state.history & (0b111 << 6) == 0b111 << 6
                && qionglan.is_some_and(|helper| {
                    tx.state()
                        .skills
                        .get(helper)
                        .is_some_and(|instance| instance.lifetime_fire_count == 0)
                })
            {
                let mut candidates = vec![0usize, 1, 3, 4, 5];
                let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
                rng.shuffle(&mut candidates);
                if candidates[..2].contains(&3) && candidates[..2].contains(&4) {
                    candidates.swap(1, 2);
                }
                tx.state_mut().rng = rng.into_state();
                for line in candidates.into_iter().take(2) {
                    events.extend(add_dili_line(tx, player, line)?);
                }
                if let Some(helper) = qionglan {
                    tx.state_mut()
                        .skills
                        .get_mut(helper)
                        .expect("Qionglan helper exists")
                        .lifetime_fire_count = 1;
                }
                refresh_dili_line_enablement(tx, player, state.history);
            }
            Ok(events)
        }
        2 if origin.def.raw() == YUHENG_DEF => {
            if origin.play_phase_use_count != 0 {
                return Ok(Vec::new());
            }
            deal_damage(
                tx,
                PlayerId::NONE,
                player,
                1,
                true,
                0,
                false,
                CardId::NONE,
                0,
                0,
            )
        }
        3 if origin.def.raw() == DILI_LINES[0] => {
            let Some((_, state)) = yuheng_state(tx, player) else {
                return Ok(Vec::new());
            };
            let rank = tx.state().cards.get(card).map_or(0, |card| card.rank);
            if state.history & 0b111 != 0b111 || !matches!(rank, 2 | 3 | 5 | 7 | 11 | 13) {
                return Ok(Vec::new());
            }
            add_card_use_policy_skill(tx, player, card, 1 << 1, origin.def)
        }
        4 if origin.def.raw() == DILI_LINES[1] => {
            let Some((_, state)) = yuheng_state(tx, player) else {
                return Ok(Vec::new());
            };
            if state.history & (0b111 << 3) != 0b111 << 3
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Judge
            {
                return Ok(Vec::new());
            }
            let events = skip_next_judge_phase(tx, player)?;
            tx.state_mut()
                .turn
                .set_flag(turn_flag::PRE_NORMAL_EXTRA_DRAW, 1);
            Ok(events)
        }
        5 if origin.def.raw() == DILI_LINES[5] => {
            let Some((_, state)) = yuheng_state(tx, player) else {
                return Ok(Vec::new());
            };
            let Some(source) = tx.state().cards.get(card).cloned() else {
                return Ok(Vec::new());
            };
            let Some(slot) = source.kind.equip_slot() else {
                return Ok(Vec::new());
            };
            if state.history & (0b111 << 15) != 0b111 << 15
                || source.owner != player
                || !tx
                    .state()
                    .zones
                    .get(source.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Equip)
                || CHANGANDAJIAN_DEFS.contains(&source.def.raw())
            {
                return Ok(Vec::new());
            }
            let ship_def = engine_api::CardDefId::new(CHANGANDAJIAN_DEFS[slot.index()]);
            let metadata = tx
                .state()
                .content
                .card_metadata(ship_def)
                .cloned()
                .ok_or_else(|| EffectError::ContentFault("Changandajian card missing".into()))?;
            let kind = match metadata.kind.as_str() {
                "weapon" => CardKind::Weapon,
                "armor" => CardKind::Armor,
                "horse_def" => CardKind::HorseDef,
                "horse_off" => CardKind::HorseOff,
                "treasure" => CardKind::Treasure,
                _ => {
                    return Err(EffectError::ContentFault(
                        "Changandajian card kind is invalid".into(),
                    ));
                }
            };
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            let equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .ok_or_else(|| EffectError::EngineFault("equipment zone missing".into()))?;
            let mut events = move_cards(tx, vec![card], discard, Some(source.zone))?;
            let spade = origin.state_blob.first().copied().unwrap_or(0) != 0;
            let ship = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: ship_def,
                kind,
                suit: if spade { Suit::Spade } else { Suit::Heart },
                rank: 10,
                weapon_range: metadata.weapon_range,
                zone: processing,
                owner: PlayerId::NONE,
                face_up: true,
            });
            tx.state_mut()
                .zones
                .get_mut(processing)
                .expect("processing zone exists")
                .cards
                .push(ship);
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("Yuanlv skill exists")
                .state_blob = vec![u8::from(!spade)];
            events.extend(move_cards(tx, vec![ship], equip, Some(processing))?);
            Ok(events)
        }
        128 if origin.def.raw() == CHANGANDAJIAN_DESTROY_DEF => {
            let ships: Vec<_> = event_cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .and_then(|card| tx.state().content.card_metadata(card.def))
                        .is_some_and(|card| {
                            card.on_leave_equip_skill_def == CHANGANDAJIAN_DESTROY_DEF
                        })
                })
                .collect();
            if ships.is_empty() {
                return Ok(Vec::new());
            }
            let recover = ships
                .iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(**card)
                        .is_some_and(|card| card.def.raw() == CHANGANDAJIAN_EQUIP2_DEF)
                })
                .count()
                .min(u32::MAX as usize) as u32;
            let removed = tx
                .state()
                .zones
                .find(ZoneKind::Removed, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("removed zone missing".into()))?;
            let mut events = Vec::new();
            for card in &ships {
                let from = tx
                    .state()
                    .cards
                    .get(*card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::EngineFault("ship card missing".into()))?;
                events.extend(move_cards(tx, vec![*card], removed, Some(from))?);
            }
            events.extend(recover_hp(tx, player, player, recover)?);
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDiliYuheng {
                    player,
                    action: 129,
                    card: CardId::NONE,
                    event_cards: ships,
                    skill,
                },
            );
            Ok(events)
        }
        129 if origin.def.raw() == CHANGANDAJIAN_DESTROY_DEF => open_changandajian_target_choice(
            tx,
            player,
            event_cards.len().min(u32::MAX as usize) as u32,
            skill,
        ),
        _ => Ok(Vec::new()),
    }
}

pub(crate) fn add_source_turn_bound_skill(
    tx: &mut Transaction,
    target: PlayerId,
    skill_def: engine_api::SkillDefId,
    source: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let events = add_skill_with_state(tx, target, skill_def, None)?;
    let skill = tx
        .state()
        .players
        .get(target)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def == skill_def)
        })
        .copied()
        .ok_or_else(|| EffectError::EngineFault("source-turn-bound helper was not added".into()))?;
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| instance.add_source_turn_bound(source))
    {
        return Err(EffectError::EngineFault(
            "source-turn-bound helper source could not be recorded".into(),
        ));
    }
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_qixing_weather(
    tx: &mut Transaction,
    player: PlayerId,
    qixing_skill_def: engine_api::SkillDefId,
    helper_skill_def: engine_api::SkillDefId,
    max_targets: u8,
    stage: u8,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if max_targets == 0
        || qixing_skill_def.is_none()
        || helper_skill_def.is_none()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill) {
            return Ok(Vec::new());
        }
        let cards = tx.state().skill_pile_cards(player, qixing_skill_def.raw());
        let max = cards
            .len()
            .min(tx.state().players.alive_by_seat().len())
            .min(usize::from(max_targets))
            .min(u8::MAX as usize) as u8;
        if max == 0 {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            max,
            true,
            cards.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseQixingWeatherCards,
            vec![
                player.raw(),
                skill.raw(),
                qixing_skill_def.raw(),
                helper_skill_def.raw(),
                u32::from(max_targets),
            ],
        );
    }
    if stage != 1 || cards.len() != targets.len() || cards.is_empty() {
        return Err(EffectError::Illegal("invalid Qixing weather state".into()));
    }
    let mut unique_targets = std::collections::BTreeSet::new();
    if !targets.iter().all(|&target| {
        unique_targets.insert(target)
            && tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
    }) {
        return Err(EffectError::Illegal(
            "Qixing weather targets are no longer legal".into(),
        ));
    }
    let mut events = Vec::new();
    for target in targets {
        events.extend(add_source_turn_bound_skill(
            tx,
            target,
            helper_skill_def,
            player,
        )?);
    }
    Ok(events)
}

pub(crate) fn resolve_nzry_longnu(
    tx: &mut Transaction,
    player: PlayerId,
    fire_skill_def: engine_api::SkillDefId,
    thunder_skill_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if fire_skill_def.is_none() || thunder_skill_def.is_none() || stage > 1 {
        return Err(EffectError::ContentFault(
            "Longnu requires two helper skills and a valid stage".into(),
        ));
    }
    if stage == 0 {
        let fire = {
            let state = tx.state();
            let instance = state
                .skills
                .get(skill)
                .ok_or_else(|| EffectError::ContentFault("Longnu skill missing".into()))?;
            if instance.owner != player
                || !state.skill_effectively_enabled(instance)
                || state.turn.current != player
                || state.turn.phase != Phase::Play
                || !state
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            !instance.nzry_longnu_fire_branch()
        };
        tx.state_mut()
            .skills
            .get_mut(skill)
            .unwrap()
            .set_nzry_longnu_fire_branch(fire);
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveNzryLongnu {
                player,
                fire_skill_def,
                thunder_skill_def,
                stage: 1,
                skill,
            });
        if fire {
            return super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| Vec::new());
        }
        return change_max_hp(tx, player, -1);
    }

    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let fire = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| instance.owner == player)
        .is_some_and(|instance| instance.nzry_longnu_fire_branch());
    super::run_effects(
        tx,
        vec![
            EffectRequest::DrawCards { player, count: 1 },
            EffectRequest::AddSkill {
                player,
                skill_def: if fire {
                    fire_skill_def
                } else {
                    thunder_skill_def
                },
            },
        ],
    )
    .map(|_| Vec::new())
}
