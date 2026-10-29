//! Effect primitives for `potjixi_targets` and related helpers.

use super::*;

pub(crate) fn begin_pottaishici_response_cost(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| {
            skill.owner == player
                && skill.state_blob.len() >= 14
                && skill.state_blob[8] == 1
                && u32::from_le_bytes(skill.state_blob[9..13].try_into().unwrap()) == card.raw()
        })
        .ok_or_else(|| EffectError::EngineFault("势战烈响应状态丢失".into()))?;
    let index = skill.state_blob[13] as usize;
    let skill = skill.id;
    let targets = crate::command::play::saved_sha_targets(tx, player, card)?;
    open_pottaishici_response_cost(tx, player, card, skill, &targets, index)
}

pub(crate) fn resolve_pottaishici_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&POTTAISHICI_CUSTOM_CHOICE) || payload.len() < 2 {
        return Err(EffectError::EngineFault("势太史慈续体损坏".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() != 7 {
                return Err(EffectError::EngineFault("势战烈目标续体损坏".into()));
            }
            let player = PlayerId::new(payload[2]);
            let card = CardId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let hanzhan_def = engine_api::SkillDefId::new(payload[5]);
            let zhanlie_def = engine_api::SkillDefId::new(payload[6]);
            if observer != player || cancelled {
                return remove_pottaishici_virtual_card(tx, card);
            }
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let distinct = targets
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == targets.len();
            let base_limit = crate::query::sha_target_limit_for(tx.state(), player) as usize;
            if targets.is_empty()
                || targets.len() > base_limit
                || !distinct
                || targets.iter().any(|target| {
                    crate::command::play::validate_sha_target_with_options(
                        tx, player, card, *target, false,
                    )
                    .is_err()
                })
            {
                return Err(EffectError::Illegal("势战烈目标已变化".into()));
            }
            let marks = pottaishici_mark_count(tx, player, zhanlie_def);
            if marks > 0 {
                run_effects(
                    tx,
                    vec![EffectRequest::RemoveMark {
                        player,
                        mark: engine_api::MarkDefId::new(zhanlie_def.raw()),
                        amount: marks,
                    }],
                )?;
            }
            let max = (marks / 2).min(4) as u8;
            if max == 0 {
                return complete_pottaishici_sha(tx, player, card, skill, &targets, 0, &[]);
            }
            let mut next = vec![
                POTTAISHICI_CUSTOM_CHOICE,
                1,
                player.raw(),
                card.raw(),
                skill.raw(),
                hanzhan_def.raw(),
                zhanlie_def.raw(),
                targets.len() as u32,
            ];
            next.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                0,
                max,
                true,
                vec![1, 2, 4, 8],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 8 {
                return Err(EffectError::EngineFault("势战烈选项续体损坏".into()));
            }
            let player = PlayerId::new(payload[2]);
            let card = CardId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let hanzhan_def = engine_api::SkillDefId::new(payload[5]);
            let zhanlie_def = engine_api::SkillDefId::new(payload[6]);
            let count = payload[7] as usize;
            if observer != player || payload.len() != 8 + count {
                return Err(EffectError::Illegal("势战烈选项上下文已变化".into()));
            }
            let targets: Vec<_> = payload[8..].iter().copied().map(PlayerId::new).collect();
            let selected = if cancelled { &[][..] } else { selections };
            let distinct = selected
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == selected.len();
            if !distinct || selected.iter().any(|choice| ![1, 2, 4, 8].contains(choice)) {
                return Err(EffectError::Illegal("势战烈选项非法".into()));
            }
            let choices = selected.iter().fold(0, |mask, choice| mask | choice);
            if choices & 1 != 0 {
                let options: Vec<_> = tx
                    .state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|target| !targets.contains(target))
                    .filter(|target| {
                        crate::command::play::validate_sha_target_with_options(
                            tx, player, card, *target, false,
                        )
                        .is_ok()
                    })
                    .collect();
                if !options.is_empty() {
                    let mut next = vec![
                        POTTAISHICI_CUSTOM_CHOICE,
                        2,
                        player.raw(),
                        card.raw(),
                        skill.raw(),
                        hanzhan_def.raw(),
                        zhanlie_def.raw(),
                        choices,
                        targets.len() as u32,
                    ];
                    next.extend(targets.iter().map(|target| target.raw()));
                    return open_choice(
                        tx,
                        player,
                        0,
                        1,
                        true,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        next,
                    )
                    .map(|_| ());
                }
            }
            complete_pottaishici_sha(tx, player, card, skill, &targets, choices, &[])
        }
        2 => {
            if payload.len() < 9 {
                return Err(EffectError::EngineFault("势战烈额外目标续体损坏".into()));
            }
            let player = PlayerId::new(payload[2]);
            let card = CardId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let choices = payload[7];
            let count = payload[8] as usize;
            if observer != player || payload.len() != 9 + count {
                return Err(EffectError::Illegal("势战烈额外目标上下文已变化".into()));
            }
            let mut targets: Vec<_> = payload[9..].iter().copied().map(PlayerId::new).collect();
            if !cancelled {
                if selections.len() != 1 {
                    return Err(EffectError::Illegal("势战烈额外目标非法".into()));
                }
                let extra = PlayerId::new(selections[0]);
                if targets.contains(&extra)
                    || crate::command::play::validate_sha_target_with_options(
                        tx, player, card, extra, false,
                    )
                    .is_err()
                {
                    return Err(EffectError::Illegal("势战烈额外目标已变化".into()));
                }
                targets.push(extra);
            }
            complete_pottaishici_sha(tx, player, card, skill, &targets, choices, &[])
        }
        3 => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault("势战烈响应续体损坏".into()));
            }
            let player = PlayerId::new(payload[2]);
            let card = CardId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let count = payload[5] as usize;
            let index = payload[6] as usize;
            if count == 0 || index >= count || payload.len() != 7 + count {
                return Err(EffectError::EngineFault("势战烈响应续体损坏".into()));
            }
            let targets: Vec<_> = payload[7..].iter().copied().map(PlayerId::new).collect();
            if observer != targets[index] {
                return Err(EffectError::Illegal("势战烈响应者已变化".into()));
            }
            if cancelled || selections.is_empty() {
                crate::command::play::make_saved_sha_target_unrespondable(
                    tx,
                    player,
                    card,
                    targets[index],
                )?;
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    if instance.state_blob.len() >= 14 {
                        instance.state_blob[13] = (index + 1).min(u8::MAX as usize) as u8;
                    }
                }
                return open_pottaishici_response_cost(
                    tx,
                    player,
                    card,
                    skill,
                    &targets,
                    index + 1,
                );
            } else {
                if selections.len() != 1 {
                    return Err(EffectError::Illegal("势战烈响应弃牌非法".into()));
                }
                let selected = CardId::new(selections[0]);
                if !crate::query::discardable_hand_equip_cards(tx.state(), observer)
                    .contains(&selected)
                {
                    return Err(EffectError::Illegal("势战烈响应弃牌已变化".into()));
                }
                let from = tx
                    .state()
                    .cards
                    .get(selected)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("势战烈响应弃牌不存在".into()))?;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    if instance.state_blob.len() >= 14 {
                        instance.state_blob[13] = (index + 1).min(u8::MAX as usize) as u8;
                    }
                }
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::MoveCardsWithReason {
                            cards: vec![selected],
                            to: discard,
                            from: Some(from),
                            reason: CardMoveReason::Discard,
                        },
                        EffectRequest::ResolvePottaishici {
                            player,
                            target: player,
                            action: 7,
                            cards: vec![card],
                            hanzhan_def: engine_api::SkillDefId::NONE,
                            zhanlie_def: engine_api::SkillDefId::NONE,
                            skill,
                        },
                    ],
                )
                .map(|_| ());
            }
        }
        4 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault("势振锋分支续体损坏".into()));
            }
            let player = PlayerId::new(payload[2]);
            let hanzhan_def = engine_api::SkillDefId::new(payload[4]);
            let zhanlie_def = engine_api::SkillDefId::new(payload[5]);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("势振锋选择非法".into()));
            }
            match selections[0] {
                0 => run_effects(
                    tx,
                    vec![EffectRequest::RecoverHp {
                        source: player,
                        player,
                        amount: 2,
                    }],
                )
                .map(|_| ()),
                1 => {
                    let has_hanzhan = pottaishici_skill_id(tx, player, hanzhan_def).is_some();
                    let has_zhanlie = pottaishici_skill_id(tx, player, zhanlie_def).is_some();
                    let mut options = Vec::new();
                    if has_hanzhan {
                        options.extend([11, 12, 13]);
                    }
                    if has_zhanlie {
                        options.extend([21, 22, 23]);
                    }
                    let max = u8::from(has_hanzhan) + u8::from(has_zhanlie);
                    open_choice(
                        tx,
                        player,
                        1,
                        max,
                        false,
                        options,
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        vec![
                            POTTAISHICI_CUSTOM_CHOICE,
                            5,
                            player.raw(),
                            hanzhan_def.raw(),
                            zhanlie_def.raw(),
                        ],
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal("势振锋分支非法".into())),
            }
        }
        5 => {
            if payload.len() != 5 {
                return Err(EffectError::EngineFault("势振锋改写续体损坏".into()));
            }
            let player = PlayerId::new(payload[2]);
            let hanzhan_def = engine_api::SkillDefId::new(payload[3]);
            let zhanlie_def = engine_api::SkillDefId::new(payload[4]);
            if observer != player || cancelled || selections.is_empty() || selections.len() > 2 {
                return Err(EffectError::Illegal("势振锋改写非法".into()));
            }
            let mut seen_hanzhan = false;
            let mut seen_zhanlie = false;
            for choice in selections {
                let (skill_def, mode, seen) = match *choice {
                    11..=13 => (hanzhan_def, (*choice - 10) as u8, &mut seen_hanzhan),
                    21..=23 => (zhanlie_def, (*choice - 20) as u8, &mut seen_zhanlie),
                    _ => return Err(EffectError::Illegal("势振锋改写项非法".into())),
                };
                if *seen {
                    return Err(EffectError::Illegal("势振锋同技能重复改写".into()));
                }
                *seen = true;
                let skill = pottaishici_skill_id(tx, player, skill_def)
                    .ok_or_else(|| EffectError::Illegal("势振锋技能已失去".into()))?;
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    if instance.state_blob.len() < 8 {
                        instance.state_blob.resize(8, 0);
                    }
                    instance.state_blob[0] = mode;
                }
            }
            Ok(())
        }
        _ => Err(EffectError::EngineFault("未知势太史慈续体".into())),
    }
}

