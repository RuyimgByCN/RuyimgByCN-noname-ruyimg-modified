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

pub(crate) fn resolve_twyuyan(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, source, sha, _skill] = payload else {
        return Err(EffectError::EngineFault("bad TW Yuyan payload".into()));
    };
    let player = PlayerId::new(*player);
    let source = PlayerId::new(*source);
    let sha = CardId::new(*sha);
    if observer != source {
        return Err(EffectError::Illegal("TW Yuyan chooser changed".into()));
    }
    if cancelled || selections.is_empty() {
        if !selections.is_empty()
            || !tx
                .state_mut()
                .resolution
                .exclude_saved_sha_target(sha, player)
        {
            return Err(EffectError::Illegal(
                "TW Yuyan Sha target is no longer cancellable".into(),
            ));
        }
        return Ok(());
    }
    let [card] = selections else {
        return Err(EffectError::Illegal(
            "choose at most one TW Yuyan gift".into(),
        ));
    };
    let card = CardId::new(*card);
    if !crate::effect::twyuyan_gift_cards(tx, player, source, sha).contains(&card) {
        return Err(EffectError::Illegal("TW Yuyan gift became illegal".into()));
    }
    let card_state = tx.state().cards.get(card).unwrap();
    let from = card_state.zone;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("TW Yuyan owner hand missing".into()))?;
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

