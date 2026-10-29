//! Effect primitives for `choose_gain_target_card_alternate_zone_then_target_draw` and related helpers.

use super::*;

/// 惬诣 (clanqieyi) play-phase start: reset the per-suit first-use storage keys and
/// privately view the top two draw-pile cards.
///
/// Upstream `phaseUseBegin` shows the top 2 without reordering (viewCards). The per-suit
/// first-use gate (upstream `clanqieyi_effect` temp skill tracking used suits) is
/// approximated with per-suit storage keys 10..13 (suit tag offset), cleared here so each
/// play phase restarts the gate.
pub(crate) fn qieyi_phase_start(
    tx: &mut Transaction,
    player: engine_api::PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    for suit in 0..4 {
        tx.state_mut()
            .skills
            .storage_set(skill, QIEYI_SUIT_KEY_BASE + suit, 0);
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("惬诣 draw pile missing".into()))?;
    if tx
        .state()
        .zones
        .get(draw)
        .is_none_or(|zone| zone.cards.is_empty())
    {
        reshuffle_discard_into_draw(tx)?;
    }
    let top: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| zone.cards.iter().rev().take(2).copied().collect())
        .unwrap_or_default();
    if top.is_empty() {
        return Ok(Vec::new());
    }
    Ok(vec![engine_api::DomainEvent::CardsRevealed {
        cards: top,
        to: vec![player],
    }])
}

/// 惬诣 (clanqieyi) card-used: on the first use of a suit this play phase, reveal the top
/// draw-pile card; same color or same kind as the used card → gain it, otherwise open the
/// put-one-card-on-top choice.
pub(crate) fn qieyi_use_after(
    tx: &mut Transaction,
    player: engine_api::PlayerId,
    event_card: engine_api::CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    let suit_tag = {
        let Some(event) = tx.state().cards.get(event_card) else {
            return Ok(Vec::new());
        };
        crate::query::effective_card_suit(tx.state(), player, event) as u32
    };
    if suit_tag >= 4 {
        return Ok(Vec::new());
    }
    let key = QIEYI_SUIT_KEY_BASE + suit_tag;
    if tx.state().skills.storage_get(skill, key) != 0 {
        return Ok(Vec::new());
    }
    tx.state_mut().skills.storage_set(skill, key, 1);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("惬诣 draw pile missing".into()))?;
    if tx
        .state()
        .zones
        .get(draw)
        .is_none_or(|zone| zone.cards.is_empty())
    {
        reshuffle_discard_into_draw(tx)?;
    }
    let Some(top) = tx
        .state()
        .zones
        .get(draw)
        .and_then(|zone| zone.cards.last().copied())
    else {
        return Ok(Vec::new());
    };
    let mut events = vec![engine_api::DomainEvent::CardsRevealed {
        cards: vec![top],
        to: Vec::new(),
    }];
    let (same_color, used_kind, top_kind) = {
        let (Some(event), Some(top_card)) =
            (tx.state().cards.get(event_card), tx.state().cards.get(top))
        else {
            return Ok(events);
        };
        let used_kind = crate::query::effective_card_kind(tx.state(), player, event);
        let top_kind = crate::query::effective_card_kind(tx.state(), player, top_card);
        let top_suit = crate::query::effective_card_suit(tx.state(), player, top_card);
        let suit = match suit_tag {
            0 => engine_api::Suit::Spade,
            1 => engine_api::Suit::Heart,
            2 => engine_api::Suit::Club,
            _ => engine_api::Suit::Diamond,
        };
        let same_color = crate::query::card_color_mask_for_suit(suit) != 0
            && crate::query::card_color_mask_for_suit(suit)
                == crate::query::card_color_mask_for_suit(top_suit);
        (same_color, used_kind, top_kind)
    };
    if same_color || used_kind == top_kind {
        // Same color or same kind → gain the revealed card.
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("惬诣 hand zone missing".into()))?;
        crate::effect::primitives::pack_mobile::move_one_card(tx, top, draw, hand, player, true)?;
        return Ok(events);
    }
    // Otherwise: choose one hand/equip card to put on top of the pile.
    let hand_zone = tx.state().zones.find(ZoneKind::Hand, player);
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, player);
    let mut options: Vec<u32> = Vec::new();
    if let Some(zone) = hand_zone {
        if let Some(zone_state) = tx.state().zones.get(zone) {
            options.extend(zone_state.cards.iter().map(|card| card.raw()));
        }
    }
    if let Some(zone) = equip_zone {
        if let Some(zone_state) = tx.state().zones.get(zone) {
            options.extend(zone_state.cards.iter().map(|card| card.raw()));
        }
    }
    if options.is_empty() {
        return Ok(events);
    }
    events.extend(open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::QieyiPutTopChoice,
        vec![0, player.raw(), skill.raw()],
    )?);
    Ok(events)
}

