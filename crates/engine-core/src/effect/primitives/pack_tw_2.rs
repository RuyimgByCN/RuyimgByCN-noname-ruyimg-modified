//! Effect primitive implementations — tw 2 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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

pub(crate) const RESOLVE_DDD_LIANGXI_CHOICE: u32 = 0xDCDD_6400;
pub(crate) const DDD_TONGYU_DEF: u32 = 2_302_761_270;
pub(crate) const DDD_TONGYU_BLOCKER_DEF: u32 = 3_861_572_400;

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

pub(crate) fn allocate_virtual_card_kind(tx: &mut Transaction, kind: CardKind) -> Result<CardId, EffectError> {
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

pub(crate) fn resolve_zhugeruoxue_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload.first() != Some(&RESOLVE_ZHUGERUOXUE_CHOICE) {
        return Err(EffectError::EngineFault(
            "truncated Zhugeruoxue choice".into(),
        ));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    match stage {
        0 => {
            let skill = engine_api::SkillId::new(payload[3]);
            if observer != player {
                return Err(EffectError::Illegal("Qiongying observer changed".into()));
            }
            if cancelled && selections.is_empty() {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.use_count = instance.use_count.saturating_sub(1);
                    instance.play_phase_use_count = instance.play_phase_use_count.saturating_sub(1);
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Qiongying requires one field card".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !zhugeruoxue_movable_field_cards(tx).contains(&card) {
                return Err(EffectError::Illegal("Qiongying field card changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugeruoxue {
                    player,
                    target: PlayerId::NONE,
                    card,
                    action: 1,
                    remaining: 0,
                    used_mask: 0,
                    repeated: false,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() != 5 || observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Qiongying destination".into()));
            }
            let card = CardId::new(payload[3]);
            let target = PlayerId::new(selections[0]);
            if !crate::query::field_card_move_destinations(tx.state(), card).contains(&target) {
                return Err(EffectError::Illegal("Qiongying destination changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugeruoxue {
                    player,
                    target,
                    card,
                    action: 2,
                    remaining: 0,
                    used_mask: 0,
                    repeated: false,
                    skill: engine_api::SkillId::new(payload[4]),
                }],
            )
            .map(|_| ())
        }
        3 => {
            if payload.len() != 5 || observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Qiongying discard".into()));
            }
            let card = CardId::new(selections[0]);
            let suit = crate::command::judge::suit_from_tag(payload[3]);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card)
                || !tx.state().cards.get(card).is_some_and(|state| {
                    Some(state.zone) == hand
                        && crate::query::effective_card_suit(tx.state(), player, state) == suit
                })
            {
                return Err(EffectError::Illegal("Qiongying discard changed".into()));
            }
            discard_cards(tx, player, vec![card]).map(|_| ())
        }
        10 => {
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Nuanhui target".into()));
            }
            let target = PlayerId::new(selections[0]);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Nuanhui target changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugeruoxue {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 11,
                    remaining: 0,
                    used_mask: 0,
                    repeated: false,
                    skill: engine_api::SkillId::new(payload[3]),
                }],
            )
            .map(|_| ())
        }
        12 => {
            if payload.len() < 9 {
                return Err(EffectError::EngineFault(
                    "truncated Nuanhui card choice".into(),
                ));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[7]);
            if observer != target {
                return Err(EffectError::Illegal("Nuanhui chooser changed".into()));
            }
            if cancelled && selections.is_empty() {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveZhugeruoxue {
                        player,
                        target,
                        card: CardId::NONE,
                        action: 12,
                        remaining: 0,
                        used_mask: payload[5],
                        repeated: payload[6] != 0,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            if selections.len() != 1 || !payload[8..].contains(&selections[0]) {
                return Err(EffectError::Illegal("invalid Nuanhui basic choice".into()));
            }
            if !zhugeruoxue_basic_options(tx, target)?.contains(&selections[0]) {
                return Err(EffectError::Illegal("Nuanhui basic choice changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugeruoxue {
                    player,
                    target,
                    card: CardId::new(selections[0]),
                    action: 13,
                    remaining: payload[4],
                    used_mask: payload[5],
                    repeated: payload[6] != 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        13 => {
            if payload.len() < 13 || cancelled {
                return Err(EffectError::Illegal("invalid Nuanhui target choice".into()));
            }
            let target = PlayerId::new(payload[3]);
            let card = CardId::new(payload[4]);
            let kind = u8::try_from(payload[5])
                .ok()
                .and_then(CardKind::from_tag)
                .filter(|kind| zhugeruoxue_basic_name_bit(*kind).is_some())
                .ok_or_else(|| EffectError::EngineFault("bad Nuanhui target kind".into()))?;
            let min = usize::try_from(payload[10]).unwrap_or(usize::MAX);
            let max = usize::try_from(payload[11]).unwrap_or(usize::MAX);
            let offered: Vec<_> = payload[12..].iter().copied().map(PlayerId::new).collect();
            if observer != target
                || selections.len() < min
                || selections.len() > max
                || selections
                    .iter()
                    .any(|selection| !payload[12..].contains(selection))
            {
                return Err(EffectError::Illegal(
                    "Nuanhui target selection changed".into(),
                ));
            }
            let crate::command::play::SkillCardTargetSpec::Select {
                min: current_min,
                max: current_max,
                options,
            } = crate::command::play::skill_card_target_spec(tx, target, card)?
            else {
                return Err(EffectError::Illegal("Nuanhui target shape changed".into()));
            };
            let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            if selected.len() < usize::from(current_min)
                || selected.len() > usize::from(current_max)
                || selected.iter().any(|candidate| {
                    !distinct.insert(*candidate)
                        || !options.contains(candidate)
                        || !offered.contains(candidate)
                })
            {
                return Err(EffectError::Illegal(
                    "Nuanhui targets became illegal".into(),
                ));
            }
            play_zhugeruoxue_virtual_basic(
                tx,
                player,
                target,
                card,
                kind,
                &selected,
                payload[6],
                payload[7],
                payload[8] != 0,
                engine_api::SkillId::new(payload[9]),
            )
        }
        _ => Err(EffectError::EngineFault(
            "unknown Zhugeruoxue choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twxiawei(
    tx: &mut Transaction,
    player: PlayerId,
    wangxing_skill_def: u32,
    action: u8,
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
        .ok_or_else(|| EffectError::Illegal("TW Xiawei skill disappeared".into()))?;
    match action {
        0 => {
            let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
            if let Some(reference) = tx
                .state_mut()
                .skill_piles
                .iter_mut()
                .find(|reference| reference.zone == pile)
            {
                reference.usable_as_hand = true;
            }
            let mut selected = Vec::new();
            for kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
                let Some(zone) = tx.state().zones.find(kind, PlayerId::NONE) else {
                    continue;
                };
                let cards = tx
                    .state()
                    .zones
                    .get(zone)
                    .map(|zone| zone.cards.clone())
                    .unwrap_or_default();
                for card in cards.into_iter().rev() {
                    if selected.len() == 2 {
                        break;
                    }
                    if tx
                        .state()
                        .cards
                        .get(card)
                        .is_some_and(|card| card.kind.is_basic())
                    {
                        selected.push((zone, card));
                    }
                }
                if selected.len() == 2 {
                    break;
                }
            }
            let mut events = Vec::new();
            for from in selected
                .iter()
                .map(|(zone, _)| *zone)
                .collect::<std::collections::BTreeSet<_>>()
            {
                let cards = selected
                    .iter()
                    .filter_map(|(zone, card)| (*zone == from).then_some(*card))
                    .collect();
                events.extend(move_cards_with_reason(
                    tx,
                    cards,
                    pile,
                    Some(from),
                    CardMoveReason::Unspecified,
                )?);
            }
            Ok(events)
        }
        1 => {
            let cards = tx.state().skill_pile_cards(player, skill_def);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("TW Xiawei discard pile missing".into()))?;
            move_cards_with_reason(tx, cards, discard, Some(pile), CardMoveReason::Discard)
        }
        2 if stage == 0 => open_choice(
            tx,
            player,
            0,
            1,
            true,
            vec![1, 2, 3, 4],
            ChoiceKind::Custom,
            ContinuationKind::ResolveTwxiawei,
            vec![2, player.raw(), skill.raw(), wangxing_skill_def],
        ),
        2 if (1..=4).contains(&stage) => {
            let count = u32::from(stage) + 1;
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("TW Xiawei draw pile missing".into()))?;
            while tx
                .state()
                .zones
                .get(draw)
                .map_or(0, |zone| zone.cards.len())
                < count as usize
            {
                let before = tx
                    .state()
                    .zones
                    .get(draw)
                    .map_or(0, |zone| zone.cards.len());
                reshuffle_discard_into_draw(tx)?;
                if tx
                    .state()
                    .zones
                    .get(draw)
                    .map_or(0, |zone| zone.cards.len())
                    == before
                {
                    break;
                }
            }
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .map(|zone| {
                    zone.cards
                        .iter()
                        .rev()
                        .take(count as usize)
                        .copied()
                        .collect()
                })
                .unwrap_or_default();
            let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
            if let Some(reference) = tx
                .state_mut()
                .skill_piles
                .iter_mut()
                .find(|reference| reference.zone == pile)
            {
                reference.usable_as_hand = true;
            }
            let mut events =
                move_cards_with_reason(tx, cards, pile, Some(draw), CardMoveReason::Unspecified)?;
            events.extend(run_effects(
                tx,
                vec![
                    EffectRequest::AddSkill {
                        player,
                        skill_def: engine_api::SkillDefId::new(wangxing_skill_def),
                    },
                    EffectRequest::AddMark {
                        player,
                        mark: engine_api::MarkDefId::new(wangxing_skill_def),
                        delta: i32::from(stage),
                    },
                ],
            )?);
            Ok(events)
        }
        3 => {
            increase_bound_damage_by_one(tx, tx.state().resolution.current_damage_id)?;
            Ok(Vec::new())
        }
        4 => {
            let count = tx
                .state()
                .players
                .get(player)
                .and_then(|participant| {
                    participant
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == wangxing_skill_def)
                        .map(|(_, count)| (*count).max(0) as usize)
                })
                .unwrap_or(0);
            if count == 0 {
                return remove_skill(tx, skill);
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.len() < count {
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::RemoveSkill { skill },
                        EffectRequest::RemoveMark {
                            player,
                            mark: engine_api::MarkDefId::new(wangxing_skill_def),
                            amount: count as u32,
                        },
                        EffectRequest::ChangeMaxHp { player, delta: -1 },
                    ],
                );
            }
            let mut payload = vec![
                4,
                player.raw(),
                skill.raw(),
                wangxing_skill_def,
                count as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                count as u8,
                count as u8,
                true,
                options.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwxiawei,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Xiawei resolver action/stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twbianfuren(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: u32,
    action: u8,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            let owner_is_max_max_hp = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter_map(|candidate| tx.state().players.get(candidate).map(|p| p.max_hp))
                .max()
                .is_some_and(|max_hp| {
                    tx.state()
                        .players
                        .get(player)
                        .is_some_and(|owner| owner.max_hp == max_hp)
                });
            let mut effects = Vec::new();
            if target != player {
                let damage_id = tx.state().resolution.current_damage_id;
                if damage_id == 0 {
                    return Err(EffectError::ContentFault(
                        "TW Wanwei used outside damage_before".into(),
                    ));
                }
                if !tx
                    .state()
                    .resolution
                    .prevented_damage_ids
                    .contains(&damage_id)
                {
                    tx.state_mut()
                        .resolution
                        .prevented_damage_ids
                        .push(damage_id);
                }
                effects.push(EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                });
            }
            if target == player || owner_is_max_max_hp {
                effects.push(EffectRequest::AddSkillUntilTurnEnd {
                    player,
                    skill_def: engine_api::SkillDefId::new(helper_skill_def),
                });
            }
            run_effects(tx, effects)
        }
        (1, 0) => {
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveTwbianfuren {
                    player,
                    target: PlayerId::NONE,
                    helper_skill_def,
                    action,
                    stage: 1,
                    cards: Vec::new(),
                    skill,
                },
            );
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
        }
        (1, 1) => use_twbianfuren_bottom_card(tx, player, skill),
        (2, 0) => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let hand_count = tx.state().hand_cards(player).len() as i32;
            let hand_limit = match crate::query::query(
                tx.state(),
                &rule_api::QueryRequest::HandLimit { player },
                &mut crate::query::QueryContext::new(),
            ) {
                rule_api::QueryResponse::Int(value) => value.max(0),
                _ => return Err(EffectError::Illegal("hand-limit query failed".into())),
            };
            let max = (hand_count - hand_limit).max(1) as usize;
            let max = options.len().min(max).min(u8::MAX as usize) as u8;
            let mut payload = vec![0, player.raw(), helper_skill_def, skill.raw()];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                max,
                true,
                options.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwbianfuren,
                payload,
            )
        }
        (2, 1) => finish_twyuejian_cards(tx, player, cards, Vec::new(), helper_skill_def),
        _ => Err(EffectError::ContentFault(
            "unknown TW Bianfuren resolver action/stage".into(),
        )),
    }
}

