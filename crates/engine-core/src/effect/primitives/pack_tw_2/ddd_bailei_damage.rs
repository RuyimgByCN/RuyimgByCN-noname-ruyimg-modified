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
