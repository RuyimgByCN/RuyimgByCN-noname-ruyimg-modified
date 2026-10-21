//! Effect primitives for `discard_all_judge` and related helpers.

use super::*;

pub(crate) fn discard_all_judge(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Judge, player)
        .ok_or_else(|| EffectError::Illegal("no judgment zone".into()))?;
    let cards = tx
        .state()
        .zones
        .get(from)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    if cards.is_empty() {
        return Ok(vec![]);
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    move_cards_with_reason(tx, cards, discard, Some(from), CardMoveReason::Discard)
}

pub(crate) fn completed_judgment_event(
    player: PlayerId,
    result: crate::command::judge::JudgmentResult,
    cause_card: CardId,
) -> DomainEvent {
    DomainEvent::JudgmentResolved {
        player,
        result_card: result.card,
        suit: result.suit,
        rank: result.rank,
        cause_card,
    }
}

pub(crate) fn prepend_event(event: DomainEvent, mut events: Vec<DomainEvent>) -> Vec<DomainEvent> {
    events.insert(0, event);
    events
}

pub(crate) fn judge_set_skill_mark_to_rank(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::ContentFault(
            "judgment rank mark requires the player's enabled skill".into(),
        ));
    }
    let mark = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::ContentFault("judgment rank skill is missing".into()))?;
    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let suit = result.suit;
    let rank = result.rank;
    if rank == 0 {
        return Err(EffectError::ContentFault(
            "judgment produced a rankless card".into(),
        ));
    }
    let mut events = vec![completed_judgment_event(player, result, CardId::NONE)];
    tx.push_event(DomainEvent::RuleNote {
        code: 232,
        payload: vec![
            player.raw(),
            crate::command::judge::suit_tag(suit),
            u32::from(rank),
        ],
    });
    let has_mark = tx
        .state()
        .players
        .get(player)
        .is_some_and(|player| player.marks.iter().any(|(id, _)| *id == mark));
    if has_mark {
        events.extend(remove_mark(tx, player, mark, 0)?);
    }
    events.extend(add_mark(tx, player, mark, i32::from(rank))?);
    Ok(events)
}

pub(crate) fn choose_discard_one_from_each_linked(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::ContentFault(
            "linked-player discard requires the player's enabled skill".into(),
        ));
    }
    let owner_linked = tx.state().players.get(player).is_some_and(|owner| {
        owner.alive
            && owner
                .marks
                .iter()
                .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
    });
    if !owner_linked {
        return Ok(Vec::new());
    }
    let effects = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            tx.state().players.get(target).is_some_and(|participant| {
                participant
                    .marks
                    .iter()
                    .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
            }) && !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquip,
            )
            .is_empty()
        })
        .map(|target| EffectRequest::ChooseDiscardTargetCard {
            player,
            target,
            scope: TargetCardScope::HandEquip,
        })
        .collect();
    super::super::run_effects(tx, effects)
}

pub(crate) fn judge_gain_equipment_by_color(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal("bad judging player".into()));
    }
    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let suit = result.suit;
    let rank = result.rank;
    let judgment_event = completed_judgment_event(player, result, CardId::NONE);
    tx.push_event(DomainEvent::RuleNote {
        code: 232, // standalone skill judgment
        payload: vec![
            player.raw(),
            crate::command::judge::suit_tag(suit),
            rank as u32,
        ],
    });
    let zone_kind = match suit {
        engine_api::Suit::Heart | engine_api::Suit::Diamond => ZoneKind::DrawPile,
        engine_api::Suit::Spade | engine_api::Suit::Club => ZoneKind::DiscardPile,
        engine_api::Suit::None => {
            return Err(EffectError::ContentFault(
                "judgment produced a suitless card".into(),
            ));
        }
    };
    let from = tx
        .state()
        .zones
        .find(zone_kind, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("equipment source zone missing".into()))?;
    let equipment = tx.state().zones.get(from).and_then(|zone| {
        zone.cards.iter().rev().copied().find(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind.is_equip())
        })
    });
    let Some(equipment) = equipment else {
        return Ok(vec![judgment_event]);
    };
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    move_cards(tx, vec![equipment], hand, Some(from))
        .map(|events| prepend_event(judgment_event, events))
}

