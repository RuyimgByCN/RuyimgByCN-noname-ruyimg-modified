//! Effect primitives for `give_cards` and related helpers.

use super::*;

pub(crate) fn give_cards(
    tx: &mut Transaction,
    cards: Vec<CardId>,
    to: ZoneId,
    from: ZoneId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let giver = tx
        .state()
        .zones
        .get(from)
        .map_or(PlayerId::NONE, |zone| zone.owner);
    give_cards_as(tx, giver, cards, to, from)
}

pub(crate) fn give_cards_as(
    tx: &mut Transaction,
    giver: PlayerId,
    cards: Vec<CardId>,
    to: ZoneId,
    from: ZoneId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let recipient = tx
        .state()
        .zones
        .get(to)
        .map_or(PlayerId::NONE, |zone| zone.owner);
    let given = cards.clone();
    let mut events = move_cards_with_reason(tx, cards, to, Some(from), CardMoveReason::Give)?;
    if giver.is_some() && recipient.is_some() && giver != recipient && !given.is_empty() {
        events.push(DomainEvent::CardsGiven {
            giver,
            recipient,
            cards: given,
        });
    }
    Ok(events)
}

pub(crate) fn move_cards_with_reason(
    tx: &mut Transaction,
    cards: Vec<CardId>,
    to: ZoneId,
    from: Option<ZoneId>,
    reason: CardMoveReason,
) -> Result<Vec<DomainEvent>, EffectError> {
    let to_zone = tx
        .state()
        .zones
        .get(to)
        .ok_or_else(|| EffectError::Illegal(format!("bad to zone {}", to.raw())))?
        .clone();
    let owner = to_zone.owner;
    let face_up = matches!(
        to_zone.kind,
        ZoneKind::DiscardPile
            | ZoneKind::Equip
            | ZoneKind::Processing
            | ZoneKind::Judge
            | ZoneKind::Expansion
    );
    let mut moved_from = from.unwrap_or(ZoneId::NONE);
    let mut equipment_transitions = Vec::new();
    for card in &cards {
        let c = tx
            .state()
            .cards
            .get(*card)
            .ok_or_else(|| EffectError::Illegal(format!("bad card {}", card.raw())))?
            .clone();
        let src = c.zone;
        let playable_pile = reason
            .is_use_or_respond()
            .then(|| {
                tx.state()
                    .skill_piles
                    .iter()
                    .find(|pile| pile.owner == c.owner && pile.zone == src && pile.usable_as_hand)
                    .copied()
            })
            .flatten();
        if let Some(pile) = playable_pile {
            tx.state_mut()
                .turn
                .record_playable_skill_pile_card(c.owner, *card);
            tx.state_mut()
                .turn
                .record_playable_skill_pile(c.owner, pile.skill_def);
        }
        let source_zone = tx
            .state()
            .zones
            .get(src)
            .ok_or_else(|| EffectError::Illegal(format!("bad from zone {}", src.raw())))?;
        equipment_transitions.push((source_zone.kind, c.owner, c.def));
        if from.is_some() && from != Some(src) {
            return Err(EffectError::Illegal("card not in from zone".into()));
        }
        moved_from = src;
        let previous_owner = c.owner;
        move_one_card(tx, *card, src, to, owner, face_up)?;
        // Opt-in per-player「本回合失去牌」计数（沙城 draw-X）：仅当对局中存在
        // 已声明 wants_lost_count 的技能堆（城族）时计数——其他包零额外状态、零哈希漂移。
        if previous_owner.is_some()
            && !reason.is_use_or_respond()
            && (owner != previous_owner
                || matches!(to_zone.kind, ZoneKind::DiscardPile | ZoneKind::DrawPile))
            && tx
                .state()
                .skill_piles
                .iter()
                .any(|pile| pile.wants_lost_count)
        {
            let key = crate::state::turn_flag::lost_card_count_by_player(previous_owner);
            let bumped = tx.state().turn.flag(key) + 1;
            tx.state_mut().turn.set_flag(key, bumped);
        }
    }
    let mut events = vec![DomainEvent::CardsMoved {
        cards,
        from: moved_from,
        to,
        reason,
    }];
    if to_zone.kind == ZoneKind::Equip {
        for (_, _, card_def) in &equipment_transitions {
            let Some(metadata) = tx.state().content.card_metadata(*card_def).cloned() else {
                continue;
            };
            for skill_def in metadata.equip_skills {
                events.extend(add_skill(
                    tx,
                    owner,
                    engine_api::SkillDefId::new(skill_def),
                )?);
            }
            if metadata.on_leave_equip_skill_def != 0 {
                events.extend(add_skill(
                    tx,
                    owner,
                    engine_api::SkillDefId::new(metadata.on_leave_equip_skill_def),
                )?);
            }
        }
    }
    for (source_kind, previous_owner, card_def) in equipment_transitions {
        if source_kind != ZoneKind::Equip
            || (to_zone.kind == ZoneKind::Equip && previous_owner == owner)
            || previous_owner.is_none()
        {
            continue;
        }
        let Some(metadata) = tx.state().content.card_metadata(card_def).cloned() else {
            continue;
        };
        for skill_def in metadata.equip_skills {
            let still_granted = tx
                .state()
                .zones
                .find(ZoneKind::Equip, previous_owner)
                .and_then(|zone| tx.state().zones.get(zone))
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .filter_map(|card| tx.state().cards.get(*card))
                .filter_map(|card| tx.state().content.card_metadata(card.def))
                .any(|card| card.equip_skills.contains(&skill_def));
            if still_granted {
                continue;
            }
            let skill = tx
                .state()
                .players
                .get(previous_owner)
                .into_iter()
                .flat_map(|player| player.skills.iter())
                .find(|skill| {
                    tx.state()
                        .skills
                        .get(**skill)
                        .is_some_and(|instance| instance.def.raw() == skill_def)
                })
                .copied();
            if let Some(skill) = skill {
                events.extend(remove_skill(tx, skill)?);
            }
        }
    }
    Ok(events)
}

