//! Command operations for `resolve_tuxi_gain_other_hands_replace_draw` and related helpers.

use super::*;

pub(crate) fn resolve_tuxi_gain_other_hands_replace_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("truncated Tuxi payload".into()))?;
    let player = PlayerId::new(
        *payload
            .get(1)
            .ok_or_else(|| EffectError::EngineFault("truncated Tuxi player".into()))?,
    );
    let skill = engine_api::SkillId::new(
        *payload
            .get(2)
            .ok_or_else(|| EffectError::EngineFault("truncated Tuxi skill".into()))?,
    );
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "Tuxi choice cannot be cancelled".into(),
        ));
    }

    match stage {
        0 | 2 => {
            let reduce_draw = stage == 2;
            let draw_count = if reduce_draw {
                *payload
                    .get(3)
                    .ok_or_else(|| EffectError::EngineFault("truncated Tuxi draw count".into()))?
            } else {
                0
            };
            let option_count_index = if reduce_draw { 4 } else { 3 };
            let option_count = *payload
                .get(option_count_index)
                .ok_or_else(|| EffectError::EngineFault("truncated Tuxi targets".into()))?
                as usize;
            let first_option = option_count_index + 1;
            let max_targets = if reduce_draw {
                option_count.min(draw_count as usize)
            } else {
                option_count.min(2)
            };
            if payload.len() != first_option + option_count
                || selections.is_empty()
                || selections.len() > max_targets
            {
                return Err(EffectError::Illegal(
                    "Tuxi target count is outside the pending draw bound".into(),
                ));
            }
            let snapshotted = &payload[first_option..];
            let mut seen = std::collections::BTreeSet::new();
            let mut targets: Vec<_> = selections
                .iter()
                .map(|raw| {
                    let target = PlayerId::new(*raw);
                    if !seen.insert(*raw)
                        || !snapshotted.contains(raw)
                        || target == player
                        || !tx
                            .state()
                            .players
                            .get(target)
                            .is_some_and(|participant| participant.alive)
                        || tx.state().hand_count(target) == 0
                    {
                        return Err(EffectError::Illegal(
                            "Tuxi target is invalid or stale".into(),
                        ));
                    }
                    Ok(target)
                })
                .collect::<Result<_, _>>()?;
            targets.sort_by_key(|target| {
                tx.state()
                    .players
                    .get(*target)
                    .map_or(u8::MAX, |participant| participant.seat.raw())
            });
            run_effects(
                tx,
                vec![if reduce_draw {
                    EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
                        player,
                        targets,
                        index: 0,
                        selected: CardId::NONE,
                        draw_count,
                        skill,
                    }
                } else {
                    EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
                        player,
                        targets,
                        index: 0,
                        selected: CardId::NONE,
                        skill,
                    }
                }],
            )
            .map(|_| ())
        }
        1 | 3 => {
            let reduce_draw = stage == 3;
            let draw_count = if reduce_draw {
                *payload
                    .get(3)
                    .ok_or_else(|| EffectError::EngineFault("truncated Tuxi draw count".into()))?
            } else {
                0
            };
            let index_index = if reduce_draw { 4 } else { 3 };
            let index = *payload
                .get(index_index)
                .ok_or_else(|| EffectError::EngineFault("truncated Tuxi index".into()))?;
            let target_count = *payload
                .get(index_index + 1)
                .ok_or_else(|| EffectError::EngineFault("truncated Tuxi target count".into()))?
                as usize;
            let first_target = index_index + 2;
            let option_count_index = first_target
                .checked_add(target_count)
                .ok_or_else(|| EffectError::EngineFault("oversized Tuxi payload".into()))?;
            let option_count = *payload
                .get(option_count_index)
                .ok_or_else(|| EffectError::EngineFault("truncated Tuxi card options".into()))?
                as usize;
            if payload.len() != option_count_index + 1 + option_count || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Tuxi requires exactly one target hand card".into(),
                ));
            }
            let targets: Vec<_> = payload[first_target..option_count_index]
                .iter()
                .map(|raw| PlayerId::new(*raw))
                .collect();
            let target = targets
                .get(index as usize)
                .copied()
                .ok_or_else(|| EffectError::Illegal("Tuxi target index is stale".into()))?;
            let selected = CardId::new(selections[0]);
            if !payload[option_count_index + 1..].contains(&selected.raw())
                || !tx.state().hand_cards(target).contains(&selected)
            {
                return Err(EffectError::Illegal(
                    "Tuxi hand-card selection is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![if reduce_draw {
                    EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
                        player,
                        targets,
                        index,
                        selected,
                        draw_count,
                        skill,
                    }
                } else {
                    EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
                        player,
                        targets,
                        index,
                        selected,
                        skill,
                    }
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown Tuxi stage".into())),
    }
}

