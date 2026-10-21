//! Command operations for `resolve_qiaobian_discard` and related helpers.

use super::*;

/// 巧变（弃牌阶段开始）：支付一张手牌，随后按阶段开启收益选择或直接跳过。
pub(crate) fn resolve_qiaobian_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let phase = *payload.get(2).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "巧变 choice must be made by the owner".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal("巧变 requires one hand card".into()));
    }
    let card = engine_api::CardId::new(selections[0]);
    let hand = tx.state().hand_cards(player);
    if !hand.contains(&card) {
        return Err(EffectError::Illegal(
            "巧变 card is no longer in hand".into(),
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
            cards: vec![card],
            to: discard,
            from: Some(hand_zone),
        }],
    )?;
    match phase {
        0 | 3 => qiaobian_skip_phase(tx, player),
        1 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player && tx.state().hand_count(candidate) > 0)
                .collect();
            if options.is_empty() {
                return qiaobian_skip_phase(tx, player);
            }
            crate::effect::open_choice(
                tx,
                player,
                0,
                2,
                true,
                options.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_QIAOBIAN_DRAW_TARGETS,
                    2,
                    u32::from(phase),
                    player.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        2 => {
            let mut options = Vec::new();
            for candidate in tx.state().players.alive_by_seat() {
                for zone_kind in [crate::state::ZoneKind::Equip, crate::state::ZoneKind::Judge] {
                    if let Some(zone) = tx.state().zones.find(zone_kind, candidate) {
                        if let Some(zone) = tx.state().zones.get(zone) {
                            options.extend(zone.cards.iter().map(|card| card.raw()));
                        }
                    }
                }
            }
            if options.is_empty() {
                return qiaobian_skip_phase(tx, player);
            }
            crate::effect::open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                vec![
                    crate::effect::RESOLVE_QIAOBIAN_PLAY_CARD,
                    2,
                    u32::from(phase),
                    player.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown 巧变 phase".into())),
    }
}

/// 巧变（摸牌阶段收益）：获至多两名其他角色各一张手牌，逐名开选牌。
pub(crate) fn resolve_qiaobian_draw_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let phase = *payload.get(2).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "巧变 target choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return qiaobian_skip_phase(tx, player);
    }
    if selections.len() > 2 {
        return Err(EffectError::Illegal("巧变 at most two targets".into()));
    }
    let targets: Vec<_> = selections.iter().map(|&raw| PlayerId::new(raw)).collect();
    if targets.iter().any(|&target| {
        target == player
            || !tx.state().players.get(target).is_some_and(|p| p.alive)
            || tx.state().hand_count(target) == 0
    }) || targets
        .iter()
        .collect::<std::collections::HashSet<_>>()
        .len()
        != targets.len()
    {
        return Err(EffectError::Illegal(
            "巧变 targets are no longer legal".into(),
        ));
    }
    let first = targets[0];
    let mut choice_payload = vec![
        crate::effect::RESOLVE_QIAOBIAN_DRAW_PICK,
        3,
        u32::from(phase),
        player.raw(),
        skill.raw(),
        targets.len() as u32,
    ];
    choice_payload.extend(targets.iter().map(|target| target.raw()));
    choice_payload.push(0);
    crate::effect::open_choice(
        tx,
        first,
        1,
        1,
        false,
        tx.state()
            .hand_cards(first)
            .into_iter()
            .map(engine_api::CardId::raw)
            .collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        choice_payload,
    )
    .map(|_| ())
}

