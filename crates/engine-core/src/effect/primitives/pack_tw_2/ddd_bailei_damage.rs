//! Effect primitives for `ddd_bailei_damage` and related helpers.

use super::*;

pub(crate) fn ddd_bailei_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDddZhangkai {
            player,
            actor: target,
            card: CardId::NONE,
            cards: Vec::new(),
            amount: 0,
            action: 5,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
}

pub(crate) fn resolve_ddd_zhangkai(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    cards: Vec<CardId>,
    amount: u32,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action != 5 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let recovery_id = tx.state().resolution.current_recovery_id;
            let valid = actor != player
                && recovery_id != 0
                && amount > 0
                && tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32)
                && tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.def.raw() == DDD_JIEXING_DEF && instance.lifetime_fire_count > 0
                });
            if !valid {
                return Ok(Vec::new());
            }
            tx.state_mut().skills.get_mut(skill).unwrap().content_flags |=
                DDD_JIEXING_IGNORE_STOLEN_RECOVERY;
            let mut events = apply_effect(tx, EffectRequest::PreventRecovery { recovery_id })?;
            events.extend(apply_effect(
                tx,
                EffectRequest::RecoverHp {
                    source: player,
                    player,
                    amount,
                },
            )?);
            Ok(events)
        }
        1 => {
            if actor == player
                || card.is_none()
                || tx.state().skills.get(skill).is_none_or(|instance| {
                    instance.def.raw() != DDD_JIEXING_DEF || instance.lifetime_fire_count == 0
                })
            {
                return Ok(Vec::new());
            }
            let movable = ddd_zhangkai_equip_materials(tx, player, actor, &cards);
            if movable.is_empty() {
                return Ok(Vec::new());
            }
            crate::command::play::cancel_saved_card_use(tx, actor, card)?;
            ddd_zhangkai_equip(tx, player, movable)
        }
        2 => {
            let Some(instance) = tx.state_mut().skills.get_mut(skill) else {
                return Ok(Vec::new());
            };
            if instance.def.raw() != DDD_JIEXING_DEF || instance.lifetime_fire_count == 0 {
                return Ok(Vec::new());
            }
            if instance.content_flags & DDD_JIEXING_IGNORE_STOLEN_RECOVERY != 0 {
                instance.content_flags &= !DDD_JIEXING_IGNORE_STOLEN_RECOVERY;
            } else {
                instance.lifetime_fire_count = 0;
            }
            Ok(Vec::new())
        }
        3 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                if instance.def.raw() == DDD_JIEXING_DEF {
                    instance.lifetime_fire_count = 0;
                }
            }
            Ok(Vec::new())
        }
        4 => {
            if tx
                .state()
                .skills
                .get(skill)
                .is_none_or(|instance| instance.def.raw() != DDD_BAILEI_DEF)
            {
                return Ok(Vec::new());
            }
            let leader = ddd_zhangkai_unique_max_equip(tx);
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|target| target.hp == 1)
                        || leader == Some(*target)
                            && !ddd_zhangkai_gainable(tx, player, *target).is_empty()
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
                vec![RESOLVE_DDD_ZHANGKAI_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        5 => {
            let valid = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && instance.def.raw() == DDD_BAILEI_DEF
            });
            if valid
                && tx
                    .state()
                    .players
                    .get(actor)
                    .is_none_or(|target| !target.alive)
            {
                return remove_skill(tx, skill);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::EngineFault("unknown 3D精选张闿 action".into())),
    }
}