pub(crate) fn use_twbianfuren_bottom_card(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("TW Wanwei draw pile missing".into()))?;
    if tx
        .state()
        .zones
        .get(draw)
        .is_none_or(|zone| zone.cards.is_empty())
    {
        reshuffle_discard_into_draw(tx)?;
    }
    let Some(card) = tx
        .state()
        .zones
        .get(draw)
        .and_then(|zone| zone.cards.first().copied())
    else {
        return Ok(Vec::new());
    };
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("TW Wanwei hand missing".into()))?;
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: vec![card],
        to: Vec::new(),
    }];
    events.extend(move_cards(tx, vec![card], hand, Some(draw))?);
    match crate::command::play::skill_card_target_spec(tx, player, card) {
        Ok(crate::command::play::SkillCardTargetSpec::Immediate(targets)) => {
            events.extend(use_card(tx, player, card, targets)?);
        }
        Ok(crate::command::play::SkillCardTargetSpec::Select { min, max, options }) => {
            let mut payload = vec![
                3,
                player.raw(),
                card.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            events.extend(open_choice(
                tx,
                player,
                min,
                max,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwbianfuren,
                payload,
            )?);
        }
        _ => {
            move_cards(tx, vec![card], draw, Some(hand))?;
            let zone = tx
                .state_mut()
                .zones
                .get_mut(draw)
                .expect("validated draw pile");
            zone.cards.retain(|candidate| *candidate != card);
            zone.cards.insert(0, card);
        }
    }
    Ok(events)
}