/// 巧变（摸牌阶段收益）：目标交一张手牌，随后进入下一目标或跳过阶段。
pub(crate) fn resolve_qiaobian_draw_pick(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let phase = *payload.get(2).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let count = *payload.get(5).unwrap_or(&0) as usize;
    let index = *payload.get(6 + count).unwrap_or(&0) as usize;
    let target = PlayerId::new(*payload.get(6 + index).unwrap_or(&0));
    if observer != target {
        return Err(EffectError::Illegal(
            "巧变 pick must be made by the giving target".into(),
        ));
    }
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "巧变 requires the target to give a card".into(),
        ));
    }
    let card = engine_api::CardId::new(selections[0]);
    let hand = tx.state().hand_cards(target);
    if !hand.contains(&card) {
        return Err(EffectError::Illegal(
            "巧变 given card is no longer in hand".into(),
        ));
    }
    let hand_zone = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let owner_hand = tx
        .state()
        .zones
        .find(crate::state::ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no owner hand zone".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: owner_hand,
            from: Some(hand_zone),
        }],
    )?;
    if index + 1 >= count {
        return qiaobian_skip_phase(tx, player);
    }
    let next = PlayerId::new(*payload.get(6 + index + 1).unwrap_or(&0));
    let mut choice_payload = vec![
        crate::effect::RESOLVE_QIAOBIAN_DRAW_PICK,
        3,
        u32::from(phase),
        player.raw(),
        skill.raw(),
        count as u32,
    ];
    choice_payload.extend((0..count).map(|i| *payload.get(6 + i).unwrap_or(&0)));
    choice_payload.push((index + 1) as u32);
    crate::effect::open_choice(
        tx,
        next,
        1,
        1,
        false,
        tx.state()
            .hand_cards(next)
            .into_iter()
            .map(engine_api::CardId::raw)
            .collect(),
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::Custom,
        choice_payload,
    )
    .map(|_| ())
}

/// 巧变（出牌阶段收益）：选择场上的一张牌后，选择移动目的地。
pub(crate) fn resolve_qiaobian_play_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let phase = *payload.get(2).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "巧变 card choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return qiaobian_skip_phase(tx, player);
    }
    let card = engine_api::CardId::new(selections[0]);
    let Some(value) = tx.state().cards.get(card) else {
        return Err(EffectError::Illegal("巧变 field card vanished".into()));
    };
    if value.owner == PlayerId::NONE
        || !tx.state().players.get(value.owner).is_some_and(|p| p.alive)
    {
        return Err(EffectError::Illegal("巧变 field card owner is gone".into()));
    }
    let in_field = tx
        .state()
        .zones
        .get(value.zone)
        .map(|zone| {
            matches!(
                zone.kind,
                crate::state::ZoneKind::Equip | crate::state::ZoneKind::Judge
            ) && zone.cards.contains(&card)
        })
        .unwrap_or(false);
    if !in_field {
        return Err(EffectError::Illegal(
            "巧变 card is no longer on the field".into(),
        ));
    }
    let owner = value.owner;
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != owner)
        .collect();
    if options.is_empty() {
        return qiaobian_skip_phase(tx, player);
    }
    crate::effect::open_choice(
        tx,
        player,
        0,
        1,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        engine_api::ChoiceKind::SelectTargets,
        crate::state::ContinuationKind::Custom,
        vec![
            crate::effect::RESOLVE_QIAOBIAN_PLAY_TARGET,
            3,
            u32::from(phase),
            player.raw(),
            skill.raw(),
            card.raw(),
        ],
    )
    .map(|_| ())
}

/// 巧变（出牌阶段收益）：按牌类别移动到目的地角色的同类别区域，随后跳过阶段。
pub(crate) fn resolve_qiaobian_play_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let phase = *payload.get(2).unwrap_or(&0);
    let player = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let card = engine_api::CardId::new(*payload.get(5).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "巧变 move choice must be made by the owner".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return qiaobian_skip_phase(tx, player);
    }
    let dest = PlayerId::new(selections[0]);
    let Some(value) = tx.state().cards.get(card) else {
        return Err(EffectError::Illegal("巧变 field card vanished".into()));
    };
    if dest == value.owner || !tx.state().players.get(dest).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal(
            "巧变 destination is no longer legal".into(),
        ));
    }
    let zone_kind = if value.kind.is_delayed_trick() {
        crate::state::ZoneKind::Judge
    } else {
        crate::state::ZoneKind::Equip
    };
    let dest_zone = tx
        .state()
        .zones
        .find(zone_kind, dest)
        .ok_or_else(|| EffectError::Illegal("no destination zone".into()))?;
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: dest_zone,
            from: Some(value.zone),
        }],
    )?;
    qiaobian_skip_phase(tx, player)
}