pub(crate) fn resolve_ddd_zhangkai_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4
        || payload.first() != Some(&RESOLVE_DDD_ZHANGKAI_CHOICE)
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::Illegal("张闿选择无效".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if !originating_skill_is_enabled(tx, player, skill)
        || tx
            .state()
            .skills
            .get(skill)
            .is_none_or(|instance| instance.def.raw() != DDD_BAILEI_DEF)
    {
        return Err(EffectError::Illegal("拜泪技能已失效".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() != 4 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("拜泪目标选择无效".into()));
            }
            let target = PlayerId::new(selections[0]);
            let hp_one = tx
                .state()
                .players
                .get(target)
                .is_some_and(|target| target.alive && target.hp == 1);
            let gainable = ddd_zhangkai_gainable(tx, player, target);
            let max_equip = ddd_zhangkai_unique_max_equip(tx) == Some(target);
            if !hp_one && !(max_equip && !gainable.is_empty()) {
                return Err(EffectError::Illegal("拜泪目标已失效".into()));
            }
            if max_equip && !gainable.is_empty() {
                let mut next = vec![
                    RESOLVE_DDD_ZHANGKAI_CHOICE,
                    1,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                    u32::from(hp_one),
                ];
                next.extend(gainable.iter().map(|card| card.raw()));
                open_choice(
                    tx,
                    player,
                    u8::from(!hp_one),
                    1,
                    hp_one,
                    gainable.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    next,
                )?;
            } else {
                ddd_bailei_damage(tx, player, target, skill)?;
            }
            Ok(())
        }
        1 => {
            if payload.len() < 6 {
                return Err(EffectError::Illegal("拜泪取牌数据无效".into()));
            }
            let target = PlayerId::new(payload[4]);
            let may_damage = payload[5] == 1;
            if cancelled || selections.is_empty() {
                if !may_damage
                    || !tx
                        .state()
                        .players
                        .get(target)
                        .is_some_and(|target| target.alive && target.hp == 1)
                {
                    return Err(EffectError::Illegal("拜泪不能取消取牌".into()));
                }
                ddd_bailei_damage(tx, player, target, skill)?;
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("拜泪须选择一张牌".into()));
            }
            let card = CardId::new(selections[0]);
            if !payload[6..].contains(&card.raw())
                || !ddd_zhangkai_gainable(tx, player, target).contains(&card)
            {
                return Err(EffectError::Illegal("拜泪所选牌已失效".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("拜泪找不到手牌区".into()))?;
            move_cards(tx, vec![card], hand, Some(from))?;
            Ok(())
        }
        _ => Err(EffectError::Illegal("未知张闿选择阶段".into())),
    }
}

pub(crate) fn ddd_tongyu_deferred_wugu(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    mut update: impl FnMut(&mut Vec<PlayerId>, &mut Vec<PlayerId>),
) -> bool {
    let resolution = &mut tx.state_mut().resolution;
    for effect in &mut resolution.deferred_effects {
        if let EffectRequest::ContinueWugu {
            actor,
            source_card,
            targets,
            excluded,
            ..
        } = effect
        {
            if *actor == player && *source_card == card {
                update(targets, excluded);
                return true;
            }
        }
    }
    resolution
        .deferred_tail_effects
        .iter_mut()
        .find_map(|effect| match effect {
            EffectRequest::ContinueWugu {
                actor,
                source_card,
                targets,
                excluded,
                ..
            } if *actor == player && *source_card == card => {
                update(targets, excluded);
                Some(())
            }
            _ => None,
        })
        .is_some()
}

pub(crate) fn ddd_tongyu_reorder_target_triggers(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    card: CardId,
    order: &[PlayerId],
) {
    let triggers = &mut tx.state_mut().resolution.deferred_triggers;
    let positions: Vec<_> = triggers
        .iter()
        .enumerate()
        .filter_map(|(index, trigger)| {
            (trigger.active.skill_id == skill
                && trigger.context.kind == crate::trigger::EventKind::CardTargeted
                && trigger.context.card == Some(card))
            .then_some(index)
        })
        .collect();
    let selected: Vec<_> = order
        .iter()
        .filter_map(|target| {
            positions.iter().find_map(|index| {
                (triggers[*index].context.player == *target).then(|| triggers[*index].clone())
            })
        })
        .collect();
    if selected.len() == positions.len() {
        for (index, trigger) in positions.into_iter().zip(selected) {
            triggers[index] = trigger;
        }
    }
}

