//! Effect primitive implementations — bingshi 4 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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

pub(crate) fn resolve_potjugu(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let count = {
        let instance = tx
            .state_mut()
            .skills
            .get_mut(skill)
            .ok_or_else(|| EffectError::EngineFault("拒孤 skill disappeared".into()))?;
        instance.turn_fire_count = instance.turn_fire_count.saturating_add(1);
        instance.turn_fire_count
    };
    let mut events = run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }])?;
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let need = options.len().min(count as usize);
    if need == 0 {
        return Ok(events);
    }
    let need_u8 = u8::try_from(need)
        .map_err(|_| EffectError::Illegal("拒孤 discard count exceeds 255".into()))?;
    let mut payload = vec![
        POTJUGU_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        need as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    events.extend(open_choice(
        tx,
        player,
        need_u8,
        need_u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )?);
    Ok(events)
}

pub(crate) fn resolve_potsifeng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("伺锋 skill disappeared".into()))?;
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .any(|target| target != player)
            {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev())
                .take(3)
                .copied()
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
            events.extend(begin_potsifeng_distribution(tx, player, skill, cards)?);
            Ok(events)
        }
        3 => {
            let cards = tx.state().skill_pile_cards(target, skill_def);
            let Some(first) = cards.first().copied() else {
                return Ok(Vec::new());
            };
            let pile = ensure_skill_pile_zone(tx, target, skill_def)?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let different = crate::query::effective_card_color_mask(tx.state(), target, card)
                != crate::query::effective_card_color_mask(tx.state(), target, first);
            let mut events = move_cards(tx, vec![first], discard, Some(pile))?;
            if different {
                let options = crate::query::discardable_target_cards(
                    tx.state(),
                    target,
                    target,
                    TargetCardScope::Hand,
                );
                if !options.is_empty() {
                    let mut payload = vec![POTSIFENG_CUSTOM_CHOICE, 2, target.raw()];
                    payload.extend(options.iter().map(|card| card.raw()));
                    events.extend(open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        options.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        payload,
                    )?);
                }
            }
            Ok(events)
        }
        4 => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                POTSIFENG_CUSTOM_CHOICE,
                3,
                player.raw(),
                target.raw(),
                skill.raw(),
            ],
        ),
        5 | 6 => {
            let cards = tx.state().skill_pile_cards(target, skill_def);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let pile = ensure_skill_pile_zone(tx, target, skill_def)?;
            if stage == 6 {
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::EngineFault("伺锋 owner hand missing".into()))?;
                return move_cards(tx, cards, hand, Some(pile));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let mut events = move_cards(tx, cards, discard, Some(pile))?;
            events.extend(run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("bad 伺锋 stage".into())),
    }
}

pub(crate) fn potfaen_state(tx: &Transaction, skill: engine_api::SkillId) -> (PlayerId, u8) {
    let Some(blob) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| &instance.state_blob)
    else {
        return (PlayerId::NONE, 0);
    };
    if blob.len() != 9 || &blob[..4] != b"FEN1" {
        return (PlayerId::NONE, 0);
    }
    (
        PlayerId::new(u32::from_le_bytes(blob[4..8].try_into().unwrap())),
        blob[8],
    )
}

pub(crate) fn set_potfaen_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    last_user: PlayerId,
    pending: u8,
) {
    let mut blob = b"FEN1".to_vec();
    blob.extend_from_slice(&last_user.raw().to_le_bytes());
    blob.push(pending);
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn open_potfaen_primary(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut options = vec![1];
    if !crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty() {
        options.push(2);
    }
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            POTCHENQUN_CUSTOM_CHOICE,
            0,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn resolve_potchenqun(
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
            let (previous, pending) = potfaen_state(tx, skill);
            set_potfaen_state(tx, skill, target, 0);
            let offer = previous == player;
            match pending {
                1 => {
                    let mut effects = vec![EffectRequest::DrawCards {
                        player: target,
                        count: 1,
                    }];
                    if offer {
                        effects.push(EffectRequest::ResolvePotchenqun {
                            player,
                            target,
                            stage: 2,
                            skill,
                        });
                    }
                    run_effects(tx, effects)
                }
                2 => {
                    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
                    if options.is_empty() {
                        return if offer {
                            open_potfaen_primary(tx, player, target, skill)
                        } else {
                            Ok(Vec::new())
                        };
                    }
                    let mut payload = vec![
                        POTCHENQUN_CUSTOM_CHOICE,
                        2,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        u32::from(offer),
                    ];
                    payload.extend(options.iter().map(|card| card.raw()));
                    open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        options.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
                _ if offer => open_potfaen_primary(tx, player, target, skill),
                _ => Ok(Vec::new()),
            }
        }
        1 => {
            let losses: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|target| (target, tx.state().turn.card_loss_count(target)))
                .collect();
            let max = losses.iter().map(|(_, count)| *count).max().unwrap_or(0);
            if max == 0 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = losses
                .into_iter()
                .filter_map(|(target, count)| (count == max).then_some(target.raw()))
                .collect();
            let mut payload = vec![POTCHENQUN_CUSTOM_CHOICE, 3, player.raw(), skill.raw()];
            payload.extend_from_slice(&options);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => open_potfaen_primary(tx, player, target, skill),
        _ => Err(EffectError::ContentFault("bad 势陈群 stage".into())),
    }
}

pub(crate) fn resolve_potwanglie(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() || helper_skill_def.is_none() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                POTCHENDAO_CUSTOM_CHOICE,
                0,
                player.raw(),
                helper_skill_def.raw(),
                skill.raw(),
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            let armed = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.phase_card_use_policy())
                .is_some_and(|(armed, _, _)| armed == card);
            if !armed {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_phase_card_use_policy_state();
            add_skill(tx, player, helper_skill_def)
        }
        2 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_phase_card_use_policy_state();
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad 势陈到往烈 stage".into())),
    }
}

pub(crate) fn pothongyi_mark(tx: &Transaction, skill: engine_api::SkillId) -> engine_api::MarkDefId {
    tx.state()
        .skills
        .get(skill)
        .map(|instance| engine_api::MarkDefId::new(instance.def.raw()))
        .unwrap_or(engine_api::MarkDefId::NONE)
}

