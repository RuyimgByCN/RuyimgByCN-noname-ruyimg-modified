//! Effect primitives for `choose_targets_damage` and related helpers.

use super::*;

pub(crate) fn choose_targets_damage(
    tx: &mut Transaction,
    player: PlayerId,
    min_count: u8,
    max_count: u8,
    scope: DrawTargetScope,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if min_count == 0
        || min_count > max_count
        || amount == 0
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Err(EffectError::Illegal(
            "target-damage choice requires a valid range, positive damage, and enabled skill"
                .into(),
        ));
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| player_target_matches(tx.state(), player, target, scope))
        .collect();
    if targets.len() < usize::from(min_count) {
        return Ok(Vec::new());
    }
    let available = targets.len().min(usize::from(u8::MAX)) as u8;
    open_choice(
        tx,
        player,
        min_count,
        max_count.min(available),
        false,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetsDamage,
        vec![
            player.raw(),
            u32::from(min_count),
            u32::from(max_count),
            u32::from(scope.tag()),
            amount,
            skill.raw(),
        ],
    )
}

pub(crate) fn choose_optional_targets_draw(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    min_count: u8,
    max_count: u8,
    scope: DrawTargetScope,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0
        || min_count == 0
        || min_count > max_count
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| player_target_matches(tx.state(), player, target, scope))
        .collect();
    if targets.len() < usize::from(min_count) {
        return Ok(Vec::new());
    }
    let available = targets.len().min(usize::from(u8::MAX)) as u8;
    open_choice(
        tx,
        player,
        min_count,
        max_count.min(available),
        true,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetDraw,
        vec![
            player.raw(),
            count,
            u32::from(min_count),
            u32::from(max_count),
            u32::from(scope.tag()),
            skill.raw(),
        ],
    )
}

pub(crate) fn choose_repolu_targets_draw(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || tx.state().players.get(player).is_none()
    {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    let max_count = targets.len().min(usize::from(u8::MAX)) as u8;
    open_choice(
        tx,
        player,
        1,
        max_count,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseRepoluTargetsDraw,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_zhuiyi(
    tx: &mut Transaction,
    player: PlayerId,
    killer: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 追忆在拥有者阵亡时结算，只要求座位存在而非存活。
    if tx.state().players.get(player).is_none() {
        return Ok(Vec::new());
    }
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && target != killer)
        .collect();
    GuardedChoice::new(
        skill,
        Participants::owner_any_state(player),
        ContinuationKind::ResolveZhuiyi,
        vec![player.raw(), killer.raw(), skill.raw()],
    )
    .shape(ChoiceShape::one_optional())
    .open_targets(tx, others)
}

pub(crate) fn resolve_xinfu_guolun(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    player_card: CardId,
    target_card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid_context = originating_skill_is_enabled(tx, player, skill)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Play
        && target != player
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive);
    if !valid_context {
        return Err(EffectError::Illegal("Guolun context changed".into()));
    }
    if player_card.is_none() && target_card.is_none() {
        if tx.state().hand_count(player) == 0 {
            return Err(EffectError::Illegal("Guolun owner hand is empty".into()));
        }
        let options = tx.state().hand_cards(target);
        if options.is_empty() {
            return Err(EffectError::Illegal("Guolun target hand is empty".into()));
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ResolveXinfuGuolun,
            vec![0, player.raw(), target.raw(), skill.raw()],
        );
    }
    if player_card.is_none() || target_card.is_none() {
        return Err(EffectError::EngineFault(
            "Guolun exchange requires both cards".into(),
        ));
    }

    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("Guolun target hand zone missing".into()))?;
    let player_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("Guolun owner hand zone missing".into()))?;
    let target_card_state = tx
        .state()
        .cards
        .get(target_card)
        .filter(|card| card.owner == target && card.zone == target_hand)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("Guolun target card moved".into()))?;
    let player_card_state = tx
        .state()
        .cards
        .get(player_card)
        .filter(|card| {
            card.owner == player
                && [ZoneKind::Hand, ZoneKind::Equip]
                    .into_iter()
                    .any(|kind| tx.state().zones.find(kind, player) == Some(card.zone))
        })
        .cloned()
        .ok_or_else(|| EffectError::Illegal("Guolun owner card moved".into()))?;
    let player_number = match crate::query::query(
        tx.state(),
        &QueryRequest::CardAttribute {
            card: player_card,
            attr: 1,
        },
        &mut crate::query::QueryContext::new(),
    ) {
        QueryResponse::Int(number) => number,
        _ => {
            return Err(EffectError::EngineFault(
                "Guolun owner card number query failed".into(),
            ));
        }
    };
    let target_number = match crate::query::query(
        tx.state(),
        &QueryRequest::CardAttribute {
            card: target_card,
            attr: 1,
        },
        &mut crate::query::QueryContext::new(),
    ) {
        QueryResponse::Int(number) => number,
        _ => {
            return Err(EffectError::EngineFault(
                "Guolun target card number query failed".into(),
            ));
        }
    };

    let mut events = move_cards(
        tx,
        vec![player_card],
        target_hand,
        Some(player_card_state.zone),
    )?;
    events.extend(move_cards(
        tx,
        vec![target_card],
        player_hand,
        Some(target_card_state.zone),
    )?);

    let drawer = if player_number < target_number {
        player
    } else if target_number < player_number {
        target
    } else {
        PlayerId::NONE
    };
    if drawer.is_some() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::DrawCardsFromSkill {
                player: drawer,
                count: 1,
                skill,
            });
    }
    Ok(events)
}