pub(crate) fn resolve_nzry_feijun(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    branch: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let hand_more_exists = |tx: &Transaction| {
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .any(|candidate| {
                candidate != player
                    && tx.state().hand_count(candidate) > tx.state().hand_count(player)
            })
    };
    let equip_more_exists = |tx: &Transaction| {
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .any(|candidate| {
                candidate != player
                    && tx
                        .state()
                        .zones
                        .find(ZoneKind::Equip, candidate)
                        .map_or(0, |zone| {
                            tx.state().zones.get(zone).map_or(0, |z| z.cards.len())
                        })
                        > tx.state()
                            .zones
                            .find(ZoneKind::Equip, player)
                            .map_or(0, |zone| {
                                tx.state().zones.get(zone).map_or(0, |z| z.cards.len())
                            })
            })
    };
    match stage {
        0 => {
            let hand_option = hand_more_exists(tx);
            let equip_option = equip_more_exists(tx);
            let options: Vec<u32> = match (hand_option, equip_option) {
                (true, true) => vec![0, 1],
                (true, false) => vec![0],
                (false, true) => vec![1],
                (false, false) => return Ok(Vec::new()),
            };
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_NZRY_FEIJUN_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    0, // stage: branch choice
                ],
            )
        }
        1 => {
            // branch in payload[4]; choose the target.
            let candidates: Vec<_> = if branch == 0 {
                tx.state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|candidate| {
                        *candidate != player
                            && tx.state().hand_count(*candidate) > tx.state().hand_count(player)
                    })
                    .collect()
            } else {
                tx.state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|candidate| {
                        *candidate != player
                            && tx
                                .state()
                                .zones
                                .find(ZoneKind::Equip, *candidate)
                                .map_or(0, |zone| {
                                    tx.state().zones.get(zone).map_or(0, |z| z.cards.len())
                                })
                                > tx.state()
                                    .zones
                                    .find(ZoneKind::Equip, player)
                                    .map_or(0, |zone| {
                                        tx.state().zones.get(zone).map_or(0, |z| z.cards.len())
                                    })
                    })
                    .collect()
            };
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                candidates
                    .into_iter()
                    .map(|candidate| candidate.raw())
                    .collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_NZRY_FEIJUN_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    1, // stage: target choice
                    u32::from(branch),
                ],
            )
        }
        2 => {
            // The target chooses the card to give (branch 0) or discard (branch 1).
            if target.is_none()
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let options: Vec<_> = if branch == 0 {
                crate::query::discardable_hand_equip_cards(tx.state(), target)
            } else {
                crate::query::discardable_equipment_cards(tx.state(), player, target)
            }
            .into_iter()
            .map(CardId::raw)
            .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_NZRY_FEIJUN_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    2, // stage: target card choice
                    u32::from(branch),
                    target.raw(),
                ],
            )
        }
        _ => {
            // stage 3: commit give/discard, then arm 兵略 for a first-time target.
            if card.is_none()
                || target.is_none()
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("Feijun discard pile is missing".into()))?;
            let mut events = Vec::new();
            if branch == 0 {
                let target_hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::Illegal("Feijun target hand is missing".into()))?;
                let owner_hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::Illegal("Feijun owner hand is missing".into()))?;
                events.extend(give_cards(tx, vec![card], owner_hand, target_hand)?);
            } else {
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|card_state| card_state.zone)
                    .ok_or_else(|| EffectError::Illegal("Feijun equipment missing".into()))?;
                events.extend(move_cards(tx, vec![card], discard, Some(from))?);
            }
            let first_time = !tx.state().players.get(target).is_some_and(|participant| {
                participant
                    .marks
                    .iter()
                    .any(|(mark, count)| *mark == FEIJUN_TARGET_MARK && *count > 0)
            });
            if first_time {
                events.extend(add_mark(tx, target, FEIJUN_TARGET_MARK, 1)?);
                tx.state_mut()
                    .turn
                    .set_flag(turn_flag::feijun_new_target_by_player(player), 1);
            }
            Ok(events)
        }
    }
}

