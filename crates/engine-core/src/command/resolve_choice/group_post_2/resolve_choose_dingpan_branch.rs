//! Command operations for `resolve_tuxi_gain_other_hands_replace_draw` and related helpers.

use super::*;

pub(crate) fn resolve_choose_dingpan_branch(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, target, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Dingpan branch payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let target = PlayerId::new(*target);
    let skill = engine_api::SkillId::new(*skill);
    if observer != target
        || cancelled
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Dingpan branch context changed".into(),
        ));
    }
    let [branch] = selections else {
        return Err(EffectError::Illegal(
            "Dingpan requires exactly one branch".into(),
        ));
    };
    match branch {
        0 => run_effects(
            tx,
            vec![EffectRequest::ChooseDiscardTargetCard {
                player,
                target,
                scope: rule_api::TargetCardScope::Equip,
            }],
        )
        .map(|_| ()),
        1 => {
            let equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .ok_or_else(|| EffectError::EngineFault("missing Dingpan equipment zone".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("missing Dingpan hand zone".into()))?;
            let cards = tx
                .state()
                .zones
                .get(equip)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default();
            let tail = EffectRequest::ResolveDingpan {
                player,
                target,
                stage: 2,
                skill,
            };
            if cards.is_empty() {
                run_effects(tx, vec![tail]).map(|_| ())
            } else {
                tx.state_mut().resolution.deferred_tail_effects.push(tail);
                run_effects(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards,
                        to: hand,
                        from: Some(equip),
                    }],
                )
                .map(|_| ())
            }
        }
        _ => Err(EffectError::Illegal("unknown Dingpan branch".into())),
    }
}