pub(crate) fn player_target_matches(
    state: &crate::state::GameState,
    chooser: PlayerId,
    target: PlayerId,
    scope: DrawTargetScope,
) -> bool {
    let Some(target_player) = state.players.get(target).filter(|player| player.alive) else {
        return false;
    };
    if !matches!(
        scope,
        DrawTargetScope::Any
            | DrawTargetScope::MinHand
            | DrawTargetScope::SameHp
            | DrawTargetScope::HandEmpty
            | DrawTargetScope::Wounded
    ) && target == chooser
    {
        return false;
    }
    let sex = state
        .content
        .character(target_player.character_def)
        .map(|character| character.sex.as_str())
        .unwrap_or("");
    match scope {
        DrawTargetScope::Other | DrawTargetScope::Any => true,
        DrawTargetScope::MaleOther => matches!(sex, "male" | "double"),
        DrawTargetScope::FemaleOther => matches!(sex, "female" | "double"),
        DrawTargetScope::MinHand => {
            let target_hand = state.hand_count(target);
            state
                .players
                .iter_live()
                .filter(|player| player.alive)
                .all(|player| target_hand <= state.hand_count(player.id))
        }
        DrawTargetScope::SameHp => state
            .players
            .get(chooser)
            .filter(|player| player.alive)
            .is_some_and(|player| target_player.hp == player.hp),
        DrawTargetScope::HandEmpty => state.hand_count(target) == 0,
        DrawTargetScope::Wounded => target_player.hp < target_player.max_hp as i32,
        DrawTargetScope::HandLteChooser => {
            state
                .players
                .get(chooser)
                .is_some_and(|player| player.alive)
                && state.hand_count(target) <= state.hand_count(chooser)
        }
    }
}

pub(crate) fn choose_discard_hand_add_skill_by_color(
    tx: &mut Transaction,
    player: PlayerId,
    red_skill_def: engine_api::SkillDefId,
    black_skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.zone == hand)
        })
        .collect();
    if options.is_empty() {
        return Ok(vec![]);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DiscardHandAddSkillByColor,
        vec![player.raw(), red_skill_def.raw(), black_skill_def.raw()],
    )
}

pub(crate) fn reveal_top_gain_non_basic_or_swap_basic(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|player| player.alive)
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
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
        .and_then(|zone| zone.cards.last().copied())
    else {
        return Ok(Vec::new());
    };
    let is_basic = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card| card.kind.is_basic());
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
    let mut events = vec![
        DomainEvent::CardsMoved {
            cards: vec![card],
            from: draw,
            to: processing,
            reason: CardMoveReason::Unspecified,
        },
        DomainEvent::CardsRevealed {
            cards: vec![card],
            to: Vec::new(),
        },
    ];
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    if !is_basic {
        events.extend(move_cards(tx, vec![card], hand, Some(processing))?);
        return Ok(events);
    }
    let options = tx.state().hand_cards(player);
    if options.is_empty() {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
        events.extend(move_cards(tx, vec![card], discard, Some(processing))?);
        return Ok(events);
    }
    events.extend(open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::RevealTopGainNonBasicOrSwapBasic,
        vec![player.raw(), card.raw()],
    )?);
    Ok(events)
}

