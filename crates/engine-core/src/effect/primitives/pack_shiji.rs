//! Effect primitive implementations — shiji cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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

/// Custom-continuation marker for ABI 423 上议选牌（payload[0]）。
pub(crate) const RESOLVE_SPSHANGYI_CHOICE: u32 = 0xDCDA_4230;

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

/// 压军（崔琰，ABI 428）续体标记（payload[0]）；stage 在 payload[1]。
pub(crate) const RESOLVE_SPYAJUN_CUSTOM_CHOICE: u32 = 0xDCDA_4280;

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

/// 尊帝（崔琰，ABI 429）续体标记（payload[0]）；stage 在 payload[1]。
pub(crate) const RESOLVE_SPZUNDI_CUSTOM_CHOICE: u32 = 0xDCDA_4290;

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

/// 佯解（朱儁，ABI 430）续体标记（payload[0]）；stage 在 payload[1]。
pub(crate) const RESOLVE_XINYANGJIE_CUSTOM_CHOICE: u32 = 0xDCDA_4300;

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

/// 清雨（糜夫人，ABI 431）续体标记（payload[0]）。
pub(crate) const RESOLVE_QINGYU_CUSTOM_CHOICE: u32 = 0xDCDA_4310;
/// 清雨使命失败授技目标标记（payload[0]）。
pub(crate) const RESOLVE_QINGYU_FAIL_TARGET: u32 = 0xDCDA_4311;

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

/// 整肃（ABI 432）续体标记（payload[0]）。
pub(crate) const RESOLVE_ZHENGSU_CHOICE: u32 = 0xDCDA_4320;

/// 整肃三技能定义（核心库通用系统，朱儁侯封 / 吕范燕集共用）。
pub(crate) const ZHENGSU_LEIJIN: u32 = 2_584_022_884;
pub(crate) const ZHENGSU_BIANZHEN: u32 = 3_044_720_297;
pub(crate) const ZHENGSU_MINGZHI: u32 = 3_824_072_820;

/// 侯封 / 燕集（ABI 433）续体标记（payload[0]）。
pub(crate) const RESOLVE_HOUFENG_CHOICE: u32 = 0xDCDA_4330;

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

/// 战海（周处，ABI 435）续体标记（payload[0]）。
pub(crate) const RESOLVE_RECHUHAI_CHOICE: u32 = 0xDCDA_4350;

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

/// 乡害（周处初始技能，纯 DSL 草稿）def id。
pub(crate) const XIANGHAI_DEF: u32 = 3_356_961_773;
/// 彰名（使命成功获得）def id。
pub(crate) const ZHANGMING_DEF: u32 = 3_550_704_513;

/// 凶竖（贾充，ABI 436）续体标记（payload[0]）。
pub(crate) const RESOLVE_XIONGSHU_CHOICE: u32 = 0xDCDA_4360;