pub(crate) fn resolve_tanlu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, target, amount, skill] = payload else {
        return Err(EffectError::EngineFault("bad Tanlu payload".into()));
    };
    let player = PlayerId::new(*player);
    let target = PlayerId::new(*target);
    let skill = engine_api::SkillId::new(*skill);
    if observer != target
        || cancelled
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Tanlu choice context changed".into()));
    }
    match stage {
        0 => {
            let [branch] = selections else {
                return Err(EffectError::Illegal(
                    "Tanlu requires exactly one branch".into(),
                ));
            };
            match branch {
                0 => {
                    let cards = crate::query::gainable_target_cards(
                        tx.state(),
                        player,
                        target,
                        rule_api::TargetCardScope::Hand,
                    );
                    if cards.is_empty() {
                        return Ok(());
                    }
                    if cards.len() <= *amount as usize {
                        let from =
                            tx.state()
                                .zones
                                .find(ZoneKind::Hand, target)
                                .ok_or_else(|| {
                                    EffectError::EngineFault("missing Tanlu giver hand".into())
                                })?;
                        let to =
                            tx.state()
                                .zones
                                .find(ZoneKind::Hand, player)
                                .ok_or_else(|| {
                                    EffectError::EngineFault("missing Tanlu recipient hand".into())
                                })?;
                        return run_effects(tx, vec![EffectRequest::GiveCards { cards, to, from }])
                            .map(|_| ());
                    }
                    // ponytail: ChoiceRequest counts are u8; widen that protocol if a legal hand can
                    // ever exceed 255 cards.
                    let count = u8::try_from(*amount).map_err(|_| {
                        EffectError::ContentFault("Tanlu gift exceeds choice protocol".into())
                    })?;
                    crate::effect::open_choice(
                        tx,
                        target,
                        count,
                        count,
                        false,
                        cards.into_iter().map(CardId::raw).collect(),
                        engine_api::ChoiceKind::SelectCards,
                        ContinuationKind::ResolveTanlu,
                        vec![1, player.raw(), target.raw(), *amount, skill.raw()],
                    )
                    .map(|_| ())
                }
                1 => run_effects(
                    tx,
                    vec![EffectRequest::ResolveTanlu {
                        player,
                        target,
                        amount: *amount,
                        stage: 1,
                        skill,
                    }],
                )
                .map(|_| ()),
                _ => Err(EffectError::Illegal("unknown Tanlu branch".into())),
            }
        }
        1 => {
            let available = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::Hand,
            );
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            if cards.len() != *amount as usize
                || cards
                    .iter()
                    .any(|card| !distinct.insert(card.raw()) || !available.contains(card))
            {
                return Err(EffectError::Illegal(
                    "Tanlu gift cards are invalid or stale".into(),
                ));
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("missing Tanlu giver hand".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("missing Tanlu recipient hand".into()))?;
            run_effects(tx, vec![EffectRequest::GiveCards { cards, to, from }]).map(|_| ())
        }
        2 => {
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Tanlu requires one owner hand card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            if !crate::query::discardable_target_cards(
                tx.state(),
                target,
                player,
                rule_api::TargetCardScope::Hand,
            )
            .contains(&selected)
            {
                return Err(EffectError::Illegal(
                    "Tanlu discard card is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![selected],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Tanlu continuation stage".into(),
        )),
    }
}

pub(crate) fn resolve_target_discard_then_duel(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, target, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad target-discard Duel continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let target = PlayerId::new(*target);
    if observer != target || cancelled {
        return Err(EffectError::Illegal(
            "target-discard Duel requires the target's forced choice".into(),
        ));
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "target-discard Duel requires exactly one card".into(),
        ));
    };
    let selected = CardId::new(*selected);
    if !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&selected) {
        return Err(EffectError::Illegal(
            "target-discard Duel card is invalid or stale".into(),
        ));
    }
    let discarded_was_sha = tx.state().cards.get(selected).is_some_and(|card| {
        crate::query::effective_card_kind(tx.state(), target, card) == CardKind::Sha
    });
    match run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: target,
            cards: vec![selected],
        }],
    ) {
        Ok(_) => crate::effect::resolve_target_discard_duel(tx, player, target, discarded_was_sha)
            .map(|_| ()),
        Err(EffectError::ChoiceRequested) => {
            tx.state_mut().resolution.resume_kind =
                ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner;
            tx.state_mut().resolution.resume_payload = vec![
                player.raw(),
                target.raw(),
                *skill,
                u32::from(discarded_was_sha),
            ];
            Err(EffectError::ChoiceRequested)
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn resolve_choose_target_lose_hp_discard_hand_sha_if_basic(
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
                            rule_api::TargetCardScope::Hand,
                        )
                        .is_empty()
                })
            {
                return Err(EffectError::Illegal(
                    "conditional Sha target choice context changed".into(),
                ));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
                    player,
                    target: target.expect("validated conditional Sha target"),
                    skill,
                    stage: 1,
                    discarded_basic: false,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| ())
        }
        Some(1) if payload.len() == 4 => {
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let selected = selections.first().copied().map(CardId::new);
            if observer != target
                || cancelled
                || selections.len() != 1
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
                        rule_api::TargetCardScope::Hand,
                    )
                    .contains(&card)
                })
            {
                return Err(EffectError::Illegal(
                    "conditional Sha discard choice context changed".into(),
                ));
            }
            let selected = selected.expect("validated conditional Sha discard");
            let discarded_basic = tx.state().cards.get(selected).is_some_and(|card| {
                crate::query::effective_card_kind(tx.state(), target, card).is_basic()
            });
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ChooseTargetLoseHpDiscardHandShaIfBasic {
                    player,
                    target,
                    skill,
                    stage: 2,
                    discarded_basic,
                },
            );
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
            "bad target-discard conditional Sha continuation".into(),
        )),
    }
}

