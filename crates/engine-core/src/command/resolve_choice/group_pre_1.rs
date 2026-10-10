//! Choice-continuation handlers (pre 1).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

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

pub(crate) fn resolve_oldimeng(
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
                    "bad Oldimeng first-target payload".into(),
                ));
            };
            let [first_raw] = selections else {
                return Err(EffectError::Illegal(
                    "Oldimeng requires a first target".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player {
                return Err(EffectError::Illegal("invalid Oldimeng first target".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveOldimeng {
                    player,
                    stage: 1,
                    first: PlayerId::new(*first_raw),
                    second: PlayerId::NONE,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            let [_, player_raw, first_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Oldimeng second-target payload".into(),
                ));
            };
            let [second_raw] = selections else {
                return Err(EffectError::Illegal(
                    "Oldimeng requires a second target".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player {
                return Err(EffectError::Illegal(
                    "invalid Oldimeng second target".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveOldimeng {
                    player,
                    stage: 1,
                    first: PlayerId::new(*first_raw),
                    second: PlayerId::new(*second_raw),
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        Some(2) => {
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "bad Oldimeng discard payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let option_count = payload[3] as usize;
            let required = payload[4] as usize;
            if cancelled
                || observer != player
                || payload.len() != 5 + option_count
                || selections.len() != required
            {
                return Err(EffectError::Illegal(
                    "invalid Oldimeng discard count".into(),
                ));
            }
            let original: std::collections::BTreeSet<_> = payload[5..].iter().copied().collect();
            let current: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            let mut unique = std::collections::BTreeSet::new();
            if !selections.iter().all(|card| {
                unique.insert(*card) && original.contains(card) && current.contains(card)
            }) {
                return Err(EffectError::Illegal(
                    "Oldimeng discard cards became stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Oldimeng choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_rejieyin(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Rejieyin choice requires one selection".into(),
        ));
    }
    match payload.first().copied() {
        Some(0) => {
            let [_, player_raw, card_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Rejieyin target payload".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if observer != player {
                return Err(EffectError::Illegal("wrong Rejieyin chooser".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveRejieyin {
                    player,
                    target: PlayerId::new(selections[0]),
                    stage: 1,
                    card: CardId::new(*card_raw),
                    equip: false,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            let [_, player_raw, target_raw, card_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Rejieyin branch payload".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if observer != player || !matches!(selections, [0] | [1]) {
                return Err(EffectError::Illegal(
                    "invalid Rejieyin placement branch".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveRejieyin {
                    player,
                    target: PlayerId::new(*target_raw),
                    stage: 2,
                    card: CardId::new(*card_raw),
                    equip: selections[0] == 0,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Rejieyin choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_reanxu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let Some(stage) = payload.first().copied() else {
        return Err(EffectError::EngineFault("bad Reanxu payload".into()));
    };
    match stage {
        0 => {
            let [_, player_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Reanxu first-target payload".into(),
                ));
            };
            let [first] = selections else {
                return Err(EffectError::Illegal(
                    "Reanxu requires one first target".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player {
                return Err(EffectError::Illegal("invalid Reanxu first target".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveReanxu {
                    player,
                    stage: 1,
                    first: PlayerId::new(*first),
                    second: PlayerId::NONE,
                    card: CardId::NONE,
                    from_hand: false,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        1 => {
            let [_, player_raw, first_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Reanxu second-target payload".into(),
                ));
            };
            let [second] = selections else {
                return Err(EffectError::Illegal(
                    "Reanxu requires one second target".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player {
                return Err(EffectError::Illegal("invalid Reanxu second target".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveReanxu {
                    player,
                    stage: 1,
                    first: PlayerId::new(*first_raw),
                    second: PlayerId::new(*second),
                    card: CardId::NONE,
                    from_hand: false,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        2 => {
            let [_, player_raw, first_raw, second_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault("bad Reanxu card payload".into()));
            };
            let [card] = selections else {
                return Err(EffectError::Illegal("Reanxu requires one card".into()));
            };
            let second = PlayerId::new(*second_raw);
            if cancelled || observer != second {
                return Err(EffectError::Illegal("invalid Reanxu card choice".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveReanxu {
                    player: PlayerId::new(*player_raw),
                    stage: 2,
                    first: PlayerId::new(*first_raw),
                    second,
                    card: CardId::new(*card),
                    from_hand: false,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        3 => {
            let [_, player_raw, lower_raw, _skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Reanxu optional-draw payload".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player || !matches!(selections, [0] | [1]) {
                return Err(EffectError::Illegal("invalid Reanxu optional draw".into()));
            }
            if selections[0] == 0 {
                return Ok(());
            }
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player: PlayerId::new(*lower_raw),
                    count: 1,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Reanxu choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_oldanxu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let Some(stage) = payload.first().copied() else {
        return Err(EffectError::EngineFault("bad old Anxu payload".into()));
    };
    match stage {
        0 => {
            let [_, player_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad old Anxu giver payload".into(),
                ));
            };
            let [giver] = selections else {
                return Err(EffectError::Illegal("old Anxu requires one giver".into()));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player {
                return Err(EffectError::Illegal("invalid old Anxu giver".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveOldAnxu {
                    player,
                    stage: 1,
                    giver: PlayerId::new(*giver),
                    receiver: PlayerId::NONE,
                    card: CardId::NONE,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        1 => {
            let [_, player_raw, giver_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault(
                    "bad old Anxu receiver payload".into(),
                ));
            };
            let [receiver] = selections else {
                return Err(EffectError::Illegal(
                    "old Anxu requires one receiver".into(),
                ));
            };
            let player = PlayerId::new(*player_raw);
            if cancelled || observer != player {
                return Err(EffectError::Illegal("invalid old Anxu receiver".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveOldAnxu {
                    player,
                    stage: 2,
                    giver: PlayerId::new(*giver_raw),
                    receiver: PlayerId::new(*receiver),
                    card: CardId::NONE,
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        2 => {
            let [_, player_raw, giver_raw, receiver_raw, skill_raw] = payload else {
                return Err(EffectError::EngineFault("bad old Anxu card payload".into()));
            };
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "old Anxu requires one hand card".into(),
                ));
            };
            let receiver = PlayerId::new(*receiver_raw);
            if cancelled || observer != receiver {
                return Err(EffectError::Illegal("invalid old Anxu card choice".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveOldAnxu {
                    player: PlayerId::new(*player_raw),
                    stage: 3,
                    giver: PlayerId::new(*giver_raw),
                    receiver,
                    card: CardId::new(*card),
                    skill: engine_api::SkillId::new(*skill_raw),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown old Anxu choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_oldqinqing(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, lord_raw, skill_raw] = payload else {
        return Err(EffectError::EngineFault(
            "bad old Qinqing target payload".into(),
        ));
    };
    let [target] = selections else {
        return Err(EffectError::Illegal(
            "old Qinqing requires one target".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if cancelled || observer != player {
        return Err(EffectError::Illegal("invalid old Qinqing target".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveOldQinqing {
            player,
            target: PlayerId::new(*target),
            lord: PlayerId::new(*lord_raw),
            stage: 1,
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_oldmingjian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, skill_raw] = payload else {
        return Err(EffectError::EngineFault(
            "bad old Mingjian target payload".into(),
        ));
    };
    let [target] = selections else {
        return Err(EffectError::Illegal(
            "old Mingjian requires one target".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if cancelled || observer != player {
        return Err(EffectError::Illegal("invalid old Mingjian target".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveOldMingjian {
            player,
            target: PlayerId::new(*target),
            stage: 1,
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_oldxuanfeng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled {
        return Err(EffectError::Illegal(
            "old Xuanfeng choice cannot be cancelled".into(),
        ));
    }
    match payload {
        [0, player, card, skill] => {
            let [target] = selections else {
                return Err(EffectError::Illegal(
                    "old Xuanfeng requires one target".into(),
                ));
            };
            let player = PlayerId::new(*player);
            if observer != player {
                return Err(EffectError::Illegal("invalid old Xuanfeng chooser".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveOldXuanfeng {
                    player,
                    target: PlayerId::new(*target),
                    card: CardId::new(*card),
                    stage: 1,
                    branch: 0,
                    skill: engine_api::SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        [1, player, target, card, skill] => {
            let [branch @ (1 | 2)] = selections else {
                return Err(EffectError::Illegal(
                    "old Xuanfeng requires one branch".into(),
                ));
            };
            let player = PlayerId::new(*player);
            if observer != player {
                return Err(EffectError::Illegal("invalid old Xuanfeng chooser".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveOldXuanfeng {
                    player,
                    target: PlayerId::new(*target),
                    card: CardId::new(*card),
                    stage: 2,
                    branch: *branch as u8,
                    skill: engine_api::SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad old Xuanfeng choice payload".into(),
        )),
    }
}

pub(crate) fn resolve_jianrou(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, damage_id, skill] = payload else {
        return Err(EffectError::EngineFault("bad Jianrou payload".into()));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if cancelled || observer != player || selections.len() != 2 || selections[0] == selections[1] {
        return Err(EffectError::Illegal(
            "Jianrou requires two distinct cards".into(),
        ));
    }
    let cards = selections
        .iter()
        .copied()
        .map(CardId::new)
        .collect::<Vec<_>>();
    let legal = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if cards.iter().any(|card| !legal.contains(card)) {
        return Err(EffectError::Illegal(
            "Jianrou discard selection became illegal".into(),
        ));
    }
    let attributes = cards
        .iter()
        .map(|card| {
            let card_state = tx.state().cards.get(*card).unwrap();
            let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
            (
                crate::query::effective_card_color_mask(tx.state(), player, *card),
                if kind.is_basic() {
                    1
                } else if kind.is_trick() {
                    2
                } else if kind.is_equip() {
                    3
                } else {
                    0
                },
            )
        })
        .collect::<Vec<_>>();
    let refund = attributes[0].0 == attributes[1].0 || attributes[0].1 == attributes[1].1;
    if refund {
        tx.state_mut()
            .skills
            .get_mut(skill)
            .ok_or_else(|| EffectError::EngineFault("Jianrou skill disappeared".into()))?
            .round_fire_round = 0;
    }
    let mut effects = vec![
        EffectRequest::DiscardCards { player, cards },
        EffectRequest::PreventDamage {
            damage_id: *damage_id,
        },
    ];
    if refund {
        effects.push(EffectRequest::DrawCardsFromSkill {
            player,
            count: 1,
            skill,
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_oldhuisheng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload {
        [0, player, source, damage_id, skill] => {
            let player = PlayerId::new(*player);
            if cancelled || observer != player || selections.is_empty() {
                return Err(EffectError::Illegal(
                    "old Huisheng requires at least one owner card".into(),
                ));
            }
            let legal = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let mut distinct = std::collections::BTreeSet::new();
            let cards = selections
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>();
            if cards
                .iter()
                .any(|card| !distinct.insert(*card) || !legal.contains(card))
            {
                return Err(EffectError::Illegal(
                    "old Huisheng owner cards became illegal".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveOldHuisheng {
                    player,
                    source: PlayerId::new(*source),
                    damage_id: *damage_id,
                    stage: 1,
                    cards,
                    skill: engine_api::SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        [1, player, source, damage_id, _skill, count, cards @ ..] => {
            let player = PlayerId::new(*player);
            let source = PlayerId::new(*source);
            if observer != source || cards.len() != *count as usize {
                return Err(EffectError::Illegal(
                    "invalid old Huisheng source choice".into(),
                ));
            }
            if !cancelled {
                let [selected] = selections else {
                    return Err(EffectError::Illegal(
                        "old Huisheng source must gain one card".into(),
                    ));
                };
                if !cards.contains(selected) {
                    return Err(EffectError::Illegal(
                        "old Huisheng selected card is not revealed".into(),
                    ));
                }
                let card = CardId::new(*selected);
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .filter(|card| card.owner == player)
                    .map(|card| card.zone)
                    .ok_or_else(|| {
                        EffectError::Illegal("old Huisheng selected card moved".into())
                    })?;
                let to = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::GiveCards {
                            cards: vec![card],
                            to,
                            from,
                        },
                        EffectRequest::PreventDamage {
                            damage_id: *damage_id,
                        },
                    ],
                )
                .map(|_| ());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), source);
            if options.len() < *count as usize {
                return Err(EffectError::Illegal(
                    "old Huisheng discard branch is no longer payable".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                source,
                *count as u8,
                *count as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ResolveOldHuisheng,
                vec![2, source.raw(), *count],
            )
            .map(|_| ())
        }
        [2, source, count] => {
            let source = PlayerId::new(*source);
            if cancelled || observer != source || selections.len() != *count as usize {
                return Err(EffectError::Illegal(
                    "old Huisheng requires the exact discard count".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: source,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad old Huisheng choice payload".into(),
        )),
    }
}

pub(crate) fn resolve_oldjiefan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, dying, current, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad old Jiefan choice payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let [card] = selections else {
        return Err(EffectError::Illegal(
            "old Jiefan requires exactly one Sha source".into(),
        ));
    };
    if cancelled || observer != player {
        return Err(EffectError::Illegal(
            "invalid old Jiefan Sha source choice".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveOldJiefan {
            player,
            dying: PlayerId::new(*dying),
            current: PlayerId::new(*current),
            card: CardId::new(*card),
            damage_id: 0,
            stage: 1,
            skill: engine_api::SkillId::new(*skill),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_chegu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Chegu choice stage".into()))?;
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    if cancelled || observer != player {
        return Err(EffectError::Illegal("invalid Chegu choice actor".into()));
    }
    match stage {
        0 => {
            let [branch] = selections else {
                return Err(EffectError::Illegal(
                    "Chegu requires exactly one branch".into(),
                ));
            };
            if !matches!(branch, 1 | 2) {
                return Err(EffectError::Illegal("invalid Chegu branch".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveChegu {
                    player,
                    card: CardId::new(*payload.get(2).unwrap_or(&0)),
                    targets: Vec::new(),
                    cards: Vec::new(),
                    branch: *branch as u8,
                    index: 0,
                    stage: 1,
                    skill: engine_api::SkillId::new(*payload.get(3).unwrap_or(&0)),
                }],
            )
            .map(|_| ())
        }
        1 => {
            if selections.is_empty() {
                return Err(EffectError::Illegal(
                    "Chegu requires at least one discard target".into(),
                ));
            }
            let mut distinct = std::collections::BTreeSet::new();
            let targets = selections
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            if targets.iter().any(|target| {
                !distinct.insert(*target)
                    || crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        *target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
            }) {
                return Err(EffectError::Illegal(
                    "Chegu discard target became illegal".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveChegu {
                    player,
                    card: CardId::new(*payload.get(2).unwrap_or(&0)),
                    targets,
                    cards: Vec::new(),
                    branch: *payload.get(3).unwrap_or(&0) as u8,
                    index: 0,
                    stage: 2,
                    skill: engine_api::SkillId::new(*payload.get(4).unwrap_or(&0)),
                }],
            )
            .map(|_| ())
        }
        2 => {
            let branch = *payload.get(3).unwrap_or(&0) as u8;
            let index = *payload.get(4).unwrap_or(&0) as u8;
            let skill = engine_api::SkillId::new(*payload.get(5).unwrap_or(&0));
            let target_len = *payload.get(6).unwrap_or(&u32::MAX) as usize;
            let targets_end = 7usize.saturating_add(target_len);
            let targets = payload
                .get(7..targets_end)
                .ok_or_else(|| EffectError::EngineFault("truncated Chegu targets".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            let discarded_len = *payload
                .get(targets_end)
                .ok_or_else(|| EffectError::EngineFault("missing Chegu discard count".into()))?
                as usize;
            let discarded = payload
                .get(targets_end + 1..targets_end + 1 + discarded_len)
                .ok_or_else(|| EffectError::EngineFault("truncated Chegu discards".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>();
            let target = *targets
                .get(usize::from(index))
                .ok_or_else(|| EffectError::EngineFault("bad Chegu target index".into()))?;
            let legal = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let mut distinct = std::collections::BTreeSet::new();
            let selected = selections
                .iter()
                .copied()
                .map(CardId::new)
                .collect::<Vec<_>>();
            if selected.is_empty()
                || selected
                    .iter()
                    .any(|card| !distinct.insert(*card) || !legal.contains(card))
            {
                return Err(EffectError::Illegal(
                    "Chegu discard cards became illegal".into(),
                ));
            }
            let mut cards = discarded;
            cards.extend(selected.iter().copied());
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardTargetCards {
                        actor: player,
                        target,
                        cards: selected,
                        scope: rule_api::TargetCardScope::HandEquip,
                    },
                    EffectRequest::ResolveChegu {
                        player,
                        card: CardId::new(*payload.get(2).unwrap_or(&0)),
                        targets,
                        cards,
                        branch,
                        index: index.saturating_add(1),
                        stage: 2,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Chegu choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_junkguixin(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Junkguixin choice stage".into()))?;
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Junkguixin requires exactly one choice".into(),
        ));
    };
    if cancelled || observer != player {
        return Err(EffectError::Illegal(
            "invalid Junkguixin choice actor".into(),
        ));
    }
    let skill = engine_api::SkillId::new(*payload.last().unwrap_or(&0));
    let effects = match stage {
        0 if matches!(selected, 1 | 2) => vec![EffectRequest::ResolveJunkguixin {
            player,
            target: PlayerId::NONE,
            stage: *selected as u8,
            group: 0,
            skill,
        }],
        1 => {
            let selected = engine_api::SkillDefId::new(*selected);
            if !crate::query::gainable_lord_skill_defs(tx.state(), player).contains(&selected) {
                return Err(EffectError::Illegal(
                    "selected Junkguixin lord skill became illegal".into(),
                ));
            }
            vec![EffectRequest::AddSkill {
                player,
                skill_def: selected,
            }]
        }
        2 => {
            let target = PlayerId::new(*selected);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|target_state| target_state.alive && target != player)
            {
                return Err(EffectError::Illegal(
                    "selected Junkguixin target became illegal".into(),
                ));
            }
            vec![EffectRequest::ResolveJunkguixin {
                player,
                target,
                stage: 3,
                group: 0,
                skill,
            }]
        }
        3 if (1..=8).contains(selected) => vec![EffectRequest::ResolveJunkguixin {
            player,
            target: PlayerId::new(*payload.get(2).unwrap_or(&0)),
            stage: 4,
            group: *selected as u8,
            skill,
        }],
        _ => return Err(EffectError::Illegal("invalid Junkguixin choice".into())),
    };
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_guhuo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled {
        return Err(EffectError::Illegal("Guhuo choice cannot cancel".into()));
    }
    match payload {
        [0, player, skill] => {
            let player = PlayerId::new(*player);
            let skill = engine_api::SkillId::new(*skill);
            let [source] = selections else {
                return Err(EffectError::Illegal(
                    "Guhuo requires exactly one hidden source".into(),
                ));
            };
            let source = CardId::new(*source);
            if observer != player || !tx.state().hand_cards(player).contains(&source) {
                return Err(EffectError::Illegal(
                    "Guhuo hidden source became illegal".into(),
                ));
            }
            let mut challengers = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player
                        && tx
                            .state()
                            .players
                            .get(*candidate)
                            .is_some_and(|candidate| candidate.hp != 0)
                })
                .collect::<Vec<_>>();
            if let Some(current) = challengers
                .iter()
                .position(|candidate| *candidate == tx.state().turn.current)
            {
                challengers.rotate_left(current);
            }
            if challengers.is_empty() {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .ok_or_else(|| EffectError::EngineFault("Guhuo skill disappeared".into()))?
                    .set_source_free_physical_card(source);
                return Ok(());
            }
            open_guhuo_challenge(tx, player, skill, source, 0, &challengers, &[])
        }
        [1, player, skill, source, index, challenger_len, rest @ ..] => {
            let player = PlayerId::new(*player);
            let skill = engine_api::SkillId::new(*skill);
            let source = CardId::new(*source);
            let challenger_len = *challenger_len as usize;
            let challengers = rest
                .get(..challenger_len)
                .ok_or_else(|| EffectError::EngineFault("truncated Guhuo challengers".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            let betrayed_len = *rest
                .get(challenger_len)
                .ok_or_else(|| EffectError::EngineFault("missing Guhuo challenge count".into()))?
                as usize;
            let mut betrayed = rest
                .get(challenger_len + 1..challenger_len + 1 + betrayed_len)
                .ok_or_else(|| EffectError::EngineFault("truncated Guhuo challenges".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            let index = *index as usize;
            let challenger = *challengers
                .get(index)
                .ok_or_else(|| EffectError::EngineFault("bad Guhuo challenger index".into()))?;
            let [decision @ (0 | 1)] = selections else {
                return Err(EffectError::Illegal(
                    "Guhuo challenge requires ally or challenge".into(),
                ));
            };
            if observer != challenger {
                return Err(EffectError::Illegal(
                    "Guhuo challenge answered by the wrong player".into(),
                ));
            }
            if *decision == 1 {
                betrayed.push(challenger);
            }
            if index + 1 < challengers.len() {
                return open_guhuo_challenge(
                    tx,
                    player,
                    skill,
                    source,
                    index + 1,
                    &challengers,
                    &betrayed,
                );
            }
            settle_guhuo(tx, player, skill, source, &betrayed)
        }
        _ => Err(EffectError::EngineFault(
            "bad Guhuo continuation payload".into(),
        )),
    }
}

pub(crate) fn resolve_face_down_view_as(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill, mode @ ..] = payload else {
        return Err(EffectError::EngineFault(
            "bad face-down view-as payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if mode.first() == Some(&5) {
        let kind = mode
            .get(1)
            .and_then(|tag| CardKind::from_tag(*tag as u8))
            .ok_or_else(|| EffectError::EngineFault("missing 腹笥 card declaration".into()))?;
        let [source] = selections else {
            return Err(EffectError::Illegal("腹笥须选择一张明置手牌".into()));
        };
        let source = CardId::new(*source);
        let legal = crate::command::skill::dddfusi_sources(tx.state(), player, skill, kind);
        if cancelled || observer != player || !legal.contains(&source) {
            return Err(EffectError::Illegal("腹笥明置牌已不可用".into()));
        }
        let provider = tx
            .state()
            .cards
            .get(source)
            .map(|card| card.owner)
            .ok_or_else(|| EffectError::Illegal("腹笥明置牌已消失".into()))?;
        if crate::command::skill::dddfusi_policy(tx.state(), skill, provider) == 1 {
            crate::command::skill::set_dddfusi_approval(tx.state_mut(), skill, source, provider, None)?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("腹笥借牌技能已消失".into()))?
                .set_source_free_physical_card(source);
            return Ok(());
        }
        crate::effect::open_choice(
            tx,
            provider,
            1,
            1,
            false,
            vec![0, 1, 2, 3],
            engine_api::ChoiceKind::Custom,
            ContinuationKind::ResolveFaceDownViewAs,
            vec![
                player.raw(),
                skill.raw(),
                6,
                kind.tag() as u32,
                source.raw(),
            ],
        )?;
        return Ok(());
    }
    if mode.first() == Some(&6) {
        let source = CardId::new(*mode.get(2).unwrap_or(&0));
        let provider = tx
            .state()
            .cards
            .get(source)
            .map(|card| card.owner)
            .ok_or_else(|| EffectError::Illegal("腹笥明置牌已消失".into()))?;
        let [decision] = selections else {
            return Err(EffectError::Illegal("腹笥牌主须确认".into()));
        };
        if cancelled || observer != provider || !matches!(*decision, 0..=3) {
            return Err(EffectError::Illegal("腹笥确认已失效".into()));
        }
        let policy = match *decision {
            2 => Some(1),
            3 => Some(2),
            _ => None,
        };
        if matches!(*decision, 0 | 2) {
            crate::command::skill::set_dddfusi_approval(tx.state_mut(), skill, source, provider, policy)?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("腹笥借牌技能已消失".into()))?
                .set_source_free_physical_card(source);
        } else {
            if policy.is_some() {
                crate::command::skill::set_dddfusi_approval(
                    tx.state_mut(),
                    skill,
                    CardId::NONE,
                    provider,
                    policy,
                )?;
            }
            tx.state_mut().resolution.resume_kind = ContinuationKind::None;
            tx.state_mut().resolution.resume_payload.clear();
        }
        return Ok(());
    }
    if mode == [4] {
        let legal = crate::command::skill::dddshilie_sources(tx.state(), player);
        let selected: Vec<_> = selections.iter().copied().map(CardId::new).collect();
        let unique: std::collections::BTreeSet<_> = selected.iter().copied().collect();
        let threshold = crate::command::skill::dddshilie_threshold(tx.state(), player).ok_or_else(|| {
            EffectError::Illegal("示烈 is unavailable outside a live turn".into())
        })?;
        let total = selected.iter().try_fold(0u16, |total, card| {
            tx.state().cards.get(*card).map(|card| {
                total.saturating_add(u16::from(crate::query::effective_card_number(
                    tx.state(),
                    player,
                    card,
                )))
            })
        });
        if cancelled
            || observer != player
            || selected.is_empty()
            || unique.len() != selected.len()
            || selected.iter().any(|card| !legal.contains(card))
            || total.is_none_or(|total| total < threshold)
            || !tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            })
        {
            return Err(EffectError::Illegal("invalid 示烈 shown cards".into()));
        }
        run_effects(
            tx,
            vec![EffectRequest::RevealCards {
                cards: selected.clone(),
                to: Vec::new(),
            }],
        )?;
        for card in &selected {
            tx.state_mut()
                .cards
                .get_mut(*card)
                .expect("validated 示烈 card")
                .face_up = true;
        }
        if total == Some(threshold) {
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player,
                    count: selected.len().min(u32::MAX as usize) as u32,
                }],
            )?;
        }
        return Ok(());
    }
    let [source] = selections else {
        return Err(EffectError::Illegal(
            "face-down view-as requires one source card".into(),
        ));
    };
    let source = CardId::new(*source);
    let hand_equip_mark = mode == [1];
    let potdaozhuan = mode.first() == Some(&2);
    let dczhaowen = mode == [3];
    let legal = if potdaozhuan {
        crate::command::skill::potdaozhuan_cost_cards(tx.state(), player)
    } else if hand_equip_mark {
        crate::command::skill::hand_equip_view_as_sources(tx.state(), player)
    } else if dczhaowen {
        tx.state().skills.get(skill).map_or_else(Vec::new, |skill| {
            crate::command::skill::dczhaowen_view_as_sources(tx.state(), skill)
        })
    } else {
        crate::command::skill::face_down_view_as_sources(tx.state(), player)
    };
    if cancelled || observer != player || !legal.contains(&source) {
        return Err(EffectError::Illegal(
            "face-down view-as source became illegal".into(),
        ));
    }
    if potdaozhuan {
        let kind = mode
            .get(1)
            .and_then(|tag| CardKind::from_tag(*tag as u8))
            .ok_or_else(|| EffectError::EngineFault("missing 道转 declaration".into()))?;
        let current = tx.state().turn.current;
        let source_owner = tx
            .state()
            .cards
            .get(source)
            .map(|card| card.owner)
            .ok_or_else(|| EffectError::Illegal("道转 cost card disappeared".into()))?;
        let name_tag = if kind.is_sha() {
            CardKind::Sha.tag()
        } else {
            kind.tag()
        };
        let name_bit = (name_tag < 32)
            .then_some(1u32 << name_tag)
            .ok_or_else(|| EffectError::Illegal("unsupported 道转 basic name".into()))?;
        let round = tx.state().turn.round;
        tx.state_mut()
            .skills
            .get_mut(skill)
            .ok_or_else(|| EffectError::EngineFault("道转 skill disappeared".into()))?
            .record_potdaozhuan(
                round,
                current,
                name_bit,
                source_owner == current && current != player,
            );
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
        let from = tx
            .state()
            .cards
            .get(source)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("道转 cost card disappeared".into()))?;
        run_effects(
            tx,
            vec![EffectRequest::MoveCardsWithReason {
                cards: vec![source],
                to: discard,
                from: Some(from),
                reason: CardMoveReason::Discard,
            }],
        )?;
        return Ok(());
    }
    if !hand_equip_mark {
        let kind = tx
            .state()
            .resolution
            .resume_payload
            .get(2)
            .and_then(|kind| CardKind::from_tag(*kind as u8))
            .ok_or_else(|| EffectError::EngineFault("missing face-down declaration".into()))?;
        crate::command::skill::record_face_down_view_as_kind(tx.state_mut(), skill, kind)?;
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("face-down view-as skill disappeared".into()))?
        .set_source_free_physical_card(source);
    Ok(())
}

pub(crate) fn resolve_swap_lower_hand_view_as(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad swap-lower-hand view-as payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = SkillId::new(*skill);
    let [target] = selections else {
        return Err(EffectError::Illegal(
            "swap-lower-hand view-as requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target);
    if cancelled
        || observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !crate::command::skill::swap_lower_hand_targets(tx.state(), player).contains(&target)
    {
        return Err(EffectError::Illegal(
            "swap-lower-hand view-as target became illegal".into(),
        ));
    }
    let player_cards = tx.state().hand_cards(player);
    let target_cards = tx.state().hand_cards(target);
    let player_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("source hand missing".into()))?;
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("target hand missing".into()))?;
    crate::effect::move_cards(tx, player_cards, target_hand, Some(player_hand))?;
    crate::effect::move_cards(tx, target_cards, player_hand, Some(target_hand))?;
    Ok(())
}

pub(crate) fn resolve_yao_chenwei(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, third, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Yao Chenwei continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Yao Chenwei requires exactly one selection".into(),
        ));
    };
    if cancelled {
        return Err(EffectError::Illegal(
            "Yao Chenwei choice cannot cancel".into(),
        ));
    }
    let effect = match stage {
        0 if observer == player && matches!(third, 0 | 1) => EffectRequest::ResolveYaoChenwei {
            player,
            target: PlayerId::new(*selected),
            card: CardId::NONE,
            mode: *third as u8,
            stage: 1,
            skill,
        },
        1 => {
            let target = PlayerId::new(*third);
            if observer != target {
                return Err(EffectError::Illegal(
                    "Yao Chenwei card must be chosen by its target".into(),
                ));
            }
            EffectRequest::ResolveYaoChenwei {
                player,
                target,
                card: CardId::new(*selected),
                mode: 0,
                stage: 2,
                skill,
            }
        }
        2 if observer == player => EffectRequest::ResolveYaoChenwei {
            player,
            target: PlayerId::new(*third),
            card: CardId::new(*selected),
            mode: 1,
            stage: 2,
            skill,
        },
        _ => {
            return Err(EffectError::Illegal(
                "invalid Yao Chenwei choice actor or stage".into(),
            ));
        }
    };
    run_effects(tx, vec![effect]).map(|_| ())
}

pub(crate) fn resolve_youdi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, target, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Youdi continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let target = PlayerId::new(*target);
    let skill = engine_api::SkillId::new(*skill);
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Youdi requires exactly one selection".into(),
        ));
    };
    if cancelled {
        return Err(EffectError::Illegal("Youdi choice cannot cancel".into()));
    }
    let effect = match stage {
        0 if observer == player => EffectRequest::ResolveYoudi {
            player,
            target: PlayerId::new(*selected),
            card: CardId::NONE,
            stage: 1,
            skill,
        },
        1 if observer == target => EffectRequest::ResolveYoudi {
            player,
            target,
            card: CardId::new(*selected),
            stage: 2,
            skill,
        },
        2 if observer == player => EffectRequest::ResolveYoudi {
            player,
            target,
            card: CardId::new(*selected),
            stage: 4,
            skill,
        },
        _ => {
            return Err(EffectError::Illegal(
                "invalid Youdi choice actor or stage".into(),
            ));
        }
    };
    run_effects(tx, vec![effect]).map(|_| ())
}

pub(crate) fn resolve_kuangfu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Kuangfu requires exactly one selection".into(),
        ));
    };
    if cancelled {
        return Err(EffectError::Illegal("Kuangfu choice cannot cancel".into()));
    }
    let effect = match payload {
        [0, player, target, skill] if observer == PlayerId::new(*player) => {
            EffectRequest::ResolveKuangfu {
                player: PlayerId::new(*player),
                target: PlayerId::new(*target),
                card: CardId::new(*selected),
                stage: 1,
                skill: engine_api::SkillId::new(*skill),
            }
        }
        [1, player, target, card, skill] if observer == PlayerId::new(*player) => {
            EffectRequest::ResolveKuangfu {
                player: PlayerId::new(*player),
                target: PlayerId::new(*target),
                card: CardId::new(*card),
                stage: match selected {
                    0 => 2,
                    1 => 3,
                    _ => return Err(EffectError::Illegal("unknown Kuangfu branch".into())),
                },
                skill: engine_api::SkillId::new(*skill),
            }
        }
        _ => {
            return Err(EffectError::Illegal(
                "invalid Kuangfu choice actor or stage".into(),
            ));
        }
    };
    run_effects(tx, vec![effect]).map(|_| ())
}

pub(crate) fn resolve_shanjia(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, required, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Shanjia continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let required = usize::try_from(*required).unwrap_or(usize::MAX);
    let skill = engine_api::SkillId::new(*skill);
    let legal = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let mut cards = selections
        .iter()
        .copied()
        .map(CardId::new)
        .collect::<Vec<_>>();
    cards.sort_by_key(|card| card.raw());
    if cancelled
        || observer != player
        || cards.len() != required
        || cards.windows(2).any(|pair| pair[0] == pair[1])
        || cards.iter().any(|card| !legal.contains(card))
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "Shanjia discard choice is no longer legal".into(),
        ));
    }
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    if cards.iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|card| Some(card.zone) == equip)
    }) {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ChooseUseVirtualSha {
                player,
                ignore_distance: true,
            });
    }
    run_effects(
        tx,
        vec![EffectRequest::DiscardTargetCards {
            actor: player,
            target: player,
            cards,
            scope: rule_api::TargetCardScope::HandEquip,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbshanjia(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, required, distance_skill_def, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad mobile Shanjia continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let required = usize::try_from(*required).unwrap_or(usize::MAX);
    let skill = SkillId::new(*skill);
    let legal = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let mut cards = selections
        .iter()
        .copied()
        .map(CardId::new)
        .collect::<Vec<_>>();
    cards.sort_by_key(|card| card.raw());
    if cancelled
        || observer != player
        || cards.len() != required
        || cards.windows(2).any(|pair| pair[0] == pair[1])
        || cards.iter().any(|card| !legal.contains(card))
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "mobile Shanjia discard choice is no longer legal".into(),
        ));
    }
    let mut reward_mask = 0b11;
    for card in &cards {
        match crate::effect::mbshanjia_discard_category(tx.state(), player, *card) {
            1 => reward_mask &= !0b01,
            2 => reward_mask &= !0b10,
            _ => {}
        }
    }
    if reward_mask != 0 {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveMbshanjia {
                player,
                lost_equipment_count: 0,
                distance_skill_def: SkillDefId::new(*distance_skill_def),
                stage: 2,
                reward_mask,
                skill,
            });
    }
    run_effects(
        tx,
        vec![EffectRequest::DiscardTargetCards {
            actor: player,
            target: player,
            cards,
            scope: rule_api::TargetCardScope::HandEquip,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_daoji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let (player, target, skill, tw) = match payload {
        [player, target, skill] => (*player, *target, *skill, false),
        [player, target, skill, tw] => (*player, *target, *skill, *tw != 0),
        _ => {
            return Err(EffectError::EngineFault(
                "bad Daoji continuation payload".into(),
            ));
        }
    };
    let player = PlayerId::new(player);
    let target = PlayerId::new(target);
    let skill = SkillId::new(skill);
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Daoji requires exactly one card".into(),
        ));
    };
    let selected = CardId::new(*selected);
    let legal = crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        if tw {
            rule_api::TargetCardScope::HandEquip
        } else {
            rule_api::TargetCardScope::Equip
        },
    );
    if cancelled
        || observer != player
        || !legal.contains(&selected)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "Daoji card choice is no longer legal".into(),
        ));
    }
    let from = tx
        .state()
        .cards
        .get(selected)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("Daoji selected card is missing".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Daoji owner hand is missing".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveDaoji {
            player,
            target,
            card: selected,
            stage: 1,
            tw,
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![selected],
            to,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_give_all_hand_then_choose_use_one_or_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 {
        return Err(EffectError::EngineFault(
            "bad give-all-hand use-or-damage payload".into(),
        ));
    }
    let stage = payload[0];
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = SkillId::new(payload[3]);
    let card = CardId::new(payload[4]);
    let count = payload[5] as usize;
    let cards = payload
        .get(6..6usize.saturating_add(count))
        .ok_or_else(|| EffectError::EngineFault("truncated gifted-card payload".into()))?
        .iter()
        .copied()
        .map(CardId::new)
        .collect::<Vec<_>>();
    if observer != target {
        return Err(EffectError::Illegal("gift recipient must choose".into()));
    }

    let take_damage = |tx: &mut Transaction| {
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
    };
    let use_card = |tx: &mut Transaction, card: CardId, targets: Vec<PlayerId>| {
        let target_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::EngineFault("gift recipient hand missing".into()))?;
        let owner_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::EngineFault("gift owner hand missing".into()))?;
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
        let remaining: Vec<_> = cards
            .iter()
            .copied()
            .filter(|candidate| *candidate != card)
            .filter(|candidate| {
                tx.state()
                    .cards
                    .get(*candidate)
                    .is_some_and(|state| state.owner == target && state.zone == target_hand)
            })
            .collect();
        let mut effects = vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: processing,
            from: Some(target_hand),
        }];
        if !remaining.is_empty()
            && tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
        {
            effects.push(EffectRequest::MoveCards {
                cards: remaining,
                to: owner_hand,
                from: Some(target_hand),
            });
        }
        effects.push(EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
            player,
            target,
            cards: cards.clone(),
            targets,
            stage: 2,
            card,
            skill,
        });
        run_effects(tx, effects).map(|_| ())
    };

    match stage {
        1 => {
            if cancelled {
                return take_damage(tx);
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal("choose one gifted card".into()));
            };
            let selected = CardId::new(*selected);
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            if !cards.contains(&selected)
                || !tx
                    .state()
                    .cards
                    .get(selected)
                    .is_some_and(|state| state.owner == target && hand == Some(state.zone))
            {
                return Err(EffectError::Illegal("gifted card moved".into()));
            }
            match crate::command::play::skill_card_target_spec(tx, target, selected)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    use_card(tx, selected, targets)
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    let mut next = vec![
                        2,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        selected.raw(),
                        cards.len().min(u32::MAX as usize) as u32,
                    ];
                    next.extend(cards.iter().map(|card| card.raw()));
                    open_choice(
                        tx,
                        target,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::GiveAllHandThenChooseUseOneOrDamage,
                        next,
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
                crate::command::play::SkillCardTargetSpec::Pairs(pairs) => {
                    let mut next = vec![
                        3,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        selected.raw(),
                        cards.len().min(u32::MAX as usize) as u32,
                    ];
                    next.extend(cards.iter().map(|card| card.raw()));
                    open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        (1..=pairs.len() as u32).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::GiveAllHandThenChooseUseOneOrDamage,
                        next,
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
            }
        }
        2 => {
            if cancelled {
                return Err(EffectError::Illegal(
                    "card target choice cannot cancel".into(),
                ));
            }
            let crate::command::play::SkillCardTargetSpec::Select { min, max, options } =
                crate::command::play::skill_card_target_spec(tx, target, card)?
            else {
                return Err(EffectError::Illegal("card target shape changed".into()));
            };
            let mut seen = std::collections::BTreeSet::new();
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            if targets.len() < min as usize
                || targets.len() > max as usize
                || targets
                    .iter()
                    .any(|target| !seen.insert(*target) || !options.contains(target))
            {
                return Err(EffectError::Illegal("illegal gifted-card targets".into()));
            }
            use_card(tx, card, targets)
        }
        3 => {
            let [selected] = selections else {
                return Err(EffectError::Illegal("choose one target pair".into()));
            };
            let crate::command::play::SkillCardTargetSpec::Pairs(pairs) =
                crate::command::play::skill_card_target_spec(tx, target, card)?
            else {
                return Err(EffectError::Illegal("card target shape changed".into()));
            };
            let index = selected
                .checked_sub(1)
                .map(|value| value as usize)
                .ok_or_else(|| EffectError::Illegal("bad target pair".into()))?;
            let (first, second) = pairs
                .get(index)
                .copied()
                .ok_or_else(|| EffectError::Illegal("bad target pair".into()))?;
            if cancelled {
                return Err(EffectError::Illegal("target pair cannot cancel".into()));
            }
            use_card(tx, card, vec![first, second])
        }
        _ => Err(EffectError::EngineFault(
            "unknown give-all-hand use-or-damage stage".into(),
        )),
    }
}

pub(crate) fn resolve_yuanlve_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [marker, stage, player, target, card, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Yuanlve choice payload".into(),
        ));
    };
    if *marker != crate::effect::YUANLVE_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad Yuanlve choice marker".into()));
    }
    let player = PlayerId::new(*player);
    let target = PlayerId::new(*target);
    let card = CardId::new(*card);
    let skill = SkillId::new(*skill);
    if observer != target {
        return Err(EffectError::Illegal(
            "only the Yuanlve recipient may choose".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("Yuanlve recipient hand missing".into()))?;
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.owner == target && state.zone == hand)
    {
        return Err(EffectError::Illegal("Yuanlve card moved before use".into()));
    }

    let commit = |tx: &mut Transaction, targets: Vec<PlayerId>| {
        let processing = tx
            .state()
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
        run_effects(
            tx,
            vec![
                EffectRequest::MoveCards {
                    cards: vec![card],
                    to: processing,
                    from: Some(hand),
                },
                EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
                    player,
                    target,
                    card,
                    targets,
                    stage: 2,
                    skill,
                },
            ],
        )
        .map(|_| ())
    };

    match *stage {
        1 => {
            if cancelled {
                return Ok(());
            }
            if selections != [card.raw()] {
                return Err(EffectError::Illegal("choose the exact Yuanlve card".into()));
            }
            match crate::command::play::skill_card_target_spec(tx, target, card)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => commit(tx, targets),
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    open_choice(
                        tx,
                        target,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        vec![
                            crate::effect::YUANLVE_CUSTOM_CHOICE,
                            2,
                            player.raw(),
                            target.raw(),
                            card.raw(),
                            skill.raw(),
                        ],
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
                crate::command::play::SkillCardTargetSpec::Pairs(pairs) => {
                    open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        (1..=pairs.len() as u32).collect(),
                        ChoiceKind::Custom,
                        ContinuationKind::Custom,
                        vec![
                            crate::effect::YUANLVE_CUSTOM_CHOICE,
                            3,
                            player.raw(),
                            target.raw(),
                            card.raw(),
                            skill.raw(),
                        ],
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
            }
        }
        2 => {
            if cancelled {
                return Err(EffectError::Illegal("Yuanlve targets cannot cancel".into()));
            }
            let crate::command::play::SkillCardTargetSpec::Select { min, max, options } =
                crate::command::play::skill_card_target_spec(tx, target, card)?
            else {
                return Err(EffectError::Illegal("Yuanlve target shape changed".into()));
            };
            let mut seen = std::collections::BTreeSet::new();
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            if targets.len() < usize::from(min)
                || targets.len() > usize::from(max)
                || targets
                    .iter()
                    .any(|selected| !seen.insert(*selected) || !options.contains(selected))
            {
                return Err(EffectError::Illegal("illegal Yuanlve targets".into()));
            }
            commit(tx, targets)
        }
        3 => {
            if cancelled {
                return Err(EffectError::Illegal(
                    "Yuanlve target pair cannot cancel".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "choose one Yuanlve target pair".into(),
                ));
            };
            let crate::command::play::SkillCardTargetSpec::Pairs(pairs) =
                crate::command::play::skill_card_target_spec(tx, target, card)?
            else {
                return Err(EffectError::Illegal("Yuanlve pair shape changed".into()));
            };
            let index = selected
                .checked_sub(1)
                .map(|value| value as usize)
                .ok_or_else(|| EffectError::Illegal("bad Yuanlve target pair".into()))?;
            let (first, second) = pairs
                .get(index)
                .copied()
                .ok_or_else(|| EffectError::Illegal("bad Yuanlve target pair".into()))?;
            commit(tx, vec![first, second])
        }
        _ => Err(EffectError::EngineFault(
            "unknown Yuanlve choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_category_then_target_give_matching_or_sha_unrespondable(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, sha, target, skill, rest @ ..] = payload else {
        return Err(EffectError::EngineFault(
            "bad category-gift-or-direct-hit payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let sha = CardId::new(*sha);
    let target = PlayerId::new(*target);
    let skill = SkillId::new(*skill);
    let damage_mode = match *stage {
        0 => rest.first() == Some(&crate::effect::CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE),
        1 => rest.get(1) == Some(&crate::effect::CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE),
        _ => false,
    };
    if ![player, target].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    }) {
        return Err(EffectError::Illegal(
            "category-gift-or-direct-hit participants changed".into(),
        ));
    }
    if damage_mode {
        if tx.state().resolution.current_damage_id != sha.raw() {
            return Err(EffectError::Illegal(
                "category-gift damage context changed".into(),
            ));
        }
    } else {
        crate::command::play::validate_saved_sha_target(tx, player, sha, target)?;
    }
    let matching = |tx: &Transaction, category: u8| {
        [ZoneKind::Hand, ZoneKind::Equip]
            .into_iter()
            .filter_map(|kind| tx.state().zones.find(kind, target))
            .filter_map(|zone| tx.state().zones.get(zone))
            .flat_map(|zone| zone.cards.iter().copied())
            .filter(|card| {
                crate::effect::effective_card_category(tx.state(), target, *card) == category
            })
            .collect::<Vec<_>>()
    };
    let fallback = |tx: &mut Transaction| {
        if damage_mode {
            crate::effect::increase_bound_damage_by_one(tx, sha.raw()).map(|_| ())
        } else {
            run_effects(
                tx,
                vec![EffectRequest::MakeCurrentShaTargetUnrespondable {
                    source: player,
                    card: sha,
                    target,
                }],
            )
            .map(|_| ())
        }
    };

    match *stage {
        0 => {
            if cancelled || observer != player {
                return Err(EffectError::Illegal(
                    "category choice cannot cancel or change actor".into(),
                ));
            }
            let [category] = selections else {
                return Err(EffectError::Illegal("choose one card category".into()));
            };
            let category = u8::try_from(*category)
                .ok()
                .filter(|category| (1..=3).contains(category))
                .ok_or_else(|| EffectError::Illegal("unknown card category".into()))?;
            let options = matching(tx, category);
            if options.is_empty() {
                return fallback(tx);
            }
            let mut next = vec![
                1,
                player.raw(),
                sha.raw(),
                target.raw(),
                skill.raw(),
                u32::from(category),
            ];
            if damage_mode {
                next.push(crate::effect::CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE);
            }
            open_choice(
                tx,
                target,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseCategoryThenTargetGiveMatchingOrShaUnrespondable,
                next,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        1 => {
            if observer != target {
                return Err(EffectError::Illegal("Sha target must answer".into()));
            }
            if cancelled {
                if selections.is_empty() {
                    return fallback(tx);
                }
                return Err(EffectError::Illegal(
                    "cancelled category gift contains a card".into(),
                ));
            }
            let category = rest
                .first()
                .and_then(|category| u8::try_from(*category).ok())
                .ok_or_else(|| EffectError::EngineFault("missing card category".into()))?;
            let [selected] = selections else {
                return Err(EffectError::Illegal("give one matching card".into()));
            };
            let selected = CardId::new(*selected);
            if !matching(tx, category).contains(&selected) {
                return Err(EffectError::Illegal("matching category card moved".into()));
            }
            let from = response_card_zone(tx, target, selected)?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("recipient hand missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![selected],
                    to,
                    from,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown category-gift-or-direct-hit stage".into(),
        )),
    }
}

pub(crate) fn resolve_jiaohua(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Jiaohua stage".into()))?;
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Jiaohua requires exactly one selection".into(),
        ));
    };
    match stage {
        0 => {
            let [_, player, skill, option_count, options @ ..] = payload else {
                return Err(EffectError::EngineFault(
                    "truncated Jiaohua category payload".into(),
                ));
            };
            if options.len() != *option_count as usize {
                return Err(EffectError::EngineFault(
                    "bad Jiaohua category payload".into(),
                ));
            }
            let player = PlayerId::new(*player);
            let category = u8::try_from(*selected)
                .map_err(|_| EffectError::Illegal("bad Jiaohua category token".into()))?;
            if cancelled || observer != player || !options.contains(selected) {
                return Err(EffectError::Illegal(
                    "Jiaohua category choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveJiaohua {
                    player,
                    category,
                    target: PlayerId::NONE,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        1 => {
            let [_, player, category, skill, option_count, options @ ..] = payload else {
                return Err(EffectError::EngineFault(
                    "truncated Jiaohua target payload".into(),
                ));
            };
            if options.len() != *option_count as usize {
                return Err(EffectError::EngineFault(
                    "bad Jiaohua target payload".into(),
                ));
            }
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*selected);
            let category = u8::try_from(*category)
                .map_err(|_| EffectError::Illegal("bad Jiaohua category state".into()))?;
            if cancelled
                || observer != player
                || !options.contains(selected)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Jiaohua target choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveJiaohua {
                    player,
                    category,
                    target,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Jiaohua choice stage".into(),
        )),
    }
}