pub(crate) fn reveal_top_gain_if_color(
    tx: &mut Transaction,
    player: PlayerId,
    color_mask: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !matches!(color_mask, 1 | 2) {
        return Err(EffectError::ContentFault(
            "top-card color must be red or black".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|player| player.alive)
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
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
        .and_then(|zone| zone.cards.last().copied())
    else {
        return Ok(Vec::new());
    };
    let matches_color = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card| crate::query::card_color_mask_for_suit(card.suit) == color_mask);
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
    let mut events = vec![
        DomainEvent::CardsMoved {
            cards: vec![card],
            from: draw,
            to: processing,
            reason: CardMoveReason::Unspecified,
        },
        DomainEvent::CardsRevealed {
            cards: vec![card],
            to: Vec::new(),
        },
    ];
    let destination = if matches_color {
        tx.state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("no hand".into()))?
    } else {
        tx.state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?
    };
    events.extend(move_cards(tx, vec![card], destination, Some(processing))?);
    Ok(events)
}

pub(crate) fn choose_gain_top_cards_by_suit(
    tx: &mut Transaction,
    player: PlayerId,
    count: u8,
    suit: Suit,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 || suit == Suit::None {
        return Err(EffectError::ContentFault(
            "top-card suit choice requires a positive count and concrete suit".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|player| player.alive)
    {
        return Err(EffectError::Illegal(
            "top-card suit choice requires a living player".into(),
        ));
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let available = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| zone.cards.len())
        .unwrap_or(0);
    if available < usize::from(count) {
        reshuffle_discard_into_draw(tx)?;
    }
    let zone = tx
        .state()
        .zones
        .get(draw)
        .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
    let take = usize::from(count).min(zone.cards.len());
    if take == 0 {
        return Ok(Vec::new());
    }
    let top: Vec<CardId> = zone.cards[zone.cards.len() - take..].to_vec();
    let options: Vec<u32> = top
        .iter()
        .filter_map(|card| {
            tx.state()
                .cards
                .get(*card)
                .filter(|card| crate::query::effective_card_suit(tx.state(), player, card) == suit)
                .map(|card| card.id.raw())
        })
        .collect();
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: top.clone(),
        to: vec![player],
    }];
    if options.is_empty() {
        return Ok(events);
    }
    let mut payload = vec![
        player.raw(),
        crate::command::judge::suit_tag(suit),
        take as u32,
    ];
    payload.extend(top.iter().map(|card| card.raw()));
    events.extend(open_choice(
        tx,
        player,
        1,
        options.len() as u8,
        true,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGainTopCardsBySuit,
        payload,
    )?);
    Ok(events)
}

pub(crate) fn discard_all_hand_and_equip(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let cards = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if cards.is_empty() {
        return Ok(vec![]);
    }
    discard_cards(tx, player, cards)
}

pub(crate) fn discard_all_hand_equip_judge(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "dead or missing player zones cannot be discarded".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let batches: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
        .into_iter()
        .filter_map(|kind| {
            let zone = tx.state().zones.find(kind, player)?;
            let cards = tx.state().zones.get(zone)?.cards.clone();
            (!cards.is_empty()).then_some((zone, cards))
        })
        .collect();
    let mut events = Vec::new();
    for (from, cards) in batches {
        events.extend(move_cards_with_reason(
            tx,
            cards,
            discard,
            Some(from),
            CardMoveReason::Discard,
        )?);
    }
    Ok(events)
}

pub(crate) fn damage_then_discard_all_judge(
    tx: &mut Transaction,
    player: PlayerId,
    ready_to_discard: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(participant) = tx.state().players.get(player) else {
        return Err(EffectError::Illegal("missing player".into()));
    };
    if ready_to_discard {
        return discard_all_judge(tx, player);
    }
    if !participant.alive {
        return Err(EffectError::Illegal(
            "dead player cannot start self-damage".into(),
        ));
    }
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::DamageThenDiscardAllJudge {
            player,
            ready_to_discard: true,
        },
    );
    super::super::run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target: player,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
}

