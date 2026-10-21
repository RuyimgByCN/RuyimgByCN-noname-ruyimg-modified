//! Effect primitives for `resolve_spchijie` and related helpers.

use super::*;

/// 持节（辛毗，ABI 422）：成为其他角色单目标牌目标时判定，点数 > 6 取消该目标。
pub(crate) fn resolve_spchijie(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || card.is_none()
        || source == player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    // 目标须可被移除（当前牌含该目标）。
    let mut probe = tx.state().resolution.clone();
    if !probe.exclude_saved_sha_target(card, player) {
        return Ok(Vec::new());
    }
    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let rank = result.rank;
    tx.push_event(DomainEvent::RuleNote {
        code: 234,
        payload: vec![player.raw(), card.raw(), u32::from(rank)],
    });
    if rank > 6 {
        // 保持目标排除。
        tx.state_mut()
            .resolution
            .exclude_saved_sha_target(card, player);
    }
    Ok(vec![completed_judgment_event(player, result, CardId::NONE)])
}

/// 上议（江庆，ABI 423）：展示目标手牌并获得其中一张。
pub(crate) fn resolve_spshangyi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let hand = tx.state().hand_cards(target);
    if hand.is_empty() {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    events.extend(run_effects(
        tx,
        vec![EffectRequest::RevealCards {
            cards: hand.clone(),
            to: Vec::new(),
        }],
    )?);
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        hand.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            RESOLVE_SPSHANGYI_CHOICE,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )?;
    Ok(events)
}

/// 简议（江庆，ABI 424）：其他角色结束阶段获得弃牌堆中的防具牌。
pub(crate) fn resolve_spjianyi(
    tx: &mut Transaction,
    player: PlayerId,
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
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    let Some(discard) = discard else {
        return Ok(Vec::new());
    };
    let armors: Vec<_> = tx
        .state()
        .zones
        .get(discard)
        .map(|zone| {
            zone.cards
                .iter()
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|value| value.kind == CardKind::Armor)
                })
                .collect()
        })
        .unwrap_or_default();
    if armors.is_empty() {
        return Ok(Vec::new());
    }
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: armors,
            to: to_hand,
            from: Some(discard),
        }],
    )
}

/// 谏喻（费祎，ABI 425）：mode 0 授予喻标记（active 的 2 名目标）；
/// mode 1 触发时（出牌阶段喻标记角色指定另一喻标记角色为目标）目标摸 1。
pub(crate) fn resolve_fyjianyu(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    target: PlayerId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 喻标记定义 ID（= fyjianyu 技能 ID）。
    const YU_MARK: u32 = 2_248_481_428;
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if target.is_none() {
        // mode 0：授予 2 名目标喻标记。
        let mut effects = Vec::new();
        for recipient in targets {
            if recipient == player || !tx.state().players.get(recipient).is_some_and(|p| p.alive) {
                continue;
            }
            effects.push(EffectRequest::AddMark {
                player: recipient,
                mark: engine_api::MarkDefId::new(YU_MARK),
                delta: 1,
            });
        }
        return run_effects(tx, effects);
    }
    // mode 1：目标摸 1（filter 已校验标记与出牌阶段条件）。
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    run_effects(
        tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 1,
        }],
    )
}