pub(crate) fn judge_recover_on_suit(
    tx: &mut Transaction,
    source: PlayerId,
    player: PlayerId,
    expected_suit: engine_api::Suit,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    judge_recover_other_on_suit(tx, source, player, player, expected_suit, amount)
}

pub(crate) fn judge_recover_other_on_suit(
    tx: &mut Transaction,
    source: PlayerId,
    judge_player: PlayerId,
    recover_player: PlayerId,
    expected_suit: engine_api::Suit,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(judge_player)
        .is_some_and(|player| player.alive)
    {
        return Err(EffectError::Illegal("bad judging player".into()));
    }
    if !tx
        .state()
        .players
        .get(recover_player)
        .is_some_and(|player| player.alive)
    {
        return Err(EffectError::Illegal("bad recovery player".into()));
    }
    if expected_suit == engine_api::Suit::None {
        return Err(EffectError::ContentFault(
            "judgment recovery requires a physical suit".into(),
        ));
    }
    let result = crate::command::judge::flip_judgment(
        tx,
        judge_player,
        crate::command::judge::JudgmentFavor::Suit(expected_suit),
    )?;
    let suit = result.suit;
    let rank = result.rank;
    let judgment_event = completed_judgment_event(judge_player, result, CardId::NONE);
    tx.push_event(DomainEvent::RuleNote {
        code: 232, // standalone skill judgment
        payload: vec![
            judge_player.raw(),
            crate::command::judge::suit_tag(suit),
            u32::from(rank),
        ],
    });
    if suit != expected_suit {
        return Ok(vec![judgment_event]);
    }
    recover_hp(tx, source, recover_player, amount)
        .map(|events| prepend_event(judgment_event, events))
}

pub(crate) fn judge_damage_other_on_suit(
    tx: &mut Transaction,
    player: PlayerId,
    expected_suit: engine_api::Suit,
    amount: u32,
    nature: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("bad judging player".into()));
    }
    if expected_suit == engine_api::Suit::None {
        return Err(EffectError::ContentFault(
            "judgment damage requires a physical suit".into(),
        ));
    }
    if nature > rule_api::damage_nature::THUNDER {
        return Err(EffectError::ContentFault(
            "unsupported judgment damage nature".into(),
        ));
    }
    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::Suit(expected_suit),
    )?;
    let suit = result.suit;
    let rank = result.rank;
    tx.push_event(DomainEvent::RuleNote {
        code: 232, // standalone skill judgment
        payload: vec![
            player.raw(),
            crate::command::judge::suit_tag(suit),
            u32::from(rank),
        ],
    });
    if suit != expected_suit || amount == 0 {
        return Ok(vec![completed_judgment_event(player, result, CardId::NONE)]);
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(vec![completed_judgment_event(player, result, CardId::NONE)]);
    }
    let emit = EffectRequest::EmitJudgmentResolved {
        player,
        result_card: result.card,
        suit,
        rank,
        cause_card: CardId::NONE,
    };
    let resume = vec![player.raw(), amount, u32::from(nature)];
    match crate::effect::run_effects(tx, vec![emit]) {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {
            let resolution = &mut tx.state_mut().resolution;
            resolution.resume_kind = ContinuationKind::JudgeDamageOtherOnSuit;
            resolution.resume_payload = resume;
            return Err(EffectError::ChoiceRequested);
        }
        Err(error) => return Err(error),
    }
    open_judgment_damage_other_choice(tx, player, amount, nature)
}

pub(crate) fn open_judgment_damage_other_choice(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
    nature: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .map(PlayerId::raw)
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
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::JudgeDamageOtherOnSuit,
        vec![player.raw(), amount, u32::from(nature)],
    )
}

