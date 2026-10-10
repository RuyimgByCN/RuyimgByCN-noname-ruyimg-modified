//! Effect primitive implementations — sp 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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
    super::run_effects(tx, effects)
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
        return super::run_effects(
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
        return super::run_effects(
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

/// 欢乐卡牌【手里剑】使用结算（ABI 642, ADR 0607）：目标弃一张装备牌或受 1 点伤害。
///
/// 复用 `open_xiaoguo_target_equipment_or_damage` 的「弃装备或伤害」choice 结算。
pub(crate) fn resolve_shoulijian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target
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
    {
        return Ok(Vec::new());
    }
    // If the target has no equipment to discard, they take 1 damage directly;
    // otherwise they choose (handled by the Xiaoguo continuation).
    open_xiaoguo_target_equipment_or_damage(tx, player, target, skill)?;
    Ok(Vec::new())
}

/// 取消使用牌（ABI 643, ADR 0608）：gwent 杜达元帅1 特殊卡——当你成为其他角色使用牌
/// 的目标时，取消该牌对你生效，并获取对你使用的实体牌。
///
/// 触发上下文为 `trigger card_targeted`（牌使用结算中、逐目标触发）。此刻杀牌走
/// `sha_resolution_records`/ResumePlayedSha，锦囊走 WuxieWindow resume_payload，两者
/// 均已挂起，可经 `exclude_saved_card_target` 把 `player` 从当前牌结算目标中排除；
/// 不可排除的牌型（如无中生有类全目标锦囊）整效果视为未发动（graceful no-op）。
pub(crate) fn cancel_incoming_card_use(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == source
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    // The holder must still be a declared target of the suspended card use.
    let (_, removable) = crate::command::play::current_card_target_adjustments(tx, source, card)?;
    if !removable.contains(&player) {
        // 批 155：多目标锦囊（南蛮/万箭 AOE 队列）在触发效果执行时尚未挂起
        // （resume_kind=None）——把排除意图暂存，`advance_aoe_queue` 逐目标时消费。
        if tx.state().resolution.resume_kind == crate::state::ContinuationKind::None {
            tx.state_mut()
                .deferred_card_exclusions
                .push((card.raw(), player.raw()));
        }
        return Ok(Vec::new());
    }
    // 取消：把 holder 从当前牌结算目标中排除（杀走 sha 记录、锦囊走 Wuxie payload；
    // 批 155 起南蛮/万箭 AOE 队列经 deferred_card_exclusions 延迟排除——触发效果
    // 执行时 wuxie 载荷未挂起（resume_kind=None）或挂起但 NANMAN/WANJIAN 尚无
    // removable 计算（批 155 已补）皆可达此路径）。
    if crate::command::play::exclude_saved_card_target(tx, card, player).is_err() {
        // 该牌型无法被排除（全目标锦囊等）：整效果未发动。
        return Ok(Vec::new());
    }
    // 获得对你使用的实体牌（杜达元帅语义；啖酪共用本原语，多获实体牌为已注记残差）。
    let used = tx.state().resolution.card_source_cards(source, card);
    gain_event_cards(tx, player, &used)
}

/// Custom-choice marker for the 择一加技能 primitive.
pub(crate) const CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS_CUSTOM_CHOICE: u32 = 0x4353_4453;

/// Custom-choice marker for the 水淹七军 (guozhan) target choice.
pub(crate) const SHUIYANQIJUNX_CUSTOM_CHOICE: u32 = 0x5359_514A;

/// Custom-choice marker for the 克复中原 (guozhan) target choice.
pub(crate) const KEFUZHONGYUAN_CUSTOM_CHOICE: u32 = 0x4B46_5A59;

/// Custom-choice marker for the 固国安邦 (guozhan) staged settlement.
pub(crate) const GUGUOANBANG_CUSTOM_CHOICE: u32 = 0x4747_4142;

/// Custom-choice marker for the 联军盛宴 (guozhan) draw/recover split.
pub(crate) const LIANJUNSHENGYAN_CUSTOM_CHOICE: u32 = 0x4C4A_5359;

/// Custom-choice marker for the 文和乱武 (guozhan) staged settlement.
pub(crate) const WENHELUANWU_CUSTOM_CHOICE: u32 = 0x5745_4C57;

/// Custom-choice marker for the 号令天下 (guozhan) per-player option chain.
pub(crate) const HAOLINGTIANXIA_CUSTOM_CHOICE: u32 = 0x484C_5458;

/// Custom-choice marker for the 敕令 (guozhan) per-target option choice.
pub(crate) const CHILING_CUSTOM_CHOICE: u32 = 0x4348_4C4E;

/// 择一加技能（ABI 644, ADR 0612）：gwent 呼朋（hupeng）——玩家在两个技能定义间
/// 二选一，将所选技能置入目标（上游为 chooseVCardButton 择一「杜达元帅1/2」实体卡
/// 置入目标手牌；本引擎以技能近似杜达元帅卡，故置入所选技能）。
///
/// 复用 `ContinuationKind::Custom` 魔数分发（payload[0] 为本魔数），不新增
/// ContinuationKind 变体，保持回放序数稳定。
pub(crate) fn choose_other_gain_one_of_two_skills(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    first_def: engine_api::SkillDefId,
    second_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target
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
        || first_def.is_none()
        || second_def.is_none()
        || first_def == second_def
    {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS_CUSTOM_CHOICE,
            0,
            player.raw(),
            target.raw(),
            first_def.raw(),
            second_def.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn open_xiaoguo_target_equipment_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
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
        return Ok(());
    }
    let options = crate::query::standard_xiaoguo_equipment_cards(tx.state(), target);
    if options.is_empty() {
        return super::run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: player,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .map(|_| ());
    }
    open_choice(
        tx,
        target,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage,
        vec![2, player.raw(), target.raw(), skill.raw()],
    )
    .map(|_| ())
}

pub(crate) fn judge_prevent_damage_lose_max_hp_on_nonheart(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(player).is_none() || tx.state().players.get(target).is_none() {
        return Err(EffectError::Illegal("bad judgment participant".into()));
    }
    if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
        return Err(EffectError::ContentFault(
            "judgment damage prevention used outside its damage_before event".into(),
        ));
    }
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

    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::NonSuit(engine_api::Suit::Heart),
    )?;
    let suit = result.suit;
    let rank = result.rank;
    let judgment_event = completed_judgment_event(player, result, CardId::NONE);
    tx.push_event(DomainEvent::RuleNote {
        code: 234,
        payload: vec![
            player.raw(),
            target.raw(),
            damage_id,
            crate::command::judge::suit_tag(suit),
            u32::from(rank),
        ],
    });
    if suit == engine_api::Suit::Heart {
        return Ok(vec![judgment_event]);
    }
    if !tx
        .state()
        .resolution
        .prevented_damage_ids
        .contains(&damage_id)
    {
        tx.state_mut()
            .resolution
            .prevented_damage_ids
            .push(damage_id);
    }
    change_max_hp(tx, target, -1).map(|events| prepend_event(judgment_event, events))
}