pub(crate) fn resolve_ddd_liangxi(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    cards: Vec<CardId>,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx
            .state()
            .skills
            .get(skill)
            .is_none_or(|instance| instance.def.raw() != DDD_TONGYU_DEF)
    {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_LIANGXI_CHOICE,
                    0,
                    player.raw(),
                    skill.raw(),
                    card.raw(),
                ],
            )?;
            Ok(Vec::new())
        }
        1 => {
            if actor.is_none() || card.is_none() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                actor,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_LIANGXI_CHOICE,
                    1,
                    player.raw(),
                    skill.raw(),
                    actor.raw(),
                    card.raw(),
                ],
            )?;
            Ok(Vec::new())
        }
        2 => {
            let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let cards: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| Some(state.zone) == discard)
                })
                .collect();
            let targets: Vec<_> = tx
                .state()
                .resolution
                .ddd_tongyu_rejectors(player)
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|participant| participant.alive)
                })
                .collect();
            if cards.is_empty() || targets.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DDD_LIANGXI_CHOICE,
                2,
                player.raw(),
                skill.raw(),
                cards.len() as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::EngineFault("unknown 3D精选梁习 action".into())),
    }
}

pub(crate) fn resolve_ddd_liangxi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload.first() != Some(&RESOLVE_DDD_LIANGXI_CHOICE) {
        return Err(EffectError::Illegal("梁习选择无效".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if !originating_skill_is_enabled(tx, player, skill)
        || tx
            .state()
            .skills
            .get(skill)
            .is_none_or(|instance| instance.def.raw() != DDD_TONGYU_DEF)
    {
        return Err(EffectError::Illegal("仝御技能已失效".into()));
    }
    match payload[1] {
        0 => {
            if observer != player || cancelled || selections.len() != 1 || selections[0] > 1 {
                return Err(EffectError::Illegal("仝御结算方向无效".into()));
            }
            let card = CardId::new(payload[4]);
            let mut order = Vec::new();
            if !ddd_tongyu_deferred_wugu(tx, player, card, |targets, _| {
                if let Some(start) = targets.iter().position(|target| *target == player) {
                    targets.rotate_left(start);
                }
                if selections[0] == 1 && targets.len() > 1 {
                    targets[1..].reverse();
                }
                order = targets.clone();
            }) {
                return Err(EffectError::Illegal("仝御五谷结算已失效".into()));
            }
            ddd_tongyu_reorder_target_triggers(tx, skill, card, &order);
            Ok(())
        }
        1 => {
            if payload.len() != 6
                || observer != PlayerId::new(payload[4])
                || cancelled
                || selections.len() != 1
                || selections[0] > 1
            {
                return Err(EffectError::Illegal("仝御目标选择无效".into()));
            }
            let target = PlayerId::new(payload[4]);
            let card = CardId::new(payload[5]);
            if selections[0] == 0 {
                run_effects(
                    tx,
                    vec![EffectRequest::AddSkillUntilTurnEnd {
                        player: target,
                        skill_def: engine_api::SkillDefId::new(DDD_TONGYU_BLOCKER_DEF),
                    }],
                )?;
            } else {
                if !tx
                    .state_mut()
                    .resolution
                    .record_ddd_tongyu_rejector(player, card, target)
                    || !ddd_tongyu_deferred_wugu(tx, player, card, |_, excluded| {
                        if !excluded.contains(&target) {
                            excluded.push(target);
                        }
                    })
                {
                    return Err(EffectError::Illegal("仝御目标已失效".into()));
                }
            }
            Ok(())
        }
        2 => {
            let count = payload[4] as usize;
            if payload.len() != 5 + count || observer != player {
                return Err(EffectError::Illegal("仝御余牌数据无效".into()));
            }
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("仝御须选择一名角色".into()));
            }
            let target = PlayerId::new(selections[0]);
            if !tx
                .state()
                .resolution
                .ddd_tongyu_rejectors(player)
                .any(|candidate| candidate == target)
                || tx
                    .state()
                    .players
                    .get(target)
                    .is_none_or(|participant| !participant.alive)
            {
                return Err(EffectError::Illegal("仝御赠牌目标已失效".into()));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("仝御找不到弃牌堆".into()))?;
            let cards: Vec<_> = payload[5..]
                .iter()
                .copied()
                .map(CardId::new)
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.zone == discard)
                })
                .collect();
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("仝御找不到目标手牌区".into()))?;
            if !cards.is_empty() {
                run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards,
                        to: hand,
                        from: Some(discard),
                    }],
                )?;
            }
            Ok(())
        }
        _ => Err(EffectError::Illegal("未知梁习选择阶段".into())),
    }
}