pub(crate) fn judge_source_discard_two_hand_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    judge_target_discard_two_or_damage(tx, player, source, true)
}

pub(crate) fn judge_target_discard_two_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    hand_only: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("bad Ganglie judging player".into()));
    }
    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::NonSuit(engine_api::Suit::Heart),
    )?;
    let suit = result.suit;
    let rank = result.rank;
    tx.push_event(DomainEvent::RuleNote {
        code: 235,
        payload: vec![
            player.raw(),
            target.raw(),
            crate::command::judge::suit_tag(suit),
            u32::from(rank),
        ],
    });
    if suit == engine_api::Suit::Heart
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(vec![completed_judgment_event(player, result, CardId::NONE)]);
    }
    let emit = EffectRequest::EmitJudgmentResolved {
        player,
        result_card: result.card,
        suit,
        rank,
        cause_card: CardId::NONE,
    };
    let resume = vec![player.raw(), target.raw(), u32::from(hand_only)];
    match crate::effect::run_effects(tx, vec![emit]) {
        Ok(_) => {}
        Err(EffectError::ChoiceRequested) => {
            let resolution = &mut tx.state_mut().resolution;
            resolution.resume_kind = ContinuationKind::GanglieSourceDiscardOrDamage;
            resolution.resume_payload = resume;
            return Err(EffectError::ChoiceRequested);
        }
        Err(error) => return Err(error),
    }
    open_ganglie_target_discard_or_damage(tx, player, target, hand_only)
}

pub(crate) fn open_ganglie_target_discard_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    hand_only: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options = ganglie_discardable_cards(tx, target, hand_only);
    if options.len() < 2 {
        return super::super::run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: player,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        );
    }
    open_choice(
        tx,
        target,
        2,
        2,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::GanglieSourceDiscardOrDamage,
        vec![player.raw(), target.raw(), u32::from(hand_only)],
    )
}

pub(crate) fn recast_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive && !participant.dying)
    {
        return Err(EffectError::Illegal("bad recasting player".into()));
    }
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::Illegal("no equipment zone".into()))?;
    let judge = tx
        .state()
        .zones
        .find(ZoneKind::Judge, player)
        .ok_or_else(|| EffectError::Illegal("no judge zone".into()))?;
    let mut seen = std::collections::BTreeSet::new();
    let mut by_zone = std::collections::BTreeMap::<u32, Vec<CardId>>::new();
    for card in &cards {
        if !seen.insert(card.raw()) {
            return Err(EffectError::Illegal("duplicate recast card".into()));
        }
        let state = tx
            .state()
            .cards
            .get(*card)
            .ok_or_else(|| EffectError::Illegal("bad recast card".into()))?;
        if state.owner != player
            || (state.zone != hand && state.zone != equip && state.zone != judge)
        {
            return Err(EffectError::Illegal(
                "recast card is outside the player's hand, equipment, or judge zone".into(),
            ));
        }
        by_zone.entry(state.zone.raw()).or_default().push(*card);
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut events = Vec::new();
    for (from, cards) in by_zone {
        events.extend(move_cards_with_reason(
            tx,
            cards,
            discard,
            Some(ZoneId::new(from)),
            CardMoveReason::Recast,
        )?);
    }
    events.extend(draw_cards(tx, player, cards.len() as u32)?);
    Ok(events)
}