pub(crate) fn make_current_sha_target_unrespondable(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(source)
        .is_some_and(|player| player.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|player| player.alive)
        || !tx.state().cards.get(card).is_some_and(|value| {
            crate::query::effective_card_kind(tx.state(), source, value).is_sha()
        })
    {
        return Err(EffectError::Illegal(
            "invalid current Sha response participant".into(),
        ));
    }
    if !tx
        .state_mut()
        .resolution
        .make_saved_sha_target_unrespondable(source, card, target)
    {
        return Err(EffectError::Illegal(
            "no matching currently resolving Sha target".into(),
        ));
    }
    Ok(vec![DomainEvent::RuleNote {
        code: 234,
        payload: vec![source.raw(), card.raw(), target.raw()],
    }])
}

pub(crate) fn increase_current_sha_target_damage(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    target: PlayerId,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0 {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(source)
        .is_some_and(|player| player.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|player| player.alive)
        || !tx.state().cards.get(card).is_some_and(|value| {
            crate::query::effective_card_kind(tx.state(), source, value).is_sha()
        })
    {
        return Err(EffectError::Illegal(
            "invalid current Sha damage participant".into(),
        ));
    }
    crate::command::play::increase_saved_sha_target_damage(tx, source, card, target, amount)?;
    Ok(vec![DomainEvent::RuleNote {
        code: 235,
        payload: vec![source.raw(), card.raw(), target.raw(), amount],
    }])
}

pub(crate) fn choose_discard_target_card_resolve_current_sha(
    tx: &mut Transaction,
    source: PlayerId,
    card: CardId,
    target: PlayerId,
    nonbasic_bonus_skill: Option<engine_api::SkillDefId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    crate::command::play::validate_saved_sha_target(tx, source, card, target)?;
    open_discard_target_card_choice(
        tx,
        source,
        target,
        rule_api::TargetCardScope::HandEquip,
        ContinuationKind::ChooseDiscardTargetCardResolveCurrentSha,
        false,
        nonbasic_bonus_skill,
    )
}

pub(crate) fn resolve_dcqiaomeng(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    target: PlayerId,
    selected: CardId,
    helper_skill_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let context_valid = originating_skill_is_enabled(tx, player, skill)
        && !helper_skill_def.is_none()
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && crate::command::play::validate_saved_targeted_card_use(tx, player, card).is_ok()
        && tx
            .state()
            .cards
            .get(card)
            .map(|card| crate::query::effective_card_suit(tx.state(), player, card))
            .is_some_and(|suit| matches!(suit, Suit::Spade | Suit::Club));
    if !context_valid {
        return if stage == 0 {
            Ok(Vec::new())
        } else {
            Err(EffectError::Illegal(
                "Dcqiaomeng current-card context changed".into(),
            ))
        };
    }
    let eligible_targets = || {
        let mut seen = std::collections::BTreeSet::new();
        targets
            .iter()
            .copied()
            .filter(|candidate| seen.insert(*candidate))
            .filter(|candidate| *candidate != player)
            .filter(|candidate| {
                tx.state()
                    .players
                    .get(*candidate)
                    .is_some_and(|participant| participant.alive)
            })
            .filter(|candidate| {
                !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    *candidate,
                    TargetCardScope::HandEquip,
                )
                .is_empty()
            })
            .collect::<Vec<_>>()
    };
    let mut payload = vec![
        stage as u32,
        player.raw(),
        card.raw(),
        target.raw(),
        helper_skill_def.raw(),
        skill.raw(),
        targets.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(targets.iter().copied().map(PlayerId::raw));
    match stage {
        0 => {
            let options = eligible_targets();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveDcqiaomeng,
                payload,
            )
        }
        1 => {
            if !eligible_targets().contains(&target) {
                return Err(EffectError::Illegal(
                    "Dcqiaomeng selected target is no longer legal".into(),
                ));
            }
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveDcqiaomeng,
                payload,
            )
        }
        2 => {
            if !targets.contains(&target)
                || !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    target,
                    TargetCardScope::HandEquip,
                )
                .contains(&selected)
            {
                return Err(EffectError::Illegal(
                    "Dcqiaomeng target card is no longer legal".into(),
                ));
            }
            let selected_kind = tx
                .state()
                .cards
                .get(selected)
                .map(|card| card.kind)
                .ok_or_else(|| EffectError::Illegal("Dcqiaomeng card is missing".into()))?;
            let mut effects = vec![EffectRequest::DiscardTargetCards {
                actor: player,
                target,
                cards: vec![selected],
                scope: TargetCardScope::HandEquip,
            }];
            if selected_kind.is_equip() {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("missing discard pile".into()))?;
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::EngineFault("missing Dcqiaomeng hand".into()))?;
                effects.push(EffectRequest::MoveCards {
                    cards: vec![selected],
                    to: hand,
                    from: Some(discard),
                });
            } else if selected_kind.is_trick() {
                effects.push(EffectRequest::AddCardUsePolicySkill {
                    player,
                    card,
                    policy_mask: 1 << 1,
                    skill_def: helper_skill_def,
                });
            }
            run_effects(tx, effects).map(|_| Vec::new())
        }
        _ => Err(EffectError::EngineFault("invalid Dcqiaomeng stage".into())),
    }
}