/// Move a single card; records journal.
pub fn move_one_card(
    tx: &mut Transaction,
    card: CardId,
    from: ZoneId,
    to: ZoneId,
    new_owner: PlayerId,
    face_up: bool,
) -> Result<(), EffectError> {
    let loss_owner = (from != to && tx.state().content.tracks_turn_card_loss_history())
        .then(|| {
            let from_zone = tx.state().zones.get(from)?;
            let card_state = tx.state().cards.get(card)?;
            let previous_owner = card_state.owner;
            let stays_in_owned_region = tx.state().zones.get(to).is_some_and(|to_zone| {
                to_zone.owner == previous_owner
                    && matches!(to_zone.kind, ZoneKind::Hand | ZoneKind::Equip)
            });
            (previous_owner.is_some()
                && matches!(from_zone.kind, ZoneKind::Hand | ZoneKind::Equip)
                && !stays_in_owned_region)
                .then_some(previous_owner)
        })
        .flatten();
    let leaves_previous_owner_hand = from != to
        && tx
            .state()
            .zones
            .get(from)
            .is_some_and(|zone| zone.kind == ZoneKind::Hand)
        && tx.state().cards.get(card).is_some_and(|state| {
            state.owner.is_some()
                && !(state.owner == new_owner
                    && tx
                        .state()
                        .zones
                        .get(to)
                        .is_some_and(|zone| zone.kind == ZoneKind::Hand))
        });
    let record_gain = from != to
        && new_owner != PlayerId::NONE
        && tx.state().content.tracks_turn_card_gain_count()
        && tx
            .state()
            .zones
            .get(to)
            .is_some_and(|zone| zone.kind == ZoneKind::Hand);
    let (prev_owner, prev_face, prev_index) = {
        let c = tx
            .state()
            .cards
            .get(card)
            .ok_or_else(|| EffectError::EngineFault("card missing".into()))?;
        let idx = tx
            .state()
            .zones
            .get(from)
            .and_then(|z| z.cards.iter().position(|x| *x == card))
            .ok_or_else(|| {
                EffectError::Illegal(format!("card {} not in zone {}", card.raw(), from.raw()))
            })?;
        (c.owner, c.face_up, idx as u32)
    };
    tx.journal(JournalEntry::CardMove {
        card,
        prev_zone: from,
        prev_owner,
        prev_face_up: prev_face,
        prev_index,
    });
    if !tx.state_mut().zones.remove_card(from, card) {
        return Err(EffectError::Illegal(format!(
            "card {} not in zone {}",
            card.raw(),
            from.raw()
        )));
    }
    if !tx.state_mut().zones.push_card(to, card) {
        return Err(EffectError::EngineFault("to zone missing".into()));
    }
    let c = tx
        .state_mut()
        .cards
        .get_mut(card)
        .ok_or_else(|| EffectError::EngineFault("card missing".into()))?;
    c.zone = to;
    c.owner = new_owner;
    c.face_up = face_up;
    if leaves_previous_owner_hand {
        tx.state_mut().face_down_hand_cards.remove(&card);
        let skill_ids: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .filter(|skill| skill.owner == prev_owner)
            .map(|skill| skill.id)
            .collect();
        for skill in skill_ids {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("collected live skill")
                .forget_remembered_hand_card(card);
        }
    }
    if record_gain {
        tx.state_mut().turn.record_card_gain(new_owner, card);
    }
    if from != to
        && new_owner.is_some()
        && tx.state().turn.phase == Phase::Play
        && tx.state().content.tracks_play_phase_card_gain_history()
        && tx
            .state()
            .zones
            .get(to)
            .is_some_and(|zone| zone.kind == ZoneKind::Hand)
    {
        tx.state_mut().turn.record_play_phase_card_gain(new_owner);
    }
    if let Some(previous_owner) = loss_owner {
        tx.state_mut().turn.record_card_loss(previous_owner, card);
        // gaintag_map foundation: record any content tags carried by the lost card so a later
        // trigger can detect "a tagged card was consumed this turn".
        let tags: Vec<u32> = tx
            .state()
            .content_card_tags
            .get(&card)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default();
        for tag in tags {
            tx.state_mut().turn.record_lost_card_tag(tag);
        }
    }
    Ok(())
}

