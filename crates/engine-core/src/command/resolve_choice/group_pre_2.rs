//! Choice-continuation handlers (pre 2).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

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
            let (add_targets, _) = crate::command::play::current_card_target_adjustments(tx, source, card)?;
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
                    crate::command::play::adjust_saved_card_target(tx, source, card, original, false)?;
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

pub(crate) fn resolve_twchenzhen(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            let [_, player, helper, skill] = payload else {
                return Err(EffectError::EngineFault("bad TW Muyue kind payload".into()));
            };
            if cancelled || observer.raw() != *player {
                return Err(EffectError::Illegal(
                    "TW Muyue kind cannot be cancelled".into(),
                ));
            }
            let [kind] = selections else {
                return Err(EffectError::Illegal(
                    "TW Muyue requires one card kind".into(),
                ));
            };
            let kind = u8::try_from(*kind)
                .ok()
                .and_then(CardKind::from_tag)
                .filter(|kind| kind.is_basic() || kind.is_trick())
                .ok_or_else(|| EffectError::Illegal("invalid TW Muyue kind".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwchenzhen {
                    player: PlayerId::new(*player),
                    target: PlayerId::NONE,
                    helper_skill_def: *helper,
                    record_skill_def: 0,
                    action: 0,
                    stage: 1,
                    card_kind: kind.tag(),
                    card: CardId::NONE,
                    branch: 0,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 6 || cancelled || observer.raw() != payload[1] {
                return Err(EffectError::Illegal("bad TW Muyue discard choice".into()));
            }
            let [card] = selections else {
                return Err(EffectError::Illegal("TW Muyue requires one discard".into()));
            };
            if !payload[5..].contains(card) {
                return Err(EffectError::Illegal("invalid TW Muyue discard".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwchenzhen {
                    player: PlayerId::new(payload[1]),
                    target: PlayerId::NONE,
                    helper_skill_def: payload[2],
                    record_skill_def: 0,
                    action: 0,
                    stage: 2,
                    card_kind: payload[4] as u8,
                    card: CardId::new(*card),
                    branch: 0,
                    skill: SkillId::new(payload[3]),
                }],
            )
            .map(|_| ())
        }
        Some(3) => {
            let [_, player, helper, _skill, kind, matching] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Muyue target payload".into(),
                ));
            };
            if cancelled || observer.raw() != *player {
                return Err(EffectError::Illegal(
                    "TW Muyue target cannot be cancelled".into(),
                ));
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("TW Muyue requires one target".into()));
            };
            let target = PlayerId::new(*target);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid TW Muyue target".into()));
            }
            let mut effects = vec![EffectRequest::GainCardByKind {
                player: target,
                kind: *kind as u8,
            }];
            if *matching != 0 {
                effects.push(EffectRequest::AddSkill {
                    player: PlayerId::new(*player),
                    skill_def: SkillDefId::new(*helper),
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        Some(10) => {
            let [_, owner, effect, record, _skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Chayi target payload".into(),
                ));
            };
            if observer.raw() != *owner {
                return Err(EffectError::Illegal("wrong TW Chayi chooser".into()));
            }
            if cancelled {
                return Ok(());
            }
            let [target] = selections else {
                return Err(EffectError::Illegal("TW Chayi requires one target".into()));
            };
            let target = PlayerId::new(*target);
            if target.raw() == *owner
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid TW Chayi target".into()));
            }
            if tx.state().hand_cards(target).is_empty() {
                return crate::effect::apply_twchayi_initial(tx, target, 1, *effect, *record)
                    .map(|_| ());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwchenzhen,
                vec![11, *owner, target.raw(), *effect, *record],
            )?;
            Ok(())
        }
        Some(11) => {
            let [_, _owner, target, effect, record] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Chayi branch payload".into(),
                ));
            };
            if cancelled || observer.raw() != *target {
                return Err(EffectError::Illegal(
                    "TW Chayi branch cannot be cancelled".into(),
                ));
            }
            let [branch @ 0..=1] = selections else {
                return Err(EffectError::Illegal("invalid TW Chayi branch".into()));
            };
            crate::effect::apply_twchayi_initial(
                tx,
                PlayerId::new(*target),
                *branch as u8,
                *effect,
                *record,
            )
            .map(|_| ())
        }
        Some(12) => {
            if payload.len() < 6 || cancelled || observer.raw() != payload[1] {
                return Err(EffectError::Illegal("bad TW Chayi discard payload".into()));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let mark = engine_api::MarkDefId::new(payload[3]);
            let need = payload[4] as usize;
            let mut unique = std::collections::BTreeSet::new();
            if selections.len() != need
                || selections
                    .iter()
                    .any(|card| !payload[5..].contains(card) || !unique.insert(*card))
            {
                return Err(EffectError::Illegal("invalid TW Chayi discard".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::RemoveSkill { skill },
                    EffectRequest::RemoveMark {
                        player,
                        mark,
                        amount: need as u32,
                    },
                    EffectRequest::DiscardCards {
                        player,
                        cards: selections.iter().copied().map(CardId::new).collect(),
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Chenzhen choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twqingxi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || cancelled || observer.raw() != payload[2] {
        return Err(EffectError::Illegal("bad TW Qingxi choice".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    let skill = SkillId::new(payload[4]);
    match payload[0] {
        0 => {
            let [branch] = selections else {
                return Err(EffectError::Illegal("TW Qingxi requires one branch".into()));
            };
            if *branch > 1 {
                return Err(EffectError::Illegal("invalid TW Qingxi branch".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwqingxi {
                    player,
                    target,
                    card,
                    stage: if *branch == 0 { 1 } else { 2 },
                    count: 0,
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault(
                    "bad TW Qingxi discard payload".into(),
                ));
            }
            let need = usize::from(payload[5].min(u8::MAX.into()) as u8)
                .min(payload.len().saturating_sub(6));
            if selections.len() != need
                || selections.iter().any(|card| !payload[6..].contains(card))
            {
                return Err(EffectError::Illegal(
                    "invalid TW Qingxi equipment choice".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwqingxi {
                    player,
                    target,
                    card,
                    stage: 6,
                    count: payload[5] as u8,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Qingxi choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twzaoli(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 || cancelled || observer.raw() != payload[0] {
        return Err(EffectError::Illegal("bad TW Zaoli choice".into()));
    }
    let mandatory_len = payload[2] as usize;
    let split = 3usize
        .checked_add(mandatory_len)
        .filter(|split| *split <= payload.len())
        .ok_or_else(|| EffectError::EngineFault("bad TW Zaoli payload".into()))?;
    if selections.is_empty()
        || selections
            .iter()
            .any(|card| !payload[split..].contains(card))
    {
        return Err(EffectError::Illegal(
            "invalid TW Zaoli discard choice".into(),
        ));
    }
    let cards = payload[3..split]
        .iter()
        .chain(selections)
        .copied()
        .map(CardId::new)
        .collect();
    run_effects(
        tx,
        vec![EffectRequest::ResolveTwzaoli {
            player: PlayerId::new(payload[0]),
            stage: 1,
            cards,
            slots: Vec::new(),
            count: 0,
            skill: SkillId::new(payload[1]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_twzhengjian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled {
        return Err(EffectError::Illegal(
            "TW Zhengjian choice cannot be cancelled".into(),
        ));
    }
    match payload.first().copied() {
        Some(0) => {
            let [_, player, _helper, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhengjian setup payload".into(),
                ));
            };
            let [branch @ 0..=1] = selections else {
                return Err(EffectError::Illegal("invalid TW Zhengjian branch".into()));
            };
            if observer.raw() != *player {
                return Err(EffectError::Illegal("wrong TW Zhengjian chooser".into()));
            }
            tx.state_mut()
                .skills
                .get_mut(SkillId::new(*skill))
                .ok_or_else(|| EffectError::Illegal("TW Zhengjian skill disappeared".into()))?
                .set_twzhengjian_state(*branch as u8, false, Vec::new());
            Ok(())
        }
        Some(1) => {
            if payload.len() < 6 || observer.raw() != payload[2] {
                return Err(EffectError::Illegal("bad TW Zhengjian give choice".into()));
            }
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "TW Zhengjian requires one card".into(),
                ));
            };
            if !payload[5..].contains(card) {
                return Err(EffectError::Illegal("invalid TW Zhengjian card".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwzhengjian {
                    player: PlayerId::new(payload[1]),
                    target: PlayerId::new(payload[2]),
                    helper_skill_def: payload[3],
                    action: 1,
                    stage: 1,
                    card: CardId::new(*card),
                    branch: 0,
                    skill: SkillId::new(payload[4]),
                }],
            )
            .map(|_| ())
        }
        Some(2) => {
            let [_, player, target, helper, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhengjian damage payload".into(),
                ));
            };
            let [choice @ 0..=3] = selections else {
                return Err(EffectError::Illegal(
                    "invalid TW Zhengjian damage choice".into(),
                ));
            };
            if observer.raw() != *player {
                return Err(EffectError::Illegal("wrong TW Zhengjian chooser".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwzhengjian {
                    player: PlayerId::new(*player),
                    target: PlayerId::new(*target),
                    helper_skill_def: *helper,
                    action: 1,
                    stage: 3,
                    card: CardId::NONE,
                    branch: *choice as u8,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        Some(3) => {
            let [_, player, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhengjian switch payload".into(),
                ));
            };
            let [choice @ 0..=1] = selections else {
                return Err(EffectError::Illegal("invalid TW Zhengjian switch".into()));
            };
            if observer.raw() != *player {
                return Err(EffectError::Illegal("wrong TW Zhengjian chooser".into()));
            }
            if *choice != 0 {
                let instance = tx
                    .state_mut()
                    .skills
                    .get_mut(SkillId::new(*skill))
                    .ok_or_else(|| EffectError::Illegal("TW Zhengjian skill disappeared".into()))?;
                let (branch, awakened, contributors) = instance.twzhengjian_state();
                instance.set_twzhengjian_state(1 - branch.unwrap_or(0), awakened, contributors);
            }
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Zhengjian choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twtianyu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled {
        return Err(EffectError::Illegal(
            "TW Tianyu choice cannot be cancelled".into(),
        ));
    }
    match payload.first().copied() {
        Some(0) => {
            let [_, player, target, helper, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhenxi branch payload".into(),
                ));
            };
            let [branch @ 1..=3] = selections else {
                return Err(EffectError::Illegal("invalid TW Zhenxi branch".into()));
            };
            if observer.raw() != *player {
                return Err(EffectError::Illegal("wrong TW Zhenxi chooser".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwtianyu {
                    player: PlayerId::new(*player),
                    target: PlayerId::new(*target),
                    helper_skill_def: *helper,
                    action: 0,
                    stage: 1,
                    card: CardId::NONE,
                    branch: *branch as u8,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 7 || observer.raw() != payload[1] || selections.is_empty() {
                return Err(EffectError::Illegal("bad TW Zhenxi discard choice".into()));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[4]);
            let branch = payload[5] as u8;
            let legal = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::Hand,
            );
            if selections
                .iter()
                .any(|card| !payload[6..].contains(card) || !legal.contains(&CardId::new(*card)))
            {
                return Err(EffectError::Illegal(
                    "invalid TW Zhenxi discard card".into(),
                ));
            }
            if branch & 2 != 0 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveTwtianyu {
                        player,
                        target,
                        helper_skill_def: payload[3],
                        action: 0,
                        stage: 2,
                        card: CardId::NONE,
                        branch,
                        skill,
                    },
                );
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
        Some(2) => {
            let [_, player, source, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhenxi field-card payload".into(),
                ));
            };
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "TW Zhenxi requires one field card".into(),
                ));
            };
            if observer.raw() != *player
                || !crate::query::movable_field_cards_from(tx.state(), PlayerId::new(*source))
                    .contains(&CardId::new(*card))
            {
                return Err(EffectError::Illegal("invalid TW Zhenxi field card".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwtianyu {
                    player: PlayerId::new(*player),
                    target: PlayerId::new(*source),
                    helper_skill_def: 0,
                    action: 0,
                    stage: 3,
                    card: CardId::new(*card),
                    branch: 2,
                    skill: SkillId::new(*skill),
                }],
            )
            .map(|_| ())
        }
        Some(3) => {
            let [_, player, source, card, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad TW Zhenxi destination payload".into(),
                ));
            };
            let [destination] = selections else {
                return Err(EffectError::Illegal(
                    "TW Zhenxi requires one destination".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let source = PlayerId::new(*source);
            let card = CardId::new(*card);
            let destination = PlayerId::new(*destination);
            if observer != player
                || !tx
                    .state()
                    .skills
                    .get(SkillId::new(*skill))
                    .is_some_and(|instance| {
                        instance.owner == player && tx.state().skill_effectively_enabled(instance)
                    })
                || !crate::query::movable_field_cards_from(tx.state(), source).contains(&card)
                || !crate::query::field_card_move_destinations(tx.state(), card)
                    .contains(&destination)
            {
                return Err(EffectError::Illegal("invalid TW Zhenxi destination".into()));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("TW Zhenxi field card missing".into()))?;
            let kind = tx
                .state()
                .zones
                .get(from)
                .map(|zone| zone.kind)
                .ok_or_else(|| EffectError::Illegal("TW Zhenxi field zone missing".into()))?;
            let to =
                tx.state().zones.find(kind, destination).ok_or_else(|| {
                    EffectError::Illegal("TW Zhenxi destination zone missing".into())
                })?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to,
                    from: Some(from),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad TW Tianyu choice payload".into(),
        )),
    }
}

pub(crate) fn resolve_twwujing(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    if cancelled {
        return Ok(());
    }
    match payload.first().copied() {
        Some(0) => {
            if payload.len() != 3 || observer != player || selections.is_empty() {
                return Err(EffectError::Illegal("bad TW Congji card choice".into()));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards.iter().any(|card| {
                tx.state().cards.get(*card).is_none_or(|card_state| {
                    card_state.zone != discard
                        || crate::query::effective_card_color_mask(tx.state(), player, *card)
                            != 0b01
                })
            }) {
                return Err(EffectError::Illegal("TW Congji card left discard".into()));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(());
            }
            let mut next = vec![1, player.raw(), payload[2], cards.len() as u32];
            next.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwwujing,
                next,
            )?;
            Ok(())
        }
        Some(1) => {
            if payload.len() < 5 || observer != player {
                return Err(EffectError::Illegal("bad TW Congji target choice".into()));
            }
            let [target] = selections else {
                return Err(EffectError::Illegal(
                    "TW Congji requires one recipient".into(),
                ));
            };
            let target = PlayerId::new(*target);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid TW Congji recipient".into()));
            }
            let count = payload[3] as usize;
            let cards: Vec<_> = payload
                .get(4..4 + count)
                .ok_or_else(|| EffectError::EngineFault("truncated TW Congji cards".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
            if cards.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_none_or(|card_state| card_state.zone != discard)
            }) {
                return Err(EffectError::Illegal("TW Congji card left discard".into()));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("recipient hand missing".into()))?;
            crate::effect::move_cards(tx, cards, hand, Some(discard)).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad TW Wujing choice payload".into(),
        )),
    }
}

pub(crate) fn resolve_twshanghe(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("bad TW Shanghe card choice".into()));
    }
    let player = PlayerId::new(payload[0]);
    let giver = PlayerId::new(payload[1]);
    let skill = SkillId::new(payload[2]);
    let count = payload[4] as usize;
    let remaining: Vec<_> = payload
        .get(5..5 + count)
        .ok_or_else(|| EffectError::EngineFault("truncated TW Shanghe queue".into()))?
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    let card = CardId::new(selections[0]);
    let in_giver_zone = [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
        tx.state().zones.find(kind, giver).is_some_and(|zone| {
            tx.state()
                .cards
                .get(card)
                .is_some_and(|card_state| card_state.owner == giver && card_state.zone == zone)
        })
    });
    if observer != giver
        || !in_giver_zone
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|owner| owner.alive)
    {
        return Err(EffectError::Illegal("TW Shanghe context changed".into()));
    }
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("TW Shanghe card missing".into()))?;
    let from = card_state.zone;
    let got_jiu = payload[3] != 0
        || crate::query::effective_card_kind(tx.state(), player, card_state) == CardKind::Jiu;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("TW Shanghe owner hand missing".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveTwshanghe {
            player,
            stage: 1,
            remaining,
            got_jiu,
            skill,
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

pub(crate) fn resolve_twwangchang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let mode = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = SkillId::new(if mode == 3 {
        *payload.get(2).unwrap_or(&0)
    } else {
        *payload.get(4).unwrap_or(&0)
    });
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("TW Wangchang context changed".into()));
    }
    if mode == 3 {
        if cancelled || payload.len() != 3 || selections.is_empty() {
            return Err(EffectError::Illegal("bad TW Kaiji target choice".into()));
        }
        let max = 1 + tx.state().lifetime_dying_players.len();
        let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
        let mut unique = targets.clone();
        unique.sort_unstable_by_key(|target| target.raw());
        unique.dedup();
        if targets.len() > max
            || unique.len() != targets.len()
            || targets.iter().any(|target| {
                !tx.state()
                    .players
                    .get(*target)
                    .is_some_and(|participant| participant.alive)
            })
        {
            return Err(EffectError::Illegal("invalid TW Kaiji targets".into()));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveTwwangchang {
                player,
                target: PlayerId::NONE,
                card: CardId::NONE,
                selected_card: CardId::NONE,
                action: 0,
                stage: 1,
                targets,
                skill,
            }],
        )
        .map(|_| ());
    }
    if payload.len() != 5 {
        return Err(EffectError::EngineFault(
            "bad TW Shepan continuation".into(),
        ));
    }
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    match mode {
        0 => {
            if cancelled || selections.len() != 1 || !matches!(selections[0], 0 | 1) {
                return Err(EffectError::Illegal("bad TW Shepan branch choice".into()));
            }
            if selections[0] == 0 {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveTwwangchang {
                        player,
                        target,
                        card,
                        selected_card: CardId::NONE,
                        action: 1,
                        stage: 2,
                        targets: Vec::new(),
                        skill,
                    }],
                )
                .map(|_| ());
            }
            let options = crate::command::play::target_zone_card_tokens(tx, target, true, true, true);
            if options.is_empty() {
                return Err(EffectError::Illegal("TW Shepan target region empty".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwwangchang,
                vec![1, player.raw(), target.raw(), card.raw(), skill.raw()],
            )?;
            Ok(())
        }
        1 => {
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Shepan card choice".into()));
            }
            let selected_card = CardId::new(selections[0]);
            if !crate::command::play::target_zone_card_tokens(tx, target, true, true, true)
                .contains(&selected_card.raw())
            {
                return Err(EffectError::Illegal("invalid TW Shepan target card".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwwangchang {
                    player,
                    target,
                    card,
                    selected_card,
                    action: 1,
                    stage: 4,
                    targets: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            if cancelled {
                return Ok(());
            }
            if selections != [1] {
                return Err(EffectError::Illegal("bad TW Shepan nullify choice".into()));
            }
            crate::command::play::exclude_saved_card_target(tx, card, player)
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Wangchang continuation".into(),
        )),
    }
}

pub(crate) fn resolve_twcaozhao(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let mode = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = SkillId::new(*payload.last().unwrap_or(&0));
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal("TW Caozhao context changed".into()));
    }
    match mode {
        0 => {
            if observer != player || cancelled || payload.len() != 5 || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Fuzuan target choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            let defs = crate::query::conversion_skill_defs(tx.state(), target);
            if defs.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Fuzuan target has no conversion skill".into(),
                ));
            }
            if defs.len() == 1 {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveTwcaozhao {
                        player,
                        target,
                        source: PlayerId::NONE,
                        card: CardId::NONE,
                        selected_skill_def: defs[0].raw(),
                        twfuzuan_skill_def: payload[2],
                        twfeifu_skill_def: payload[3],
                        action: 0,
                        stage: 1,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                defs.into_iter().map(engine_api::SkillDefId::raw).collect(),
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwcaozhao,
                vec![
                    1,
                    player.raw(),
                    target.raw(),
                    payload[2],
                    payload[3],
                    skill.raw(),
                ],
            )?;
            Ok(())
        }
        1 => {
            if observer != player || cancelled || payload.len() != 6 || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Fuzuan skill choice".into()));
            }
            let target = PlayerId::new(payload[2]);
            if !crate::query::conversion_skill_defs(tx.state(), target)
                .iter()
                .any(|skill| skill.raw() == selections[0])
            {
                return Err(EffectError::Illegal(
                    "invalid TW Fuzuan conversion skill".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveTwcaozhao {
                    player,
                    target,
                    source: PlayerId::NONE,
                    card: CardId::NONE,
                    selected_skill_def: selections[0],
                    twfuzuan_skill_def: payload[3],
                    twfeifu_skill_def: payload[4],
                    action: 0,
                    stage: 1,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 => {
            if observer != player || payload.len() != 5 {
                return Err(EffectError::Illegal("bad TW Chongqi target choice".into()));
            }
            if cancelled {
                return if selections.is_empty() {
                    Ok(())
                } else {
                    Err(EffectError::Illegal(
                        "cancelled TW Chongqi has a target".into(),
                    ))
                };
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("TW Chongqi needs one target".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid TW Chongqi target".into()));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::ChangeMaxHp { player, delta: -1 },
                    EffectRequest::AddSkill {
                        player: target,
                        skill_def: engine_api::SkillDefId::new(payload[2]),
                    },
                ],
            )
            .map(|_| ())
        }
        3 => {
            if payload.len() != 6 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Feifu card choice".into()));
            }
            let giver = PlayerId::new(payload[2]);
            let source = PlayerId::new(payload[3]);
            if observer != giver
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("TW Feifu participants changed".into()));
            }
            let card = CardId::new(selections[0]);
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|state| state.zone)
                .ok_or_else(|| EffectError::Illegal("TW Feifu selected card missing".into()))?;
            if ![ZoneKind::Hand, ZoneKind::Equip]
                .into_iter()
                .any(|kind| tx.state().zones.find(kind, giver) == Some(from))
            {
                return Err(EffectError::Illegal("TW Feifu card left giver".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, source)
                .ok_or_else(|| {
                    EffectError::EngineFault("TW Feifu recipient hand missing".into())
                })?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwcaozhao {
                    player,
                    target: giver,
                    source,
                    card,
                    selected_skill_def: 0,
                    twfuzuan_skill_def: 0,
                    twfeifu_skill_def: 0,
                    action: 2,
                    stage: 1,
                    skill,
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
        _ => Err(EffectError::EngineFault(
            "unknown TW Caozhao continuation".into(),
        )),
    }
}

pub(crate) fn resolve_twguohuai(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("bad TW Guohuai choice".into()));
    }
    let mode = payload[0];
    let player = PlayerId::new(payload[1]);
    let source = PlayerId::new(payload[2]);
    let mark_def = payload[3];
    let helper_skill_def = payload[4];
    let skill = SkillId::new(payload[5]);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("TW Guohuai context changed".into()));
    }
    match (mode, selections[0]) {
        (0, 1) => run_effects(
            tx,
            vec![
                EffectRequest::RemoveMark {
                    player,
                    mark: engine_api::MarkDefId::new(mark_def),
                    amount: 1,
                },
                EffectRequest::AddSkillUntilTurnEnd {
                    player: source,
                    skill_def: engine_api::SkillDefId::new(helper_skill_def),
                },
            ],
        )
        .map(|_| ()),
        (0, 2) => {
            let cards = crate::query::discardable_hand_equip_cards(tx.state(), source);
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Yuzhang source has no cards".into(),
                ));
            }
            let count = cards.len().min(2) as u8;
            run_effects(
                tx,
                vec![EffectRequest::RemoveMark {
                    player,
                    mark: engine_api::MarkDefId::new(mark_def),
                    amount: 1,
                }],
            )?;
            open_choice(
                tx,
                source,
                count,
                count,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwguohuai,
                vec![
                    1,
                    player.raw(),
                    source.raw(),
                    mark_def,
                    helper_skill_def,
                    skill.raw(),
                ],
            )?;
            Ok(())
        }
        (1, _) => {
            let allowed: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), source)
                    .into_iter()
                    .collect();
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards.len() != allowed.len().min(2)
                || cards.iter().any(|card| !allowed.contains(card))
            {
                return Err(EffectError::Illegal("invalid TW Yuzhang discard".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: source,
                    cards,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("invalid TW Guohuai branch".into())),
    }
}

pub(crate) fn resolve_twchengpu_custom(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::TWCHENGPU_CUSTOM_CHOICE)
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("bad TW Chengpu choice".into()));
    }
    let mode = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = SkillId::new(*payload.last().unwrap_or(&0));
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal("TW Chengpu context changed".into()));
    }
    match mode {
        0 => {
            if observer != player || payload.len() != 5 {
                return Err(EffectError::Illegal("bad TW Chunlao target choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            let cards = crate::effect::twchengpu_field_cards(tx.state(), target);
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "TW Chunlao target has no field card".into(),
                ));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    crate::effect::TWCHENGPU_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    payload[3],
                    skill.raw(),
                ],
            )?;
            Ok(())
        }
        1 => {
            if observer != player || payload.len() != 6 {
                return Err(EffectError::Illegal("bad TW Chunlao card choice".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill_def = payload[4];
            let card = CardId::new(selections[0]);
            if !crate::effect::twchengpu_field_cards(tx.state(), target).contains(&card) {
                return Err(EffectError::Illegal("TW Chunlao card moved".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let to = crate::effect::ensure_skill_pile_zone(tx, target, skill_def)?;
            crate::effect::move_cards_with_reason(
                tx,
                vec![card],
                to,
                Some(from),
                engine_api::CardMoveReason::Give,
            )?;
            Ok(())
        }
        2 => {
            if payload.len() != 7 {
                return Err(EffectError::Illegal("bad TW Chunlao gift choice".into()));
            }
            let source = PlayerId::new(payload[3]);
            let sha = CardId::new(payload[4]);
            let card = CardId::new(selections[0]);
            if observer != source
                || !crate::effect::twchengpu_hand_equip_cards(tx.state(), source).contains(&card)
            {
                return Err(EffectError::Illegal("TW Chunlao gift card moved".into()));
            }
            let from = tx.state().cards.get(card).unwrap().zone;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("TW Chunlao owner hand missing".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwchengpu {
                    player,
                    target: source,
                    source,
                    card: sha,
                    chunlao_skill_def: payload[5],
                    action: 2,
                    stage: 1,
                    selected_card: card,
                    skill,
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
        _ => Err(EffectError::Illegal(
            "unknown TW Chengpu custom choice".into(),
        )),
    }
}

pub(crate) fn resolve_twcaoang_custom(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6
        || payload.first() != Some(&crate::effect::TWCAOANG_CUSTOM_CHOICE)
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("bad TW Caoang horse choice".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let helper_skill_def = payload[4];
    let skill = SkillId::new(payload[5]);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|target| target.alive)
    {
        return Err(EffectError::Illegal(
            "TW Caoang horse context changed".into(),
        ));
    }
    if stage == 0 {
        if cancelled || selections == [0] {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_twxiaolian_redirect();
            return Ok(());
        }
        if selections != [1] {
            return Err(EffectError::Illegal("bad TW Caoang confirmation".into()));
        }
        let options = crate::effect::twchengpu_hand_equip_cards(tx.state(), player);
        let mut next = payload.to_vec();
        next[1] = 1;
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            engine_api::ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            next,
        )
        .map(|_| ());
    }
    if stage != 1 || cancelled {
        return Err(EffectError::Illegal("bad TW Caoang horse stage".into()));
    }
    let card = CardId::new(selections[0]);
    if !crate::effect::twchengpu_hand_equip_cards(tx.state(), player).contains(&card) {

        return Err(EffectError::Illegal("TW Caoang horse card moved".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: engine_api::SkillDefId::new(helper_skill_def),
        }],
    )?;
    let from = tx.state().cards.get(card).unwrap().zone;
    let to = crate::effect::ensure_skill_pile_zone(tx, target, helper_skill_def)?;
    crate::effect::move_cards_with_reason(
        tx,
        vec![card],
        to,
        Some(from),
        engine_api::CardMoveReason::Unspecified,
    )?;
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .clear_twxiaolian_redirect();
    Ok(())
}

pub(crate) fn resolve_twniujin_custom(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.get(1).copied() {
        Some(0) => {
            if payload.len() != 4 {
                return Err(EffectError::EngineFault(
                    "bad TW Cuorui damage continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[2]);
            let skill = SkillId::new(payload[3]);
            if observer != player
                || !tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
            {
                return Err(EffectError::Illegal(
                    "TW Cuorui damage context changed".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "TW Cuorui chooses at most one target".into(),
                ));
            };
            let target = PlayerId::new(*selected);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("TW Cuorui target is invalid".into()));
            }
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
        Some(1) => {
            if payload.len() != 5 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("bad TW Liewei reward choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let limited_skill_def = engine_api::SkillDefId::new(payload[3]);
            let skill = SkillId::new(payload[4]);
            if observer != player
                || !tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
            {
                return Err(EffectError::Illegal("TW Liewei context changed".into()));
            }
            match selections[0] {
                0 => {
                    run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }]).map(|_| ())
                }
                1 => {
                    let limited = tx
                        .state()
                        .skills
                        .sorted_live()
                        .into_iter()
                        .find(|instance| {
                            instance.owner == player && instance.def == limited_skill_def
                        })
                        .map(|instance| instance.id)
                        .ok_or_else(|| {
                            EffectError::Illegal("TW Cuorui skill disappeared".into())
                        })?;
                    let instance = tx.state_mut().skills.get_mut(limited).unwrap();
                    if instance.lifetime_fire_count == 0 {
                        return Err(EffectError::Illegal(
                            "TW Cuorui is already available".into(),
                        ));
                    }
                    instance.lifetime_fire_count = 0;
                    Ok(())
                }
                _ => Err(EffectError::Illegal("unknown TW Liewei reward".into())),
            }
        }
        _ => Err(EffectError::EngineFault(
            "unknown TW Niujin continuation".into(),
        )),
    }
}

pub(crate) fn resolve_twmoukui_custom(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload.first() != Some(&crate::effect::TWMOUKUI_CUSTOM_CHOICE) {
        return Err(EffectError::EngineFault(
            "bad TW Moukui continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let card = CardId::new(payload[3]);
    let skill = SkillId::new(payload[4]);
    if observer != player {
        return Err(EffectError::Illegal(
            "TW Moukui choice must be resolved by its owner".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled TW Moukui selected a branch".into(),
            ))
        };
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || !tx
        .state()
        .resolution
        .sha_target_resolving(player, card, target)
    {
        return Err(EffectError::Illegal("TW Moukui context changed".into()));
    }
    match selections {
        [1] => run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player,
                count: 1,
                skill,
            }],
        )
        .map(|_| ()),
        [2] if !crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .is_empty() =>
        {
            run_effects(
                tx,
                vec![EffectRequest::ChooseDiscardTargetCard {
                    player,
                    target,
                    scope: rule_api::TargetCardScope::HandEquip,
                }],
            )
            .map(|_| ())
        }
        [3] => {
            if !tx
                .state_mut()
                .resolution
                .record_sha_backwater_target(player, card, skill, target)
            {
                return Err(EffectError::Illegal(
                    "TW Moukui Sha target disappeared".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 1,
                        skill,
                    },
                    EffectRequest::ChooseDiscardTargetCard {
                        player,
                        target,
                        scope: rule_api::TargetCardScope::HandEquip,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("unknown TW Moukui branch".into())),
    }
}

pub(crate) fn resolve_luanqun(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault(
                    "truncated Luanqun selection payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let effect_skill_def = SkillDefId::new(payload[2]);
            let direct_hit_skill_def = SkillDefId::new(payload[3]);
            let skill = SkillId::new(payload[4]);
            let index = payload[5] as usize;
            let count = payload[6] as usize;
            if count == 0 || index >= count || payload.len() != 7 + count + index {
                return Err(EffectError::EngineFault(
                    "bad Luanqun selection payload".into(),
                ));
            }
            let participants: Vec<_> = payload[7..7 + count]
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect();
            let mut cards: Vec<_> = payload[7 + count..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Luanqun requires exactly one hand card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            let chooser = participants[index];
            let hand = tx.state().zones.find(ZoneKind::Hand, chooser);
            if cancelled
                || observer != chooser
                || !tx
                    .state()
                    .cards
                    .get(selected)
                    .is_some_and(|card| Some(card.zone) == hand && card.owner == chooser)
            {
                return Err(EffectError::Illegal(
                    "Luanqun private selection context changed".into(),
                ));
            }
            cards.push(selected);
            if index + 1 < count {
                let next = participants[index + 1];
                let options = tx.state().hand_cards(next);
                if options.is_empty() {
                    return Err(EffectError::Illegal(
                        "Luanqun participant lost every hand card".into(),
                    ));
                }
                let mut next_payload = vec![
                    0,
                    player.raw(),
                    effect_skill_def.raw(),
                    direct_hit_skill_def.raw(),
                    skill.raw(),
                    (index + 1) as u32,
                    count as u32,
                ];
                next_payload.extend(participants.iter().map(|player| player.raw()));
                next_payload.extend(cards.iter().map(|card| card.raw()));
                return crate::effect::open_choice(
                    tx,
                    next,
                    1,
                    1,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    engine_api::ChoiceKind::SelectCards,
                    ContinuationKind::ResolveLuanqun,
                    next_payload,
                )
                .map(|_| ());
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveLuanqun {
                    player,
                    effect_skill_def,
                    direct_hit_skill_def,
                    stage: 1,
                    participants,
                    cards,
                    skill,
                }],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() < 4 {
                return Err(EffectError::EngineFault(
                    "truncated Luanqun gain payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = SkillId::new(payload[2]);
            let count = payload[3] as usize;
            let options = &payload[4..];
            if options.len() != count || observer != player {
                return Err(EffectError::EngineFault("bad Luanqun gain payload".into()));
            }
            if cancelled {
                if selections.is_empty() {
                    return Ok(());
                }
                return Err(EffectError::Illegal(
                    "cancelled Luanqun gain carried selections".into(),
                ));
            }
            let cap = if tx.state().ruleset.name.starts_with("identity") {
                4
            } else {
                2
            };
            let mut seen = std::collections::BTreeSet::new();
            if selections.is_empty()
                || selections.len() > cap
                || selections
                    .iter()
                    .any(|selected| !options.contains(selected) || !seen.insert(*selected))
                || !tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
            {
                return Err(EffectError::Illegal(
                    "Luanqun gain selection is no longer legal".into(),
                ));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Luanqun owner hand missing".into()))?;
            let mut effects = Vec::new();
            for selected in selections {
                let card = CardId::new(*selected);
                let state = tx
                    .state()
                    .cards
                    .get(card)
                    .ok_or_else(|| EffectError::Illegal("Luanqun card disappeared".into()))?;
                let from = tx.state().zones.get(state.zone).ok_or_else(|| {
                    EffectError::EngineFault("Luanqun source hand missing".into())
                })?;
                if from.kind != ZoneKind::Hand || state.owner == player {
                    return Err(EffectError::Illegal(
                        "Luanqun matching card left its hand".into(),
                    ));
                }
                effects.push(EffectRequest::MoveCards {
                    cards: vec![card],
                    to,
                    from: Some(state.zone),
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Luanqun choice stage".into(),
        )),
    }
}

/// Resolve the 择一加技能 (choose one of two skills to grant) custom choice.
///
/// ABI 644 (ADR 0612): the chooser picked option 1 (first_def) or 2 (second_def);
/// the chosen skill definition is granted to `target`.
pub(crate) fn resolve_choose_other_gain_one_of_two_skills(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS_CUSTOM_CHOICE)
        || payload.len() != 7
    {
        return Err(EffectError::EngineFault(
            "malformed choose-one-of-two-skills continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let first_def = SkillDefId::new(payload[4]);
    let second_def = SkillDefId::new(payload[5]);
    let skill = SkillId::new(payload[6]);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "choose-one-of-two-skills context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one of the two skills".into(),
        ));
    }
    let chosen = match selections[0] {
        1 => first_def,
        2 => second_def,
        _ => {
            return Err(EffectError::Illegal(
                "choose-one-of-two-skills option is out of range".into(),
            ))
        }
    };
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: chosen,
        }],
    )
    .map(|_| ())
}

/// Resolve the 水淹七军 (guozhan) target choice: 1=弃置全部装备牌, 2=受 1 点雷电伤害.
///
/// The queued-trick tail rides in payload[3..]; the chosen effect runs and then the
/// independent trick queue advances (or finishes when the tail is absent).
pub(crate) fn resolve_shuiyanqijunx_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::SHUIYANQIJUNX_CUSTOM_CHOICE)
        || payload.len() < 3
    {
        return Err(EffectError::EngineFault(
            "malformed 水淹七军 continuation".into(),
        ));
    }
    let actor = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let queue = payload.get(3..).unwrap_or_default();
    if observer != target {
        return Err(EffectError::Illegal(
            "水淹七军 choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one 水淹七军 option".into(),
        ));
    }
    let effects = match selections[0] {
        1 => vec![EffectRequest::DiscardAllTargetEquipment { actor, target }],
        2 => vec![EffectRequest::DealDamage {
            source: actor,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
        _ => {
            return Err(EffectError::Illegal(
                "水淹七军 option is out of range".into(),
            ))
        }
    };
    if queue.is_empty() {
        return run_effects(tx, effects).map(|_| ());
    }
    crate::command::play::run_effects_then_finish_independent_trick_target(tx, effects, queue)
}

/// Resolve the 克复中原 (guozhan) target choice: 1=视为使用杀（蜀势力伤害基数+1），
/// 2=摸牌（蜀势力摸两张，否则一张）。
///
/// The queued-trick tail rides in payload[3..]; the chosen effect runs and then the
/// independent trick queue advances (or finishes when the tail is absent).
pub(crate) fn resolve_kefuzhongyuan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::KEFUZHONGYUAN_CUSTOM_CHOICE)
        || payload.len() < 3
    {
        return Err(EffectError::EngineFault(
            "malformed 克复中原 continuation".into(),
        ));
    }
    let actor = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let queue = payload.get(3..).unwrap_or_default();
    if observer != target {
        return Err(EffectError::Illegal(
            "克复中原 choice observer mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one 克复中原 option".into(),
        ));
    }
    let is_shu = tx.state().player_group(target).map(str::to_string).as_deref() == Some("shu");
    let effects = match selections[0] {
        1 => vec![EffectRequest::ChooseUseVirtualShaWithBonus {
            player: target,
            ignore_distance: false,
            damage_bonus: if is_shu { 1 } else { 0 },
        }],
        2 => vec![EffectRequest::DrawCards {
            player: target,
            count: if is_shu { 2 } else { 1 },
        }],
        _ => {
            return Err(EffectError::Illegal(
                "克复中原 option is out of range".into(),
            ))
        }
    };
    if queue.is_empty() {
        return run_effects(tx, effects).map(|_| ());
    }
    crate::command::play::run_effects_then_finish_independent_trick_target(tx, effects, queue)
}

/// Resolve the 固国安邦 (guozhan) staged settlement.
///
/// Stage 1: discard ≥ 6 hand cards. Stage 2: if 吴, pick 1–2 of the discarded cards to
/// give. Stage 3: pick one other 吴 target to receive them. Single give round (上游循环
/// 给多目标为近似省略).
pub(crate) fn resolve_guguoanbang_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::GUGUOANBANG_CUSTOM_CHOICE)
        || payload.len() < 4
    {
        return Err(EffectError::EngineFault(
            "malformed 固国安邦 continuation".into(),
        ));
    }
    let stage = payload[1];
    let actor = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let tail = payload.get(4..).unwrap_or_default();
    if observer != target {
        return Err(EffectError::Illegal(
            "固国安邦 choice observer mismatch".into(),
        ));
    }
    let finish = |tx: &mut Transaction| -> Result<(), EffectError> {
        crate::command::play::finish_independent_trick_target(tx, tail)
    };
    match stage {
        1 => {
            if cancelled {
                return finish(tx);
            }
            if selections.len() < 6 {
                return Err(EffectError::Illegal(
                    "固国安邦 requires discarding at least six hand cards".into(),
                ));
            }
            let hand = tx.state().hand_cards(target);
            if selections.iter().any(|raw| !hand.contains(&CardId::new(*raw))) {
                return Err(EffectError::Illegal(
                    "固国安邦 discard selection is not in hand".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )?;
            let is_wu = tx
                .state()
                .player_group(target)
                .map(str::to_string)
                .as_deref()
                == Some("wu");
            let other_wu: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != target
                        && tx
                            .state()
                            .player_group(*candidate)
                            .map(str::to_string)
                            .as_deref()
                            == Some("wu")
                })
                .collect();
            if !is_wu || selections.is_empty() || other_wu.is_empty() {
                return finish(tx);
            }
            let mut next = vec![
                crate::effect::GUGUOANBANG_CUSTOM_CHOICE,
                2,
                actor.raw(),
                target.raw(),
            ];
            next.push(selections.len().min(2) as u32);
            next.extend(selections.iter().copied());
            next.extend_from_slice(tail);
            crate::effect::open_choice(
                tx,
                target,
                1,
                selections.len().min(2) as u8,
                false,
                selections.to_vec(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                next,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        2 => {
            if cancelled {
                return finish(tx);
            }
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "truncated 固国安邦 give-cards payload".into(),
                ));
            }
            let give_count = payload[4] as usize;
            let give_cards: Vec<_> = payload
                .get(5..5 + give_count)
                .ok_or_else(|| EffectError::EngineFault("truncated give cards".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let tail2 = payload.get(5 + give_count..).unwrap_or_default();
            if selections.is_empty()
                || selections.len() > give_count
                || selections.iter().any(|raw| !give_cards.contains(&CardId::new(*raw)))
            {
                return Err(EffectError::Illegal(
                    "固国安邦 give-card selection is invalid".into(),
                ));
            }
            let other_wu: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != target
                        && tx
                            .state()
                            .player_group(*candidate)
                            .map(str::to_string)
                            .as_deref()
                            == Some("wu")
                })
                .collect();
            if other_wu.is_empty() {
                return finish(tx);
            }
            let mut next = vec![
                crate::effect::GUGUOANBANG_CUSTOM_CHOICE,
                3,
                actor.raw(),
                target.raw(),
            ];
            next.push(selections.len() as u32);
            next.extend(selections.iter().copied());
            next.extend_from_slice(tail2);
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                other_wu.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                crate::state::ContinuationKind::Custom,
                next,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        3 => {
            if cancelled || selections.len() != 1 {
                return finish(tx);
            }
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "truncated 固国安邦 give-target payload".into(),
                ));
            }
            let give_count = payload[4] as usize;
            let give_cards: Vec<_> = payload
                .get(5..5 + give_count)
                .ok_or_else(|| EffectError::EngineFault("truncated give cards".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let tail3 = payload.get(5 + give_count..).unwrap_or_default();
            let recipient = PlayerId::new(selections[0]);
            if recipient == target
                || !tx
                    .state()
                    .players
                    .get(recipient)
                    .is_some_and(|p| p.alive)
                || tx
                    .state()
                    .player_group(recipient)
                    .map(str::to_string)
                    .as_deref()
                    != Some("wu")
            {
                return Err(EffectError::Illegal(
                    "固国安邦 recipient must be another living 吴 player".into(),
                ));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, recipient)
                .ok_or_else(|| EffectError::Illegal("recipient hand missing".into()))?;
            let from = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: give_cards,
                    to,
                    from: Some(from),
                }],
            )?;
            crate::command::play::finish_independent_trick_target(tx, tail3)
        }
        _ => Err(EffectError::Illegal(
            "固国安邦 stage is out of range".into(),
        )),
    }
}

/// Resolve the 联军盛宴 (guozhan) draw/recover split for the user: the picked token is
/// the recover amount; draw = X − recover. The queued-trick tail rides after [magic,
/// actor, X].
pub(crate) fn resolve_lianjunshengyan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::LIANJUNSHENGYAN_CUSTOM_CHOICE)
        || payload.len() < 3
    {
        return Err(EffectError::EngineFault(
            "malformed 联军盛宴 continuation".into(),
        ));
    }
    let actor = PlayerId::new(payload[1]);
    let x = payload[2];
    let tail = payload.get(3..).unwrap_or_default();
    if observer != actor {
        return Err(EffectError::Illegal(
            "联军盛宴 choice observer mismatch".into(),
        ));
    }
    if cancelled || selections.len() != 1 {
        return crate::command::play::finish_independent_trick_target(tx, tail);
    }
    let recover = selections[0];
    let participant = tx
        .state()
        .players
        .get(actor)
        .ok_or_else(|| EffectError::Illegal("user missing".into()))?;
    let damaged = participant
        .max_hp
        .saturating_sub(participant.hp.max(0) as u32);
    let cap = x.min(damaged);
    if recover > cap {
        return Err(EffectError::Illegal(
            "联军盛宴 recover amount exceeds the damage cap".into(),
        ));
    }
    let draw = x.saturating_sub(recover);
    let mut effects = Vec::with_capacity(2);
    if draw > 0 {
        effects.push(EffectRequest::DrawCards {
            player: actor,
            count: draw,
        });
    }
    if recover > 0 {
        effects.push(EffectRequest::RecoverHp {
            source: actor,
            player: actor,
            amount: recover,
        });
    }
    if tail.is_empty() {
        return run_effects(tx, effects).map(|_| ());
    }
    crate::command::play::run_effects_then_finish_independent_trick_target(tx, effects, tail)
}

/// Resolve the 文和乱武 (guozhan) staged settlement.
///
/// Stage 1: the card user (actor) picks ① 令目标弃置两张类型不同的手牌 or ② 弃置其一张
/// 手牌. Stage 2: the discard choice — for ① the target picks 2 (or 1 when only a single
/// card type is available) of their hand cards; for ② the actor picks 1 of the target's
/// hand cards. After the discard, a 群 faction target with an empty hand draws up to its
/// current HP (capped at 5), mirroring upstream `target.draw(Math.min(5, target.hp))`.
/// Payload: [magic, stage, actor, target, discarder, count, ...queued-tail].
pub(crate) fn resolve_wenheluanwu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::WENHELUANWU_CUSTOM_CHOICE)
        || payload.len() < 4
    {
        return Err(EffectError::EngineFault(
            "malformed 文和乱武 continuation".into(),
        ));
    }
    let stage = payload[1];
    let actor = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    match stage {
        1 => {
            if observer != actor {
                return Err(EffectError::Illegal(
                    "文和乱武 stage-1 choice belongs to the user".into(),
                ));
            }
            let tail = payload.get(4..).unwrap_or_default();
            let finish = |tx: &mut Transaction| -> Result<(), EffectError> {
                crate::command::play::finish_independent_trick_target(tx, tail)
            };
            if cancelled {
                return finish(tx);
            }
            if selections.len() != 1 || !matches!(selections[0], 1 | 2) {
                return Err(EffectError::Illegal(
                    "choose exactly one 文和乱武 option".into(),
                ));
            }
            let hand = tx.state().hand_cards(target);
            if hand.is_empty() {
                return finish(tx);
            }
            let (discarder, count) = if selections[0] == 1 {
                // ① 令目标弃置两张类型不同的手牌；仅一种类型可弃时退化为弃一张。
                let mut groups = std::collections::BTreeSet::new();
                for cid in &hand {
                    groups.insert(
                        tx.state()
                            .cards
                            .get(*cid)
                            .map(|c| c.kind.card_type_group())
                            .unwrap_or(0),
                    );
                }
                (1, if groups.len() >= 2 { 2 } else { 1 })
            } else {
                (2, 1)
            };
            let mut next = vec![
                crate::effect::WENHELUANWU_CUSTOM_CHOICE,
                2,
                actor.raw(),
                target.raw(),
                discarder,
                count,
            ];
            next.extend_from_slice(tail);
            let chooser = if discarder == 1 { target } else { actor };
            crate::effect::open_choice(
                tx,
                chooser,
                count as u8,
                count as u8,
                false,
                hand.into_iter().map(|cid| cid.raw()).collect(),
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::Custom,
                next,
            )?;
            Err(EffectError::ChoiceRequested)
        }
        2 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault(
                    "truncated 文和乱武 discard payload".into(),
                ));
            }
            let discarder = payload[4];
            let count = payload[5] as usize;
            let tail = payload.get(6..).unwrap_or_default();
            let finish = |tx: &mut Transaction| -> Result<(), EffectError> {
                crate::command::play::finish_independent_trick_target(tx, tail)
            };
            if cancelled {
                return finish(tx);
            }
            let expected_observer = if discarder == 1 { target } else { actor };
            if observer != expected_observer || !matches!(discarder, 1 | 2) {
                return Err(EffectError::Illegal(
                    "文和乱武 discard choice observer mismatch".into(),
                ));
            }
            if count == 0 || count > 2 || selections.len() != count {
                return Err(EffectError::Illegal(
                    "文和乱武 discard selection count is invalid".into(),
                ));
            }
            let hand = tx.state().hand_cards(target);
            if selections
                .iter()
                .any(|raw| !hand.contains(&CardId::new(*raw)))
            {
                return Err(EffectError::Illegal(
                    "文和乱武 discard selection is not in target hand".into(),
                ));
            }
            if count == 2 {
                let groups: Vec<u8> = selections
                    .iter()
                    .map(|raw| {
                        tx.state()
                            .cards
                            .get(CardId::new(*raw))
                            .map(|c| c.kind.card_type_group())
                            .unwrap_or(0)
                    })
                    .collect();
                if groups[0] == groups[1] {
                    return Err(EffectError::Illegal(
                        "文和乱武 requires two hand cards of different types".into(),
                    ));
                }
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )?;
            // 群势力且弃后空手：摸至当前体力（至多 5）。弃牌已完成，按弃后手牌判断。
            let is_qun = tx
                .state()
                .player_group(target)
                .map(str::to_string)
                .as_deref()
                == Some("qun");
            if is_qun && tx.state().hand_cards(target).is_empty() {
                let hp = tx
                    .state()
                    .players
                    .get(target)
                    .map(|p| p.hp.max(0) as u32)
                    .unwrap_or(0);
                if hp > 0 {
                    run_effects(
                        tx,
                        vec![EffectRequest::DrawCards {
                            player: target,
                            count: 5.min(hp),
                        }],
                    )?;
                }
            }
            finish(tx)
        }
        _ => Err(EffectError::Illegal(
            "文和乱武 stage is out of range".into(),
        )),
    }
}