/// 凶竖（贾充，ABI 436）：其他角色出牌阶段开始时弃置 X 张牌（X=凶竖标记数，
/// 每轮递增）并展示其一张手牌，预测其是否会使用该牌；该出牌阶段结束时预测对
/// 则其受 1 点伤害，错则获得该牌（目标区域或弃牌堆）。
/// stage 0 弃牌（标记>0 时）+加标记 → 1 选目标手牌展示 → 2 预测 →
/// 3（phase_play_end）结算。
pub(crate) fn resolve_xiongshu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
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
            if tx.state().turn.current != target || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let mark = tx
                .state()
                .players
                .get(player)
                .and_then(|p| {
                    p.marks
                        .iter()
                        .find(|(m, _)| *m == XIONGSHU_MARK)
                        .map(|(_, n)| *n)
                })
                .unwrap_or(0)
                .max(0) as u32;
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if (options.len() as u32) < mark {
                return Ok(Vec::new());
            }
            if mark == 0 {
                // 无弃牌代价直接推进；发动即标记 +1（下次弃 1 张）。
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::AddMark {
                            player,
                            mark: engine_api::MarkDefId::new(XIONGSHU_MARK),
                            delta: 1,
                        },
                        EffectRequest::ResolveXiongshu {
                            player,
                            target,
                            card: CardId::NONE,
                            action: 0,
                            stage: 1,
                            skill,
                        },
                    ],
                );
            }
            open_choice(
                tx,
                player,
                mark as u8,
                mark as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XIONGSHU_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    mark,
                    skill.raw(),
                ],
            )
        }
        1 => {
            // 选目标一张手牌展示。
            let hand = tx.state().hand_cards(target);
            if hand.is_empty() {
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
                    RESOLVE_XIONGSHU_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            // 预测：0 会使用 / 1 不会使用。
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XIONGSHU_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            // 结算：预测对 → 目标 1 点伤害；错 → 获得该牌（目标区域或弃牌堆）。
            // blob "XS1" + target(4) + card(4) + action(1)。
            if tx.state().turn.current != target {
                return Ok(Vec::new());
            }
            let blob = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.state_blob.strip_prefix(b"XS1"))
                .map(|rest| rest.to_vec())
                .unwrap_or_default();
            if blob.len() < 9 {
                return Ok(Vec::new());
            }
            let saved_target =
                PlayerId::new(u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]));
            let saved_card = CardId::new(u32::from_le_bytes([blob[4], blob[5], blob[6], blob[7]]));
            let predicted = blob[8];
            if saved_target != target {
                return Ok(Vec::new());
            }
            // 清空预测状态。
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
            }
            // 近似：目标区域不再持有该牌视为已使用（旧 JS 以 useCard 历史判定）。
            let used = !tx.state().cards.get(saved_card).is_some_and(|value| {
                value.owner == target
                    && matches!(
                        tx.state().zones.get(value.zone).map(|z| z.kind),
                        Some(ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge)
                    )
            });
            let correct = used == (predicted == 0);
            if correct {
                return deal_damage(
                    tx,
                    PlayerId::NONE,
                    target,
                    1,
                    false,
                    rule_api::damage_nature::NORMAL,
                    false,
                    CardId::NONE,
                    0,
                    0,
                );
            }
            // 预测错：获得该牌（目标区域或弃牌堆）。
            let from = tx.state().cards.get(saved_card).map(|value| value.zone);
            let Some(from) = from else {
                return Ok(Vec::new());
            };
            let to_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
            move_cards(tx, vec![saved_card], to_hand, Some(from))
        }
        _ => Err(EffectError::ContentFault("unknown Xiongshu stage".into())),
    }
}

/// 凶竖标记键（= 凶竖技能 def id）。
pub(crate) const XIONGSHU_MARK: u32 = 3_731_458_110;

/// 奸回（贾充，ABI 437）：伤害结算后——作为伤害来源且受伤者与上次相同则摸一张；
/// 受伤且来源与上次相同（连续同来源）则来源强制弃一张手牌或装备。
/// blob "JH1" + last_source(4) + last_target(4)。
pub(crate) fn resolve_jianhui(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || source == target
    {
        return Ok(Vec::new());
    }
    let blob = tx
        .state()
        .skills
        .get(skill)
        .and_then(|instance| instance.state_blob.strip_prefix(b"JH1"))
        .map(|rest| rest.to_vec())
        .unwrap_or_default();
    let last_source = if blob.len() >= 4 {
        PlayerId::new(u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]))
    } else {
        PlayerId::NONE
    };
    let last_target = if blob.len() >= 8 {
        PlayerId::new(u32::from_le_bytes([blob[4], blob[5], blob[6], blob[7]]))
    } else {
        PlayerId::NONE
    };
    let mut effects = Vec::new();
    if player == source {
        // 作为来源：连续对同一角色造成伤害 → 摸一张。
        if target == last_target && source != target {
            effects.push(EffectRequest::DrawCards { player, count: 1 });
        }
    } else if player == target {
        // 受伤：来源与上次相同（连续同来源）→ 来源强制弃一张手牌或装备。
        if source == last_source && source.is_some() && source != player {
            effects.push(EffectRequest::ChooseDiscardOneHandOrEquip { player: source });
        }
    }
    // 更新记录。
    let mut out = Vec::with_capacity(8);
    out.extend_from_slice(&source.raw().to_le_bytes());
    out.extend_from_slice(&target.raw().to_le_bytes());
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"JH1".as_slice(), &out].concat();
    }
    run_effects(tx, effects)
}