/// Reshuffle discard pile into draw pile using RuleRng.
pub fn reshuffle_discard_into_draw(tx: &mut Transaction) -> Result<(), EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let cards = tx
        .state()
        .zones
        .get(discard)
        .map(|z| z.cards.clone())
        .unwrap_or_default();
    if cards.is_empty() {
        return Ok(());
    }
    for cid in &cards {
        move_one_card(tx, *cid, discard, draw, PlayerId::NONE, false)?;
    }
    // Shuffle draw pile
    let mut pile = tx
        .state()
        .zones
        .get(draw)
        .map(|z| z.cards.clone())
        .unwrap_or_default();
    {
        let mut rng = RuleRng::from_state(tx.state().rng);
        tx.journal(JournalEntry::RngCheckpoint);
        rng.shuffle(&mut pile);
        tx.state_mut().rng = rng.into_state();
    }
    if let Some(z) = tx.state_mut().zones.get_mut(draw) {
        z.cards = pile;
    }
    tx.state_mut().draw_reshuffle_count = tx.state().draw_reshuffle_count.saturating_add(1);
    // Fix card zone/owner already set by move_one
    Ok(())
}

pub(crate) fn use_card(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let intent = crate::intent::resolve_card_use_intent(
        tx.state(),
        crate::intent::CardUseIntent {
            player,
            card,
            targets,
            cancelled: false,
        },
    );
    if intent.cancelled {
        return Ok(Vec::new());
    }
    let borrowed_refills = pothaoshi_refill_effects(tx.state(), player, &[card]);
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), player, card))
        .ok_or_else(|| EffectError::Illegal(format!("bad card {}", card.raw())))?;
    let source_was_hand = tx
        .state()
        .cards
        .get(card)
        .and_then(|card| tx.state().zones.get(card.zone))
        .is_some_and(|zone| zone.kind == ZoneKind::Hand);
    let move_after_use = {
        let c = tx
            .state()
            .cards
            .get(card)
            .ok_or_else(|| EffectError::Illegal(format!("bad card {}", card.raw())))?
            .clone();
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
        let view_as_material =
            tx.state().turn.flag(crate::state::turn_flag::VIEW_AS_CARD) == card.raw() as i32;
        let fusi_material = view_as_material
            && crate::command::skill::dddfusi_approved_source(tx.state(), player, card).is_some();
        let equip = tx.state().zones.find(ZoneKind::Equip, player);
        let planned_source = tx.state().resolution.has_planned_card_source(player, card);
        if kind.is_equip() && (c.zone == hand || planned_source) && !view_as_material {
            None
        } else if c.zone == hand
            || tx.state().playable_hand_cards(player).contains(&card)
            || fusi_material
            || (view_as_material && equip == Some(c.zone))
        {
            if c.zone != hand
                && tx
                    .state()
                    .skill_piles
                    .iter()
                    .any(|pile| pile.owner == player && pile.zone == c.zone && pile.usable_as_hand)
            {
                tx.state_mut()
                    .turn
                    .record_playable_skill_pile_card(player, card);
            }
            let destination_kind = if c.def.is_none() {
                ZoneKind::Removed
            } else {
                ZoneKind::DiscardPile
            };
            let destination = tx
                .state()
                .zones
                .find(destination_kind, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no post-use destination".into()))?;
            Some((c.zone, destination))
        } else {
            let equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .ok_or_else(|| EffectError::Illegal("no equip".into()))?;
            let kind = crate::query::effective_card_kind(tx.state(), player, &c);
            if c.zone != equip || !kind.is_equip() {
                return Err(EffectError::Illegal("card not in hand or equipped".into()));
            }
            None
        }
    };
    if let Some((from, destination)) = move_after_use {
        move_one_card(tx, card, from, destination, PlayerId::NONE, true)?;
    }
    let mut events = Vec::new();
    for refill in borrowed_refills {
        events.extend(apply_effect(tx, refill)?);
    }
    if player == tx.state().turn.current {
        let used = tx.state().turn.flag(turn_flag::CARD_USED);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::CARD_USED, used.saturating_add(1));
        if intent.targets.iter().any(|&target| target != player) {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::CARD_TARGETED_OTHER, 1);
        }
    }
    let view_as_skill_def = tx
        .state()
        .skills
        .get(engine_api::SkillId::new(
            tx.state().turn.flag(turn_flag::VIEW_AS_SKILL).max(0) as u32,
        ))
        .map_or(0, |skill| skill.def.raw());
    tx.state_mut().resolution.register_card_resolution(
        player,
        card,
        kind.tag(),
        intent.targets.clone(),
        source_was_hand,
        view_as_skill_def,
    );
    events.push(DomainEvent::CardUsed {
        player,
        card,
        targets: intent.targets,
    });
    Ok(events)
}