/// Resolve the 号令天下 (guozhan) per-player chain.
///
/// Payload: `[magic, stage, mode?, ...chain_queue]` where chain_queue is the
/// independent-trick-queue superset documented at `advance_haolingtianxia_player`.
/// Stage 1 = option choice (observer = current chain player): ① 弃牌视为对目标使用杀
/// （魏势力免弃牌）, ② 弃置（魏势力改为获得）目标一张牌. Stage 2 = card pick:
/// mode 1 = 出杀费用弃本人「he」牌, mode 2 = 弃目标「he」牌, mode 3 = 魏获得目标
/// 「he」牌. Any card-moving effect may suspend on a trigger choice; the whole step is
/// run through the IndependentTrickQueue resume so the chain continues afterwards.
pub(crate) fn resolve_haolingtianxia_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::HAOLINGTIANXIA_CUSTOM_CHOICE)
        || payload.len() < 11
    {
        return Err(EffectError::EngineFault(
            "malformed 号令天下 continuation".into(),
        ));
    }
    let stage = payload[1];
    let queue = if stage == 1 { &payload[2..] } else { &payload[3..] };
    if queue.len() < 8 {
        return Err(EffectError::EngineFault(
            "truncated 号令天下 chain queue".into(),
        ));
    }
    let index = queue[5] as usize;
    let len = queue[6] as usize;
    let current = PlayerId::new(queue[7 + index]);
    let trick_target = PlayerId::new(queue[7 + len]);
    if observer != current {
        return Err(EffectError::Illegal(
            "号令天下 choice observer mismatch".into(),
        ));
    }
    let is_wei = tx
        .state()
        .player_group(current)
        .map(str::to_string)
        .as_deref()
        == Some("wei");
    let advance = |tx: &mut Transaction| -> Result<(), EffectError> {
        let mut next = queue.to_vec();
        next[5] = next[5].saturating_add(1);
        crate::command::play::advance_haolingtianxia_player(tx, &next)
    };
    let step_effects = |tx: &mut Transaction,
                        effects: Vec<EffectRequest>|
     -> Result<(), EffectError> {
        let mut next = queue.to_vec();
        next[5] = next[5].saturating_add(1);
        crate::command::play::run_effects_then_resume_trick_queue(tx, effects, next)
    };
    match stage {
        1 => {
            if cancelled {
                return advance(tx);
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose exactly one 号令天下 option".into(),
                ));
            }
            match selections[0] {
                1 => {
                    // ① 弃一张牌（魏免），视为对目标使用杀。
                    if is_wei {
                        return step_effects(
                            tx,
                            vec![EffectRequest::DealDamage {
                                source: current,
                                target: trick_target,
                                amount: 1,
                                is_lose_hp: false,
                                nature: rule_api::damage_nature::NORMAL,
                            }],
                        );
                    }
                    let tokens =
                        crate::command::play::target_zone_card_tokens(tx, current, true, true, false);
                    if tokens.is_empty() {
                        return advance(tx);
                    }
                    let mut next = vec![
                        crate::effect::HAOLINGTIANXIA_CUSTOM_CHOICE,
                        2,
                        1, // mode: sha cost discard
                    ];
                    next.extend_from_slice(queue);
                    crate::effect::open_choice(
                        tx,
                        current,
                        1,
                        1,
                        true,
                        tokens,
                        engine_api::ChoiceKind::SelectCards,
                        crate::state::ContinuationKind::Custom,
                        next,
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
                2 => {
                    // ② 弃置（魏势力改为获得）目标一张「he」牌。
                    let mode = if is_wei { 3 } else { 2 };
                    let tokens: Vec<u32> = if is_wei {
                        crate::query::gainable_target_cards(
                            tx.state(),
                            current,
                            trick_target,
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .into_iter()
                        .map(|cid| cid.raw())
                        .collect()
                    } else {
                        crate::command::play::target_zone_card_tokens(
                            tx, trick_target, true, true, false,
                        )
                    };
                    if tokens.is_empty() {
                        return advance(tx);
                    }
                    let mut next = vec![
                        crate::effect::HAOLINGTIANXIA_CUSTOM_CHOICE,
                        2,
                        mode,
                    ];
                    next.extend_from_slice(queue);
                    crate::effect::open_choice(
                        tx,
                        current,
                        1,
                        1,
                        true,
                        tokens,
                        engine_api::ChoiceKind::SelectCards,
                        crate::state::ContinuationKind::Custom,
                        next,
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
                _ => Err(EffectError::Illegal(
                    "号令天下 option is out of range".into(),
                )),
            }
        }
        2 => {
            if cancelled {
                return advance(tx);
            }
            let mode = payload[2];
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "pick exactly one 号令天下 card".into(),
                ));
            }
            let sel = CardId::new(selections[0]);
            match mode {
                1 => {
                    // 出杀费用：本人弃一张「he」牌，然后对目标造成 1 点伤害。
                    // 近似说明：引擎杀管线（ResumePlayedSha）独占 resume 单槽，杀完成后
                    // 无法续链号令天下链；以直接伤害近似（无闪响应），ADR 0623 备注。
                    let tokens =
                        crate::command::play::target_zone_card_tokens(tx, current, true, true, false);
                    if !tokens.contains(&selections[0]) {
                        return Err(EffectError::Illegal(
                            "号令天下 sha cost must be a current he card".into(),
                        ));
                    }
                    step_effects(
                        tx,
                        vec![
                            EffectRequest::DiscardCards {
                                player: current,
                                cards: vec![sel],
                            },
                            EffectRequest::DealDamage {
                                source: current,
                                target: trick_target,
                                amount: 1,
                                is_lose_hp: false,
                                nature: rule_api::damage_nature::NORMAL,
                            },
                        ],
                    )
                }
                2 => {
                    // 弃置目标一张「he」牌。
                    let tokens = crate::command::play::target_zone_card_tokens(
                        tx, trick_target, true, true, false,
                    );
                    if !tokens.contains(&selections[0]) {
                        return Err(EffectError::Illegal(
                            "号令天下 discard must be a target he card".into(),
                        ));
                    }
                    step_effects(
                        tx,
                        vec![EffectRequest::DiscardCards {
                            player: trick_target,
                            cards: vec![sel],
                        }],
                    )
                }
                3 => {
                    // 魏势力获得目标一张「he」牌。
                    let gainable = crate::query::gainable_target_cards(
                        tx.state(),
                        current,
                        trick_target,
                        rule_api::TargetCardScope::HandEquip,
                    );
                    if !gainable.contains(&sel) {
                        return Err(EffectError::Illegal(
                            "号令天下 gain must be a gainable target he card".into(),
                        ));
                    }
                    let zone = tx
                        .state()
                        .cards
                        .get(sel)
                        .map(|c| c.zone)
                        .ok_or_else(|| EffectError::Illegal("bad card".into()))?;
                    let to_hand = tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, current)
                        .ok_or_else(|| EffectError::Illegal("no current hand".into()))?;
                    step_effects(
                        tx,
                        vec![EffectRequest::MoveCards {
                            cards: vec![sel],
                            to: to_hand,
                            from: Some(zone),
                        }],
                    )
                }
                _ => Err(EffectError::Illegal(
                    "号令天下 mode is out of range".into(),
                )),
            }
        }
        _ => Err(EffectError::Illegal(
            "号令天下 stage is out of range".into(),
        )),
    }
}