pub(crate) fn take_from_hand(
    tx: &mut Transaction,
    from: PlayerId,
    to: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if from == to {
        return Ok(vec![]);
    }
    let hand = tx.state().hand_cards(from);
    let Some(card) = hand.first().copied() else {
        return Ok(vec![]);
    };
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, to)
        .ok_or_else(|| EffectError::Illegal("no dest hand".into()))?;
    move_cards(tx, vec![card], to_hand, None)
}

pub(crate) fn draw_cards(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {

    if tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal(format!("no player {}", player.raw())));
    }
    let draw_zone = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand_zone = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;

    let mut drawn = Vec::new();
    for _ in 0..count {
        // Reshuffle if empty (noname: 洗牌)
        let empty = tx
            .state()
            .zones
            .get(draw_zone)
            .map(|z| z.cards.is_empty())
            .unwrap_or(true);
        if empty {
            reshuffle_discard_into_draw(tx)?;
        }
        let card = {
            let z = tx
                .state()
                .zones
                .get(draw_zone)
                .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
            match z.cards.last().copied() {
                Some(c) => c,
                None => break,
            }
        };
        move_one_card(tx, card, draw_zone, hand_zone, player, false)?;
        drawn.push(card);
    }
    if drawn.is_empty() && count > 0 {
        return Ok(vec![]);
    }
    if !drawn.is_empty() && tx.state().turn.phase == Phase::Play {
        tx.state_mut().turn.record_play_phase_draw(player);
    }
    Ok(vec![DomainEvent::CardsDrawn {
        player,
        cards: drawn,
    }])
}

