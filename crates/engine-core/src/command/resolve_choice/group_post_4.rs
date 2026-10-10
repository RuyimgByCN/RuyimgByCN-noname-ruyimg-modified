//! Choice-continuation handlers (post 4).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

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

pub(crate) fn resolve_resolve_zhiba(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, actor, skill, stage(, winner, player_card, target_card)]
    if payload.len() < 5 {
        return Err(EffectError::EngineFault("bad Zhiba continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Zhiba observer mismatch".into()));
    }
    let actor = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let stage = payload[4];
    if cancelled || selections.len() != 1 {
        return Ok(());
    }
    if stage == 0 {
        // 觉醒拒绝选择：1 = 拒绝（停止），0 = 继续拼点。
        if selections[0] == 1 {
            return Ok(());
        }
        tx.state_mut().skills.get_mut(skill).map(|instance| {
            instance.set_zhiba_compare(actor);
        });
        let options: Vec<_> = tx
            .state()
            .hand_cards(actor)
            .into_iter()
            .map(CardId::raw)
            .collect();
        return crate::effect::open_choice(
            tx,
            actor,
            1,
            1,
            false,
            options,
            engine_api::ChoiceKind::SelectCards,
            crate::state::ContinuationKind::ChooseHandComparePlayerCard,
            vec![
                actor.raw(),
                player.raw(),
                engine_api::SkillDefId::NONE.raw(),
                engine_api::SkillDefId::NONE.raw(),
                skill.raw(),
            ],
        )
        .map(|_| ());
    }
    // stage 1: 取牌选择；1 = 获得两张拼点牌。
    if payload.len() != 8 {
        return Err(EffectError::EngineFault("bad Zhiba settle payload".into()));
    }
    if selections[0] != 1 {
        tx.state_mut().skills.get_mut(skill).map(|instance| {
            instance.clear_zhiba_compare();
        });
        return Ok(());
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveZhibaSettle {
            player,
            actor,
            winner: PlayerId::new(payload[5]),
            player_card: engine_api::CardId::new(payload[6]),
            target_card: engine_api::CardId::new(payload[7]),
            stage: 1,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_tiaoxin(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let mut target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let alive = |id: PlayerId| {
        tx.state()
            .players
            .get(id)
            .is_some_and(|participant| participant.alive)
    };
    match stage {
        1 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "挑衅 target choice must be made by the owner".into(),
                ));
            }
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "挑衅 requires exactly one target".into(),
                ));
            }
            let chosen = PlayerId::new(selections[0]);
            if !alive(chosen)
                || crate::query::discardable_hand_equip_cards(tx.state(), chosen).is_empty()
            {
                return Err(EffectError::Illegal(
                    "挑衅 target is no longer legal".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                chosen,
                1,
                1,
                true,
                vec![1, 0],
                engine_api::ChoiceKind::Custom,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    chosen.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            if observer != target || !alive(target) {
                return Err(EffectError::Illegal(
                    "挑衅 response must come from the challenged player".into(),
                ));
            }
            if cancelled || selections.first().copied() != Some(1) {
                return crate::effect::open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    crate::query::discardable_hand_equip_cards(tx.state(), target)
                        .into_iter()
                        .map(engine_api::CardId::raw)
                        .collect(),
                    engine_api::ChoiceKind::SelectCards,
                    crate::state::ContinuationKind::Custom,
                    vec![
                        crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                        4,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ],
                )
                .map(|_| ());
            }
            let cards = crate::command::play::prompted_sha_source_cards(tx.state(), target);
            if cards.is_empty() {
                return crate::effect::open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    crate::query::discardable_hand_equip_cards(tx.state(), target)
                        .into_iter()
                        .map(engine_api::CardId::raw)
                        .collect(),
                    engine_api::ChoiceKind::SelectCards,
                    crate::state::ContinuationKind::Custom,
                    vec![
                        crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                        4,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ],
                )
                .map(|_| ());
            }
            crate::effect::open_choice(
                tx,
                target,
                0,
                1,
                true,
                cards.into_iter().map(engine_api::CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                    3,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        3 => {
            if observer != target || !alive(target) {
                return Err(EffectError::Illegal(
                    "挑衅 Sha source must come from the challenged player".into(),
                ));
            }
            let mut discard_choice = || {
                crate::effect::open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    crate::query::discardable_hand_equip_cards(tx.state(), target)
                        .into_iter()
                        .map(engine_api::CardId::raw)
                        .collect(),
                    engine_api::ChoiceKind::SelectCards,
                    crate::state::ContinuationKind::Custom,
                    vec![
                        crate::effect::RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                        4,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ],
                )
                .map(|_| ())
            };
            if cancelled || selections.is_empty() {
                return discard_choice();
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "choose at most one 挑衅 Sha source".into(),
                ));
            };
            let card = engine_api::CardId::new(*selected);
            if !crate::command::play::prompted_sha_source_cards(tx.state(), target).contains(&card) {
                return Err(EffectError::Illegal(
                    "挑衅 Sha source is no longer legal".into(),
                ));
            }
            crate::command::play::play_prompted_sha_source_unlimited(tx, target, card, &[player])
        }
        4 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "挑衅 discard choice must be made by the owner".into(),
                ));
            }
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "挑衅 requires one discard card".into(),
                ));
            }
            let card = engine_api::CardId::new(selections[0]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&card) {
                return Err(EffectError::Illegal(
                    "挑衅 discard card is no longer legal".into(),
                ));
            }
            let zone = tx
                .state()
                .cards
                .get(card)
                .map(|value| value.zone)
                .ok_or_else(|| EffectError::Illegal("挑衅 discard card missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(zone),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 挑衅 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_zhengrong(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    match stage {
        1 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "征荣 target choice must be made by the owner".into(),
                ));
            }
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("征荣 requires one target".into()));
            }
            let chosen = PlayerId::new(selections[0]);
            let owner_hand = tx.state().hand_count(player);
            if chosen == player
                || !tx.state().players.get(chosen).is_some_and(|p| p.alive)
                || tx.state().hand_count(chosen) < owner_hand
                || crate::query::discardable_hand_equip_cards(tx.state(), chosen).is_empty()
            {
                return Err(EffectError::Illegal(
                    "征荣 target is no longer legal".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                crate::query::discardable_hand_equip_cards(tx.state(), chosen)
                    .into_iter()
                    .map(engine_api::CardId::raw)
                    .collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_ZHENGRONG_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    chosen.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "征荣 card choice must be made by the owner".into(),
                ));
            }
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("征荣 requires one card".into()));
            }
            let card = engine_api::CardId::new(selections[0]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&card) {
                return Err(EffectError::Illegal("征荣 card is no longer legal".into()));
            }
            let zone = tx
                .state()
                .cards
                .get(card)
                .map(|value| value.zone)
                .ok_or_else(|| EffectError::Illegal("征荣 card missing".into()))?;
            let skill_def = crate::effect::skill_pile_def(tx, player, skill)?;
            let pile = crate::effect::ensure_skill_pile_zone(tx, player, skill_def)?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: pile,
                    from: Some(zone),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 征荣 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_hongju(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let qingjian_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let rong_def = engine_api::SkillDefId::new(*payload.get(4).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(5).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "鸿举 choice must be made by the owner".into(),
        ));
    }
    let rong = tx.state().skill_pile_cards(player, rong_def.raw());
    match stage {
        1 => {
            if cancelled || selections.first().copied() != Some(1) {
                return crate::effect::run_effects(
                    tx,
                    vec![
                        EffectRequest::ChangeMaxHp { player, delta: -1 },
                        EffectRequest::AddSkill {
                            player,
                            skill_def: qingjian_def,
                        },
                    ],
                )
                .map(|_| ());
            }
            let count = rong.len();
            let hand = tx.state().hand_cards(player);
            if count == 0 || hand.len() < count {
                return crate::effect::run_effects(
                    tx,
                    vec![
                        EffectRequest::ChangeMaxHp { player, delta: -1 },
                        EffectRequest::AddSkill {
                            player,
                            skill_def: qingjian_def,
                        },
                    ],
                )
                .map(|_| ());
            }
            crate::effect::open_choice(
                tx,
                player,
                count.min(u8::MAX as usize) as u8,
                count.min(u8::MAX as usize) as u8,
                false,
                hand.into_iter().map(engine_api::CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_HONGJU_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    qingjian_def.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            if selections.len() != rong.len() {
                return Err(EffectError::Illegal(
                    "鸿举 hand selection count mismatch".into(),
                ));
            }
            let hand = tx.state().hand_cards(player);
            let selected: Vec<_> = selections
                .iter()
                .map(|&raw| engine_api::CardId::new(raw))
                .collect();
            if selected.iter().any(|card| !hand.contains(card)) {
                return Err(EffectError::Illegal(
                    "鸿举 hand card is no longer legal".into(),
                ));
            }
            let hand_zone = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            let pile = crate::effect::ensure_skill_pile_zone(tx, player, rong_def.raw())?;
            crate::effect::run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: rong.clone(),
                        to: hand_zone,
                        from: Some(pile),
                    },
                    EffectRequest::MoveCards {
                        cards: selected,
                        to: pile,
                        from: Some(hand_zone),
                    },
                ],
            )?;
            crate::effect::run_effects(
                tx,
                vec![
                    EffectRequest::ChangeMaxHp { player, delta: -1 },
                    EffectRequest::AddSkill {
                        player,
                        skill_def: qingjian_def,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 鸿举 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_qingjian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let rong_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "清俭 choice must be made by the owner".into(),
        ));
    }
    match stage {
        1 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("清俭 requires one 荣".into()));
            }
            let rong = tx.state().skill_pile_cards(player, rong_def.raw());
            if !rong.contains(&engine_api::CardId::new(selections[0])) {
                return Err(EffectError::Illegal("清俭 荣 is no longer legal".into()));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                tx.state()
                    .hand_cards(player)
                    .into_iter()
                    .map(engine_api::CardId::raw)
                    .collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_QINGJIAN_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    rong_def.raw(),
                    selections[0],
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("清俭 requires one hand card".into()));
            }
            let hand_card = engine_api::CardId::new(selections[0]);
            if !tx.state().hand_cards(player).contains(&hand_card) {
                return Err(EffectError::Illegal(
                    "清俭 hand card is no longer legal".into(),
                ));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .filter(|&candidate| {
                    !crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        candidate,
                        rule_api::TargetCardScope::EquipJudge,
                    )
                    .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Err(EffectError::Illegal("清俭 has no legal target".into()));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_QINGJIAN_CUSTOM_CHOICE,
                    3,
                    player.raw(),
                    rong_def.raw(),
                    payload.get(4).copied().unwrap_or(0),
                    selections[0],
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        3 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("清俭 requires one target".into()));
            }
            let target = PlayerId::new(selections[0]);
            let cards = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::EquipJudge,
            );
            if cards.is_empty() {
                return Err(EffectError::Illegal("清俭 target has no ej card".into()));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(engine_api::CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_QINGJIAN_CUSTOM_CHOICE,
                    4,
                    player.raw(),
                    rong_def.raw(),
                    payload.get(4).copied().unwrap_or(0),
                    payload.get(5).copied().unwrap_or(0),
                    selections[0],
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        4 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("清俭 requires one ej card".into()));
            }
            let rong_card = engine_api::CardId::new(*payload.get(4).unwrap_or(&0));
            let hand_card = engine_api::CardId::new(*payload.get(5).unwrap_or(&0));
            let target = PlayerId::new(*payload.get(6).unwrap_or(&0));
            let ej_card = engine_api::CardId::new(selections[0]);
            if !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::EquipJudge,
            )
            .contains(&ej_card)
            {
                return Err(EffectError::Illegal(
                    "清俭 ej card is no longer legal".into(),
                ));
            }
            let rong_zone = crate::effect::ensure_skill_pile_zone(tx, player, rong_def.raw())?;
            let hand_zone = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            let discard = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let ej_zone = tx
                .state()
                .cards
                .get(ej_card)
                .map(|value| value.zone)
                .ok_or_else(|| EffectError::Illegal("清俭 ej card missing".into()))?;
            let hand_card_zone = tx
                .state()
                .cards
                .get(hand_card)
                .map(|value| value.zone)
                .ok_or_else(|| EffectError::Illegal("清俭 hand card missing".into()))?;
            crate::effect::run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![rong_card],
                        to: hand_zone,
                        from: Some(rong_zone),
                    },
                    EffectRequest::MoveCards {
                        cards: vec![hand_card],
                        to: discard,
                        from: Some(hand_card_zone),
                    },
                    EffectRequest::MoveCards {
                        cards: vec![ej_card],
                        to: discard,
                        from: Some(ej_zone),
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 清俭 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_releiji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "雷击 target choice must be made by the owner".into(),
        ));
    }
    match stage {
        1 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("雷击 requires one target".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player || !tx.state().players.get(target).is_some_and(|p| p.alive) {
                return Err(EffectError::Illegal(
                    "雷击 target is no longer legal".into(),
                ));
            }
            // 翻亮判定牌（留在 processing 供鬼道改判）。
            let draw = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            if tx
                .state()
                .zones
                .get(draw)
                .is_none_or(|zone| zone.cards.is_empty())
            {
                crate::effect::reshuffle_discard_into_draw(tx)?;
            }
            let result_card = tx
                .state()
                .zones
                .get(draw)
                .and_then(|zone| zone.cards.last().copied())
                .ok_or_else(|| EffectError::Illegal("draw empty".into()))?;
            let processing = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
            crate::effect::move_one_card(tx, result_card, draw, processing, PlayerId::NONE, true)?;
            // 征询改判者（座次自目标起），复用 judgment_rewrite_cards 候选。
            let order = tx.state().players.alive_by_seat();
            let Some(start) = order.iter().position(|&p| p == target) else {
                return Err(EffectError::EngineFault(
                    "target missing from seat order".into(),
                ));
            };
            let rewriters: Vec<_> = (0..order.len())
                .map(|offset| order[(start + offset) % order.len()])
                .filter(|&p| crate::query::can_rewrite_judgment(tx.state(), p))
                .collect();
            if rewriters.is_empty() {
                return settle_releiji_judgment(tx, player, target, result_card, skill);
            }
            open_rewrite_releiji(tx, &rewriters, player, target, result_card, skill)
        }
        2 | 3 => {
            // 改判选择（stage 2）：换牌后继续下一改判者或结算。
            let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
            let result_card = engine_api::CardId::new(*payload.get(5).unwrap_or(&0));
            let rewriter_len = *payload.get(6).unwrap_or(&0) as usize;
            let rewriters: Vec<_> = payload
                .get(7..7 + rewriter_len)
                .filter(|rest| rest.len() == rewriter_len)
                .ok_or_else(|| EffectError::EngineFault("truncated 雷击 rewriter queue".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            if !cancelled && selections.len() == 1 {
                let card = engine_api::CardId::new(selections[0]);
                if observer == rewriters[0] {
                    let legal = crate::command::judge::judgment_rewrite_cards(tx, rewriters[0])
                        .contains(&card);
                    if !legal {
                        return Err(EffectError::Illegal(
                            "雷击 rewrite card no longer legal".into(),
                        ));
                    }
                    let discard = tx
                        .state()
                        .zones
                        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
                    let card_zone = tx
                        .state()
                        .cards
                        .get(card)
                        .map(|value| value.zone)
                        .ok_or_else(|| EffectError::Illegal("rewrite card missing".into()))?;
                    // 改判牌消费：原判定牌 → 弃牌堆；改判牌成为新判定牌（processing）。
                    let processing = tx
                        .state()
                        .zones
                        .find(crate::state::ZoneKind::Processing, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
                    crate::effect::move_one_card(
                        tx,
                        result_card,
                        processing,
                        discard,
                        PlayerId::NONE,
                        false,
                    )?;
                    crate::effect::move_one_card(
                        tx,
                        card,
                        card_zone,
                        processing,
                        PlayerId::NONE,
                        true,
                    )?;
                    let new_result = card;
                    let rest = &rewriters[1..];
                    if rest.is_empty() {
                        return settle_releiji_judgment(tx, player, target, new_result, skill);
                    }
                    return open_rewrite_releiji(tx, rest, player, target, new_result, skill);
                }
            }
            // 放弃/无合法改判牌：继续下一改判者。
            let rest = &rewriters[1..];
            if rest.is_empty() {
                return settle_releiji_judgment(tx, player, target, result_card, skill);
            }
            open_rewrite_releiji(tx, rest, player, target, result_card, skill)
        }
        _ => Err(EffectError::EngineFault("unknown 雷击 choice stage".into())),
    }
}

/// Open the next rewriter's rewrite choice for the 雷击 judgment.
pub(crate) fn open_rewrite_releiji(
    tx: &mut Transaction,
    rewriters: &[PlayerId],
    player: PlayerId,
    target: PlayerId,
    result_card: engine_api::CardId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let Some(&rewriter) = rewriters.first() else {
        return settle_releiji_judgment(tx, player, target, result_card, skill);
    };
    let options = crate::command::judge::judgment_rewrite_cards(tx, rewriter);
    if options.is_empty() {
        return open_rewrite_releiji(tx, &rewriters[1..], player, target, result_card, skill);
    }
    let mut payload = vec![
        crate::effect::RESOLVE_RELEIJI_CUSTOM_CHOICE,
        2,
        player.raw(),
        target.raw(),
        skill.raw(),
        result_card.raw(),
        rewriters.len() as u32,
    ];
    payload.extend(rewriters.iter().map(|p| p.raw()));
    crate::effect::open_choice(
        tx,
        rewriter,
        0,
        1,
        true,
        options.into_iter().map(engine_api::CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        payload,
    )
    .map(|_| ())
}

/// Settle the 雷击 judgment: move the final card to discard and apply the suit branch.
pub(crate) fn settle_releiji_judgment(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    result_card: engine_api::CardId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let processing = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![result_card],
            to: discard,
            from: Some(processing),
        }],
    )?;
    let suit = tx
        .state()
        .cards
        .get(result_card)
        .map(|value| crate::query::effective_card_suit(tx.state(), target, value))
        .unwrap_or(engine_api::Suit::None);
    let rank = tx
        .state()
        .cards
        .get(result_card)
        .map(|value| value.rank)
        .unwrap_or(0);
    tx.push_event(engine_api::DomainEvent::RuleNote {
        code: 234,
        payload: vec![
            player.raw(),
            target.raw(),
            crate::command::judge::suit_tag(suit),
            rank as u32,
            skill.raw(),
        ],
    });
    match suit {
        engine_api::Suit::Spade => crate::effect::run_effects(
            tx,
            vec![EffectRequest::DealCardDamage {
                source: PlayerId::NONE,
                target,
                amount: 2,
                is_lose_hp: false,
                nature: rule_api::damage_nature::THUNDER,
                card: engine_api::CardId::NONE,
                card_kind: 0,
                card_flags: 0,
            }],
        )
        .map(|_| ()),
        engine_api::Suit::Club => crate::effect::run_effects(
            tx,
            vec![
                EffectRequest::RecoverHp {
                    source: PlayerId::NONE,
                    player,
                    amount: 1,
                },
                EffectRequest::DealCardDamage {
                    source: PlayerId::NONE,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::THUNDER,
                    card: engine_api::CardId::NONE,
                    card_kind: 0,
                    card_flags: 0,
                },
            ],
        )
        .map(|_| ()),
        _ => Ok(()),
    }
}