/// 压军（崔琰，ABI 428）：出牌阶段开始，手牌中有本回合获得的牌时，选一张本回合
/// 新获得的牌与一名其他角色拼点；赢则可将拼点牌置于牌堆顶，输则添加“戒”标记
/// （手牌上限减标记数，由纯 DSL 辅助技能 spyajun_less 表达）。
/// stage 0 选新获牌 → 1 选拼点目标 → 2 目标选拼点牌 → 3 提交拼点并分流 →
/// 4 赢家可选将拼点牌置牌堆顶。续体在 resolve_choice.rs 校验选择后重入。
pub(crate) fn resolve_spyajun(
    tx: &mut Transaction,
    player: PlayerId,
    less_def: engine_api::SkillDefId,
    card: CardId,
    target: PlayerId,
    target_card: CardId,
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
    match stage {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let gained: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|candidate| tx.state().turn.gained_hand_card(player, *candidate))
                .map(CardId::raw)
                .collect();
            if gained.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                gained,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SPYAJUN_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    less_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player && !tx.state().hand_cards(*candidate).is_empty()
                })
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SPYAJUN_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    less_def.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let hand = tx.state().hand_cards(target);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SPYAJUN_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    less_def.raw(),
                    card.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            let state = tx.state();
            if state.turn.current != player || state.turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let (player_hand, target_hand, discard, winner) =
                hand_compare_commit_context(state, player, target, card, target_card, skill)?;
            let mut effects = vec![
                EffectRequest::RevealCards {
                    cards: vec![card, target_card],
                    to: Vec::new(),
                },
                EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(player_hand),
                },
                EffectRequest::MoveCards {
                    cards: vec![target_card],
                    to: discard,
                    from: Some(target_hand),
                },
                EffectRequest::EmitHandCompareFinished {
                    player,
                    target,
                    player_card: card,
                    target_card,
                    winner,
                },
            ];
            if winner == player {
                effects.push(EffectRequest::ResolveSpyajun {
                    player,
                    less_def,
                    card,
                    target,
                    target_card: CardId::NONE,
                    stage: 4,
                    skill,
                });
            } else {
                effects.push(EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(less_def.raw()),
                    delta: 1,
                });
            }
            run_effects(tx, effects)
        }
        4 => {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            if !tx
                .state()
                .zones
                .get(discard)
                .is_some_and(|zone| zone.cards.contains(&card))
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                vec![card.raw()],
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SPYAJUN_CUSTOM_CHOICE,
                    3,
                    player.raw(),
                    less_def.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        5 => {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            if !tx
                .state()
                .zones
                .get(discard)
                .is_some_and(|zone| zone.cards.contains(&card))
            {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            move_cards(tx, vec![card], draw, Some(discard))
        }
        _ => Err(EffectError::ContentFault("unknown Spyajun stage".into())),
    }
}

/// 尊帝（崔琰，ABI 429）：出牌阶段限一次，弃置一张手牌交给一名其他角色，然后
/// 判定：黑色其摸三张，红色其移动一张场上牌（目标选择场上牌与落点）。
/// stage 0 判定 + 选牌 → 1 选落点 → 2 提交移动。续体在 resolve_choice.rs 重入。
pub(crate) fn resolve_spzundi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    destination: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || target == player
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
            let result = crate::command::judge::flip_judgment(
                tx,
                player,
                crate::command::judge::JudgmentFavor::Neutral,
            )?;
            let mut events = vec![completed_judgment_event(player, result, CardId::NONE)];
            if matches!(result.suit, Suit::Spade | Suit::Club) {
                events.extend(draw_cards(tx, target, 3)?);
                return Ok(events);
            }
            let candidates: Vec<_> = tx
                .state()
                .players
                .iter_live()
                .flat_map(|participant| {
                    crate::query::movable_field_cards_from(tx.state(), participant.id)
                })
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|value| {
                        !crate::query::field_card_move_destinations(tx.state(), *card)
                            .contains(&value.owner)
                    })
                })
                .collect();
            let mut seen = std::collections::BTreeSet::new();
            let options: Vec<_> = candidates
                .into_iter()
                .filter(|card| seen.insert(card.raw()))
                .map(CardId::raw)
                .collect();
            if options.is_empty() {
                return Ok(events);
            }
            events.extend(open_choice(
                tx,
                target,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SPZUNDI_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )?);
            Ok(events)
        }
        1 => {
            let destinations = crate::query::field_card_move_destinations(tx.state(), card);
            if destinations.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                destinations.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SPZUNDI_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let movable = tx
                .state()
                .players
                .iter_live()
                .flat_map(|participant| {
                    crate::query::movable_field_cards_from(tx.state(), participant.id)
                })
                .any(|candidate| candidate == card);
            if !movable
                || !crate::query::field_card_move_destinations(tx.state(), card)
                    .contains(&destination)
            {
                return Err(EffectError::Illegal(
                    "Spzundi field card or destination no longer legal".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Spzundi field card missing".into()))?;
            let kind = tx
                .state()
                .zones
                .get(from)
                .map(|zone| zone.kind)
                .ok_or_else(|| EffectError::Illegal("Spzundi field zone missing".into()))?;
            let to =
                tx.state().zones.find(kind, destination).ok_or_else(|| {
                    EffectError::Illegal("Spzundi destination zone missing".into())
                })?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to,
                    from: Some(from),
                }],
            )
        }
        _ => Err(EffectError::ContentFault("unknown Spzundi stage".into())),
    }
}