pub(crate) fn resolve_xinkuangfu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    sha_target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    if stage == 1 {
        let mut options = vec![player.raw()];
        options.extend(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| {
                    candidate != player
                        && !crate::query::discardable_target_cards(
                            tx.state(),
                            player,
                            candidate,
                            rule_api::TargetCardScope::Equip,
                        )
                        .is_empty()
                })
                .map(PlayerId::raw),
        );
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            options,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                RESOLVE_XINKUANGFU_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                target.raw(),
                sha_target.raw(),
                1,
            ],
        );
    }
    if stage == 2 {
        // The owner picks one equipment of the target to discard.
        if target.is_none() || target == player {
            // Self: the owner picks from their own equipment.
            let options: Vec<_> = crate::query::discardable_target_cards(
                tx.state(),
                player,
                player,
                rule_api::TargetCardScope::Equip,
            )
            .into_iter()
            .map(CardId::raw)
            .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            return open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XINKUANGFU_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                    sha_target.raw(),
                    2,
                ],
            );
        }
        let options: Vec<_> = crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::Equip,
        )
        .into_iter()
        .map(CardId::raw)
        .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_XINKUANGFU_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                target.raw(),
                sha_target.raw(),
                2,
            ],
        )
    } else {
        // Stage 3: the owner picks the Sha target.
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| candidate != player)
            .map(PlayerId::raw)
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
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                RESOLVE_XINKUANGFU_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                target.raw(),
                sha_target.raw(),
                3,
            ],
        )
    }
}

pub(crate) fn dcchiying_in_range(
    state: &crate::state::GameState,
    from: PlayerId,
    to: PlayerId,
) -> bool {
    use rule_api::QueryRequest;
    let mut ctx = crate::query::QueryContext::new();
    let range =
        match crate::query::query(state, &QueryRequest::AttackRange { player: from }, &mut ctx) {
            rule_api::QueryResponse::Int(value) => value,
            _ => return false,
        };
    let mut ctx = crate::query::QueryContext::new();
    match crate::query::query(state, &QueryRequest::Distance { from, to }, &mut ctx) {
        rule_api::QueryResponse::Int(distance) => distance <= range,
        _ => false,
    }
}