pub(crate) fn resolve_resolve_huangtian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let actor = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    match stage {
        1 => {
            if observer != actor {
                return Err(EffectError::Illegal(
                    "黄天 choice must be made by the Qun player".into(),
                ));
            }
            if cancelled || selections.len() != 1 {
                return Ok(()); // 拒绝/取消：不赠送。
            }
            let card = engine_api::CardId::new(selections[0]);
            let legal = tx.state().hand_cards(actor).into_iter().any(|c| c == card)
                && tx.state().cards.get(card).is_some_and(|value| {
                    matches!(
                        crate::query::effective_card_kind(tx.state(), actor, value),
                        crate::state::CardKind::Shan | crate::state::CardKind::TrickShandian
                    )
                });
            if !legal {
                return Err(EffectError::Illegal("黄天 card is no longer legal".into()));
            }
            let zone = tx
                .state()
                .cards
                .get(card)
                .map(|value| value.zone)
                .ok_or_else(|| EffectError::Illegal("黄天 card missing".into()))?;
            let hand = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no lord hand zone".into()))?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: hand,
                    from: Some(zone),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 黄天 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_haoshi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "好施 choice must be made by the owner".into(),
        ));
    }
    match stage {
        2 => {
            if cancelled || selections.is_empty() {
                return Err(EffectError::Illegal(
                    "好施 requires the half hand cards".into(),
                ));
            }
            let give = tx.state().hand_count(player) / 2;
            if selections.len() != give as usize
                || !tx.state().players.get(target).is_some_and(|p| p.alive)
            {
                return Err(EffectError::Illegal("好施 target or count changed".into()));
            }
            let cards: Vec<_> = selections
                .iter()
                .map(|&raw| engine_api::CardId::new(raw))
                .collect();
            let hand = tx.state().hand_cards(player);
            if cards.iter().any(|card| !hand.contains(card)) {
                return Err(EffectError::Illegal("好施 card is no longer legal".into()));
            }
            let hand_zone = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("no target hand zone".into()))?;
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: target_hand,
                    from: Some(hand_zone),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 好施 choice stage".into())),
    }
}

