//! Effect primitives for `resolve_mbfeijing` and related helpers.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mbfeijing(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    targets: Vec<PlayerId>,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("飞径 skill disappeared".into()))?
                .state_blob
                .clear();
            let Some((clockwise, counterclockwise)) =
                mbfeijing_seat_paths(tx.state(), player, target)
            else {
                return Ok(Vec::new());
            };
            match (clockwise.is_empty(), counterclockwise.is_empty()) {
                (true, true) => Ok(Vec::new()),
                (false, true) => {
                    resolve_mbfeijing(tx, player, target, card, 1, clockwise, Vec::new(), skill)
                }
                (true, false) => resolve_mbfeijing(
                    tx,
                    player,
                    target,
                    card,
                    1,
                    counterclockwise,
                    Vec::new(),
                    skill,
                ),
                (false, false) => open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![1, 2],
                    ChoiceKind::Custom,
                    ContinuationKind::Custom,
                    vec![
                        MBFEIJING_CUSTOM_CHOICE,
                        0,
                        player.raw(),
                        target.raw(),
                        card.raw(),
                        skill.raw(),
                    ],
                ),
            }
        }
        1 => {
            let targets: Vec<_> = targets
                .into_iter()
                .filter(|role| {
                    !crate::query::discardable_target_cards(
                        tx.state(),
                        *role,
                        *role,
                        TargetCardScope::Hand,
                    )
                    .is_empty()
                })
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_mbfeijing_card_choice(tx, player, target, card, skill, targets, cards)
        }
        2 => open_mbfeijing_color_choice(tx, player, card, skill, targets, cards),
        _ => Err(EffectError::ContentFault("bad 飞径 stage".into())),
    }
}