/// 佯解（朱儁，ABI 430）：出牌阶段限一次与一名其他角色拼点，**点数小者胜**；
/// 拼点失败时可选一名其他角色视为对拼点目标使用火【杀】。
/// stage 0 拥有者选拼点牌 → 1 目标选拼点牌 → 2 提交（小者胜）并开火杀来源
/// choice → 3 提交虚拟火杀。
pub(crate) fn resolve_xinyangjie(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    target_card: CardId,
    source: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || target == player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
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
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() || tx.state().hand_cards(target).is_empty() {
                return Ok(Vec::new());
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
                vec![
                    RESOLVE_XINYANGJIE_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let hand = tx.state().hand_cards(target);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XINYANGJIE_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let player_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Xinyangjie owner hand missing".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("Xinyangjie target hand missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let player_rank = tx
                .state()
                .cards
                .get(card)
                .filter(|value| value.owner == player && value.zone == player_hand)
                .map(|value| value.rank)
                .ok_or_else(|| EffectError::Illegal("owner comparison card left hand".into()))?;
            let target_rank = tx
                .state()
                .cards
                .get(target_card)
                .filter(|value| value.owner == target && value.zone == target_hand)
                .map(|value| value.rank)
                .ok_or_else(|| EffectError::Illegal("target comparison card left hand".into()))?;
            // 点数小者胜（旧 JS chooseToCompare small）。
            let winner = if player_rank < target_rank {
                player
            } else if target_rank < player_rank {
                target
            } else {
                PlayerId::NONE
            };
            let mut effects = vec![
                EffectRequest::RevealCards {
                    cards: vec![card, target_card],
                    to: Vec::new(),
                },
                EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(player_hand),
                },
                EffectRequest::MoveCards {
                    cards: vec![target_card],
                    to: discard,
                    from: Some(target_hand),
                },
                EffectRequest::EmitHandCompareFinished {
                    player,
                    target,
                    player_card: card,
                    target_card,
                    winner,
                },
            ];
            if winner == target {
                // 拼点失败：可选一名其他角色视为对目标使用火【杀】。
                effects.push(EffectRequest::ResolveXinyangjie {
                    player,
                    target,
                    card,
                    target_card: CardId::NONE,
                    source: PlayerId::NONE,
                    stage: 3,
                    skill,
                });
            }
            run_effects(tx, effects)
        }
        3 => {
            let candidates: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player
                        && *candidate != target
                        && crate::effect::dcchiying_in_range(tx.state(), *candidate, target)
                })
                .map(PlayerId::raw)
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                candidates,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XINYANGJIE_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        4 => {
            // 对拼点目标使用虚拟火【杀】（CardKind::ShaFire tag=5）。
            run_effects(
                tx,
                vec![EffectRequest::UseVirtualCardOn {
                    player: source,
                    kind: u8::from(crate::state::CardKind::ShaFire.tag()),
                    target,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("unknown Xinyangjie stage".into())),
    }
}

/// 清雨（糜夫人，ABI 431）：使命技——
/// action 0 受伤前（damage_before，可弃 he ≥ 2）强制弃 2 张并防伤；
/// action 1 准备阶段满血且手牌 < 当前轮数时使命成功，获【选存】；
/// action 2 濒死时使命失败，减 1 点体力上限并选一人获【勇决】（开选人
/// choice，仅一名其他角色时直接授予）；action 3 授予勇决。
/// 使命状态以技能实例 state_blob（"QY1"+status）记录，唤醒后三个分支均失效。
pub(crate) fn resolve_qingyu(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    xuancun_def: engine_api::SkillDefId,
    mbyongjue_def: engine_api::SkillDefId,
    target: PlayerId,
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
    let status = tx
        .state()
        .skills
        .get(skill)
        .and_then(|instance| instance.state_blob.strip_prefix(b"QY1"))
        .and_then(|rest| rest.first())
        .copied()
        .unwrap_or(0);
    match action {
        0 => {
            if status != 0 {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.len() < 2 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                2,
                2,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_QINGYU_CUSTOM_CHOICE, player.raw(), skill.raw()],
            )
        }
        1 => {
            if status != 0 {
                return Ok(Vec::new());
            }
            let healthy = tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.hp == participant.max_hp as i32);
            // 旧 JS 手牌 < roundNumber；引擎 turn.round 为已结束轮数，+1 近似当前轮。
            let round = tx.state().turn.round.saturating_add(1);
            if !healthy || tx.state().hand_count(player) >= round {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = [b"QY1".as_slice(), &[1]].concat();
            }
            add_skill(tx, player, xuancun_def)
        }
        2 => {
            if status != 0 {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = [b"QY1".as_slice(), &[1]].concat();
            }
            let candidates: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
                .map(PlayerId::raw)
                .collect();
            let mut events =
                run_effects(tx, vec![EffectRequest::ChangeMaxHp { player, delta: -1 }])?;
            if candidates.len() == 1 {
                events.extend(add_skill(tx, PlayerId::new(candidates[0]), mbyongjue_def)?);
                return Ok(events);
            }
            if candidates.is_empty() {
                return Ok(events);
            }
            events.extend(open_choice(
                tx,
                player,
                1,
                1,
                false,
                candidates,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_QINGYU_FAIL_TARGET,
                    player.raw(),
                    mbyongjue_def.raw(),
                    skill.raw(),
                ],
            )?);
            Ok(events)
        }
        3 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
                || target == player
            {
                return Ok(Vec::new());
            }
            add_skill(tx, target, mbyongjue_def)
        }
        _ => Err(EffectError::ContentFault("unknown Qingyu action".into())),
    }
}