pub(crate) fn caoyi_helper_entries(blob: &[u8]) -> Option<Vec<(PlayerId, u8)>> {
    if !blob.starts_with(b"MYI1") || (blob.len() - 4) % 5 != 0 {
        return None;
    }
    blob[4..]
        .chunks_exact(5)
        .map(|entry| {
            let source = PlayerId::new(u32::from_le_bytes(entry[..4].try_into().ok()?));
            (entry[4] <= 1).then_some((source, entry[4]))
        })
        .collect()
}

pub(crate) fn add_caoyi_helper(
    tx: &mut Transaction,
    target: PlayerId,
    source: PlayerId,
    mode: u8,
    helper_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let existing = tx.state().players.get(target).and_then(|player| {
        player.skills.iter().copied().find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def == helper_def)
        })
    });
    let (skill, events) = match existing {
        Some(skill) => (skill, Vec::new()),
        None => grant_skill_instance(tx, target, helper_def, b"MYI1".to_vec())?,
    };
    let current = tx.state().turn.current;
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("Miyi helper disappeared".into()))?;
    if caoyi_helper_entries(&instance.state_blob).is_none() {
        return Err(EffectError::ContentFault(
            "invalid Miyi helper state".into(),
        ));
    }
    instance
        .state_blob
        .extend_from_slice(&source.raw().to_le_bytes());
    instance.state_blob.push(mode);
    instance.temporary_until_turn_end_of = current;
    Ok(events)
}