pub(crate) fn resolve_gain_one_hand_from_selected_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(
        *payload
            .first()
            .ok_or_else(|| EffectError::EngineFault("truncated Cuorui player".into()))?,
    );
    let skill = engine_api::SkillId::new(
        *payload
            .get(1)
            .ok_or_else(|| EffectError::EngineFault("truncated Cuorui skill".into()))?,
    );
    let index = *payload
        .get(2)
        .ok_or_else(|| EffectError::EngineFault("truncated Cuorui index".into()))?;
    let target_count = *payload
        .get(3)
        .ok_or_else(|| EffectError::EngineFault("truncated Cuorui target count".into()))?
        as usize;
    let option_count_index = 4usize
        .checked_add(target_count)
        .ok_or_else(|| EffectError::EngineFault("oversized Cuorui payload".into()))?;
    let option_count = *payload
        .get(option_count_index)
        .ok_or_else(|| EffectError::EngineFault("truncated Cuorui card options".into()))?
        as usize;
    if observer != player
        || cancelled
        || selections.len() != 1
        || payload.len() != option_count_index + 1 + option_count
    {
        return Err(EffectError::Illegal(
            "Cuorui requires exactly one target hand card".into(),
        ));
    }
    let targets: Vec<_> = payload[4..option_count_index]
        .iter()
        .map(|raw| PlayerId::new(*raw))
        .collect();
    let target = targets
        .get(index as usize)
        .copied()
        .ok_or_else(|| EffectError::Illegal("Cuorui target index is stale".into()))?;
    let selected = CardId::new(selections[0]);
    if !payload[option_count_index + 1..].contains(&selected.raw())
        || !tx.state().hand_cards(target).contains(&selected)
    {
        return Err(EffectError::Illegal(
            "Cuorui hand-card selection is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::GainOneHandFromSelectedTargets {
            player,
            targets,
            index,
            selected,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_yiji_distribute_top_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("truncated Yiji payload".into()))?;
    let player = PlayerId::new(
        *payload
            .get(1)
            .ok_or_else(|| EffectError::EngineFault("truncated Yiji player".into()))?,
    );
    let skill = engine_api::SkillId::new(
        *payload
            .get(2)
            .ok_or_else(|| EffectError::EngineFault("truncated Yiji skill".into()))?,
    );
    let card_count = *payload
        .get(3)
        .ok_or_else(|| EffectError::EngineFault("truncated Yiji card count".into()))?
        as usize;
    let cards_end = 4usize
        .checked_add(card_count)
        .ok_or_else(|| EffectError::EngineFault("oversized Yiji payload".into()))?;
    if observer != player || cancelled || payload.len() < cards_end {
        return Err(EffectError::Illegal(
            "Yiji choice cannot be cancelled".into(),
        ));
    }
    let cards: Vec<_> = payload[4..cards_end]
        .iter()
        .map(|raw| CardId::new(*raw))
        .collect();

    match stage {
        0 => {
            if payload.len() != cards_end || selections.is_empty() || selections.len() > cards.len()
            {
                return Err(EffectError::Illegal(
                    "Yiji requires a nonempty remaining-card subset".into(),
                ));
            }
            let mut seen = std::collections::BTreeSet::new();
            let selected: Vec<_> = selections
                .iter()
                .map(|raw| {
                    let card = CardId::new(*raw);
                    if !seen.insert(*raw) || !cards.contains(&card) {
                        return Err(EffectError::Illegal(
                            "Yiji subset is invalid or duplicated".into(),
                        ));
                    }
                    Ok(card)
                })
                .collect::<Result<_, _>>()?;
            run_effects(
                tx,
                vec![EffectRequest::DistributeTopTwoCards {
                    player,
                    cards,
                    selected,
                    target: PlayerId::NONE,
                    skill,
                    ready_to_choose: true,
                }],
            )
            .map(|_| ())
        }
        1 => {
            let selected_count = *payload
                .get(cards_end)
                .ok_or_else(|| EffectError::EngineFault("truncated Yiji subset".into()))?
                as usize;
            if payload.len() != cards_end + 1 + selected_count || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Yiji requires exactly one recipient".into(),
                ));
            }
            let selected: Vec<_> = payload[cards_end + 1..]
                .iter()
                .map(|raw| CardId::new(*raw))
                .collect();
            let target = PlayerId::new(selections[0]);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Yiji recipient is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DistributeTopTwoCards {
                    player,
                    cards,
                    selected,
                    target,
                    skill,
                    ready_to_choose: true,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown Yiji stage".into())),
    }
}

