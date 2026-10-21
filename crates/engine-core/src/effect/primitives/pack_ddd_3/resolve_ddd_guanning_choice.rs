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

pub(crate) fn resolve_ddd_liuhong(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let desired = u32::from(ddd_shixing_female_available(tx.state())) | 2;
            refresh_ddd_shixing(tx, player, skill, desired)
        }
        1 => {
            let trick = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.mark & 2);
            let desired = trick | u32::from(ddd_shixing_female_available(tx.state()));
            refresh_ddd_shixing(tx, player, skill, desired)
        }
        2 => {
            let desired = tx
                .state()
                .skills
                .get(skill)
                .map_or(0, |instance| instance.mark & 1);
            refresh_ddd_shixing(tx, player, skill, desired)
        }
        3 => {
            let alive = tx.state().players.alive_by_seat();
            let linked = alive
                .iter()
                .filter(|target| {
                    tx.state().players.get(**target).is_some_and(|participant| {
                        participant
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                    })
                })
                .count();
            let qun = alive
                .iter()
                .filter(|target| tx.state().player_group(**target) == Some("qun"))
                .count();
            let minimum = qun.saturating_sub(linked);
            let options: Vec<_> = alive
                .into_iter()
                .filter(|target| {
                    tx.state().players.get(*target).is_some_and(|participant| {
                        !participant
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                    })
                })
                .map(PlayerId::raw)
                .collect();
            if minimum == 0 || options.len() < minimum {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                u8::try_from(minimum).unwrap_or(u8::MAX),
                u8::try_from(options.len()).unwrap_or(u8::MAX),
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_LIUHONG_CHOICE,
                    0,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        4 => {
            let count = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| tx.state().player_group(*target) == Some("qun"))
                .count()
                .min(u32::MAX as usize) as u32;
            run_effects(tx, vec![EffectRequest::DrawCards { player, count }])
        }
        5 => {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                actor,
                player,
                rule_api::TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_LIUHONG_CHOICE,
                    1,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::EngineFault("unknown 3D精选刘宏 action".into())),
    }
}

pub(crate) fn resolve_ddd_liuhong_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload.first() != Some(&RESOLVE_DDD_LIUHONG_CHOICE) || cancelled {
        return Err(EffectError::Illegal("刘宏选择无效".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let actor = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal("刘宏技能已失效".into()));
    }
    match stage {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal("仅刘宏可选择党锢目标".into()));
            }
            let alive = tx.state().players.alive_by_seat();
            let linked = alive
                .iter()
                .filter(|target| {
                    tx.state().players.get(**target).is_some_and(|participant| {
                        participant
                            .marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                    })
                })
                .count();
            let minimum = alive
                .iter()
                .filter(|target| tx.state().player_group(**target) == Some("qun"))
                .count()
                .saturating_sub(linked);
            let mut distinct = std::collections::BTreeSet::new();
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            if targets.len() < minimum
                || targets.iter().any(|target| {
                    !distinct.insert(*target)
                        || !alive.contains(target)
                        || tx.state().players.get(*target).is_none_or(|participant| {
                            participant.marks.iter().any(|(mark, count)| {
                                *mark == crate::content::MARK_CHAIN && *count > 0
                            })
                        })
                })
            {
                return Err(EffectError::Illegal("党锢目标已变化".into()));
            }
            run_effects(
                tx,
                targets
                    .into_iter()
                    .map(|target| EffectRequest::AddMark {
                        player: target,
                        mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                        delta: 1,
                    })
                    .collect(),
            )?;
            Ok(())
        }
        1 => {
            if observer != actor || selections.len() != 1 {
                return Err(EffectError::Illegal("仅死亡角色可选择覆宗弃牌".into()));
            }
            let card = CardId::new(selections[0]);
            if tx.state().player_group(actor) != Some("qun")
                || !crate::query::discardable_target_cards(
                    tx.state(),
                    actor,
                    player,
                    rule_api::TargetCardScope::HandEquip,
                )
                .contains(&card)
            {
                return Err(EffectError::Illegal("覆宗弃牌目标已变化".into()));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("覆宗牌已失效".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("弃牌堆不存在".into()))?;
            move_cards_with_reason(tx, vec![card], discard, Some(from), CardMoveReason::Discard)?;
            Ok(())
        }
        _ => Err(EffectError::Illegal("未知刘宏选择阶段".into())),
    }
}

pub(crate) fn ddd_xiahouxuan_hand_equip_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(player);
    if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
        cards.extend(
            tx.state()
                .zones
                .get(equip)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().copied()),
        );
    }
    cards
}