pub(crate) fn open_mbfeijing_card_choice(
    tx: &mut Transaction,
    player: PlayerId,
    original: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let index = cards.len();
    let Some(&chooser) = targets.get(index) else {
        return finalize_mbfeijing_discards(tx, player, card, skill, targets, cards);
    };
    let options =
        crate::query::discardable_target_cards(tx.state(), chooser, chooser, TargetCardScope::Hand);
    if options.is_empty() {
        return Err(EffectError::Illegal(
            "飞径 chooser no longer has a hand card".into(),
        ));
    }
    let mut payload = vec![
        MBFEIJING_CUSTOM_CHOICE,
        1,
        player.raw(),
        original.raw(),
        card.raw(),
        skill.raw(),
        targets.len() as u32,
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
    payload.extend(cards.iter().map(|card| card.raw()));
    open_choice(
        tx,
        chooser,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn continue_mbfeijing_card_choices(
    tx: &mut Transaction,
    player: PlayerId,
    original: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    open_mbfeijing_card_choice(tx, player, original, card, skill, targets, cards)
}

pub(crate) fn finalize_mbfeijing_discards(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut effects = vec![EffectRequest::RevealCards {
        cards: cards.clone(),
        to: Vec::new(),
    }];
    effects.extend(
        targets
            .iter()
            .copied()
            .zip(cards.iter().copied())
            .map(|(owner, card)| EffectRequest::DiscardCards {
                player: owner,
                cards: vec![card],
            }),
    );
    effects.push(EffectRequest::ResolveMbfeijing {
        player,
        target: PlayerId::NONE,
        card,
        stage: 2,
        targets,
        cards,
        skill,
    });
    run_effects(tx, effects)
}

pub(crate) fn open_mbfeijing_color_choice(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut colors: Vec<_> = cards
        .iter()
        .map(|selected| crate::query::effective_card_color_mask(tx.state(), player, *selected))
        .filter(|color| *color != 0)
        .map(u32::from)
        .collect();
    colors.sort_unstable();
    colors.dedup();
    if colors.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        MBFEIJING_CUSTOM_CHOICE,
        2,
        player.raw(),
        card.raw(),
        skill.raw(),
        targets.len() as u32,
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
    payload.extend(cards.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        colors,
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn commit_mbfeijing_color(
    tx: &mut Transaction,
    player: PlayerId,
    sha: CardId,
    skill: engine_api::SkillId,
    color: u8,
    targets: Vec<PlayerId>,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !matches!(color, 0b01 | 0b10) || targets.len() != cards.len() {
        return Err(EffectError::Illegal("invalid 飞径 color selection".into()));
    }
    let mut recorded = Vec::new();
    for (target, selected) in targets.into_iter().zip(cards) {
        let matches = tx.state().cards.get(selected).is_some()
            && crate::query::effective_card_color_mask(tx.state(), player, selected) == color;
        if matches
            && crate::command::play::saved_sha_additional_targets(tx, player, sha)?
                .contains(&target)
        {
            crate::command::play::add_saved_sha_target(tx, player, sha, target)?;
            recorded.push((target, selected));
        }
    }
    let mut blob = b"FJG1".to_vec();
    blob.extend_from_slice(&sha.raw().to_le_bytes());
    blob.extend_from_slice(&(recorded.len() as u32).to_le_bytes());
    for (target, selected) in recorded {
        blob.extend_from_slice(&target.raw().to_le_bytes());
        blob.extend_from_slice(&selected.raw().to_le_bytes());
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("飞径 skill disappeared".into()))?
        .state_blob = blob;
    Ok(Vec::new())
}

pub(crate) fn resolve_mbxiaoge(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let owner_skills = tx
                .state()
                .players
                .get(player)
                .map(|owner| owner.skills.clone())
                .unwrap_or_default();
            let found = owner_skills.into_iter().find_map(|id| {
                let instance = tx.state().skills.get(id)?;
                let blob = &instance.state_blob;
                if blob.get(..4) != Some(b"FJG1")
                    || blob.len() < 12
                    || (blob.len() - 12) % 8 != 0
                    || u32::from_le_bytes(blob[4..8].try_into().ok()?) != card.raw()
                {
                    return None;
                }
                blob[12..]
                    .chunks_exact(8)
                    .enumerate()
                    .find_map(|(index, pair)| {
                        (u32::from_le_bytes(pair[..4].try_into().ok()?) == target.raw()).then(
                            || {
                                (
                                    id,
                                    index,
                                    CardId::new(u32::from_le_bytes(pair[4..].try_into().unwrap())),
                                )
                            },
                        )
                    })
            });
            let Some((feijing, index, discarded)) = found else {
                return Ok(Vec::new());
            };
            let blob = &mut tx.state_mut().skills.get_mut(feijing).unwrap().state_blob;
            let at = 12 + index * 8;
            blob.drain(at..at + 8);
            let count = ((blob.len() - 12) / 8) as u32;
            blob[8..12].copy_from_slice(&count.to_le_bytes());
            let mut effects = vec![EffectRequest::PreventDamage {
                damage_id: tx.state().resolution.current_damage_id,
            }];
            if tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.hp < owner.max_hp as i32)
            {
                effects.push(EffectRequest::RecoverHp {
                    source: player,
                    player,
                    amount: 1,
                });
            }
            let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            if tx
                .state()
                .cards
                .get(discarded)
                .is_some_and(|state| Some(state.zone) == discard)
            {
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::EngineFault("骁戈 owner hand missing".into()))?;
                effects.push(EffectRequest::MoveCards {
                    cards: vec![discarded],
                    to: hand,
                    from: discard,
                });
            }
            run_effects(tx, effects)
        }
        1 => {
            let owner_skills = tx
                .state()
                .players
                .get(player)
                .map(|owner| owner.skills.clone())
                .unwrap_or_default();
            for id in owner_skills {
                if tx.state().skills.get(id).is_some_and(|instance| {
                    instance.state_blob.get(..4) == Some(b"FJG1")
                        && instance.state_blob.len() >= 8
                        && u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap())
                            == card.raw()
                }) {
                    tx.state_mut()
                        .skills
                        .get_mut(id)
                        .unwrap()
                        .state_blob
                        .clear();
                }
            }
            let original = targets.first().copied().unwrap_or(target);
            if original.is_none()
                || crate::command::play::validate_source_free_card_on_target(
                    tx,
                    player,
                    CardKind::TrickJuedou,
                    original,
                )
                .is_err()
            {
                return Ok(Vec::new());
            }
            run_effects(
                tx,
                vec![EffectRequest::UseVirtualCardOn {
                    player,
                    kind: CardKind::TrickJuedou.tag(),
                    target: original,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("bad 骁戈 stage".into())),
    }
}

pub(crate) fn mbxuye_region_cards(tx: &Transaction, target: PlayerId) -> Vec<CardId> {
    [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, target))
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect()
}

pub(crate) fn resolve_mbxuye(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    was_max: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let hand = tx.state().hand_count(target);
            let was_max = tx
                .state()
                .players
                .iter_live()
                .all(|role| hand >= tx.state().hand_count(role.id));
            let mut events = draw_cards(tx, target, 2)?;
            events.extend(resolve_mbxuye(tx, player, target, 1, was_max, skill)?);
            Ok(events)
        }
        1 => {
            let hand = tx.state().hand_count(target);
            let now_max = tx
                .state()
                .players
                .iter_live()
                .all(|role| hand >= tx.state().hand_count(role.id));
            let options = mbxuye_region_cards(tx, target);
            if was_max || !now_max || options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    MBXUYE_CUSTOM_CHOICE,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault("bad 蓄业 stage".into())),
    }
}

pub(crate) fn mbkuangxiang_state(
    state: &crate::state::GameState,
    skill: engine_api::SkillId,
) -> Option<(PlayerId, u32, Vec<CardId>, Vec<CardId>)> {
    let blob = &state.skills.get(skill)?.state_blob;
    if blob.len() < 16 || &blob[..4] != b"KXI1" {
        return None;
    }
    let target = PlayerId::new(u32::from_le_bytes(blob[4..8].try_into().ok()?));
    let fired = u32::from_le_bytes(blob[8..12].try_into().ok()?);
    let owner_count = u32::from_le_bytes(blob[12..16].try_into().ok()?) as usize;
    if (blob.len() - 16) % 4 != 0 || owner_count > (blob.len() - 16) / 4 {
        return None;
    }
    let cards: Vec<_> = blob[16..]
        .chunks_exact(4)
        .map(|raw| CardId::new(u32::from_le_bytes(raw.try_into().unwrap())))
        .collect();
    Some((
        target,
        fired,
        cards[..owner_count].to_vec(),
        cards[owner_count..].to_vec(),
    ))
}

