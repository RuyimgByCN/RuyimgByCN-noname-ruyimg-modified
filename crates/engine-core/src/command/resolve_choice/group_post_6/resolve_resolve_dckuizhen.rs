//! Command operations for `resolve_resolve_dckuizhen` and related helpers.

use super::*;

pub(crate) fn resolve_resolve_dckuizhen(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Dckuizhen continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Dckuizhen choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Dckuizhen context changed".into()));
    }
    let target = PlayerId::new(selections[0]);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Dckuizhen target stale".into()));
    }
    // Remember the initiator and let it use a Duel on the owner. The DuelResolved event
    // later drives the outcome (resolve_dckuizhen_resolve).
    run_effects(
        tx,
        vec![
            EffectRequest::AddMark {
                player,
                mark: engine_api::MarkDefId::new(crate::effect::DCKUIZHEN_PENDING_MARK),
                delta: target.raw() as i32,
            },
            EffectRequest::UseVirtualCardOn {
                player: target,
                kind: crate::state::CardKind::TrickJuedou.tag(),
                target: player,
            },
        ],
    )
    .map(|_| ())
}

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

pub(crate) fn resolve_resolve_dcmanzhi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, branch, stage, target_raw, count?]
    if payload.len() < 6 {
        return Err(EffectError::EngineFault("bad Dcmanzhi continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let branch = payload[3];
    let stage = payload[4];
    let target = PlayerId::new(payload[5]);
    let count = payload.get(6).copied().unwrap_or(0) as usize;
    // Stage (1,1) is the target's own give-card choice; every other stage is the owner's.
    let expected_observer = if branch == 1 && stage == 1 {
        target
    } else {
        player
    };
    if observer != expected_observer {
        return Err(EffectError::Illegal(
            "Dcmanzhi choice observer mismatch".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Dcmanzhi context changed".into()));
    }
    match (branch, stage) {
        (0, 0) => {
            // Branch selection.
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Dcmanzhi branch selection invalid".into(),
                ));
            }
            let branch = *selections.first().unwrap();
            if branch == 1 {
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
                        crate::effect::RESOLVE_DCMANZHI_CUSTOM_CHOICE,
                        player.raw(),
                        skill.raw(),
                        1,
                        0,
                        0,
                        0,
                    ],
                )
                .map(|_| ())
            } else if branch == 2 {
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
                                rule_api::TargetCardScope::HandEquipJudge,
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
                        crate::effect::RESOLVE_DCMANZHI_CUSTOM_CHOICE,
                        player.raw(),
                        skill.raw(),
                        2,
                        0,
                        0,
                        0,
                    ],
                )
                .map(|_| ())
            } else {
                Err(EffectError::Illegal("Dcmanzhi branch out of range".into()))
            }
        }
        (1, 0) => {
            // Give branch: target chosen; the target hands over up to two he cards.
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("Dcmanzhi give target invalid".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Dcmanzhi give target stale".into()));
            }
            let options: Vec<_> = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            )
            .into_iter()
            .map(CardId::raw)
            .collect();
            if options.is_empty() {
                return Ok(());
            }
            let num = options.len().min(2).min(u8::MAX as usize) as u8;
            crate::effect::open_choice(
                tx,
                target,
                num,
                num,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_DCMANZHI_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    1,
                    1,
                    target.raw(),
                    u32::from(num),
                ],
            )
            .map(|_| ())
        }
        (1, 1) => {
            // Give branch: move the chosen cards to the owner, then the target uses a Sha.
            if selections.is_empty() || selections.len() != count || cancelled {
                return Err(EffectError::Illegal("Dcmanzhi give cards invalid".into()));
            }
            let allowed = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let mut moves = Vec::new();
            for raw in selections {
                let card = CardId::new(*raw);
                if !allowed.contains(&card) {
                    return Err(EffectError::Illegal("Dcmanzhi give card invalid".into()));
                }
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("Dcmanzhi card missing".into()))?;
                moves.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, player)
                        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
                    from: Some(from),
                });
            }
            moves.push(EffectRequest::UseVirtualCardOn {
                player: target,
                kind: crate::state::CardKind::Sha.tag(),
                target: player,
            });
            moves.push(EffectRequest::AddMark {
                player,
                mark: engine_api::MarkDefId::new(crate::effect::DCMANZHI_GIVE_MARK),
                delta: 1,
            });
            run_effects(tx, moves).map(|_| ())
        }
        (2, 0) => {
            // Gain branch: target chosen; the owner picks up to two hej cards.
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("Dcmanzhi gain target invalid".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Dcmanzhi gain target stale".into()));
            }
            let options: Vec<_> = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquipJudge,
            )
            .into_iter()
            .map(CardId::raw)
            .collect();
            if options.is_empty() {
                return Ok(());
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                options.len().min(2).min(u8::MAX as usize) as u8,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_DCMANZHI_CUSTOM_CHOICE,
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
            // Gain branch: move the taken cards to the owner, then give back the same count.
            if selections.is_empty() || cancelled {
                return Err(EffectError::Illegal("Dcmanzhi gain cards invalid".into()));
            }
            let allowed = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquipJudge,
            );
            let taken: Vec<CardId> = selections.iter().map(|id| CardId::new(*id)).collect();
            let mut moves = Vec::new();
            for card in &taken {
                if !allowed.contains(card) {
                    return Err(EffectError::Illegal("Dcmanzhi gain card invalid".into()));
                }
                let from = tx
                    .state()
                    .cards
                    .get(*card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("Dcmanzhi card missing".into()))?;
                moves.push(EffectRequest::MoveCards {
                    cards: vec![*card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, player)
                        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
                    from: Some(from),
                });
            }
            run_effects(tx, moves)?;
            let num = taken.len();
            let hand_options: Vec<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            if num == 0 || hand_options.len() < num {
                return Ok(());
            }
            crate::effect::open_choice(
                tx,
                player,
                num.min(u8::MAX as usize) as u8,
                num.min(u8::MAX as usize) as u8,
                false,
                hand_options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_DCMANZHI_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    2,
                    2,
                    target.raw(),
                    num as u32,
                ],
            )
            .map(|_| ())
        }
        (2, 2) => {
            // Gain branch: give the chosen cards back and draw one.
            if selections.is_empty() || selections.len() != count || cancelled {
                return Err(EffectError::Illegal(
                    "Dcmanzhi give-back cards invalid".into(),
                ));
            }
            let allowed = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let mut moves = Vec::new();
            for raw in selections {
                let card = CardId::new(*raw);
                if !allowed.contains(&card) {
                    return Err(EffectError::Illegal(
                        "Dcmanzhi give-back card invalid".into(),
                    ));
                }
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("Dcmanzhi card missing".into()))?;
                moves.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, target)
                        .ok_or_else(|| EffectError::Illegal("target hand missing".into()))?,
                    from: Some(from),
                });
            }
            moves.push(EffectRequest::DrawCardsFromSkill {
                player,
                count: 1,
                skill,
            });
            moves.push(EffectRequest::AddMark {
                player,
                mark: engine_api::MarkDefId::new(crate::effect::DCMANZHI_GAIN_MARK),
                delta: 1,
            });
            run_effects(tx, moves).map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "Dcmanzhi continuation stage mismatch".into(),
        )),
    }
}