pub(crate) fn disable_equip_slots(
    tx: &mut Transaction,
    player: PlayerId,
    slots: Vec<u8>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let player_state = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("unknown player".into()))?;
    if !player_state.alive {
        return Ok(Vec::new());
    }

    let mut requested = Vec::new();
    for tag in slots {
        let slot = EquipSlot::from_tag(tag)
            .ok_or_else(|| EffectError::ContentFault(format!("invalid equipment slot {tag}")))?;
        if !requested.contains(&slot)
            && tx
                .state()
                .players
                .get(player)
                .is_some_and(|state| state.disabled_equip_slots[slot.index()] == 0)
        {
            requested.push(slot);
        }
    }
    if requested.is_empty() {
        return Ok(Vec::new());
    }

    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::Illegal("no equip zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let equipped: Vec<CardId> = tx
        .state()
        .zones
        .get(equip)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .and_then(|card| card.kind.equip_slot())
                .is_some_and(|slot| requested.contains(&slot))
        })
        .collect();

    for slot in &requested {
        tx.state_mut()
            .players
            .get_mut(player)
            .expect("validated player")
            .disabled_equip_slots[slot.index()] = 1;
    }

    let mut events = if equipped.is_empty() {
        Vec::new()
    } else {
        move_cards(tx, equipped, discard, Some(equip))?
    };
    events.extend(
        requested
            .into_iter()
            .map(|slot| DomainEvent::EquipSlotDisabled {
                player,
                slot: slot as u8,
            }),
    );
    Ok(events)
}

pub(crate) fn enable_equip_slot(
    tx: &mut Transaction,
    player: PlayerId,
    slot: u8,
    source: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let slot = EquipSlot::from_tag(slot)
        .ok_or_else(|| EffectError::ContentFault(format!("invalid equipment slot {slot}")))?;
    let Some(player_state) = tx.state().players.get(player) else {
        return Err(EffectError::Illegal("unknown player".into()));
    };
    if !player_state.alive || player_state.disabled_equip_slots[slot.index()] == 0 {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .players
        .get_mut(player)
        .expect("validated player")
        .disabled_equip_slots[slot.index()] -= 1;
    Ok(vec![DomainEvent::EquipSlotEnabled {
        source,
        player,
        slot: slot as u8,
    }])
}

pub(crate) fn judge_cancel_current_sha_target_on_red(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() || tx.state().players.get(target).is_none() {
        return Err(EffectError::Illegal("bad judgment participant".into()));
    }
    let mut probe = tx.state().resolution.clone();
    if !probe.exclude_saved_sha_target(card, target) {
        return Err(EffectError::Illegal(
            "no matching currently resolving Sha target".into(),
        ));
    }

    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::Red,
    )?;
    let suit = result.suit;
    let rank = result.rank;
    tx.push_event(DomainEvent::RuleNote {
        code: 233,
        payload: vec![
            player.raw(),
            card.raw(),
            target.raw(),
            crate::command::judge::suit_tag(suit),
            rank as u32,
        ],
    });
    if matches!(suit, engine_api::Suit::Heart | engine_api::Suit::Diamond)
        && !tx
            .state_mut()
            .resolution
            .exclude_saved_sha_target(card, target)
    {
        return Err(EffectError::EngineFault(
            "saved Sha target changed during judgment".into(),
        ));
    }
    Ok(vec![completed_judgment_event(player, result, CardId::NONE)])
}

