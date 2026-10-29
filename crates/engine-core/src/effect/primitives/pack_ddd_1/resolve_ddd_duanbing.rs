//! Effect primitives for `resolve_ddd_duanbing` and related helpers.

use super::*;

pub(crate) fn resolve_ddd_duanbing(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            let cards = crate::query::ddd_duanbing_cost_cards(tx.state(), player);
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || cards.is_empty()
            {
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
                vec![RESOLVE_DDD_DUANBING_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if card.is_none() || !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let cards = crate::query::ddd_duanbing_transfer_cards(tx.state(), player, target);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Judge, player)
                .ok_or_else(|| EffectError::EngineFault("短兵 owner judge zone missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Judge, target)
                .ok_or_else(|| EffectError::EngineFault("短兵 target judge zone missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to,
                    from: Some(from),
                }],
            )
            .map(|_| Vec::new())
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddDuanbing {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 3,
                    skill,
                },
            );
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }]).map(|_| Vec::new())
        }
        3 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            choose_use_virtual_sha(tx, player, true, false, false, skill, 0, 0)
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选短兵 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_duanbing_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [magic, stage, player, skill] = payload else {
        return Err(EffectError::Illegal("bad 3D精选短兵 choice".into()));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "3D精选短兵 requires one cost card".into(),
        ));
    };
    let selected = CardId::new(*selected);
    if *magic != RESOLVE_DDD_DUANBING_CHOICE
        || *stage != 0
        || cancelled
        || observer != player
        || !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !crate::query::ddd_duanbing_cost_cards(tx.state(), player).contains(&selected)
    {
        return Err(EffectError::Illegal("stale 3D精选短兵 cost".into()));
    }
    let from = tx
        .state()
        .cards
        .get(selected)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("3D精选短兵 cost disappeared".into()))?;
    let judge = tx
        .state()
        .zones
        .find(ZoneKind::Judge, player)
        .ok_or_else(|| EffectError::EngineFault("3D精选短兵 judge zone missing".into()))?;
    tx.state_mut()
        .cards
        .get_mut(selected)
        .expect("validated 3D精选短兵 cost")
        .kind = CardKind::TrickBingliang;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDddDuanbing {
            player,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 2,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![selected],
            to: judge,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn ddd_wuzhi_can_receive_bingliang(tx: &Transaction, target: PlayerId) -> bool {
    tx.state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        && !tx.state().disabled_judge_zones.contains(&target)
        && tx
            .state()
            .zones
            .find(ZoneKind::Judge, target)
            .and_then(|zone| tx.state().zones.get(zone))
            .is_some_and(|zone| {
                zone.cards.iter().all(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_none_or(|card| card.kind != CardKind::TrickBingliang)
                })
            })
}

pub(crate) fn ddd_wuzhi_xuxiao_card(tx: &Transaction, player: PlayerId, card: CardId) -> bool {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    tx.state().cards.get(card).is_some_and(|state| {
        Some(state.zone) == discard
            && state.owner.is_none()
            && crate::query::effective_card_kind(tx.state(), player, state).is_basic()
            && crate::query::effective_card_color_mask(tx.state(), player, card) == 0b10
    }) && tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .any(|target| ddd_wuzhi_can_receive_bingliang(tx, target))
}

pub(crate) fn ddd_wuzhi_finish_lingyong_card(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("灵涌处理区不存在".into()))?;
    if tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.zone == processing)
    {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("灵涌弃牌堆不存在".into()))?;
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveDddWuzhi {
                player,
                card: CardId::NONE,
                action: 5,
                cards: Vec::new(),
                skill,
            });
        return move_cards(tx, vec![card], discard, Some(processing));
    }
    resolve_ddd_wuzhi(tx, player, CardId::NONE, 5, Vec::new(), skill)
}

