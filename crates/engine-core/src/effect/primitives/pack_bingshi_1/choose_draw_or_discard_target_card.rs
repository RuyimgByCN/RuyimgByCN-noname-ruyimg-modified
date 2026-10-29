//! Effect primitives for `begin_choose_compare_selected_hand_card` and related helpers.

use super::*;

pub(crate) fn choose_draw_or_discard_target_card(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    draw_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if draw_count == 0
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == player)
    {
        return Err(EffectError::ContentFault(
            "comparison win choice requires its originating skill".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let can_discard = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        && !crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .is_empty();
    if !can_discard {
        return draw_cards(tx, player, draw_count);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseDrawOrDiscardTargetCard,
        vec![player.raw(), target.raw(), draw_count, skill.raw()],
    )
}

pub(crate) fn record_compare_failure_seat_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    failure_count: u32,
    lose_hp: u32,
    damage: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<EffectRequest>, EffectError> {
    if failure_count != 2 || lose_hp == 0 || damage == 0 {
        return Err(EffectError::ContentFault(
            "comparison failure path requires positive second-failure values".into(),
        ));
    }
    let instance = tx
        .state()
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("comparison active skill missing".into()))?;
    if instance.owner != player {
        return Err(EffectError::ContentFault(
            "comparison active skill owner mismatch".into(),
        ));
    }
    let recorded = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .and_then(|instance| instance.record_active_compare_failure(target))
        .ok_or_else(|| EffectError::Illegal("comparison failure history changed".into()))?;
    if recorded < failure_count as usize {
        return Ok(Vec::new());
    }
    if recorded != failure_count as usize {
        return Err(EffectError::Illegal(
            "comparison failure threshold was already reached".into(),
        ));
    }
    let failures = tx
        .state()
        .skills
        .get(skill)
        .and_then(|instance| instance.active_compare_failures())
        .ok_or_else(|| EffectError::EngineFault("comparison failure history missing".into()))?;
    let [first, second] = failures.as_slice() else {
        return Err(EffectError::EngineFault(
            "comparison failure path requires exactly two targets".into(),
        ));
    };
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated comparison skill")
        .active_turn_banned = true;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ChooseSeatPathDamage {
            player,
            first: *first,
            second: *second,
            amount: damage,
            skill,
        });
    Ok(vec![EffectRequest::DealDamage {
        source: PlayerId::NONE,
        target: player,
        amount: lose_hp,
        is_lose_hp: true,
        nature: rule_api::damage_nature::NORMAL,
    }])
}

pub(crate) fn comparison_seat_paths(
    state: &crate::state::GameState,
    first: PlayerId,
    second: PlayerId,
) -> Option<(Vec<PlayerId>, Vec<PlayerId>)> {
    let mut order: Vec<_> = state.players.iter_live().collect();
    order.sort_by_key(|player| player.seat.raw());
    let first_index = order.iter().position(|player| player.id == first)?;
    let second_index = order.iter().position(|player| player.id == second)?;
    if first_index == second_index || order.len() < 2 {
        return None;
    }
    let (start, end) = if first_index < second_index {
        (first_index, second_index)
    } else {
        (second_index, first_index)
    };
    let build = |step: isize| {
        let mut index = start;
        let mut path = vec![order[start].id];
        while index != end {
            index = (index as isize + step).rem_euclid(order.len() as isize) as usize;
            path.insert(0, order[index].id);
        }
        path.into_iter()
            .filter(|&player| {
                player == first
                    || player == second
                    || state
                        .players
                        .get(player)
                        .is_some_and(|participant| participant.alive)
            })
            .collect::<Vec<_>>()
    };
    Some((build(-1), build(1)))
}

pub(crate) fn ordered_path_damage(
    tx: &mut Transaction,
    player: PlayerId,
    path: Vec<PlayerId>,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets = path
        .into_iter()
        .filter(|&target| {
            target != player
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
        })
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    super::super::run_effects(
        tx,
        vec![EffectRequest::DealDamageEachInOrder {
            source: player,
            targets,
            amount,
            skill,
        }],
    )
    .map(|_| Vec::new())
}

pub(crate) fn choose_seat_path_damage(
    tx: &mut Transaction,
    player: PlayerId,
    first: PlayerId,
    second: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == player)
    {
        return Err(EffectError::ContentFault(
            "seat-path damage requires its originating skill".into(),
        ));
    }
    let Some((first_path, second_path)) = comparison_seat_paths(tx.state(), first, second) else {
        return Ok(Vec::new());
    };
    if first_path.is_empty() || second_path.is_empty() {
        return Ok(Vec::new());
    }
    if first_path.len() < second_path.len() {
        return ordered_path_damage(tx, player, first_path, amount, skill);
    }
    if second_path.len() < first_path.len() {
        return ordered_path_damage(tx, player, second_path, amount, skill);
    }
    let mut payload = vec![
        player.raw(),
        amount,
        skill.raw(),
        first_path.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(first_path.into_iter().map(PlayerId::raw));
    payload.extend(second_path.into_iter().map(PlayerId::raw));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseSeatPathDamage,
        payload,
    )
}

pub(crate) fn continue_damage_each_in_order(
    tx: &mut Transaction,
    source: PlayerId,
    targets: Vec<PlayerId>,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<EffectRequest>, EffectError> {
    if amount == 0
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == source)
    {
        return Err(EffectError::ContentFault(
            "ordered damage requires its originating skill".into(),
        ));
    }
    let Some((index, target)) = targets.iter().copied().enumerate().find(|&(_, target)| {
        target != source
            && tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
    }) else {
        return Ok(Vec::new());
    };
    let remaining = targets[index + 1..].to_vec();
    if !remaining.is_empty() {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DealDamageEachInOrder {
                source,
                targets: remaining,
                amount,
                skill,
            },
        );
    }
    Ok(vec![EffectRequest::DealDamage {
        source,
        target,
        amount,
        is_lose_hp: false,
        nature: rule_api::damage_nature::NORMAL,
    }])
}