/// 整肃（核心库通用系统，ABI 432）：三个整肃条件之一的状态跟踪与弃牌阶段判定。
/// mode 0 垒金：出牌阶段使用点数严格递增的牌且不少于三张（card_used 时把点数
/// 序列写入技能实例 state_blob "ZS0" 段，非递增即记失败）。
/// mode 1 变阵：出牌阶段使用的牌花色全部相同且不少于两张（弃牌阶段末由花色
/// 掩码 + 用牌计数判定）。
/// mode 2 明志：弃牌阶段弃置的牌花色全部不同且不少于两张（弃牌阶段末由
/// discard_phase_discards_for 判定）。
/// 判定成功时受益者（"GR" 段记录的授予者 + 自己）各自摸两张或回复一点体力
/// （ChooseDrawOrRecover），随后移除临时整肃技能。
pub(crate) fn resolve_zhengsu(
    tx: &mut Transaction,
    player: PlayerId,
    mode: u8,
    stage: u8,
    card: CardId,
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
    match (mode, stage) {
        (0, 0) => {
            // 垒金：card_used 跟踪点数递增序列（状态段 "ZS0" 在前，授予者段
            // "GR" 以 0xFE 分隔附后，互不覆盖）。
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let Some(rank) = tx.state().cards.get(card).map(|value| value.rank) else {
                return Ok(Vec::new());
            };
            let blob = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| instance.state_blob.clone())
                .unwrap_or_default();
            let (zs, tail) = if let Some(pos) = blob.windows(3).position(|w| w == b"ZS0") {
                let rest = &blob[pos + 3..];
                let split = rest
                    .iter()
                    .position(|byte| *byte == 0xFE)
                    .unwrap_or(rest.len());
                (&rest[..split], &rest[split..])
            } else {
                // 尚无 ZS0 段：整个 blob（可能为授予者 GR 段）作为尾部保留。
                (&[][..], &blob[..])
            };
            let mut fail = zs.first().copied().unwrap_or(0);
            let mut ranks: Vec<u8> = zs.get(1..).map(|slice| slice.to_vec()).unwrap_or_default();
            if fail == 0 {
                if let Some(last) = ranks.last().copied() {
                    if rank <= last {
                        fail = 1;
                    }
                }
                if fail == 0 {
                    ranks.push(rank);
                }
            }
            let mut out = vec![fail];
            out.extend(ranks);
            let mut new_blob = Vec::with_capacity(out.len() + tail.len() + 4);
            new_blob.extend_from_slice(b"ZS0");
            new_blob.extend_from_slice(&out);
            if !tail.is_empty() {
                // 授予者段以 0xFE 分隔附后（tail 若已以 0xFE 开头则不重复）。
                if tail.first() != Some(&0xFE) {
                    new_blob.push(0xFE);
                }
                new_blob.extend_from_slice(tail);
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = new_blob;
            }
            Ok(Vec::new())
        }
        (0, 1) | (1, 1) | (2, 1) => {
            if tx.state().turn.current != player {
                return Ok(Vec::new());
            }
            let succeed = match mode {
                0 => {
                    // 垒金：点数严格递增且不少于三张。
                    let blob = tx
                        .state()
                        .skills
                        .get(skill)
                        .map(|instance| instance.state_blob.clone())
                        .unwrap_or_default();
                    let Some(rest) = blob.strip_prefix(b"ZS0") else {
                        return Ok(Vec::new());
                    };
                    let end = rest
                        .iter()
                        .position(|byte| *byte == 0xFE)
                        .unwrap_or(rest.len());
                    rest[..end].first().copied().unwrap_or(1) == 0 && end >= 4
                }
                1 => {
                    // 变阵：出牌阶段同花色且不少于两张。
                    let key =
                        crate::state::turn_flag::play_phase_card_suit_history_by_player(player);
                    let suits = tx.state().turn.flag(key).max(0) as u32;
                    suits.count_ones() == 1
                        && tx.state().turn.play_phase_card_use_count(player) >= 2
                }
                2 => {
                    // 明志：弃牌阶段花色全异且不少于两张。
                    let mut seen: u32 = 0;
                    let mut count = 0u32;
                    let mut distinct = true;
                    for card in crate::effect::discard_phase_discards_for(tx.state(), player) {
                        let Some(suit) = tx
                            .state()
                            .cards
                            .get(card)
                            .map(|value| crate::command::judge::suit_tag(value.suit))
                        else {
                            continue;
                        };
                        if suit == 0 {
                            distinct = false;
                            break;
                        }
                        let bit = 1u32 << suit;
                        if seen & bit != 0 {
                            distinct = false;
                            break;
                        }
                        seen |= bit;
                        count += 1;
                    }
                    distinct && count >= 2
                }
                _ => return Err(EffectError::ContentFault("unknown Zhengsu mode".into())),
            };
            if !succeed {
                // 整肃失败：移除临时技能，无奖励。
                let instance = tx
                    .state()
                    .skills
                    .get(skill)
                    .map(|instance| instance.id)
                    .ok_or_else(|| EffectError::EngineFault("Zhengsu skill missing".into()))?;
                return run_effects(tx, vec![EffectRequest::RemoveSkill { skill: instance }]);
            }
            // 整肃成功：受益者 = 授予者（"GR" 段）+ 自己；各自摸二或回复一。
            let mut beneficiaries = vec![player];
            // 授予者段以 0xFE 分隔附于状态段之后（"ZS0… 0xFE GR…" 或纯 "GR…"）。
            if let Some(rest) = tx.state().skills.get(skill).and_then(|instance| {
                let blob = &instance.state_blob;
                if let Some(rest) = blob.strip_prefix(b"GR") {
                    Some(rest)
                } else {
                    blob.windows(2)
                        .position(|window| window == [0xFE, b'G'])
                        .and_then(|pos| blob.get(pos + 3..))
                }
            }) {
                for chunk in rest.chunks_exact(4) {
                    let raw = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    let grantee = PlayerId::new(raw);
                    if grantee.is_some() && grantee != player && !beneficiaries.contains(&grantee) {
                        beneficiaries.push(grantee);
                    }
                }
            }
            let mut effects = Vec::new();
            for beneficiary in beneficiaries {
                effects.push(EffectRequest::ChooseDrawOrRecover {
                    player: beneficiary,
                    draw_count: 2,
                    recover_count: 1,
                    allow_cancel: false,
                });
            }
            if let Some(instance) = tx.state().skills.get(skill) {
                effects.push(EffectRequest::RemoveSkill { skill: instance.id });
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("unknown Zhengsu stage".into())),
    }
}