/// 弃哀（王粲，ABI 439）续体标记（payload[0]）。
pub(crate) const RESOLVE_SPQIAI_CHOICE: u32 = 0xDCDA_4390;

/// 弃哀（王粲，ABI 439）：出牌阶段限一次将一张非基本牌交给一名其他角色——
/// 满血则摸两张，已受伤则目标二选一（令拥有者回复 1 点体力 / 摸两张）。
pub(crate) fn resolve_spqiai(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
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
            if card.is_none() {
                return Ok(Vec::new());
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Spqiai gift card missing".into()))?;
            let to_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("recipient hand missing".into()))?;
            let mut events = move_cards_with_reason(
                tx,
                vec![card],
                to_hand,
                Some(from),
                engine_api::CardMoveReason::Give,
            )?;
            let wounded = tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.hp < participant.max_hp as i32);
            if !wounded {
                events.extend(draw_cards(tx, player, 2)?);
                return Ok(events);
            }
            events.extend(open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SPQIAI_CHOICE,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )?);
            Ok(events)
        }
        1 => {
            // 目标二选一：0 令拥有者回复 1 / 1 令拥有者摸两张。
            let wounded = tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.hp < participant.max_hp as i32);
            if wounded {
                return recover_hp(tx, player, player, 1);
            }
            draw_cards(tx, player, 2)
        }
        _ => Err(EffectError::ContentFault("unknown Spqiai stage".into())),
    }
}

/// 擅袭（王粲，ABI 440）续体标记（payload[0]）。
pub(crate) const RESOLVE_SPSHANXI_CHOICE: u32 = 0xDCDA_4400;
/// 索命（王粲，ABI 441）续体标记（payload[0]）。
pub(crate) const RESOLVE_SUOMING_CHOICE: u32 = 0xDCDA_4410;

/// 檄标记键（= 擅袭技能 def id）。
pub(crate) const SPSHANXI_MARK: u32 = 2_378_382_374;

/// 擅袭（王粲，ABI 440）：出牌阶段开始选一名无“檄”标记的角色获得“檄”标记
/// （全局唯一，同时清除他人标记）。
pub(crate) fn resolve_spshanxi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
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
            let candidates: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player
                        && !tx
                            .state()
                            .players
                            .get(*candidate)
                            .is_some_and(|participant| {
                                participant
                                    .marks
                                    .iter()
                                    .any(|(m, n)| *m == SPSHANXI_MARK && *n > 0)
                            })
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
                vec![RESOLVE_SPSHANXI_CHOICE, player.raw(), skill.raw()],
            )
        }
        1 => {
            // 檄标记全局唯一：清除所有角色的檄标记后授予目标。
            let mut effects = Vec::new();
            for participant in tx.state().players.iter_live() {
                if participant
                    .marks
                    .iter()
                    .any(|(m, n)| *m == SPSHANXI_MARK && *n > 0)
                {
                    effects.push(EffectRequest::RemoveMark {
                        player: participant.id,
                        mark: engine_api::MarkDefId::new(SPSHANXI_MARK),
                        amount: 0, // 0 = 移除整项（引擎语义）
                    });
                }
            }
            effects.push(EffectRequest::AddMark {
                player: target,
                mark: engine_api::MarkDefId::new(SPSHANXI_MARK),
                delta: 1,
            });
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("unknown Spshanxi stage".into())),
    }
}

/// 索命（王粲，ABI 441）：其他角色回复体力后（其有檄标记且体力 > 0）其交给你
/// 2 张牌或失去 1 点体力（可交不足时直接失体）。
pub(crate) fn resolve_spshanxi_suoming(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || actor == player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|participant| participant.alive && participant.hp > 0)
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), actor);
            if options.len() < 2 {
                // 可交不足：直接失去 1 点体力。
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: actor,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                );
            }
            open_choice(
                tx,
                actor,
                2,
                2,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_SUOMING_CHOICE,
                    player.raw(),
                    actor.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            // 交 2 张牌给拥有者；取消（未交）→ 失去 1 点体力由 handler 处理。
            let cards = {
                let mut got = Vec::new();
                if let Some(rest) = tx
                    .state()
                    .skills
                    .get(skill)
                    .and_then(|instance| instance.state_blob.strip_prefix(b"SM1"))
                {
                    for chunk in rest.chunks_exact(4) {
                        got.push(CardId::new(u32::from_le_bytes([
                            chunk[0], chunk[1], chunk[2], chunk[3],
                        ])));
                    }
                }
                got
            };
            if cards.len() != 2 {
                return Ok(Vec::new());
            }
            let to_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
            let mut effects = Vec::new();
            for card in &cards {
                if let Some(from) = tx.state().cards.get(*card).map(|value| value.zone) {
                    effects.push(EffectRequest::MoveCards {
                        cards: vec![*card],
                        to: to_hand,
                        from: Some(from),
                    });
                }
            }
            // 清空交牌记录。
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("unknown Suoming stage".into())),
    }
}