/// Resolve the 敕令 (guozhan) per-target choice.
///
/// Payload: `[magic, stage, mode?, target, ...queued-tail]`.
/// Stage 1 = the target picks ① 明置一张武将牌并摸一张 ② 失去1点体力 ③ 弃置一张装备牌
/// （③ 仅在有装备牌时可选，① 的槽位选择在 Stage 2）。Stage 2 mode 1 = 选一个暗置槽
/// 位明置；mode 3 = 选一张装备牌弃置。明置经 `mode::reveal_general`（确定性状态变更），
/// 摸牌经 `DrawCards`。
pub(crate) fn resolve_chiling_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&crate::effect::CHILING_CUSTOM_CHOICE) || payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "malformed 敕令 continuation".into(),
        ));
    }
    let stage = payload[1];
    // Stage 1: [magic, 1, target, ...tail]; Stage 2: [magic, 2, mode, target, ...tail].
    let target = PlayerId::new(payload[if stage == 1 { 2 } else { 3 }]);
    let tail_offset = if stage == 1 { 3 } else { 4 };
    let tail = &payload[tail_offset.min(payload.len())..];
    let finish = |tx: &mut Transaction| -> Result<(), EffectError> {
        crate::command::play::finish_independent_trick_target(tx, tail)
    };
    if observer != target {
        return Err(EffectError::Illegal("敕令 choice observer mismatch".into()));
    }
    match stage {
        1 => {
            if cancelled {
                return finish(tx);
            }
            if selections.len() != 1 || !matches!(selections[0], 1 | 2 | 3) {
                return Err(EffectError::Illegal(
                    "choose exactly one 敕令 option".into(),
                ));
            }
            match selections[0] {
                1 => {
                    // ① 选要明置的暗置槽位（仅列出仍暗置的槽）。
                    let unseen: Vec<u32> = tx
                        .state()
                        .players
                        .get(target)
                        .map(|p| {
                            p.unseen
                                .iter()
                                .enumerate()
                                .filter_map(|(i, &hidden)| hidden.then_some(i as u32))
                                .collect()
                        })
                        .unwrap_or_default();
                    if unseen.is_empty() {
                        return finish(tx);
                    }
                    let mut next = vec![
                        crate::effect::CHILING_CUSTOM_CHOICE,
                        2,
                        1, // mode: reveal slot
                        target.raw(),
                    ];
                    next.extend_from_slice(tail);
                    crate::effect::open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        unseen,
                        engine_api::ChoiceKind::Custom,
                        crate::state::ContinuationKind::Custom,
                        next,
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
                2 => {
                    // ② 失去1点体力（无来源）。
                    let effects = vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }];
                    if tail.is_empty() {
                        return run_effects(tx, effects).map(|_| ());
                    }
                    crate::command::play::run_effects_then_finish_independent_trick_target(
                        tx, effects, tail,
                    )
                }
                _ => {
                    // ③ 弃置一张装备牌（he 中的装备类牌）。
                    let tokens = crate::command::play::equip_typed_zone_cards(tx, target);
                    if tokens.is_empty() {
                        return finish(tx);
                    }
                    let mut next = vec![
                        crate::effect::CHILING_CUSTOM_CHOICE,
                        2,
                        3, // mode: discard equipment
                        target.raw(),
                    ];
                    next.extend_from_slice(tail);
                    crate::effect::open_choice(
                        tx,
                        target,
                        1,
                        1,
                        false,
                        tokens,
                        engine_api::ChoiceKind::SelectCards,
                        crate::state::ContinuationKind::Custom,
                        next,
                    )?;
                    Err(EffectError::ChoiceRequested)
                }
            }
        }
        2 => {
            if cancelled {
                return finish(tx);
            }
            let mode = payload[2];
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "pick exactly one 敕令 card/slot".into(),
                ));
            }
            match mode {
                1 => {
                    // 明置选中的暗置槽位，然后摸一张牌。
                    let slot = selections[0];
                    let unseen = tx
                        .state()
                        .players
                        .get(target)
                        .map(|p| p.unseen.get(slot as usize).copied().unwrap_or(false))
                        .unwrap_or(false);
                    if !unseen {
                        return Err(EffectError::Illegal(
                            "敕令 reveal slot must be unseen".into(),
                        ));
                    }
                    crate::mode::reveal_general(
                        tx.state_mut(),
                        target,
                        u8::try_from(slot).unwrap_or(u8::MAX),
                    )
                    .map_err(EffectError::Illegal)?;
                    let effects = vec![EffectRequest::DrawCards {
                        player: target,
                        count: 1,
                    }];
                    if tail.is_empty() {
                        return run_effects(tx, effects).map(|_| ());
                    }
                    crate::command::play::run_effects_then_finish_independent_trick_target(
                        tx, effects, tail,
                    )
                }
                3 => {
                    // 弃置一张装备牌。
                    let tokens = crate::command::play::equip_typed_zone_cards(tx, target);
                    if !tokens.contains(&selections[0]) {
                        return Err(EffectError::Illegal(
                            "敕令 discard must be an equipment-typed zone card".into(),
                        ));
                    }
                    let effects = vec![EffectRequest::DiscardCards {
                        player: target,
                        cards: vec![CardId::new(selections[0])],
                    }];
                    if tail.is_empty() {
                        return run_effects(tx, effects).map(|_| ());
                    }
                    crate::command::play::run_effects_then_finish_independent_trick_target(
                        tx, effects, tail,
                    )
                }
                _ => Err(EffectError::Illegal("敕令 mode is out of range".into())),
            }
        }
        _ => Err(EffectError::Illegal("敕令 stage is out of range".into())),
    }
}