pub(crate) fn pothongyi_pending(tx: &Transaction, skill: engine_api::SkillId) -> Option<(u8, u32)> {
    let blob = &tx.state().skills.get(skill)?.state_blob;
    (blob.len() == 9 && blob.get(..4) == Some(b"PHY1"))
        .then(|| (blob[4], u32::from_le_bytes(blob[5..9].try_into().unwrap())))
}

pub(crate) fn pothongyi_mark_count(tx: &Transaction, player: PlayerId, mark: engine_api::MarkDefId) -> u32 {
    tx.state()
        .players
        .get(player)
        .and_then(|player| {
            player
                .marks
                .iter()
                .find(|(id, _)| *id == mark.raw())
                .map(|(_, count)| (*count).max(0) as u32)
        })
        .unwrap_or(0)
}

pub(crate) fn resolve_pothongyi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mark = pothongyi_mark(tx, skill);
    let count = pothongyi_mark_count(tx, player, mark).min(5);
    match stage {
        0 => add_mark(tx, player, mark.raw(), 4_i32.saturating_sub(count as i32)),
        1 if count < 5 => add_mark(tx, player, mark.raw(), 1),
        1 => Ok(Vec::new()),
        2 if count > 0 => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![POTCHENDAO_CUSTOM_CHOICE, 1, player.raw(), skill.raw()],
        ),
        2 => Ok(Vec::new()),
        3 => {
            let Some((branch, captured)) = pothongyi_pending(tx, skill) else {
                return Ok(Vec::new());
            };
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            if branch == 0 {
                remove_mark(tx, player, mark.raw(), 0)
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player,
                        count: captured,
                    }],
                )
            }
        }
        _ => Err(EffectError::ContentFault("bad 势陈到弘毅 stage".into())),
    }
}

pub(crate) fn commit_pothongyi_prepare(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    branch: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if branch > 1 || !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal("invalid 弘毅 branch".into()));
    }
    let mark = pothongyi_mark(tx, skill);
    let count = pothongyi_mark_count(tx, player, mark).min(5);
    if count == 0 {
        return Ok(Vec::new());
    }
    let mut state = b"PHY1".to_vec();
    state.push(branch);
    state.extend_from_slice(&count.to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = state;
    if branch == 0 {
        run_effects(tx, vec![EffectRequest::DrawCards { player, count }])
    } else {
        remove_mark(tx, player, mark.raw(), 0)
    }
}

pub(crate) fn arm_potwanglie(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    if !tx.state().hand_cards(player).contains(&card)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Err(EffectError::Illegal("invalid 往烈 hand card".into()));
    }
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob =
        crate::state::SkillInstance::phase_card_use_policy_state(
            card,
            crate::state::SkillInstance::CARD_USE_POLICY_UNRESPONDABLE
                | crate::state::SkillInstance::CARD_USE_POLICY_DISTANCE_UNLIMITED,
            0,
        );
    Ok(())
}

pub(crate) fn first_shared_pile_card_by_category(
    tx: &Transaction,
    player: PlayerId,
    allowed: impl Fn(u8) -> bool,
) -> Result<Option<(ZoneId, CardId)>, EffectError> {
    for zone_kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let from = tx
            .state()
            .zones
            .find(zone_kind, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("shared card zone missing".into()))?;
        if let Some(card) = tx.state().zones.get(from).and_then(|zone| {
            zone.cards
                .iter()
                .rev()
                .copied()
                .find(|card| allowed(effective_card_category(tx.state(), player, *card)))
        }) {
            return Ok(Some((from, card)));
        }
    }
    Ok(None)
}

pub(crate) fn resolve_potguansha(
    tx: &mut Transaction,
    player: PlayerId,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || helper_skill_def.is_none() {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::Illegal("no equip zone".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let hand_cards = tx.state().zones.get(hand).unwrap().cards.clone();
    let equip_cards = tx.state().zones.get(equip).unwrap().cards.clone();
    let replace_count = hand_cards.len().saturating_add(equip_cards.len());
    if replace_count == 0 {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    if !hand_cards.is_empty() {
        events.extend(move_cards(tx, hand_cards, discard, Some(hand))?);
    }
    if !equip_cards.is_empty() {
        events.extend(move_cards(tx, equip_cards, discard, Some(equip))?);
    }
    let mut gained = 0u32;
    while gained < replace_count.min(u32::MAX as usize) as u32 {
        let Some((from, card)) =
            first_shared_pile_card_by_category(tx, player, |category| category == 1)?
        else {
            break;
        };
        events.extend(move_cards(tx, vec![card], hand, Some(from))?);
        gained += 1;
    }
    if gained > 0 {
        events.extend(add_skill(tx, player, helper_skill_def)?);
        events.extend(add_mark(tx, player, helper_skill_def.raw(), gained as i32)?);
    }
    Ok(events)
}

pub(crate) fn potjiyu_state(tx: &Transaction, skill: engine_api::SkillId) -> (u8, Vec<CardId>) {
    let Some(blob) = tx.state().skills.get(skill).map(|skill| &skill.state_blob) else {
        return (0, Vec::new());
    };
    if blob.len() < 5 || (blob.len() - 5) % 4 != 0 || blob.get(..4) != Some(b"PJY1") {
        return (0, Vec::new());
    }
    (
        blob[4].min(2),
        blob[5..]
            .chunks_exact(4)
            .map(|raw| CardId::new(u32::from_le_bytes(raw.try_into().unwrap())))
            .filter(|card| card.is_some())
            .collect(),
    )
}

pub(crate) fn set_potjiyu_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    refreshes: u8,
    cards: &[CardId],
) {
    let mut blob = b"PJY1".to_vec();
    blob.push(refreshes.min(2));
    for card in cards {
        blob.extend_from_slice(&card.raw().to_le_bytes());
    }
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn resolve_potjiyu(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let source_category = effective_card_category(tx.state(), player, card);
            if source_category == 0
                || !tx.state().hand_cards(player).contains(&card)
                || !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card)
            {
                return Err(EffectError::Illegal("invalid 急御 hand card".into()));
            }
            let hand = tx.state().cards.get(card).unwrap().zone;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let mut events = move_cards(tx, vec![card], discard, Some(hand))?;
            let (refreshes, _) = potjiyu_state(tx, skill);
            let mut categories = 1u8 << source_category;
            let mut gained = Vec::new();
            loop {
                let Some((from, gain)) =
                    first_shared_pile_card_by_category(tx, player, |category| {
                        category > 0 && categories & (1u8 << category) == 0
                    })?
                else {
                    break;
                };
                let category = effective_card_category(tx.state(), player, gain);
                categories |= 1u8 << category;
                events.extend(move_cards(tx, vec![gain], hand, Some(from))?);
                gained.push(gain);
                if categories & 0b1110 == 0b1110 {
                    break;
                }
            }
            if !gained.is_empty() {
                set_potjiyu_state(tx, skill, refreshes, &gained);
            }
            Ok(events)
        }
        1 => {
            let (refreshes, tracked) = potjiyu_state(tx, skill);
            if tracked.is_empty() {
                return Ok(Vec::new());
            }
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if tracked.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| Some(card.zone) == hand)
            }) {
                return Ok(Vec::new());
            }
            let next = refreshes.saturating_add(1).min(2);
            set_potjiyu_state(tx, skill, next, &[]);
            if refreshes < 2 {
                tx.state_mut().skills.get_mut(skill).unwrap().use_count = 0;
            }
            Ok(Vec::new())
        }
        2 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad 势娄圭急御 stage".into())),
    }
}