pub(crate) fn replace_draw_gain_top_two_add_skills_if_different_colors(
    tx: &mut Transaction,
    player: PlayerId,
    first_skill_def: engine_api::SkillDefId,
    second_skill_def: engine_api::SkillDefId,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || tx.state().turn.current != player
            || tx.state().turn.phase != Phase::Draw
            || tx.state().turn.draw_phase_is_fixed()
        {
            return Err(EffectError::Illegal(
                "invalid top-two draw replacement context".into(),
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
            < 2
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let zone = tx
            .state()
            .zones
            .get(draw)
            .ok_or_else(|| EffectError::EngineFault("draw pile disappeared".into()))?;
        let take = 2.min(zone.cards.len());
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
            EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                player,
                first_skill_def,
                second_skill_def,
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
    if stage == 1 {
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
        if cards.is_empty()
            || cards.iter().any(|card| {
                !tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.zone == processing)
            })
        {
            return Err(EffectError::Illegal(
                "top-two revealed cards left processing".into(),
            ));
        }
        if cards.len() == 2
            && crate::query::effective_card_color_mask(tx.state(), player, cards[0])
                != crate::query::effective_card_color_mask(tx.state(), player, cards[1])
        {
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ReplaceDrawGainTopTwoAddSkillsIfDifferentColors {
                    player,
                    first_skill_def,
                    second_skill_def,
                    stage: 2,
                    cards: cards.clone(),
                    skill,
                },
            );
        }
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
        return move_cards(tx, cards, hand, Some(processing));
    }
    if stage != 2 {
        return Err(EffectError::ContentFault(
            "bad top-two draw replacement stage".into(),
        ));
    }
    let mut events = add_skill_until_turn_end(tx, player, first_skill_def)?;
    events.extend(add_skill_until_turn_end(tx, player, second_skill_def)?);
    Ok(events)
}

pub(crate) fn resolve_new_reluoyi(
    tx: &mut Transaction,
    player: PlayerId,
    buff_skill_def: engine_api::SkillDefId,
    stage: u8,
    cards: Vec<CardId>,
    accepted: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if buff_skill_def.is_none() {
        return Err(EffectError::ContentFault(
            "New Reluoyi requires a damage helper skill".into(),
        ));
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || tx.state().turn.current != player
            || tx.state().turn.phase != Phase::Draw
            || tx.state().turn.draw_phase_is_fixed()
        {
            return Err(EffectError::Illegal(
                "invalid New Reluoyi draw replacement context".into(),
            ));
        }
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
            < 3
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let zone = tx
            .state()
            .zones
            .get(draw)
            .ok_or_else(|| EffectError::EngineFault("draw pile disappeared".into()))?;
        let take = 3.min(zone.cards.len());
        if take == 0 {
            return Ok(Vec::new());
        }
        let cards = zone.cards[zone.cards.len() - take..].to_vec();
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveNewReluoyi {
                player,
                buff_skill_def,
                stage: 1,
                cards: cards.clone(),
                accepted: false,
                skill,
            });
        let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
        events.push(DomainEvent::CardsRevealed {
            cards,
            to: Vec::new(),
        });
        return Ok(events);
    }
    if stage == 3 {
        return add_source_turn_bound_skill(tx, player, buff_skill_def, player);
    }
    if cards.is_empty()
        || cards.iter().any(|card| {
            !tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.zone == processing)
        })
    {
        return Err(EffectError::Illegal(
            "New Reluoyi revealed cards left processing".into(),
        ));
    }
    if stage == 1 {
        let mut payload = vec![
            player.raw(),
            buff_skill_def.raw(),
            skill.raw(),
            cards.len() as u32,
        ];
        payload.extend(cards.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::ResolveNewReluoyi,
            payload,
        );
    }
    if stage != 2 {
        return Err(EffectError::ContentFault(
            "bad New Reluoyi resolution stage".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    if !accepted {
        return move_cards(tx, cards, discard, Some(processing));
    }

    tx.state_mut().turn.set_draw_phase_fixed_count(0);
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def,
            stage: 3,
            cards: Vec::new(),
            accepted: true,
            skill,
        });
    let (gain, rest): (Vec<_>, Vec<_>) = cards.into_iter().partition(|card| {
        tx.state().cards.get(*card).is_some_and(|card| {
            card.kind.is_basic() || matches!(card.kind, CardKind::TrickJuedou | CardKind::Weapon)
        })
    });
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
    let mut events = Vec::new();
    if !gain.is_empty() {
        events.extend(move_cards(tx, gain, hand, Some(processing))?);
    }
    if !rest.is_empty() {
        events.extend(move_cards(tx, rest, discard, Some(processing))?);
    }
    Ok(events)
}