pub(crate) fn mbkuangxiang_exhausted_role(
    state: &crate::state::GameState,
    skill: engine_api::SkillId,
    owner: PlayerId,
    role: PlayerId,
    moved: &[CardId],
) -> bool {
    let Some((target, fired, owner_received, target_received)) = mbkuangxiang_state(state, skill)
    else {
        return false;
    };
    let (bit, tracked) = if role == owner {
        (1, owner_received)
    } else if role == target {
        (2, target_received)
    } else {
        return false;
    };
    !tracked.is_empty()
        && fired & bit == 0
        && moved.iter().any(|card| tracked.contains(card))
        && state
            .hand_cards(role)
            .iter()
            .all(|card| !tracked.contains(card))
}

pub(crate) fn resolve_mbkuangxiang(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if player == target
                || ![player, target].into_iter().all(|role| {
                    tx.state()
                        .players
                        .get(role)
                        .is_some_and(|participant| participant.alive)
                })
                || tx.state().hand_count(target) > tx.state().hand_count(player)
            {
                return Err(EffectError::Illegal("invalid 狂想 target".into()));
            }
            let player_cards = tx.state().hand_cards(player);
            let target_cards = tx.state().hand_cards(target);
            let player_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("狂想 owner hand missing".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("狂想 target hand missing".into()))?;
            let mut events = move_cards(tx, player_cards.clone(), target_hand, Some(player_hand))?;
            events.extend(move_cards(
                tx,
                target_cards.clone(),
                player_hand,
                Some(target_hand),
            )?);
            let mut blob = b"KXI1".to_vec();
            blob.extend_from_slice(&target.raw().to_le_bytes());
            blob.extend_from_slice(&0u32.to_le_bytes());
            blob.extend_from_slice(&(target_cards.len() as u32).to_le_bytes());
            for card in target_cards.iter().chain(player_cards.iter()) {
                blob.extend_from_slice(&card.raw().to_le_bytes());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("狂想 skill disappeared".into()))?
                .state_blob = blob;
            Ok(events)
        }
        1 => {
            let Some((tracked_target, fired, owner_received, target_received)) =
                mbkuangxiang_state(tx.state(), skill)
            else {
                return Ok(Vec::new());
            };
            let (bit, tracked) = if target == player {
                (1, owner_received)
            } else if target == tracked_target {
                (2, target_received)
            } else {
                return Ok(Vec::new());
            };
            if tracked.is_empty()
                || fired & bit != 0
                || tx
                    .state()
                    .hand_cards(target)
                    .iter()
                    .any(|card| tracked.contains(card))
            {
                return Ok(Vec::new());
            }
            let blob = &mut tx.state_mut().skills.get_mut(skill).unwrap().state_blob;
            blob[8..12].copy_from_slice(&(fired | bit).to_le_bytes());
            resolve_mbxuye(tx, player, player, 0, false, skill)
        }
        2 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("狂想 skill disappeared".into()))?
                .state_blob
                .clear();
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad 狂想 stage".into())),
    }
}

pub(crate) fn resolve_potchengpu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
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
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![POTCHENGPU_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("督作 target became invalid".into()));
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            let card = tx
                .state()
                .zones
                .get(draw)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev())
                .copied()
                .find(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.kind == CardKind::ShaFire)
                });
            let Some(card) = card else {
                return Ok(Vec::new());
            };
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("督作 target hand missing".into()))?;
            move_cards_with_reason(tx, vec![card], hand, Some(draw), CardMoveReason::Potduzuo)
        }
        2 => {
            let damage_id = if damage_id == 0 {
                tx.state().resolution.current_damage_id
            } else {
                damage_id
            };
            let mut options = vec![player, target];
            options.sort_by_key(|role| role.raw());
            options.dedup();
            options.retain(|role| {
                tx.state().players.get(*role).is_some_and(|participant| {
                    participant.alive && tx.state().hand_count(*role) > participant.hp.max(0) as u32
                })
            });
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    POTCHENGPU_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    damage_id,
                    skill.raw(),
                ],
            )
        }
        3 => {
            let Some(participant) = tx.state().players.get(target) else {
                return Ok(Vec::new());
            };
            let need = tx
                .state()
                .hand_count(target)
                .saturating_sub(participant.hp.max(0) as u32) as usize;
            if need == 0 {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_target_cards(
                tx.state(),
                target,
                target,
                TargetCardScope::Hand,
            );
            if options.len() < need {
                return discard_cards(tx, target, options);
            }
            let need_u8 = u8::try_from(need)
                .map_err(|_| EffectError::Illegal("避憾 discard count exceeds 255".into()))?;
            let mut payload = vec![
                POTCHENGPU_CUSTOM_CHOICE,
                3,
                player.raw(),
                target.raw(),
                damage_id,
                skill.raw(),
                need as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                need_u8,
                need_u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault("bad 势程普 stage".into())),
    }
}