pub(crate) fn resolve_resolve_dimeng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "缔盟 choice must be made by the owner".into(),
        ));
    }
    match stage {
        1 => {
            if cancelled || selections.len() != 2 {
                return Err(EffectError::Illegal("缔盟 requires two targets".into()));
            }
            let left = PlayerId::new(selections[0]);
            let right = PlayerId::new(selections[1]);
            if left == right
                || left == player
                || right == player
                || !tx.state().players.get(left).is_some_and(|p| p.alive)
                || !tx.state().players.get(right).is_some_and(|p| p.alive)
            {
                return Err(EffectError::Illegal(
                    "缔盟 targets are no longer legal".into(),
                ));
            }
            let diff = tx
                .state()
                .hand_count(left)
                .abs_diff(tx.state().hand_count(right));
            let options = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .chain(
                    tx.state()
                        .zones
                        .find(crate::state::ZoneKind::Equip, player)
                        .map(|zone| {
                            tx.state()
                                .zones
                                .get(zone)
                                .map(|zone| zone.cards.clone())
                                .unwrap_or_default()
                        })
                        .unwrap_or_default(),
                )
                .collect::<Vec<_>>();
            if options.len() < diff as usize {
                return Err(EffectError::Illegal(
                    "缔盟 cannot pay the hand difference".into(),
                ));
            }
            if diff == 0 {
                // 手牌相等：直接交换，无需弃牌。
                return exchange_dimeng_hands(tx, player, left, right, &[], skill);
            }
            crate::effect::open_choice(
                tx,
                player,
                diff.min(u8::MAX as u32) as u8,
                diff.min(u8::MAX as u32) as u8,
                false,
                options.into_iter().map(engine_api::CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_DIMENG_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    left.raw(),
                    right.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled {
                return Err(EffectError::Illegal(
                    "缔盟 requires the discard payment".into(),
                ));
            }
            let left = PlayerId::new(*payload.get(3).unwrap_or(&0));
            let right = PlayerId::new(*payload.get(4).unwrap_or(&0));
            let diff = tx
                .state()
                .hand_count(left)
                .abs_diff(tx.state().hand_count(right));
            if selections.len() != diff as usize {
                return Err(EffectError::Illegal(
                    "缔盟 discard count must equal the difference".into(),
                ));
            }
            let cards: Vec<_> = selections
                .iter()
                .map(|&raw| engine_api::CardId::new(raw))
                .collect();
            let owner_cards = tx.state().hand_cards(player);
            let equip_cards = tx
                .state()
                .zones
                .find(crate::state::ZoneKind::Equip, player)
                .map(|zone| {
                    tx.state()
                        .zones
                        .get(zone)
                        .map(|zone| zone.cards.clone())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let all = owner_cards
                .into_iter()
                .chain(equip_cards)
                .collect::<Vec<_>>();
            if cards.iter().any(|card| !all.contains(card)) {
                return Err(EffectError::Illegal("缔盟 card is no longer legal".into()));
            }
            exchange_dimeng_hands(tx, player, left, right, &cards, skill)
        }
        _ => Err(EffectError::EngineFault("unknown 缔盟 choice stage".into())),
    }
}

/// 缔盟结算：弃置代价牌后交换两名角色手牌。
pub(crate) fn exchange_dimeng_hands(
    tx: &mut Transaction,
    player: PlayerId,
    left: PlayerId,
    right: PlayerId,
    discard_cards: &[engine_api::CardId],
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut requests = Vec::new();
    for card in discard_cards {
        let zone = tx
            .state()
            .cards
            .get(*card)
            .map(|value| value.zone)
            .ok_or_else(|| EffectError::Illegal("缔盟 discard card missing".into()))?;
        requests.push(EffectRequest::MoveCards {
            cards: vec![*card],
            to: discard,
            from: Some(zone),
        });
    }
    let left_hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, left)
        .ok_or_else(|| EffectError::Illegal("no left hand zone".into()))?;
    let right_hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, right)
        .ok_or_else(|| EffectError::Illegal("no right hand zone".into()))?;
    let left_cards = tx.state().hand_cards(left);
    let right_cards = tx.state().hand_cards(right);
    requests.push(EffectRequest::MoveCards {
        cards: left_cards,
        to: right_hand,
        from: Some(left_hand),
    });
    requests.push(EffectRequest::MoveCards {
        cards: right_cards,
        to: left_hand,
        from: Some(right_hand),
    });
    crate::effect::run_effects(tx, requests).map(|_| ())
}

/// 享乐：使用者弃一张基本牌，否则此杀对持有者无效。
pub(crate) fn resolve_resolve_xiangle(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let source = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let card = CardId::new(*payload.get(4).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(5).unwrap_or(&0));
    if observer != source {
        return Err(EffectError::Illegal(
            "享乐 choice must be made by the Sha user".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        // 拒绝弃牌：此杀对持有者无效。
        return crate::effect::run_effects(
            tx,
            vec![EffectRequest::ExcludeSavedShaTarget {
                card,
                target: player,
            }],
        )
        .map(|_| ());
    }
    let picked = CardId::new(selections[0]);
    let hand = tx.state().hand_cards(source);
    let basic_ok = tx
        .state()
        .cards
        .get(picked)
        .is_some_and(|value| value.kind.is_basic());
    if !hand.contains(&picked) || !basic_ok {
        return Err(EffectError::Illegal("享乐 card is no longer legal".into()));
    }
    let hand_zone = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, source)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: discard,
            from: Some(hand_zone),
        }],
    )
    .map(|_| ())
}

