//! Command operations for `resolve_choose_other_yinghun` and related helpers.

use super::*;

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
