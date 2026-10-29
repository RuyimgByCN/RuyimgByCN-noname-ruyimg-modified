//! Effect primitives for `resolve_ddd_jijian_kind_choice` and related helpers.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_ddd_zhaoang(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    stage: u8,
    card: CardId,
    kind: u8,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !ddd_fenji_phase_valid(tx, player, action)
            {
                return Ok(Vec::new());
            }
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.content_flags);
            let options = crate::command::play::source_free_ordinary_trick_kinds(
                tx.state(),
                player,
                used_mask,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options
                    .into_iter()
                    .map(|candidate| u32::from(candidate.tag()))
                    .collect(),
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_ZHAOANG_CHOICE,
                    0,
                    player.raw(),
                    u32::from(action),
                    skill.raw(),
                ],
            )
        }
        1 => {
            if !ddd_fenji_phase_valid(tx, player, action) {
                return Err(EffectError::Illegal("奋计阶段已经变化".into()));
            }
            let kind = CardKind::from_tag(kind)
                .ok_or_else(|| EffectError::Illegal("奋计锦囊牌名无效".into()))?;
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.content_flags);
            if !crate::command::play::source_free_ordinary_trick_kinds(
                tx.state(),
                player,
                used_mask,
            )
            .contains(&kind)
            {
                return Err(EffectError::Illegal("奋计锦囊已使用或已无合法目标".into()));
            }
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.set_content_flag(kind.tag(), true))
            {
                return Err(EffectError::ContentFault("奋计无法记录已用牌名".into()));
            }
            if action == 0 {
                tx.state_mut().turn.set_draw_phase_fixed_count(0);
            } else {
                tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 1);
            }
            let card = allocate_virtual_card_kind(tx, kind)?;
            match crate::command::play::skill_card_target_spec(tx, player, card)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    ddd_fenji_play(tx, player, action, card, kind, &targets, skill)?;
                    Ok(Vec::new())
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    let mut payload = vec![
                        RESOLVE_DDD_ZHAOANG_CHOICE,
                        1,
                        player.raw(),
                        u32::from(action),
                        card.raw(),
                        u32::from(kind.tag()),
                        skill.raw(),
                        u32::from(min),
                        u32::from(max),
                    ];
                    payload.extend(options.iter().map(|target| target.raw()));
                    open_choice(
                        tx,
                        player,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
                crate::command::play::SkillCardTargetSpec::Pairs(pairs) => {
                    let mut payload = vec![
                        RESOLVE_DDD_ZHAOANG_CHOICE,
                        2,
                        player.raw(),
                        u32::from(action),
                        card.raw(),
                        u32::from(kind.tag()),
                        skill.raw(),
                        pairs.len().min(u32::MAX as usize) as u32,
                    ];
                    for (first, second) in &pairs {
                        payload.push(first.raw());
                        payload.push(second.raw());
                    }
                    open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        (0..pairs.len().min(u32::MAX as usize) as u32).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
            }
        }
        2 => {
            if action == 0 {
                let damage = ddd_fenji_total_damage(tx.state()).saturating_sub(count);
                if damage == 0 {
                    Ok(Vec::new())
                } else {
                    run_effects(
                        tx,
                        vec![EffectRequest::DrawCards {
                            player,
                            count: damage,
                        }],
                    )
                }
            } else if action == 1 {
                let actual = count.min(
                    crate::query::discardable_hand_equip_cards(tx.state(), player)
                        .len()
                        .min(u32::MAX as usize) as u32,
                );
                if actual == 0 {
                    Ok(Vec::new())
                } else {
                    run_effects(
                        tx,
                        vec![EffectRequest::DiscardHandEquipCount {
                            player,
                            count: actual,
                            skill,
                        }],
                    )
                }
            } else {
                Err(EffectError::EngineFault("unknown 奋计 phase action".into()))
            }
        }
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选赵昂 continuation".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_zhaoang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload.first() != Some(&RESOLVE_DDD_ZHAOANG_CHOICE)
        || observer != PlayerId::new(payload[2])
        || cancelled
    {
        return Err(EffectError::Illegal("奋计选择无效".into()));
    }
    let player = PlayerId::new(payload[2]);
    let action = payload[3] as u8;
    match payload[1] {
        0 => {
            let [kind] = selections else {
                return Err(EffectError::Illegal("奋计必须选择一个锦囊牌名".into()));
            };
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddZhaoang {
                    player,
                    action,
                    stage: 1,
                    card: CardId::NONE,
                    kind: *kind as u8,
                    count: 0,
                    skill: engine_api::SkillId::new(payload[4]),
                }],
            )?;
        }
        1 => {
            if payload.len() < 9 {
                return Err(EffectError::EngineFault("truncated 奋计目标续体".into()));
            }
            let card = CardId::new(payload[4]);
            let kind = CardKind::from_tag(payload[5] as u8)
                .ok_or_else(|| EffectError::Illegal("奋计目标牌名无效".into()))?;
            let skill = engine_api::SkillId::new(payload[6]);
            let min = payload[7] as usize;
            let max = payload[8] as usize;
            let offered: Vec<_> = payload[9..].iter().copied().map(PlayerId::new).collect();
            let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            let crate::command::play::SkillCardTargetSpec::Select {
                min: current_min,
                max: current_max,
                options,
            } = crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("奋计目标形态已变化".into()));
            };
            if selected.len() < min
                || selected.len() > max
                || selected.len() < usize::from(current_min)
                || selected.len() > usize::from(current_max)
                || selected.iter().any(|target| {
                    !distinct.insert(*target)
                        || !offered.contains(target)
                        || !options.contains(target)
                })
            {
                return Err(EffectError::Illegal("奋计目标已失效".into()));
            }
            ddd_fenji_play(tx, player, action, card, kind, &selected, skill)?;
        }
        2 => {
            if payload.len() < 8 || selections.len() != 1 {
                return Err(EffectError::EngineFault("truncated 奋计借刀续体".into()));
            }
            let card = CardId::new(payload[4]);
            let kind = CardKind::from_tag(payload[5] as u8)
                .ok_or_else(|| EffectError::Illegal("奋计借刀牌名无效".into()))?;
            let skill = engine_api::SkillId::new(payload[6]);
            let pair_count = payload[7] as usize;
            if payload.len() != 8 + pair_count.saturating_mul(2)
                || selections[0] as usize >= pair_count
            {
                return Err(EffectError::Illegal("奋计借刀目标组合无效".into()));
            }
            let index = 8 + selections[0] as usize * 2;
            let selected = (
                PlayerId::new(payload[index]),
                PlayerId::new(payload[index + 1]),
            );
            let crate::command::play::SkillCardTargetSpec::Pairs(current) =
                crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("奋计借刀目标形态已变化".into()));
            };
            if !current.contains(&selected) {
                return Err(EffectError::Illegal("奋计借刀目标已失效".into()));
            }
            ddd_fenji_play(
                tx,
                player,
                action,
                card,
                kind,
                &[selected.0, selected.1],
                skill,
            )?;
        }
        _ => return Err(EffectError::EngineFault("unknown 奋计 choice".into())),
    }
    Ok(())
}