pub(crate) fn finish_twyuejian_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    top: Vec<CardId>,
    helper_skill_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    if selected.len() != cards.len() {
        return Err(EffectError::Illegal(
            "duplicate TW Yuejian selected card".into(),
        ));
    }
    let mut seen_top = std::collections::BTreeSet::new();
    if top
        .iter()
        .any(|card| !selected.contains(card) || !seen_top.insert(*card))
    {
        return Err(EffectError::Illegal("invalid TW Yuejian top cards".into()));
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    if cards.iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_none_or(|state| state.owner != player || ![hand, equip].contains(&Some(state.zone)))
    }) {
        return Err(EffectError::Illegal(
            "TW Yuejian selected card moved".into(),
        ));
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("TW Yuejian draw pile missing".into()))?;
    let mut events = Vec::new();
    for from in [hand, equip].into_iter().flatten() {
        let group: Vec<_> = cards
            .iter()
            .copied()
            .filter(|card| tx.state().cards.get(*card).is_some_and(|c| c.zone == from))
            .collect();
        if !group.is_empty() {
            events.extend(move_cards(tx, group, draw, Some(from))?);
        }
    }
    let top_set: std::collections::BTreeSet<_> = top.iter().copied().collect();
    let bottom: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| !top_set.contains(card))
        .collect();
    let zone = tx
        .state_mut()
        .zones
        .get_mut(draw)
        .expect("validated draw pile");
    zone.cards.retain(|card| !selected.contains(card));
    zone.cards.splice(0..0, bottom);
    zone.cards.extend(top);
    let mut effects = Vec::new();
    if cards.len() >= 3 {
        effects.push(EffectRequest::ChangeMaxHp { player, delta: 1 });
    }
    if cards.len() >= 2 {
        effects.push(EffectRequest::RecoverHp {
            source: player,
            player,
            amount: 1,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player,
        skill_def: engine_api::SkillDefId::new(helper_skill_def),
    });
    effects.push(EffectRequest::AddMark {
        player,
        mark: engine_api::MarkDefId::new(helper_skill_def),
        delta: 1,
    });
    events.extend(run_effects(tx, effects)?);
    Ok(events)
}

