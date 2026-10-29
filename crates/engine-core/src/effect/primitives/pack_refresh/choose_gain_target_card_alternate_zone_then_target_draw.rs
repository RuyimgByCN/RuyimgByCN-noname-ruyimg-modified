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
