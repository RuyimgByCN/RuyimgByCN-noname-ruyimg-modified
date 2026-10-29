//! Effect primitives for `replace_draw_with_distinct_suit_selection` and related helpers.

use super::*;

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