pub(crate) fn suit_code(suit: Suit) -> u8 {
    match suit {
        Suit::Spade => 1,
        Suit::Heart => 2,
        Suit::Club => 3,
        Suit::Diamond => 4,
        Suit::None => 0,
    }
}

pub(crate) fn mbzhuji_state(tx: &Transaction, skill: engine_api::SkillId) -> Option<(u8, u32, u32, CardId)> {
    let blob = &tx.state().skills.get(skill)?.state_blob;
    if blob.len() != 17 || blob.get(..4) != Some(b"MZJ1") {
        return None;
    }
    Some((
        blob[4],
        u32::from_le_bytes(blob[5..9].try_into().unwrap()),
        u32::from_le_bytes(blob[9..13].try_into().unwrap()),
        CardId::new(u32::from_le_bytes(blob[13..17].try_into().unwrap())),
    ))
}

pub(crate) fn set_mbzhuji_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    suit: u8,
    equip_count: u32,
    discarded: u32,
    card: CardId,
) {
    let mut blob = b"MZJ1".to_vec();
    blob.push(suit);
    blob.extend_from_slice(&equip_count.to_le_bytes());
    blob.extend_from_slice(&discarded.to_le_bytes());
    blob.extend_from_slice(&card.raw().to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn mbzhuji_legal_suits(tx: &Transaction, player: PlayerId) -> Vec<u32> {
    let discardable: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .collect();
    (1..=4)
        .filter(|suit| {
            let cards: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        u32::from(suit_code(crate::query::effective_card_suit(
                            tx.state(),
                            player,
                            card,
                        ))) == *suit
                    })
                })
                .collect();
            !cards.is_empty() && cards.iter().all(|card| discardable.contains(card))
        })
        .collect()
}

pub(crate) fn first_shared_equip_of_suit(
    tx: &Transaction,
    player: PlayerId,
    suit: u8,
) -> Result<Option<(ZoneId, CardId)>, EffectError> {
    for kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let zone = tx
            .state()
            .zones
            .find(kind, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("shared card zone missing".into()))?;
        if let Some(card) = tx.state().zones.get(zone).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state().cards.get(*card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), player, card).is_equip()
                        && suit_code(crate::query::effective_card_suit(tx.state(), player, card))
                            == suit
                })
            })
        }) {
            return Ok(Some((zone, card)));
        }
    }
    Ok(None)
}

pub(crate) fn resolve_mbzhuji(
    tx: &mut Transaction,
    player: PlayerId,
    choice: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let options = mbzhuji_legal_suits(tx, player);
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
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![MBZHUJI_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if !mbzhuji_legal_suits(tx, player).contains(&u32::from(choice)) {
                return Err(EffectError::Illegal("筑墼花色不再合法".into()));
            }
            let cards: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        suit_code(crate::query::effective_card_suit(tx.state(), player, card))
                            == choice
                    })
                })
                .collect();
            let equip_count = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len().min(u32::MAX as usize) as u32);
            set_mbzhuji_state(
                tx,
                skill,
                choice,
                equip_count,
                cards.len().min(u32::MAX as usize) as u32,
                CardId::NONE,
            );
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbzhuji {
                    player,
                    choice: 0,
                    stage: 2,
                    skill,
                });
            discard_cards(tx, player, cards)
        }
        2 => {
            let Some((suit, equip_count, discarded, _)) = mbzhuji_state(tx, skill) else {
                return Ok(Vec::new());
            };
            let Some((from, card)) = first_shared_equip_of_suit(tx, player, suit)? else {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .unwrap()
                    .state_blob
                    .clear();
                return Ok(Vec::new());
            };
            set_mbzhuji_state(tx, skill, suit, equip_count, discarded, card);
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbzhuji {
                    player,
                    choice: 0,
                    stage: 3,
                    skill,
                });
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("筑墼手牌区不存在".into()))?;
            move_cards(tx, vec![card], hand, Some(from))
        }
        3 => {
            let Some((_, _, _, card)) = mbzhuji_state(tx, skill) else {
                return Ok(Vec::new());
            };
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if tx
                .state()
                .cards
                .get(card)
                .is_some_and(|card| Some(card.zone) == hand)
            {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveMbzhuji {
                        player,
                        choice: 0,
                        stage: 4,
                        skill,
                    },
                );
                return match crate::command::play::use_equipment_from_hand(tx, player, card) {
                    Ok(()) => Ok(Vec::new()),
                    Err(EffectError::Illegal(_)) => {
                        tx.state_mut().resolution.deferred_tail_effects.pop();
                        resolve_mbzhuji(tx, player, 0, 4, skill)
                    }
                    Err(error) => Err(error),
                };
            }
            resolve_mbzhuji(tx, player, 0, 4, skill)
        }
        4 => {
            let Some((_, equip_count, discarded, _)) = mbzhuji_state(tx, skill) else {
                return Ok(Vec::new());
            };
            if discarded < equip_count {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .unwrap()
                    .state_blob
                    .clear();
                return Ok(Vec::new());
            }
            let participant = tx.state().players.get(player).unwrap();
            let mut options = vec![1];
            if participant.hp < participant.max_hp as i32 {
                options.push(2);
            }
            let hujia = participant
                .marks
                .iter()
                .find(|(mark, _)| *mark == crate::content::MARK_HUJIA)
                .map_or(0, |(_, count)| (*count).max(0));
            if hujia < 5 {
                options.push(3);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![MBZHUJI_CUSTOM_CHOICE, 1, player.raw(), skill.raw()],
            )
        }
        5 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            match choice {
                1 => draw_cards(tx, player, 2),
                2 if tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|player| player.hp < player.max_hp as i32) =>
                {
                    recover_hp(tx, player, player, 1)
                }
                3 => {
                    let hujia = tx
                        .state()
                        .players
                        .get(player)
                        .and_then(|player| {
                            player
                                .marks
                                .iter()
                                .find(|(mark, _)| *mark == crate::content::MARK_HUJIA)
                        })
                        .map_or(0, |(_, count)| (*count).max(0));
                    if hujia >= 5 {
                        return Err(EffectError::Illegal("护甲已达上限".into()));
                    }
                    add_mark(tx, player, crate::content::MARK_HUJIA, 1)
                }
                _ => Err(EffectError::Illegal("筑墼收益不再合法".into())),
            }
        }
        _ => Err(EffectError::ContentFault("bad 兵势孙韶筑墼 stage".into())),
    }
}