pub(crate) fn judge_make_current_sha_target_unrespondable_on_red(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
    retieji: bool,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
    judgment_suit: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if retieji {
        crate::command::play::validate_saved_sha_target(tx, player, card, target)?;
        if !originating_skill_is_enabled(tx, player, skill) {
            return Ok(Vec::new());
        }
        if !ready_to_choose {
            let result = crate::command::judge::flip_judgment(
                tx,
                player,
                crate::command::judge::JudgmentFavor::Neutral,
            )?;
            let suit = crate::command::judge::suit_tag(result.suit) as u8;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::JudgeMakeCurrentShaTargetUnrespondableOnRed {
                    player,
                    card,
                    target,
                    retieji: true,
                    skill,
                    ready_to_choose: true,
                    judgment_suit: suit,
                },
            );
            return Ok(vec![completed_judgment_event(player, result, CardId::NONE)]);
        }

        let suit = crate::command::judge::suit_from_tag(u32::from(judgment_suit));
        if suit == engine_api::Suit::None {
            return Err(EffectError::ContentFault(
                "Retieji judgment produced a suitless card".into(),
            ));
        }
        if !tx
            .state()
            .turn
            .non_locked_skill_suppressed_players
            .contains(&target)
        {
            tx.state_mut()
                .turn
                .non_locked_skill_suppressed_players
                .push(target);
            tx.state_mut()
                .turn
                .non_locked_skill_suppressed_players
                .sort_by_key(|player| player.raw());
            tx.state_mut().rebuild_derived_indexes();
        }
        let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), target)
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_suit(tx.state(), target, card) == suit
                })
            })
            .map(CardId::raw)
            .collect();
        if options.is_empty() {
            let mut events = make_current_sha_target_unrespondable(tx, player, card, target)?;
            events.extend(increase_current_sha_target_damage(
                tx, player, card, target, 1,
            )?);
            return Ok(events);
        }
        return open_choice(
            tx,
            target,
            0,
            1,
            true,
            options,
            ChoiceKind::SelectCards,
            ContinuationKind::ResolveRetieji,
            vec![
                player.raw(),
                card.raw(),
                target.raw(),
                u32::from(judgment_suit),
            ],
        );
    }

    // ponytail: Standard identity has no Shouyue aura; add its non-spade branch when
    // lord/group aura state becomes authoritative during the Guozhan migration.
    crate::command::play::validate_saved_sha_target(tx, player, card, target)?;
    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::Red,
    )?;
    let judgment = completed_judgment_event(player, result, CardId::NONE);
    if matches!(
        result.suit,
        engine_api::Suit::Heart | engine_api::Suit::Diamond
    ) {
        make_current_sha_target_unrespondable(tx, player, card, target)
            .map(|events| prepend_event(judgment, events))
    } else {
        Ok(vec![judgment])
    }
}