pub(crate) fn draw_remembered_hand_cards(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let events = draw_cards(tx, player, count)?;
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let cards: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsDrawn {
                player: drawer,
                cards,
            } if *drawer == player => Some(cards.as_slice()),
            _ => None,
        })
        .flatten()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
        })
        .collect();
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .expect("validated originating skill")
        .remember_hand_cards(&cards)
    {
        return Err(EffectError::ContentFault(
            "remembered hand cards conflict with existing skill state".into(),
        ));
    }
    Ok(events)
}

pub(crate) fn discard_remembered_hand_cards(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let remembered = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .expect("validated originating skill")
        .take_remembered_hand_cards()
        .ok_or_else(|| {
            EffectError::ContentFault(
                "remembered hand cards conflict with existing skill state".into(),
            )
        })?;
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let discardable: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .collect();
    let cards: Vec<_> = remembered
        .into_iter()
        .filter(|card| {
            discardable.contains(card)
                && tx
                    .state()
                    .cards
                    .get(*card)
                    .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
        })
        .collect();
    if cards.is_empty() {
        Ok(Vec::new())
    } else {
        discard_cards(tx, player, cards)
    }
}

pub(crate) fn draw_each_by_seat(
    tx: &mut Transaction,
    first: PlayerId,
    second: PlayerId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if first == second {
        return Err(EffectError::Illegal(
            "draw_each_by_seat requires distinct players".into(),
        ));
    }
    let seats = tx.state().players.alive_by_seat();
    if !seats.contains(&first) || !seats.contains(&second) {
        return Ok(Vec::new());
    }
    let mut players = [first, second];
    players.sort_by_key(|player| {
        seats
            .iter()
            .position(|seated| seated == player)
            .unwrap_or(usize::MAX)
    });
    let mut events = draw_cards(tx, players[0], count)?;
    events.extend(draw_cards(tx, players[1], count)?);
    Ok(events)
}

pub(crate) fn draw_selected_targets(
    tx: &mut Transaction,
    targets: &[PlayerId],
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if targets.is_empty() {
        return Err(EffectError::ContentFault(
            "selected-target draw requires at least one target".into(),
        ));
    }
    let order = tx.state().players.alive_by_seat();
    let mut sorted = Vec::with_capacity(targets.len());
    for &target in targets {
        if sorted.contains(&target) {
            return Err(EffectError::ContentFault(
                "selected-target draw received duplicate targets".into(),
            ));
        }
        if !order.contains(&target) {
            return Err(EffectError::Illegal(
                "selected-target draw requires living targets".into(),
            ));
        }
        sorted.push(target);
    }
    sorted.sort_by_key(|target| {
        order
            .iter()
            .position(|candidate| candidate == target)
            .expect("validated living target")
    });
    let mut events = Vec::new();
    for target in sorted {
        events.extend(draw_cards(tx, target, count)?);
    }
    Ok(events)
}

pub(crate) fn draw_linked_players(tx: &mut Transaction, count: u32) -> Result<Vec<DomainEvent>, EffectError> {
    let players: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&player| {
            tx.state().players.get(player).is_some_and(|state| {
                state
                    .marks
                    .iter()
                    .any(|(mark, value)| *mark == crate::content::MARK_CHAIN && *value > 0)
            })
        })
        .collect();
    let mut events = Vec::new();
    for player in players {
        events.extend(draw_cards(tx, player, count)?);
    }
    Ok(events)
}

pub(crate) fn damage_then_draw_selected_targets(
    tx: &mut Transaction,
    source: PlayerId,
    targets: &[PlayerId],
    damage: u32,
    draw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(source)
        .is_some_and(|player| player.alive)
        || targets.is_empty()
    {
        return Err(EffectError::ContentFault(
            "selected-target damage requires a living source and targets".into(),
        ));
    }
    let order = tx.state().players.alive_by_seat();
    let mut sorted = targets.to_vec();
    sorted.sort_by_key(|target| {
        order
            .iter()
            .position(|candidate| candidate == target)
            .unwrap_or(usize::MAX)
    });
    if sorted.iter().any(|target| !order.contains(target))
        || sorted.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err(EffectError::ContentFault(
            "selected-target damage requires distinct living targets".into(),
        ));
    }
    let effects = sorted
        .into_iter()
        .flat_map(|target| {
            [
                EffectRequest::DealDamage {
                    source,
                    target,
                    amount: damage,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                },
                EffectRequest::DrawCards {
                    player: target,
                    count: draw,
                },
            ]
        })
        .collect();
    super::run_effects(tx, effects).map(|_| Vec::new())
}