pub(crate) fn potlibing_state(tx: &Transaction, skill: engine_api::SkillId) -> (u32, u32, u32, u32) {
    let Some(blob) = tx.state().skills.get(skill).map(|skill| &skill.state_blob) else {
        return (0, 0, 0, 0);
    };
    if blob.len() != 20 || &blob[4..8] != b"LIB1" {
        return (0, 0, 0, 0);
    }
    (
        u32::from_le_bytes(blob[0..4].try_into().unwrap()),
        u32::from_le_bytes(blob[8..12].try_into().unwrap()),
        u32::from_le_bytes(blob[12..16].try_into().unwrap()),
        u32::from_le_bytes(blob[16..20].try_into().unwrap()),
    )
}

pub(crate) fn set_potlibing_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    attack: u32,
    step: u32,
    draws: u32,
    damage: u32,
) {
    let mut blob = attack.to_le_bytes().to_vec();
    blob.extend_from_slice(b"LIB1");
    blob.extend_from_slice(&step.to_le_bytes());
    blob.extend_from_slice(&draws.to_le_bytes());
    blob.extend_from_slice(&damage.to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn resolve_potlibing(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let Some(card_state) = tx.state().cards.get(card) else {
        return Ok(Vec::new());
    };
    let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
    let damage_card = kind.is_sha() || kind.is_damage_trick();
    let (attack, step, draws, damage) = potlibing_state(tx, skill);
    match stage {
        0 if damage_card => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            Ok(Vec::new())
        }
        0 => match step {
            0 => {
                set_potlibing_state(tx, skill, attack.saturating_add(1), 1, draws, damage);
                Ok(Vec::new())
            }
            1 => {
                let count = draws.saturating_add(1);
                set_potlibing_state(tx, skill, attack, 2, count, damage);
                draw_cards(tx, player, count)
            }
            _ => {
                set_potlibing_state(tx, skill, attack, 0, draws, damage.saturating_add(1));
                Ok(Vec::new())
            }
        },
        1 if damage_card && damage > 0 => {
            set_potlibing_state(tx, skill, attack, step, draws, 0);
            if !matches!(kind, CardKind::TrickShandian) {
                crate::command::play::increase_saved_card_base_damage(tx, player, card, damage)?;
            }
            Ok(Vec::new())
        }
        1 => Ok(Vec::new()),
        _ => Err(EffectError::ContentFault("bad 兵势夏侯霸砺兵 stage".into())),
    }
}

pub(crate) fn resolve_potpoxi(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|player| player.alive && !player.dying)
    {
        return Ok(Vec::new());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
    let mut seen = std::collections::BTreeSet::new();
    let mut cards: Vec<_> = cards
        .into_iter()
        .filter(|card| {
            seen.insert(card.raw())
                && tx.state().cards.get(*card).is_some_and(|state| {
                    state.zone == discard
                        && crate::query::effective_card_kind(tx.state(), player, state).is_sha()
                })
        })
        .collect();
    while let Some(card) = cards.first().copied() {
        cards.remove(0);
        let legal = crate::command::play::prompted_sha_targets_unlimited(tx, player, card);
        if legal.is_empty() {
            continue;
        }
        let (min, max) = if crate::query::sha_targets_all_for(tx.state(), player) {
            let count = u8::try_from(legal.len()).unwrap_or(u8::MAX);
            (count, count)
        } else {
            (
                1,
                u8::try_from(
                    legal
                        .len()
                        .min(crate::query::sha_target_limit_for(tx.state(), player) as usize),
                )
                .unwrap_or(u8::MAX),
            )
        };
        let mut payload = vec![
            POTPOXI_CUSTOM_CHOICE,
            player.raw(),
            card.raw(),
            skill.raw(),
            cards.len() as u32,
        ];
        payload.extend(cards.iter().map(|card| card.raw()));
        payload.push(legal.len() as u32);
        payload.extend(legal.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            player,
            min,
            max,
            false,
            legal.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            payload,
        );
    }
    Ok(Vec::new())
}

#[derive(Clone, Copy)]
pub(crate) struct PotfuanState {
    recorded: u32,
    used_round: u32,
    used: u32,
    only: u32,
    pending_card: CardId,
    pending_target: PlayerId,
    pending_first: u32,
}

impl Default for PotfuanState {
    fn default() -> Self {
        PotfuanState {
            recorded: 0,
            used_round: 0,
            used: 0,
            only: 0,
            pending_card: CardId::NONE,
            pending_target: PlayerId::NONE,
            pending_first: 0,
        }
    }
}

pub(crate) fn potfuan_state(tx: &Transaction, skill: engine_api::SkillId) -> PotfuanState {
    let Some(blob) = tx.state().skills.get(skill).map(|skill| &skill.state_blob) else {
        return PotfuanState::default();
    };
    if blob.len() != 32 || &blob[..4] != b"FUA1" {
        return PotfuanState::default();
    }
    let read = |range: std::ops::Range<usize>| u32::from_le_bytes(blob[range].try_into().unwrap());
    PotfuanState {
        recorded: read(4..8),
        used_round: read(8..12),
        used: read(12..16),
        only: read(16..20),
        pending_card: CardId::new(read(20..24)),
        pending_target: PlayerId::new(read(24..28)),
        pending_first: read(28..32),
    }
}

pub(crate) fn set_potfuan_state(tx: &mut Transaction, skill: engine_api::SkillId, state: PotfuanState) {
    let mut blob = b"FUA1".to_vec();
    for value in [
        state.recorded,
        state.used_round,
        state.used,
        state.only,
        state.pending_card.raw(),
        state.pending_target.raw(),
        state.pending_first,
    ] {
        blob.extend_from_slice(&value.to_le_bytes());
    }
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn suit_bit(suit: engine_api::Suit) -> u32 {
    match suit {
        engine_api::Suit::Spade => 1,
        engine_api::Suit::Heart => 2,
        engine_api::Suit::Club => 4,
        engine_api::Suit::Diamond => 8,
        engine_api::Suit::None => 0,
    }
}

pub(crate) fn suit_tag_from_bit(bit: u32) -> Option<u32> {
    match bit {
        1 => Some(1),
        2 => Some(2),
        4 => Some(3),
        8 => Some(4),
        _ => None,
    }
}

pub(crate) fn suit_bit_from_tag(tag: u32) -> Option<u32> {
    match tag {
        1 => Some(1),
        2 => Some(2),
        3 => Some(4),
        4 => Some(8),
        _ => None,
    }
}

pub(crate) fn attack_range_value(tx: &Transaction, player: PlayerId) -> i32 {
    match crate::query::query(
        tx.state(),
        &rule_api::QueryRequest::AttackRange { player },
        &mut crate::query::QueryContext::new(),
    ) {
        rule_api::QueryResponse::Int(value) => value.max(0),
        _ => 0,
    }
}

pub(crate) fn find_skill_by_def(
    tx: &Transaction,
    player: PlayerId,
    def: engine_api::SkillDefId,
) -> Option<engine_api::SkillId> {
    tx.state()
        .players
        .get(player)?
        .skills
        .iter()
        .copied()
        .find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def == def)
        })
}

