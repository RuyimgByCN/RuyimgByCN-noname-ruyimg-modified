//! Effect primitives for `draw_then_choose_give_or_discard_hand` and related helpers.

use super::*;

pub(crate) fn draw_then_choose_give_or_discard_hand(
    tx: &mut Transaction,
    player: PlayerId,
    draw_count: u32,
    payment_count: u32,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
    recipient_scope: DrawTargetScope,
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
    if payment_count == 0 || payment_count > u8::MAX as u32 {
        return Err(EffectError::ContentFault(
            "give-or-discard hand payment count must be in 1..=255".into(),
        ));
    }
    if !ready_to_choose {
        let events = if draw_count == 0 {
            Vec::new()
        } else {
            draw_cards(tx, player, draw_count)?
        };
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DrawThenChooseGiveOrDiscardHand {
                player,
                draw_count,
                payment_count,
                skill,
                ready_to_choose: true,
            },
        );
        return Ok(events);
    }

    let hand_cards = hand_cards_sorted(tx, player);
    if hand_cards.len() < payment_count as usize {
        return open_draw_give_or_discard_hand_discard(tx, player, payment_count, skill);
    }
    let has_recipient = tx
        .state()
        .players
        .iter_live()
        .any(|target| player_target_matches(tx.state(), player, target.id, recipient_scope));
    let has_discard = !discardable_hand_cards_sorted(tx, player).is_empty();
    let mut branches = Vec::with_capacity(2);
    if has_recipient {
        branches.push(1);
    }
    if has_discard {
        branches.push(2);
    }
    match branches.as_slice() {
        [] => Ok(Vec::new()),
        [2] => open_draw_give_or_discard_hand_discard(tx, player, payment_count, skill),
        _ => open_choice(
            tx,
            player,
            1,
            1,
            false,
            branches,
            ChoiceKind::Custom,
            ContinuationKind::DrawThenChooseGiveOrDiscardHand,
            vec![
                0,
                player.raw(),
                payment_count,
                skill.raw(),
                recipient_scope.tag() as u32,
            ],
        ),
    }
}

pub(crate) fn finish_optional_hand_distribution(
    tx: &mut Transaction,
    player: PlayerId,
    give_cap: u32,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cards.len() != targets.len() || cards.len() > give_cap as usize {
        return Err(EffectError::EngineFault(
            "invalid optional hand-distribution plan".into(),
        ));
    }
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("giver hand zone missing".into()))?;
    let current: std::collections::BTreeSet<_> =
        hand_cards_sorted(tx, player).into_iter().collect();
    let mut distinct = std::collections::BTreeSet::new();
    if cards
        .iter()
        .any(|card| !distinct.insert(*card) || !current.contains(card))
    {
        return Err(EffectError::Illegal(
            "planned hand-distribution card is duplicated or stale".into(),
        ));
    }

    let mut groups: Vec<(PlayerId, ZoneId, Vec<CardId>)> = Vec::new();
    for (card, target) in cards.into_iter().zip(targets) {
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "planned hand-distribution recipient is stale".into(),
            ));
        }
        if let Some((_, _, grouped_cards)) = groups
            .iter_mut()
            .find(|(group_target, _, _)| *group_target == target)
        {
            grouped_cards.push(card);
            continue;
        }
        let to = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::EngineFault("recipient hand zone missing".into()))?;
        groups.push((target, to, vec![card]));
    }

    let mut events = Vec::new();
    for (_, to, cards) in groups {
        events.extend(give_cards_as(tx, player, cards, to, from)?);
    }
    Ok(events)
}