pub(crate) fn resolve_remoshi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let pile_skill_def = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("invalid Remoshi skill context".into()))?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if helper_skill_def.is_none() {
                return Err(EffectError::ContentFault(
                    "Remoshi requires its presentation helper".into(),
                ));
            }
            let source_cards: Vec<_> = tx
                .state()
                .resolution
                .sha_source_cards(player, card)
                .into_iter()
                .filter_map(|card| {
                    tx.state()
                        .cards
                        .get(card)
                        .filter(|card| card.def != engine_api::CardDefId::NONE)
                        .and_then(|card| {
                            tx.state()
                                .zones
                                .get(card.zone)
                                .filter(|zone| {
                                    matches!(
                                        zone.kind,
                                        ZoneKind::Processing | ZoneKind::DiscardPile
                                    )
                                })
                                .map(|_| (card.id, card.zone))
                        })
                })
                .collect();
            if source_cards.is_empty() {
                return Err(EffectError::Illegal(
                    "Remoshi Sha sources are no longer gainable".into(),
                ));
            }
            let has_armor_or_mount = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .is_some_and(|zone| {
                    zone.cards.iter().any(|card| {
                        tx.state().cards.get(*card).is_some_and(|card| {
                            matches!(
                                card.kind,
                                CardKind::Armor | CardKind::HorseOff | CardKind::HorseDef
                            )
                        })
                    })
                });
            if !has_armor_or_mount {
                return Ok(Vec::new());
            }
            let mut events = add_skill(tx, target, helper_skill_def)?;
            let pile = ensure_skill_pile_zone(tx, target, pile_skill_def)?;
            for (card, from) in source_cards {
                events.extend(move_cards(tx, vec![card], pile, Some(from))?);
            }
            Ok(events)
        }
        1 => {
            let cards = tx.state().skill_pile_cards(target, pile_skill_def);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let from = ensure_skill_pile_zone(tx, target, pile_skill_def)?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Remoshi owner hand missing".into()))?;
            give_cards(tx, cards, to, from)
        }
        _ => Err(EffectError::ContentFault(
            "unknown Remoshi resolution action".into(),
        )),
    }
}

pub(crate) fn resolve_xinyaoming(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid Xinyaoming skill context".into(),
        ));
    }
    let used_mask = tx
        .state()
        .skills
        .get(skill)
        .map_or(u8::MAX, |instance| instance.xinyaoming_used_mask());
    if target.is_none() {
        if action != 0 {
            return Err(EffectError::ContentFault(
                "Xinyaoming target choice requires action zero".into(),
            ));
        }
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| {
                !crate::query::xinyaoming_available_actions(tx.state(), player, *target, used_mask)
                    .is_empty()
            })
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        let mut payload = vec![0, player.raw(), skill.raw(), options.len() as u32];
        payload.extend(options.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::ResolveXinyaoming,
            payload,
        );
    }
    let actions = crate::query::xinyaoming_available_actions(tx.state(), player, target, used_mask);
    if action == 0 {
        return match actions.as_slice() {
            [] => Err(EffectError::Illegal(
                "Xinyaoming target has no available action".into(),
            )),
            [only] => resolve_xinyaoming(tx, player, target, *only, skill),
            _ => {
                let mut payload = vec![
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    actions.len() as u32,
                ];
                payload.extend(actions.iter().map(|action| u32::from(*action)));
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    actions.into_iter().map(u32::from).collect(),
                    ChoiceKind::Custom,
                    ContinuationKind::ResolveXinyaoming,
                    payload,
                )
            }
        };
    }
    if !actions.contains(&action) {
        return Err(EffectError::Illegal(
            "Xinyaoming action is unavailable".into(),
        ));
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("Xinyaoming skill disappeared".into()))?
        .mark_xinyaoming_action(action);
    match action {
        crate::query::XINYAOMING_DRAW => draw_cards(tx, target, 1),
        crate::query::XINYAOMING_DISCARD_HAND => {
            choose_discard_target_card(tx, player, target, rule_api::TargetCardScope::Hand)
        }
        crate::query::XINYAOMING_DISCARD_DRAW => {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                target,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let max = options.len().min(2) as u8;
            let mut payload = vec![
                2,
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                max,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveXinyaoming,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown Xinyaoming action".into(),
        )),
    }
}