/// 放权：弃一张手牌后选择一名其他角色进行额外回合。
pub(crate) fn resolve_fangquan_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "放权 choice must be made by the owner".into(),
        ));
    }
    if cancelled {
        // 放弃放权：清除标记，结束阶段不再授回合。
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.state_blob.clear();
        }
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("放权 requires one hand card".into()));
    }
    let picked = CardId::new(selections[0]);
    let hand = tx.state().hand_cards(player);
    if !hand.contains(&picked) {
        return Err(EffectError::Illegal(
            "放权 card is no longer in hand".into(),
        ));
    }
    let hand_zone = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: discard,
            from: Some(hand_zone),
        }],
    )?;
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player)
        .collect();
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        engine_api::ChoiceKind::SelectTargets,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_FANGQUAN_TARGET,
            2,
            player.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 放权：授目标一个额外回合并清除标记。
pub(crate) fn resolve_fangquan_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "放权 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.state_blob.clear();
        }
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if target == player || !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "放权 target is no longer legal".into(),
        ));
    }
    if tx.state().turn.extra_turn_queue.len() >= 8 {
        return Err(EffectError::Illegal("放权 extra turn queue is full".into()));
    }
    tx.state_mut().turn.extra_turn_queue.push(target);
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob.clear();
    }
    Ok(())
}

