//! Command operations for `resolve_choose_other_give_one_hand_equip` and related helpers.

use super::*;

pub(crate) fn resolve_retieji(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [source, card, target, suit] = payload else {
        return Err(EffectError::EngineFault(
            "bad Retieji choice payload".into(),
        ));
    };
    let source = PlayerId::new(*source);
    let card = CardId::new(*card);
    let target = PlayerId::new(*target);
    let suit = crate::command::judge::suit_from_tag(*suit);
    if observer != target
        || ![source, target].into_iter().all(|player| {
            tx.state()
                .players
                .get(player)
                .is_some_and(|state| state.alive)
        })
    {
        return Err(EffectError::Illegal(
            "Retieji choice context changed".into(),
        ));
    }
    crate::command::play::validate_saved_sha_target(tx, source, card, target)?;
    if cancelled || selections.is_empty() {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled Retieji choice contains a card".into(),
            ));
        }
        return run_effects(
            tx,
            vec![
                EffectRequest::MakeCurrentShaTargetUnrespondable {
                    source,
                    card,
                    target,
                },
                EffectRequest::IncreaseCurrentShaTargetDamage {
                    source,
                    card,
                    target,
                    amount: 1,
                },
            ],
        )
        .map(|_| ());
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Retieji accepts at most one card".into(),
        ));
    };
    let selected = CardId::new(*selected);
    if !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&selected)
        || !tx
            .state()
            .cards
            .get(selected)
            .is_some_and(|card| crate::query::effective_card_suit(tx.state(), target, card) == suit)
    {
        return Err(EffectError::Illegal(
            "Retieji card is no longer eligible".into(),
        ));
    }
    let from = response_card_zone(tx, target, selected)?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, source)
        .ok_or_else(|| EffectError::EngineFault("Retieji recipient hand is missing".into()))?;
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

