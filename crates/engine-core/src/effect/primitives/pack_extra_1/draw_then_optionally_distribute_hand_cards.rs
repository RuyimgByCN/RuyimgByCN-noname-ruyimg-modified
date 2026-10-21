//! Effect primitives for `draw_then_optionally_distribute_hand_cards` and related helpers.

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
    all.extend(super::super::run_effects(
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
            super::super::run_effects(
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
            super::super::run_effects(
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
                return super::super::run_effects(
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
            super::super::run_effects(
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