/// 罪论（0<N<3）：选 N 张置牌堆顶并获其余。
pub(crate) fn resolve_zuilun_pick(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let count = *payload.get(4).unwrap_or(&0) as usize;
    if observer != player {
        return Err(EffectError::Illegal(
            "罪论 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.len() != count || count == 0 || count >= 3 {
        return Err(EffectError::Illegal(
            "罪论 requires exactly the counted picks".into(),
        ));
    }
    let revealed: Vec<_> = payload
        .get(5..5 + 3)
        .unwrap_or(&[])
        .iter()
        .map(|&raw| CardId::new(raw))
        .collect();
    let picks: Vec<_> = selections.iter().map(|&raw| CardId::new(raw)).collect();
    let processing = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let processing_cards = tx
        .state()
        .zones
        .get(processing)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    if picks
        .iter()
        .any(|card| !revealed.contains(card) || !processing_cards.contains(card))
    {
        return Err(EffectError::Illegal(
            "罪论 pick is no longer revealed".into(),
        ));
    }
    let draw = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let mut requests = Vec::new();
    for &card in &picks {
        requests.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: draw,
            from: Some(processing),
        });
    }
    let keep: Vec<_> = revealed
        .iter()
        .copied()
        .filter(|card| !picks.contains(card))
        .collect();
    if !keep.is_empty() {
        requests.push(EffectRequest::MoveCards {
            cards: keep,
            to: hand,
            from: Some(processing),
        });
    }
    crate::effect::run_effects(tx, requests).map(|_| ())
}