pub(crate) fn resolve_twfeiyi(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    stage: u8,
    _card: CardId,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            if first_shared_pile_card_by_kind(tx, CardKind::TrickTiaojiyanmei)?.is_some() {
                return gain_card_by_kind(tx, player, CardKind::TrickTiaojiyanmei.tag());
            }
            let used: Vec<_> = tx
                .state()
                .cards
                .sorted_live()
                .into_iter()
                .filter(|card| card.kind == CardKind::TrickTiaojiyanmei)
                .map(|card| card.suit)
                .collect();
            let missing: Vec<_> = [
                engine_api::Suit::Spade,
                engine_api::Suit::Heart,
                engine_api::Suit::Club,
                engine_api::Suit::Diamond,
            ]
            .into_iter()
            .filter(|suit| !used.contains(suit))
            .collect();
            let Some(suit) = (!missing.is_empty()).then(|| {
                let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
                let suit = missing[rng.gen_range_u32(missing.len() as u32) as usize];
                tx.state_mut().rng = rng.into_state();
                suit
            }) else {
                return Ok(Vec::new());
            };
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("TW Shengxi hand missing".into()))?;
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::NONE,
                kind: CardKind::TrickTiaojiyanmei,
                suit,
                rank: 6,
                weapon_range: 0,
                zone: hand,
                owner: player,
                face_up: false,
            });
            tx.state_mut()
                .zones
                .get_mut(hand)
                .expect("validated hand")
                .cards
                .push(card);
            Ok(vec![DomainEvent::CardsMoved {
                cards: vec![card],
                from: ZoneId::NONE,
                to: hand,
                reason: engine_api::CardMoveReason::Unspecified,
            }])
        }
        (1, 0) => open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![
                CardKind::TrickWuzhong.tag().into(),
                CardKind::TrickGuohe.tag().into(),
                CardKind::TrickWuxie.tag().into(),
            ],
            ChoiceKind::Custom,
            ContinuationKind::ResolveTwfeiyi,
            vec![1, player.raw(), skill.raw()],
        ),
        (2, 0) => {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("TW Kuanji discard pile missing".into()))?;
            let mut cards: Vec<_> = event_cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.zone == discard)
                })
                .collect();
            cards.sort_unstable_by_key(|card| card.raw());
            cards.dedup();
            if cards.is_empty()
                || !tx
                    .state()
                    .players
                    .iter_live()
                    .any(|target| target.alive && target.id != player)
            {
                return Ok(Vec::new());
            }
            let mut payload = vec![2, player.raw(), skill.raw()];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                cards.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwfeiyi,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Feiyi resolver action/stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twchenzhen(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    helper_skill_def: u32,
    record_skill_def: u32,
    action: u8,
    stage: u8,
    card_kind: u8,
    card: CardId,
    branch: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            twmuyue_card_kinds()
                .into_iter()
                .map(|kind| kind.tag().into())
                .collect(),
            ChoiceKind::Custom,
            ContinuationKind::ResolveTwchenzhen,
            vec![0, player.raw(), helper_skill_def, skill.raw()],
        ),
        (0, 1) => {
            let has_helper = player_has_skill_def(
                tx,
                player,
                engine_api::SkillDefId::new(helper_skill_def),
                true,
            );
            if has_helper {
                run_effects(
                    tx,
                    vec![EffectRequest::RemoveSkillByDef {
                        player,
                        skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    }],
                )?;
                return open_twmuyue_target(tx, player, card_kind, false, helper_skill_def, skill);
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                1,
                player.raw(),
                helper_skill_def,
                skill.raw(),
                card_kind.into(),
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwchenzhen,
                payload,
            )
        }
        (0, 2) => {
            let declared = CardKind::from_tag(card_kind)
                .ok_or_else(|| EffectError::Illegal("bad TW Muyue card kind".into()))?;
            let selected = tx
                .state()
                .cards
                .get(card)
                .ok_or_else(|| EffectError::Illegal("TW Muyue discard card missing".into()))?;
            let matches =
                crate::query::effective_card_kind(tx.state(), player, selected) == declared;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveTwchenzhen {
                    player,
                    target: PlayerId::NONE,
                    helper_skill_def,
                    record_skill_def: 0,
                    action: 0,
                    stage: 3,
                    card_kind,
                    card: CardId::NONE,
                    branch: u8::from(matches),
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                }],
            )
        }
        (0, 3) => open_twmuyue_target(tx, player, card_kind, branch != 0, helper_skill_def, skill),
        (1, 0) => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
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
                options.iter().map(|target| target.raw()).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwchenzhen,
                vec![
                    10,
                    player.raw(),
                    helper_skill_def,
                    record_skill_def,
                    skill.raw(),
                ],
            )
        }
        (2, 0) => {
            let count = tx
                .state()
                .players
                .get(player)
                .and_then(|participant| {
                    participant
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == helper_skill_def)
                        .map(|(_, count)| (*count).max(0) as usize)
                })
                .unwrap_or(0);
            let mut effects = vec![
                EffectRequest::RemoveSkill { skill },
                EffectRequest::RemoveMark {
                    player,
                    mark: engine_api::MarkDefId::new(helper_skill_def),
                    amount: count as u32,
                },
            ];
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let need = count.min(options.len());
            if need == 0 {
                return run_effects(tx, effects);
            }
            if options.len() <= need {
                effects.push(EffectRequest::DiscardCards {
                    player,
                    cards: options,
                });
                return run_effects(tx, effects);
            }
            let mut payload = vec![12, player.raw(), skill.raw(), helper_skill_def, need as u32];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                need as u8,
                need as u8,
                false,
                options.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwchenzhen,
                payload,
            )
        }
        (3, 0) => {
            let Some((previous_branch, hand_count)) = tx
                .state()
                .skills
                .get(skill)
                .and_then(|skill| skill.twchayi_state())
            else {
                return remove_skill(tx, skill);
            };
            let changed = tx.state().hand_count(player) != hand_count;
            let mut events = Vec::new();
            if changed {
                if previous_branch == 0 {
                    events.extend(run_effects(
                        tx,
                        vec![
                            EffectRequest::AddSkill {
                                player,
                                skill_def: engine_api::SkillDefId::new(helper_skill_def),
                            },
                            EffectRequest::AddMark {
                                player,
                                mark: engine_api::MarkDefId::new(helper_skill_def),
                                delta: 1,
                            },
                        ],
                    )?);
                } else {
                    let cards = tx.state().hand_cards(player);
                    if !cards.is_empty() {
                        events.push(DomainEvent::CardsRevealed {
                            cards,
                            to: Vec::new(),
                        });
                    }
                }
            }
            events.extend(remove_skill(tx, skill)?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Chenzhen resolver action/stage".into(),
        )),
    }
}