pub(crate) fn choose_discard_basic_then_target_discard_equip_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || player == target
    {
        return Err(EffectError::Illegal("invalid Xiaoguo context".into()));
    }
    let options = crate::query::standard_xiaoguo_basic_hand_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage,
        vec![0, player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn target_discard_then_duel_if_non_sha_and_hp_gte_owner(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid target-discard Duel context".into(),
        ));
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
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
        ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner,
        vec![player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_target_discard_duel(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    discarded_was_sha: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if discarded_was_sha
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || tx.state().players.get(target).unwrap().hp < tx.state().players.get(player).unwrap().hp
        || crate::command::play::validate_source_free_card_on_target(
            tx,
            player,
            CardKind::TrickJuedou,
            target,
        )
        .is_err()
    {
        return Ok(Vec::new());
    }
    use_virtual_card_on(tx, player, CardKind::TrickJuedou.tag(), target)
}

pub(crate) fn choose_target_lose_hp_discard_hand_sha_if_basic(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    stage: u8,
    discarded_basic: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
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
                .filter(|&target| {
                    !crate::query::discardable_target_cards(
                        tx.state(),
                        target,
                        target,
                        rule_api::TargetCardScope::Hand,
                    )
                    .is_empty()
                })
                .map(PlayerId::raw)
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
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseTargetLoseHpDiscardHandShaIfBasic,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                target,
                target,
                rule_api::TargetCardScope::Hand,
            );
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
                || options.is_empty()
            {
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
                ContinuationKind::ChooseTargetLoseHpDiscardHandShaIfBasic,
                vec![1, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            if discarded_basic
                && tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                try_use_virtual_sha_on_without_distance(tx, player, target, false, 0)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Err(EffectError::ContentFault(
            "bad target-discard conditional Sha stage".into(),
        )),
    }
}

pub(crate) fn choose_current_sha_target_discard_redirect_to_owner(
    tx: &mut Transaction,
    owner: PlayerId,
    source: PlayerId,
    card: CardId,
    target: PlayerId,
    skill: engine_api::SkillId,
    ready_to_redirect: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participants_alive = [owner, source, target].into_iter().all(|player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    });
    if !participants_alive || source == owner || target == owner {
        return Ok(Vec::new());
    }
    if !originating_skill_is_enabled(tx, owner, skill)
        || crate::command::play::validate_saved_sha_target(tx, source, card, target).is_err()
        || crate::command::play::validate_saved_sha_target(tx, source, card, owner).is_ok()
        || !crate::query::player_in_attack_range(tx.state(), target, owner)
        || crate::command::play::validate_sha_target_with_options(tx, source, card, owner, true)
            .is_err()
    {
        return Ok(Vec::new());
    }
    if ready_to_redirect {
        let targets =
            crate::command::play::redirect_saved_sha_target(tx, source, card, target, owner)?;
        return Ok(vec![DomainEvent::CardTargetAdded {
            player: source,
            card,
            target: owner,
            targets,
        }]);
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        target,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::TongjiDiscardRedirectSha,
        vec![
            owner.raw(),
            source.raw(),
            card.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn choose_discard_redirect_current_sha(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|source| source.alive)
        || source == player
    {
        return Ok(Vec::new());
    }
    let candidates = crate::command::play::saved_sha_redirect_targets_in_attack_range(
        tx, source, card, player, player,
    )
    .unwrap_or_default();
    if target.is_some() {
        if !candidates.contains(&target) {
            return Ok(Vec::new());
        }
        let targets =
            crate::command::play::redirect_saved_sha_target(tx, source, card, player, target)?;
        return Ok(vec![DomainEvent::CardTargetAdded {
            player: source,
            card,
            target,
            targets,
        }]);
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() || candidates.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::LiuliDiscardRedirectSha,
        vec![0, player.raw(), source.raw(), card.raw(), skill.raw()],
    )
}

pub(crate) fn choose_target_gain_bottom_card(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid_owner = originating_skill_is_enabled(tx, player, skill)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Play
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive);
    if !valid_owner {
        return Err(EffectError::Illegal("invalid Jijie context".into()));
    }

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Jijie draw pile is missing".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Jijie processing zone is missing".into()))?;

    if card.is_none() {
        if target.is_some() {
            return Err(EffectError::EngineFault(
                "Jijie target exists before card staging".into(),
            ));
        }
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
            .and_then(|zone| zone.cards.first())
            .copied()
        else {
            return Ok(Vec::new());
        };
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ChooseTargetGainBottomCard {
                player,
                card,
                target: PlayerId::NONE,
                skill,
            },
        );
        let mut events = move_cards(tx, vec![card], processing, Some(draw))?;
        events.push(DomainEvent::CardsRevealed {
            cards: vec![card],
            to: Vec::new(),
        });
        return Ok(events);
    }

    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.zone == processing && state.owner.is_none())
    {
        return Err(EffectError::Illegal(
            "Jijie bottom card left processing".into(),
        ));
    }
    if target.is_none() {
        let options = tx.state().players.alive_by_seat();
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|target| target.raw()).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::JijieChooseBottomCardRecipient,
            vec![player.raw(), card.raw(), skill.raw()],
        );
    }
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Jijie recipient is invalid or stale".into(),
        ));
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("Jijie recipient hand is missing".into()))?;
    give_cards_as(tx, player, vec![card], to, processing)
}

pub(crate) fn black_gain_repeat_request(
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
    protect_hand_limit: bool,
) -> EffectRequest {
    if protect_hand_limit {
        EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
            player,
            card,
            skill,
            ready_to_choose,
        }
    } else {
        EffectRequest::JudgeBlackGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
        }
    }
}