pub(crate) fn commit_ddd_langhuai(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx
            .state()
            .skills
            .get(skill)
            .is_none_or(|instance| instance.def.raw() != DDD_LANGHUAI_DEF)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Draw
        || tx.state().turn.draw_phase_fixed_count().is_some()
    {
        return Ok(Vec::new());
    }

    let hand = tx.state().hand_cards(player);
    let mut suits = 0u8;
    for card in &hand {
        let Some(card) = tx.state().cards.get(*card) else {
            continue;
        };
        suits |= match crate::query::effective_card_suit(tx.state(), player, card) {
            Suit::Spade => 1,
            Suit::Heart => 2,
            Suit::Club => 4,
            Suit::Diamond => 8,
            Suit::None => 0,
        };
    }
    let branch = tx
        .state()
        .skills
        .get(skill)
        .map_or(0, |instance| instance.mark & 1);
    let distinct = suits.count_ones();
    let count = if branch == 0 { distinct } else { 4 - distinct };
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.mark ^= 1;
    }
    if let Some(xuanlun) =
        skill_instance_by_def(tx, player, engine_api::SkillDefId::new(DDD_XUANLUN_DEF))
    {
        if let Some(instance) = tx.state_mut().skills.get_mut(xuanlun) {
            instance.mark &= !DDD_XUANLUN_FORCE_LANGHUAI;
        }
    }
    tx.state_mut().turn.set_draw_phase_fixed_count(0);

    let mut events = Vec::new();
    if !hand.is_empty() {
        events.push(DomainEvent::CardsRevealed {
            cards: hand,
            to: Vec::new(),
        });
    }
    events.extend(draw_cards(tx, player, count)?);
    Ok(events)
}

pub(crate) fn open_ddd_xuanlun_card_choice(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = ddd_xiahouxuan_hand_equip_cards(tx, player);
    let count = options.len().min(4);
    if count == 0 {
        return Ok(Vec::new());
    }
    if options.len() <= 4 {
        let mut tokens: Vec<_> = options.iter().map(|card| card.raw()).collect();
        tokens.push(0);
        let mut payload = vec![
            RESOLVE_DDD_XIAHOUXUAN_CHOICE,
            3,
            player.raw(),
            skill.raw(),
            count as u32,
        ];
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            (count + 1) as u8,
            (count + 1) as u8,
            false,
            tokens,
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            payload,
        );
    }
    let mut payload = vec![
        RESOLVE_DDD_XIAHOUXUAN_CHOICE,
        2,
        player.raw(),
        skill.raw(),
        count as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        count as u8,
        count as u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn commit_ddd_xuanlun_arrangement(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let encoded = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.state_blob.clone())
        .unwrap_or_default();
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob.clear();
    }
    if encoded.is_empty() || encoded.len() % 4 != 0 {
        return Err(EffectError::Illegal("玄论牌堆排列已失效".into()));
    }
    let order: Vec<u32> = encoded
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect();
    let Some(split) = order.iter().position(|token| *token == 0) else {
        return Err(EffectError::Illegal("玄论牌堆排列缺少顶底分界".into()));
    };
    if order.iter().filter(|token| **token == 0).count() != 1 {
        return Err(EffectError::Illegal("玄论牌堆排列分界无效".into()));
    }
    let cards: Vec<_> = order
        .iter()
        .copied()
        .filter(|token| *token != 0)
        .map(CardId::new)
        .collect();
    let current = ddd_xiahouxuan_hand_equip_cards(tx, player);
    let mut unique = std::collections::BTreeSet::new();
    if cards.is_empty()
        || cards
            .iter()
            .any(|card| !unique.insert(*card) || !current.contains(card))
    {
        return Err(EffectError::Illegal("玄论牌堆排列中的牌已失效".into()));
    }

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("玄论找不到牌堆".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("玄论找不到手牌区".into()))?;
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    let hand_cards: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.zone == hand)
        })
        .collect();
    let equip_cards: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            equip.is_some_and(|equip| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.zone == equip)
            })
        })
        .collect();
    if hand_cards.len() + equip_cards.len() != cards.len() {
        return Err(EffectError::Illegal(
            "玄论只能排列自己的手牌或装备牌".into(),
        ));
    }

    let top: Vec<_> = order[..split].iter().copied().map(CardId::new).collect();
    let bottom: Vec<_> = order[split + 1..]
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: cards.clone(),
        to: Vec::new(),
    }];
    if !hand_cards.is_empty() {
        events.extend(move_cards(tx, hand_cards, draw, Some(hand))?);
    }
    if !equip_cards.is_empty() {
        events.extend(move_cards(
            tx,
            equip_cards,
            draw,
            Some(equip.expect("non-empty equipment selection has an equipment zone")),
        )?);
    }
    let mut middle = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    middle.retain(|card| !unique.contains(card));
    let zone = tx
        .state_mut()
        .zones
        .get_mut(draw)
        .expect("validated draw pile remains live");
    zone.cards = bottom;
    zone.cards.extend(middle);
    zone.cards.extend(top.into_iter().rev());
    Ok(events)
}