pub(crate) fn resolve_new_reluoyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, buff_skill_def, skill, card_count, cards @ ..] = payload else {
        return Err(EffectError::EngineFault(
            "bad New Reluoyi choice payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let card_count = usize::try_from(*card_count)
        .map_err(|_| EffectError::EngineFault("bad New Reluoyi card count".into()))?;
    if observer != player
        || cancelled
        || selections.len() != 1
        || cards.len() != card_count
        || !matches!(selections[0], 0 | 1)
    {
        return Err(EffectError::Illegal(
            "New Reluoyi choice context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveNewReluoyi {
            player,
            buff_skill_def: SkillDefId::new(*buff_skill_def),
            stage: 2,
            cards: cards.iter().copied().map(CardId::new).collect(),
            accepted: selections[0] == 1,
            skill: SkillId::new(*skill),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_xinyaoming(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Xinyaoming stage".into()))?;
    match stage {
        0 => {
            let option_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated Xinyaoming target payload".into())
            })? as usize;
            if payload.len() != 4 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Xinyaoming target payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Xinyaoming requires exactly one target".into(),
                ));
            };
            let target = PlayerId::new(*selected);
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .filter(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
                .map_or(u8::MAX, |instance| instance.xinyaoming_used_mask());
            if cancelled
                || observer != player
                || !payload[4..].contains(selected)
                || crate::query::xinyaoming_available_actions(tx.state(), player, target, used_mask)
                    .is_empty()
            {
                return Err(EffectError::Illegal(
                    "Xinyaoming target choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveXinyaoming {
                    player,
                    target,
                    action: 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Xinyaoming action payload".into())
            })? as usize;
            if payload.len() != 5 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Xinyaoming action payload".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Xinyaoming requires exactly one action".into(),
                ));
            };
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            let used_mask = tx
                .state()
                .skills
                .get(skill)
                .filter(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
                .map_or(u8::MAX, |instance| instance.xinyaoming_used_mask());
            let action = u8::try_from(*selected)
                .map_err(|_| EffectError::Illegal("bad Xinyaoming action token".into()))?;
            if cancelled
                || observer != player
                || !payload[5..].contains(selected)
                || !crate::query::xinyaoming_available_actions(
                    tx.state(),
                    player,
                    target,
                    used_mask,
                )
                .contains(&action)
            {
                return Err(EffectError::Illegal(
                    "Xinyaoming action choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveXinyaoming {
                    player,
                    target,
                    action,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Xinyaoming discard payload".into())
            })? as usize;
            if payload.len() != 5 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Xinyaoming discard payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let current = crate::query::discardable_target_cards(
                tx.state(),
                target,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            });
            if cancelled
                || observer != target
                || !skill_enabled
                || cards.is_empty()
                || cards.len() > 2
                || cards.iter().enumerate().any(|(index, card)| {
                    cards[..index].contains(card)
                        || !payload[5..].contains(&card.raw())
                        || !current.contains(card)
                })
            {
                return Err(EffectError::Illegal(
                    "Xinyaoming discard choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: target,
                        cards: cards.clone(),
                    },
                    EffectRequest::DrawCardsFromSkill {
                        player: target,
                        count: cards.len() as u32,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Xinyaoming choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_rehuaiyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Rehuaiyi stage".into()))?;
    match stage {
        0 => {
            let option_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated Rehuaiyi color payload".into())
            })? as usize;
            if payload.len() != 4 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Rehuaiyi color payload".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Rehuaiyi requires exactly one color".into(),
                ));
            };
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let color = u8::try_from(*selected)
                .map_err(|_| EffectError::Illegal("bad Rehuaiyi color token".into()))?;
            if cancelled
                || observer != player
                || !matches!(color, 0b01 | 0b10)
                || !payload[4..].contains(selected)
            {
                return Err(EffectError::Illegal(
                    "Rehuaiyi color choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveRehuaiyi {
                    player,
                    stage: 0,
                    color,
                    discarded_count: 0,
                    targets: Vec::new(),
                    index: 0,
                    gained_count: 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            let discarded_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated Rehuaiyi target payload".into())
            })?;
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Rehuaiyi target options".into())
            })? as usize;
            if payload.len() != 5 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Rehuaiyi target payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            if observer != player || (cancelled && !selections.is_empty()) {
                return Err(EffectError::Illegal(
                    "invalid Rehuaiyi target cancellation".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let max = (discarded_count as usize).min(option_count);
            let mut unique = std::collections::BTreeSet::new();
            if selections.is_empty()
                || selections.len() > max
                || !selections.iter().all(|selected| {
                    unique.insert(*selected)
                        && payload[5..].contains(selected)
                        && PlayerId::new(*selected) != player
                        && !crate::query::gainable_target_cards(
                            tx.state(),
                            player,
                            PlayerId::new(*selected),
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
            {
                return Err(EffectError::Illegal(
                    "Rehuaiyi target choice context changed".into(),
                ));
            }
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| selected.contains(&target.raw()))
                .collect();
            run_effects(
                tx,
                vec![EffectRequest::ResolveRehuaiyi {
                    player,
                    stage: 2,
                    color: 0,
                    discarded_count: 0,
                    targets,
                    index: 0,
                    gained_count: 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            let target_count = *payload
                .get(5)
                .ok_or_else(|| EffectError::EngineFault("truncated Rehuaiyi gain payload".into()))?
                as usize;
            let options_index = 6usize
                .checked_add(target_count)
                .ok_or_else(|| EffectError::EngineFault("Rehuaiyi payload overflow".into()))?;
            let option_count = *payload
                .get(options_index)
                .ok_or_else(|| EffectError::EngineFault("truncated Rehuaiyi card options".into()))?
                as usize;
            if payload.len() != options_index + 1 + option_count {
                return Err(EffectError::EngineFault("bad Rehuaiyi gain payload".into()));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Rehuaiyi requires exactly one target card".into(),
                ));
            };
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let index = payload[3] as usize;
            let gained_count = payload[4];
            let targets: Vec<_> = payload[6..options_index]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let target = targets.get(index).copied().ok_or_else(|| {
                EffectError::EngineFault("Rehuaiyi target index out of bounds".into())
            })?;
            let card = CardId::new(*selected);
            let current = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let skill_valid = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            });
            if cancelled
                || observer != player
                || !skill_valid
                || !payload[options_index + 1..].contains(selected)
                || !current.contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Rehuaiyi target-card choice context changed".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Rehuaiyi card disappeared".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Rehuaiyi owner hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to,
                        from: Some(from),
                    },
                    EffectRequest::ResolveRehuaiyi {
                        player,
                        stage: 2,
                        color: 0,
                        discarded_count: 0,
                        targets,
                        index: (index as u32).saturating_add(1),
                        gained_count: gained_count.saturating_add(1),
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Rehuaiyi choice stage".into(),
        )),
    }
}