/// 通夺（刘巴，ABI 442）续体标记（payload[0]）。
pub(crate) const RESOLVE_TONGDUO_CHOICE: u32 = 0xDCDA_4420;

/// 通夺（刘巴，ABI 442）：成为其他角色单目标牌目标时（每回合一次）选一名有牌
/// 角色强制重铸一张牌。stage 0 选角色 → 1 该角色选牌 → 2 提交重铸。
pub(crate) fn resolve_tongduo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
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
            let candidates: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player && !candidate_hej_cards(tx, *candidate).is_empty()
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
                vec![RESOLVE_TONGDUO_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options = candidate_hej_cards(tx, target);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_TONGDUO_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            if !candidate_hej_cards(tx, target).contains(&card) {
                return Err(EffectError::Illegal(
                    "Tongduo recast card left zones".into(),
                ));
            }
            recast_cards(tx, target, vec![card])
        }
        _ => Err(EffectError::ContentFault("unknown Tongduo stage".into())),
    }
}

/// 一名角色手/装/判三区的全部牌（重铸可选范围）。
pub(crate) fn candidate_hej_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = Vec::new();
    for kind in [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge] {
        if let Some(zone) = tx.state().zones.find(kind, player) {
            if let Some(zone) = tx.state().zones.get(zone) {
                cards.extend(zone.cards.iter().copied());
            }
        }
    }
    cards
}

/// 怨清（华歆，ABI 443）：出牌阶段结束时，本阶段用过的每种类别（基本/锦囊/
/// 装备）从弃牌堆权威随机取一张置入仁库（技能牌堆）。
pub(crate) fn resolve_yuanqing(
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
    let mask = tx.state().turn.owner_card_category_mask(player);
    if mask == 0 {
        return Ok(Vec::new());
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::EngineFault("Yuanqing skill missing".into()))?;
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut effects = Vec::new();
    for category in 0..3u32 {
        if mask & (1 << category) == 0 {
            continue;
        }
        // 弃牌堆该类牌随机取一张（权威 RNG）。
        let candidates: Vec<CardId> = tx
            .state()
            .zones
            .get(discard)
            .map(|zone| {
                zone.cards
                    .iter()
                    .copied()
                    .filter(|card| card_category_of(tx.state(), *card) == Some(category))
                    .collect()
            })
            .unwrap_or_default();
        if candidates.is_empty() {
            continue;
        }
        let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
        let picked = candidates[rng.gen_range_u32(candidates.len() as u32) as usize];
        tx.state_mut().rng = rng.into_state();
        effects.push(EffectRequest::MoveCards {
            cards: vec![picked],
            to: pile,
            from: Some(discard),
        });
    }
    run_effects(tx, effects)
}

/// 疏陈（华歆，ABI 444）：其他角色濒死时（每轮一次）若仁库（怨清技能牌堆）
/// > 3 张则获得全部并令其回复 1。
pub(crate) fn resolve_shuchen(
    tx: &mut Transaction,
    player: PlayerId,
    dying: PlayerId,
    renku_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || dying == player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(dying)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let pile_cards = tx.state().skill_pile_cards(player, renku_def.raw());
    if pile_cards.len() <= 3 {
        return Ok(Vec::new());
    }
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    let pile_zone = tx
        .state()
        .zones
        .find(ZoneKind::Expansion, player)
        .and_then(|zone| {
            tx.state()
                .skill_piles
                .iter()
                .find(|pile| pile.zone == zone && pile.skill_def == renku_def.raw())
                .map(|_| zone)
        });
    let mut effects = Vec::new();
    if let Some(from) = pile_zone {
        effects.push(EffectRequest::MoveCards {
            cards: pile_cards,
            to: to_hand,
            from: Some(from),
        });
    }
    effects.push(EffectRequest::RecoverHp {
        player: dying,
        amount: 1,
        source: PlayerId::NONE,
    });
    run_effects(tx, effects)
}

