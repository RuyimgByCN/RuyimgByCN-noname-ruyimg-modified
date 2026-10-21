//! Effect primitives for `potjixi_targets` and related helpers.

use super::*;

pub(crate) fn potjixi_targets(tx: &Transaction, skill: engine_api::SkillId) -> Vec<PlayerId> {
    tx.state()
        .skills
        .get(skill)
        .map(|instance| {
            instance
                .state_blob
                .chunks_exact(4)
                .map(|bytes| PlayerId::new(u32::from_le_bytes(bytes.try_into().unwrap())))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn resolve_potjixi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            Ok(Vec::new())
        }
        1 => {
            if target.is_none() || target == player {
                return Ok(Vec::new());
            }
            let mut recorded = potjixi_targets(tx, skill);
            if !recorded.contains(&target) {
                recorded.push(target);
                recorded.sort_unstable();
                tx.state_mut().skills.get_mut(skill).unwrap().state_blob = recorded
                    .into_iter()
                    .flat_map(|target| target.raw().to_le_bytes())
                    .collect();
            }
            Ok(Vec::new())
        }
        2 => {
            let current = tx.state().turn.current;
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                current,
                TargetCardScope::HandEquip,
            );
            if potjixi_targets(tx, skill).is_empty() || options.is_empty() {
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
                    POTJIXI_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    current.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            let recorded = potjixi_targets(tx, skill);
            let options: Vec<_> = recorded
                .into_iter()
                .filter(|candidate| {
                    *candidate != player
                        && tx
                            .state()
                            .players
                            .get(*candidate)
                            .is_some_and(|participant| participant.alive)
                        && !crate::query::gainable_target_cards(
                            tx.state(),
                            player,
                            *candidate,
                            TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                options.len().min(u8::MAX as usize) as u8,
                true,
                options.iter().map(|target| target.raw()).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![POTJIXI_CUSTOM_CHOICE, 1, player.raw(), skill.raw()],
            )
        }
        4 => {
            let recorded: std::collections::BTreeSet<_> =
                potjixi_targets(tx, skill).into_iter().collect();
            let selected: std::collections::BTreeSet<_> = targets.iter().copied().collect();
            if targets.is_empty()
                || selected.len() != targets.len()
                || targets.iter().any(|target| {
                    !recorded.contains(target)
                        || crate::query::gainable_target_cards(
                            tx.state(),
                            player,
                            *target,
                            TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
            {
                return Err(EffectError::Illegal("势急袭目标已变化".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::UseVirtualCardOnSelectedTargets {
                    player,
                    kind: CardKind::TrickShunshou.tag(),
                    targets,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("势急袭 action 非法".into())),
    }
}

pub(crate) fn resolve_potceduan(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || !crate::query::player_in_attack_range(tx.state(), target, player)
            {
                return Err(EffectError::Illegal("势策断目标已变化".into()));
            }
            let candidates: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    !tx.state().hand_cards(*candidate).is_empty()
                        && crate::query::player_in_attack_range(tx.state(), target, *candidate)
                })
                .collect();
            let Some(first) = candidates.first().copied() else {
                return Ok(Vec::new());
            };
            let mut payload = vec![
                POTCEDUAN_CUSTOM_CHOICE,
                player.raw(),
                target.raw(),
                skill.raw(),
                candidates.len() as u32,
                0,
            ];
            payload.extend(candidates.iter().map(|candidate| candidate.raw()));
            open_choice(
                tx,
                first,
                1,
                1,
                false,
                tx.state()
                    .hand_cards(first)
                    .into_iter()
                    .map(CardId::raw)
                    .collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        1 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
            }
            draw_cards(tx, player, 1)
        }
        _ => Err(EffectError::ContentFault("势策断 action 非法".into())),
    }
}

pub(crate) fn complete_potceduan(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    candidates: &[PlayerId],
    revealed: &[CardId],
) -> Result<(), EffectError> {
    if candidates.len() != revealed.len() {
        return Err(EffectError::EngineFault("势策断展示列表损坏".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::RevealCards {
            cards: revealed.to_vec(),
            to: Vec::new(),
        }],
    )?;
    let mut colors = [0usize; 2];
    for (owner, card) in candidates.iter().zip(revealed) {
        let Some(card) = tx.state().cards.get(*card) else {
            return Err(EffectError::Illegal("势策断展示牌已移动".into()));
        };
        let red = matches!(
            crate::query::effective_card_suit(tx.state(), *owner, card),
            engine_api::Suit::Heart | engine_api::Suit::Diamond
        );
        colors[usize::from(red)] += 1;
    }
    let max = *colors.iter().max().unwrap_or(&0);
    let materials: Vec<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                let red = matches!(
                    crate::query::effective_card_suit(tx.state(), player, card),
                    engine_api::Suit::Heart | engine_api::Suit::Diamond
                );
                colors[usize::from(red)] == max
            })
        })
        .collect();
    let Some(first) = materials.first().copied() else {
        return Ok(());
    };
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = first.raw().to_le_bytes().to_vec();
    }
    match crate::command::play::play_as_kind_with_extra_damage_flags(
        tx,
        player,
        first,
        CardKind::Sha,
        &[target],
        true,
        false,
        &[],
        true,
        true,
        0,
        &materials[1..],
        0,
    ) {
        Err(EffectError::Illegal(_)) => Ok(()),
        result => result,
    }
}

pub(crate) fn pottaishici_skill_id(
    tx: &Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
) -> Option<engine_api::SkillId> {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == player && instance.def == skill_def)
        .map(|instance| instance.id)
}

