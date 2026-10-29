//! Command operations for `resolve_draw_give_or_discard_hand_discard` and related helpers.

use super::*;

pub(crate) fn resolve_choose_repolu_targets_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Repolu target continuation".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player {
        return Err(EffectError::Illegal("Repolu chooser mismatch".into()));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Repolu choice must not select targets".into(),
            ))
        };
    }
    if selections.is_empty()
        || selections.len() > usize::from(u8::MAX)
        || selections
            .iter()
            .enumerate()
            .any(|(index, target)| selections[..index].contains(target))
    {
        return Err(EffectError::Illegal(
            "Repolu requires distinct living targets".into(),
        ));
    }
    let mark = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("Repolu skill context changed".into()))?;
    let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if selected.iter().any(|target| {
        !tx.state()
            .players
            .get(*target)
            .is_some_and(|participant| participant.alive)
    }) {
        return Err(EffectError::Illegal(
            "Repolu target is no longer alive".into(),
        ));
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| selected.contains(target))
        .collect();
    if targets.len() != selected.len() {
        return Err(EffectError::Illegal(
            "Repolu target seat order could not be resolved".into(),
        ));
    }
    let previous = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("Repolu owner is missing".into()))?
        .marks
        .iter()
        .find(|(existing, _)| *existing == mark)
        .map(|(_, count)| *count)
        .unwrap_or(0)
        .max(0) as u32;
    let draw_count = previous.saturating_add(1);
    let mut effects = vec![EffectRequest::AddMark {
        player,
        mark: engine_api::MarkDefId::new(mark),
        delta: 1,
    }];
    effects.extend(
        targets
            .into_iter()
            .map(|target| EffectRequest::DrawCardsFromSkill {
                player: target,
                count: draw_count,
                skill,
            }),
    );
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_zhuiyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, killer, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Zhuiyi target continuation".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let killer = PlayerId::new(*killer);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player {
        return Err(EffectError::Illegal("Zhuiyi chooser mismatch".into()));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Zhuiyi choice must not select a target".into(),
            ))
        };
    }
    let [target] = selections else {
        return Err(EffectError::Illegal(
            "Zhuiyi requires exactly one target".into(),
        ));
    };
    let target = PlayerId::new(*target);
    if target == player
        || target == killer
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Zhuiyi target is no longer legal".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal("Zhuiyi skill context changed".into()));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::RecoverHp {
                source: PlayerId::NONE,
                player: target,
                amount: 1,
            },
            EffectRequest::DrawCardsFromSkill {
                player: target,
                count: 3,
                skill,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_xinfu_guolun(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Guolun choice stage".into()))?;
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal("Guolun chooser mismatch".into()));
    }
    match stage {
        0 => {
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "Guolun requires one target hand card".into(),
                ));
            };
            if cancelled {
                return Err(EffectError::Illegal(
                    "Guolun target-hand choice cannot be cancelled".into(),
                ));
            }
            let target_card = CardId::new(*card);
            if !tx.state().hand_cards(target).contains(&target_card) {
                return Err(EffectError::Illegal("Guolun target hand card moved".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::RevealCards {
                    cards: vec![target_card],
                    to: Vec::new(),
                }],
            )?;
            let mut options = tx.state().hand_cards(player);
            if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
                options.extend(
                    tx.state()
                        .zones
                        .get(equip)
                        .map(|zone| zone.cards.clone())
                        .unwrap_or_default(),
                );
            }
            if options.is_empty() {
                return Ok(());
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ResolveXinfuGuolun,
                vec![
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    target_card.raw(),
                ],
            )
            .map(|_| ())
        }
        1 => {
            if cancelled {
                return if selections.is_empty() {
                    Ok(())
                } else {
                    Err(EffectError::Illegal(
                        "cancelled Guolun choice must not select a card".into(),
                    ))
                };
            }
            let [player_card] = selections else {
                return Err(EffectError::Illegal(
                    "Guolun exchange requires one owner card".into(),
                ));
            };
            let player_card = CardId::new(*player_card);
            let target_card =
                CardId::new(*payload.get(4).ok_or_else(|| {
                    EffectError::EngineFault("missing Guolun target card".into())
                })?);
            run_effects(
                tx,
                vec![
                    EffectRequest::RevealCards {
                        cards: vec![player_card],
                        to: Vec::new(),
                    },
                    EffectRequest::ResolveXinfuGuolun {
                        player,
                        target,
                        player_card,
                        target_card,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Guolun choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_target_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(
        *payload
            .first()
            .ok_or_else(|| EffectError::EngineFault("missing target-draw chooser".into()))?,
    );
    if observer != player {
        return Err(EffectError::Illegal("target-draw chooser mismatch".into()));
    }
    if (cancelled && payload.len() != 6) || (payload.len() == 3 && selections.len() != 1) {
        return Err(EffectError::Illegal(
            "choose the required number of draw targets".into(),
        ));
    }
    if payload.len() == 3 {
        let cap = payload[1];
        let skill = engine_api::SkillId::new(payload[2]);
        let target = PlayerId::new(selections[0]);
        let participant = tx
            .state()
            .players
            .get(target)
            .filter(|participant| participant.alive)
            .ok_or_else(|| EffectError::Illegal("selected draw target is not alive".into()))?;
        if cap == 0 {
            return Err(EffectError::EngineFault(
                "maximum-HP draw cap must be positive".into(),
            ));
        }
        let count = participant
            .max_hp
            .min(cap)
            .saturating_sub(tx.state().hand_count(target));
        return run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player: target,
                count,
                skill,
            }],
        )
        .map(|_| ());
    }
    let (min_count, max_count, scope_index, skill_index, optional) = match payload.len() {
        4 => (1usize, 1usize, 2usize, 3usize, false),
        5 => (
            payload[2] as usize,
            payload[2] as usize,
            3usize,
            4usize,
            false,
        ),
        6 => (
            payload[2] as usize,
            payload[3] as usize,
            4usize,
            5usize,
            true,
        ),
        _ => {
            return Err(EffectError::EngineFault(
                "bad target-draw continuation".into(),
            ));
        }
    };
    let count = payload[1];
    let scope = u8::try_from(payload[scope_index])
        .ok()
        .and_then(DrawTargetScope::from_tag)
        .ok_or_else(|| EffectError::EngineFault("bad target-draw scope".into()))?;
    let skill = engine_api::SkillId::new(payload[skill_index]);
    if cancelled && optional {
        return Ok(());
    }
    if min_count == 0
        || min_count > max_count
        || cancelled
        || selections.len() < min_count
        || selections.len() > max_count
    {
        return Err(EffectError::Illegal(
            "choose the required number of draw targets".into(),
        ));
    }
    if selections
        .iter()
        .enumerate()
        .any(|(index, token)| selections[..index].contains(token))
    {
        return Err(EffectError::Illegal("draw targets must be distinct".into()));
    }
    let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if selected
        .iter()
        .any(|target| !crate::effect::player_target_matches(tx.state(), player, *target, scope))
    {
        return Err(EffectError::Illegal(
            "selected player is outside the draw target scope".into(),
        ));
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| selected.contains(target))
        .collect();
    if targets.len() != selections.len() {
        return Err(EffectError::Illegal(
            "draw target seat order could not be resolved".into(),
        ));
    }
    run_effects(
        tx,
        targets
            .into_iter()
            .map(|target| EffectRequest::DrawCardsFromSkill {
                player: target,
                count,
                skill,
            })
            .collect(),
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_targets_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6 {
        return Err(EffectError::EngineFault(
            "bad target-damage continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let min_count = u8::try_from(payload[1])
        .map_err(|_| EffectError::EngineFault("bad target-damage minimum".into()))?;
    let max_count = u8::try_from(payload[2])
        .map_err(|_| EffectError::EngineFault("bad target-damage maximum".into()))?;
    let scope = u8::try_from(payload[3])
        .ok()
        .and_then(DrawTargetScope::from_tag)
        .ok_or_else(|| EffectError::EngineFault("bad target-damage scope".into()))?;
    let amount = payload[4];
    let skill = engine_api::SkillId::new(payload[5]);
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if observer != player
        || cancelled
        || min_count == 0
        || min_count > max_count
        || amount == 0
        || !skill_enabled
        || selections.len() < usize::from(min_count)
        || selections.len() > usize::from(max_count)
    {
        return Err(EffectError::Illegal(
            "choose the required number of damage targets".into(),
        ));
    }
    if selections
        .iter()
        .enumerate()
        .any(|(index, token)| selections[..index].contains(token))
    {
        return Err(EffectError::Illegal(
            "damage targets must be distinct".into(),
        ));
    }
    let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if selected
        .iter()
        .any(|target| !crate::effect::player_target_matches(tx.state(), player, *target, scope))
    {
        return Err(EffectError::Illegal(
            "selected player is outside the damage target scope".into(),
        ));
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| selected.contains(target))
        .collect();
    if targets.len() != selections.len() {
        return Err(EffectError::Illegal(
            "damage target seat order could not be resolved".into(),
        ));
    }
    run_effects(
        tx,
        targets
            .into_iter()
            .map(|target| EffectRequest::DealDamage {
                source: player,
                target,
                amount,
                is_lose_hp: false,
                nature: 0,
            })
            .collect(),
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_target_draw_then_owner_if_below_max_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad target-draw-follow-up continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let count = payload[1];
    let skill = engine_api::SkillId::new(payload[2]);
    let target = selections.first().copied().map(PlayerId::new);
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if observer != player
        || cancelled
        || selections.len() != 1
        || count == 0
        || !skill_enabled
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !target.is_some_and(|target| {
            tx.state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        })
    {
        return Err(EffectError::Illegal(
            "target-draw-follow-up choice context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
            player,
            target: target.expect("validated target selection"),
            count,
            skill,
            ready_to_check: false,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) if payload.len() == 3 => {
            let player = PlayerId::new(payload[1]);
            let skill = engine_api::SkillId::new(payload[2]);
            let target = selections.first().copied().map(PlayerId::new);
            let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            });
            if observer != player
                || cancelled
                || selections.len() != 1
                || !skill_enabled
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.alive)
                || !target.is_some_and(|target| {
                    tx.state()
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                        && !crate::query::discardable_target_cards(
                            tx.state(),
                            target,
                            target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
            {
                return Err(EffectError::Illegal(
                    "self-discard target choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                        player,
                        target: target.expect("validated self-discard target"),
                        skill,
                        ready_to_check: false,
                    },
                ],
            )
            .map(|_| ())
        }
        Some(1) if payload.len() == 4 => {
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let selected = selections.first().copied().map(CardId::new);
            let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            });
            if observer != target
                || cancelled
                || selections.len() != 1
                || !skill_enabled
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
                || !selected.is_some_and(|card| {
                    crate::query::discardable_target_cards(
                        tx.state(),
                        target,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .contains(&card)
                })
            {
                return Err(EffectError::Illegal(
                    "self-discard card choice context changed".into(),
                ));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                    player,
                    target,
                    skill,
                    ready_to_check: true,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: vec![selected.expect("validated self-discard card")],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad self-discard hand-vs-HP continuation".into(),
        )),
    }
}

pub(crate) fn resolve_choose_give_one_hand_equip_or_recover(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::EngineFault(
            "truncated give-or-recover continuation".into(),
        ));
    }
    let owner = PlayerId::new(payload[0]);
    let giver = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let option_count = payload[4] as usize;
    if payload.len() != 5 + option_count {
        return Err(EffectError::EngineFault(
            "bad give-or-recover option snapshot".into(),
        ));
    }
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == owner && tx.state().skill_effectively_enabled(instance)
    });
    let alive = |player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    };
    let target_dying =
        tx.state().players.get(target).is_some_and(|participant| {
            participant.alive && participant.dying && participant.hp <= 0
        });
    if observer != giver
        || !skill_enabled
        || !alive(owner)
        || !alive(giver)
        || !target_dying
        || selections.len() > 1
        || (cancelled && !selections.is_empty())
    {
        return Err(EffectError::Illegal(
            "give-or-recover choice context changed".into(),
        ));
    }

    if cancelled || selections.is_empty() {
        return run_effects(
            tx,
            vec![EffectRequest::RecoverHp {
                source: PlayerId::NONE,
                player: target,
                amount: 1,
            }],
        )
        .map(|_| ());
    }

    let card = CardId::new(selections[0]);
    if !payload[5..].contains(&card.raw()) {
        return Err(EffectError::Illegal(
            "card was not offered for give-or-recover".into(),
        ));
    }
    let from = response_card_zone(tx, giver, card)?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no recipient hand".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::GiveCards {
            cards: vec![card],
            to,
            from,
        }],
    )
    .map(|_| ())
}