/// 侯封 / 燕集（ABI 433）：出牌阶段开始选择整肃类型授予目标。
/// mode 0 侯封（朱儁）：目标为其他角色，授予时记录授予者供奖励共享；
/// mode 1 燕集（吕范）：目标为自己。
pub(crate) fn resolve_houfeng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    mode: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
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
            if tx.state().turn.current != target || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let owned = |def: u32| {
                tx.state().skills.sorted_live().iter().any(|instance| {
                    instance.owner == target
                        && instance.def.raw() == def
                        && tx.state().skill_effectively_enabled(instance)
                })
            };
            let mut options = Vec::new();
            for (tag, def) in [
                (1u32, ZHENGSU_LEIJIN),
                (2, ZHENGSU_BIANZHEN),
                (3, ZHENGSU_MINGZHI),
            ] {
                if !owned(def) {
                    options.push(tag);
                }
            }
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
                vec![
                    RESOLVE_HOUFENG_CHOICE,
                    player.raw(),
                    target.raw(),
                    u32::from(mode),
                    skill.raw(),
                ],
            )
        }
        1 => {
            // 授予整肃技能（临时，弃牌阶段判定后自移除）并记录授予者。
            let def = match mode {
                0 => ZHENGSU_LEIJIN,
                1 => ZHENGSU_BIANZHEN,
                2 => ZHENGSU_MINGZHI,
                _ => return Err(EffectError::ContentFault("unknown Houfeng type".into())),
            };
            if tx
                .state()
                .skills
                .sorted_live()
                .iter()
                .any(|instance| instance.owner == target && instance.def.raw() == def)
            {
                return Ok(Vec::new());
            }
            let mut events = add_skill(tx, target, engine_api::SkillDefId::new(def))?;
            // 侯封（授他人）记录授予者；燕集（自己）无授予者。
            if mode == 0 && player != target {
                if let Some(grantee) = tx
                    .state()
                    .skills
                    .sorted_live()
                    .iter()
                    .find(|instance| instance.owner == target && instance.def.raw() == def)
                {
                    let grantee_id = grantee.id;
                    let existing = tx
                        .state()
                        .skills
                        .get(grantee_id)
                        .and_then(|instance| instance.state_blob.strip_prefix(b"GR"))
                        .map(|rest| rest.to_vec())
                        .unwrap_or_default();
                    let mut out = existing;
                    out.extend_from_slice(&player.raw().to_le_bytes());
                    if let Some(instance) = tx.state_mut().skills.get_mut(grantee_id) {
                        instance.state_blob = [b"GR".as_slice(), &out].concat();
                    }
                }
            }
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown Houfeng stage".into())),
    }
}

