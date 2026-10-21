//! Trigger operations for `remember_card_action` and related helpers.

use super::*;

pub(crate) fn remember_card_action(tx: &mut Transaction, event: &DomainEvent) {
    if tx.state().turn.phase != Phase::Discard
        && matches!(
            event,
            DomainEvent::CardsMoved {
                cards,
                reason: CardMoveReason::Discard,
                ..
            } if !cards.is_empty()
        )
    {
        tx.state_mut()
            .turn
            .set_flag(turn_flag::NON_DISCARD_PHASE_DISCARD_SEEN, 1);
    }
    if tx.state().content.tracks_owner_card_category_history() {
        match event {
            DomainEvent::CardUsed { player, card, .. } => {
                let category = tx.state().cards.get(*card).and_then(|card| {
                    let kind = crate::query::effective_card_kind(tx.state(), *player, card);
                    if kind.is_basic() {
                        Some(0)
                    } else if kind.is_trick() {
                        Some(1)
                    } else if kind.is_equip() {
                        Some(2)
                    } else {
                        None
                    }
                });
                if let Some(category) = category {
                    tx.state_mut()
                        .turn
                        .remember_owner_card_category(*player, category);
                }
            }
            DomainEvent::CardsMoved {
                cards,
                from,
                reason: CardMoveReason::Discard,
                ..
            } => {
                let owner = tx.state().zones.get(*from).and_then(|zone| {
                    matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip).then_some(zone.owner)
                });
                if let Some(owner) = owner.filter(|owner| !owner.is_none()) {
                    let categories: Vec<_> = cards
                        .iter()
                        .filter_map(|card| {
                            tx.state().cards.get(*card).and_then(|card| {
                                let kind =
                                    crate::query::effective_card_kind(tx.state(), owner, card);
                                if kind.is_basic() {
                                    Some(0)
                                } else if kind.is_trick() {
                                    Some(1)
                                } else if kind.is_equip() {
                                    Some(2)
                                } else {
                                    None
                                }
                            })
                        })
                        .collect();
                    for category in categories {
                        tx.state_mut()
                            .turn
                            .remember_owner_card_category(owner, category);
                    }
                }
            }
            _ => {}
        }
    }
    if tx.state().content.tracks_discard_phase_discard_count()
        && tx.state().turn.phase == Phase::Discard
    {
        if let DomainEvent::CardsMoved {
            cards,
            from,
            reason: CardMoveReason::Discard,
            ..
        } = event
        {
            let counts = tx.state().zones.get(*from).is_some_and(|zone| {
                zone.owner == tx.state().turn.current
                    && matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip)
            });
            if counts {
                let current = tx.state().turn.flag(turn_flag::DISCARD_PHASE_DISCARDED);
                tx.state_mut().turn.set_flag(
                    turn_flag::DISCARD_PHASE_DISCARDED,
                    current.saturating_add(i32::try_from(cards.len()).unwrap_or(i32::MAX)),
                );
            }
        }
    }
    // twjieqiu 劫囚（§7 装备栏编排）：BUFF 标记目标的每次弃牌即时等量恢复
    // 禁用槽——独立顶层块，不受 tracks_discard_phase_discard_count 门控。
    if let DomainEvent::CardsMoved {
        cards,
        reason: CardMoveReason::Discard,
        ..
    } = event
    {
        let mover = tx.state().turn.current;
        let buffed = tx.state().players.get(mover).is_some_and(|p| {
            p.alive
                && p.marks
                    .iter()
                    .any(|(id, count)| *id == crate::content::TWJIEQIU_BUFF_MARK && *count > 0)
        });
        if buffed {
            let mut remaining = cards.len() as u32;
            for tag in 1u8..=5u8 {
                if remaining == 0 {
                    break;
                }
                let Some(slot) = crate::state::EquipSlot::from_tag(tag) else {
                    continue;
                };
                let index = slot.index();
                let still_disabled = tx
                    .state()
                    .players
                    .get(mover)
                    .and_then(|p| p.disabled_equip_slots.get(index))
                    .copied()
                    .unwrap_or(0)
                    > 0;
                if still_disabled {
                    if let Some(p) = tx.state_mut().players.get_mut(mover) {
                        if let Some(count) = p.disabled_equip_slots.get_mut(index) {
                            *count -= 1;
                        }
                    }
                    remaining -= 1;
                }
            }
        }
    }
    if tx.state().content.tracks_turn_discarded_cards() {
        if let DomainEvent::CardsMoved {
            cards,
            from,
            to,
            reason: CardMoveReason::Discard,
            ..
        } = event
        {
            let from_player = tx
                .state()
                .zones
                .get(*from)
                .is_some_and(|zone| zone.owner.is_some());
            let to_discard = tx
                .state()
                .zones
                .get(*to)
                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile);
            if from_player && to_discard {
                tx.state_mut()
                    .turn
                    .record_discarded_cards(cards.iter().copied());
            }
        }
    }
    if tx.state().content.tracks_turn_haoyi_cards() {
        if let DomainEvent::CardsMoved { cards, to, .. } = event {
            let to_discard = tx
                .state()
                .zones
                .get(*to)
                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile);
            if to_discard {
                tx.state_mut()
                    .turn
                    .record_entered_discard_cards(cards.iter().copied());
            }
        }
    }
    if tx.state().content.tracks_phase_target_hp_history()
        && tx.state().turn.flag(turn_flag::PHASE_HISTORY_SUPPRESSED) == 0
    {
        match event {
            DomainEvent::CardUsed { targets, .. } => {
                for target in targets {
                    tx.state_mut().turn.mark_phase_card_target(*target);
                }
            }
            DomainEvent::CardTargetAdded { target, .. } => {
                tx.state_mut().turn.mark_phase_card_target(*target);
            }
            _ => {}
        }
    }
    if tx.state().content.tracks_discard_pile_suit_history() {
        if let DomainEvent::CardsMoved { cards, to, .. } = event {
            if tx
                .state()
                .zones
                .get(*to)
                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
            {
                let suits: Vec<_> = cards
                    .iter()
                    .filter_map(|card| tx.state().cards.get(*card).map(|card| card.suit))
                    .collect();
                tx.state_mut().turn.remember_discard_pile_suits(suits);
            }
        }
    }
    if let DomainEvent::CardsMoved {
        from,
        reason: CardMoveReason::Discard,
        ..
    } = event
    {
        if let Some(owner) = tx.state().zones.get(*from).and_then(|zone| {
            (zone.kind == ZoneKind::Hand && zone.owner.is_some()).then_some(zone.owner)
        }) {
            let key = turn_flag::hand_discard_event_by_player(owner);
            let count = tx.state().turn.flag(key);
            tx.state_mut().turn.set_flag(key, count.saturating_add(1));
        }
    }
    if let DomainEvent::CardUsed {
        player,
        card,
        targets,
    } = event
    {
        if tx.state().content.tracks_ddd_chengjing_history()
            && *player == tx.state().turn.current
            && tx.state().turn.flag(turn_flag::VIEW_AS_SKILL) == 0
        {
            let materials = tx.state().resolution.card_source_cards(*player, *card);
            if let [material] = materials.as_slice() {
                let eligible = tx.state().cards.get(*material).is_some_and(|physical| {
                    physical.def.is_some()
                        && (physical.kind.is_basic()
                            || (physical.kind.is_trick()
                                && !physical.kind.is_delayed_trick()
                                && !matches!(
                                    physical.kind,
                                    CardKind::TrickWuxie | CardKind::TrickYoudishenru
                                )))
                });
                if eligible {
                    tx.state_mut()
                        .turn
                        .ddd_chengjing_eligible_uses
                        .push(*material);
                }
            }
        }
        if tx.state().content.tracks_turn_card_target_counts() {
            for target in targets
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
            {
                tx.state_mut().turn.record_card_target(*player, target);
            }
        }
        let tracks_red_targets = tx.state().content.tracks_turn_red_card_target_history();
        let card_is_red = tracks_red_targets
            && tx.state().cards.get(*card).is_some_and(|value| {
                matches!(
                    crate::query::effective_card_suit(tx.state(), *player, value),
                    engine_api::Suit::Heart | engine_api::Suit::Diamond
                )
            });
        if card_is_red {
            for target in targets {
                tx.state_mut().turn.remember_red_card_target(*target, *card);
            }
        }
    }
    let (action_player, action_card) = match event {
        DomainEvent::CardUsed { player, card, .. }
        | DomainEvent::CardResponded { player, card, .. } => (*player, *card),
        _ => return,
    };
    if tx.state().content.tracks_turn_use_response_discard_count() && action_card.is_some() {
        let in_discard = tx
            .state()
            .cards
            .get(action_card)
            .and_then(|card| tx.state().zones.get(card.zone))
            .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile);
        if in_discard {
            let count = tx.state().turn.flag(turn_flag::USE_RESPONSE_DISCARDED);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::USE_RESPONSE_DISCARDED, count.saturating_add(1));
        }
    }
    if tx
        .state()
        .content
        .tracks_per_player_turn_card_action_count()
    {
        let key = turn_flag::card_action_count_by_player(action_player);
        let count = tx.state().turn.flag(key);
        tx.state_mut().turn.set_flag(key, count.saturating_add(1));
    }
    if tx.state().content.tracks_lifetime_card_action_count() {
        let total = tx
            .state_mut()
            .card_action_totals
            .entry(action_player)
            .or_insert(0);
        *total = total.saturating_add(1);
    }
    let DomainEvent::CardUsed {
        player,
        card,
        targets,
    } = event
    else {
        return;
    };
    if tx.state().content.tracks_turn_used_card_suit_history() {
        if let Some(suit) = tx
            .state()
            .cards
            .get(*card)
            .map(|value| crate::query::effective_card_suit(tx.state(), *player, value))
        {
            tx.state_mut().turn.remember_used_card_suit(*player, suit);
        }
    }
    if tx.state().content.tracks_turn_basic_card_variant_history() {
        let variant = tx.state().cards.get(*card).and_then(|value| {
            match crate::query::effective_card_kind(tx.state(), *player, value) {
                CardKind::Sha => Some(0),
                CardKind::ShaFire => Some(1),
                CardKind::ShaThunder => Some(2),
                CardKind::Shan => Some(3),
                CardKind::Tao => Some(4),
                CardKind::Jiu => Some(5),
                _ => None,
            }
        });
        if let Some(variant) = variant {
            tx.state_mut()
                .turn
                .remember_basic_card_variant(*player, variant);
        }
    }
    if *player == tx.state().turn.current
        && tx.state().turn.phase == Phase::Play
        && tx.state().content.tracks_per_player_play_phase_card_count()
    {
        let key = turn_flag::play_phase_card_used_by_player(*player);
        let count = tx.state().turn.flag(key);
        tx.state_mut().turn.set_flag(key, count.saturating_add(1));
    }
    if *player == tx.state().turn.current
        && tx.state().turn.phase == Phase::Play
        && tx.state().content.tracks_play_phase_card_suit_history()
    {
        if let Some(suit) = tx
            .state()
            .cards
            .get(*card)
            .map(|value| crate::query::effective_card_suit(tx.state(), *player, value))
        {
            tx.state_mut()
                .turn
                .remember_play_phase_card_suit_used(*player, suit);
        }
    }
    if tx.state().turn.phase == Phase::Play
        && (*player == tx.state().turn.current
            || tx
                .state()
                .content
                .tracks_play_card_category_history_all_users())
        && (tx.state().content.tracks_play_card_category_history()
            || tx.state().content.tracks_play_card_category_duplicates())
    {
        let category = tx.state().cards.get(*card).and_then(|card| {
            let kind = crate::query::effective_card_kind(tx.state(), *player, card);
            if kind.is_basic() {
                Some(0)
            } else if kind.is_trick() {
                Some(1)
            } else if kind.is_equip() {
                Some(2)
            } else {
                None
            }
        });
        if let Some(category) = category {
            let track_duplicate = tx.state().content.tracks_play_card_category_duplicates();
            tx.state_mut().turn.remember_play_card_category_used(
                *player,
                category,
                track_duplicate,
            );
        }
    }
    if tx.state().content.tracks_per_player_turn_sha_use_count() {
        let key = tx.state().cards.get(*card).and_then(|card| {
            match crate::query::effective_card_kind(tx.state(), *player, card) {
                kind if kind.is_sha() => Some(turn_flag::sha_used_by_player(*player)),
                CardKind::TrickJuedou => Some(turn_flag::juedou_used_by_player(*player)),
                _ => None,
            }
        });
        if let Some(key) = key {
            let count = tx.state().turn.flag(key);
            tx.state_mut().turn.set_flag(key, count.saturating_add(1));
        }
    }
    let track_turn =
        *player == tx.state().turn.current && tx.state().content.tracks_turn_card_history();
    let track_turn_name =
        *player == tx.state().turn.current && tx.state().content.tracks_turn_card_name_history();
    let track_play = *player == tx.state().turn.current
        && tx.state().turn.phase == Phase::Play
        && tx.state().content.tracks_play_card_history();
    let track_count = tx.state().content.tracks_per_player_turn_card_count();
    let track_game = tx.state().content.tracks_game_card_history();
    if !track_turn && !track_turn_name && !track_play && !track_count && !track_game {
        return;
    }
    if track_turn_name {
        let (kind, def) = {
            let state = tx.state();
            let Some(card) = state.cards.get(*card) else {
                return;
            };
            (
                crate::query::effective_card_kind(state, *player, card).tag(),
                card.def.raw(),
            )
        };
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_CARD_KIND, i32::from(kind));
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_CARD_DEF, def as i32);
        let mut mask: i32 = 0;
        for target in targets {
            if let Some(bit) = target
                .raw()
                .checked_sub(1)
                .and_then(|shift| 1_i32.checked_shl(shift))
            {
                mask |= bit;
            }
        }
        tx.state_mut()
            .turn
            .set_flag(turn_flag::LAST_CARD_TARGETS, mask);
        let is_sha = tx.state().cards.get(*card).is_some_and(|value| {
            crate::query::effective_card_kind(tx.state(), *player, value).is_sha()
        });
        if is_sha {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::LAST_SHA_TARGETS, mask);
        }
    }
    if track_turn || track_play || track_game {
        let meta = {
            let state = tx.state();
            let Some(card) = state.cards.get(*card) else {
                return;
            };
            let suit = crate::query::effective_card_suit(state, *player, card);
            u32::from(crate::command::judge::suit_tag(suit))
                | (u32::from(card.rank) << 8)
                | (u32::from(state.turn.phase.to_u8()) << 16)
        };
        if track_turn {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::LAST_CARD_META, meta as i32);
        }
        if track_play {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::LAST_PLAY_CARD_META, meta as i32);
        }
        if track_game {
            tx.state_mut().turn.set_flag(
                turn_flag::last_game_card_meta_by_player(*player),
                meta as i32,
            );
        }
    }
    if track_count {
        let key = turn_flag::card_used_by_player(*player);
        let count = tx.state().turn.flag(key);
        tx.state_mut().turn.set_flag(key, count.saturating_add(1));
        // NOTE: round-scoped card-use counting (cards_used_this_round) is recorded lazily by
        // content that opts into the round-history ABI, not here, to keep the canonical state
        // hash stable for packages that pre-date the round-history query.
    }
}