pub(crate) fn resolve_ddd_xiahouxuan(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let forced =
                skill_instance_by_def(tx, player, engine_api::SkillDefId::new(DDD_XUANLUN_DEF))
                    .and_then(|xuanlun| tx.state().skills.get(xuanlun))
                    .is_some_and(|instance| instance.mark & DDD_XUANLUN_FORCE_LANGHUAI != 0);
            if forced {
                commit_ddd_langhuai(tx, player, skill)
            } else {
                open_choice(
                    tx,
                    player,
                    0,
                    1,
                    true,
                    vec![1],
                    ChoiceKind::Confirm,
                    ContinuationKind::Custom,
                    vec![RESOLVE_DDD_XIAHOUXUAN_CHOICE, 0, player.raw(), skill.raw()],
                )
            }
        }
        1 => {
            if tx
                .state()
                .skills
                .get(skill)
                .is_none_or(|instance| instance.def.raw() != DDD_XUANLUN_DEF)
            {
                return Err(EffectError::Illegal("玄论技能实例无效".into()));
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.mark |= DDD_XUANLUN_PENDING;
            }
            draw_cards(tx, player, 4)
        }
        2 => {
            let mark = tx
                .state()
                .skills
                .get(skill)
                .filter(|instance| instance.def.raw() == DDD_XUANLUN_DEF)
                .map_or(0, |instance| instance.mark);
            if mark & DDD_XUANLUN_PENDING == 0 {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.mark &= !DDD_XUANLUN_PENDING;
            }
            let forced = mark & DDD_XUANLUN_FORCE_LANGHUAI != 0;
            if forced && tx.state().hand_cards(player).is_empty() {
                return Ok(Vec::new());
            }
            let card_count = ddd_xiahouxuan_hand_equip_cards(tx, player).len();
            let can_arrange = forced || card_count >= 4;
            if forced {
                return open_ddd_xuanlun_card_choice(tx, player, skill);
            }
            if !can_arrange {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.mark |= DDD_XUANLUN_FORCE_LANGHUAI;
                }
                return Ok(Vec::new());
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
                vec![RESOLVE_DDD_XIAHOUXUAN_CHOICE, 1, player.raw(), skill.raw()],
            )
        }
        3 => commit_ddd_xuanlun_arrangement(tx, player, skill),
        4 => commit_ddd_langhuai(tx, player, skill),
        _ => Err(EffectError::EngineFault(
            "unknown 3D精选夏侯玄 action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xiahouxuan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4
        || payload.first() != Some(&RESOLVE_DDD_XIAHOUXUAN_CHOICE)
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::Illegal("夏侯玄选择无效".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal("夏侯玄技能已失效".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() != 4 {
                return Err(EffectError::Illegal("朗怀确认数据无效".into()));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("朗怀确认选择无效".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddXiahouxuan {
                    player,
                    action: 4,
                    skill,
                }],
            )?;
            Ok(())
        }
        1 => {
            if payload.len() != 4 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("玄论分支选择无效".into()));
            }
            match selections[0] {
                1 => {
                    open_ddd_xuanlun_card_choice(tx, player, skill)?;
                }
                2 => {
                    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                        instance.mark |= DDD_XUANLUN_FORCE_LANGHUAI;
                    }
                }
                _ => return Err(EffectError::Illegal("玄论分支不存在".into())),
            }
            Ok(())
        }
        2 => {
            if payload.len() < 5 {
                return Err(EffectError::Illegal("玄论选牌数据无效".into()));
            }
            let count = usize::try_from(payload[4])
                .map_err(|_| EffectError::Illegal("玄论选牌数量无效".into()))?;
            if cancelled
                || selections.len() != count
                || payload.len() != 5 + ddd_xiahouxuan_hand_equip_cards(tx, player).len()
            {
                return Err(EffectError::Illegal("玄论选牌无效".into()));
            }
            let offered = &payload[5..];
            let current = ddd_xiahouxuan_hand_equip_cards(tx, player);
            let mut unique = std::collections::BTreeSet::new();
            if selections.iter().any(|card| {
                !unique.insert(*card)
                    || !offered.contains(card)
                    || !current.contains(&CardId::new(*card))
            }) {
                return Err(EffectError::Illegal("玄论所选牌已失效".into()));
            }
            let mut options = selections.to_vec();
            options.push(0);
            let mut next = vec![
                RESOLVE_DDD_XIAHOUXUAN_CHOICE,
                3,
                player.raw(),
                skill.raw(),
                count as u32,
            ];
            next.extend(selections);
            open_choice(
                tx,
                player,
                (count + 1) as u8,
                (count + 1) as u8,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                next,
            )?;
            Ok(())
        }
        3 => {
            if payload.len() < 5 {
                return Err(EffectError::Illegal("玄论排列数据无效".into()));
            }
            let count = usize::try_from(payload[4])
                .map_err(|_| EffectError::Illegal("玄论排列数量无效".into()))?;
            if payload.len() != 5 + count
                || cancelled
                || selections.len() != count + 1
                || selections.iter().filter(|token| **token == 0).count() != 1
            {
                return Err(EffectError::Illegal("玄论牌堆排列无效".into()));
            }
            let expected: std::collections::BTreeSet<_> = payload[5..].iter().copied().collect();
            let actual: std::collections::BTreeSet<_> = selections
                .iter()
                .copied()
                .filter(|token| *token != 0)
                .collect();
            if expected.len() != count || actual != expected {
                return Err(EffectError::Illegal("玄论牌堆排列内容无效".into()));
            }
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("玄论技能已失效".into()))?;
            instance.state_blob.clear();
            for token in selections {
                instance.state_blob.extend_from_slice(&token.to_le_bytes());
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddXiahouxuan {
                    player,
                    action: 3,
                    skill,
                }],
            )?;
            Ok(())
        }
        _ => Err(EffectError::Illegal("未知夏侯玄选择阶段".into())),
    }
}