pub(crate) fn twmuyue_card_kinds() -> Vec<CardKind> {
    (1..=38)
        .filter_map(CardKind::from_tag)
        .filter(|kind| kind.is_basic() || kind.is_trick())
        .collect()
}

pub(crate) fn resolve_twqianju(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::EngineFault("TW Qianju equip zone missing".into()))?;
    for zone_kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let from = tx
            .state()
            .zones
            .find(zone_kind, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("shared card zone missing".into()))?;
        let card = tx.state().zones.get(from).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .and_then(|card| card.kind.equip_slot())
                    .is_some_and(|slot| {
                        tx.state()
                            .players
                            .get(player)
                            .is_some_and(|owner| owner.equip_slot_enabled(slot))
                            && tx.state().zones.get(equip).is_some_and(|zone| {
                                zone.cards.iter().all(|existing| {
                                    tx.state()
                                        .cards
                                        .get(*existing)
                                        .and_then(|card| card.kind.equip_slot())
                                        != Some(slot)
                                })
                            })
                    })
            })
        });
        if let Some(card) = card {
            return move_cards(tx, vec![card], equip, Some(from));
        }
    }
    Ok(Vec::new())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twqingxi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    count: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    crate::command::play::validate_saved_sha_target(tx, player, card, target)?;
    if stage == 0 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let target_equips =
                crate::query::discardable_equipment_cards(tx.state(), target, target);
            if target_equips.is_empty() {
                return resolve_twqingxi(tx, player, target, card, 1, 0, Vec::new(), skill);
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwqingxi,
                vec![0, player.raw(), target.raw(), card.raw(), skill.raw()],
            )
        }
        1 => {
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(1, |zone| zone.cards.len().max(1)) as u32;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwqingxi {
                    player,
                    target,
                    card,
                    stage: 4,
                    count: 0,
                    cards: Vec::new(),
                    skill,
                });
            draw_cards(tx, player, draw)
        }
        2 => {
            let target_equips =
                crate::query::discardable_equipment_cards(tx.state(), target, target);
            let discarded = target_equips.len().min(u8::MAX as usize) as u8;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwqingxi {
                    player,
                    target,
                    card,
                    stage: 3,
                    count: discarded,
                    cards: Vec::new(),
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: target_equips,
                }],
            )
        }
        3 => {
            let source_equips =
                crate::query::discardable_equipment_cards(tx.state(), target, player);
            let need = usize::from(count).min(source_equips.len());
            if need == 0 {
                return resolve_twqingxi(tx, player, target, card, 5, count, Vec::new(), skill);
            }
            if source_equips.len() <= need {
                return resolve_twqingxi(tx, player, target, card, 6, count, source_equips, skill);
            }
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                card.raw(),
                skill.raw(),
                count.into(),
            ];
            payload.extend(source_equips.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                need as u8,
                need as u8,
                false,
                source_equips.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwqingxi,
                payload,
            )
        }
        4 => make_current_sha_target_unrespondable(tx, player, card, target),
        5 => increase_current_sha_target_damage(tx, player, card, target, 1),
        6 => {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwqingxi {
                    player,
                    target,
                    card,
                    stage: 5,
                    count,
                    cards: Vec::new(),
                    skill,
                });
            run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])
        }
        _ => Err(EffectError::ContentFault("unknown TW Qingxi stage".into())),
    }
}

