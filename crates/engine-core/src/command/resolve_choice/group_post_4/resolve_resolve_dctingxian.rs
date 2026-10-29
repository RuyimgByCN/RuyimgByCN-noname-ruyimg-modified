//! Command operations for `resolve_resolve_dctingxian` and related helpers.

use super::*;

pub(crate) fn resolve_resolve_dctingxian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad Dctingxian continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let card = engine_api::CardId::new(payload[2]);
    if observer != player
        || cancelled
        || selections.is_empty()
        || card.is_none()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Dctingxian exclusion context changed".into(),
        ));
    }
    for raw in selections {
        let target = PlayerId::new(*raw);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "Dctingxian target is invalid or stale".into(),
            ));
        }
        if !tx
            .state_mut()
            .resolution
            .exclude_saved_sha_target(card, target)
        {
            return Err(EffectError::Illegal(
                "Dctingxian Sha exclusion unavailable".into(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn resolve_resolve_dcyingtu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, target, stage]
    if payload.len() != 5 {
        return Err(EffectError::EngineFault("bad Dcyingtu continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Dcyingtu observer mismatch".into()));
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
        return Err(EffectError::Illegal("Dcyingtu context changed".into()));
    }
    let target = PlayerId::new(payload[3]);
    let stage = payload[4];
    let card = CardId::new(selections[0]);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    if stage == 1 {
        // Gain the chosen card from the target.
        let allowed = crate::query::gainable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        );
        if !allowed.contains(&card) {
            return Err(EffectError::Illegal("Dcyingtu gained card invalid".into()));
        }
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Dcyingtu card missing".into()))?;
        run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: hand,
                from: Some(from),
            }],
        )?;
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDcyingtu {
                player,
                target,
                stage: 2,
                cards: Vec::new(),
                skill: engine_api::SkillId::new(payload[2]),
            }],
        )
        .map(|_| ());
    }
    // Stage 2: give the chosen card to the other neighbor.
    let neighbors = crate::query::alive_seat_neighbors(tx.state(), player);
    let other = neighbors
        .into_iter()
        .find(|candidate| *candidate != target)
        .ok_or_else(|| EffectError::Illegal("Dcyingtu other neighbor missing".into()))?;
    let allowed = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if !allowed.contains(&card) {
        return Err(EffectError::Illegal("Dcyingtu given card invalid".into()));
    }
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("Dcyingtu card missing".into()))?;
    let mut effects = vec![EffectRequest::MoveCards {
        cards: vec![card],
        to: tx
            .state()
            .zones
            .find(ZoneKind::Hand, other)
            .ok_or_else(|| EffectError::Illegal("other hand missing".into()))?,
        from: Some(from),
    }];
    // If the given card is an equipment, the recipient uses it.
    if tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card| card.kind.is_equip())
    {
        effects.push(EffectRequest::UseCard {
            player: other,
            card,
            targets: vec![other],
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_resolve_dcxiangshu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, target] for the number choice;
    //          [magic, player, skill, target, 1, delta] for the gain choice.
    if payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "bad Dcxiangshu continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Dcxiangshu observer mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    if payload.len() == 4 {
        // The number choice.
        if selections.len() != 1 || *selections.first().unwrap() > 5 {
            return Err(EffectError::Illegal(
                "Dcxiangshu number selection invalid".into(),
            ));
        }
        let target = PlayerId::new(payload[3]);
        if !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("Dcxiangshu context changed".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::AddMark {
                player,
                mark: engine_api::MarkDefId::new(crate::effect::DCXIANG_SHU_NUMBER_MARK),
                delta: *selections.first().unwrap() as i32,
            }],
        )
        .map(|_| ());
    }
    // The gain choice (payload len 6: ... target, 1, delta).
    if payload.len() != 6 || payload[4] != 1 {
        return Err(EffectError::EngineFault(
            "bad Dcxiangshu gain payload".into(),
        ));
    }
    let target = PlayerId::new(payload[3]);
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Dcxiangshu gain selection invalid".into(),
        ));
    }
    let card = CardId::new(selections[0]);
    let allowed = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    );
    if !allowed.contains(&card) {
        return Err(EffectError::Illegal(
            "Dcxiangshu gained card invalid".into(),
        ));
    }
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("Dcxiangshu card missing".into()))?;
    let mut effects = vec![EffectRequest::MoveCards {
        cards: vec![card],
        to: tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
        from: Some(from),
    }];
    // delta == 0 also deals 1 damage.
    if payload[5] == 0 {
        effects.push(EffectRequest::DealDamage {
            source: player,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_resolve_zhijian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, card, skill]
    if payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Zhijian continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Zhijian observer mismatch".into()));
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
        return Err(EffectError::Illegal("Zhijian context changed".into()));
    }
    let card = CardId::new(payload[2]);
    let target = PlayerId::new(selections[0]);
    let skill = engine_api::SkillId::new(payload[3]);
    run_effects(
        tx,
        vec![EffectRequest::ResolveZhijian {
            player,
            card,
            stage: 1,
            target,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_guzheng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, event_player, stage(, return_card)]
    if payload.len() < 5 {
        return Err(EffectError::EngineFault("bad Guzheng continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Guzheng observer mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let skill = engine_api::SkillId::new(payload[2]);
    let event_player = PlayerId::new(payload[3]);
    let stage = payload[4];
    if stage == 1 {
        if selections.len() != 1
            || !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            || !tx
                .state()
                .players
                .get(event_player)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("Guzheng context changed".into()));
        }
        let return_card = CardId::new(selections[0]);
        let allowed: Vec<_> =
            crate::effect::discard_phase_discards_for(tx.state(), event_player).collect();
        if !allowed.contains(&return_card) {
            return Err(EffectError::Illegal("Guzheng returned card invalid".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveGuzheng {
                player,
                event_player,
                stage: 1,
                cards: Vec::new(),
                return_card,
                take_rest: false,
                skill,
            }],
        )
        .map(|_| ());
    }
    if stage == 2 {
        if payload.len() != 6
            || selections.len() != 1
            || !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "Guzheng take-rest selection invalid".into(),
            ));
        }
        let return_card = CardId::new(payload[5]);
        let take_rest = selections[0] == 1;
        return run_effects(
            tx,
            vec![EffectRequest::ResolveGuzheng {
                player,
                event_player,
                stage: 2,
                cards: Vec::new(),
                return_card,
                take_rest,
                skill,
            }],
        )
        .map(|_| ());
    }
    Err(EffectError::EngineFault(
        "bad Guzheng continuation stage".into(),
    ))
}

pub(crate) fn resolve_resolve_drlt_weidi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, stage(, card_count, cards...)]
    if payload.len() < 4 {
        return Err(EffectError::EngineFault("bad Weidi continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Weidi observer mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let skill = engine_api::SkillId::new(payload[2]);
    let stage = payload[3];
    match stage {
        0 => {
            if selections.is_empty()
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Weidi context changed".into()));
            }
            let mut next = vec![
                RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                1,
                selections.len() as u32,
            ];
            next.extend(selections.iter().copied());
            crate::effect::open_choice(
                tx,
                player,
                selections.len() as u8,
                selections.len() as u8,
                true,
                tx.state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|candidate| {
                        *candidate != player && tx.state().player_group(*candidate) == Some("qun")
                    })
                    .map(|candidate| candidate.raw())
                    .collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        1 => {
            let card_count = *payload.get(4).unwrap_or(&0) as usize;
            if payload.len() < 5 + card_count || selections.len() != card_count || card_count == 0 {
                return Err(EffectError::Illegal(
                    "Weidi target selection invalid".into(),
                ));
            }
            let cards: Vec<_> = payload[5..5 + card_count]
                .iter()
                .map(|raw| CardId::new(*raw))
                .collect();
            let targets: Vec<_> = selections.iter().map(|raw| PlayerId::new(*raw)).collect();
            run_effects(
                tx,
                vec![EffectRequest::ResolveDrltWeidi {
                    player,
                    cards,
                    targets,
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad Weidi continuation stage".into(),
        )),
    }
}

pub(crate) fn resolve_resolve_nzry_feijun(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, stage(, branch|branch+target)]
    if payload.len() < 4 {
        return Err(EffectError::EngineFault("bad Feijun continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let stage = payload[3];
    // Stage 2 is the target's own card choice; other stages are the owner's.
    let expected_observer = if stage == 2 {
        PlayerId::new(*payload.get(5).unwrap_or(&0))
    } else {
        player
    };
    if observer != expected_observer {
        return Err(EffectError::Illegal("Feijun observer mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let skill = engine_api::SkillId::new(payload[2]);
    match stage {
        0 => {
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Feijun branch selection invalid".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveNzryFeijun {
                    player,
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    branch: selections[0] as u8,
                    stage: 1,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if selections.len() != 1
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Feijun context changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveNzryFeijun {
                    player,
                    target: PlayerId::new(selections[0]),
                    card: engine_api::CardId::NONE,
                    branch: payload[4] as u8,
                    stage: 2,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            // The target's card choice: observer is the target.
            if selections.len() != 1 {
                return Err(EffectError::Illegal("Feijun card selection invalid".into()));
            }
            let target = PlayerId::new(payload[5]);
            run_effects(
                tx,
                vec![EffectRequest::ResolveNzryFeijun {
                    player,
                    target,
                    card: CardId::new(selections[0]),
                    branch: payload[4] as u8,
                    stage: 3,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad Feijun continuation stage".into(),
        )),
    }
}