pub(crate) fn resolve_resolve_xinfu_qinguo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad XinfuQinguo continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "XinfuQinguo choice context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("XinfuQinguo context changed".into()));
    }
    let target = PlayerId::new(selections[0]);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("XinfuQinguo target invalid".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::UseVirtualCardOn {
            player,
            kind: crate::state::CardKind::Sha.tag(),
            target,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_gongjian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, count, index, shared..., accum_len, accum...]
    if payload.len() < 6 {
        return Err(EffectError::EngineFault("bad Gongjian continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let count = payload[3] as usize;
    let index = payload[4] as usize;
    let shared = &payload[5..5 + count];
    if payload.len() != 5 + count + 1 + payload[5 + count] as usize || observer != player {
        return Err(EffectError::Illegal(
            "Gongjian continuation payload mismatch".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Err(EffectError::Illegal(
            "Gongjian discard cannot be cancelled".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.current != player
    {
        return Err(EffectError::Illegal(
            "Gongjian choice context changed".into(),
        ));
    }
    let target = PlayerId::new(shared[index]);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Gongjian target is no longer alive".into(),
        ));
    }
    let allowed = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    );
    let accum_start = 5 + count;
    let accum_len = payload[accum_start] as usize;
    let mut accumulated: Vec<CardId> = payload[accum_start + 1..accum_start + 1 + accum_len]
        .iter()
        .map(|id| CardId::new(*id))
        .collect();
    let mut moves = Vec::new();
    for raw in selections {
        let card = CardId::new(*raw);
        if !allowed.contains(&card) || accumulated.contains(&card) {
            return Err(EffectError::Illegal(
                "Gongjian discard card invalid or duplicate".into(),
            ));
        }
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Gongjian card missing".into()))?;
        moves.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap(),
            from: Some(from),
        });
        accumulated.push(card);
    }
    run_effects(tx, moves)?;
    if index + 1 < count {
        let shared_ids: Vec<PlayerId> = shared.iter().map(|id| PlayerId::new(*id)).collect();
        crate::effect::open_gongjian_discard_choice(
            tx,
            player,
            engine_api::SkillId::new(payload[2]),
            &shared_ids,
            index + 1,
            accumulated.clone(),
        )
        .map(|_| ())
    } else {
        // Gain every discarded Sha.
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        let mut gains = Vec::new();
        for card in accumulated {
            if tx
                .state()
                .cards
                .get(card)
                .is_some_and(|card| card.kind.is_sha())
            {
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("Gongjian card missing".into()))?;
                gains.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, player)
                        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
                    from: Some(from),
                });
            }
        }
        let _ = discard;
        run_effects(tx, gains).map(|_| ())
    }
}