pub(crate) fn pottaishici_mode(
    tx: &Transaction,
    player: PlayerId,
    skill_def: engine_api::SkillDefId,
) -> u8 {
    pottaishici_skill_id(tx, player, skill_def)
        .and_then(|skill| tx.state().skills.get(skill))
        .and_then(|skill| skill.state_blob.first().copied())
        .filter(|mode| *mode <= 3)
        .unwrap_or(0)
}

pub(crate) fn pottaishici_x(
    tx: &Transaction,
    owner: PlayerId,
    subject: PlayerId,
    skill_def: engine_api::SkillDefId,
    default_attack_range: bool,
) -> u32 {
    let Some(participant) = tx.state().players.get(subject) else {
        return 0;
    };
    match pottaishici_mode(tx, owner, skill_def) {
        1 => participant.hp.max(0) as u32,
        2 => (participant.max_hp as i32 - participant.hp).max(0) as u32,
        3 => tx.state().players.alive_by_seat().len() as u32,
        _ if default_attack_range => current_attack_range(tx, owner) as u32,
        _ => participant.max_hp.max(0) as u32,
    }
}

pub(crate) fn pottaishici_mark_count(
    tx: &Transaction,
    player: PlayerId,
    zhanlie_def: engine_api::SkillDefId,
) -> u32 {
    tx.state()
        .players
        .get(player)
        .and_then(|owner| {
            owner
                .marks
                .iter()
                .find(|(mark, _)| *mark == zhanlie_def.raw())
                .map(|(_, count)| (*count).max(0) as u32)
        })
        .unwrap_or(0)
}

pub(crate) fn remove_pottaishici_virtual_card(
    tx: &mut Transaction,
    card: CardId,
) -> Result<(), EffectError> {
    let Some(from) = tx.state().cards.get(card).map(|card| card.zone) else {
        return Ok(());
    };
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("removed zone missing".into()))?;
    if from != removed {
        move_one_card(tx, card, from, removed, PlayerId::NONE, true)?;
    }
    Ok(())
}

pub(crate) fn resolve_pottaishici(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    cards: Vec<CardId>,
    hanzhan_def: engine_api::SkillDefId,
    zhanlie_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let quota = pottaishici_x(tx, player, player, zhanlie_def, true).min(u8::MAX as u32);
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                let mode = instance
                    .state_blob
                    .first()
                    .copied()
                    .filter(|mode| *mode <= 3)
                    .unwrap_or(0);
                instance.state_blob = vec![mode, quota as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            }
            Ok(Vec::new())
        }
        1 => {
            let eligible = cards
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        card.kind.is_sha()
                            && tx
                                .state()
                                .zones
                                .get(card.zone)
                                .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
                    })
                })
                .count() as u32;
            let (quota, seen) = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| {
                    (instance.state_blob.len() >= 3)
                        .then_some((instance.state_blob[1] as u32, instance.state_blob[2] as u32))
                })
                .unwrap_or((0, 0));
            let gain = eligible
                .min(quota.saturating_sub(seen))
                .min(6u32.saturating_sub(pottaishici_mark_count(tx, player, zhanlie_def)));
            if gain == 0 {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob[2] = instance.state_blob[2].saturating_add(gain as u8);
            }
            run_effects(
                tx,
                vec![EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(zhanlie_def.raw()),
                    delta: gain as i32,
                }],
            )
        }
        2 => {
            let card = materialize_virtual_card(tx, player, CardKind::Sha)?;
            let legal: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    crate::command::play::validate_sha_target_with_options(
                        tx, player, card, *target, false,
                    )
                    .is_ok()
                })
                .collect();
            if legal.is_empty() {
                remove_pottaishici_virtual_card(tx, card)?;
                return Ok(Vec::new());
            }
            let max = legal
                .len()
                .min(crate::query::sha_target_limit_for(tx.state(), player) as usize)
                .min(u8::MAX as usize) as u8;
            open_choice(
                tx,
                player,
                1,
                max,
                true,
                legal.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    POTTAISHICI_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    card.raw(),
                    skill.raw(),
                    hanzhan_def.raw(),
                    zhanlie_def.raw(),
                ],
            )
        }
        3 => {
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("势酣战目标已变化".into()));
            }
            let mut effects = Vec::new();
            for drawer in [player, target] {
                let x = pottaishici_x(tx, player, drawer, hanzhan_def, false);
                let count = x.saturating_sub(tx.state().hand_count(drawer)).min(3);
                if count > 0 {
                    effects.push(EffectRequest::DrawCards {
                        player: drawer,
                        count,
                    });
                }
            }
            effects.push(EffectRequest::ResolvePottaishici {
                player,
                target,
                action: 6,
                cards: Vec::new(),
                hanzhan_def,
                zhanlie_def,
                skill,
            });
            run_effects(tx, effects)
        }
        6 => {
            if tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive && !participant.dying)
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                let card = materialize_virtual_card(tx, player, CardKind::TrickJuedou)?;
                crate::command::play::play_as_kind(
                    tx,
                    player,
                    card,
                    CardKind::TrickJuedou,
                    &[target],
                    false,
                    false,
                    false,
                    false,
                )?;
            }
            Ok(Vec::new())
        }
        4 => {
            let wounded = tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| (participant.hp as u32) < participant.max_hp);
            let has_hanzhan = pottaishici_skill_id(tx, player, hanzhan_def).is_some();
            let has_zhanlie = pottaishici_skill_id(tx, player, zhanlie_def).is_some();
            let mut options = Vec::new();
            if wounded {
                options.push(0);
            }
            if has_hanzhan || has_zhanlie {
                options.push(1);
            }
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
                    POTTAISHICI_CUSTOM_CHOICE,
                    4,
                    player.raw(),
                    skill.raw(),
                    hanzhan_def.raw(),
                    zhanlie_def.raw(),
                ],
            )
        }
        5 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                if instance.state_blob.len() >= 8 {
                    instance.state_blob[3..8].fill(0);
                }
            }
            draw_cards(tx, player, 3)
        }
        7 => {
            let card = cards
                .first()
                .copied()
                .ok_or_else(|| EffectError::EngineFault("势战烈响应牌丢失".into()))?;
            begin_pottaishici_response_cost(tx, player, card).map(|_| Vec::new())
        }
        _ => Err(EffectError::ContentFault("势太史慈 action 非法".into())),
    }
}