pub(crate) fn resolve_youdi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = |tx: &Transaction, player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    };
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !alive(tx, player)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::End
                || crate::query::discardable_hand_equip_cards(tx.state(), player).is_empty()
            {
                return Ok(Vec::new());
            }
            let options = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveYoudi,
                vec![0, player.raw(), 0, skill.raw()],
            )
        }
        1 => {
            if player == target || !alive(tx, player) || !alive(tx, target) {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_target_cards(
                tx.state(),
                target,
                player,
                TargetCardScope::HandEquip,
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
                ContinuationKind::ResolveYoudi,
                vec![1, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            let legal = crate::query::discardable_target_cards(
                tx.state(),
                target,
                player,
                TargetCardScope::HandEquip,
            );
            if !legal.contains(&card) {
                return Err(EffectError::Illegal(
                    "Youdi owner card became illegal".into(),
                ));
            }
            let discarded_was_sha = tx.state().cards.get(card).is_some_and(|card| {
                crate::query::effective_card_kind(tx.state(), player, card).is_sha()
            });
            if !discarded_was_sha {
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(EffectRequest::ResolveYoudi {
                        player,
                        target,
                        card: CardId::NONE,
                        stage: 3,
                        skill,
                    });
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardTargetCards {
                    actor: target,
                    target: player,
                    cards: vec![card],
                    scope: TargetCardScope::HandEquip,
                }],
            )
            .map(|_| Vec::new())
        }
        3 => {
            if !alive(tx, player) || !alive(tx, target) {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveYoudi,
                vec![2, player.raw(), target.raw(), skill.raw()],
            )
        }
        4 => {
            if !alive(tx, player) || !alive(tx, target) {
                return Ok(Vec::new());
            }
            let legal = crate::query::discardable_hand_equip_cards(tx.state(), target);
            if !legal.contains(&card) {
                return Err(EffectError::Illegal(
                    "Youdi target card became illegal".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .ok_or_else(|| EffectError::Illegal("bad Youdi target card".into()))?
                .zone;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Youdi owner hand missing".into()))?;
            move_cards(tx, vec![card], to, Some(from))
        }
        _ => Err(EffectError::ContentFault("unknown Youdi stage".into())),
    }
}

pub(crate) fn move_target_equipment_to_player_effects(
    state: &crate::state::GameState,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
) -> Result<Vec<EffectRequest>, EffectError> {
    if !crate::query::movable_target_equipment_cards(state, player, target).contains(&card) {
        return Err(EffectError::Illegal(
            "selected equipment card is no longer movable".into(),
        ));
    }
    let card_state = state
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("missing selected equipment card".into()))?;
    let slot = card_state
        .kind
        .equip_slot()
        .ok_or_else(|| EffectError::Illegal("selected card is not equipment".into()))?;
    let from = card_state.zone;
    let to = state
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::Illegal("missing destination equipment zone".into()))?;
    let replaced = state
        .zones
        .get(to)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|existing| {
            state
                .cards
                .get(*existing)
                .and_then(|card| card.kind.equip_slot())
                == Some(slot)
        })
        .collect::<Vec<_>>();
    let mut effects = Vec::with_capacity(2);
    if !replaced.is_empty() {
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("missing discard pile".into()))?;
        effects.push(EffectRequest::MoveCards {
            cards: replaced,
            to: discard,
            from: Some(to),
        });
    }
    effects.push(EffectRequest::MoveCards {
        cards: vec![card],
        to,
        from: Some(from),
    });
    Ok(effects)
}

