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