pub(crate) fn rehuaiyi_eligible_colors(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<u8> {
    let hand = state.hand_cards(player);
    let discardable: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(state, player)
            .into_iter()
            .collect();
    [0b01, 0b10]
        .into_iter()
        .filter(|color| {
            let matching: Vec<_> = hand
                .iter()
                .copied()
                .filter(|card| {
                    crate::query::effective_card_color_mask(state, player, *card) == *color
                })
                .collect();
            !matching.is_empty() && matching.iter().all(|card| discardable.contains(card))
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_rehuaiyi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    color: u8,
    discarded_count: u32,
    targets: Vec<PlayerId>,
    mut index: u32,
    gained_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_valid = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if !skill_valid
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && !participant.dying)
    {
        return Err(EffectError::Illegal(
            "invalid Rehuaiyi skill context".into(),
        ));
    }

    match stage {
        0 if color == 0 => {
            if discarded_count != 0 || !targets.is_empty() || index != 0 || gained_count != 0 {
                return Err(EffectError::ContentFault(
                    "bad initial Rehuaiyi progress".into(),
                ));
            }
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "Rehuaiyi requires a nonempty hand".into(),
                ));
            }
            let color_mask = cards.iter().fold(0, |mask, card| {
                mask | crate::query::effective_card_color_mask(tx.state(), player, *card)
            }) & 0b11;
            let mut events = vec![DomainEvent::CardsRevealed {
                cards,
                to: Vec::new(),
            }];
            if matches!(color_mask, 0b01 | 0b10) {
                events.extend(draw_cards(tx, player, 1)?);
                return Ok(events);
            }
            if color_mask != 0b11 {
                return Ok(events);
            }

            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("Rehuaiyi skill disappeared".into()))?;
            instance.play_phase_use_count = instance.play_phase_use_count.max(2);

            let colors = rehuaiyi_eligible_colors(tx.state(), player);
            match colors.as_slice() {
                [] => Ok(events),
                [only] => {
                    events.extend(resolve_rehuaiyi(
                        tx,
                        player,
                        0,
                        *only,
                        0,
                        Vec::new(),
                        0,
                        0,
                        skill,
                    )?);
                    Ok(events)
                }
                _ => {
                    let mut payload = vec![0, player.raw(), skill.raw(), colors.len() as u32];
                    payload.extend(colors.iter().map(|color| u32::from(*color)));
                    events.extend(open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        colors.into_iter().map(u32::from).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::ResolveRehuaiyi,
                        payload,
                    )?);
                    Ok(events)
                }
            }
        }
        0 if matches!(color, 0b01 | 0b10) => {
            let eligible = rehuaiyi_eligible_colors(tx.state(), player);
            if !eligible.contains(&color) {
                return Err(EffectError::Illegal(
                    "selected Rehuaiyi color is no longer discardable".into(),
                ));
            }
            let cards: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    crate::query::effective_card_color_mask(tx.state(), player, *card) == color
                })
                .collect();
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "selected Rehuaiyi color has no hand cards".into(),
                ));
            }
            let count = cards.len().min(u32::MAX as usize) as u32;
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards { player, cards },
                    EffectRequest::ResolveRehuaiyi {
                        player,
                        stage: 1,
                        color: 0,
                        discarded_count: count,
                        targets: Vec::new(),
                        index: 0,
                        gained_count: 0,
                        skill,
                    },
                ],
            )
        }
        1 => {
            if color != 0
                || discarded_count == 0
                || !targets.is_empty()
                || index != 0
                || gained_count != 0
            {
                return Err(EffectError::ContentFault(
                    "bad Rehuaiyi target-choice progress".into(),
                ));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    target != player
                        && !crate::query::gainable_target_cards(
                            tx.state(),
                            player,
                            target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let max = options
                .len()
                .min(discarded_count as usize)
                .min(u8::MAX as usize) as u8;
            let mut payload = vec![
                1,
                player.raw(),
                skill.raw(),
                discarded_count,
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                max,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveRehuaiyi,
                payload,
            )
        }
        2 => {
            if color != 0 || discarded_count != 0 || targets.is_empty() {
                return Err(EffectError::ContentFault(
                    "bad Rehuaiyi gain progress".into(),
                ));
            }
            while let Some(target) = targets.get(index as usize).copied() {
                let options = crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                );
                if options.is_empty() {
                    index = index.saturating_add(1);
                    continue;
                }
                let mut payload = vec![
                    2,
                    player.raw(),
                    skill.raw(),
                    index,
                    gained_count,
                    targets.len() as u32,
                ];
                payload.extend(targets.iter().map(|target| target.raw()));
                payload.push(options.len() as u32);
                payload.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::ResolveRehuaiyi,
                    payload,
                );
            }
            if gained_count >= 2 {
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                );
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown Rehuaiyi resolution stage".into(),
        )),
    }
}