pub(crate) fn use_field_card(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let intent = crate::intent::resolve_card_use_intent(
        tx.state(),
        crate::intent::CardUseIntent {
            player,
            card,
            targets,
            cancelled: false,
        },
    );
    if intent.cancelled {
        return Ok(Vec::new());
    }
    let source = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal(format!("bad card {}", card.raw())))?
        .zone;
    let kind = crate::query::effective_card_kind(
        tx.state(),
        player,
        tx.state().cards.get(card).expect("validated field card"),
    );
    if !tx
        .state()
        .zones
        .get(source)
        .is_some_and(|zone| matches!(zone.kind, ZoneKind::Equip | ZoneKind::Judge))
    {
        return Err(EffectError::Illegal("card not in a field zone".into()));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut events =
        move_cards_with_reason(tx, vec![card], discard, Some(source), CardMoveReason::Use)?;
    if player == tx.state().turn.current {
        let used = tx.state().turn.flag(turn_flag::CARD_USED);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::CARD_USED, used.saturating_add(1));
        if intent.targets.iter().any(|&target| target != player) {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::CARD_TARGETED_OTHER, 1);
        }
    }
    tx.state_mut().resolution.register_card_resolution(
        player,
        card,
        kind.tag(),
        intent.targets.clone(),
        false,
        0,
    );
    events.push(DomainEvent::CardUsed {
        player,
        card,
        targets: intent.targets,
    });
    Ok(events)
}

pub(crate) fn choose_use_virtual_sha(
    tx: &mut Transaction,
    player: PlayerId,
    ignore_distance: bool,
    ignore_use_count: bool,
    draw_on_no_use: bool,
    skill: engine_api::SkillId,
    lose_hp_before_use: u32,
    damage_bonus: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let actor = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad virtual Sha user".into()))?;
    if !actor.alive || actor.dying {
        return Ok(Vec::new());
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no removed zone".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);

    let used = crate::query::sha_play_phase_use_count(tx.state());
    if crate::query::card_use_cancelled_for_card(tx.state(), player, card)
        || (!ignore_use_count
            && used >= crate::query::sha_use_limit_for_card(tx.state(), player, card))
    {
        move_one_card(tx, card, processing, removed, PlayerId::NONE, true)?;
        return if draw_on_no_use {
            draw_cards(tx, player, 1)
        } else {
            Ok(Vec::new())
        };
    }
    let legal: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            crate::command::play::validate_sha_target_with_options(
                tx,
                player,
                card,
                target,
                ignore_distance,
            )
            .is_ok()
        })
        .collect();
    if legal.is_empty() {
        move_one_card(tx, card, processing, removed, PlayerId::NONE, true)?;
        return if draw_on_no_use {
            draw_cards(tx, player, 1)
        } else {
            Ok(Vec::new())
        };
    }
    let (min_count, max_count) = if crate::query::sha_targets_all_for(tx.state(), player) {
        let count = u8::try_from(legal.len()).unwrap_or(u8::MAX);
        (count, count)
    } else {
        (
            1,
            u8::try_from(
                legal
                    .len()
                    .min(crate::query::sha_target_limit_for(tx.state(), player) as usize),
            )
            .unwrap_or(u8::MAX),
        )
    };
    open_choice(
        tx,
        player,
        min_count,
        max_count,
        true,
        legal.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::VirtualShaTargets,
        vec![
            player.raw(),
            card.raw(),
            u32::from(ignore_distance),
            u32::from(ignore_use_count),
            u32::from(draw_on_no_use),
            skill.raw(),
            lose_hp_before_use,
            damage_bonus,
        ],
    )
}
