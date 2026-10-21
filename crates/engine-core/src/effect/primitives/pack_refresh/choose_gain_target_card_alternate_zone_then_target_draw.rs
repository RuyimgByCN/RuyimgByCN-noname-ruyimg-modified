//! Effect primitives for `choose_gain_target_card_alternate_zone_then_target_draw` and related helpers.

use super::*;

pub(crate) fn choose_gain_target_card_alternate_zone_then_target_draw(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    selected: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    const HAND_USED: u8 = 1;
    const EQUIP_USED: u8 = 2;

    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid alternate-zone target gain context".into(),
        ));
    }
    let used = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.play_phase_category_mask)
        .ok_or_else(|| EffectError::Illegal("alternate-zone skill is missing".into()))?;
    let hand = tx.state().zones.find(ZoneKind::Hand, target);
    let equip = tx.state().zones.find(ZoneKind::Equip, target);
    let mut options = Vec::new();
    if used & HAND_USED == 0 {
        options.extend(
            hand.and_then(|zone| tx.state().zones.get(zone))
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied()),
        );
    }
    if used & EQUIP_USED == 0 {
        options.extend(
            equip
                .and_then(|zone| tx.state().zones.get(zone))
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied()),
        );
    }
    if selected.is_none() {
        if options.is_empty() {
            return Err(EffectError::Illegal(
                "target has no card in an unused zone".into(),
            ));
        }
        let mut payload = vec![
            player.raw(),
            target.raw(),
            skill.raw(),
            options.len() as u32,
        ];
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|card| card.raw()).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseGainTargetCardAlternateZoneThenTargetDraw,
            payload,
        );
    }
    if !options.contains(&selected) {
        return Err(EffectError::Illegal(
            "selected target card is stale or from an already used zone".into(),
        ));
    }
    let card = tx
        .state()
        .cards
        .get(selected)
        .ok_or_else(|| EffectError::Illegal("selected target card is missing".into()))?;
    let (from, used_bit) = if Some(card.zone) == hand {
        (card.zone, HAND_USED)
    } else if Some(card.zone) == equip {
        (card.zone, EQUIP_USED)
    } else {
        return Err(EffectError::Illegal(
            "selected card left the target hand/equipment".into(),
        ));
    };
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated alternate-zone skill")
        .play_phase_category_mask |= used_bit;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::DrawCards {
            player: target,
            count: 1,
        });
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("skill owner hand missing".into()))?;
    move_cards(tx, vec![selected], to, Some(from))
}

/// 神君 (clanshenjun) reveal: reveal and gaintag every hand card sharing the
/// event card's effective kind (杀 or 锦囊). The tag is the skill's definition
/// id, so only 神君's own phase-end use consumes these cards.
pub(crate) fn clanshenjun_reveal(
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
    let Some(event) = tx.state().cards.get(event_card) else {
        return Ok(Vec::new());
    };
    let kind = crate::query::effective_card_kind(tx.state(), player, event);
    if !(kind.is_sha() || kind.is_trick()) {
        return Ok(Vec::new());
    }
    let matching: Vec<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|candidate| {
                crate::query::effective_card_kind(tx.state(), player, candidate) == kind
            })
        })
        .collect();
    if matching.is_empty() {
        return Ok(Vec::new());
    }
    let tag = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .unwrap_or(0);
    for card in &matching {
        tx.state_mut().add_content_card_tag(*card, tag);
    }
    Ok(vec![engine_api::DomainEvent::CardsRevealed {
        cards: matching,
        to: Vec::new(),
    }])
}

/// 神君 (clanshenjun) phase-end use: open a choice over the gaintagged hand
/// cards (stage 0), then a target choice (stage 1), then play the selected card
/// as its own kind through the normal card pipeline.
///
/// NOTE: the choice continuation needs a `ContinuationKind` discriminant, but that enum is
/// `repr(u8)` and already full (0..=255). The phase-end use path is therefore stubbed until
/// the enum is widened to `repr(u16)`. The clanshenjun DSL remains a no-op, so this is never
/// reached in shipped content.
pub(crate) fn clanshenjun_use(
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
    let Some(tag) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
    else {
        return Ok(Vec::new());
    };
    let tagged: Vec<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| tx.state().card_has_content_tag(*card, tag))
        .collect();
    if tagged.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        tagged.into_iter().map(engine_api::CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::ClanshenjunUseCard,
        vec![0, player.raw(), skill.raw()],
    )
}

/// Choice continuation for 神君's phase-end use.
pub(crate) fn resolve_clanshenjun_use_choice(
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
            "神君 use must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "神君 requires exactly one tagged card".into(),
                ));
            };
            let card = engine_api::CardId::new(*card);
            let tag = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.def.raw())
                .unwrap_or(0);
            if !tx.state().hand_cards(player).contains(&card)
                || !tx.state().card_has_content_tag(card, tag)
            {
                return Err(EffectError::Illegal(
                    "神君 card is no longer tagged or in hand".into(),
                ));
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .collect();
            if targets.is_empty() {
                return Err(EffectError::Illegal("神君 has no target".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets.into_iter().map(engine_api::PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::ClanshenjunUseCard,
                vec![1, player.raw(), skill.raw(), card.raw()],
            )
            .map(|_| ())
        }
        1 => {
            if cancelled {
                return Ok(());
            }
            let [target] = selections else {
                return Err(EffectError::Illegal(
                    "神君 requires exactly one target".into(),
                ));
            };
            let card = engine_api::CardId::new(*payload.get(3).unwrap_or(&0));
            let target = engine_api::PlayerId::new(*target);
            let Some(card_state) = tx.state().cards.get(card) else {
                return Err(EffectError::Illegal("神君 card missing".into()));
            };
            let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
            crate::command::play::play_as_kind(
                tx,
                player,
                card,
                kind,
                &[target],
                false,
                false,
                false,
                true,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::ContentFault("神君 stage out of range".into())),
    }
}

/// 迂志 (clanyuzhi) round-start: open a choice over the owner's hand cards; the
/// continuation reveals + gaintags the chosen card, draws its name length in cards
/// (tagging the draws), and records the round draw counts in keyed storage.
pub(crate) fn clanyuzhi_reveal(
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
    let options: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            crate::query::card_name_length(tx.state(), player, *card).is_some_and(|len| len > 0)
        })
        .map(engine_api::CardId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::ClanyuzhiRevealChoice,
        vec![0, player.raw(), skill.raw()],
    )
}