/// 罪论（N==3）：目标确定后一起失去 1 点体力。
pub(crate) fn resolve_zuilun_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "罪论 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if target == player || !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "罪论 target is no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: player,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            },
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            },
        ],
    )
    .map(|_| ())
}

/// 拒战（阳分支）：获得目标一张牌并禁己方用牌。
pub(crate) fn resolve_juzhan_gain(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "拒战 gain choice must be made by the owner".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("拒战 requires one gained card".into()));
    }
    let picked = CardId::new(selections[0]);
    let target_cards = tx.state().hand_cards(target);
    let equip_cards = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Equip, target)
        .map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if !target_cards.contains(&picked) && !equip_cards.contains(&picked) {
        return Err(EffectError::Illegal("拒战 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    let owner_hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no owner hand zone".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: owner_hand,
            from: from_zone,
        }],
    )?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::BanUseAgainst {
            user: player,
            target,
        }],
    )
    .map(|_| ())
}

/// 奇制：选定目标后弃其一张牌并令其摸一张，计数 +1。
pub(crate) fn resolve_qizhi_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "奇制 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if target == player || !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "奇制 target is no longer legal".into(),
        ));
    }
    let mut options = tx.state().hand_cards(target);
    options.extend(
        tx.state()
            .zones
            .find(crate::state::ZoneKind::Equip, target)
            .map(|zone| {
                tx.state()
                    .zones
                    .get(zone)
                    .map(|zone| zone.cards.clone())
                    .unwrap_or_default()
            })
            .unwrap_or_default(),
    );
    if options.is_empty() {
        return Ok(());
    }
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(engine_api::CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_QIZHI_CARD,
            2,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 奇制：弃置选定牌并令目标摸一张。
pub(crate) fn resolve_qizhi_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "奇制 card choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let picked = CardId::new(selections[0]);
    let target_cards = tx.state().hand_cards(target);
    let equip_cards = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Equip, target)
        .map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if !target_cards.contains(&picked) && !equip_cards.contains(&picked) {
        return Err(EffectError::Illegal("奇制 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![picked],
                to: discard,
                from: from_zone,
            },
            EffectRequest::DrawCards {
                player: target,
                count: 1,
            },
        ],
    )?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::ResolveQizhi {
            player,
            targets: Vec::new(),
            stage: 2,
            skill,
        }],
    )
    .map(|_| ())
}