pub(crate) fn resolve_tongji_discard_redirect_sha(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [owner, source, card, target, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Tongji redirect continuation payload".into(),
        ));
    };
    let owner = PlayerId::new(*owner);
    let source = PlayerId::new(*source);
    let card = CardId::new(*card);
    let target = PlayerId::new(*target);
    let skill = engine_api::SkillId::new(*skill);
    let valid = observer == target
        && [owner, source, target].into_iter().all(|player| {
            tx.state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
        })
        && tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == owner && tx.state().skill_effectively_enabled(instance)
        })
        && crate::command::play::validate_saved_sha_target(tx, source, card, target).is_ok()
        && crate::command::play::validate_saved_sha_target(tx, source, card, owner).is_err()
        && crate::query::player_in_attack_range(tx.state(), target, owner)
        && crate::command::play::validate_sha_target_with_options(tx, source, card, owner, true)
            .is_ok();
    if !valid {
        return Err(EffectError::Illegal(
            "Tongji redirect context changed".into(),
        ));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled Tongji cost contains a card".into(),
            ));
        }
        return Ok(());
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "Tongji requires exactly one hand/equipment card".into(),
        ));
    };
    let selected = CardId::new(*selected);
    if !crate::query::discardable_hand_equip_cards(tx.state(), target).contains(&selected) {
        return Err(EffectError::Illegal(
            "Tongji discard cost is invalid or stale".into(),
        ));
    }
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::ChooseCurrentShaTargetDiscardRedirectToOwner {
            owner,
            source,
            card,
            target,
            skill,
            ready_to_redirect: true,
        },
    );
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: target,
            cards: vec![selected],
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_liuli_discard_redirect_sha(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("truncated Liuli payload".into()))?;
    let player = PlayerId::new(
        *payload
            .get(1)
            .ok_or_else(|| EffectError::EngineFault("truncated Liuli player".into()))?,
    );
    let source = PlayerId::new(
        *payload
            .get(2)
            .ok_or_else(|| EffectError::EngineFault("truncated Liuli source".into()))?,
    );
    let card = CardId::new(
        *payload
            .get(3)
            .ok_or_else(|| EffectError::EngineFault("truncated Liuli card".into()))?,
    );
    let skill = engine_api::SkillId::new(
        *payload
            .get(4)
            .ok_or_else(|| EffectError::EngineFault("truncated Liuli skill".into()))?,
    );
    let context_valid = observer == player
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|player| player.alive)
        && tx
            .state()
            .players
            .get(source)
            .is_some_and(|source| source.alive)
        && tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        && !crate::command::play::saved_sha_redirect_targets_in_attack_range(
            tx, source, card, player, player,
        )
        .unwrap_or_default()
        .is_empty();
    if !context_valid {
        return Err(EffectError::Illegal(
            "Liuli redirect context changed".into(),
        ));
    }
    match stage {
        0 => {
            if payload.len() != 5 {
                return Err(EffectError::EngineFault("bad Liuli discard payload".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled Liuli cost contains a card".into(),
                    ));
                }
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Liuli requires exactly one hand/equipment card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&selected) {
                return Err(EffectError::Illegal(
                    "Liuli discard cost is invalid or stale".into(),
                ));
            }
            let candidates = crate::command::play::saved_sha_redirect_targets_in_attack_range(
                tx, source, card, player, player,
            )?;
            crate::effect::open_choice(
                tx,
                player,
                0,
                1,
                true,
                candidates.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::LiuliDiscardRedirectSha,
                vec![
                    1,
                    player.raw(),
                    source.raw(),
                    card.raw(),
                    skill.raw(),
                    selected.raw(),
                ],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault("bad Liuli target payload".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled Liuli target contains a player".into(),
                    ));
                }
                return Ok(());
            }
            let [target] = selections else {
                return Err(EffectError::Illegal(
                    "Liuli requires exactly one replacement target".into(),
                ));
            };
            let target = PlayerId::new(*target);
            let candidates = crate::command::play::saved_sha_redirect_targets_in_attack_range(
                tx, source, card, player, player,
            )?;
            if !candidates.contains(&target) {
                return Err(EffectError::Illegal(
                    "Liuli replacement target is invalid or stale".into(),
                ));
            }
            let selected = CardId::new(payload[5]);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&selected) {
                return Err(EffectError::Illegal(
                    "Liuli discard cost is invalid or stale".into(),
                ));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ChooseDiscardRedirectCurrentSha {
                    player,
                    source,
                    card,
                    target,
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![selected],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown Liuli stage".into())),
    }
}