pub(crate) fn resolve_twguoyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            if payload.len() != 9 || observer.raw() != payload[2] || cancelled {
                return Err(EffectError::Illegal("bad TW Guoyi branch choice".into()));
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal(
                    "TW Guoyi requires one branch choice".into(),
                ));
            };
            if *choice > 1 {
                return Err(EffectError::Illegal("unknown TW Guoyi branch".into()));
            }
            let mut commit = payload.to_vec();
            commit.push(*choice);
            let effects = crate::effect::commit_twguoyi_choice(tx, &commit)?;
            if *choice == 0 {
                return run_effects(tx, effects).map(|_| ());
            }
            let target = PlayerId::new(payload[2]);
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let need = (payload[8] as usize).min(options.len());
            if need == 0 {
                return Ok(());
            }
            let mut next = payload.to_vec();
            next[0] = 1;
            next.push(need as u32);
            next.extend(options.iter().map(|card| card.raw()));
            crate::effect::open_choice(
                tx,
                target,
                need as u8,
                need as u8,
                false,
                options.into_iter().map(|card| card.raw()).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwguoyi,
                next,
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 10 || observer.raw() != payload[2] || cancelled {
                return Err(EffectError::Illegal("bad TW Guoyi discard choice".into()));
            }
            let need = payload[9] as usize;
            let options = &payload[10..];
            let mut unique = std::collections::BTreeSet::new();
            if selections.len() != need
                || selections
                    .iter()
                    .any(|card| !options.contains(card) || !unique.insert(*card))
            {
                return Err(EffectError::Illegal(
                    "TW Guoyi discard selection changed".into(),
                ));
            }
            let target = PlayerId::new(payload[2]);
            let legal = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards.iter().any(|card| !legal.contains(card)) {
                return Err(EffectError::Illegal(
                    "TW Guoyi selected card is no longer discardable".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Guoyi choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twchuhai(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            if payload.len() < 6 || observer.raw() != payload[2] || cancelled {
                return Err(EffectError::Illegal("bad TW Chuhai gift choice".into()));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "TW Chuhai requires exactly one gift".into(),
                ));
            };
            if !payload[5..].contains(selected) {
                return Err(EffectError::Illegal(
                    "TW Chuhai gift selection changed".into(),
                ));
            }
            let owner = PlayerId::new(payload[1]);
            let giver = PlayerId::new(payload[2]);
            let card = CardId::new(*selected);
            let Some(from) = tx.state().cards.get(card).and_then(|card_state| {
                (card_state.owner == giver
                    && [ZoneKind::Hand, ZoneKind::Equip]
                        .into_iter()
                        .any(|kind| tx.state().zones.find(kind, giver) == Some(card_state.zone)))
                .then_some(card_state.zone)
            }) else {
                return Err(EffectError::Illegal(
                    "TW Chuhai gift is no longer in the giver's hand/equipment".into(),
                ));
            };
            let Some(to) = tx.state().zones.find(ZoneKind::Hand, owner) else {
                return Err(EffectError::EngineFault(
                    "TW Chuhai owner hand zone missing".into(),
                ));
            };
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwchuhai {
                    player: owner,
                    action: 0,
                    stage: 1,
                    index: payload[4],
                    event_cards: Vec::new(),
                    skill: SkillId::new(payload[3]),
                });
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
        Some(1) => {
            if payload.len() < 5 || observer.raw() != payload[1] || cancelled {
                return Err(EffectError::Illegal("bad TW Chuhai discard choice".into()));
            }
            let count = payload[3] as usize;
            if payload.len() != 4 + count {
                return Err(EffectError::EngineFault(
                    "bad TW Chuhai discard payload".into(),
                ));
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "TW Chuhai requires exactly one discard".into(),
                ));
            };
            if !payload[4..].contains(selected) {
                return Err(EffectError::Illegal(
                    "TW Chuhai discard selection changed".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let card = CardId::new(*selected);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if !tx
                .state()
                .cards
                .get(card)
                .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
            {
                return Err(EffectError::Illegal(
                    "TW Chuhai discard is no longer in hand".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Chuhai choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twyizhu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            if payload.len() < 5 || observer.raw() != payload[1] || cancelled {
                return Err(EffectError::Illegal("bad TW Yizhu plant choice".into()));
            }
            let need = payload[3] as usize;
            let options = &payload[4..];
            let mut unique = std::collections::BTreeSet::new();
            if selections.len() != need
                || selections
                    .iter()
                    .any(|card| !options.contains(card) || !unique.insert(*card))
            {
                return Err(EffectError::Illegal(
                    "TW Yizhu plant selection changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwyizhu {
                    player: PlayerId::new(payload[1]),
                    source: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 0,
                    stage: 2,
                    event_cards: selections.iter().copied().map(CardId::new).collect(),
                    skill: SkillId::new(payload[2]),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 8 || observer.raw() != payload[1] {
                return Err(EffectError::Illegal("bad TW Yizhu target choice".into()));
            }
            let player = PlayerId::new(payload[1]);
            let source = PlayerId::new(payload[2]);
            let card = CardId::new(payload[3]);
            let skill = SkillId::new(payload[4]);
            let original = PlayerId::new(payload[5]);
            let count = payload[6] as usize;
            if payload.len() != 7 + count {
                return Err(EffectError::EngineFault(
                    "bad TW Yizhu tracked-card payload".into(),
                ));
            }
            let marked: Vec<_> = payload[7..].iter().copied().map(CardId::new).collect();
            if cancelled {
                return crate::effect::finish_twyizhu_used_cards(tx, player, &marked, skill)
                    .map(|_| ());
            }
            let (add_targets, _) =
                crate::command::play::current_card_target_adjustments(tx, source, card)?;
            match selections {
                [target] => {
                    let target = PlayerId::new(*target);
                    if target == original || !add_targets.contains(&target) {
                        return Err(EffectError::Illegal(
                            "TW Yizhu additional target changed".into(),
                        ));
                    }
                    crate::command::play::adjust_saved_card_target(tx, source, card, target, true)?;
                }
                [first, second] => {
                    let first = PlayerId::new(*first);
                    let second = PlayerId::new(*second);
                    let target = if first == original {
                        second
                    } else if second == original {
                        first
                    } else {
                        return Err(EffectError::Illegal(
                            "TW Yizhu replacement must include the original target".into(),
                        ));
                    };
                    if !add_targets.contains(&target) {
                        return Err(EffectError::Illegal(
                            "TW Yizhu replacement target changed".into(),
                        ));
                    }
                    crate::command::play::adjust_saved_card_target(tx, source, card, target, true)?;
                    crate::command::play::adjust_saved_card_target(
                        tx, source, card, original, false,
                    )?;
                }
                _ => {
                    return Err(EffectError::Illegal(
                        "TW Yizhu requires one added target or a replacement pair".into(),
                    ));
                }
            }
            crate::effect::finish_twyizhu_used_cards(tx, player, &marked, skill).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Yizhu choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twxiawei(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(2) => {
            let [_, player, skill, wangxing_skill_def] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Xiawei option payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            if observer != player {
                return Err(EffectError::Illegal("wrong TW Xiawei chooser".into()));
            }
            if cancelled {
                return Ok(());
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal(
                    "TW Xiawei requires one value from one through four".into(),
                ));
            };
            if !(1..=4).contains(choice) {
                return Err(EffectError::Illegal("bad TW Xiawei value".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwxiawei {
                    player,
                    wangxing_skill_def: *wangxing_skill_def,
                    action: 2,
                    stage: *choice as u8,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        Some(4) => {
            if payload.len() < 6 || observer.raw() != payload[1] {
                return Err(EffectError::EngineFault(
                    "bad TW Wangxing discard payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let mark = engine_api::MarkDefId::new(payload[3]);
            let count = payload[4] as usize;
            let options = &payload[5..];
            let mut effects = vec![
                EffectRequest::RemoveSkill { skill },
                EffectRequest::RemoveMark {
                    player,
                    mark,
                    amount: count as u32,
                },
            ];
            if cancelled {
                effects.push(EffectRequest::ChangeMaxHp { player, delta: -1 });
            } else {
                let mut unique = std::collections::BTreeSet::new();
                if selections.len() != count
                    || selections
                        .iter()
                        .any(|card| !options.contains(card) || !unique.insert(*card))
                {
                    return Err(EffectError::Illegal(
                        "TW Wangxing discard selection changed".into(),
                    ));
                }
                let legal = crate::query::discardable_hand_equip_cards(tx.state(), player);
                let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
                if cards.iter().any(|card| !legal.contains(card)) {
                    return Err(EffectError::Illegal(
                        "TW Wangxing selected card is no longer discardable".into(),
                    ));
                }
                effects.push(EffectRequest::DiscardCards { player, cards });
            }
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Xiawei choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twbianfuren(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            if payload.len() < 5 || observer.raw() != payload[1] {
                return Err(EffectError::EngineFault(
                    "bad TW Yuejian card payload".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let mut unique = std::collections::BTreeSet::new();
            if selections.is_empty()
                || selections
                    .iter()
                    .any(|card| !payload[4..].contains(card) || !unique.insert(*card))
            {
                return Err(EffectError::Illegal(
                    "invalid TW Yuejian selected cards".into(),
                ));
            }
            let mut next = vec![
                1,
                payload[1],
                payload[2],
                payload[3],
                selections.len() as u32,
            ];
            next.extend(selections);
            open_choice(
                tx,
                observer,
                0,
                selections.len().min(u8::MAX as usize) as u8,
                false,
                selections.to_vec(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwbianfuren,
                next,
            )?;
            Ok(())
        }
        Some(1) => {
            if payload.len() < 5 || observer.raw() != payload[1] || cancelled {
                return Err(EffectError::Illegal(
                    "TW Yuejian partition cannot be cancelled".into(),
                ));
            }
            let count = payload[4] as usize;
            if payload.len() != 5 + count {
                return Err(EffectError::EngineFault(
                    "bad TW Yuejian partition payload".into(),
                ));
            }
            let cards: Vec<_> = payload[5..].iter().copied().map(CardId::new).collect();
            let mut unique = std::collections::BTreeSet::new();
            let top: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if top
                .iter()
                .any(|card| !cards.contains(card) || !unique.insert(*card))
            {
                return Err(EffectError::Illegal(
                    "invalid TW Yuejian top partition".into(),
                ));
            }
            crate::effect::finish_twyuejian_cards(tx, observer, cards, top, payload[2]).map(|_| ())
        }
        Some(3) => {
            if payload.len() < 5 || observer.raw() != payload[1] || cancelled {
                return Err(EffectError::Illegal(
                    "TW Wanwei card target cannot be cancelled".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let card = CardId::new(payload[2]);
            let option_count = payload[4] as usize;
            if payload.len() != 5 + option_count {
                return Err(EffectError::EngineFault(
                    "bad TW Wanwei target payload".into(),
                ));
            }
            let crate::command::play::SkillCardTargetSpec::Select { min, max, options } =
                crate::command::play::skill_card_target_spec(tx, player, card)?
            else {
                return Err(EffectError::Illegal(
                    "TW Wanwei target contract changed".into(),
                ));
            };
            let mut unique = std::collections::BTreeSet::new();
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            if targets.len() < usize::from(min)
                || targets.len() > usize::from(max)
                || targets.iter().any(|target| {
                    !unique.insert(*target)
                        || !options.contains(target)
                        || !payload[5..].contains(&target.raw())
                })
            {
                return Err(EffectError::Illegal(
                    "invalid TW Wanwei card targets".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::UseCard {
                    player,
                    card,
                    targets,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Bianfuren choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twfeiyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(1) => {
            let [_, player, _skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Shengxi choice payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            if observer != player {
                return Err(EffectError::Illegal("wrong TW Shengxi chooser".into()));
            }
            if cancelled {
                return Ok(());
            }
            let [kind] = selections else {
                return Err(EffectError::Illegal(
                    "TW Shengxi requires one zhinang kind".into(),
                ));
            };
            if ![
                CardKind::TrickWuzhong.tag() as u32,
                CardKind::TrickGuohe.tag() as u32,
                CardKind::TrickWuxie.tag() as u32,
            ]
            .contains(kind)
            {
                return Err(EffectError::Illegal("invalid zhinang kind".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::GainCardByKind {
                        player,
                        kind: *kind as u8,
                    },
                    EffectRequest::DrawCards { player, count: 1 },
                ],
            )
            .map(|_| ())
        }
        Some(2) => {
            if payload.len() < 4 || observer.raw() != payload[1] {
                return Err(EffectError::EngineFault(
                    "bad TW Kuanji card payload".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "TW Kuanji requires one discard card".into(),
                ));
            };
            if !payload[3..].contains(card) {
                return Err(EffectError::Illegal("invalid TW Kuanji card".into()));
            }
            let player = PlayerId::new(payload[1]);
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .collect();
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.iter().map(|target| target.raw()).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwfeiyi,
                vec![3, player.raw(), payload[2], *card],
            )?;
            Ok(())
        }
        Some(3) => {
            let [_, player, _skill, card] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Kuanji target payload".into(),
                ));
            };
            let player = PlayerId::new(*player);
            if cancelled || observer != player {
                return Err(EffectError::Illegal(
                    "TW Kuanji target cannot be cancelled".into(),
                ));
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("TW Kuanji requires one target".into()));
            };
            let target = PlayerId::new(*target);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid TW Kuanji target".into()));
            }
            let card = CardId::new(*card);
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
            if tx
                .state()
                .cards
                .get(card)
                .is_none_or(|state| state.zone != discard)
            {
                return Err(EffectError::Illegal(
                    "TW Kuanji card left discard pile".into(),
                ));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("target hand missing".into()))?;
            crate::effect::move_cards(tx, vec![card], hand, Some(discard)).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Feiyi choice stage".into(),
        )),
    }
}