/// 进趋：弃置至奇制次数张。
pub(crate) fn resolve_jinqu_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let qizhi_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "进趋 choice must be made by the owner".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let keep = crate::effect::skill_instance_by_def(tx, player, qizhi_def)
        .and_then(|id| tx.state().skills.get(id))
        .map(|instance| crate::effect::qizhi_count(tx, instance.id))
        .unwrap_or(0);
    let expected = tx.state().hand_count(player).saturating_sub(keep);
    if selections.len() != expected as usize {
        return Err(EffectError::Illegal("进趋 discard count changed".into()));
    }
    let cards: Vec<_> = selections.iter().map(|&raw| CardId::new(raw)).collect();
    let hand = tx.state().hand_cards(player);
    if cards.iter().any(|card| !hand.contains(card)) {
        return Err(EffectError::Illegal(
            "进趋 card is no longer in hand".into(),
        ));
    }
    let hand_zone = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand_zone),
        }],
    )
    .map(|_| ())
}

/// 荐相：目标确认后摸一张。
pub(crate) fn resolve_jianxiang_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "荐相 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "荐相 target is no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 1,
        }],
    )
    .map(|_| ())
}

/// 审时：交牌给目标（阴=来源并记录追踪；阳=手牌最多者）。
pub(crate) fn resolve_shenshi_give(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let source = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "审时 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let picked = CardId::new(selections[0]);
    let owner_cards = tx.state().hand_cards(player);
    let equip_cards = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Equip, player)
        .map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if !owner_cards.contains(&picked) && !equip_cards.contains(&picked) {
        return Err(EffectError::Illegal("审时 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    if stage == 1 {
        // 阴：交给来源并记录追踪牌。
        let source_hand = tx
            .state()
            .zones
            .find(crate::state::ZoneKind::Hand, source)
            .ok_or_else(|| EffectError::Illegal("no source hand zone".into()))?;
        crate::effect::run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: vec![picked],
                to: source_hand,
                from: from_zone,
            }],
        )?;
        let mut blob = b"SS1".to_vec();
        blob.push(1);
        blob.extend_from_slice(&source.raw().to_le_bytes());
        blob.extend_from_slice(&picked.raw().to_le_bytes());
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.state_blob = blob;
        }
        return Ok(());
    }
    // 阳：选手牌最多者，交牌并造成 1 点伤害。
    let max_hand = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|candidate| tx.state().hand_count(candidate))
        .max()
        .unwrap_or(0);
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player && tx.state().hand_count(candidate) == max_hand)
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
        options.into_iter().map(PlayerId::raw).collect(),
        engine_api::ChoiceKind::SelectTargets,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_SHENSHI_TARGET,
            3,
            player.raw(),
            skill.raw(),
            picked.raw(),
        ],
    )
    .map(|_| ())
}

/// 审时（阳）：交牌 + 伤害，若目标死亡可令一名角色摸至四张。
pub(crate) fn resolve_shenshi_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let picked = CardId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "审时 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "审时 target is no longer legal".into(),
        ));
    }
    let target_hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand zone".into()))?;
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    crate::effect::run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![picked],
                to: target_hand,
                from: from_zone,
            },
            EffectRequest::DealDamage {
                source: player,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            },
        ],
    )?;
    if tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Ok(());
    }
    // 目标死亡：可选令一名角色手牌摸至四张。
    let options: Vec<_> = tx.state().players.alive_by_seat().into_iter().collect();
    if options.is_empty() {
        return Ok(());
    }
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        engine_api::ChoiceKind::SelectTargets,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_SHENSHI_REWARD,
            4,
            player.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 审时（阳）：死亡奖励——目标手牌摸至四张。
pub(crate) fn resolve_shenshi_reward(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "审时 reward choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    let draw = 4u32.saturating_sub(tx.state().hand_count(target));
    if draw > 0 {
        crate::effect::run_effects(
            tx,
            vec![EffectRequest::DrawCards {
                player: target,
                count: draw,
            }],
        )?;
    }
    Ok(())
}