pub(crate) fn plan_triggers(
    tx: &mut Transaction,
    ctx: TriggerContext,
    fires: &mut TriggerFireRecord,
) -> Result<Vec<PendingTrigger>, EffectError> {
    if tx.state().resolution.trigger_budget == 0 {
        return Err(EffectError::BudgetExceeded);
    }

    let alive = tx.state().players.alive_by_seat();
    let current = tx.state().turn.current;
    let kind = ctx.kind;

    // Clone actives so we can mutably update skill turn_fire_count later.
    let ordered: Vec<(TriggerOrderKey, ActiveTrigger)> = tx
        .state()
        .triggers
        .collect_sorted(kind, |active| {
            let seat_rel = seat_rel_to_current(&alive, current, active.owner);
            TriggerOrderKey::new(
                timing_from_u16(active.timing),
                active.priority,
                active.optional,
                seat_rel,
                active.skill_id.raw(),
                active.trigger_def.raw(),
            )
        })
        .into_iter()
        .map(|(k, a)| (k, a.clone()))
        .collect();

    let mut pending = Vec::new();
    for (_key, active) in ordered {
        let view_as_skill = tx.state().turn.flag(turn_flag::VIEW_AS_SKILL).max(0) as u32;
        let view_as_card = tx.state().turn.flag(turn_flag::VIEW_AS_CARD).max(0) as u32;
        let history = TriggerHistorySnapshot::capture(tx.state(), &active, &ctx, view_as_skill);
        if !filter_match(
            tx.state(),
            &active,
            &ctx,
            view_as_skill,
            view_as_card,
            Some(&history),
        ) {
            continue;
        }
        let def = tx
            .state()
            .content
            .find_def(active.trigger_def)
            .ok_or_else(|| {
                EffectError::EngineFault(format!(
                    "missing trigger def {}",
                    active.trigger_def.raw()
                ))
            })?;

        let count = fires.get(active.skill_id, active.trigger_def);
        if count >= def.max_per_segment {
            continue;
        }
        if let Some(limit) = turn_limit(&def.filter) {
            let fired = tx
                .state()
                .skills
                .get(active.skill_id)
                .map(|skill| skill.turn_fire_count)
                .unwrap_or(0);
            if fired.saturating_add(count) >= limit {
                continue;
            }
        }
        if let Some(limit) = play_phase_limit(&def.filter) {
            let fired = tx
                .state()
                .skills
                .get(active.skill_id)
                .map(|skill| skill.play_phase_fire_count)
                .unwrap_or(0);
            if fired.saturating_add(count) >= limit {
                continue;
            }
        }
        if once_per_round(&def.filter) {
            let fired_this_round = tx
                .state()
                .skills
                .get(active.skill_id)
                .is_some_and(|skill| skill.round_fire_round == tx.state().turn.round);
            if fired_this_round || count > 0 {
                continue;
            }
        }
        if let Some(limit) = lifetime_limit(&def.filter) {
            let fired = tx
                .state()
                .skills
                .get(active.skill_id)
                .map(|skill| skill.lifetime_fire_count)
                .unwrap_or(0);
            if fired.saturating_add(count) >= limit {
                continue;
            }
        }

        let repeat = if matches!(
            def.filter,
            TriggerFilter::OwnerSourceOrTargetOtherAlivePerDamagePoint
                | TriggerFilter::OwnerSourceDistanceLeOnePerDamagePoint
                | TriggerFilter::OwnerIsEventPlayerOtherSourceAlivePerDamagePoint
                | TriggerFilter::OwnerIsEventPlayerPerDamagePoint
                | TriggerFilter::OwnerIsEventPlayerPerDamagePointOtherHandEquipJudgeNonempty
                | TriggerFilter::OwnerIsEventPlayerSourceAliveGainableCardPerDamagePoint
                | TriggerFilter::OwnerIsTargetSourceHandNonemptyPerDamagePoint
                | TriggerFilter::OwnerFromEquipPerCard
        ) {
            ctx.amount
        } else {
            1
        }
        .min(def.max_per_segment.saturating_sub(count));
        for _ in 0..repeat {
            fires.inc(active.skill_id, active.trigger_def);
            if tx.state().resolution.trigger_budget == 0 {
                return Err(EffectError::BudgetExceeded);
            }
            pending.push(PendingTrigger {
                active: active.clone(),
                context: ctx.clone(),
                view_as_skill,
                view_as_card,
                history: Some(history),
            });
        }
    }
    Ok(pending)
}