pub(crate) fn resolve_kuangfu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid_players = |tx: &Transaction| {
        player != target
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
    };
    if !valid_players(tx) || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let discardable = crate::query::discardable_equipment_cards(tx.state(), player, target);
    match stage {
        0 => {
            if discardable.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                discardable.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveKuangfu,
                vec![0, player.raw(), target.raw(), skill.raw()],
            )
        }
        1 => {
            if !discardable.contains(&card) {
                return Err(EffectError::Illegal(
                    "Kuangfu equipment became illegal".into(),
                ));
            }
            if crate::query::movable_target_equipment_cards(tx.state(), player, target)
                .contains(&card)
            {
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Confirm,
                    ContinuationKind::ResolveKuangfu,
                    vec![1, player.raw(), target.raw(), card.raw(), skill.raw()],
                )
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::DiscardTargetCards {
                        actor: player,
                        target,
                        cards: vec![card],
                        scope: TargetCardScope::Equip,
                    }],
                )
                .map(|_| Vec::new())
            }
        }
        2 => {
            if !discardable.contains(&card) {
                return Err(EffectError::Illegal(
                    "Kuangfu equipment became illegal".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardTargetCards {
                    actor: player,
                    target,
                    cards: vec![card],
                    scope: TargetCardScope::Equip,
                }],
            )
            .map(|_| Vec::new())
        }
        3 => {
            let effects =
                move_target_equipment_to_player_effects(tx.state(), player, target, card)?;
            run_effects(tx, effects).map(|_| Vec::new())
        }
        _ => Err(EffectError::ContentFault("unknown Kuangfu stage".into())),
    }
}

pub(crate) fn resolve_shanjia(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    let count = count.min(7);
    match stage {
        0 => {
            if count == 0
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveShanjia {
                    player,
                    count,
                    stage: 1,
                    skill,
                });
            draw_cards(tx, player, count)
        }
        1 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let required = usize::try_from(count)
                .unwrap_or(usize::MAX)
                .min(options.len());
            if required == 0 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                required.try_into().unwrap_or(u8::MAX),
                required.try_into().unwrap_or(u8::MAX),
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveShanjia,
                vec![player.raw(), required as u32, skill.raw()],
            )
        }
        _ => Err(EffectError::ContentFault("unknown Shanjia stage".into())),
    }
}

pub(crate) fn resolve_mbshanjia(
    tx: &mut Transaction,
    player: PlayerId,
    lost_equipment_count: u32,
    distance_skill_def: engine_api::SkillDefId,
    stage: u8,
    reward_mask: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    if !alive {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbshanjia {
                    player,
                    lost_equipment_count,
                    distance_skill_def,
                    stage: 1,
                    reward_mask: 0,
                    skill,
                });
            draw_cards(tx, player, 3)
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let required = usize::try_from(3u32.saturating_sub(lost_equipment_count))
                .unwrap_or(usize::MAX)
                .min(options.len());
            if required == 0 {
                return resolve_mbshanjia(
                    tx,
                    player,
                    lost_equipment_count,
                    distance_skill_def,
                    2,
                    0b11,
                    skill,
                );
            }
            open_choice(
                tx,
                player,
                required.try_into().unwrap_or(u8::MAX),
                required.try_into().unwrap_or(u8::MAX),
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveMbshanjia,
                vec![
                    player.raw(),
                    required as u32,
                    distance_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            if reward_mask & 0b11 == 0b11 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveMbshanjia {
                        player,
                        lost_equipment_count,
                        distance_skill_def,
                        stage: 3,
                        reward_mask,
                        skill,
                    },
                );
            }
            if reward_mask & 0b10 != 0 {
                add_skill(tx, player, distance_skill_def)
            } else if reward_mask & 0b01 != 0 {
                resolve_mbshanjia(
                    tx,
                    player,
                    lost_equipment_count,
                    distance_skill_def,
                    3,
                    reward_mask,
                    skill,
                )
            } else {
                Ok(Vec::new())
            }
        }
        3 => choose_use_virtual_sha(tx, player, false, true, false, skill, 0, 0),
        _ => Err(EffectError::ContentFault(
            "unknown mobile Shanjia stage".into(),
        )),
    }
}