pub(crate) fn potfuan_hand_suits(tx: &Transaction, player: PlayerId) -> u32 {
    tx.state()
        .hand_cards(player)
        .into_iter()
        .filter_map(|card| tx.state().cards.get(card))
        .fold(0, |mask, card| {
            mask | suit_bit(crate::query::effective_card_suit(tx.state(), player, card))
        })
}

pub(crate) fn allocate_virtual_sha(tx: &mut Transaction) -> Result<CardId, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);
    Ok(card)
}

pub(crate) fn remove_unused_virtual_card(tx: &mut Transaction, card: CardId) -> Result<(), EffectError> {
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::EngineFault("virtual card missing".into()))?;
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("removed zone missing".into()))?;
    move_one_card(tx, card, from, removed, PlayerId::NONE, true)
}

pub(crate) fn potfuan_sha_targets(tx: &Transaction, player: PlayerId, card: CardId) -> Vec<PlayerId> {
    if crate::query::card_use_cancelled_for_card(tx.state(), player, card) {
        return Vec::new();
    }
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            crate::command::play::validate_sha_target_with_options(tx, player, card, *target, true)
                .is_ok()
        })
        .collect()
}

pub(crate) fn silence_nonlocked_until_own_turn_end(
    tx: &mut Transaction,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|player| player.alive)
    {
        return Ok(Vec::new());
    }
    let (helper, events) =
        grant_skill_instance(tx, target, engine_api::SkillDefId::NONE, Vec::new())?;
    tx.state_mut()
        .skills
        .get_mut(helper)
        .unwrap()
        .temporary_until_turn_end_of = target;
    let disabled: Vec<_> = tx
        .state()
        .players
        .get(target)
        .unwrap()
        .skills
        .iter()
        .copied()
        .filter(|skill| {
            *skill != helper
                && tx.state().skills.get(*skill).is_some_and(|instance| {
                    tx.state()
                        .content
                        .skill_metadata(instance.def)
                        .is_some_and(|metadata| !metadata.locked)
                })
        })
        .collect();
    for skill in disabled {
        let instance = tx.state_mut().skills.get_mut(skill).unwrap();
        instance.disabled_by_skills.push(helper);
        instance
            .disabled_by_skills
            .sort_by_key(|reason| reason.raw());
        instance.disabled_by_skills.dedup();
    }
    tx.state_mut().rebuild_derived_indexes();
    Ok(events)
}

