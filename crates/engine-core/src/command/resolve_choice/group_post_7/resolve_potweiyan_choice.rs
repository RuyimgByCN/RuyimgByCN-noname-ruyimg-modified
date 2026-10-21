//! Command operations for `resolve_potweiyan_choice` and related helpers.

use super::*;

pub(crate) fn resolve_potweiyan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 2 || payload[0] != crate::effect::POTWEIYAN_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 势魏延 payload".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() != 10 || observer.raw() != payload[2] {
                return Err(EffectError::Illegal("invalid 壮誓弃牌选择".into()));
            }
            let player = PlayerId::new(payload[2]);
            let mission = SkillId::new(payload[3]);
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || !tx.state().skills.get(mission).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
            {
                return Err(EffectError::Illegal("壮誓阶段已变化".into()));
            }
            let defs = payload[4..10]
                .iter()
                .copied()
                .map(SkillDefId::new)
                .collect::<Vec<_>>();
            if cancelled {
                return crate::effect::open_potweiyan_hp_choice(
                    tx, player, mission, defs[0], defs[1], defs[2], defs[3], defs[4], defs[5], 0,
                )
                .map(|_| ());
            }
            if selections.is_empty() || selections.len() > u8::MAX as usize {
                return Err(EffectError::Illegal("壮誓需选择至少一张手牌".into()));
            }
            let legal: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .into_iter()
                    .filter(|card| tx.state().hand_cards(player).contains(card))
                    .collect();
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != cards.len()
                || cards.iter().any(|card| !legal.contains(card))
            {
                return Err(EffectError::Illegal("壮誓弃牌已变化".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: cards.clone(),
                    },
                    EffectRequest::ResolvePotweiyan {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::new(cards.len() as u32),
                        action: 14,
                        skill: mission,
                    },
                ],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() != 11 || observer.raw() != payload[2] {
                return Err(EffectError::Illegal("invalid 壮誓失去体力选择".into()));
            }
            let player = PlayerId::new(payload[2]);
            let mission = SkillId::new(payload[3]);
            if cancelled {
                if payload[10] == 0 {
                    return run_effects(
                        tx,
                        vec![EffectRequest::ResolvePotweiyan {
                            player,
                            target: PlayerId::NONE,
                            card: CardId::NONE,
                            action: 3,
                            skill: mission,
                        }],
                    )
                    .map(|_| ());
                }
                return Ok(());
            }
            let [amount] = selections else {
                return Err(EffectError::Illegal("壮誓需选择体力值".into()));
            };
            let hp = tx
                .state()
                .players
                .get(player)
                .map_or(0, |participant| participant.hp.max(0) as u32);
            if *amount == 0 || *amount > hp {
                return Err(EffectError::Illegal("壮誓失去体力数已变化".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::ResolvePotweiyan {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::new(*amount),
                        action: 15,
                        skill: mission,
                    },
                    EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: *amount,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    },
                ],
            )
            .map(|_| ())
        }
        2 => {
            if payload.len() != 5
                || observer.raw() != payload[2]
                || cancelled
                || selections.len() != 1
            {
                return Err(EffectError::Illegal("invalid 饮战弃牌选择".into()));
            }
            let player = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            let target = PlayerId::new(payload[4]);
            let card = CardId::new(selections[0]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&card) {
                return Err(EffectError::Illegal("饮战目标牌已变化".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: target,
                        cards: vec![card],
                    },
                    EffectRequest::ResolvePotweiyan {
                        player,
                        target,
                        card,
                        action: 11,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        3 => {
            if payload.len() != 5 || observer.raw() != payload[2] {
                return Err(EffectError::Illegal("invalid 狂骨选择".into()));
            }
            if cancelled {
                return Ok(());
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal("狂骨需选择一项".into()));
            };
            let player = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            let effects = match *choice {
                0 => vec![EffectRequest::RecoverHp {
                    player,
                    amount: 1,
                    source: PlayerId::NONE,
                }],
                1 => vec![EffectRequest::DrawCards { player, count: 1 }],
                2 => vec![
                    EffectRequest::RecoverHp {
                        player,
                        amount: 1,
                        source: PlayerId::NONE,
                    },
                    EffectRequest::DrawCards { player, count: 1 },
                    EffectRequest::ResolvePotweiyan {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        action: 12,
                        skill,
                    },
                ],
                _ => return Err(EffectError::Illegal("invalid 狂骨分支".into())),
            };
            run_effects(tx, effects).map(|_| ())
        }
        4 => {
            if payload.len() != 5
                || observer.raw() != payload[2]
                || cancelled
                || selections.len() != 1
            {
                return Err(EffectError::Illegal("invalid 狂骨背水弃牌".into()));
            }
            let player = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            let card = CardId::new(selections[0]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal("狂骨背水弃牌已变化".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![card],
                    },
                    EffectRequest::ResolvePotweiyan {
                        player,
                        target: PlayerId::NONE,
                        card: CardId::NONE,
                        action: 13,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 势魏延 choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_potjixi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || observer.raw() != payload[2] {
        return Err(EffectError::Illegal("invalid 势急袭选择者".into()));
    }
    let player = PlayerId::new(payload[2]);
    match payload[1] {
        0 => {
            if payload.len() != 5 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid 势急袭弃牌选择".into()));
            }
            let current = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let card = CardId::new(selections[0]);
            if !crate::query::discardable_target_cards(
                tx.state(),
                player,
                current,
                rule_api::TargetCardScope::HandEquip,
            )
            .contains(&card)
            {
                return Err(EffectError::Illegal("势急袭弃牌已不可用".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: current,
                        cards: vec![card],
                    },
                    EffectRequest::ResolvePotjixi {
                        player,
                        target: current,
                        action: 3,
                        targets: Vec::new(),
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() != 4 {
                return Err(EffectError::EngineFault("malformed 势急袭目标选择".into()));
            }
            if cancelled {
                return Ok(());
            }
            if selections.is_empty() {
                return Err(EffectError::Illegal("势急袭至少需要一个目标".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolvePotjixi {
                    player,
                    target: PlayerId::NONE,
                    action: 4,
                    targets: selections.iter().copied().map(PlayerId::new).collect(),
                    skill: engine_api::SkillId::new(payload[3]),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 势急袭 choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_potceduan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("invalid 势策断展示选择".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let count = payload[4] as usize;
    let index = payload[5] as usize;
    if count == 0 || index >= count || payload.len() != 6 + count + index {
        return Err(EffectError::EngineFault("malformed 势策断展示续体".into()));
    }
    let candidates: Vec<_> = payload[6..6 + count]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    let expected = candidates[index];
    let selected = CardId::new(selections[0]);
    if observer != expected || !tx.state().hand_cards(expected).contains(&selected) {
        return Err(EffectError::Illegal("势策断展示牌已变化".into()));
    }
    let mut next = payload.to_vec();
    next.push(selected.raw());
    next[5] = (index + 1) as u32;
    if index + 1 < count {
        let observer = candidates[index + 1];
        return crate::effect::open_choice(
            tx,
            observer,
            1,
            1,
            false,
            tx.state()
                .hand_cards(observer)
                .into_iter()
                .map(CardId::raw)
                .collect(),
            engine_api::ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            next,
        )
        .map(|_| ());
    }
    let revealed = next[6 + count..]
        .iter()
        .copied()
        .map(CardId::new)
        .collect::<Vec<_>>();
    crate::effect::complete_potceduan(tx, player, target, skill, &candidates, &revealed)
}

pub(crate) fn resolve_mantianguohai_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || observer.raw() != payload[2] || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("invalid 瞒天过海 card choice".into()));
    }
    let stage = payload[1];
    let actor = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let selected = CardId::new(selections[0]);
    let Some(card) = tx.state().cards.get(selected).cloned() else {
        return Err(EffectError::Illegal("瞒天过海 selected card moved".into()));
    };
    match stage {
        0 => {
            let queue_len = payload[4] as usize;
            if payload.len() != 5 + queue_len
                || !crate::query::gainable_target_cards(
                    tx.state(),
                    actor,
                    target,
                    rule_api::TargetCardScope::HandEquipJudge,
                )
                .contains(&selected)
            {
                return Err(EffectError::Illegal("invalid 瞒天过海 gain choice".into()));
            }
            let mut queue = payload[5..].to_vec();
            if queue.len() < 7 {
                return Err(EffectError::EngineFault("malformed 瞒天过海 queue".into()));
            }
            let index = queue[5] as usize;
            let count = queue[6] as usize;
            if index >= count
                || queue.len() < 7 + count
                || PlayerId::new(queue[7 + index]) != target
            {
                return Err(EffectError::EngineFault("stale 瞒天过海 queue".into()));
            }
            queue[7 + index] |= crate::command::play::MANTIANGUOHAI_SUCCESS_MASK;
            queue[5] = queue[5].saturating_add(1);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, actor)
                .ok_or_else(|| EffectError::Illegal("瞒天过海 user has no hand zone".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCardsWithReason {
                        cards: vec![selected],
                        to: hand,
                        from: Some(card.zone),
                        reason: engine_api::CardMoveReason::Unspecified,
                    },
                    EffectRequest::ResolveMantianguohai {
                        player: actor,
                        stage: 0,
                        payload: queue,
                    },
                ],
            )
            .map(|_| ())
        }
        1 => {
            let count = payload[4] as usize;
            if payload.len() != 5 + count
                || count == 0
                || PlayerId::new(payload[5]) != target
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|player| player.alive)
            {
                return Err(EffectError::Illegal("stale 瞒天过海 give target".into()));
            }
            let hand = tx.state().zones.find(ZoneKind::Hand, actor);
            let equip = tx.state().zones.find(ZoneKind::Equip, actor);
            if card.owner != actor || ![hand, equip].contains(&Some(card.zone)) {
                return Err(EffectError::Illegal("invalid 瞒天过海 give card".into()));
            }
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("瞒天过海 target has no hand zone".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCardsWithReason {
                        cards: vec![selected],
                        to: target_hand,
                        from: Some(card.zone),
                        reason: engine_api::CardMoveReason::Give,
                    },
                    EffectRequest::ResolveMantianguohai {
                        player: actor,
                        stage: 1,
                        payload: payload[6..].to_vec(),
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 瞒天过海 choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_potchengpu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4
        || payload[0] != crate::effect::POTCHENGPU_CUSTOM_CHOICE
        || cancelled
        || selections.len() != 1 && payload[1] != 3
    {
        return Err(EffectError::Illegal("invalid 势程普 selection".into()));
    }
    let player = PlayerId::new(payload[2]);
    match payload[1] {
        0 if observer == player && payload.len() == 4 => crate::effect::run_effects(
            tx,
            vec![EffectRequest::ResolvePotchengpu {
                player,
                target: PlayerId::new(selections[0]),
                stage: 1,
                damage_id: 0,
                skill: SkillId::new(payload[3]),
            }],
        )
        .map(|_| ()),
        2 if observer == player && payload.len() == 6 => crate::effect::run_effects(
            tx,
            vec![EffectRequest::ResolvePotchengpu {
                player,
                target: PlayerId::new(selections[0]),
                stage: 3,
                damage_id: payload[4],
                skill: SkillId::new(payload[5]),
            }],
        )
        .map(|_| ()),
        3 if payload.len() >= 7 && observer.raw() == payload[3] => {
            let need = payload[6] as usize;
            if selections.len() != need
                || selections
                    .iter()
                    .any(|selected| !payload[7..].contains(selected))
            {
                return Err(EffectError::Illegal("invalid 避憾 hand cards".into()));
            }
            crate::effect::commit_potbihan(
                tx,
                observer,
                payload[4],
                selections.iter().copied().map(CardId::new).collect(),
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid 势程普 choice stage".into())),
    }
}

pub(crate) fn resolve_potjugu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload[0] != crate::effect::POTJUGU_CUSTOM_CHOICE
        || observer.raw() != payload[1]
        || cancelled
        || selections.len() != payload[3] as usize
        || selections
            .iter()
            .any(|selected| !payload[4..].contains(selected))
    {
        return Err(EffectError::Illegal("invalid 拒孤 selection".into()));
    }
    crate::effect::discard_cards(
        tx,
        observer,
        selections.iter().copied().map(CardId::new).collect(),
    )
    .map(|_| ())
}

pub(crate) fn resolve_potsifeng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 || payload[0] != crate::effect::POTSIFENG_CUSTOM_CHOICE || cancelled {
        return Err(EffectError::Illegal("invalid 伺锋 selection".into()));
    }
    match payload[1] {
        0 if payload.len() >= 5 && observer.raw() == payload[2] && !selections.is_empty() => {
            let cards = &payload[4..];
            if selections.iter().any(|selected| !cards.contains(selected)) {
                return Err(EffectError::Illegal("invalid 伺锋 cards".into()));
            }
            let remaining: Vec<_> = cards
                .iter()
                .copied()
                .filter(|card| !selections.contains(card))
                .collect();
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != observer)
                .map(PlayerId::raw)
                .collect();
            let mut next = vec![
                crate::effect::POTSIFENG_CUSTOM_CHOICE,
                1,
                observer.raw(),
                payload[3],
                selections.len() as u32,
            ];
            next.extend_from_slice(selections);
            next.extend(remaining);
            crate::effect::open_choice(
                tx,
                observer,
                1,
                1,
                false,
                options,
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        1 if payload.len() >= 6 && observer.raw() == payload[2] && selections.len() == 1 => {
            let count = payload[4] as usize;
            if payload.len() < 5 + count {
                return Err(EffectError::Illegal("invalid 伺锋 distribution".into()));
            }
            crate::effect::commit_potsifeng_distribution(
                tx,
                observer,
                PlayerId::new(selections[0]),
                SkillId::new(payload[3]),
                payload[5..5 + count]
                    .iter()
                    .copied()
                    .map(CardId::new)
                    .collect(),
                payload[5 + count..]
                    .iter()
                    .copied()
                    .map(CardId::new)
                    .collect(),
            )
            .map(|_| ())
        }
        2 if observer.raw() == payload[2] && selections.len() == 1 => {
            if !payload[3..].contains(&selections[0]) {
                return Err(EffectError::Illegal("invalid 伺锋 hand discard".into()));
            }
            crate::effect::discard_cards(tx, observer, vec![CardId::new(selections[0])]).map(|_| ())
        }
        3 if payload.len() == 5 && observer.raw() == payload[2] && selections.len() == 1 => {
            let stage = match selections[0] {
                0 => 5,
                1 => 6,
                _ => return Err(EffectError::Illegal("invalid 伺锋 branch".into())),
            };
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::ResolvePotsifeng {
                    player: observer,
                    target: PlayerId::new(payload[3]),
                    card: CardId::NONE,
                    stage,
                    skill: SkillId::new(payload[4]),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid 伺锋 choice stage".into())),
    }
}

pub(crate) fn resolve_potchenqun(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != crate::effect::POTCHENQUN_CUSTOM_CHOICE {
        return Err(EffectError::Illegal("invalid 势陈群 selection".into()));
    }
    match payload[1] {
        0 if payload.len() == 5 && observer.raw() == payload[2] => {
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("invalid 法恩 branch".into()));
            }
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let skill = SkillId::new(payload[4]);
            match selections[0] {
                1 => {
                    crate::effect::set_potfaen_pending(tx, skill, 2);
                    crate::effect::run_effects(
                        tx,
                        vec![EffectRequest::DrawCards {
                            player: target,
                            count: 1,
                        }],
                    )
                    .map(|_| ())
                }
                2 => {
                    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
                    if options.is_empty() {
                        return Err(EffectError::Illegal(
                            "法恩 discard no longer available".into(),
                        ));
                    }
                    let mut next = vec![
                        crate::effect::POTCHENQUN_CUSTOM_CHOICE,
                        1,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ];
                    next.extend(options.iter().map(|card| card.raw()));
                    crate::effect::open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        options.into_iter().map(CardId::raw).collect(),
                        ChoiceKind::SelectCards,
                        ContinuationKind::Custom,
                        next,
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal("invalid 法恩 branch".into())),
            }
        }
        1 if payload.len() >= 6 && observer.raw() == payload[3] && !cancelled => {
            if selections.len() != 1 || !payload[5..].contains(&selections[0]) {
                return Err(EffectError::Illegal("invalid 法恩 discard".into()));
            }
            let skill = SkillId::new(payload[4]);
            crate::effect::set_potfaen_pending(tx, skill, 1);
            crate::effect::run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: observer,
                    cards: vec![CardId::new(selections[0])],
                }],
            )
            .map(|_| ())
        }
        2 if payload.len() >= 7 && observer.raw() == payload[3] && !cancelled => {
            if selections.len() != 1 || !payload[6..].contains(&selections[0]) {
                return Err(EffectError::Illegal("invalid 法恩 delayed discard".into()));
            }
            let mut effects = vec![EffectRequest::DiscardCards {
                player: observer,
                cards: vec![CardId::new(selections[0])],
            }];
            if payload[5] != 0 {
                effects.push(EffectRequest::ResolvePotchenqun {
                    player: PlayerId::new(payload[2]),
                    target: observer,
                    stage: 2,
                    skill: SkillId::new(payload[4]),
                });
            }
            crate::effect::run_effects(tx, effects).map(|_| ())
        }
        3 if payload.len() >= 5 && observer.raw() == payload[2] && !cancelled => {
            if selections.len() != 1 || !payload[4..].contains(&selections[0]) {
                return Err(EffectError::Illegal("invalid 定品 target".into()));
            }
            let target = PlayerId::new(selections[0]);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("定品 target no longer alive".into()));
            }
            crate::effect::queue_extra_draw_phase(tx, target);
            Ok(())
        }
        _ => Err(EffectError::Illegal("invalid 势陈群 choice stage".into())),
    }
}

pub(crate) fn resolve_potchendao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != crate::effect::POTCHENDAO_CUSTOM_CHOICE {
        return Err(EffectError::Illegal("invalid 势陈到 selection".into()));
    }
    match payload[1] {
        0 if payload.len() >= 6 && observer.raw() == payload[2] => {
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 || !payload[5..].contains(&selections[0]) {
                return Err(EffectError::Illegal("invalid 往烈 card".into()));
            }
            crate::effect::arm_potwanglie(
                tx,
                observer,
                CardId::new(selections[0]),
                SkillId::new(payload[4]),
            )
        }
        1 if payload.len() == 4 && observer.raw() == payload[2] && !cancelled => {
            if selections.len() != 1 || selections[0] > 1 {
                return Err(EffectError::Illegal("invalid 弘毅 branch".into()));
            }
            crate::effect::commit_pothongyi_prepare(
                tx,
                observer,
                SkillId::new(payload[3]),
                selections[0] as u8,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid 势陈到 choice stage".into())),
    }
}

pub(crate) fn resolve_mbzhuji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4
        || payload[0] != crate::effect::MBZHUJI_CUSTOM_CHOICE
        || observer.raw() != payload[2]
    {
        return Err(EffectError::Illegal("invalid 筑墼 selection".into()));
    }
    if cancelled && payload[1] == 0 {
        return Ok(());
    }
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("invalid 筑墼 selection".into()));
    }
    let (stage, choice) = match (payload[1], selections[0]) {
        (0, choice @ 1..=4) => (1, choice as u8),
        (1, choice @ 1..=3) => (5, choice as u8),
        _ => return Err(EffectError::Illegal("invalid 筑墼 selection".into())),
    };
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbzhuji {
            player: observer,
            choice,
            stage,
            skill: SkillId::new(payload[3]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_potpoxi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != crate::effect::POTPOXI_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("truncated 破袭 selection".into()));
    }
    let player = PlayerId::new(payload[1]);
    let card = CardId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let remaining_count = payload[4] as usize;
    let legal_count_at = 5usize.saturating_add(remaining_count);
    let legal_count = *payload
        .get(legal_count_at)
        .ok_or_else(|| EffectError::EngineFault("truncated 破袭 target count".into()))?
        as usize;
    if payload.len() != legal_count_at.saturating_add(1).saturating_add(legal_count)
        || observer != player
        || cancelled
        || selections.is_empty()
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "破袭 selection context changed".into(),
        ));
    }
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    let legal_now = crate::command::play::prompted_sha_targets_unlimited(tx, player, card);
    if selected.len() != selections.len()
        || !selected
            .iter()
            .all(|target| payload[legal_count_at + 1..].contains(target))
        || !selections
            .iter()
            .all(|target| legal_now.contains(&PlayerId::new(*target)))
    {
        return Err(EffectError::Illegal(
            "破袭 targets are no longer legal".into(),
        ));
    }
    let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    let owner_hand = tx.state().hand_count(player);
    let unrespondable: Vec<_> = targets
        .iter()
        .copied()
        .filter(|target| tx.state().hand_count(*target) <= owner_hand)
        .collect();
    let remaining: Vec<_> = payload[5..legal_count_at]
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    if !remaining.is_empty() {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolvePotpoxi {
                player,
                cards: remaining,
                skill,
            });
    }
    crate::command::play::play_discard_sha_unlimited(tx, player, card, &targets, &unrespondable)
}

pub(crate) fn resolve_potfuan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5
        || payload[0] != crate::effect::POTFUAN_CUSTOM_CHOICE
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("invalid 伏暗 selection".into()));
    }
    match payload[1] {
        0 if payload.len() == 6 && observer.raw() == payload[2] => {
            crate::effect::commit_potfuan_suit(
                tx,
                observer,
                selections[0],
                SkillId::new(payload[5]),
            )
        }
        1 if payload.len() >= 9 && observer.raw() == payload[2] => {
            if !payload[8..].contains(&selections[0]) {
                return Err(EffectError::Illegal("invalid 伏暗 target".into()));
            }
            crate::effect::commit_potfuan_target(
                tx,
                observer,
                PlayerId::new(selections[0]),
                CardId::new(payload[3]),
                payload[4],
                SkillId::new(payload[7]),
            )
        }
        2 if payload.len() == 5 && observer.raw() == payload[2] => {
            crate::effect::commit_potfuan_branch(
                tx,
                observer,
                PlayerId::new(payload[3]),
                selections[0],
                SkillId::new(payload[4]),
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid 伏暗 choice stage".into())),
    }
}

pub(crate) fn resolve_mbjianji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 7
        || payload[0] != crate::effect::MBJIANJI_CUSTOM_CHOICE
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("invalid 间计 selection".into()));
    }
    let player = PlayerId::new(payload[2]);
    let shared = CardId::new(payload[3]);
    let first = PlayerId::new(payload[4]);
    let second = PlayerId::new(payload[5]);
    let skill = SkillId::new(payload[6]);
    let selected = CardId::new(selections[0]);
    let valid = |participant: PlayerId| {
        selected == shared
            || tx.state().cards.get(selected).is_some_and(|card| {
                card.owner == participant
                    && tx.state().zones.get(card.zone).is_some_and(|zone| {
                        zone.kind == ZoneKind::Hand && zone.owner == participant
                    })
            })
    };
    match payload[1] {
        1 if observer == first && valid(first) => {
            let mut options: Vec<_> = tx
                .state()
                .hand_cards(second)
                .into_iter()
                .map(CardId::raw)
                .collect();
            options.push(shared.raw());
            options.sort_unstable();
            options.dedup();
            crate::effect::open_choice(
                tx,
                second,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::MBJIANJI_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    shared.raw(),
                    first.raw(),
                    second.raw(),
                    skill.raw(),
                    selected.raw(),
                ],
            )
            .map(|_| ())
        }
        2 if payload.len() == 8 && observer == second && valid(second) => {
            crate::effect::commit_mbjianji_compare(
                tx,
                player,
                first,
                second,
                shared,
                CardId::new(payload[7]),
                selected,
                skill,
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "间计 comparison context changed".into(),
        )),
    }
}