/// Stage 0 of the 惬诣 put-on-top choice: validate the selected hand/equip card and move
/// it to the top of the draw pile (the draw pile's top is its last element).
pub(crate) fn resolve_qieyi_put_top_choice(
    tx: &mut Transaction,
    observer: engine_api::PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&0);
    let player = engine_api::PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "惬诣 must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "惬诣 requires exactly one card".into(),
                ));
            };
            let card = engine_api::CardId::new(*card);
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|state| state.zone)
                .ok_or_else(|| EffectError::Illegal("惬诣 card missing".into()))?;
            let from_zone = tx
                .state()
                .zones
                .get(from)
                .ok_or_else(|| EffectError::Illegal("惬诣 card zone missing".into()))?;
            if from_zone.owner != player
                || !matches!(from_zone.kind, ZoneKind::Hand | ZoneKind::Equip)
            {
                return Err(EffectError::Illegal(
                    "惬诣 card must be in the owner's hand or equipment".into(),
                ));
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("惬诣 draw pile missing".into()))?;
            if !tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.owner == player)
            {
                return Err(EffectError::Illegal("惬诣 skill missing".into()));
            }
            crate::effect::primitives::pack_mobile::move_one_card(
                tx,
                card,
                from,
                draw,
                PlayerId::NONE,
                true,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::ContentFault("惬诣 stage out of range".into())),
    }
}

/// 谏直 (clanjianzhi) phase-end: if 惬诣 (clanqieyi) fired this turn, open the judgment
/// count choice (1..=惬诣 fires). The continuation runs the judgments, assigns
/// matching-suit judge cards to players, and deals 1 thunder damage per non-matching
/// judge card.
///
/// Cross-skill state query: the 惬诣 instance's `turn_fire_count` (incremented by the
/// trigger dispatcher on every fire) approximates upstream `getHistory("useSkill",
/// clanqieyi).length` for the current turn.
pub(crate) fn jianzhi_phase_end(
    tx: &mut Transaction,
    player: engine_api::PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<engine_api::DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    let qieyi_fires = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .filter(|instance| instance.owner == player && instance.def.raw() == JIANZHI_CLANQIEYI)
        .map(|instance| instance.turn_fire_count)
        .next()
        .unwrap_or(0);
    if qieyi_fires == 0 {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = (1..=qieyi_fires).collect();
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        engine_api::ChoiceKind::Custom,
        crate::state::ContinuationKind::JianzhiCountChoice,
        vec![0, player.raw(), skill.raw()],
    )
}

/// Stage 0 of 谏直: run the chosen number of judgments (raw top-pile reveals), split the
/// judge cards by this play phase's used suits, open the assignment loop, and remember
/// whether any non-matching judge card appeared.
pub(crate) fn resolve_jianzhi_count_choice(
    tx: &mut Transaction,
    observer: engine_api::PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&0);
    let player = engine_api::PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "谏直 must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [count] = selections else {
                return Err(EffectError::Illegal(
                    "谏直 requires one judgment count".into(),
                ));
            };
            let count = *count;
            if count == 0 || count > 16 {
                return Err(EffectError::Illegal(
                    "谏直 judgment count out of range".into(),
                ));
            }
            let mut judge_cards = Vec::new();
            for _ in 0..count {
                let draw = tx
                    .state()
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("谏直 draw pile missing".into()))?;
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
                    break;
                };
                tx.push_event(engine_api::DomainEvent::CardsRevealed {
                    cards: vec![card],
                    to: Vec::new(),
                });
                // Upstream `judge()` removes the judge card from the pile; the engine has
                // no expansion/limbo zone, so the card is held in the discard pile until the
                // assignment loop moves it to its recipient's hand.
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("谏直 discard pile missing".into()))?;
                crate::effect::primitives::pack_mobile::move_one_card(
                    tx,
                    card,
                    draw,
                    discard,
                    PlayerId::NONE,
                    true,
                )?;
                judge_cards.push(card);
            }
            if judge_cards.is_empty() {
                return Ok(());
            }
            let suit_mask = tx.state().turn.play_phase_card_suit_mask(player);
            let mut matching = Vec::new();
            let mut has_non_matching = false;
            for card in &judge_cards {
                let Some(card_state) = tx.state().cards.get(*card) else {
                    continue;
                };
                let bit = match card_state.suit {
                    engine_api::Suit::Spade => 1,
                    engine_api::Suit::Heart => 2,
                    engine_api::Suit::Club => 4,
                    engine_api::Suit::Diamond => 8,
                    engine_api::Suit::None => 0,
                };
                if bit != 0 && suit_mask & bit != 0 {
                    matching.push(card.raw());
                } else {
                    has_non_matching = true;
                }
            }
            if matching.is_empty() {
                if has_non_matching {
                    crate::effect::primitives::pack_mobile::deal_damage(
                        tx,
                        engine_api::PlayerId::NONE,
                        player,
                        1,
                        false,
                        2,
                        false,
                        engine_api::CardId::NONE,
                        0,
                        0,
                    )?;
                }
                return Ok(());
            }
            let mut next_payload = vec![0, player.raw(), skill.raw(), u32::from(has_non_matching)];
            next_payload.extend(matching.iter().copied());
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                matching,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::JianzhiAssignCard,
                next_payload,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::ContentFault("谏直 stage out of range".into())),
    }
}