/// Execute an ordered trigger queue until completion or a choice barrier.
///
/// A queued trigger keeps its original event context, but its filter is re-evaluated against the
/// current state immediately before execution. Effects from an earlier trigger are committed
/// before the next trigger is considered so replacement/exclusion filters can invalidate stale
/// queued work without opening an optional prompt.
pub fn run_pending_triggers(
    tx: &mut Transaction,
    pending: Vec<PendingTrigger>,
) -> Result<Vec<EffectRequest>, EffectError> {
    let mut queue: VecDeque<_> = pending.into();
    let mut effects = Vec::new();
    loop {
        if !effects.is_empty() && !queue.is_empty() {
            match crate::effect::run_effects(tx, std::mem::take(&mut effects)) {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {
                    tx.state_mut().resolution.deferred_triggers.extend(queue);
                    return Err(EffectError::ChoiceRequested);
                }
                Err(error) => return Err(error),
            }
        }

        let Some(task) = queue.pop_front() else {
            break;
        };
        if !filter_match(
            tx.state(),
            &task.active,
            &task.context,
            task.view_as_skill,
            task.view_as_card,
            task.history.as_ref(),
        ) {
            continue;
        }
        if task.active.optional {
            let owner = task.active.owner;
            let choice_actor = tx
                .state()
                .content
                .find_def(task.active.trigger_def)
                .map(|definition| definition.choice_actor)
                .ok_or_else(|| {
                    EffectError::EngineFault(format!(
                        "missing trigger def {}",
                        task.active.trigger_def.raw()
                    ))
                })?;
            let observer = match choice_actor {
                TriggerChoiceActor::Owner => owner,
                TriggerChoiceActor::EventPlayer => task.context.player,
                TriggerChoiceActor::EventSource => task.context.source,
            };
            if observer.is_none() {
                return Err(EffectError::ContentFault(format!(
                    "optional trigger {} has no {:?} choice actor",
                    task.active.trigger_def.raw(),
                    choice_actor
                )));
            }
            let mut confirmed = task;
            confirmed.active.optional = false;
            let r = &mut tx.state_mut().resolution;
            r.optional_trigger = Some(confirmed);
            r.deferred_effects.extend(effects);
            r.deferred_triggers.extend(queue);
            crate::effect::open_choice(
                tx,
                observer,
                1,
                1,
                false,
                vec![0, 1],
                engine_api::ChoiceKind::Confirm,
                crate::state::ContinuationKind::OptionalTrigger,
                Vec::new(),
            )?;
            return Err(EffectError::ChoiceRequested);
        }
        spend_trigger_budget(tx, 1)?;
        let marker = [
            task.active.skill_id.raw(),
            task.active.trigger_def.raw(),
            task.context.kind.tag(),
        ];
        let handler = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .ok_or_else(|| {
                EffectError::EngineFault(format!(
                    "missing trigger def {}",
                    task.active.trigger_def.raw()
                ))
            })?
            .handler
            .clone();
        let turn_limited = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .and_then(|definition| turn_limit(&definition.filter))
            .is_some();
        if turn_limited {
            if let Some(skill) = tx.state_mut().skills.get_mut(task.active.skill_id) {
                skill.turn_fire_count = skill.turn_fire_count.saturating_add(1);
            }
        }
        let play_phase_limited = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .and_then(|definition| play_phase_limit(&definition.filter))
            .is_some();
        if play_phase_limited {
            if let Some(skill) = tx.state_mut().skills.get_mut(task.active.skill_id) {
                skill.play_phase_fire_count = skill.play_phase_fire_count.saturating_add(1);
            }
        }
        let round_limited = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .is_some_and(|definition| once_per_round(&definition.filter));
        if round_limited {
            let round = tx.state().turn.round;
            if let Some(skill) = tx.state_mut().skills.get_mut(task.active.skill_id) {
                skill.round_fire_round = round;
            }
        }
        let lifetime_limited = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .and_then(|definition| lifetime_limit(&definition.filter))
            .is_some();
        if lifetime_limited {
            if let Some(skill) = tx.state_mut().skills.get_mut(task.active.skill_id) {
                skill.lifetime_fire_count = skill.lifetime_fire_count.saturating_add(1);
            }
        }
        let first_for_target = tx
            .state()
            .content
            .find_def(task.active.trigger_def)
            .is_some_and(|definition| {
                definition.filter == TriggerFilter::OwnerAliveOtherDyingFirstPerTarget
            });
        if first_for_target {
            if let Some(skill) = tx.state_mut().skills.get_mut(task.active.skill_id) {
                skill.record_lifetime_target(task.context.player);
            }
        }

        if let TriggerHandler::RunVm { module, entry_pc } = handler {
            let Some(bytecode) = tx.state().content.module(&module).cloned() else {
                effects.push(EffectRequest::EmitRuleEvent {
                    code: 101,
                    payload: vec![task.active.skill_id.raw()],
                });
                effects.push(trigger_marker(marker));
                continue;
            };
            let _ = entry_pc;
            let fuel = tx.state().resolution.effect_budget.min(10_000);
            let ctx = [
                task.active.owner.raw(),
                task.context.player.raw(),
                task.context.source.raw(),
                task.context.card.map(|c| c.raw()).unwrap_or(0),
            ];
            let expose_only_raw_black_discards = tx
                .state()
                .content
                .find_def(task.active.trigger_def)
                .is_some_and(|definition| {
                    matches!(
                        definition.filter,
                        TriggerFilter::OwnerAliveOtherDiscardedBlackToDiscard
                    )
                });
            let event_cards: Vec<_> = task
                .context
                .cards
                .iter()
                .filter(|card| {
                    !expose_only_raw_black_discards
                        || tx.state().cards.get(**card).is_some_and(|card_state| {
                            crate::query::card_color_mask_for_suit(card_state.suit) == 0b10
                        })
                })
                .map(|card| card.raw())
                .collect();
            let event_targets: Vec<_> = task
                .context
                .targets
                .iter()
                .map(|target| target.raw())
                .collect();
            let outcome = run_module_with_event_cards(
                tx.state(),
                &bytecode,
                task.active.skill_id.raw(),
                ctx,
                task.context.amount.max(task.context.target_count),
                &event_cards,
                &event_targets,
                task.context.nature,
                task.context.card_kind.unwrap_or(0),
                task.context.card_flags,
                task.context
                    .to_zone
                    .and_then(|zone| tx.state().zones.get(zone))
                    .map(|zone| zone.owner)
                    .unwrap_or(PlayerId::NONE),
                ddd_zhengjun_move_mask(tx.state(), &task.context),
                fuel,
            )
            .map_err(|e| EffectError::ContentFault(format!("vm: {e}")))?;
            effects.extend(outcome.effects);
            if let Some(pause) = outcome.paused {
                let r = &mut tx.state_mut().resolution;
                r.deferred_effects.extend(effects);
                r.deferred_triggers.extend(queue);
                crate::command::resolve_choice::open_vm_choice(
                    tx,
                    task.active.owner,
                    &module,
                    task.active.skill_id.raw(),
                    ctx,
                    marker,
                    pause,
                )?;
                return Err(EffectError::ChoiceRequested);
            }
        } else {
            effects.extend(native_handler_effects(&task.active, &handler));
        }
        effects.push(trigger_marker(marker));
    }
    Ok(effects)
}