pub(crate) fn resolve_resolve_xinfu_lianpian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad XinfuLianpian continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let drawn = CardId::new(payload[3]);
    if observer != player {
        return Err(EffectError::Illegal(
            "XinfuLianpian choice context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx.state().hand_cards(player).contains(&drawn)
    {
        return Err(EffectError::Illegal(
            "XinfuLianpian give context changed".into(),
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
            "XinfuLianpian give target invalid".into(),
        ));
    }
    let from = tx
        .state()
        .cards
        .get(drawn)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("XinfuLianpian card missing".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("target hand missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![drawn],
            to,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_spfengyin(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad Spfengyin continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[3]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Spfengyin give choice context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Spfengyin give selection invalid".into(),
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
        || tx.state().turn.current != target
    {
        return Err(EffectError::Illegal("Spfengyin context changed".into()));
    }
    let card = CardId::new(selections[0]);
    if !tx.state().hand_cards(player).contains(&card)
        || !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card| card.kind.is_sha())
    {
        return Err(EffectError::Illegal(
            "Spfengyin card is not a Sha in the owner's hand".into(),
        ));
    }
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("Spfengyin card missing".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("target hand missing".into()))?;
    run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![card],
                to,
                from: Some(from),
            },
            EffectRequest::SkipNextPlayPhase { player: target },
            EffectRequest::SkipNextDiscardPhase { player: target },
        ],
    )
    .map(|_| ())
}

pub(crate) fn category_distribution_payload(
    player: PlayerId,
    skill: engine_api::SkillId,
    cap: u8,
    stage: u32,
    remaining: &[CardId],
    counts: &[(PlayerId, u32)],
    pending: &[CardId],
) -> Vec<u32> {
    let mut payload = vec![
        crate::effect::REVEAL_CATEGORY_DISTRIBUTION_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        u32::from(cap),
        stage,
        remaining.len() as u32,
    ];
    payload.extend(remaining.iter().map(|card| card.raw()));
    payload.push(counts.len() as u32);
    for (target, count) in counts {
        payload.extend([target.raw(), *count]);
    }
    payload.push(pending.len() as u32);
    payload.extend(pending.iter().map(|card| card.raw()));
    payload
}

pub(crate) fn parse_category_distribution_payload(
    payload: &[u32],
) -> Result<
    (
        PlayerId,
        engine_api::SkillId,
        u8,
        u32,
        Vec<CardId>,
        Vec<(PlayerId, u32)>,
        Vec<CardId>,
    ),
    EffectError,
> {
    if payload.len() < 8
        || payload.first() != Some(&crate::effect::REVEAL_CATEGORY_DISTRIBUTION_CUSTOM_CHOICE)
    {
        return Err(EffectError::EngineFault(
            "bad category distribution continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let cap = u8::try_from(payload[3])
        .map_err(|_| EffectError::EngineFault("category distribution cap overflow".into()))?;
    let stage = payload[4];
    let remaining_len = payload[5] as usize;
    let mut cursor = 6usize;
    if payload.len() < cursor + remaining_len + 2 {
        return Err(EffectError::EngineFault(
            "truncated category distribution cards".into(),
        ));
    }
    let remaining = payload[cursor..cursor + remaining_len]
        .iter()
        .map(|id| CardId::new(*id))
        .collect();
    cursor += remaining_len;
    let count_len = payload[cursor] as usize;
    cursor += 1;
    if payload.len() < cursor + count_len * 2 + 1 {
        return Err(EffectError::EngineFault(
            "truncated category distribution counts".into(),
        ));
    }
    let mut counts = Vec::with_capacity(count_len);
    for pair in payload[cursor..cursor + count_len * 2].chunks_exact(2) {
        counts.push((PlayerId::new(pair[0]), pair[1]));
    }
    cursor += count_len * 2;
    let pending_len = payload[cursor] as usize;
    cursor += 1;
    if payload.len() != cursor + pending_len {
        return Err(EffectError::EngineFault(
            "category distribution payload length mismatch".into(),
        ));
    }
    let pending = payload[cursor..]
        .iter()
        .map(|id| CardId::new(*id))
        .collect();
    Ok((player, skill, cap, stage, remaining, counts, pending))
}

pub(crate) fn open_category_distribution_card_choice(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    cap: u8,
    remaining: &[CardId],
    counts: &[(PlayerId, u32)],
) -> Result<(), EffectError> {
    let max_free = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|candidate| {
            let used = counts
                .iter()
                .find(|(target, _)| *target == candidate)
                .map_or(0, |(_, count)| *count);
            u32::from(cap).saturating_sub(used)
        })
        .max()
        .unwrap_or(0) as u8;
    let max = max_free.min(remaining.len().min(u8::MAX as usize) as u8);
    if max == 0 {
        return Err(EffectError::Illegal(
            "category distribution has no recipient capacity".into(),
        ));
    }
    crate::effect::open_choice(
        tx,
        player,
        1,
        max,
        false,
        remaining.iter().map(|card| card.raw()).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        category_distribution_payload(player, skill, cap, 1, remaining, counts, &[]),
    )
    .map(|_| ())
}

pub(crate) fn resolve_random_gain_region_compare_color(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [marker, player_raw, skill_raw] = payload else {
        return Err(EffectError::EngineFault(
            "bad random region gain continuation".into(),
        ));
    };
    if *marker != crate::effect::RANDOM_GAIN_REGION_COMPARE_COLOR_CUSTOM_CHOICE
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal(
            "random region gain requires one target".into(),
        ));
    }
    let player = PlayerId::new(*player_raw);
    let target = PlayerId::new(selections[0]);
    let skill = engine_api::SkillId::new(*skill_raw);
    if observer != player
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "random region gain context changed".into(),
        ));
    }
    let cards: Vec<CardId> = [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
        .into_iter()
        .flat_map(|kind| {
            tx.state()
                .zones
                .find(kind, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .collect();
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "random region gain target has no card".into(),
        ));
    }
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let gained = cards[rng.gen_range_u32(cards.len() as u32) as usize];
    tx.state_mut().rng = rng.into_state();
    let from = tx
        .state()
        .cards
        .get(gained)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("random region card missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("skill owner hand missing".into()))?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    if tx
        .state()
        .zones
        .get(draw)
        .is_none_or(|zone| zone.cards.is_empty())
    {
        crate::effect::reshuffle_discard_into_draw(tx)?;
    }
    let drawn = tx
        .state()
        .zones
        .get(draw)
        .and_then(|zone| zone.cards.last().copied());
    let different_color = drawn.is_some_and(|drawn| {
        let gained_suit = tx.state().cards.get(gained).map(|card| card.suit);
        let drawn_suit = tx.state().cards.get(drawn).map(|card| card.suit);
        gained_suit.zip(drawn_suit).is_some_and(|(gained, drawn)| {
            crate::query::card_color_mask_for_suit(gained)
                != crate::query::card_color_mask_for_suit(drawn)
        })
    });
    let mut effects = vec![
        EffectRequest::MoveCards {
            cards: vec![gained],
            to: hand,
            from: Some(from),
        },
        EffectRequest::DrawCards { player, count: 1 },
    ];
    if different_color {
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    run_effects(tx, effects).map(|_| ())
}