/// 断笔（刘巴，ABI 445）续体标记（payload[0]）。
pub(crate) const RESOLVE_DUANBI_CHOICE: u32 = 0xDCDA_4450;

/// 断笔（刘巴，ABI 445）：限定技——全场手牌总数 > 人数×2 时可选任意名角色，
/// 逐目标强制弃 min(3, floor(手牌/2)) 张；全部弃完后再选一名角色权威随机获得
/// 至多 3 张仍处弃牌堆的被弃牌。被弃牌记录在 blob "DB1"。
pub(crate) fn resolve_duanbi(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 校验全场手牌总数 > 人数×2。
            let total_hand: usize = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|p| tx.state().hand_count(p) as usize)
                .sum();
            let players = tx.state().players.alive_by_seat().len();
            if total_hand <= players * 2 {
                return Ok(Vec::new());
            }
            let valid: Vec<PlayerId> = targets
                .iter()
                .copied()
                .filter(|t| {
                    *t != player
                        && tx
                            .state()
                            .players
                            .get(*t)
                            .is_some_and(|participant| participant.alive)
                        && !tx.state().hand_cards(*t).is_empty()
                })
                .collect();
            if valid.is_empty() {
                return Ok(Vec::new());
            }
            let first = valid[0];
            let count = duanbi_discard_count(tx, first);
            if count == 0 {
                return Ok(Vec::new());
            }
            // 技能启用 + owner/first 双方存活且异座由 GuardedChoice 守卫（ADR 0595）；
            // chooser 为 first，payload 尾部续带 valid[1..] 供续段按座次推进。
            let payload: Vec<u32> = vec![
                RESOLVE_DUANBI_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                first.raw(),
            ]
            .into_iter()
            .chain(valid[1..].iter().map(|t| t.raw()))
            .collect();
            GuardedChoice::new(
                skill,
                Participants::distinct_pair(player, first),
                ContinuationKind::Custom,
                payload,
            )
            .chosen_by(first)
            .shape(ChoiceShape::exactly(count as u8))
            .open_cards(tx, tx.state().hand_cards(first))
        }
        _ => Err(EffectError::ContentFault("unknown Duanbi stage".into())),
    }
}

/// 断笔目标应弃张数：min(3, floor(手牌/2))。
pub(crate) fn duanbi_discard_count(tx: &Transaction, target: PlayerId) -> usize {
    (tx.state().hand_count(target) as usize / 2).min(3)
}

/// 躁厉（孙翊，ABI 446）续体标记（payload[0]）。
pub(crate) const RESOLVE_ZAOLI_CHOICE: u32 = 0xDCDA_4460;

/// 躁厉（孙翊，ABI 446）：回合开始时有“躁厉”标记（使用/打出牌时 +1 至多 4，
/// DSL 表达）→ 弃至少一张牌（可全弃）并摸（标记数+弃牌数）张，总数 > 2 则
/// 失去 1 点体力。出牌阶段仅可使用本回合获得的牌由 DSL modifier
/// hand_card_gained_this_turn_only 表达。
pub(crate) fn resolve_zaoli(
    tx: &mut Transaction,
    player: PlayerId,
    mark: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || mark == 0
    {
        return Ok(Vec::new());
    }
    // 移除全部躁厉标记（amount 0 = 移除整项），开至少一张的可全弃弃牌 choice。
    let hand = tx.state().hand_cards(player);
    if hand.is_empty() {
        return run_effects(
            tx,
            vec![EffectRequest::RemoveMark {
                player,
                mark: engine_api::MarkDefId::new(ZAOLI_MARK),
                amount: 0,
            }],
        );
    }
    let events = run_effects(
        tx,
        vec![EffectRequest::RemoveMark {
            player,
            mark: engine_api::MarkDefId::new(ZAOLI_MARK),
            amount: 0,
        }],
    )?;
    let choice_events = open_choice(
        tx,
        player,
        1,
        hand.len() as u8,
        true,
        hand.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![RESOLVE_ZAOLI_CHOICE, 0, player.raw(), mark, skill.raw()],
    )?;
    let mut all = events;
    all.extend(choice_events);
    Ok(all)
}