/// 明任：将一张手牌置于「任」技能牌堆。
pub(crate) fn resolve_mingren_place(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.get(1).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "明任 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let picked = CardId::new(selections[0]);
    let hand = tx.state().hand_cards(player);
    if !hand.contains(&picked) {
        return Err(EffectError::Illegal(
            "明任 card is no longer in hand".into(),
        ));
    }
    let hand_zone = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("明任 skill disappeared".into()))?;
    let pile = crate::effect::ensure_skill_pile_zone(tx, player, def)?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: pile,
            from: Some(hand_zone),
        }],
    )?;
    if stage == 3 {
        // 替换：原「任」回到手牌。
        let old: Vec<_> = tx
            .state()
            .skill_piles
            .iter()
            .find(|pile| pile.owner == player && pile.skill_def == def)
            .and_then(|pile| tx.state().zones.get(pile.zone))
            .map(|zone| {
                zone.cards
                    .iter()
                    .filter(|card| **card != picked)
                    .copied()
                    .collect()
            })
            .unwrap_or_default();
        if let Some(old_card) = old.first().copied() {
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![old_card],
                    to: hand_zone,
                    from: None,
                }],
            )?;
        }
    }
    Ok(())
}

/// 贞良（阴）：弃同色牌并开攻击范围内目标选择。
pub(crate) fn resolve_zhenliang_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "贞良 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let picked = CardId::new(selections[0]);
    let owner_cards = tx.state().hand_cards(player);
    let equip_cards = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Equip, player)
        .map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .unwrap_or_default();
    if !owner_cards.contains(&picked) && !equip_cards.contains(&picked) {
        return Err(EffectError::Illegal("贞良 card is no longer legal".into()));
    }
    let from_zone = tx.state().cards.get(picked).map(|value| value.zone);
    let discard = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![picked],
            to: discard,
            from: from_zone,
        }],
    )?;
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            candidate != player && crate::effect::dcchiying_in_range(tx.state(), player, candidate)
        })
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
        options.into_iter().map(PlayerId::raw).collect(),
        engine_api::ChoiceKind::SelectTargets,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_ZHENLIANG_DAMAGE,
            2,
            player.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// 贞良（阴）：对攻击范围内目标造成 1 点伤害。
pub(crate) fn resolve_zhenliang_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "贞良 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "贞良 target is no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .map(|_| ())
}

/// 贞良（阳）：目标摸一张牌。
pub(crate) fn resolve_zhenliang_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "贞良 draw choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "贞良 draw target is no longer legal".into(),
        ));
    }
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 1,
        }],
    )
    .map(|_| ())
}

/// 溃诛：选择摸牌或伤害分支并开目标选择。
pub(crate) fn resolve_kuizhu_dispatch(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let x = *payload.get(4).unwrap_or(&0);
    if observer != player {
        return Err(EffectError::Illegal(
            "溃诛 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let damage_branch = selections[0] == 1;
    let options: Vec<_> = tx.state().players.alive_by_seat();
    if options.is_empty() {
        return Ok(());
    }
    let tag = if damage_branch {
        crate::effect::RESOLVE_KUIZHU_DAMAGE
    } else {
        crate::effect::RESOLVE_KUIZHU_DRAW
    };
    crate::effect::open_choice(
        tx,
        player,
        0,
        x.min(8) as u8,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        engine_api::ChoiceKind::SelectTargets,
        crate::state::ContinuationKind::Custom,
        vec![tag, 2, player.raw(), skill.raw(), x],
    )
    .map(|_| ())
}

/// 溃诛（摸牌分支）：至多 X 名角色各摸一张。
pub(crate) fn resolve_kuizhu_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let x = *payload.get(4).unwrap_or(&0) as usize;
    if observer != player {
        return Err(EffectError::Illegal(
            "溃诛 draw choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    if selections.len() > x || selections.len() > 8 {
        return Err(EffectError::Illegal(
            "溃诛 draw target count exceeded".into(),
        ));
    }
    let requests: Vec<_> = selections
        .iter()
        .map(|&raw| EffectRequest::DrawCards {
            player: PlayerId::new(raw),
            count: 1,
        })
        .collect();
    crate::effect::run_effects(tx, requests).map(|_| ())
}

/// 溃诛（伤害分支）：体力值之和为 X 的角色各受 1 点伤害。
pub(crate) fn resolve_kuizhu_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let x = *payload.get(4).unwrap_or(&0) as usize;
    if observer != player {
        return Err(EffectError::Illegal(
            "溃诛 damage choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let sum: usize = selections
        .iter()
        .map(|&raw| {
            tx.state()
                .players
                .get(PlayerId::new(raw))
                .map(|p| p.hp.max(0) as usize)
                .unwrap_or(0)
        })
        .sum();
    if sum != x || selections.len() > 8 {
        return Err(EffectError::Illegal(
            "溃诛 damage target hp sum must equal the discard count".into(),
        ));
    }
    let requests: Vec<_> = selections
        .iter()
        .map(|&raw| EffectRequest::DealDamage {
            source: player,
            target: PlayerId::new(raw),
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        })
        .collect();
    crate::effect::run_effects(tx, requests).map(|_| ())
}

/// 掣政：弃置目标的一张牌。
pub(crate) fn resolve_zhizheng_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "掣政 choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "掣政 target is no longer legal".into(),
        ));
    }
    let mut options = tx.state().hand_cards(target);
    options.extend(
        tx.state()
            .zones
            .find(crate::state::ZoneKind::Equip, target)
            .map(|zone| {
                tx.state()
                    .zones
                    .get(zone)
                    .map(|zone| zone.cards.clone())
                    .unwrap_or_default()
            })
            .unwrap_or_default(),
    );
    if options.is_empty() {
        return Ok(());
    }
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(engine_api::CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_ZHIZHENG_CARD,
            2,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}