pub(crate) fn ddd_liuba_he_cards(state: &crate::state::GameState, player: PlayerId) -> Vec<CardId> {
    let mut cards = state.hand_cards(player);
    if let Some(zone) = state.zones.find(ZoneKind::Equip, player) {
        cards.extend(
            state
                .zones
                .get(zone)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied()),
        );
    }
    cards.sort_by_key(|card| card.raw());
    cards
}

pub(crate) fn ddd_liuba_open_distribution_subset(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    remaining: Vec<CardId>,
    assigned: Vec<(CardId, PlayerId)>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if remaining.is_empty() {
        let (cards, targets): (Vec<_>, Vec<_>) = assigned.into_iter().unzip();
        return run_effects(
            tx,
            vec![EffectRequest::ResolveDddLiuba {
                player,
                actor: player,
                action: 14,
                cards,
                targets,
                skill,
            }],
        );
    }
    if remaining.iter().any(|card| {
        !tx.state().cards.get(*card).is_some_and(|card| {
            tx.state()
                .zones
                .get(card.zone)
                .is_some_and(|zone| zone.kind == ZoneKind::Processing)
        })
    }) {
        return Err(EffectError::Illegal("丰政待分配牌已离开处理区".into()));
    }
    let mut payload = vec![
        RESOLVE_DDD_LIUBA_CHOICE,
        1,
        player.raw(),
        skill.raw(),
        remaining.len() as u32,
    ];
    payload.extend(remaining.iter().map(|card| card.raw()));
    payload.push(assigned.len() as u32);
    for (card, target) in assigned {
        payload.extend([card.raw(), target.raw()]);
    }
    if remaining.len() == 1 {
        payload[1] = 2;
        payload.push(1);
        payload.push(remaining[0].raw());
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            payload,
        );
    }
    open_choice(
        tx,
        player,
        1,
        remaining.len().min(u8::MAX as usize) as u8,
        false,
        remaining.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn ddd_liuba_finish_distribution(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    assigned: Vec<(CardId, PlayerId)>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("丰政处理区不存在".into()))?;
    let mut groups = std::collections::BTreeMap::<PlayerId, Vec<CardId>>::new();
    for (card, target) in assigned {
        if !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card| card.zone == processing)
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target| target.alive)
        {
            return Err(EffectError::Illegal("丰政最终分配状态失效".into()));
        }
        groups.entry(target).or_default().push(card);
    }
    let lose = groups.values().any(|cards| cards.len() > 2);
    let mut events = Vec::new();
    for (target, cards) in groups {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::EngineFault("丰政目标手牌区不存在".into()))?;
        events.extend(move_cards(tx, cards, hand, Some(processing))?);
    }
    if lose {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::RemoveSkill { skill });
    }
    let _ = player;
    Ok(events)
}

pub(crate) fn ddd_liuba_open_yulv_exchange(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand = tx.state().hand_cards(player);
    if hand.is_empty() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveDddLiuba {
                player,
                actor: player,
                action: 13,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            });
        return run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }]);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        hand.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![RESOLVE_DDD_LIUBA_CHOICE, 6, player.raw(), skill.raw()],
    )
}
