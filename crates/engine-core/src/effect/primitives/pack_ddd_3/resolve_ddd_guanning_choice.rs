//! Effect primitives for `resolve_ddd_guanning_choice` and related helpers.

use super::*;

pub(crate) fn resolve_ddd_guanning_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 || payload.first() != Some(&RESOLVE_DDD_GUANNING_CHOICE) {
        return Err(EffectError::EngineFault("truncated 管宁 choice".into()));
    }
    let player = PlayerId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal("管宁选择者变化".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() != 5 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("游学目标选择无效".into()));
            }
            let source = PlayerId::new(payload[3]);
            let target = PlayerId::new(selections[0]);
            if ddd_guanning_mark_holder(tx.state()) != Some(source)
                || source == target
                || !tx.state().players.get(target).is_some_and(|participant| {
                    participant.alive
                        && !participant
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == DDD_YOUXUE_MARK && *count > 0)
                })
            {
                return Err(EffectError::Illegal("游学标记或目标已变化".into()));
            }
            let draw = ddd_guanning_distance(tx.state(), source, target);
            let mut effects = vec![
                EffectRequest::RemoveMark {
                    player: source,
                    mark: engine_api::MarkDefId::new(DDD_YOUXUE_MARK),
                    amount: 0,
                },
                EffectRequest::AddMark {
                    player: target,
                    mark: engine_api::MarkDefId::new(DDD_YOUXUE_MARK),
                    delta: 1,
                },
            ];
            if draw > 0 {
                effects.push(EffectRequest::DrawCards {
                    player,
                    count: draw,
                });
            }
            run_effects(tx, effects)?;
        }
        1 => {
            if payload.len() < 8 || cancelled {
                return Err(EffectError::Illegal("承经目标选择无效".into()));
            }
            let card = CardId::new(payload[3]);
            let kind = CardKind::from_tag(payload[4] as u8)
                .ok_or_else(|| EffectError::Illegal("承经牌名无效".into()))?;
            let skill = engine_api::SkillId::new(payload[5]);
            let min = payload[6] as usize;
            let max = payload[7] as usize;
            let offered: Vec<_> = payload[8..].iter().copied().map(PlayerId::new).collect();
            let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            let crate::command::play::SkillCardTargetSpec::Select {
                min: current_min,
                max: current_max,
                options,
            } = ddd_guanning_target_spec(tx, player, card, kind, skill)?
            else {
                return Err(EffectError::Illegal("承经目标形态已变化".into()));
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
                return Err(EffectError::Illegal("承经目标已失效".into()));
            }
            ddd_guanning_play(tx, player, card, kind, &selected, skill)?;
        }
        2 => {
            if payload.len() < 7 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("承经借刀选择无效".into()));
            }
            let card = CardId::new(payload[3]);
            let kind = CardKind::from_tag(payload[4] as u8)
                .ok_or_else(|| EffectError::Illegal("承经借刀牌名无效".into()))?;
            let skill = engine_api::SkillId::new(payload[5]);
            let pair_count = payload[6] as usize;
            if payload.len() != 7 + pair_count.saturating_mul(2)
                || selections[0] as usize >= pair_count
            {
                return Err(EffectError::Illegal("承经借刀目标组合无效".into()));
            }
            let index = 7 + selections[0] as usize * 2;
            let selected = (
                PlayerId::new(payload[index]),
                PlayerId::new(payload[index + 1]),
            );
            let crate::command::play::SkillCardTargetSpec::Pairs(current) =
                ddd_guanning_target_spec(tx, player, card, kind, skill)?
            else {
                return Err(EffectError::Illegal("承经借刀目标形态已变化".into()));
            };
            if !current.contains(&selected) {
                return Err(EffectError::Illegal("承经借刀目标已失效".into()));
            }
            ddd_guanning_play(tx, player, card, kind, &[selected.0, selected.1], skill)?;
        }
        3 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault("truncated 承经赠牌 choice".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal("取消承经赠牌时不得选择".into()));
                }
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("承经赠牌选择无效".into()));
            }
            let card = CardId::new(payload[3]);
            let target = PlayerId::new(payload[4]);
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("承经受赠者没有手牌区".into()))?;
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|source| source.zone == discard)
            {
                return Err(EffectError::Illegal("承经赠牌目标或牌已失效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::MoveCardsWithReason {
                    cards: vec![card],
                    to: hand,
                    from: Some(discard),
                    reason: CardMoveReason::Give,
                }],
            )?;
        }
        _ => return Err(EffectError::EngineFault("unknown 管宁 choice".into())),
    }
    Ok(())
}