pub(crate) fn resolve_xiaoguo_discard_basic_then_equip_or_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, target, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Xiaoguo continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let target = PlayerId::new(*target);
    let skill = engine_api::SkillId::new(*skill);
    match stage {
        0 => {
            let current = crate::query::standard_xiaoguo_basic_hand_cards(tx.state(), player);
            let valid = observer == player
                && tx.state().turn.current == target
                && tx.state().turn.phase == Phase::End
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|state| state.alive)
                && tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                });
            if !valid {
                return Err(EffectError::Illegal("Xiaoguo cost context changed".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled Xiaoguo cost contains a card".into(),
                    ));
                }
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Xiaoguo requires exactly one basic hand card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            if !current.contains(&selected) {
                return Err(EffectError::Illegal(
                    "Xiaoguo basic-card cost is invalid or stale".into(),
                ));
            }
            match run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![selected],
                }],
            ) {
                Ok(_) => crate::effect::open_xiaoguo_target_equipment_or_damage(
                    tx, player, target, skill,
                ),
                Err(EffectError::ChoiceRequested) => {
                    tx.state_mut().resolution.resume_kind =
                        ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage;
                    tx.state_mut().resolution.resume_payload =
                        vec![1, player.raw(), target.raw(), skill.raw()];
                    Err(EffectError::ChoiceRequested)
                }
                Err(error) => Err(error),
            }
        }
        2 => {
            if observer != target
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|state| state.alive)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|state| state.alive)
            {
                return Err(EffectError::Illegal(
                    "Xiaoguo consequence context changed".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled Xiaoguo equipment choice contains a card".into(),
                    ));
                }
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )
                .map(|_| ());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Xiaoguo requires exactly one equipment card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            if !crate::query::standard_xiaoguo_equipment_cards(tx.state(), target)
                .contains(&selected)
            {
                return Err(EffectError::Illegal(
                    "Xiaoguo equipment choice is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: vec![selected],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "invalid Xiaoguo choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_sp_xiaoguo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, target, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad SP Xiaoguo continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let target = PlayerId::new(*target);
    let skill = engine_api::SkillId::new(*skill);
    match stage {
        0 => {
            let valid = observer == player
                && player != target
                && tx.state().turn.current == target
                && tx.state().turn.phase == Phase::End
                && tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                && tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                });
            if !valid {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo cost context changed".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled SP Xiaoguo cost contains a card".into(),
                    ));
                }
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo requires exactly one hand card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            if !crate::query::sp_xiaoguo_hand_cards(tx.state(), player).contains(&selected) {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo hand-card cost is invalid or stale".into(),
                ));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveSpXiaoguo {
                    player,
                    target,
                    stage: 1,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![selected],
                }],
            )
            .map(|_| ())
        }
        2 => {
            if observer != target
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
                return Err(EffectError::Illegal(
                    "SP Xiaoguo consequence context changed".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled SP Xiaoguo equipment choice contains a card".into(),
                    ));
                }
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveSpXiaoguo {
                        player,
                        target,
                        stage: 2,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo requires exactly one equipment card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            if !crate::query::sp_xiaoguo_equipment_cards(tx.state(), target).contains(&selected) {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo equipment choice is invalid or stale".into(),
                ));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveSpXiaoguo {
                    player,
                    target,
                    stage: 3,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: vec![selected],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "invalid SP Xiaoguo choice stage".into(),
        )),
    }
}