pub(crate) fn resolve_ddd_wuzhi(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let result = crate::command::judge::flip_judgment(
                tx,
                player,
                crate::command::judge::JudgmentFavor::Neutral,
            )?;
            let result_card = result.card;
            let is_sha = tx.state().cards.get(result_card).is_some_and(|card| {
                crate::query::effective_card_kind(tx.state(), player, card).is_sha()
            });
            if !is_sha {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveDddWuzhi {
                        player,
                        card: result_card,
                        action: 1,
                        cards: Vec::new(),
                        skill,
                    },
                );
            }
            Ok(vec![completed_judgment_event(player, result, CardId::NONE)])
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("灵涌弃牌堆不存在".into()))?;
            if !tx
                .state()
                .cards
                .get(card)
                .is_some_and(|state| state.zone == discard)
            {
                return resolve_ddd_wuzhi(tx, player, CardId::NONE, 5, Vec::new(), skill);
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("灵涌处理区不存在".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddWuzhi {
                    player,
                    card,
                    action: 2,
                    cards: Vec::new(),
                    skill,
                });
            move_cards(tx, vec![card], processing, Some(discard))
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || crate::command::play::skill_card_target_spec(tx, player, card).is_err()
            {
                return ddd_wuzhi_finish_lingyong_card(tx, player, card, skill);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Confirm,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_WUZHI_CHOICE,
                    0,
                    player.raw(),
                    skill.raw(),
                    card.raw(),
                ],
            )
        }
        3 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return ddd_wuzhi_finish_lingyong_card(tx, player, card, skill);
            }
            match crate::command::play::skill_card_target_spec(tx, player, card)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    tx.state_mut().resolution.deferred_tail_effects.push(
                        EffectRequest::ResolveDddWuzhi {
                            player,
                            card,
                            action: 4,
                            cards: Vec::new(),
                            skill,
                        },
                    );
                    crate::command::play::handle_skill_card(tx, player, card, &targets)?;
                    Ok(Vec::new())
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    open_choice(
                        tx,
                        player,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        vec![
                            RESOLVE_DDD_WUZHI_CHOICE,
                            1,
                            player.raw(),
                            skill.raw(),
                            card.raw(),
                        ],
                    )
                }
                crate::command::play::SkillCardTargetSpec::Pairs(pairs) => open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    (1..=pairs.len() as u32).collect(),
                    ChoiceKind::Custom,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_DDD_WUZHI_CHOICE,
                        2,
                        player.raw(),
                        skill.raw(),
                        card.raw(),
                    ],
                ),
            }
        }
        4 => ddd_wuzhi_finish_lingyong_card(tx, player, card, skill),
        5 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                vec![0, 1],
                ChoiceKind::Confirm,
                ContinuationKind::Custom,
                vec![RESOLVE_DDD_WUZHI_CHOICE, 3, player.raw(), skill.raw()],
            )
        }
        10 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let mut seen = std::collections::BTreeSet::new();
            let cards: Vec<_> = cards
                .into_iter()
                .filter(|card| seen.insert(card.raw()))
                .filter(|card| ddd_wuzhi_xuxiao_card(tx, player, *card))
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                RESOLVE_DDD_WUZHI_CHOICE,
                10,
                player.raw(),
                skill.raw(),
                cards.len() as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 3D精选吴质 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_wuzhi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_WUZHI_CHOICE) {
        return Err(EffectError::Illegal("bad 3D精选吴质 choice".into()));
    }
    let stage = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal("吴质 choice observer changed".into()));
    }
    match stage {
        0 => {
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            if cancelled || selections == [0] {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveDddWuzhi {
                        player,
                        card,
                        action: 4,
                        cards: Vec::new(),
                        skill,
                    }],
                )
                .map(|_| ());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("灵涌使用选项非法".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddWuzhi {
                    player,
                    card,
                    action: 3,
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if cancelled {
                return Err(EffectError::Illegal("灵涌目标选择不可取消".into()));
            }
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let crate::command::play::SkillCardTargetSpec::Select { min, max, options } =
                crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("灵涌牌目标形状已变化".into()));
            };
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let mut seen = std::collections::BTreeSet::new();
            if targets.len() < usize::from(min)
                || targets.len() > usize::from(max)
                || targets
                    .iter()
                    .any(|target| !seen.insert(*target) || !options.contains(target))
            {
                return Err(EffectError::Illegal("灵涌牌目标非法或过期".into()));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddWuzhi {
                    player,
                    card,
                    action: 4,
                    cards: Vec::new(),
                    skill,
                });
            crate::command::play::handle_skill_card(tx, player, card, &targets)
        }
        2 => {
            if cancelled {
                return Err(EffectError::Illegal("灵涌目标对选择不可取消".into()));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("灵涌须选择一个目标对".into()));
            };
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let crate::command::play::SkillCardTargetSpec::Pairs(pairs) =
                crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal("灵涌牌目标对形状已变化".into()));
            };
            let index = selected
                .checked_sub(1)
                .map(|value| value as usize)
                .ok_or_else(|| EffectError::Illegal("灵涌目标对索引非法".into()))?;
            let (left, right) = pairs
                .get(index)
                .copied()
                .ok_or_else(|| EffectError::Illegal("灵涌目标对已过期".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveDddWuzhi {
                    player,
                    card,
                    action: 4,
                    cards: Vec::new(),
                    skill,
                });
            crate::command::play::handle_skill_card(tx, player, card, &[left, right])
        }
        3 => {
            if cancelled || selections.is_empty() || selections == [0] {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("灵涌继续选项非法".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDddWuzhi {
                    player,
                    card: CardId::NONE,
                    action: 0,
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        10 => {
            let len = *payload.get(4).unwrap_or(&0) as usize;
            let offered = payload
                .get(5..5 + len)
                .ok_or_else(|| EffectError::EngineFault("虚孝候选牌被截断".into()))?;
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("虚孝每次只能选一张牌".into()));
            };
            let card = CardId::new(*selected);
            if !offered.contains(selected) || !ddd_wuzhi_xuxiao_card(tx, player, card) {
                return Err(EffectError::Illegal("虚孝候选牌非法或过期".into()));
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| ddd_wuzhi_can_receive_bingliang(tx, *target))
                .collect();
            let remaining: Vec<_> = offered
                .iter()
                .copied()
                .filter(|candidate| *candidate != card.raw())
                .collect();
            let mut next = vec![
                RESOLVE_DDD_WUZHI_CHOICE,
                11,
                player.raw(),
                skill.raw(),
                card.raw(),
                remaining.len() as u32,
            ];
            next.extend(remaining);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        11 => {
            if cancelled {
                return Err(EffectError::Illegal("虚孝目标选择不可取消".into()));
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("虚孝须选择一名目标".into()));
            };
            let target = PlayerId::new(*target);
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let len = *payload.get(5).unwrap_or(&0) as usize;
            let remaining = payload
                .get(6..6 + len)
                .ok_or_else(|| EffectError::EngineFault("虚孝剩余牌被截断".into()))?;
            if !originating_skill_is_enabled(tx, player, skill)
                || !ddd_wuzhi_xuxiao_card(tx, player, card)
                || !ddd_wuzhi_can_receive_bingliang(tx, target)
            {
                return Err(EffectError::Illegal("虚孝牌或目标已过期".into()));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("虚孝弃牌堆不存在".into()))?;
            let judge = tx
                .state()
                .zones
                .find(ZoneKind::Judge, target)
                .ok_or_else(|| EffectError::EngineFault("虚孝判定区不存在".into()))?;
            tx.state_mut()
                .cards
                .get_mut(card)
                .expect("validated 虚孝 card")
                .kind = CardKind::TrickBingliang;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to: judge,
                        from: Some(discard),
                    },
                    EffectRequest::DrawCards {
                        player: target,
                        count: 2,
                    },
                    EffectRequest::ResolveDddWuzhi {
                        player,
                        card: CardId::NONE,
                        action: 10,
                        cards: remaining.iter().copied().map(CardId::new).collect(),
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "unknown 3D精选吴质 choice stage".into(),
        )),
    }
}