pub(crate) fn resolve_mbyuanmo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4
        || payload[0] != crate::effect::MBYUANMO_CUSTOM_CHOICE
        || observer != PlayerId::new(payload[2])
    {
        return Err(EffectError::EngineFault("bad 远谟 continuation".into()));
    }
    let player = PlayerId::new(payload[2]);
    match payload[1] {
        0 if !cancelled && selections.len() == 1 => {
            let card = CardId::new(selections[0]);
            let card_state = tx
                .state()
                .cards
                .get(card)
                .ok_or_else(|| EffectError::Illegal("远谟 field card missing".into()))?;
            let source = tx
                .state()
                .zones
                .get(card_state.zone)
                .filter(|zone| matches!(zone.kind, ZoneKind::Equip | ZoneKind::Judge))
                .map(|zone| zone.owner)
                .ok_or_else(|| EffectError::Illegal("远谟 card left the field".into()))?;
            let destinations = crate::query::field_card_move_destinations(tx.state(), card);
            if destinations.is_empty() {
                return Err(EffectError::Illegal(
                    "远谟 card is no longer movable".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                destinations.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    crate::effect::MBYUANMO_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    payload[3],
                    card.raw(),
                    source.raw(),
                    crate::effect::attack_range_population(tx, source),
                ],
            )
            .map(|_| ())
        }
        1 if payload.len() == 7 && !cancelled && selections.len() == 1 => {
            let skill = SkillId::new(payload[3]);
            let card = CardId::new(payload[4]);
            let source = PlayerId::new(payload[5]);
            let destination = PlayerId::new(selections[0]);
            if !crate::query::field_card_move_destinations(tx.state(), card).contains(&destination)
            {
                return Err(EffectError::Illegal("远谟 destination changed".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let kind = tx.state().zones.get(from).unwrap().kind;
            let to = tx
                .state()
                .zones
                .find(kind, destination)
                .ok_or_else(|| EffectError::Illegal("远谟 destination zone missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to,
                        from: Some(from),
                    },
                    EffectRequest::ResolveMbyuanmo {
                        player,
                        target: source,
                        stage: 1,
                        card,
                        value: payload[6],
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        2 if cancelled && selections.is_empty() => Ok(()),
        2 if payload.len() == 6 && !cancelled && selections == [1] => run_effects(
            tx,
            vec![EffectRequest::ResolveMbyuanmo {
                player,
                target: PlayerId::new(payload[3]),
                stage: 2,
                card: CardId::NONE,
                value: payload[4],
                skill: SkillId::new(payload[5]),
            }],
        )
        .map(|_| ()),
        _ => Err(EffectError::Illegal("invalid 远谟 selection".into())),
    }
}

pub(crate) fn resolve_refuhai(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::EngineFault("truncated Refuhai payload".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let index = payload[3] as usize;
    let count = payload[4] as usize;
    if payload[0] != crate::effect::REFUHAI_CUSTOM_CHOICE
        || count == 0
        || index >= count
        || payload.len() != 5 + count + index
        || cancelled
        || selections.len() != 1
        || !matches!(selections[0], 1 | 2)
    {
        return Err(EffectError::Illegal("invalid Refuhai choice".into()));
    }
    let participants: Vec<_> = payload[5..5 + count]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    if observer != participants[index]
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("Refuhai context changed".into()));
    }
    let mut choices = payload[5 + count..].to_vec();
    choices.push(selections[0]);
    if index + 1 < count {
        let mut next_payload = vec![
            crate::effect::REFUHAI_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            (index + 1) as u32,
            count as u32,
        ];
        next_payload.extend(participants.iter().map(|target| target.raw()));
        next_payload.extend(choices);
        return crate::effect::open_choice(
            tx,
            participants[index + 1],
            1,
            1,
            false,
            vec![1, 2],
            engine_api::ChoiceKind::Custom,
            ContinuationKind::Custom,
            next_payload,
        )
        .map(|_| ());
    }
    let mut reveal = Vec::with_capacity(count * 2);
    for (participant, choice) in participants.iter().zip(&choices) {
        reveal.extend([participant.raw(), *choice]);
    }
    tx.push_event(engine_api::DomainEvent::RuleNote {
        code: 0x4930_0001,
        payload: reveal,
    });
    let run = choices
        .iter()
        .take_while(|choice| **choice == choices[0])
        .count() as u32;
    if run > 1 {
        run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player,
                count: run,
                skill,
            }],
        )?;
    }
    Ok(())
}

pub(crate) fn resolve_yangwu_gain_hand(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [marker, player_raw, target_raw, skill_raw, count] = payload else {
        return Err(EffectError::EngineFault("bad Yangwu continuation".into()));
    };
    let player = PlayerId::new(*player_raw);
    let target = PlayerId::new(*target_raw);
    let skill = engine_api::SkillId::new(*skill_raw);
    if *marker != crate::effect::YANGWU_GAIN_HAND_CUSTOM_CHOICE
        || observer != player
        || cancelled
        || selections.len() != *count as usize
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("invalid Yangwu hand choice".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("Yangwu target hand missing".into()))?;
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let mut seen = std::collections::BTreeSet::new();
    if cards.iter().any(|card| {
        !seen.insert(card.raw())
            || !tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == target && state.zone == hand)
    }) {
        return Err(EffectError::Illegal(
            "Yangwu requires distinct current target hand cards".into(),
        ));
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Yangwu owner hand missing".into()))?;
    run_effects(
        tx,
        vec![
            EffectRequest::RevealCards {
                cards: cards.clone(),
                to: Vec::new(),
            },
            EffectRequest::GiveCards {
                cards,
                to,
                from: hand,
            },
        ],
    )?;
    Ok(())
}

pub(crate) fn resolve_gzjinfa(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 || payload[0] != crate::effect::GZJINFA_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad Jinfa continuation".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    let option_count = payload[5] as usize;
    if payload.len() != 6 + option_count
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Err(EffectError::Illegal("invalid Jinfa choice context".into()));
    }
    match stage {
        1 => {
            if observer != target || selections.len() > 1 || (cancelled && !selections.is_empty()) {
                return Err(EffectError::Illegal(
                    "invalid Jinfa equipment choice".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                let options = crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                );
                if options.is_empty() {
                    return Ok(());
                }
                let mut next = vec![
                    crate::effect::GZJINFA_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    options.len() as u32,
                ];
                next.extend(options.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    next,
                )
                .map(|_| ());
            }
            let card = CardId::new(selections[0]);
            let from = response_card_zone(tx, target, card)?;
            if !payload[6..].contains(&card.raw())
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.owner == target && state.kind.is_equip())
            {
                return Err(EffectError::Illegal("invalid Jinfa equipment gift".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Jinfa owner hand missing".into()))?;
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
                        from,
                    },
                    EffectRequest::ResolveGzjinfa {
                        player,
                        target,
                        cost_card: CardId::NONE,
                        given_card: card,
                        stage: 2,
                        skill,
                    },
                ],
            )?;
            Ok(())
        }
        2 => {
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Jinfa gain choice".into()));
            }
            let card = CardId::new(selections[0]);
            let allowed = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if !payload[6..].contains(&card.raw()) || !allowed.contains(&card) {
                return Err(EffectError::Illegal("invalid Jinfa gained card".into()));
            }
            let from = response_card_zone(tx, target, card)?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Jinfa owner hand missing".into()))?;
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
                        from,
                    },
                ],
            )?;
            Ok(())
        }
        _ => Err(EffectError::EngineFault("bad Jinfa choice stage".into())),
    }
}