pub(crate) fn resolve_caoyi(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    action: u8,
    event_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Prepare
                || helper_skill_def.is_none()
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_CAOYI_CHOICE,
                    0,
                    player.raw(),
                    helper_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) || card.raw() > 1 {
                return Ok(Vec::new());
            }
            let alive = tx.state().players.alive_by_seat();
            let selected: std::collections::BTreeSet<_> = event_targets.iter().copied().collect();
            if selected.is_empty()
                || selected.len() != event_targets.len()
                || !selected.iter().all(|target| alive.contains(target))
            {
                return Err(EffectError::Illegal("stale Miyi targets".into()));
            }
            let mut events = Vec::new();
            let mut effects = Vec::new();
            for target in alive.into_iter().filter(|target| selected.contains(target)) {
                events.extend(add_caoyi_helper(
                    tx,
                    target,
                    player,
                    card.raw() as u8,
                    helper_skill_def,
                )?);
                effects.push(if card.raw() == 0 {
                    EffectRequest::RecoverHp {
                        source: player,
                        player: target,
                        amount: 1,
                    }
                } else {
                    EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }
                });
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .extend(effects);
            Ok(events)
        }
        10 => {
            let Some(instance) = tx.state().skills.get(skill).cloned() else {
                return Ok(Vec::new());
            };
            if instance.owner != player
                || instance.def != helper_skill_def
                || tx.state().turn.phase != Phase::End
            {
                return Ok(Vec::new());
            }
            let entries = caoyi_helper_entries(&instance.state_blob)
                .ok_or_else(|| EffectError::ContentFault("invalid Miyi helper state".into()))?;
            let events = remove_skill(tx, skill)?;
            if tx
                .state()
                .players
                .get(player)
                .is_some_and(|target| target.alive)
            {
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .extend(entries.into_iter().map(|(source, initial)| {
                        if initial == 0 {
                            EffectRequest::DealDamage {
                                source,
                                target: player,
                                amount: 1,
                                is_lose_hp: false,
                                nature: rule_api::damage_nature::NORMAL,
                            }
                        } else {
                            EffectRequest::RecoverHp {
                                source,
                                player,
                                amount: 1,
                            }
                        }
                    }));
            }
            Ok(events)
        }
        11 => {
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            if instance.owner == player && instance.def == helper_skill_def {
                remove_skill(tx, skill)
            } else {
                Ok(Vec::new())
            }
        }
        20 => {
            if !originating_skill_is_enabled(tx, player, skill) || event_targets.len() != 1 {
                return Ok(Vec::new());
            }
            let target = event_targets[0];
            if !crate::command::play::source_free_sha_can_target_without_distance(
                tx.state(),
                player,
                target,
            ) {
                return Ok(Vec::new());
            }
            let virtual_card = allocate_virtual_card_kind(tx, CardKind::Sha)?;
            tx.state_mut().resolution.register_sha_resolution(
                player,
                virtual_card,
                &[target],
                false,
            );
            if !tx
                .state_mut()
                .resolution
                .mark_sha_damage_source_none(player, virtual_card)
            {
                return Err(EffectError::EngineFault(
                    "Yinjun Sha resolution missing".into(),
                ));
            }
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("Yinjun skill disappeared".into()))?;
            instance.state_blob = b"CYJ1".to_vec();
            instance
                .state_blob
                .extend_from_slice(&virtual_card.raw().to_le_bytes());
            let result = crate::command::play::handle_virtual_sha_with_policies_and_bonus(
                tx,
                player,
                virtual_card,
                &[target],
                true,
                false,
                0,
            );
            if result
                .as_ref()
                .is_err_and(|error| *error != EffectError::ChoiceRequested)
            {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.state_blob.clear();
                }
            }
            result.map(|_| Vec::new())
        }
        21 => {
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            if instance.owner != player
                || instance.state_blob.len() != 8
                || !instance.state_blob.starts_with(b"CYJ1")
                || u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()) != card.raw()
            {
                return Ok(Vec::new());
            }
            let hp = tx.state().players.get(player).map_or(0, |owner| owner.hp);
            let instance = tx.state_mut().skills.get_mut(skill).unwrap();
            instance.state_blob.clear();
            instance.turn_fire_count = instance.turn_fire_count.saturating_add(1);
            if instance.turn_fire_count as i32 > hp {
                instance.turn_disabled = true;
                tx.state_mut().rebuild_derived_indexes();
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown Caoyi resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_caoyi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload.first() != Some(&RESOLVE_CAOYI_CHOICE) {
        return Err(EffectError::EngineFault("truncated Caoyi choice".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let helper_skill_def = engine_api::SkillDefId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    if observer != player {
        return Err(EffectError::Illegal("Caoyi choice observer changed".into()));
    }
    match stage {
        0 => {
            if cancelled || selections.len() != 1 || selections[0] > 1 {
                return Err(EffectError::Illegal("invalid Miyi branch".into()));
            }
            let targets = tx.state().players.alive_by_seat();
            open_choice(
                tx,
                player,
                1,
                targets.len().min(u8::MAX as usize) as u8,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_CAOYI_CHOICE,
                    1,
                    player.raw(),
                    helper_skill_def.raw(),
                    skill.raw(),
                    selections[0],
                ],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault("truncated Miyi targets".into()));
            }
            if cancelled && selections.is_empty() {
                return Ok(());
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveCaoyi {
                    player,
                    card: CardId::new(payload[5]),
                    helper_skill_def,
                    action: 1,
                    event_targets: selections.iter().copied().map(PlayerId::new).collect(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Caoyi choice stage".into(),
        )),
    }
}

pub(crate) fn zhugeruoxue_movable_field_cards(tx: &Transaction) -> Vec<CardId> {
    let mut cards: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .flat_map(|source| crate::query::movable_field_cards_from(tx.state(), source))
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn allocate_virtual_card_kind(
    tx: &mut Transaction,
    kind: CardKind,
) -> Result<CardId, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .expect("validated processing zone")
        .cards
        .push(card);
    Ok(card)
}

pub(crate) fn zhugeruoxue_basic_options(
    tx: &mut Transaction,
    target: PlayerId,
) -> Result<Vec<u32>, EffectError> {
    let mut options = Vec::new();
    for kind in [
        CardKind::Sha,
        CardKind::Tao,
        CardKind::Jiu,
        CardKind::ShaFire,
        CardKind::ShaThunder,
    ] {
        let card = allocate_virtual_card_kind(tx, kind)?;
        let legal = matches!(
            crate::command::play::skill_card_target_spec(tx, target, card),
            Ok(crate::command::play::SkillCardTargetSpec::Immediate(_))
                | Ok(crate::command::play::SkillCardTargetSpec::Select { .. })
        );
        remove_unused_virtual_card(tx, card)?;
        if legal {
            options.push(u32::from(kind.tag()));
        }
    }
    Ok(options)
}

pub(crate) fn zhugeruoxue_basic_name_bit(kind: CardKind) -> Option<u32> {
    match kind {
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => Some(1),
        CardKind::Tao => Some(2),
        CardKind::Jiu => Some(4),
        _ => None,
    }
}

pub(crate) fn play_zhugeruoxue_virtual_basic(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
    targets: &[PlayerId],
    remaining: u32,
    used_mask: u32,
    repeated: bool,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveZhugeruoxue {
            player,
            target,
            card: CardId::NONE,
            action: 12,
            remaining,
            used_mask,
            repeated,
            skill,
        });
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, i32::from(kind.tag()));
    let result = crate::command::play::handle_skill_card(tx, target, card, targets);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_zhugeruoxue(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    remaining: u32,
    used_mask: u32,
    repeated: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let options = zhugeruoxue_movable_field_cards(tx);
            if options.is_empty() {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.use_count = instance.use_count.saturating_sub(1);
                    instance.play_phase_use_count = instance.play_phase_use_count.saturating_sub(1);
                }
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_ZHUGERUOXUE_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let destinations = crate::query::field_card_move_destinations(tx.state(), card);
            if destinations.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                destinations.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_ZHUGERUOXUE_CHOICE,
                    1,
                    player.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            if !crate::query::field_card_move_destinations(tx.state(), card).contains(&target) {
                return Err(EffectError::Illegal(
                    "Qiongying field-card destination changed".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Qiongying field card missing".into()))?;
            let zone_kind = tx
                .state()
                .zones
                .get(from)
                .map(|zone| zone.kind)
                .ok_or_else(|| EffectError::Illegal("Qiongying source zone missing".into()))?;
            let to =
                tx.state().zones.find(zone_kind, target).ok_or_else(|| {
                    EffectError::Illegal("Qiongying destination zone missing".into())
                })?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to,
                        from: Some(from),
                    },
                    EffectRequest::ResolveZhugeruoxue {
                        player,
                        target,
                        card,
                        action: 3,
                        remaining: 0,
                        used_mask: 0,
                        repeated: false,
                        skill,
                    },
                ],
            )
        }
        3 => {
            let Some(card_state) = tx.state().cards.get(card) else {
                return Ok(Vec::new());
            };
            let suit = crate::query::effective_card_suit(tx.state(), player, card_state);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .filter(|candidate| {
                    tx.state().cards.get(*candidate).is_some_and(|state| {
                        Some(state.zone) == hand
                            && crate::query::effective_card_suit(tx.state(), player, state) == suit
                    })
                })
                .collect();
            if options.is_empty() {
                let cards = tx.state().hand_cards(player);
                return if cards.is_empty() {
                    Ok(Vec::new())
                } else {
                    Ok(vec![DomainEvent::CardsRevealed {
                        cards,
                        to: Vec::new(),
                    }])
                };
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
                    RESOLVE_ZHUGERUOXUE_CHOICE,
                    3,
                    player.raw(),
                    crate::command::judge::suit_tag(suit),
                    skill.raw(),
                ],
            )
        }
        10 => {
            let targets = tx.state().players.alive_by_seat();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_ZHUGERUOXUE_CHOICE, 10, player.raw(), skill.raw()],
            )
        }
        11 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let total = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len())
                .max(1)
                .min(u32::MAX as usize) as u32;
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugeruoxue {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 12,
                    remaining: total,
                    used_mask: 0,
                    repeated: false,
                    skill,
                }],
            )
        }
        12 => {
            if remaining == 0
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                if !repeated {
                    return Ok(Vec::new());
                }
                let equip = tx.state().zones.find(ZoneKind::Equip, target);
                let cards: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), target)
                    .into_iter()
                    .filter(|card| {
                        tx.state()
                            .cards
                            .get(*card)
                            .is_some_and(|state| Some(state.zone) == equip)
                    })
                    .collect();
                return if cards.is_empty() {
                    Ok(Vec::new())
                } else {
                    discard_cards(tx, target, cards)
                };
            }
            let options = zhugeruoxue_basic_options(tx, target)?;
            if options.is_empty() {
                return resolve_zhugeruoxue(
                    tx,
                    player,
                    target,
                    CardId::NONE,
                    12,
                    0,
                    used_mask,
                    repeated,
                    skill,
                );
            }
            let mut payload = vec![
                RESOLVE_ZHUGERUOXUE_CHOICE,
                12,
                player.raw(),
                target.raw(),
                remaining,
                used_mask,
                u32::from(repeated),
                skill.raw(),
            ];
            payload.extend(options.iter().copied());
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        13 => {
            let kind = u8::try_from(card.raw())
                .ok()
                .and_then(CardKind::from_tag)
                .filter(|kind| zhugeruoxue_basic_name_bit(*kind).is_some())
                .ok_or_else(|| EffectError::ContentFault("bad Nuanhui basic kind".into()))?;
            if !zhugeruoxue_basic_options(tx, target)?.contains(&u32::from(kind.tag())) {
                return Err(EffectError::Illegal(
                    "Nuanhui basic kind became illegal".into(),
                ));
            }
            let bit = zhugeruoxue_basic_name_bit(kind).unwrap();
            let repeated = repeated || used_mask & bit != 0;
            let used_mask = used_mask | bit;
            let remaining = remaining.saturating_sub(1);
            let virtual_card = allocate_virtual_card_kind(tx, kind)?;
            match crate::command::play::skill_card_target_spec(tx, target, virtual_card)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    play_zhugeruoxue_virtual_basic(
                        tx,
                        player,
                        target,
                        virtual_card,
                        kind,
                        &targets,
                        remaining,
                        used_mask,
                        repeated,
                        skill,
                    )?;
                    Ok(Vec::new())
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    let mut payload = vec![
                        RESOLVE_ZHUGERUOXUE_CHOICE,
                        13,
                        player.raw(),
                        target.raw(),
                        virtual_card.raw(),
                        u32::from(kind.tag()),
                        remaining,
                        used_mask,
                        u32::from(repeated),
                        skill.raw(),
                        u32::from(min),
                        u32::from(max),
                    ];
                    payload.extend(options.iter().map(|target| target.raw()));
                    open_choice(
                        tx,
                        target,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
                crate::command::play::SkillCardTargetSpec::Pairs(_) => {
                    Err(EffectError::ContentFault(
                        "Nuanhui basic card cannot require paired targets".into(),
                    ))
                }
            }
        }
        _ => Err(EffectError::ContentFault(
            "unknown Zhugeruoxue resolver action".into(),
        )),
    }
}