pub(crate) fn resolve_potfuan(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    stage: u8,
    range_helper_def: engine_api::SkillDefId,
    yinxian_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let previous = attack_range_value(tx, player);
            if previous == 0 {
                return Ok(Vec::new());
            }
            let mut events = add_skill_with_state(tx, player, range_helper_def, None)?;
            if let Some(yinxian) = find_skill_by_def(tx, player, yinxian_def) {
                tx.state_mut().skills.get_mut(yinxian).unwrap().state_blob =
                    0u32.to_le_bytes().to_vec();
            }
            let draw = previous
                .saturating_sub(attack_range_value(tx, player))
                .min(5) as u32;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolvePotfuan {
                    player,
                    card: CardId::NONE,
                    stage: 1,
                    range_helper_def,
                    yinxian_def,
                    skill,
                });
            if draw > 0 {
                events.extend(draw_cards(tx, player, draw)?);
            }
            Ok(events)
        }
        1 => {
            let mut state = potfuan_state(tx, skill);
            if state.used_round != tx.state().turn.round {
                state.used_round = tx.state().turn.round;
                state.used = 0;
            }
            set_potfuan_state(tx, skill, state);
            let available = potfuan_hand_suits(tx, player) & !state.recorded & !state.used & 0b1111;
            let options: Vec<_> = [1, 2, 4, 8]
                .into_iter()
                .filter(|bit| available & bit != 0)
                .filter_map(suit_tag_from_bit)
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
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    POTFUAN_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    range_helper_def.raw(),
                    yinxian_def.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let matching = potfuan_state(tx, skill).recorded
                & tx.state().turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK) as u32
                & 0b1111;
            if tx.state().turn.current == player || matching == 0 {
                return Ok(Vec::new());
            }
            let card = allocate_virtual_sha(tx)?;
            let targets = potfuan_sha_targets(tx, player, card);
            if targets.is_empty() {
                remove_unused_virtual_card(tx, card)?;
                return Ok(Vec::new());
            }
            let mut payload = vec![
                POTFUAN_CUSTOM_CHOICE,
                1,
                player.raw(),
                card.raw(),
                matching,
                range_helper_def.raw(),
                yinxian_def.raw(),
                skill.raw(),
            ];
            payload.extend(targets.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        3 => {
            let mut state = potfuan_state(tx, skill);
            if state.pending_first == 0 || state.pending_card != card {
                return Ok(Vec::new());
            }
            state.pending_first = 0;
            set_potfuan_state(tx, skill, state);
            if state.only != 0 {
                return silence_nonlocked_until_own_turn_end(tx, state.pending_target);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    POTFUAN_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    state.pending_target.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault("bad 势张任伏暗 stage".into())),
    }
}

pub(crate) fn commit_potfuan_suit(
    tx: &mut Transaction,
    player: PlayerId,
    suit_tag: u32,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let bit = suit_bit_from_tag(suit_tag)
        .ok_or_else(|| EffectError::Illegal("invalid 伏暗 suit".into()))?;
    let mut state = potfuan_state(tx, skill);
    if state.used_round != tx.state().turn.round {
        state.used_round = tx.state().turn.round;
        state.used = 0;
    }
    if potfuan_hand_suits(tx, player) & bit == 0
        || state.recorded & bit != 0
        || state.used & bit != 0
    {
        return Err(EffectError::Illegal(
            "伏暗 suit is no longer available".into(),
        ));
    }
    state.recorded |= bit;
    state.used |= bit;
    set_potfuan_state(tx, skill, state);
    Ok(())
}

pub(crate) fn commit_potfuan_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    matching: u32,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    if !potfuan_sha_targets(tx, player, card).contains(&target) {
        return Err(EffectError::Illegal(
            "伏暗 target is no longer legal".into(),
        ));
    }
    let current = potfuan_state(tx, skill).recorded
        & tx.state().turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK) as u32
        & 0b1111;
    if current == 0 || current != matching {
        return Err(EffectError::Illegal("伏暗 recorded suits changed".into()));
    }
    let mut state = potfuan_state(tx, skill);
    state.recorded &= !matching;
    state.pending_card = card;
    state.pending_target = target;
    state.pending_first = 1;
    set_potfuan_state(tx, skill, state);
    crate::command::play::handle_virtual_sha_with_policies_and_bonus(
        tx,
        player,
        card,
        &[target],
        true,
        true,
        0,
    )
}

pub(crate) fn commit_potfuan_branch(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    branch: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut state = potfuan_state(tx, skill);
    if state.pending_target != target {
        return Err(EffectError::Illegal("伏暗 target changed".into()));
    }
    match branch {
        1 if state.only == 0 => {
            state.only = 1;
            set_potfuan_state(tx, skill, state);
            let card = allocate_virtual_sha(tx)?;
            if !potfuan_sha_targets(tx, player, card).contains(&target) {
                remove_unused_virtual_card(tx, card)?;
                return Ok(Vec::new());
            }
            crate::command::play::handle_virtual_sha_with_policies_and_bonus(
                tx,
                player,
                card,
                &[target],
                true,
                true,
                0,
            )?;
            Ok(Vec::new())
        }
        2 => silence_nonlocked_until_own_turn_end(tx, target),
        _ => Err(EffectError::Illegal("invalid 伏暗 branch".into())),
    }
}

pub(crate) fn resolve_potyinxian(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let Some(card_state) = tx.state().cards.get(card) else {
                return Ok(Vec::new());
            };
            if !crate::query::effective_card_kind(tx.state(), player, card_state).is_sha() {
                return Ok(Vec::new());
            }
            let targets = tx
                .state()
                .resolution
                .pending_card_resolutions
                .iter()
                .rev()
                .find(|pending| pending.source == player && pending.card == card)
                .map(|pending| pending.targets.clone())
                .unwrap_or_default();
            let distances: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| crate::query::player_in_attack_range(tx.state(), player, *target))
                .filter_map(|target| {
                    match crate::query::query(
                        tx.state(),
                        &rule_api::QueryRequest::Distance {
                            from: player,
                            to: target,
                        },
                        &mut crate::query::QueryContext::new(),
                    ) {
                        rule_api::QueryResponse::Int(distance) => Some((target, distance)),
                        _ => None,
                    }
                })
                .collect();
            let farthest = distances.iter().map(|(_, distance)| *distance).max();
            if farthest.is_some_and(|max| {
                distances
                    .iter()
                    .any(|(target, distance)| *distance == max && targets.contains(target))
            }) {
                crate::command::play::increase_saved_sha_base_damage(tx, player, card, 1)?;
            }
            Ok(Vec::new())
        }
        1 if tx.state().turn.current != player => {
            let current = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.state_blob.get(..4))
                .map(|raw| u32::from_le_bytes(raw.try_into().unwrap()))
                .unwrap_or(0);
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob =
                current.saturating_add(1).to_le_bytes().to_vec();
            Ok(Vec::new())
        }
        1 => Ok(Vec::new()),
        _ => Err(EffectError::ContentFault("bad 势张任引弦 stage".into())),
    }
}