pub(crate) fn resolve_jijie_choose_bottom_card_recipient(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, card, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Jijie recipient continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Jijie requires exactly one recipient".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ChooseTargetGainBottomCard {
            player,
            card: CardId::new(*card),
            target: PlayerId::new(selections[0]),
            skill: engine_api::SkillId::new(*skill),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_luoshen_repeat_black_judgment(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let (player, skill, protect_hand_limit) = match payload {
        [player, skill] => (*player, *skill, false),
        [player, skill, 1] => (*player, *skill, true),
        _ => {
            return Err(EffectError::EngineFault(
                "bad Luoshen repeat continuation payload".into(),
            ));
        }
    };
    let player = PlayerId::new(player);
    if observer != player {
        return Err(EffectError::Illegal(
            "Luoshen repeat has the wrong observer".into(),
        ));
    }
    if cancelled || selections.is_empty() || selections == [0] {
        return Ok(());
    }
    if selections != [1] {
        return Err(EffectError::Illegal(
            "Luoshen repeat requires confirm or cancel".into(),
        ));
    }
    run_effects(
        tx,
        vec![if protect_hand_limit {
            EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
                player,
                card: CardId::NONE,
                skill: engine_api::SkillId::new(skill),
                ready_to_choose: false,
            }
        } else {
            EffectRequest::JudgeBlackGainRepeat {
                player,
                card: CardId::NONE,
                skill: engine_api::SkillId::new(skill),
                ready_to_choose: false,
            }
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_judge_same_color_gain_repeat(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill, color] = payload else {
        return Err(EffectError::EngineFault(
            "bad Yjluoshen repeat continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal(
            "Yjluoshen repeat has the wrong observer".into(),
        ));
    }
    if cancelled || selections.is_empty() || selections == [0] {
        return Ok(());
    }
    if selections != [1] || !matches!(*color, 1 | 2) {
        return Err(EffectError::Illegal(
            "Yjluoshen repeat requires confirm or cancel".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::JudgeSameColorGainRepeat {
            player,
            card: CardId::NONE,
            skill: engine_api::SkillId::new(*skill),
            ready_to_choose: false,
            color: *color as u8,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_prevent_damage_or_lose_max_hp_increase_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad bound damage branch payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let damage_id = payload[1];
    let amount = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player
        || cancelled
        || selections.len() != 1
        || !matches!(selections[0], 1 | 2)
        || amount == 0
        || !crate::effect::bound_damage_source_matches(tx, damage_id, player)
    {
        return Err(EffectError::Illegal(
            "bound damage branch choice context changed".into(),
        ));
    }

    if selections[0] == 1 {
        return run_effects(tx, vec![EffectRequest::PreventDamage { damage_id }]).map(|_| ());
    }

    tx.state_mut().resolution.deferred_effects.insert(
        0,
        EffectRequest::ContinueIncreaseBoundDamage {
            player,
            damage_id,
            amount,
            skill,
        },
    );
    run_effects(tx, vec![EffectRequest::ChangeMaxHp { player, delta: -1 }]).map(|_| ())
}

pub(crate) fn resolve_choose_draw_or_increase_current_card_damage_or_both_mark(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 7 {
        return Err(EffectError::EngineFault(
            "bad current-card branch payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let card = CardId::new(payload[1]);
    let draw_count = payload[2];
    let damage_amount = payload[3];
    let mark_def = engine_api::MarkDefId::new(payload[4]);
    let mark_count = i32::try_from(payload[5])
        .map_err(|_| EffectError::EngineFault("current-card branch mark overflow".into()))?;
    let skill = engine_api::SkillId::new(payload[6]);
    let owner = tx.state().players.get(player);
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    let card_kind = tx
        .state()
        .cards
        .get(card)
        .map(|card_state| crate::query::effective_card_kind(tx.state(), player, card_state));
    if observer != player
        || cancelled
        || selections.len() != 1
        || !matches!(selections[0], 1 | 2 | 3)
        || draw_count == 0
        || damage_amount == 0
        || mark_def.is_none()
        || mark_count <= 0
        || !owner.is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32)
        || !skill_enabled
        || card_kind.is_none()
    {
        return Err(EffectError::Illegal(
            "current-card branch choice context changed".into(),
        ));
    }

    let mut effects = Vec::new();
    if matches!(selections[0], 1 | 3) {
        effects.push(EffectRequest::DrawCards {
            player,
            count: draw_count,
        });
    }
    if matches!(selections[0], 2 | 3)
        && card_kind.is_some_and(|kind| {
            kind.is_sha() || matches!(kind, CardKind::TrickNanman | CardKind::TrickWanjian)
        })
    {
        effects.push(EffectRequest::IncreaseCurrentCardBaseDamage {
            player,
            card,
            amount: damage_amount,
            skill,
        });
    }
    if selections[0] == 3 {
        effects.push(EffectRequest::AddMark {
            player,
            mark: mark_def,
            delta: mark_count,
        });
    }
    run_effects(tx, effects).map(|_| ())
}