pub(crate) fn resolve_twzaoli(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    mut cards: Vec<CardId>,
    slots: Vec<u8>,
    count: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let discardable = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let mandatory: Vec<_> = discardable
                .iter()
                .copied()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        crate::query::effective_card_kind(tx.state(), player, card).is_equip()
                    })
                })
                .collect();
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let optional: Vec<_> = discardable
                .iter()
                .copied()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == hand
                            && !crate::query::effective_card_kind(tx.state(), player, card_state)
                                .is_equip()
                    })
                })
                .collect();
            if mandatory.is_empty() && optional.is_empty() {
                return Ok(Vec::new());
            }
            if optional.is_empty() {
                return resolve_twzaoli(tx, player, 1, mandatory, Vec::new(), 0, skill);
            }
            let mut payload = vec![player.raw(), skill.raw(), mandatory.len() as u32];
            payload.extend(mandatory.iter().map(|card| card.raw()));
            payload.extend(optional.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                optional.len().min(u8::MAX as usize) as u8,
                false,
                optional.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwzaoli,
                payload,
            )
        }
        1 => {
            cards.sort_unstable_by_key(|card| card.raw());
            cards.dedup();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let mut equip_slots: Vec<_> = cards
                .iter()
                .filter_map(|card| tx.state().cards.get(*card))
                .filter_map(|card| card.kind.equip_slot())
                .map(|slot| slot as u8)
                .collect();
            equip_slots.sort_unstable();
            equip_slots.dedup();
            let discarded = cards.len().min(u8::MAX as usize) as u8;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwzaoli {
                    player,
                    stage: 2,
                    cards: Vec::new(),
                    slots: equip_slots,
                    count: discarded,
                    skill,
                });
            run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])
        }
        2 => {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwzaoli {
                    player,
                    stage: 3,
                    cards: Vec::new(),
                    slots,
                    count: 0,
                    skill,
                });
            draw_cards(tx, player, u32::from(count))
        }
        3 => {
            let equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .ok_or_else(|| EffectError::EngineFault("TW Zaoli equip zone missing".into()))?;
            let mut events = Vec::new();
            let mut equipped = 0u8;
            for tag in slots {
                let Some(slot) = EquipSlot::from_tag(tag) else {
                    continue;
                };
                let mut found = None;
                for zone_kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
                    let from = tx
                        .state()
                        .zones
                        .find(zone_kind, PlayerId::NONE)
                        .ok_or_else(|| {
                            EffectError::EngineFault("shared card zone missing".into())
                        })?;
                    if let Some(card) = tx.state().zones.get(from).and_then(|zone| {
                        zone.cards.iter().rev().copied().find(|card| {
                            tx.state()
                                .cards
                                .get(*card)
                                .and_then(|card| card.kind.equip_slot())
                                == Some(slot)
                        })
                    }) {
                        found = Some((from, card));
                        break;
                    }
                }
                if let Some((from, card)) = found {
                    events.extend(move_cards(tx, vec![card], equip, Some(from))?);
                    equipped = equipped.saturating_add(1);
                }
            }
            if equipped > 2 {
                events.extend(run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )?);
            }
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown TW Zaoli stage".into())),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twzhengjian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: u32,
    action: u8,
    stage: u8,
    card: CardId,
    branch: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ResolveTwzhengjian,
            vec![0, player.raw(), helper_skill_def, skill.raw()],
        ),
        (1, 0) => {
            let (_, awakened, _) = tx
                .state()
                .skills
                .get(skill)
                .map_or((None, false, Vec::new()), |skill| skill.twzhengjian_state());
            if awakened {
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1, 2, 3],
                    ChoiceKind::Custom,
                    ContinuationKind::ResolveTwzhengjian,
                    vec![2, player.raw(), target.raw(), helper_skill_def, skill.raw()],
                );
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![1, player.raw(), target.raw(), helper_skill_def, skill.raw()];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwzhengjian,
                payload,
            )
        }
        (1, 1) => {
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("TW Zhengjian card missing".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("TW Zhengjian hand missing".into()))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveTwzhengjian {
                    player,
                    target,
                    helper_skill_def,
                    action: 1,
                    stage: 2,
                    card: CardId::NONE,
                    branch: 0,
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to: hand,
                    from,
                }],
            )
        }
        (1, 2) => {
            let player_count = tx.state().players.sorted_live().len();
            let threshold = player_count.div_ceil(2);
            let (selected, awakened, mut contributors) = tx
                .state()
                .skills
                .get(skill)
                .map_or((None, false, Vec::new()), |skill| skill.twzhengjian_state());
            if !contributors.contains(&target) {
                contributors.push(target);
            }
            let awaken_now = !awakened && contributors.len() >= threshold;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("TW Zhengjian skill disappeared".into()))?
                .set_twzhengjian_state(selected.unwrap_or(0), awakened || awaken_now, contributors);
            if awaken_now {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveTwzhengjian {
                        player,
                        target,
                        helper_skill_def,
                        action: 1,
                        stage: 4,
                        card: CardId::NONE,
                        branch: 0,
                        skill,
                    },
                );
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::RecoverHp {
                            source: player,
                            player,
                            amount: 2,
                        },
                        EffectRequest::AddSkill {
                            player,
                            skill_def: engine_api::SkillDefId::new(helper_skill_def),
                        },
                    ],
                );
            }
            resolve_twzhengjian(
                tx,
                player,
                target,
                helper_skill_def,
                1,
                4,
                CardId::NONE,
                0,
                skill,
            )
        }
        (1, 3) => {
            let (selected, awakened, contributors) = tx
                .state()
                .skills
                .get(skill)
                .map_or((None, false, Vec::new()), |skill| skill.twzhengjian_state());
            let mut events = Vec::new();
            if branch & 0b10 != 0 {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .expect("validated TW Zhengjian skill")
                    .set_twzhengjian_state(1 - selected.unwrap_or(0), awakened, contributors);
            }
            if branch & 0b01 != 0 {
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
            }
            Ok(events)
        }
        (1, 4) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ResolveTwzhengjian,
            vec![3, player.raw(), skill.raw()],
        ),
        _ => Err(EffectError::ContentFault(
            "unknown TW Zhengjian resolver stage".into(),
        )),
    }
}