pub(crate) fn resolve_pothuanjie(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    qice_def: engine_api::SkillDefId,
    kanpo_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || qice_def.is_none() || kanpo_def.is_none()
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player
                        && tx.state().hand_count(player) + tx.state().hand_count(*candidate) > 0
                })
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
                vec![
                    POTHUANJIE_CUSTOM_CHOICE,
                    player.raw(),
                    qice_def.raw(),
                    kanpo_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_count(player) + tx.state().hand_count(target) == 0
            {
                return Err(EffectError::Illegal("invalid 共谋 target".into()));
            }
            let player_cards = tx.state().hand_cards(player);
            let target_cards = tx.state().hand_cards(target);
            let player_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("共谋 owner hand missing".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("共谋 target hand missing".into()))?;
            let mut events = move_cards(tx, player_cards, target_hand, Some(player_hand))?;
            events.extend(move_cards(
                tx,
                target_cards,
                player_hand,
                Some(target_hand),
            )?);
            events.extend(add_skill_until_turn_end(tx, player, qice_def)?);
            events.extend(add_skill_until_turn_end(tx, target, kanpo_def)?);
            Ok(events)
        }
        2 => {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("正朔 discard pile missing".into()))?;
            let mut order = tx.state().players.alive_by_seat();
            if let Some(index) = order.iter().position(|candidate| *candidate == player) {
                order.rotate_left(index);
            }
            let mut events = Vec::new();
            for role in order {
                let cards = tx.state().hand_cards(role);
                if cards.is_empty() {
                    continue;
                }
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, role)
                    .ok_or_else(|| EffectError::EngineFault("正朔 hand missing".into()))?;
                events.extend(move_cards_with_reason(
                    tx,
                    cards,
                    discard,
                    Some(hand),
                    CardMoveReason::Discard,
                )?);
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolvePothuanjie {
                    player,
                    target: PlayerId::NONE,
                    stage: 3,
                    qice_def,
                    kanpo_def,
                    skill,
                },
            );
            Ok(events)
        }
        3 => {
            reshuffle_discard_into_draw(tx)?;
            let mut order = tx.state().players.alive_by_seat();
            if let Some(index) = order.iter().position(|candidate| *candidate == player) {
                order.rotate_left(index);
            }
            let mut events = Vec::new();
            for role in order {
                events.extend(draw_cards(tx, role, 4)?);
            }
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "bad 势桓阶 resolver stage".into(),
        )),
    }
}

pub(crate) fn potnigu_count(tx: &Transaction, skill: engine_api::SkillId) -> u8 {
    tx.state()
        .skills
        .get(skill)
        .filter(|instance| instance.state_blob.len() == 5 && &instance.state_blob[..4] == b"PNG1")
        .map_or(0, |instance| instance.state_blob[4])
}

pub(crate) fn set_potnigu_count(tx: &mut Transaction, skill: engine_api::SkillId, count: u8) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = if count == 0 {
            Vec::new()
        } else {
            let mut blob = b"PNG1".to_vec();
            blob.push(count);
            blob
        };
    }
}

pub(crate) fn open_potnigu_gift(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    mut index: u8,
    mut declined: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    while usize::from(index) < targets.len() {
        let target = targets[usize::from(index)];
        let options = if tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquip,
            )
        } else {
            Vec::new()
        };
        if options.is_empty() {
            index = index.saturating_add(1);
            declined = declined.saturating_add(1);
            continue;
        }
        let mut payload = vec![
            POTNIGU_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            u32::from(index),
            u32::from(declined),
            targets.len().min(u32::MAX as usize) as u32,
        ];
        payload.extend(targets.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            target,
            1,
            1,
            true,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            payload,
        );
    }
    set_potnigu_count(tx, skill, declined);
    Ok(Vec::new())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_potnigu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    index: u8,
    declined: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let targets = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    *target != player
                        && crate::query::player_in_attack_range(tx.state(), player, *target)
                })
                .collect();
            open_potnigu_gift(tx, player, targets, 0, 0, skill)
        }
        1 => {
            if targets.get(usize::from(index)) != Some(&target) {
                return Err(EffectError::Illegal("逆固赠牌次序已变化".into()));
            }
            let mut events = Vec::new();
            let declined = if card.is_none() {
                declined.saturating_add(1)
            } else {
                let options = crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    target,
                    TargetCardScope::HandEquip,
                );
                if !options.contains(&card) {
                    return Err(EffectError::Illegal("逆固赠牌已不可交出".into()));
                }
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("逆固赠牌不存在".into()))?;
                let to = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::EngineFault("逆固接收手牌区缺失".into()))?;
                events.extend(give_cards(tx, vec![card], to, from)?);
                declined
            };
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolvePotnigu {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    targets,
                    index: index.saturating_add(1),
                    declined,
                    stage: 2,
                    skill,
                });
            Ok(events)
        }
        2 => open_potnigu_gift(tx, player, targets, index, declined, skill),
        3 => {
            let count = potnigu_count(tx, skill);
            if count == 0 {
                return Ok(Vec::new());
            }
            increase_bound_damage_by_one(tx, tx.state().resolution.current_damage_id)?;
            set_potnigu_count(tx, skill, count - 1);
            Ok(Vec::new())
        }
        4 => {
            set_potnigu_count(tx, skill, 0);
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad 势孙綝逆固 stage".into())),
    }
}

#[derive(Clone)]
pub(crate) struct PotlulianPending {
    card: CardId,
    kind: u8,
    targets: Vec<PlayerId>,
}

