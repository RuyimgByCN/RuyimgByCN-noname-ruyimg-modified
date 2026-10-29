//! Command operations for `resolve_resolve_dckuizhen` and related helpers.

use super::*;

pub(crate) fn resolve_resolve_xinxingluan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, branch, stage, target_raw, count]
    if payload.len() < 6 {
        return Err(EffectError::EngineFault(
            "bad Xinxingluan continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let branch = payload[3];
    let stage = payload[4];
    let target = PlayerId::new(payload[5]);
    let count = payload.get(6).copied().unwrap_or(0) as usize;
    // Branch 3 stages 1-2 are the target's own discard/give choices.
    let expected_observer = if branch == 3 && (stage == 1 || stage == 2) {
        target
    } else {
        player
    };
    if observer != expected_observer {
        return Err(EffectError::Illegal(
            "Xinxingluan choice observer mismatch".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Xinxingluan context changed".into()));
    }
    let rank_six = |tx: &Transaction, card: CardId| -> bool {
        tx.state()
            .cards
            .get(card)
            .is_some_and(|card| card.rank == 6)
    };
    match (branch, stage) {
        (0, 0) => {
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinxingluan branch selection invalid".into(),
                ));
            }
            let branch = *selections.first().unwrap();
            match branch {
                1 => {
                    // Scan the draw pile for up to two rank-6 cards.
                    let draw = tx
                        .state()
                        .zones
                        .find(ZoneKind::DrawPile, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
                    let pile: Vec<CardId> = tx
                        .state()
                        .zones
                        .get(draw)
                        .map(|zone| zone.cards.clone())
                        .unwrap_or_default();
                    let mut cards: Vec<CardId> = pile
                        .into_iter()
                        .filter(|card| rank_six(tx, *card))
                        .collect();
                    cards.truncate(2);
                    if cards.is_empty() {
                        // Fallback: draw six.
                        return run_effects(
                            tx,
                            vec![EffectRequest::DrawCardsFromSkill {
                                player,
                                count: 6,
                                skill,
                            }],
                        )
                        .map(|_| ());
                    }
                    crate::effect::open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        cards.iter().map(|card| card.raw()).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        vec![
                            crate::effect::RESOLVE_XINXINGLUAN_CUSTOM_CHOICE,
                            player.raw(),
                            skill.raw(),
                            1,
                            1,
                            0,
                            cards.len() as u32,
                        ],
                    )
                    .map(|_| ())
                }
                2 => {
                    let options: Vec<_> = tx
                        .state()
                        .players
                        .alive_by_seat()
                        .into_iter()
                        .filter(|&candidate| {
                            candidate != player
                                && !crate::query::discardable_target_cards(
                                    tx.state(),
                                    player,
                                    candidate,
                                    rule_api::TargetCardScope::HandEquip,
                                )
                                .is_empty()
                        })
                        .map(PlayerId::raw)
                        .collect();
                    if options.is_empty() {
                        return Ok(());
                    }
                    crate::effect::open_choice(
                        tx,
                        player,
                        1,
                        1,
                        true,
                        options,
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        vec![
                            crate::effect::RESOLVE_XINXINGLUAN_CUSTOM_CHOICE,
                            player.raw(),
                            skill.raw(),
                            2,
                            0,
                            0,
                            0,
                        ],
                    )
                    .map(|_| ())
                }
                3 => {
                    let options: Vec<_> = tx
                        .state()
                        .players
                        .alive_by_seat()
                        .into_iter()
                        .filter(|&candidate| {
                            candidate != player
                                && !crate::query::discardable_target_cards(
                                    tx.state(),
                                    player,
                                    candidate,
                                    rule_api::TargetCardScope::HandEquip,
                                )
                                .is_empty()
                        })
                        .map(PlayerId::raw)
                        .collect();
                    if options.is_empty() {
                        return Ok(());
                    }
                    crate::effect::open_choice(
                        tx,
                        player,
                        1,
                        1,
                        true,
                        options,
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        vec![
                            crate::effect::RESOLVE_XINXINGLUAN_CUSTOM_CHOICE,
                            player.raw(),
                            skill.raw(),
                            3,
                            0,
                            0,
                            0,
                        ],
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal(
                    "Xinxingluan branch out of range".into(),
                )),
            }
        }
        (1, 1) => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinxingluan pile card selection invalid".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !rank_six(tx, card) {
                return Err(EffectError::Illegal(
                    "Xinxingluan chosen pile card is not rank 6".into(),
                ));
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Xinxingluan card missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, player)
                        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
                    from: Some(from),
                }],
            )
            .map(|_| ())
        }
        (2, 0) => {
            // Gain one rank-6 card from the target's equip/judge zones.
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinxingluan gain target invalid".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Xinxingluan gain target stale".into()));
            }
            let mut options = Vec::new();
            for zone in [ZoneKind::Equip, ZoneKind::Judge] {
                if let Some(zone_id) = tx.state().zones.find(zone, target) {
                    if let Some(zone_state) = tx.state().zones.get(zone_id) {
                        options.extend(
                            zone_state
                                .cards
                                .iter()
                                .filter(|card| rank_six(tx, **card))
                                .map(|card| card.raw()),
                        );
                    }
                }
            }
            if options.is_empty() {
                return Ok(());
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_XINXINGLUAN_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    2,
                    1,
                    target.raw(),
                    0,
                ],
            )
            .map(|_| ())
        }
        (2, 1) => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinxingluan gained card invalid".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            let valid = [ZoneKind::Equip, ZoneKind::Judge].iter().any(|zone| {
                tx.state()
                    .zones
                    .find(*zone, target)
                    .and_then(|id| tx.state().zones.get(id))
                    .is_some_and(|zone_state| zone_state.cards.contains(&card))
            });
            if !valid || !rank_six(tx, card) {
                return Err(EffectError::Illegal(
                    "Xinxingluan gained card is stale".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Xinxingluan card missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, player)
                        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
                    from: Some(from),
                }],
            )
            .map(|_| ())
        }
        (3, 0) => {
            // The target discards a rank-6 hand card or gives one card.
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinxingluan discard target invalid".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Xinxingluan discard target stale".into(),
                ));
            }
            let hand = tx.state().hand_cards(target);
            let six_cards: Vec<u32> = hand
                .iter()
                .filter(|card| rank_six(tx, **card))
                .map(|card| card.raw())
                .collect();
            if six_cards.is_empty() {
                // No rank-6 card: the target must give one card.
                if hand.is_empty() {
                    return Ok(());
                }
                return crate::effect::open_choice(
                    tx,
                    target,
                    1,
                    1,
                    false,
                    hand.into_iter().map(|card| card.raw()).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![
                        crate::effect::RESOLVE_XINXINGLUAN_CUSTOM_CHOICE,
                        player.raw(),
                        skill.raw(),
                        3,
                        2,
                        target.raw(),
                        0,
                    ],
                )
                .map(|_| ());
            }
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                true,
                six_cards,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_XINXINGLUAN_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    3,
                    1,
                    target.raw(),
                    0,
                ],
            )
            .map(|_| ())
        }
        (3, 1) => {
            // Target discarded a rank-6 card (cancelled -> give one).
            if cancelled {
                let hand = tx.state().hand_cards(target);
                if hand.is_empty() {
                    return Ok(());
                }
                return crate::effect::open_choice(
                    tx,
                    target,
                    1,
                    1,
                    false,
                    hand.into_iter().map(|card| card.raw()).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![
                        crate::effect::RESOLVE_XINXINGLUAN_CUSTOM_CHOICE,
                        player.raw(),
                        skill.raw(),
                        3,
                        2,
                        target.raw(),
                        0,
                    ],
                )
                .map(|_| ());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinxingluan discard selection invalid".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !tx.state().hand_cards(target).contains(&card) || !rank_six(tx, card) {
                return Err(EffectError::Illegal(
                    "Xinxingluan discarded card is stale".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Xinxingluan card missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::DiscardPile, PlayerId::NONE)
                        .unwrap(),
                    from: Some(from),
                }],
            )
            .map(|_| ())
        }
        (3, 2) => {
            // Target gives one card to the owner.
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinxingluan give selection invalid".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal(
                    "Xinxingluan given card is stale".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Xinxingluan card missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, player)
                        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
                    from: Some(from),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "Xinxingluan continuation stage mismatch".into(),
        )),
    }
}