/// 躁厉标记键（= 躁厉技能 def id）。
pub(crate) const ZAOLI_MARK: u32 = 2_486_461_241;

/// 孤影（向宠，ABI 447）续体标记（payload[0]）。
pub(crate) const RESOLVE_GUYING_CHOICE: u32 = 0xDCDA_4470;

/// 孤影（向宠，ABI 447）：你失去一张手/装/判区牌后（每回合一次，当前回合角色为
/// 他人）其须二选一——随机交给你一张其可获得的牌 / 你收回失去的牌（若仍在弃牌堆）。
pub(crate) fn resolve_guying(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || actor == player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            // 二选一：0 随机交给你一张可获得的牌 / 1 你收回失去的牌。
            let can_give = !crate::query::gainable_target_cards(
                tx.state(),
                player,
                actor,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty();
            let lost_in_discard = tx.state().cards.get(card).is_some_and(|value| {
                tx.state()
                    .zones
                    .get(value.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
            });
            if !can_give && !lost_in_discard {
                return Ok(Vec::new());
            }
            let mut options = Vec::new();
            if can_give {
                options.push(0);
            }
            if lost_in_discard {
                options.push(1);
            }
            if options.len() == 1 {
                // 单选项直接结算。
                return resolve_guying_commit(tx, player, actor, card, options[0]);
            }
            open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_GUYING_CHOICE,
                    player.raw(),
                    actor.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            // 由续体直接结算。
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("unknown Guying stage".into())),
    }
}

/// 孤影分支结算：0 随机交牌 / 1 收回弃牌堆的失去牌。
pub(crate) fn resolve_guying_commit(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    branch: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if branch == 0 {
        let gainable = crate::query::gainable_target_cards(
            tx.state(),
            player,
            actor,
            rule_api::TargetCardScope::HandEquip,
        );
        if gainable.is_empty() {
            return Ok(Vec::new());
        }
        let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
        let picked = gainable[rng.gen_range_u32(gainable.len() as u32) as usize];
        tx.state_mut().rng = rng.into_state();
        let from = tx
            .state()
            .cards
            .get(picked)
            .map(|value| value.zone)
            .ok_or_else(|| EffectError::Illegal("Guying gift card missing".into()))?;
        let to_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
        return move_cards_with_reason(
            tx,
            vec![picked],
            to_hand,
            Some(from),
            engine_api::CardMoveReason::Give,
        );
    }
    // 收回失去的牌（若仍在弃牌堆）。
    let Some(from) = tx.state().cards.get(card).map(|value| value.zone) else {
        return Ok(Vec::new());
    };
    if !tx
        .state()
        .zones
        .get(from)
        .is_some_and(|zone| zone.kind == ZoneKind::DiscardPile)
    {
        return Ok(Vec::new());
    }
    let to_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?;
    move_cards(tx, vec![card], to_hand, Some(from))
}

/// 流冰（吴懿，ABI 426）：出牌阶段首次实体杀改为方块（直接修改实体牌花色）。
pub(crate) fn resolve_liubing(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || card.is_none() {
        return Ok(Vec::new());
    }
    if let Some(value) = tx.state_mut().cards.get_mut(card) {
        if value.kind.is_sha() {
            value.suit = engine_api::Suit::Diamond;
        }
    }
    Ok(Vec::new())
}

/// 合击（吴懿，ABI 427）：其他角色单目标决斗/红杀后，可对原目标视为使用
/// 一张【杀】或【决斗】（二选一 choice）。
pub(crate) fn resolve_heji(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
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
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        vec![0, 1],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![RESOLVE_HEJI_CHOICE, player.raw(), target.raw(), skill.raw()],
    )
}

/// Custom-continuation marker for ABI 427 合击二选一（payload[0]）。
pub(crate) const RESOLVE_HEJI_CHOICE: u32 = 0xDCDA_4270;