/// 谏直 assignment loop: stage 0 selects one matching judge card, stage 1 a recipient;
/// the card is moved to the recipient's hand, the loop re-opens while cards remain, and
/// a non-matching judge card deals 1 thunder damage to the owner once the loop ends.
pub(crate) fn resolve_jianzhi_assign_card(
    tx: &mut Transaction,
    observer: engine_api::PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&0);
    let player = engine_api::PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    let has_non_matching = *payload.get(3).unwrap_or(&0) != 0;
    let remaining: Vec<u32> = payload.get(4..).unwrap_or(&[]).to_vec();
    if observer != player {
        return Err(EffectError::Illegal(
            "谏直 must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "谏直 requires exactly one judge card".into(),
                ));
            };
            if !remaining.contains(card) {
                return Err(EffectError::Illegal(
                    "谏直 card is no longer assignable".into(),
                ));
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(engine_api::PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Err(EffectError::Illegal("谏直 has no recipient".into()));
            }
            let mut next_payload = vec![
                1,
                player.raw(),
                skill.raw(),
                u32::from(has_non_matching),
                *card,
            ];
            next_payload.extend(
                remaining
                    .iter()
                    .copied()
                    .filter(|candidate| candidate != card),
            );
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets,
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::JianzhiAssignCard,
                next_payload,
            )
            .map(|_| ())
        }
        1 => {
            if cancelled {
                return Ok(());
            }
            let [target] = selections else {
                return Err(EffectError::Illegal(
                    "谏直 requires exactly one recipient".into(),
                ));
            };
            let target = engine_api::PlayerId::new(*target);
            let card = engine_api::CardId::new(*payload.get(4).unwrap_or(&0));
            if card == engine_api::CardId::NONE || !tx.state().cards.get(card).is_some() {
                return Err(EffectError::Illegal("谏直 judge card missing".into()));
            }
            let rest: Vec<u32> = payload.get(5..).unwrap_or(&[]).to_vec();
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|state| state.zone)
                .ok_or_else(|| EffectError::Illegal("谏直 card zone missing".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("谏直 recipient hand missing".into()))?;
            crate::effect::primitives::pack_mobile::move_one_card(
                tx, card, from, hand, target, true,
            )?;
            if !rest.is_empty() {
                let mut next_payload =
                    vec![0, player.raw(), skill.raw(), u32::from(has_non_matching)];
                next_payload.extend(rest.iter().copied());
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    rest,
                    engine_api::ChoiceKind::SelectCards,
                    crate::state::ContinuationKind::JianzhiAssignCard,
                    next_payload,
                )
                .map(|_| ())
            } else if has_non_matching {
                crate::effect::primitives::pack_mobile::deal_damage(
                    tx,
                    engine_api::PlayerId::NONE,
                    player,
                    1,
                    false,
                    2,
                    false,
                    engine_api::CardId::NONE,
                    0,
                    0,
                )
                .map(|_| ())
            } else {
                Ok(())
            }
        }
        _ => Err(EffectError::ContentFault("谏直 stage out of range".into())),
    }
}
