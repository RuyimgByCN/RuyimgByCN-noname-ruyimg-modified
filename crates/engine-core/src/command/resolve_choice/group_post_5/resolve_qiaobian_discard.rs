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
