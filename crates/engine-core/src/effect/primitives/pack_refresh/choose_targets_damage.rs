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