pub(crate) fn resolve_daoji(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    tw: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || player == target
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || !tx
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
                if tw {
                    rule_api::TargetCardScope::HandEquip
                } else {
                    rule_api::TargetCardScope::Equip
                },
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveDaoji,
                if tw {
                    vec![player.raw(), target.raw(), skill.raw(), 1]
                } else {
                    vec![player.raw(), target.raw(), skill.raw()]
                },
            )
        }
        1 => {
            let Some(card_state) = tx.state().cards.get(card).cloned() else {
                return Ok(Vec::new());
            };
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let is_weapon = card_state.kind == CardKind::Weapon;
            let kind = crate::query::effective_card_kind(tx.state(), player, &card_state);
            let remains_in_hand = hand == Some(card_state.zone) && card_state.owner == player;
            if tw && remains_in_hand && kind.is_basic() {
                return run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 1,
                        skill,
                    }],
                );
            }
            if (!tw && is_weapon) || (tw && remains_in_hand && kind.equip_slot().is_some()) {
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(EffectRequest::ResolveDaoji {
                        player,
                        target,
                        card,
                        stage: 2,
                        tw,
                        skill,
                    });
            }
            let can_use = remains_in_hand
                && tx.state().players.get(player).is_some_and(|participant| {
                    participant.alive
                        && !participant.dying
                        && kind
                            .equip_slot()
                            .is_some_and(|slot| participant.equip_slot_enabled(slot))
                })
                && !crate::query::card_use_cancelled_for_card(tx.state(), player, card);
            if can_use {
                crate::command::play::use_equipment_from_hand(tx, player, card)?;
            }
            Ok(Vec::new())
        }
        2 => super::run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: player,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        _ => Err(EffectError::ContentFault(
            "unknown mobile Daoji stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn give_all_hand_then_choose_use_one_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    stage: u8,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || player == target
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
                    player,
                    target,
                    cards: cards.clone(),
                    targets: Vec::new(),
                    stage: 1,
                    card: CardId::NONE,
                    skill,
                },
            );
            give_all_hand(tx, player, target)
        }
        1 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let options: Vec<_> = cards
                .iter()
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| hand == Some(state.zone) && state.owner == target)
                        && crate::command::play::skill_card_target_spec(tx, target, *card).is_ok()
                })
                .collect();
            if options.is_empty() {
                return run_effects(
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
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                0,
                cards.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::GiveAllHandThenChooseUseOneOrDamage,
                payload,
            )
        }
        2 => {
            crate::command::play::handle_skill_card(tx, target, card, &targets)?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown give-all-hand use-or-damage stage".into(),
        )),
    }
}

pub(crate) fn choose_category_then_target_give_matching_or_sha_unrespondable(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Ok(Vec::new());
    }
    crate::command::play::validate_saved_sha_target(tx, player, card, target)?;
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2, 3],
        ChoiceKind::Custom,
        ContinuationKind::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable,
        vec![0, player.raw(), card.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) const CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE: u32 = 0xDCD9_4980;

pub(crate) fn choose_category_then_target_give_matching_or_damage_plus_one(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || damage_id == 0
        || tx.state().resolution.current_damage_id != damage_id
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Ok(Vec::new());
    }
    let has_hand_equip = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, target))
        .filter_map(|zone| tx.state().zones.get(zone))
        .any(|zone| !zone.cards.is_empty());
    if !has_hand_equip {
        increase_bound_damage_by_one(tx, damage_id)?;
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2, 3],
        ChoiceKind::Custom,
        ContinuationKind::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable,
        vec![
            0,
            player.raw(),
            damage_id,
            target.raw(),
            skill.raw(),
            CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE,
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn choose_lose_hp_use_virtual_sha_by_basic_variants(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    stage: u8,
    damage_bonus: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let damage_bonus = u32::from(tx.state().turn.distinct_basic_card_variants(player) > 1);
            choose_use_virtual_sha(tx, player, true, true, false, skill, 1, damage_bonus)
        }
        1 => {
            let processing = tx.state().zones.find(ZoneKind::Processing, PlayerId::NONE);
            let card_ready = tx
                .state()
                .cards
                .get(card)
                .is_some_and(|state| processing == Some(state.zone) && state.owner.is_none());
            if !card_ready {
                return Ok(Vec::new());
            }
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|state| state.alive && !state.dying)
            {
                let removed = tx
                    .state()
                    .zones
                    .find(ZoneKind::Removed, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("removed zone missing".into()))?;
                move_one_card(
                    tx,
                    card,
                    processing.expect("card_ready requires processing"),
                    removed,
                    PlayerId::NONE,
                    true,
                )?;
                return Ok(Vec::new());
            }
            crate::command::play::handle_virtual_sha_with_policies_and_bonus(
                tx,
                player,
                card,
                &targets,
                true,
                true,
                damage_bonus,
            )?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown lose-HP virtual Sha stage".into(),
        )),
    }
}

