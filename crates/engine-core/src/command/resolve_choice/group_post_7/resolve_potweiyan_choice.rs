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