pub(crate) fn judge_black_gain_repeat(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
    protect_hand_limit: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid = originating_skill_is_enabled(tx, player, skill)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Prepare
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive);
    if !valid {
        return if card.is_none() {
            Err(EffectError::Illegal("invalid Luoshen context".into()))
        } else {
            Ok(Vec::new())
        };
    }
    if ready_to_choose {
        return open_choice(
            tx,
            player,
            0,
            1,
            true,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::LuoshenRepeatBlackJudgment,
            if protect_hand_limit {
                vec![player.raw(), skill.raw(), 1]
            } else {
                vec![player.raw(), skill.raw()]
            },
        );
    }
    if card.is_none() {
        let result = crate::command::judge::flip_judgment(
            tx,
            player,
            crate::command::judge::JudgmentFavor::Neutral,
        )?;
        let result_card = result.card;
        let is_black = matches!(
            result.suit,
            engine_api::Suit::Spade | engine_api::Suit::Club
        );
        let judgment = completed_judgment_event(player, result, CardId::NONE);
        if is_black {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(black_gain_repeat_request(
                    player,
                    result_card,
                    skill,
                    false,
                    protect_hand_limit,
                ));
        }
        return Ok(vec![judgment]);
    }

    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(black_gain_repeat_request(
            player,
            card,
            skill,
            true,
            protect_hand_limit,
        ));
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Luoshen discard pile is missing".into()))?;
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.zone == discard)
    {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Luoshen owner hand is missing".into()))?;
    let events = move_cards(tx, vec![card], hand, Some(discard))?;
    if protect_hand_limit {
        tx.state_mut()
            .turn
            .hand_limit_exempt_cards
            .insert((player, card));
    }
    Ok(events)
}

pub(crate) fn judge_same_color_gain_repeat(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
    color: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid = originating_skill_is_enabled(tx, player, skill)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Prepare
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive);
    if !valid {
        return if card.is_none() && color == 0 {
            Err(EffectError::Illegal("invalid Yjluoshen context".into()))
        } else {
            Ok(Vec::new())
        };
    }
    if ready_to_choose {
        return open_choice(
            tx,
            player,
            0,
            1,
            true,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::JudgeSameColorGainRepeat,
            vec![player.raw(), skill.raw(), u32::from(color)],
        );
    }
    if card.is_none() {
        let result = crate::command::judge::flip_judgment(
            tx,
            player,
            crate::command::judge::JudgmentFavor::Neutral,
        )?;
        let result_card = result.card;
        let result_color = crate::query::card_color_mask_for_suit(result.suit);
        let first_color = if color == 0 { result_color } else { color };
        let judgment = completed_judgment_event(player, result, CardId::NONE);
        if result_color != 0 && result_color == first_color {
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::JudgeSameColorGainRepeat {
                    player,
                    card: result_card,
                    skill,
                    ready_to_choose: false,
                    color: first_color,
                },
            );
        }
        return Ok(vec![judgment]);
    }

    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::JudgeSameColorGainRepeat {
            player,
            card,
            skill,
            ready_to_choose: true,
            color,
        });
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Yjluoshen discard pile is missing".into()))?;
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.zone == discard)
    {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Yjluoshen owner hand is missing".into()))?;
    move_cards(tx, vec![card], hand, Some(discard))
}

pub(crate) fn use_virtual_juedou_second_on_first(
    tx: &mut Transaction,
    targets: Vec<PlayerId>,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let [target, user] = targets.as_slice() else {
        return Err(EffectError::ContentFault(
            "Lijian requires exactly two selected targets".into(),
        ));
    };
    let target = *target;
    let user = *user;
    if target == user {
        return Err(EffectError::ContentFault(
            "Lijian targets must be distinct".into(),
        ));
    }
    if card.is_none() {
        crate::command::play::validate_source_free_card_on_target(
            tx,
            user,
            CardKind::TrickJuedou,
            target,
        )?;
        let card = materialize_virtual_card(tx, user, CardKind::TrickJuedou)?;
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::UseVirtualJuedouSecondOnFirst { targets, card });
        return super::super::run_effects(
            tx,
            vec![EffectRequest::UseCard {
                player: user,
                card,
                targets: vec![target],
            }],
        );
    }
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.def.is_none() && state.kind == CardKind::TrickJuedou)
    {
        return Err(EffectError::Illegal(
            "Lijian virtual Duel context changed".into(),
        ));
    }
    crate::command::play::resolve_independent_trick_without_wuxie(
        tx,
        user,
        card,
        &[target],
        crate::command::play::trick_effect::JUEDOU,
    )?;
    Ok(Vec::new())
}
