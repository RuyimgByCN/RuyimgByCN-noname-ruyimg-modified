//! Command operations for `resolve_choose_other_yinghun` and related helpers.

use super::*;

pub(crate) fn resolve_choose_other_yinghun(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Yinghun choice stage".into()))?;
    match stage {
        0 => {
            let [_, player_raw, count, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Yinghun target payload".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            let [target_raw] = selections else {
                return Err(EffectError::Illegal("Yinghun requires one target".into()));
            };
            let target = PlayerId::new(*target_raw);
            if cancelled
                || observer != player
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Yinghun target is no longer legal".into(),
                ));
            }
            let skill = engine_api::SkillId::new(*skill_raw);
            if *count == 1 {
                return run_effects(
                    tx,
                    vec![EffectRequest::ChooseOtherYinghun {
                        player,
                        target,
                        count: *count,
                        branch: 1,
                        ready_to_discard: false,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                engine_api::ChoiceKind::Custom,
                ContinuationKind::ChooseOtherYinghun,
                vec![1, player.raw(), target.raw(), *count, skill.raw()],
            )
            .map(|_| ())
        }
        1 => {
            let [_, player_raw, target_raw, count, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Yinghun branch payload".into(),
                ));
            };
            let [branch] = selections else {
                return Err(EffectError::Illegal("Yinghun requires one branch".into()));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player || !matches!(*branch, 1 | 2) {
                return Err(EffectError::Illegal("invalid Yinghun branch".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ChooseOtherYinghun {
                    player,
                    target: PlayerId::new(*target_raw),
                    count: *count,
                    branch: *branch as u8,
                    ready_to_discard: false,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        2 => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault(
                    "bad Yinghun discard payload".into(),
                ));
            }
            let target = PlayerId::new(payload[2]);
            let required = if payload[4] == 1 { 1 } else { payload[3] } as usize;
            let option_count = payload[6] as usize;
            if cancelled
                || observer != target
                || payload.len() != 7 + option_count
                || selections.len() != required.min(option_count)
            {
                return Err(EffectError::Illegal(
                    "invalid Yinghun discard selection".into(),
                ));
            }
            let original: std::collections::BTreeSet<_> = payload[7..].iter().copied().collect();
            let current: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), target)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            let mut unique = std::collections::BTreeSet::new();
            if !selections.iter().all(|card| {
                unique.insert(*card) && original.contains(card) && current.contains(card)
            }) {
                return Err(EffectError::Illegal(
                    "Yinghun discard cards became stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Yinghun choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_other_bizheng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            let [_, player_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Bizheng target payload".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            let [target_raw] = selections else {
                return Err(EffectError::Illegal("Bizheng requires one target".into()));
            };
            let target = PlayerId::new(*target_raw);
            if cancelled
                || observer != player
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Bizheng target is no longer legal".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ChooseOtherBizheng {
                    player,
                    target,
                    stage: 1,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault(
                    "bad Bizheng discard payload".into(),
                ));
            }
            let discarder = PlayerId::new(payload[3]);
            let option_count = payload[5] as usize;
            let required = option_count.min(2);
            if cancelled
                || observer != discarder
                || payload.len() != 6 + option_count
                || selections.len() != required
            {
                return Err(EffectError::Illegal(
                    "invalid Bizheng discard selection".into(),
                ));
            }
            let original: std::collections::BTreeSet<_> = payload[6..].iter().copied().collect();
            let current: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), discarder)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            let mut unique = std::collections::BTreeSet::new();
            if !selections.iter().all(|card| {
                unique.insert(*card) && original.contains(card) && current.contains(card)
            }) {
                return Err(EffectError::Illegal(
                    "Bizheng discard cards became stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: discarder,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Bizheng choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_other_xiashu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            let [_, player_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault("bad Xiashu target payload".into()));
            };
            let player = PlayerId::new(*player_raw);
            let [target_raw] = selections else {
                return Err(EffectError::Illegal("Xiashu requires one target".into()));
            };
            let target = PlayerId::new(*target_raw);
            if cancelled
                || observer != player
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Xiashu target is no longer legal".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ChooseOtherXiashu {
                    player,
                    target,
                    stage: 1,
                    cards: Vec::new(),
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 5 {
                return Err(EffectError::EngineFault("bad Xiashu reveal payload".into()));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let option_count = payload[4] as usize;
            if cancelled
                || observer != target
                || payload.len() != 5 + option_count
                || selections.is_empty()
                || selections.len() > option_count.min(u8::MAX as usize)
            {
                return Err(EffectError::Illegal(
                    "invalid Xiashu reveal selection".into(),
                ));
            }
            let original: std::collections::BTreeSet<_> = payload[5..].iter().copied().collect();
            let current: std::collections::BTreeSet<_> = tx
                .state()
                .hand_cards(target)
                .into_iter()
                .map(CardId::raw)
                .collect();
            let mut unique = std::collections::BTreeSet::new();
            if !selections.iter().all(|card| {
                unique.insert(*card) && original.contains(card) && current.contains(card)
            }) {
                return Err(EffectError::Illegal(
                    "Xiashu revealed cards became stale".into(),
                ));
            }
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            run_effects(
                tx,
                vec![
                    EffectRequest::RevealCards {
                        cards: cards.clone(),
                        to: Vec::new(),
                    },
                    EffectRequest::ChooseOtherXiashu {
                        player,
                        target,
                        stage: 3,
                        cards,
                        skill: engine_api::SkillId::new(payload[3]),
                    },
                ],
            )
            .map(|_| ())
        }
        Some(2) => {
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "bad Xiashu partition payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let [branch] = selections else {
                return Err(EffectError::Illegal("Xiashu requires one partition".into()));
            };
            let revealed_count = payload[4] as usize;
            if cancelled
                || observer != player
                || payload.len() != 5 + revealed_count
                || !matches!(*branch, 1 | 2)
            {
                return Err(EffectError::Illegal(
                    "invalid Xiashu partition choice".into(),
                ));
            }
            let revealed: std::collections::BTreeSet<_> = payload[5..].iter().copied().collect();
            let cards: Vec<_> = tx
                .state()
                .hand_cards(target)
                .into_iter()
                .filter(|card| revealed.contains(&card.raw()) == (*branch == 1))
                .collect();
            if cards.is_empty() {
                return Ok(());
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("Xiashu target hand missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Xiashu owner hand missing".into()))?;
            run_effects(tx, vec![EffectRequest::GiveCards { cards, to, from }]).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Xiashu choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_guanwei(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 {
        return Err(EffectError::EngineFault("bad Guanwei payload".into()));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let option_count = payload[3] as usize;
    let [card_raw] = selections else {
        return Err(EffectError::Illegal(
            "Guanwei requires one discard card".into(),
        ));
    };
    if cancelled || observer != player || payload.len() != 4 + option_count {
        return Err(EffectError::Illegal("invalid Guanwei selection".into()));
    }
    let original: std::collections::BTreeSet<_> = payload[4..].iter().copied().collect();
    let current: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(CardId::raw)
            .collect();
    if !original.contains(card_raw) || !current.contains(card_raw) {
        return Err(EffectError::Illegal(
            "Guanwei discard card became stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveGuanwei {
            player,
            target,
            stage: 1,
            card: CardId::new(*card_raw),
            skill: engine_api::SkillId::new(payload[2]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_lanjiang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "bad Lanjiang confirm payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let index = payload[2];
            let skill = engine_api::SkillId::new(payload[3]);
            let count = payload[4] as usize;
            if payload.len() != 5 + count || index as usize >= count {
                return Err(EffectError::EngineFault(
                    "bad Lanjiang target snapshot".into(),
                ));
            }
            let targets: Vec<_> = payload[5..].iter().copied().map(PlayerId::new).collect();
            if cancelled || observer != targets[index as usize] || !matches!(selections, [0] | [1])
            {
                return Err(EffectError::Illegal(
                    "invalid Lanjiang draw confirmation".into(),
                ));
            }
            let next = EffectRequest::ResolveLanjiang {
                player,
                stage: 1,
                index: index.saturating_add(1),
                target: PlayerId::NONE,
                targets,
                skill,
            };
            if selections == [0] {
                return run_effects(tx, vec![next]).map(|_| ());
            }
            tx.state_mut().resolution.deferred_effects.insert(0, next);
            run_effects(
                tx,
                vec![EffectRequest::DrawCardsFromSkill {
                    player,
                    count: 1,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 4 {
                return Err(EffectError::EngineFault(
                    "bad Lanjiang damage payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = engine_api::SkillId::new(payload[2]);
            let count = payload[3] as usize;
            if observer != player || payload.len() != 4 + count {
                return Err(EffectError::Illegal(
                    "invalid Lanjiang damage choice".into(),
                ));
            }
            let targets: Vec<_> = payload[4..].iter().copied().map(PlayerId::new).collect();
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled Lanjiang damage chose a target".into(),
                    ));
                }
                return Ok(());
            }
            let [target_raw] = selections else {
                return Err(EffectError::Illegal(
                    "Lanjiang damage requires one target".into(),
                ));
            };
            let target = PlayerId::new(*target_raw);
            if !targets.contains(&target)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_count(target) != tx.state().hand_count(player)
            {
                return Err(EffectError::Illegal(
                    "Lanjiang damage target became illegal".into(),
                ));
            }
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveLanjiang {
                    player,
                    stage: 3,
                    index: 0,
                    target: PlayerId::NONE,
                    targets,
                    skill,
                },
            );
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
            .map(|_| ())
        }
        Some(2) => {
            let [_, player_raw, skill_raw, targets @ ..] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Lanjiang final draw payload".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            let target_ids: Vec<_> = targets.iter().copied().map(PlayerId::new).collect();
            let [target_raw] = selections else {
                return Err(EffectError::Illegal(
                    "Lanjiang final draw requires one target".into(),
                ));
            };
            let target = PlayerId::new(*target_raw);
            if cancelled
                || observer != player
                || !target_ids.contains(&target)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_count(target) >= tx.state().hand_count(player)
            {
                return Err(EffectError::Illegal(
                    "Lanjiang final draw target became illegal".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DrawCardsFromSkill {
                    player: target,
                    count: 1,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Lanjiang choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_rebingyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, max, self_draw, skill_raw] = payload else {
        return Err(EffectError::EngineFault("bad Rebingyi payload".into()));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player || selections.len() > *max as usize {
        return Err(EffectError::Illegal("invalid Rebingyi target count".into()));
    }
    if cancelled && !selections.is_empty() {
        return Err(EffectError::Illegal(
            "cancelled Rebingyi chose targets".into(),
        ));
    }
    let mut unique = std::collections::BTreeSet::new();
    if !selections.iter().all(|target| {
        unique.insert(*target)
            && tx
                .state()
                .players
                .get(PlayerId::new(*target))
                .is_some_and(|participant| participant.alive)
    }) {
        return Err(EffectError::Illegal(
            "Rebingyi target became illegal".into(),
        ));
    }
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    let skill = engine_api::SkillId::new(*skill_raw);
    let mut effects: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| selected.contains(&target.raw()))
        .map(|target| EffectRequest::DrawCardsFromSkill {
            player: target,
            count: 1,
            skill,
        })
        .collect();
    if *self_draw != 0 {
        effects.push(EffectRequest::DrawCardsFromSkill {
            player,
            count: 1,
            skill,
        });
    }
    run_effects(tx, effects).map(|_| ())
}