pub(crate) fn resolve_resolve_drlt_congjian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, stage(, target)]
    if payload.len() < 4 {
        return Err(EffectError::EngineFault("bad Congjian continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Congjian observer mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let skill = engine_api::SkillId::new(payload[2]);
    let stage = payload[3];
    if stage == 0 {
        if selections.len() != 1 {
            return Err(EffectError::Illegal(
                "Congjian recipient selection invalid".into(),
            ));
        }
        let target = PlayerId::new(selections[0]);
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
            return Err(EffectError::Illegal("Congjian context changed".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDrltCongjian {
                player,
                targets: Vec::new(),
                card: engine_api::CardId::NONE,
                target,
                stage: 1,
                skill,
            }],
        )
        .map(|_| ());
    }
    if stage == 1 {
        if selections.len() != 1 {
            return Err(EffectError::Illegal(
                "Congjian card selection invalid".into(),
            ));
        }
        let card = CardId::new(selections[0]);
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDrltCongjian {
                player,
                targets: Vec::new(),
                card,
                target: PlayerId::new(payload[4]),
                stage: 2,
                skill,
            }],
        )
        .map(|_| ());
    }
    Err(EffectError::EngineFault(
        "bad Congjian continuation stage".into(),
    ))
}

pub(crate) fn resolve_resolve_xinkuangfu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, target, sha_target, stage]
    if payload.len() != 6 {
        return Err(EffectError::EngineFault(
            "bad Xinkuangfu continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal("Xinkuangfu observer mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Xinkuangfu context changed".into()));
    }
    let stage = payload[5];
    match stage {
        1 => {
            // The owner picked the equipment target.
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinkuangfu target selection invalid".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            if target != player
                && (!tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                    || crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        target,
                        rule_api::TargetCardScope::Equip,
                    )
                    .is_empty())
            {
                return Err(EffectError::Illegal(
                    "Xinkuangfu target has no equipment".into(),
                ));
            }
            return run_effects(
                tx,
                vec![EffectRequest::ResolveXinkuangfu {
                    player,
                    target,
                    sha_target: PlayerId::NONE,
                    stage: 2,
                    skill: engine_api::SkillId::new(payload[2]),
                }],
            )
            .map(|_| ());
        }
        2 => {
            // The owner picked the equipment card to discard.
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Xinkuangfu equipment selection invalid".into(),
                ));
            }
            let target = PlayerId::new(payload[3]);
            let card = CardId::new(selections[0]);
            let allowed = crate::query::discardable_target_cards(
                tx.state(),
                player,
                if target.is_none() || target == player {
                    player
                } else {
                    target
                },
                rule_api::TargetCardScope::Equip,
            );
            if !allowed.contains(&card) {
                return Err(EffectError::Illegal(
                    "Xinkuangfu equipment card invalid".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Xinkuangfu card missing".into()))?;
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
            )?;
            return run_effects(
                tx,
                vec![EffectRequest::ResolveXinkuangfu {
                    player,
                    target,
                    sha_target: PlayerId::NONE,
                    stage: 3,
                    skill: engine_api::SkillId::new(payload[2]),
                }],
            )
            .map(|_| ());
        }
        3 => {
            // The owner picked the Sha target; run the Sha and branch on damage.
            if selections.len() != 1 {
                return Err(EffectError::Illegal("Xinkuangfu Sha target invalid".into()));
            }
            let sha_target = PlayerId::new(selections[0]);
            if sha_target == player
                || !tx
                    .state()
                    .players
                    .get(sha_target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Xinkuangfu Sha target stale".into()));
            }
            let equip_target = PlayerId::new(payload[3]);
            let owner_hp_before = tx
                .state()
                .players
                .get(sha_target)
                .map(|p| p.hp)
                .unwrap_or(0);
            if let Err(crate::effect::EffectError::ChoiceRequested) = run_effects(
                tx,
                vec![EffectRequest::UseVirtualCardOn {
                    player,
                    kind: crate::state::CardKind::Sha.tag(),
                    target: sha_target,
                }],
            ) {
                return Ok(());
            }
            let owner_hp_after = tx
                .state()
                .players
                .get(sha_target)
                .map(|p| p.hp)
                .unwrap_or(0);
            let damaged = owner_hp_after < owner_hp_before;
            let mut tail = Vec::new();
            if equip_target == player {
                if damaged {
                    tail.push(EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 2,
                        skill: engine_api::SkillId::new(payload[2]),
                    });
                }
            } else if !damaged {
                // The owner discards two hand cards.
                let options: Vec<_> =
                    crate::query::discardable_hand_equip_cards(tx.state(), player)
                        .into_iter()
                        .map(CardId::raw)
                        .collect();
                if options.len() < 2 {
                    return Ok(());
                }
                let mut next_payload = vec![
                    crate::effect::RESOLVE_XINKUANGFU_CUSTOM_CHOICE,
                    player.raw(),
                    engine_api::SkillId::new(payload[2]).raw(),
                    equip_target.raw(),
                    sha_target.raw(),
                    4,
                ];
                return crate::effect::open_choice(
                    tx,
                    player,
                    2,
                    2,
                    false,
                    options,
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    next_payload,
                )
                .map(|_| ());
            }
            run_effects(tx, tail).map(|_| ())
        }
        4 => {
            // Discard two hand cards.
            if selections.len() != 2 {
                return Err(EffectError::Illegal(
                    "Xinkuangfu hand discard invalid".into(),
                ));
            }
            let allowed = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let mut moves = Vec::new();
            for raw in selections {
                let card = CardId::new(*raw);
                if !allowed.contains(&card) {
                    return Err(EffectError::Illegal(
                        "Xinkuangfu hand discard card invalid".into(),
                    ));
                }
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("Xinkuangfu card missing".into()))?;
                moves.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::DiscardPile, PlayerId::NONE)
                        .unwrap(),
                    from: Some(from),
                });
            }
            run_effects(tx, moves).map(|_| ())
        }
        _ => Err(EffectError::Illegal("Xinkuangfu stage out of range".into())),
    }
}