pub(crate) fn potlulian_pending(tx: &Transaction, skill: engine_api::SkillId) -> Vec<PotlulianPending> {
    let Some(blob) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.state_blob.as_slice())
    else {
        return Vec::new();
    };
    if blob.len() < 5 || &blob[..4] != b"PLL1" {
        return Vec::new();
    }
    let mut offset = 5usize;
    let mut records = Vec::new();
    for _ in 0..blob[4] {
        if offset + 6 > blob.len() {
            return Vec::new();
        }
        let card = CardId::new(u32::from_le_bytes(
            blob[offset..offset + 4].try_into().unwrap(),
        ));
        let kind = blob[offset + 4];
        let count = usize::from(blob[offset + 5]);
        offset += 6;
        if offset + count * 4 > blob.len() {
            return Vec::new();
        }
        let targets = (0..count)
            .map(|index| {
                let start = offset + index * 4;
                PlayerId::new(u32::from_le_bytes(
                    blob[start..start + 4].try_into().unwrap(),
                ))
            })
            .collect();
        offset += count * 4;
        records.push(PotlulianPending {
            card,
            kind,
            targets,
        });
    }
    (offset == blob.len())
        .then_some(records)
        .unwrap_or_default()
}

pub(crate) fn set_potlulian_pending(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    records: &[PotlulianPending],
) {
    let mut blob = if records.is_empty() {
        Vec::new()
    } else {
        let mut blob = b"PLL1".to_vec();
        blob.push(records.len().min(8) as u8);
        for record in records.iter().take(8) {
            blob.extend_from_slice(&record.card.raw().to_le_bytes());
            blob.push(record.kind);
            blob.push(record.targets.len().min(8) as u8);
            for target in record.targets.iter().take(8) {
                blob.extend_from_slice(&target.raw().to_le_bytes());
            }
        }
        blob
    };
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = std::mem::take(&mut blob);
    }
}

pub(crate) fn potlulian_category(kind: CardKind) -> u8 {
    if kind.is_basic() {
        1
    } else if kind.is_trick() {
        2
    } else {
        4
    }
}

pub(crate) fn resolve_potlulian(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let Some(record) = tx
                .state()
                .resolution
                .pending_card_resolutions
                .iter()
                .rev()
                .find(|record| {
                    record.source == player && record.card == card && record.source_was_hand
                })
                .cloned()
            else {
                return Ok(Vec::new());
            };
            let mut records = potlulian_pending(tx, skill);
            records.retain(|pending| pending.card != card);
            records.push(PotlulianPending {
                card,
                kind: record.kind,
                targets: record.targets,
            });
            if records.len() > 8 {
                records.remove(0);
            }
            set_potlulian_pending(tx, skill, &records);
            Ok(Vec::new())
        }
        1 => {
            let mut records = potlulian_pending(tx, skill);
            let Some(position) = records.iter().position(|pending| pending.card == card) else {
                return Ok(Vec::new());
            };
            let record = records.remove(position);
            set_potlulian_pending(tx, skill, &records);
            let Some(kind) = CardKind::from_tag(record.kind) else {
                return Ok(Vec::new());
            };
            let category = potlulian_category(kind);
            let has_same_category = tx.state().hand_cards(player).into_iter().any(|hand_card| {
                tx.state()
                    .cards
                    .get(hand_card)
                    .map(|card| crate::query::effective_card_kind(tx.state(), player, card))
                    .is_some_and(|kind| potlulian_category(kind) == category)
            });
            if has_same_category {
                return Ok(Vec::new());
            }
            let owner = tx
                .state()
                .players
                .get(player)
                .ok_or_else(|| EffectError::Illegal("戮连拥有者不存在".into()))?;
            let owner_hp = owner.hp;
            let owner_equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len());
            let bool1 = record.targets.iter().any(|target| {
                tx.state()
                    .players
                    .get(*target)
                    .is_some_and(|target| target.hp <= owner_hp)
            });
            let bool2 = record.targets.iter().any(|target| {
                tx.state().players.get(*target).is_some_and(|_| {
                    tx.state()
                        .zones
                        .find(ZoneKind::Equip, *target)
                        .and_then(|zone| tx.state().zones.get(zone))
                        .map_or(0, |zone| zone.cards.len())
                        <= owner_equip
                })
            });
            let mut events = Vec::new();
            if bool1 {
                for target in record.targets.iter().copied() {
                    let unlinked = tx.state().players.get(target).is_some_and(|target| {
                        target.alive
                            && !target.marks.iter().any(|(mark, count)| {
                                *mark == crate::content::MARK_CHAIN && *count > 0
                            })
                    });
                    if unlinked {
                        events.extend(add_mark(tx, target, crate::content::MARK_CHAIN, 1)?);
                    }
                }
            }
            if bool2 {
                events.extend(draw_cards(tx, player, 1)?);
            }
            if bool1 && bool2 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolvePotlulian {
                        player,
                        card,
                        target: PlayerId::NONE,
                        stage: 5,
                        skill,
                    },
                );
            }
            Ok(events)
        }
        2 => {
            let min_hp = tx
                .state()
                .players
                .iter_live()
                .filter(|participant| participant.alive)
                .map(|participant| participant.hp)
                .min();
            if !tx.state().players.get(target).is_some_and(|participant| {
                participant.alive && min_hp.is_some_and(|min_hp| participant.hp > min_hp)
            }) {
                return Err(EffectError::Illegal("戮连火焰伤害目标已不合法".into()));
            }
            deal_damage(
                tx,
                player,
                target,
                1,
                false,
                rule_api::damage_nature::FIRE,
                false,
                CardId::NONE,
                0,
                0,
            )
        }
        5 => {
            let Some(min_hp) = tx
                .state()
                .players
                .iter_live()
                .filter(|target| target.alive)
                .map(|target| target.hp)
                .min()
            else {
                return Ok(Vec::new());
            };
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|target| target.hp > min_hp)
                })
                .collect();
            if options.is_empty() {
                Ok(Vec::new())
            } else if options.len() == 1 {
                resolve_potlulian(tx, player, card, options[0], 2, skill)
            } else {
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
                        POTLULIAN_CUSTOM_CHOICE,
                        player.raw(),
                        card.raw(),
                        skill.raw(),
                    ],
                )
            }
        }
        _ => Err(EffectError::ContentFault("bad 势孙綝戮连 stage".into())),
    }
}