pub(crate) fn gain_highest_rank_sha_from_event_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: &[CardId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "comparison gain requires its skill instance".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let max_rank = cards
        .iter()
        .filter_map(|card| tx.state().cards.get(*card))
        .filter(|card| card.zone == discard && card.kind.is_sha())
        .map(|card| card.rank)
        .max();
    let Some(max_rank) = max_rank else {
        return Ok(Vec::new());
    };
    let gain: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                card.zone == discard && card.kind.is_sha() && card.rank == max_rank
            })
        })
        .collect();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("comparison gain hand missing".into()))?;
    move_cards(tx, gain, hand, Some(discard))
}

pub(crate) fn gain_event_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: &[CardId],
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|state| state.alive)
    {
        return Ok(Vec::new());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let gain: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| state.zone == discard)
        })
        .collect();
    if gain.is_empty() {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("event-card recipient hand missing".into()))?;
    move_cards(tx, gain, hand, Some(discard))
}

/// Select the one physical result card awarded by legacy two-player `zongshi` semantics.
pub(crate) fn hand_compare_result_card(
    player: PlayerId,
    initiator: PlayerId,
    target: PlayerId,
    winner: PlayerId,
    cards: &[CardId],
) -> Option<CardId> {
    let [initiator_card, target_card] = cards else {
        return None;
    };
    if winner != PlayerId::NONE && winner != initiator && winner != target {
        return None;
    }
    if player == initiator {
        Some(if winner == player {
            *target_card
        } else {
            *initiator_card
        })
    } else if player == target {
        Some(if winner == player {
            *initiator_card
        } else {
            *target_card
        })
    } else {
        None
    }
}

pub(crate) fn gain_hand_compare_result_card(
    tx: &mut Transaction,
    player: PlayerId,
    initiator: PlayerId,
    target: PlayerId,
    winner: PlayerId,
    cards: &[CardId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "comparison result gain requires its skill instance".into(),
        ));
    }
    let Some(card) = hand_compare_result_card(player, initiator, target, winner, cards) else {
        return Ok(Vec::new());
    };
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state_card| state_card.zone == discard)
    {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("comparison result gain hand missing".into()))?;
    move_cards(tx, vec![card], hand, Some(discard))
}

pub(crate) fn choose_add_skill_bound_card_target(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(bound_target) = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .and_then(|instance| instance.bound_target())
    else {
        return Ok(Vec::new());
    };
    if !tx
        .state()
        .players
        .get(bound_target)
        .is_some_and(|participant| participant.alive)
        || !crate::command::play::saved_card_extra_target_is_legal(tx, player, card, bound_target)
    {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        vec![bound_target.raw()],
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseAddSkillBoundCardTarget,
        vec![player.raw(), card.raw(), bound_target.raw(), skill.raw()],
    )
}

pub(crate) fn choose_add_or_remove_current_card_target(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let (add_targets, remove_targets) =
        crate::command::play::current_card_target_adjustments(tx, player, card)?;
    let add = !add_targets.is_empty();
    let remove = !remove_targets.is_empty();
    if !add && !remove {
        return remove_skill(tx, skill);
    }
    if add && remove {
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ChooseAddOrRemoveCurrentCardTarget,
            vec![0, player.raw(), card.raw(), skill.raw()],
        );
    }
    let (mode, targets) = if add {
        (0, add_targets)
    } else {
        (1, remove_targets)
    };
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget,
        vec![1, player.raw(), card.raw(), skill.raw(), mode],
    )
}

pub(crate) fn choose_current_trick_targets_to_nullify(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), source, card));
    if !kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick()) {
        return Ok(Vec::new());
    }
    let (_, targets) = crate::command::play::current_card_target_adjustments(tx, source, card)?;
    if targets.len() < 2 {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        targets.len().min(u8::MAX as usize) as u8,
        true,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget,
        vec![2, player.raw(), source.raw(), card.raw(), skill.raw()],
    )
}

pub(crate) fn choose_discard_matching_equip_color_add_skill(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    if !state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "matching-color discard requires its originating skill instance".into(),
        ));
    }
    if player == target
        || !state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || state.turn.current != target
        || state.turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let options = crate::query::discardable_non_basic_matching_equip_color_cards(state, player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardMatchingEquipColorAddSkill,
        vec![player.raw(), target.raw(), skill_def.raw(), skill.raw()],
    )
}

pub(crate) fn add_card_color_lock_skill(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    color_mask: u8,
    skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let color_mask = color_mask & 0b11;
    if color_mask == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || tx.state().players.get(source).is_none()
    {
        return Ok(Vec::new());
    }
    let existing_mask = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|participant| participant.skills.iter())
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .filter(|instance| instance.def == skill_def)
                .and_then(|instance| instance.card_color_lock())
                .map(|(_, mask)| mask)
        })
        .unwrap_or(0);
    add_skill_with_state(
        tx,
        player,
        skill_def,
        Some(crate::state::SkillInstance::card_color_lock_state(
            source,
            existing_mask | color_mask,
        )),
    )
}

pub(crate) fn use_virtual_sha_from_card_color_lock_source_if_no_sha(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(source) = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .and_then(|instance| instance.card_color_lock())
        .map(|(source, _)| source)
    else {
        return Ok(Vec::new());
    };
    if tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || crate::query::sha_play_phase_use_count(tx.state()) != 0
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive && !participant.dying)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    try_use_virtual_sha_on_without_distance(tx, source, player, true, 0)
}