pub(crate) fn ddd_xuyu_pending(
    state: &crate::state::GameState,
    skill: engine_api::SkillId,
) -> bool {
    state
        .skills
        .get(skill)
        .is_some_and(|instance| instance.state_blob == [b'D', b'X', b'U', b'1', 1])
}

pub(crate) fn set_ddd_xuyu_pending(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    pending: bool,
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = DDD_XUYU_STATE
            .iter()
            .copied()
            .chain(std::iter::once(u8::from(pending)))
            .collect();
    }
}

pub(crate) fn ddd_shijian_watched(
    state: &crate::state::GameState,
    skill: engine_api::SkillId,
) -> Option<PlayerId> {
    let blob = &state.skills.get(skill)?.state_blob;
    if blob.len() != 8 || &blob[..4] != DDD_SHIJIAN_STATE {
        return None;
    }
    let player = PlayerId::new(u32::from_le_bytes(blob[4..8].try_into().ok()?));
    player.is_some().then_some(player)
}

pub(crate) fn set_ddd_shijian_watched(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    player: PlayerId,
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = DDD_SHIJIAN_STATE.to_vec();
        instance
            .state_blob
            .extend_from_slice(&player.raw().to_le_bytes());
    }
}

pub(crate) fn ddd_xujing_hand_equip_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, player))
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect()
}