pub(crate) fn trigger_marker(marker: [u32; 3]) -> EffectRequest {
    EffectRequest::EmitRuleEvent {
        code: 100,
        payload: marker.to_vec(),
    }
}

/// Card-name-scoped filter helper (Phase 3): returns true when the event's originating card
/// (from `ctx.card`, resolved against `state`) has a logical kind/suit matching `selector`.
/// Falls back to `ctx.card_kind` when the physical card is unavailable.
pub(crate) fn event_card_matches_selector(
    state: &crate::state::GameState,
    owner: engine_api::PlayerId,
    ctx: &TriggerContext,
    selector: crate::query::CardSelector,
) -> bool {
    if let Some(card_id) = ctx.card {
        if let Some(card) = state.cards.get(card_id) {
            let suit = crate::query::effective_card_suit(state, owner, card);
            return selector.matches_physical_card(state, owner, card, suit);
        }
    }
    // Fallback: use the event's resolved card kind tag and no suit constraint.
    matches!(selector, crate::query::CardSelector::Any)
        || ctx
            .card_kind
            .and_then(CardKind::from_tag)
            .is_some_and(|kind| selector.matches_kind_and_suit(kind, engine_api::Suit::Spade))
}

/// 神君: whether the owner's hand holds a card sharing the used card's
/// effective kind (同名牌). The used card is 杀 or a trick per the skill text.
pub(crate) fn hand_has_same_kind_as_event_card(
    state: &crate::state::GameState,
    owner: engine_api::PlayerId,
    ctx: &TriggerContext,
) -> bool {
    let Some(event_card) = ctx.card.and_then(|card| state.cards.get(card)) else {
        return false;
    };
    let kind = crate::query::effective_card_kind(state, owner, event_card);
    if !(kind.is_sha() || kind.is_trick()) {
        return false;
    }
    state.hand_cards(owner).into_iter().any(|card| {
        state.cards.get(card).is_some_and(|candidate| {
            crate::query::effective_card_kind(state, owner, candidate) == kind
        })
    })
}