pub(crate) fn ddd_caoshuang_card_name(
    state: &crate::state::GameState,
    actor: PlayerId,
    card: CardId,
) -> Option<(u8, u32)> {
    let card_state = state.cards.get(card)?;
    let kind = crate::query::effective_card_kind(state, actor, card_state);
    Some((
        kind.tag(),
        (kind == CardKind::Other)
            .then(|| card_state.def.raw())
            .unwrap_or(0),
    ))
}

pub(crate) fn resolve_ddd_caoshuang(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if actor == player
                || tx.state().turn.current != actor
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_CAOSHUANG_CHOICE,
                    0,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let Some((kind, def)) = ddd_caoshuang_card_name(tx.state(), actor, card) else {
                return Ok(Vec::new());
            };
            let matched = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.mark_ddd_zhuanshe_used(actor, kind, def));
            if !matched {
                return Ok(Vec::new());
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    crate::command::play::saved_card_extra_target_is_legal_ignoring_distance(
                        tx, actor, card, *target,
                    )
                })
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_CAOSHUANG_CHOICE,
                    1,
                    player.raw(),
                    actor.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let unused = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| {
                    instance
                        .ddd_zhuanshe_state()
                        .filter(|(stored_actor, _)| *stored_actor == actor)
                })
                .is_some_and(|(_, names)| names.iter().any(|(_, _, used)| !*used));
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.clear_ddd_zhuanshe_state();
            }
            if !unused {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_CAOSHUANG_CHOICE,
                    2,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            let recovery_id = tx.state().resolution.current_recovery_id;
            if recovery_id == 0 || !tx.state().hand_cards(player).is_empty() {
                return Ok(Vec::new());
            }
            let mut events = apply_effect(tx, EffectRequest::PreventRecovery { recovery_id })?;
            events.extend(apply_effect(
                tx,
                EffectRequest::DrawCards { player, count: 1 },
            )?);
            Ok(events)
        }
        _ => Err(EffectError::EngineFault("unknown 3D精选曹爽 action".into())),
    }
}

pub(crate) fn resolve_ddd_caoshuang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload.first() != Some(&RESOLVE_DDD_CAOSHUANG_CHOICE) {
        return Err(EffectError::EngineFault("truncated 曹爽 choice".into()));
    }
    let player = PlayerId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal("曹爽选择者变化".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() != 5 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("专摄赠牌选择无效".into()));
            }
            let actor = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let card = CardId::new(selections[0]);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if actor == player
                || !tx
                    .state()
                    .players
                    .get(actor)
                    .is_some_and(|target| target.alive)
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| Some(state.zone) == hand)
            {
                return Err(EffectError::Illegal("专摄赠牌上下文已变化".into()));
            }
            let card_state = tx.state().cards.get(card).unwrap();
            let kind = card_state.kind.tag();
            let def = (card_state.kind == CardKind::Other)
                .then(|| card_state.def.raw())
                .unwrap_or(0);
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, actor)
                .ok_or_else(|| EffectError::Illegal("专摄目标没有手牌区".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::RevealCards {
                        cards: vec![card],
                        to: Vec::new(),
                    },
                    EffectRequest::GiveCards {
                        cards: vec![card],
                        to,
                        from: hand.unwrap(),
                    },
                ],
            )?;
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("专摄技能已失效".into()))?;
            instance.record_ddd_zhuanshe_name(actor, kind, def);
        }
        1 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault(
                    "truncated 专摄加目标 choice".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("专摄须选择一个额外目标".into()));
            }
            crate::command::play::add_saved_card_target_ignoring_distance(
                tx,
                PlayerId::new(payload[3]),
                CardId::new(payload[4]),
                PlayerId::new(selections[0]),
            )?;
        }
        2 => {
            if payload.len() != 5 {
                return Err(EffectError::EngineFault("truncated 专摄伤害 choice".into()));
            }
            if !cancelled && selections == [1] {
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target: PlayerId::new(payload[3]),
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )?;
            }
        }
        _ => return Err(EffectError::EngineFault("unknown 曹爽 choice stage".into())),
    }
    Ok(())
}