pub(crate) fn resolve_resolve_dcchiying(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, target, stage, accum_len, accum...] (+ picked at stage 3)
    if payload.len() < 6 {
        return Err(EffectError::EngineFault(
            "bad Dcchiying continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[3]);
    let stage = payload[4];
    let accum_len = payload[5] as usize;
    if payload.len() < 6 + accum_len {
        return Err(EffectError::EngineFault(
            "bad Dcchiying accumulated-card payload".into(),
        ));
    }
    let mut accum: Vec<CardId> = payload[6..6 + accum_len]
        .iter()
        .map(|id| CardId::new(*id))
        .collect();
    if stage == 3 {
        // Stage 3: the target discarded a card from the picked player.
        if payload.len() != 7 + accum_len {
            return Err(EffectError::EngineFault(
                "bad Dcchiying stage-3 payload".into(),
            ));
        }
        if observer != target || cancelled || selections.len() != 1 {
            return Err(EffectError::Illegal(
                "Dcchiying card discard context changed".into(),
            ));
        }
        let picked = PlayerId::new(payload[6 + accum_len]);
        let card = CardId::new(selections[0]);
        let allowed = crate::query::discardable_target_cards(
            tx.state(),
            target,
            picked,
            rule_api::TargetCardScope::HandEquip,
        );
        if !allowed.contains(&card) {
            return Err(EffectError::Illegal(
                "Dcchiying discarded card invalid".into(),
            ));
        }
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Dcchiying card missing".into()))?;
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
        )?;
        accum.push(card);
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDcchiying {
                player,
                target,
                stage: 2,
                accum,
                skill: engine_api::SkillId::new(payload[2]),
            }],
        )
        .map(|_| ());
    }
    if payload.len() != 6 + accum_len {
        return Err(EffectError::EngineFault(
            "bad Dcchiying payload length".into(),
        ));
    }
    if stage == 1 {
        // The owner chose the target.
        if observer != player {
            return Err(EffectError::Illegal(
                "Dcchiying target observer mismatch".into(),
            ));
        }
        if cancelled {
            return Ok(());
        }
        if selections.len() != 1 {
            return Err(EffectError::Illegal(
                "Dcchiying target selection invalid".into(),
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
            return Err(EffectError::Illegal("Dcchiying target stale".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDcchiying {
                player,
                target,
                stage: 2,
                accum: Vec::new(),
                skill: engine_api::SkillId::new(payload[2]),
            }],
        )
        .map(|_| ());
    }
    // Stage 2: the target picked a player (or cancelled).
    if observer != target {
        return Err(EffectError::Illegal(
            "Dcchiying discard observer mismatch".into(),
        ));
    }
    if cancelled {
        return crate::effect::dcchiying_gain_basics(tx, target, accum).map(|_| ());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Dcchiying pick selection invalid".into(),
        ));
    }
    let picked = PlayerId::new(selections[0]);
    if picked == target
        || !tx
            .state()
            .players
            .get(picked)
            .is_some_and(|participant| participant.alive)
        || !crate::effect::dcchiying_in_range(tx.state(), target, picked)
    {
        return Err(EffectError::Illegal("Dcchiying picked player stale".into()));
    }
    // The target picks one card of the picked player to discard.
    let options: Vec<_> = crate::query::discardable_target_cards(
        tx.state(),
        target,
        picked,
        rule_api::TargetCardScope::HandEquip,
    )
    .into_iter()
    .map(CardId::raw)
    .collect();
    if options.is_empty() {
        return Ok(());
    }
    let mut next_payload = vec![
        crate::effect::RESOLVE_DCCHIYING_CUSTOM_CHOICE,
        player.raw(),
        engine_api::SkillId::new(payload[2]).raw(),
        target.raw(),
        3,
        accum.len() as u32,
    ];
    next_payload.extend(accum.iter().map(|card| card.raw()));
    next_payload.push(picked.raw());
    crate::effect::open_choice(
        tx,
        target,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        next_payload,
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_xuxie(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, stage, count, targets..., index]
    if payload.len() < 6 {
        return Err(EffectError::EngineFault("bad Xuxie continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Xuxie choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Xuxie context changed".into()));
    }
    let stage = payload[3];
    let count = payload[4] as usize;
    let expected = if stage == 0 { 5 + count } else { 5 + count + 1 };
    if payload.len() != expected {
        return Err(EffectError::EngineFault("bad Xuxie payload length".into()));
    }
    let targets: Vec<PlayerId> = payload[5..5 + count]
        .iter()
        .map(|id| PlayerId::new(*id))
        .collect();
    if stage == 0 {
        if selections.len() != 1 {
            return Err(EffectError::Illegal(
                "Xuxie branch selection invalid".into(),
            ));
        }
        let branch = *selections.first().unwrap();
        if branch == 1 {
            let draws: Vec<_> = targets
                .iter()
                .map(|target| EffectRequest::DrawCardsFromSkill {
                    player: *target,
                    count: 1,
                    skill: engine_api::SkillId::new(payload[2]),
                })
                .collect();
            return run_effects(tx, draws).map(|_| ());
        }
        if branch != 0 {
            return Err(EffectError::Illegal("Xuxie branch out of range".into()));
        }
        // Discard branch: start the per-target loop with the marker choice.
        let mut next_payload = payload.to_vec();
        next_payload[3] = 1;
        next_payload.push(0); // current index
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            next_payload,
        )
        .map(|_| ());
    }
    // Stage 1: the discard loop. index = payload[5 + count].
    let index = payload[5 + count] as usize;
    if index >= count {
        return Ok(());
    }
    let target = targets[index];
    let alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    let discardable = if alive {
        crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
    } else {
        Vec::new()
    };
    // Marker selection [0] or an unavailable target: advance.
    let is_marker = selections.len() == 1 && *selections.first().unwrap() == 0;
    if is_marker || discardable.is_empty() {
        let mut next_payload = payload.to_vec();
        next_payload[5 + count] = (index + 1) as u32;
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            next_payload,
        )
        .map(|_| ());
    }
    // A card selection: discard it, then advance.
    if selections.len() != 1 || !discardable.contains(&CardId::new(selections[0])) {
        return Err(EffectError::Illegal("Xuxie discard card invalid".into()));
    }
    let card = CardId::new(selections[0]);
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("Xuxie card missing".into()))?;
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
    )?;
    let mut next_payload = payload.to_vec();
    next_payload[5 + count] = (index + 1) as u32;
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![0],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        next_payload,
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_xizhen(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // Stage 0 payload: [magic, player, skill] -> the owner picked a target.
    // Stage 1 payload: [magic, player, skill, target] -> the owner picked the branch.
    if payload.len() == 3 {
        let player = PlayerId::new(payload[1]);
        if observer != player {
            return Err(EffectError::Illegal(
                "Xizhen choice observer mismatch".into(),
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
            return Err(EffectError::Illegal("Xizhen context changed".into()));
        }
        let target = PlayerId::new(selections[0]);
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal("Xizhen target stale".into()));
        }
        // Open the branch choice: 1 = Sha, 2 = Duel.
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![1, 2],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                crate::effect::RESOLVE_XIZHEN_CUSTOM_CHOICE,
                player.raw(),
                engine_api::SkillId::new(payload[2]).raw(),
                target.raw(),
            ],
        )
        .map(|_| ());
    }
    if payload.len() == 4 {
        let player = PlayerId::new(payload[1]);
        let target = PlayerId::new(payload[3]);
        if observer != player {
            return Err(EffectError::Illegal(
                "Xizhen branch observer mismatch".into(),
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
            return Err(EffectError::Illegal("Xizhen branch context changed".into()));
        }
        let branch = *selections.first().unwrap();
        let kind = match branch {
            1 => crate::state::CardKind::Sha,
            2 => crate::state::CardKind::TrickJuedou,
            _ => return Err(EffectError::Illegal("Xizhen branch out of range".into())),
        };
        return run_effects(
            tx,
            vec![EffectRequest::UseVirtualCardOn {
                player,
                kind: kind.tag(),
                target,
            }],
        )
        .map(|_| ());
    }
    Err(EffectError::EngineFault("bad Xizhen continuation".into()))
}

pub(crate) fn resolve_resolve_dcmoyu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("bad Dcmoyu continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Dcmoyu choice observer mismatch".into(),
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
        return Err(EffectError::Illegal("Dcmoyu context changed".into()));
    }
    let target = PlayerId::new(selections[0]);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Dcmoyu target stale".into()));
    }
    let skill = engine_api::SkillId::new(payload[2]);
    // Gain count: 1, plus 1 when the add-on mark from a previous use is set.
    let add = tx
        .state()
        .players
        .get(player)
        .map(|p| {
            p.marks
                .iter()
                .find(|(mark, _)| *mark == crate::effect::DCMOYU_ADD_MARK)
                .map(|(_, count)| *count)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let count = 1 + add.min(1) as usize;
    let available = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquipJudge,
    );
    let mut moves = Vec::new();
    for card in available.into_iter().take(count) {
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Dcmoyu card missing".into()))?;
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
    if add > 0 {
        moves.push(EffectRequest::RemoveMark {
            player,
            mark: engine_api::MarkDefId::new(crate::effect::DCMOYU_ADD_MARK),
            amount: add.max(0) as u32,
        });
    }
    run_effects(tx, moves)?;
    // The target uses a virtual Sha on the owner; snapshot HP for the outcome.
    let owner_hp_before = tx.state().players.get(player).map(|p| p.hp).unwrap_or(0);
    if let Err(crate::effect::EffectError::ChoiceRequested) = run_effects(
        tx,
        vec![EffectRequest::UseVirtualCardOn {
            player: target,
            kind: crate::state::CardKind::Sha.tag(),
            target: player,
        }],
    ) {
        // The Sha opened a response window; skip the branch outcome.
        return Ok(());
    }
    let owner_hp_after = tx.state().players.get(player).map(|p| p.hp).unwrap_or(0);
    let mut tail = vec![EffectRequest::AddMark {
        player,
        mark: engine_api::MarkDefId::new(target.raw()),
        delta: 1,
    }];
    if owner_hp_after < owner_hp_before {
        // Damaged: ban the active skill until the turn ends.
        tail.push(EffectRequest::BanActiveUntilTurnEnd { player, skill });
    } else {
        // Not damaged: the next use gains one more card.
        tail.push(EffectRequest::AddMark {
            player,
            mark: engine_api::MarkDefId::new(crate::effect::DCMOYU_ADD_MARK),
            delta: 1,
        });
    }
    run_effects(tx, tail).map(|_| ())
}

pub(crate) fn resolve_resolve_rekuanshi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Rekuanshi continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    if observer != player {
        return Err(EffectError::Illegal(
            "Rekuanshi choice observer mismatch".into(),
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
        return Err(EffectError::Illegal("Rekuanshi context changed".into()));
    }
    let target = PlayerId::new(selections[0]);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Rekuanshi target stale".into()));
    }
    // Mark the watched player; the damage trigger resolves the recovery.
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(crate::effect::REKUANSHI_WATCH_MARK),
            delta: 1,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_sxrmlanjiao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    // payload: [magic, player, skill, stage, 0, target, view_len, view...]
    if payload.len() < 7 {
        return Err(EffectError::EngineFault(
            "bad Sxrmlanjiao continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let stage = payload[3];
    let target = PlayerId::new(payload[5]);
    let view_len = payload[6] as usize;
    if payload.len() != 7 + view_len {
        return Err(EffectError::EngineFault(
            "bad Sxrmlanjiao payload length".into(),
        ));
    }
    let view: Vec<CardId> = payload[7..].iter().map(|id| CardId::new(*id)).collect();
    if stage == 0 {
        // Target choice made by the owner.
        if observer != player {
            return Err(EffectError::Illegal(
                "Sxrmlanjiao target choice observer mismatch".into(),
            ));
        }
        if cancelled || selections.len() != 1 {
            return Ok(());
        }
        let target = PlayerId::new(selections[0]);
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            || tx.state().hand_count(target) < 2
        {
            return Err(EffectError::Illegal("Sxrmlanjiao target stale".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveSxrmlanjiao {
                player,
                target,
                stage: 1,
                view: Vec::new(),
                skill: engine_api::SkillId::new(payload[2]),
            }],
        )
        .map(|_| ());
    }
    // Stage 1: the target's rechoose choice.
    if observer != target {
        return Err(EffectError::Illegal(
            "Sxrmlanjiao rechoose observer mismatch".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Sxrmlanjiao context changed".into()));
    }
    let rechoose = if cancelled {
        false
    } else if selections.len() == 1 {
        *selections.first().unwrap() == 1
    } else {
        false
    };
    if rechoose {
        // The target loses 1 HP and draws one, then the owner re-views.
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            return Ok(());
        }
        return run_effects(
            tx,
            vec![
                EffectRequest::DrawCardsFromSkill {
                    player: target,
                    count: 1,
                    skill: engine_api::SkillId::new(payload[2]),
                },
                EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                },
            ],
        )
        .and_then(|_| {
            run_effects(
                tx,
                vec![EffectRequest::ResolveSxrmlanjiao {
                    player,
                    target,
                    stage: 1,
                    view: Vec::new(),
                    skill: engine_api::SkillId::new(payload[2]),
                }],
            )
        })
        .map(|_| ());
    }
    // Stop: the owner gains the viewed cards; mark the target as used this turn.
    if view.is_empty() {
        return Ok(());
    }
    let mut moves = Vec::new();
    for card in &view {
        if !tx.state().hand_cards(target).contains(card) {
            continue;
        }
        let from = tx
            .state()
            .cards
            .get(*card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Sxrmlanjiao card missing".into()))?;
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
    moves.push(EffectRequest::AddMark {
        player,
        mark: engine_api::MarkDefId::new(target.raw()),
        delta: 1,
    });
    run_effects(tx, moves).map(|_| ())
}
