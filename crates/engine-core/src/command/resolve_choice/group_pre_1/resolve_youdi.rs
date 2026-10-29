//! Command operations for `resolve_chegu` and related helpers.

use super::*;

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
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    commit(tx, targets)
                }
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
