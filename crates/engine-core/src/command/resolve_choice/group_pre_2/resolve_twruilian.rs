//! Command operations for `resolve_twruilian` and related helpers.

use super::*;

pub(crate) fn resolve_twruilian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, target, skill, mask] = payload else {
        return Err(EffectError::EngineFault(
            "truncated TW Ruilian continuation".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = SkillId::new(*skill);
    if observer != player {
        return Err(EffectError::Illegal(
            "TW Ruilian choice observer changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "TW Ruilian requires exactly one selection".into(),
        ));
    };
    match stage {
        0 => {
            let target = PlayerId::new(*selected);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
                || !tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
            {
                return Err(EffectError::Illegal(
                    "TW Ruilian target choice context changed".into(),
                ));
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Ruilian skill")
                .set_twruilian_target(target);
            Ok(())
        }
        1 => {
            let category = u8::try_from(*selected)
                .map_err(|_| EffectError::Illegal("bad TW Ruilian category".into()))?;
            let bit = match category {
                1 => 1,
                2 => 2,
                3 => 4,
                _ => 0,
            };
            if bit == 0 || *mask & bit == 0 {
                return Err(EffectError::Illegal(
                    "TW Ruilian category is unavailable".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwruilian {
                    player,
                    target: PlayerId::new(*target),
                    event_cards: Vec::new(),
                    action: 3,
                    category,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Ruilian continuation stage".into(),
        )),
    }
}

pub(crate) fn resolve_twjianwei(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing TW Jianwei stage".into()))?;
    match stage {
        0 => {
            let [_, player, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Jianwei target payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            if observer != player {
                return Err(EffectError::Illegal("TW Jianwei chooser changed".into()));
            }
            if cancelled {
                return Ok(());
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("choose one TW Jianwei target".into()));
            };
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwjianwei {
                    player,
                    actor: player,
                    target: PlayerId::new(*target),
                    card: CardId::NONE,
                    winner: PlayerId::NONE,
                    stage: 2,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        1 => {
            let [_, player, actor, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Jianwei invite payload".into(),
                ));
            };
            let actor = PlayerId::new(*actor);
            if observer != actor {
                return Err(EffectError::Illegal("TW Jianwei inviter changed".into()));
            }
            if cancelled {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("confirm TW Jianwei comparison".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwjianwei {
                    player: PlayerId::new(*player),
                    actor,
                    target: PlayerId::new(*player),
                    card: CardId::NONE,
                    winner: PlayerId::NONE,
                    stage: 2,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        2 => {
            let [_, _player, actor, target, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Jianwei card payload".into(),
                ));
            };
            let actor = PlayerId::new(*actor);
            if observer != actor || cancelled {
                return Err(EffectError::Illegal(
                    "TW Jianwei card chooser changed".into(),
                ));
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "choose one TW Jianwei hand card".into(),
                ));
            };
            let skill = SkillId::new(*skill);
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("TW Jianwei skill disappeared".into()))?
                .set_twjianwei_compare(actor, PlayerId::new(*target));
            run_effects(
                tx,
                vec![EffectRequest::ChooseCompareSelectedHandCard {
                    player: actor,
                    target: PlayerId::new(*target),
                    card: CardId::new(*card),
                    win_skill_def: SkillDefId::NONE,
                    lose_skill_def: SkillDefId::NONE,
                    skill,
                }],
            )
            .map(|_| ())
        }
        3 => {
            let [_, player, loser, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Jianwei gain payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let loser = PlayerId::new(*loser);
            if observer != player || cancelled || selections.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Jianwei gain choice changed".into(),
                ));
            }
            let allowed = crate::query::gainable_target_cards(
                tx.state(),
                player,
                loser,
                rule_api::TargetCardScope::HandEquipJudge,
            );
            let mut zones = std::collections::BTreeSet::new();
            let cards: Vec<_> = selections
                .iter()
                .copied()
                .map(CardId::new)
                .inspect(|card| {
                    if let Some(card) = tx.state().cards.get(*card) {
                        zones.insert(card.zone);
                    }
                })
                .collect();
            if cards.len() != zones.len()
                || cards.iter().any(|card| !allowed.contains(card))
                || !tx
                    .state()
                    .skills
                    .get(SkillId::new(*skill))
                    .is_some_and(|instance| {
                        instance.owner == player && tx.state().skill_effectively_enabled(instance)
                    })
            {
                return Err(EffectError::Illegal(
                    "invalid TW Jianwei region cards".into(),
                ));
            }
            let to = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
            let effects = cards
                .into_iter()
                .filter_map(|card| {
                    tx.state()
                        .cards
                        .get(card)
                        .map(|state| EffectRequest::GiveCards {
                            cards: vec![card],
                            to,
                            from: state.zone,
                        })
                })
                .collect();
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Jianwei choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twtanfeng(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing TW Tanfeng stage".into()))?;
    match stage {
        0 => {
            let [_, player, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Tanfeng target payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            if observer != player {
                return Err(EffectError::Illegal("TW Tanfeng chooser changed".into()));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("choose one TW Tanfeng target".into()));
            };
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwtanfeng {
                    player,
                    target: PlayerId::new(*target),
                    card: CardId::NONE,
                    stage: 1,
                    phase: 0,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        1 => {
            let [_, player, target, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Tanfeng discard payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose one TW Tanfeng target card".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardTargetCards {
                        actor: player,
                        target,
                        cards: vec![card],
                        scope: rule_api::TargetCardScope::HandEquipJudge,
                    },
                    EffectRequest::ResolveTwtanfeng {
                        player,
                        target,
                        card: CardId::NONE,
                        stage: 2,
                        phase: 0,
                        skill: SkillId::new(*skill),
                    },
                ],
            )
            .map(|_| ())
        }
        2 => {
            let [_, player, target, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Tanfeng Sha payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            if observer != target {
                return Err(EffectError::Illegal(
                    "TW Tanfeng Sha chooser changed".into(),
                ));
            }
            let stage = if cancelled || selections.is_empty() {
                4
            } else if selections.len() == 1 {
                3
            } else {
                return Err(EffectError::Illegal(
                    "choose at most one TW Tanfeng Sha source".into(),
                ));
            };
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwtanfeng {
                    player,
                    target,
                    card: selections
                        .first()
                        .copied()
                        .map_or(CardId::NONE, CardId::new),
                    stage,
                    phase: 0,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        3 => {
            let [_, player, target, card, _skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Tanfeng extra-target payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            if observer != target || cancelled && !selections.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Tanfeng extra-target choice changed".into(),
                ));
            }
            let mut targets = vec![player];
            targets.extend(selections.iter().copied().map(PlayerId::new));
            crate::command::play::play_twtanfeng_sha_source(
                tx,
                target,
                player,
                CardId::new(*card),
                &targets,
            )
        }
        4 => {
            let [_, player, target, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Tanfeng phase payload".into(),
                ));
            };
            let target = PlayerId::new(*target);
            if observer != target || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("choose one TW Tanfeng phase".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwtanfeng {
                    player: PlayerId::new(*player),
                    target,
                    card: CardId::NONE,
                    stage: 6,
                    phase: selections[0] as u8,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Tanfeng choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twzhibian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing TW Zhibian stage".into()))?;
    match stage {
        0 => {
            let [_, player, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhibian target payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            if observer != player {
                return Err(EffectError::Illegal("TW Zhibian chooser changed".into()));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("choose one TW Zhibian target".into()));
            };
            let cards = tx.state().hand_cards(player);
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwzhibian,
                vec![4, player.raw(), *target, *skill],
            )
            .map(|_| ())
        }
        1 => {
            let [_, player, target, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhibian option payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            if observer != player {
                return Err(EffectError::Illegal(
                    "TW Zhibian option chooser changed".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [action] = selections else {
                return Err(EffectError::Illegal("choose one TW Zhibian option".into()));
            };
            match *action {
                1 => {
                    let cards = crate::effect::twzhibian_movable_cards(tx, player, target);
                    crate::effect::open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        cards.into_iter().map(CardId::raw).collect(),
                        engine_api::ChoiceKind::SelectCards,
                        ContinuationKind::ResolveTwzhibian,
                        vec![3, player.raw(), target.raw(), *skill, 1],
                    )
                    .map(|_| ())
                }
                2 => run_effects(
                    tx,
                    vec![EffectRequest::RecoverHp {
                        source: player,
                        player,
                        amount: 1,
                    }],
                )
                .map(|_| ()),
                3 => {
                    let cards: Vec<_> =
                        crate::query::discardable_hand_equip_cards(tx.state(), player)
                            .into_iter()
                            .filter(|card| {
                                tx.state().cards.get(*card).is_some_and(|card| {
                                    !crate::query::effective_card_kind(tx.state(), player, card)
                                        .is_basic()
                                })
                            })
                            .collect();
                    crate::effect::open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        cards.into_iter().map(CardId::raw).collect(),
                        engine_api::ChoiceKind::SelectCards,
                        ContinuationKind::ResolveTwzhibian,
                        vec![2, player.raw(), target.raw(), *skill],
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal("unknown TW Zhibian option".into())),
            }
        }
        2 => {
            let [_, player, target, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhibian cost payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("choose one TW Zhibian cost".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![CardId::new(selections[0])],
                    },
                    EffectRequest::ResolveTwzhibian {
                        player,
                        target: PlayerId::new(*target),
                        winner: player,
                        stage: 2,
                        skill: SkillId::new(*skill),
                    },
                ],
            )
            .map(|_| ())
        }
        3 => {
            let [_, player, target, _skill, action] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhibian move payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose one TW Zhibian movable card".into(),
                ));
            }
            let mut effects = crate::effect::twzhibian_move_effects(
                tx,
                player,
                target,
                CardId::new(selections[0]),
            )?;
            if *action == 3 {
                effects.push(EffectRequest::RecoverHp {
                    source: player,
                    player,
                    amount: 1,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        4 => {
            let [_, player, target, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhibian compare payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose one TW Zhibian compare card".into(),
                ));
            }
            let skill = SkillId::new(*skill);
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("TW Zhibian skill disappeared".into()))?
                .set_twzhibian_compare(target);
            run_effects(
                tx,
                vec![EffectRequest::ChooseCompareSelectedHandCard {
                    player,
                    target,
                    card: CardId::new(selections[0]),
                    win_skill_def: SkillDefId::NONE,
                    lose_skill_def: SkillDefId::NONE,
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Zhibian choice stage".into(),
        )),
    }
}
