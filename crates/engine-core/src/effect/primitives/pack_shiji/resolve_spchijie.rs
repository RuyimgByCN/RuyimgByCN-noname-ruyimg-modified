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