pub(crate) fn complete_pottaishici_sha(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    targets: &[PlayerId],
    choices: u32,
    unrespondable: &[PlayerId],
) -> Result<(), EffectError> {
    if choices & 8 != 0 {
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            if instance.state_blob.len() < 8 {
                instance.state_blob.resize(8, 0);
            }
            instance.state_blob[3] = 1;
            instance.state_blob[4..8].copy_from_slice(&card.raw().to_le_bytes());
        }
    }
    if choices & 4 != 0 {
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            if instance.state_blob.len() < 14 {
                instance.state_blob.resize(14, 0);
            }
            instance.state_blob[8] = 1;
            instance.state_blob[9..13].copy_from_slice(&card.raw().to_le_bytes());
            instance.state_blob[13] = 0;
        }
    }
    let result = crate::command::play::play_as_kind_with_extra_damage_flags_target_bonus(
        tx,
        player,
        card,
        CardKind::Sha,
        targets,
        false,
        false,
        unrespondable,
        true,
        false,
        0,
        &[],
        u32::from(choices & 2 != 0),
        u8::from(choices & 1 != 0),
    );
    if result
        .as_ref()
        .is_err_and(|error| !matches!(error, EffectError::ChoiceRequested))
    {
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            if instance.state_blob.len() >= 14 {
                instance.state_blob[8..14].fill(0);
            }
            if instance.state_blob.len() >= 8 {
                instance.state_blob[3..8].fill(0);
            }
        }
        remove_pottaishici_virtual_card(tx, card)?;
    }
    result
}

pub(crate) fn open_pottaishici_response_cost(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    targets: &[PlayerId],
    mut index: usize,
) -> Result<(), EffectError> {
    while index < targets.len() {
        let target = targets[index];
        let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
        if options.is_empty() {
            crate::command::play::make_saved_sha_target_unrespondable(tx, player, card, target)?;
            index += 1;
            continue;
        }
        let mut payload = vec![
            POTTAISHICI_CUSTOM_CHOICE,
            3,
            player.raw(),
            card.raw(),
            skill.raw(),
            targets.len() as u32,
            index as u32,
        ];
        payload.extend(targets.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            target,
            0,
            1,
            true,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            payload,
        )
        .map(|_| ());
    }
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        if instance.state_blob.len() >= 14 {
            instance.state_blob[8..14].fill(0);
        }
    }
    Ok(())
}

pub(crate) fn pottaishici_response_cost_armed(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> bool {
    tx.state().skills.sorted_live().into_iter().any(|skill| {
        skill.owner == player
            && skill.state_blob.len() >= 14
            && skill.state_blob[8] == 1
            && u32::from_le_bytes(skill.state_blob[9..13].try_into().unwrap()) == card.raw()
    })
}

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