/// 叶舞（gwent, ABI 645, ADR 0627）：弃置费用手牌后按种子 RNG 循环——随机敌人随机弃
/// 其一张「he」牌，弃牌颜色与费用相同则重复；每以此法弃敌方手牌加一张雾妖
/// （wuyao_def），每弃敌方装备牌加一张狼（lang_def）。雾妖/狼以技能近似（add_skill，
/// 造卡入口）。循环内模拟已弃牌（从候选中排除），保证终止且消耗 RNG 确定性。
pub(crate) fn resolve_gwyewu_chain(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    cost_cards: Vec<engine_api::CardId>,
    wuyao_def: engine_api::SkillDefId,
    lang_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let he_of = |tx: &Transaction, who: PlayerId| -> Vec<(engine_api::CardId, bool)> {
        let mut out = Vec::new();
        for cid in tx.state().hand_cards(who) {
            out.push((cid, true));
        }
        if let Some(eq) = tx.state().zones.find(ZoneKind::Equip, who) {
            if let Some(zone) = tx.state().zones.get(eq) {
                for cid in &zone.cards {
                    out.push((*cid, false));
                }
            }
        }
        out
    };
    // 费用：弃置所选手牌并记录颜色。
    let mut effects: Vec<EffectRequest> = Vec::new();
    let mut cost_color = 0u8;
    if let Some(cost) = cost_cards.first().copied() {
        if tx.state().hand_cards(player).contains(&cost) {
            cost_color = crate::query::effective_card_color_mask(tx.state(), player, cost);
            effects.push(EffectRequest::DiscardCards {
                player,
                cards: vec![cost],
            });
        }
    }
    // 循环：随机敌人 → 随机 he 牌 → 弃 → 颜色相同则重复（模拟弃牌从候选排除）。
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let mut nh = 0u32;
    let mut ne = 0u32;
    let mut discarded: Vec<engine_api::CardId> = Vec::new();
    let mut per_owner: Vec<(PlayerId, Vec<engine_api::CardId>)> = Vec::new();
    loop {
        let candidates: Vec<PlayerId> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|candidate| *candidate != player)
            .filter(|candidate| !crate::mode::same_side(tx.state(), player, *candidate))
            .filter(|candidate| {
                he_of(tx, *candidate)
                    .iter()
                    .any(|(cid, _)| !discarded.contains(cid))
            })
            .collect();
        if candidates.is_empty() {
            break;
        }
        let enemy = candidates[rng.gen_range_u32(candidates.len() as u32) as usize];
        let available: Vec<_> = he_of(tx, enemy)
            .into_iter()
            .filter(|(cid, _)| !discarded.contains(cid))
            .collect();
        let (cid, in_hand) = available[rng.gen_range_u32(available.len() as u32) as usize];
        if in_hand {
            nh += 1;
        } else {
            ne += 1;
        }
        discarded.push(cid);
        if let Some((_, cards)) = per_owner.iter_mut().find(|(owner, _)| *owner == enemy) {
            cards.push(cid);
        } else {
            per_owner.push((enemy, vec![cid]));
        }
        let color = crate::query::effective_card_color_mask(tx.state(), enemy, cid);
        if cost_color == 0 || color != cost_color {
            break;
        }
    }
    tx.state_mut().rng = rng.into_state();
    for (owner, cards) in per_owner {
        effects.push(EffectRequest::DiscardCards { player: owner, cards });
    }
    for _ in 0..nh {
        effects.push(EffectRequest::AddSkill {
            player,
            skill_def: wuyao_def,
        });
    }
    for _ in 0..ne {
        effects.push(EffectRequest::AddSkill {
            player,
            skill_def: lang_def,
        });
    }
    run_effects(tx, effects)
}

/// 起乱 (qiluan): draw Σ alive players' current-turn kill counts × (3 if the killer is
/// the owner else 1). 何太后 end-phase draw, 上游 skill.js:42081.
pub(crate) fn resolve_qiluan(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut total: u32 = 0;
    for killed in tx.state().players.iter_live() {
        if !killed.alive {
            continue;
        }
        let kills = tx.state().turn.flag(turn_flag::kill_count_by_player(killed.id));
        if kills > 0 {
            total = total.saturating_add(if killed.id == player {
                (kills as u32).saturating_mul(3)
            } else {
                kills as u32
            });
        }
    }
    if total == 0 {
        return Ok(Vec::new());
    }
    let _ = skill;
    draw_cards(tx, player, total)
}

/// 灵杉 (huamu): store the triggering event's used card onto the owner's skill pile
/// (武将牌) when the card currently sits in the shared discard pile. 上游 skill.js:24085.
pub(crate) fn store_event_card_in_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let skill_def = crate::effect::primitives::pack_bingshi_1::skill_pile_def(tx, player, skill)?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let in_discard = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.zone == discard);
    if !in_discard {
        return Ok(Vec::new());
    }
    let pile = crate::effect::primitives::pack_bingshi_1::ensure_skill_pile_zone(tx, player, skill_def)?;
    move_cards(tx, vec![card], pile, Some(discard))
}