/// 点才（吕范，ABI 434）：其他角色出牌阶段结束时，若其本回合失去牌数不小于其
/// 体力值且手牌数小于体力上限，其摸至体力上限。
pub(crate) fn resolve_mbdiancai(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || actor == player
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let lost = tx.state().turn.card_loss_count(actor);
    let hp = tx.state().players.get(actor).map(|p| p.hp).unwrap_or(0);
    let max_hp = tx.state().players.get(actor).map(|p| p.max_hp).unwrap_or(0);
    let hand = tx.state().hand_count(actor);
    if lost < hp.max(0) as u32 || hand >= max_hp {
        return Ok(Vec::new());
    }
    draw_cards(tx, actor, (max_hp - hand) as u32)
}

/// 其他 agent 在建批次（rechuhai）的临时桩；其实现落盘后移除。
pub(crate) fn resolve_rechuhai_choice(
    _tx: &mut Transaction,
    _observer: PlayerId,
    _payload: &[u32],
    _selections: &[u32],
    _cancelled: bool,
) -> Result<(), EffectError> {
    Err(EffectError::ContentFault(
        "rechuhai choice resolver pending implementation".into(),
    ))
}

/// 战海（周处，ABI 435）：职责技——出牌阶段限一次摸一张并与一名其他角色拼点
/// （装备数 < 4 时拼点牌点数 + (4 - 装备数)，上限 13）；赢则展示目标手牌并按
/// 类别从牌堆各取一张获得；输且拼点牌点数 < 7 使命失败；装备后装备数 > 2
/// 使命成功：回复至满、以【彰名】替换【乡害】。使命状态 blob "RCH1"。
/// stage 0 摸一张+选拼点牌 → 1 目标选拼点牌 → 2 提交+使命判定 →
/// 3 装备后使命成功检查。
pub(crate) fn resolve_rechuhai(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    target_card: CardId,
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
    match stage {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let mut events = draw_cards(tx, player, 1)?;
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() || tx.state().hand_cards(target).is_empty() {
                return Ok(events);
            }
            events.extend(open_choice(
                tx,
                player,
                1,
                1,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_RECHUHAI_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )?);
            Ok(events)
        }
        1 => {
            let hand = tx.state().hand_cards(target);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_RECHUHAI_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let player_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Rechuhai owner hand missing".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("Rechuhai target hand missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let Some(owner_rank) = tx
                .state()
                .cards
                .get(card)
                .filter(|value| value.owner == player && value.zone == player_hand)
                .map(|value| value.rank)
            else {
                return Err(EffectError::Illegal(
                    "owner comparison card left hand".into(),
                ));
            };
            let Some(target_rank) = tx
                .state()
                .cards
                .get(target_card)
                .filter(|value| value.owner == target && value.zone == target_hand)
                .map(|value| value.rank)
            else {
                return Err(EffectError::Illegal(
                    "target comparison card left hand".into(),
                ));
            };
            // 装备数 < 4 时拼点牌点数 + (4 - 装备数)，上限 13。
            let equip_count = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len());
            let bonus = if equip_count < 4 {
                (4 - equip_count) as u8
            } else {
                0
            };
            let effective_rank = owner_rank.saturating_add(bonus).min(13);
            let winner = if effective_rank > target_rank {
                player
            } else if target_rank > effective_rank {
                target
            } else {
                PlayerId::NONE
            };
            let mut effects = vec![
                EffectRequest::RevealCards {
                    cards: vec![card, target_card],
                    to: Vec::new(),
                },
                EffectRequest::MoveCards {
                    cards: vec![card],
                    to: discard,
                    from: Some(player_hand),
                },
                EffectRequest::MoveCards {
                    cards: vec![target_card],
                    to: discard,
                    from: Some(target_hand),
                },
                EffectRequest::EmitHandCompareFinished {
                    player,
                    target,
                    player_card: card,
                    target_card,
                    winner,
                },
            ];
            let status = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.state_blob.strip_prefix(b"RCH1"))
                .and_then(|rest| rest.first())
                .copied()
                .unwrap_or(0);
            if winner == player && status == 0 {
                // 赢：展示目标手牌并按类别从牌堆各取一张获得。
                let categories: Vec<u32> = {
                    let mut seen = Vec::new();
                    for hand_card in tx.state().hand_cards(target) {
                        if let Some(category) =
                            crate::effect::card_category_of(tx.state(), hand_card)
                        {
                            if !seen.contains(&category) {
                                seen.push(category);
                            }
                        }
                    }
                    seen
                };
                let mut gained = Vec::new();
                for category in categories {
                    if let Some((_, picked)) =
                        first_shared_pile_card_by_category(tx, player, |c| {
                            u32::from(c) == category
                        })?
                    {
                        gained.push(picked);
                    }
                }
                if !gained.is_empty() {
                    let to_hand = tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, player)
                        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
                    effects.push(EffectRequest::MoveCards {
                        cards: gained,
                        to: to_hand,
                        from: None,
                    });
                }
            } else if winner != player && owner_rank < 7 && status == 0 {
                // 输且拼点牌点数 < 7：使命失败。
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.state_blob = [b"RCH1".as_slice(), &[1]].concat();
                }
            }
            run_effects(tx, effects)
        }
        3 => {
            // 装备后使命成功检查：装备数 > 2 且使命未完成。
            let status = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.state_blob.strip_prefix(b"RCH1"))
                .and_then(|rest| rest.first())
                .copied()
                .unwrap_or(0);
            if status != 0 {
                return Ok(Vec::new());
            }
            let equip_count = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len());
            if equip_count <= 2 {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = [b"RCH1".as_slice(), &[1]].concat();
            }
            let mut effects = Vec::new();
            let max_hp = tx
                .state()
                .players
                .get(player)
                .map(|p| p.max_hp)
                .unwrap_or(0);
            let hp = tx.state().players.get(player).map(|p| p.hp).unwrap_or(0);
            if hp < max_hp as i32 {
                effects.push(EffectRequest::RecoverHp {
                    player,
                    amount: (max_hp - hp.max(0) as u32) as u32,
                    source: PlayerId::NONE,
                });
            }
            // 以【彰名】替换【乡害】。
            if let Some(instance) = tx
                .state()
                .skills
                .sorted_live()
                .into_iter()
                .find(|instance| instance.owner == player && instance.def.raw() == XIANGHAI_DEF)
            {
                effects.push(EffectRequest::RemoveSkill { skill: instance.id });
            }
            effects.push(EffectRequest::AddSkill {
                player,
                skill_def: engine_api::SkillDefId::new(ZHANGMING_DEF),
            });
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("unknown Rechuhai stage".into())),
    }
}
