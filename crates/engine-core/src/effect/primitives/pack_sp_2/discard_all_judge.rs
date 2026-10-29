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