pub(crate) fn ddd_zhangkai_unique_max_equip(tx: &Transaction) -> Option<PlayerId> {
    let counts: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| {
            let count = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len());
            (player, count)
        })
        .collect();
    let max = counts.iter().map(|(_, count)| *count).max()?;
    let mut leaders = counts.into_iter().filter(|(_, count)| *count == max);
    let leader = leaders.next()?.0;
    leaders.next().is_none().then_some(leader)
}

pub(crate) fn ddd_zhangkai_gainable(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    )
}

pub(crate) fn ddd_zhangkai_equip_materials(
    tx: &Transaction,
    player: PlayerId,
    actor: PlayerId,
    cards: &[CardId],
) -> Vec<CardId> {
    let processing = tx.state().zones.find(ZoneKind::Processing, PlayerId::NONE);
    let actor_hand = tx.state().zones.find(ZoneKind::Hand, actor);
    cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                (Some(card.zone) == processing || Some(card.zone) == actor_hand)
                    && card.kind.equip_slot().is_some_and(|slot| {
                        tx.state()
                            .players
                            .get(player)
                            .is_some_and(|owner| owner.equip_slot_enabled(slot))
                    })
            })
        })
        .collect()
}

pub(crate) fn ddd_zhangkai_equip(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::EngineFault("劫行找不到装备区".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("劫行找不到弃牌堆".into()))?;
    let mut events = Vec::new();
    for card in cards {
        let Some(card_state) = tx.state().cards.get(card).cloned() else {
            continue;
        };
        let Some(slot) = card_state.kind.equip_slot() else {
            continue;
        };
        if !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.equip_slot_enabled(slot))
        {
            continue;
        }
        let replaced: Vec<_> = tx
            .state()
            .zones
            .get(equip)
            .into_iter()
            .flat_map(|zone| zone.cards.iter().copied())
            .filter(|existing| {
                tx.state()
                    .cards
                    .get(*existing)
                    .and_then(|card| card.kind.equip_slot())
                    == Some(slot)
            })
            .collect();
        if !replaced.is_empty() {
            events.extend(move_cards_with_reason(
                tx,
                replaced,
                discard,
                Some(equip),
                CardMoveReason::Discard,
            )?);
        }
        events.extend(move_cards_with_reason(
            tx,
            vec![card],
            equip,
            Some(card_state.zone),
            CardMoveReason::Use,
        )?);
        events.push(DomainEvent::RuleNote {
            code: 50,
            payload: vec![player.raw(), card.raw()],
        });
    }
    Ok(events)
}