pub(crate) fn resolve_ddd_xuelingyun(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    cards: Vec<CardId>,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("涟洏没有手牌区".into()))?;
            let gained: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| Some(card.zone) == discard)
                })
                .collect();
            let ceiling = (gained.len() == 1)
                .then(|| tx.state().cards.get(gained[0]).map(|card| card.rank))
                .flatten()
                .filter(|rank| *rank > 0);
            let events = if gained.is_empty() {
                Vec::new()
            } else {
                move_cards_with_reason(tx, gained, hand, discard, CardMoveReason::Unspecified)?
            };
            if let Some(rank) = ceiling {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.mark = u32::from(rank);
                }
            }
            Ok(events)
        }
        1 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.mark = 0;
            }
            Ok(Vec::new())
        }
        2 => {
            if let Some(card_state) = tx.state().cards.get(card) {
                let black = crate::query::card_color_mask_for_suit(
                    crate::query::effective_card_suit(tx.state(), actor, card_state),
                ) == 0b10;
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.record_ddd_anzhi_color(black);
                }
            }
            Ok(Vec::new())
        }
        3 => {
            let targets = tx.state().players.alive_by_seat();
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XUELINGYUN_CHOICE,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        4 => {
            crate::command::play::exclude_saved_card_target(tx, card, actor)?;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.clear_ddd_anzhi_target();
            }
            Ok(Vec::new())
        }
        5 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                let (_, stored_actor, _) = instance.ddd_anzhi_state();
                if stored_actor == actor {
                    instance.clear_ddd_anzhi_target();
                }
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选薛灵芸 action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xuelingyun_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4
        || payload.first() != Some(&RESOLVE_DDD_XUELINGYUN_CHOICE)
        || observer != PlayerId::new(payload[1])
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("暗织目标选择无效".into()));
    }
    let target = PlayerId::new(selections[0]);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|target| target.alive)
    {
        return Err(EffectError::Illegal("暗织目标已失效".into()));
    }
    let skill = engine_api::SkillId::new(payload[3]);
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("暗织技能已失效".into()))?;
    instance.arm_ddd_anzhi(PlayerId::new(payload[2]), target);
    Ok(())
}

pub(crate) fn ddd_shixing_female_available(state: &crate::state::GameState) -> bool {
    state.players.alive_by_seat().into_iter().any(|player| {
        state.players.get(player).is_some_and(|participant| {
            participant.hp >= i32::try_from(participant.max_hp).unwrap_or(i32::MAX)
                && state
                    .content
                    .character(participant.character_def)
                    .is_some_and(|character| matches!(character.sex.as_str(), "female" | "double"))
        })
    })
}

pub(crate) fn refresh_ddd_shixing(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    desired: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    const XIANGLE: u32 = 2_880_679_348;
    const JIUSHI: u32 = 4_293_489_189;
    const JIUSHI1: u32 = 3_647_393_232;
    const JIUSHI3: u32 = 4_156_803_269;
    const REZHIHENG: u32 = 2_455_708_599;

    let old = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.ddd_shixing_grants())
        .unwrap_or_default();
    let mut events = Vec::new();
    for derived in old {
        if tx.state().skills.get(derived).is_some() {
            events.extend(remove_skill(tx, derived)?);
        }
    }
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.mark = desired;
        instance.set_ddd_shixing_grants(&[]);
    }

    let mut defs = Vec::new();
    if desired & 1 != 0 {
        defs.push(XIANGLE);
    }
    if desired & 2 != 0 {
        defs.extend([JIUSHI, JIUSHI1, JIUSHI3]);
    }
    if desired == 0 {
        defs.push(REZHIHENG);
    }
    let mut grants = Vec::new();
    for raw in defs {
        let def = engine_api::SkillDefId::new(raw);
        if tx.state().content.skill_metadata(def).is_none() {
            return Err(EffectError::ContentFault(format!(
                "失兴依赖技能 {raw} 未加载"
            )));
        }
        let already_owned = tx.state().players.get(player).is_some_and(|owner| {
            owner.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == def)
            })
        });
        if !already_owned {
            let (derived, added) = grant_skill_instance(tx, player, def, Vec::new())?;
            grants.push(derived);
            events.extend(added);
        }
    }
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.set_ddd_shixing_grants(&grants);
    }
    Ok(events)
}