/// Stage 0 of the 迂志 round-start: reveal + gaintag the chosen hand card, draw its
/// name length (gaintagging every drawn card), rotate the round draw counters.
pub(crate) fn resolve_clanyuzhi_reveal_choice(
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
            "迂志 must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "迂志 requires exactly one hand card".into(),
                ));
            };
            let card = engine_api::CardId::new(*card);
            if !tx.state().hand_cards(player).contains(&card) {
                return Err(EffectError::Illegal(
                    "迂志 card is no longer in hand".into(),
                ));
            }
            let Some(tag) = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.def.raw())
            else {
                return Ok(());
            };
            tx.state_mut().add_content_card_tag(card, tag);
            let name_len =
                crate::query::card_name_length(tx.state(), player, card).unwrap_or(1) as u32;
            let draw_events =
                crate::effect::primitives::pack_sp_2::draw_cards(tx, player, name_len)?;
            let mut drawn = Vec::new();
            for event in &draw_events {
                if let engine_api::DomainEvent::CardsDrawn {
                    player: drawer,
                    cards,
                } = event
                {
                    if *drawer == player {
                        drawn.extend(cards.iter().copied());
                    }
                }
            }
            for drawn_card in &drawn {
                tx.state_mut().add_content_card_tag(*drawn_card, tag);
            }
            let previous = tx.state().skills.storage_get(skill, YUZHI_KEY_DRAWN_ROUND);
            tx.state_mut()
                .skills
                .storage_set(skill, YUZHI_KEY_DRAWN_LAST, previous);
            tx.state_mut()
                .skills
                .storage_set(skill, YUZHI_KEY_DRAWN_ROUND, name_len);
            tx.state_mut()
                .skills
                .storage_set(skill, YUZHI_KEY_MARK, name_len);
            tx.push_event(engine_api::DomainEvent::CardsRevealed {
                cards: vec![card],
                to: Vec::new(),
            });
            Ok(())
        }
        _ => Err(EffectError::ContentFault("迂志 stage out of range".into())),
    }
}

/// 迂志 (clanyuzhi) round-end: discard every gaintagged hand card, then if this
/// round's 迂志 draws exceeded the previous round's (both positive) or the round's
/// used-card count, the owner loses 保族 or takes 1 thunder damage.
pub(crate) fn clanyuzhi_round_end(
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
    let Some(tag) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
    else {
        return Ok(Vec::new());
    };
    let mut events = Vec::new();
    let tagged: Vec<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| tx.state().card_has_content_tag(*card, tag))
        .collect();
    if !tagged.is_empty() {
        events.extend(crate::effect::primitives::pack_sp_1::discard_target_cards(
            tx,
            player,
            player,
            tagged,
            rule_api::TargetCardScope::Hand,
        )?);
    }
    let num1 = tx.state().skills.storage_get(skill, YUZHI_KEY_DRAWN_ROUND);
    let num2 = tx.state().skills.storage_get(skill, YUZHI_KEY_DRAWN_LAST);
    let num3 = tx.state().turn.round_card_use_count(player);
    let ambitious = num1 > 0 && num2 > 0 && num1 > num2;
    let lagging = num1 > num3;
    if !(ambitious || lagging) {
        return Ok(events);
    }
    if !crate::effect::primitives::pack_old::player_has_skill_def(
        tx,
        player,
        engine_api::SkillDefId::new(YUZHI_CLAN_BAOZU),
        false,
    ) {
        events.extend(crate::effect::primitives::pack_mobile::deal_damage(
            tx,
            player,
            player,
            1,
            false,
            2,
            false,
            engine_api::CardId::NONE,
            0,
            0,
        )?);
        return Ok(events);
    }
    events.extend(open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![0, 1],
        engine_api::ChoiceKind::Confirm,
        crate::state::ContinuationKind::ClanyuzhiEndChoice,
        vec![0, player.raw(), skill.raw()],
    )?);
    Ok(events)
}

/// Stage 0 of the 迂志 round-end choice: selection 0 loses 保族, selection 1 takes
/// 1 thunder damage.
pub(crate) fn resolve_clanyuzhi_end_choice(
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
            "迂志 round-end must be chosen by its owner".into(),
        ));
    }
    match stage {
        0 => {
            if cancelled {
                return Ok(());
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal(
                    "迂志 round-end requires one selection".into(),
                ));
            };
            match *choice {
                0 => crate::effect::primitives::pack_tw_1::remove_skill_by_def(
                    tx,
                    player,
                    engine_api::SkillDefId::new(YUZHI_CLAN_BAOZU),
                )
                .map(|_| ()),
                1 => crate::effect::primitives::pack_mobile::deal_damage(
                    tx,
                    player,
                    player,
                    1,
                    false,
                    2,
                    false,
                    engine_api::CardId::NONE,
                    0,
                    0,
                )
                .map(|_| ()),
                _ => Err(EffectError::Illegal(
                    "迂志 round-end selection out of range".into(),
                )),
            }
        }
        _ => Err(EffectError::ContentFault("迂志 stage out of range".into())),
    }
}

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
