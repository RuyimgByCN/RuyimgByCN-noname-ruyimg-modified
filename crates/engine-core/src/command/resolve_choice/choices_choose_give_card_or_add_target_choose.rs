//! 选择续行（resolve_choice）：choose 族效果。

use super::*;

pub(super) fn resolve_choice(
    tx: &mut Transaction,
    _actor: PlayerId,
    observer: PlayerId,
    kind: ContinuationKind,
    payload: Vec<u32>,
    selections: &[u32],
    cancelled: bool,
    _vm_cont: (Vec<u8>, String, u32, u32, [u32; 4], u32, [u32; 3]),
    _optional_trigger: Option<crate::trigger::PendingTrigger>,
) -> Result<(), EffectError> {
    match kind {
        ContinuationKind::ChooseGiveCardOrAddTarget => {
            resolve_choose_give_card_or_add_target(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDiscardEquipmentReduceCurrentDamage => {
            resolve_choose_discard_equipment_reduce_current_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDiscardDamageEndpointCardIncreaseIfSuit => {
            resolve_choose_discard_damage_endpoint_card_increase_if_suit(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseEndExtraDrawOrPlay => {
            resolve_choose_end_extra_draw_or_play(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseMoveTargetEquipmentOrDraw => {
            resolve_choose_move_target_equipment_or_draw(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseRecoverOwnerOrDamageByGivenEquipment => {
            resolve_choose_recover_owner_or_damage_by_given_equipment(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage => {
            resolve_choose_target_discard_weapon_range_hand_or_increase_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth => {
            resolve_choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDrawOrRecastAddSkill => {
            resolve_choose_draw_or_recast_add_skill(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrawOrDiscardTargetCardRecordSha => {
            resolve_choose_draw_or_discard_target_card_record_sha(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDiscardTargetCardsRecoverSameCategory => {
            resolve_choose_discard_target_cards_recover_same_category(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseScskuijiDiscard => {
            resolve_choose_scskuiji_discard(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChoosePskuijiDiscard => {
            resolve_choose_pskuiji_discard(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseSpjungongDiscardOrLose => {
            resolve_choose_spjungong_discard_or_lose(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseSpxizhanDiscardOrLose => {
            resolve_choose_spxizhan_discard_or_lose(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseSpsongshuCards => {
            resolve_choose_spsongshu_cards(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseCompareTargetHandCardSecondFailureSeatDamage => {
            resolve_choose_compare_target_hand_card_second_failure_seat_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseDrawOrDiscardTargetCard => {
            resolve_choose_draw_or_discard_target_card(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseSeatPathDamage => {
            resolve_choose_seat_path_damage(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGainShaForOpeningExtraPlay => {
            resolve_choose_gain_sha_for_opening_extra_play(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseHandCountDifferentTargetDrawOrRandomDiscard => {
            resolve_choose_hand_count_different_target_draw_or_random_discard(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw => {
            resolve_choose_target_discard_excess_then_self_discard_or_target_draw(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseGainOneFromDamageTargetAndNeighbors => {
            resolve_choose_gain_one_from_damage_target_and_neighbors(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChoosePreventDamageOrLoseMaxHpIncreaseDamage => {
            resolve_choose_prevent_damage_or_lose_max_hp_increase_damage(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseOneFromSkillPileDiscard => {
            resolve_choose_one_from_skill_pile_discard(
                tx, observer, &payload, selections, cancelled,
            )
        }

        ContinuationKind::ChooseTargetHandCardToPile => {
            resolve_choose_target_hand_card_to_pile(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseHandCardsToSkillPile => {
            resolve_choose_hand_cards_to_skill_pile(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseGainTopCardsBySuit => {
            if payload.len() < 3 {
                return Err(EffectError::EngineFault(
                    "bad top-card suit-choice continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[0]);
            let suit = crate::command::judge::suit_from_tag(payload[1]);
            let take = payload[2] as usize;
            if take == 0 || payload.len() != 3 + take || suit == engine_api::Suit::None {
                return Err(EffectError::EngineFault(
                    "truncated top-card suit-choice continuation".into(),
                ));
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            let zone = tx
                .state()
                .zones
                .get(draw)
                .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
            if zone.cards.len() < take
                || zone.cards[zone.cards.len() - take..]
                    .iter()
                    .map(|card| card.raw())
                    .ne(payload[3..].iter().copied())
            {
                return Err(EffectError::Illegal(
                    "viewed cards no longer match the draw-pile top".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled top-card suit choice has selections".into(),
                    ));
                }
                return Ok(());
            }
            if selections.is_empty() || selections.len() > take {
                return Err(EffectError::Illegal(
                    "select a nonempty subset of viewed cards".into(),
                ));
            }
            let selected: std::collections::BTreeSet<u32> = selections.iter().copied().collect();
            if selected.len() != selections.len()
                || !selected.iter().all(|raw| payload[3..].contains(raw))
                || !selected.iter().all(|raw| {
                    tx.state().cards.get(CardId::new(*raw)).is_some_and(|card| {
                        crate::query::effective_card_suit(tx.state(), player, card) == suit
                    })
                })
            {
                return Err(EffectError::Illegal(
                    "selected cards must be distinct viewed cards of the required suit".into(),
                ));
            }
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|player| player.alive)
            {
                return Err(EffectError::Illegal(
                    "top-card suit choice player is not alive".into(),
                ));
            }
            let cards: Vec<CardId> = selections.iter().copied().map(CardId::new).collect();
            tx.push_event(engine_api::DomainEvent::CardsRevealed {
                cards: cards.clone(),
                to: Vec::new(),
            });
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: hand,
                    from: Some(draw),
                }],
            )
            .map(|_| ())
        }

        ContinuationKind::ChooseOtherDrawByHandCount => {
            if payload.len() != 5 {
                return Err(EffectError::EngineFault(
                    "bad other-player draw continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[0]);
            let normal_count = payload[1];
            let empty_count = payload[2];
            let times = payload[3];
            let skill = engine_api::SkillId::new(payload[4]);
            if player != observer || times == 0 {
                return Err(EffectError::EngineFault(
                    "invalid other-player draw continuation owner or count".into(),
                ));
            }
            let mut effects = Vec::with_capacity(2);
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled other-player draw choice has selections".into(),
                    ));
                }
            } else {
                if selections.len() != 1 {
                    return Err(EffectError::Illegal(
                        "select exactly one other player to draw".into(),
                    ));
                }
                let target = PlayerId::new(selections[0]);
                if target == player
                    || !tx
                        .state()
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                {
                    return Err(EffectError::Illegal(
                        "draw target must be another living player".into(),
                    ));
                }
                let count = if tx.state().hand_count(target) == 0 {
                    empty_count
                } else {
                    normal_count
                };
                if count > 0 {
                    effects.push(EffectRequest::DrawCardsFromSkill {
                        player: target,
                        count,
                        skill,
                    });
                }
            }
            if times > 1 {
                effects.push(EffectRequest::ChooseOtherDrawByHandCount {
                    player,
                    normal_count,
                    empty_count,
                    times: times - 1,
                    skill,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }

        ContinuationKind::ChooseOtherDrawOrRecover => {
            let stage = *payload.first().unwrap_or(&u32::MAX);
            match stage {
                0 => {
                    if payload.len() != 6 {
                        return Err(EffectError::EngineFault(
                            "bad other-player draw-or-recover target continuation".into(),
                        ));
                    }
                    let player = PlayerId::new(payload[1]);
                    let draw_count = payload[2];
                    let recover_count = payload[3];
                    let times = payload[4];
                    let skill = engine_api::SkillId::new(payload[5]);
                    if player != observer || times == 0 {
                        return Err(EffectError::EngineFault(
                            "invalid other-player draw-or-recover owner or count".into(),
                        ));
                    }
                    if cancelled {
                        if !selections.is_empty() {
                            return Err(EffectError::Illegal(
                                "cancelled draw-or-recover target choice has selections".into(),
                            ));
                        }
                        return if times > 1 {
                            run_effects(
                                tx,
                                vec![EffectRequest::ChooseOtherDrawOrRecover {
                                    player,
                                    draw_count,
                                    recover_count,
                                    times: times - 1,
                                    skill,
                                }],
                            )
                            .map(|_| ())
                        } else {
                            Ok(())
                        };
                    }
                    if selections.len() != 1 {
                        return Err(EffectError::Illegal(
                            "select exactly one other player to draw or recover".into(),
                        ));
                    }
                    let target = PlayerId::new(selections[0]);
                    let participant =
                        tx.state().players.get(target).ok_or_else(|| {
                            EffectError::Illegal("bad draw-or-recover target".into())
                        })?;
                    if target == player || !participant.alive {
                        return Err(EffectError::Illegal(
                            "draw-or-recover target must be another living player".into(),
                        ));
                    }
                    let mut effects = Vec::with_capacity(2);
                    if participant.hp >= participant.max_hp as i32 || recover_count == 0 {
                        if draw_count > 0 {
                            effects.push(EffectRequest::DrawCardsFromSkill {
                                player: target,
                                count: draw_count,
                                skill,
                            });
                        }
                    } else if draw_count == 0 {
                        effects.push(EffectRequest::RecoverHp {
                            source: target,
                            player: target,
                            amount: recover_count,
                        });
                    } else {
                        return crate::effect::open_choice(
                            tx,
                            target,
                            1,
                            1,
                            false,
                            vec![1, 2],
                            engine_api::ChoiceKind::Custom,
                            ContinuationKind::ChooseOtherDrawOrRecover,
                            vec![
                                1,
                                player.raw(),
                                target.raw(),
                                draw_count,
                                recover_count,
                                times,
                                skill.raw(),
                            ],
                        )
                        .map(|_| ());
                    }
                    if times > 1 {
                        effects.push(EffectRequest::ChooseOtherDrawOrRecover {
                            player,
                            draw_count,
                            recover_count,
                            times: times - 1,
                            skill,
                        });
                    }
                    run_effects(tx, effects).map(|_| ())
                }
                1 => {
                    if payload.len() != 7 {
                        return Err(EffectError::EngineFault(
                            "bad other-player draw-or-recover branch continuation".into(),
                        ));
                    }
                    let player = PlayerId::new(payload[1]);
                    let target = PlayerId::new(payload[2]);
                    let draw_count = payload[3];
                    let recover_count = payload[4];
                    let times = payload[5];
                    let skill = engine_api::SkillId::new(payload[6]);
                    if observer != target || times == 0 || cancelled {
                        return Err(EffectError::Illegal(
                            "draw-or-recover branch cannot cancel or change actor".into(),
                        ));
                    }
                    if !tx
                        .state()
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                    {
                        return Err(EffectError::Illegal(
                            "draw-or-recover target is no longer living".into(),
                        ));
                    }
                    let mut effects = Vec::with_capacity(2);
                    match selections {
                        [1] => effects.push(EffectRequest::DrawCardsFromSkill {
                            player: target,
                            count: draw_count,
                            skill,
                        }),
                        [2] => effects.push(EffectRequest::RecoverHp {
                            source: target,
                            player: target,
                            amount: recover_count,
                        }),
                        _ => {
                            return Err(EffectError::Illegal(
                                "choose draw or recover exactly once".into(),
                            ));
                        }
                    }
                    if times > 1 {
                        effects.push(EffectRequest::ChooseOtherDrawOrRecover {
                            player,
                            draw_count,
                            recover_count,
                            times: times - 1,
                            skill,
                        });
                    }
                    run_effects(tx, effects).map(|_| ())
                }
                2 => {
                    if payload.len() != 3 {
                        return Err(EffectError::EngineFault(
                            "bad forced other-player recovery continuation".into(),
                        ));
                    }
                    let player = PlayerId::new(payload[1]);
                    let amount = payload[2];
                    if observer != player || cancelled {
                        return Err(EffectError::Illegal(
                            "forced other-player recovery cannot cancel or change actor".into(),
                        ));
                    }
                    if selections.len() != 1 {
                        return Err(EffectError::Illegal(
                            "select exactly one other player to recover".into(),
                        ));
                    }
                    let target = PlayerId::new(selections[0]);
                    if target == player
                        || !tx
                            .state()
                            .players
                            .get(target)
                            .is_some_and(|participant| participant.alive)
                    {
                        return Err(EffectError::Illegal(
                            "recovery target must be another living player".into(),
                        ));
                    }
                    run_effects(
                        tx,
                        vec![EffectRequest::RecoverHp {
                            source: target,
                            player: target,
                            amount,
                        }],
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::EngineFault(
                    "unknown other-player draw-or-recover continuation stage".into(),
                )),
            }
        }

        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill => {
            if payload.len() < 4 {
                return Err(EffectError::Illegal(
                    "invalid dynamic hand/equipment discard payload".into(),
                ));
            }
            let player = PlayerId::new(payload[0]);
            let skill = engine_api::SkillId::new(payload[1]);
            let required = payload[2] as usize;
            let option_count = payload[3] as usize;
            let recover_target = (payload.len() == 6 + option_count).then(|| {
                (
                    PlayerId::new(payload[4 + option_count]),
                    payload[5 + option_count],
                )
            });
            if cancelled {
                return if recover_target.is_some() {
                    Ok(())
                } else {
                    Err(EffectError::Illegal(
                        "dynamic hand/equipment discard cannot be cancelled".into(),
                    ))
                };
            }
            if player != observer
                || required == 0
                || (payload.len() != 4 + option_count && recover_target.is_none())
                || selections.len() != required
            {
                return Err(EffectError::Illegal(
                    "invalid dynamic hand/equipment discard selection".into(),
                ));
            }
            let originating_skill_enabled = tx.state().skills.get(skill).is_some_and(|entry| {
                entry.owner == player && tx.state().skill_effectively_enabled(entry)
            });
            if !originating_skill_enabled {
                return Err(EffectError::Illegal(
                    "dynamic discard source skill is no longer enabled".into(),
                ));
            }
            let original: std::collections::BTreeSet<_> =
                payload[4..4 + option_count].iter().copied().collect();
            let current: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            let mut selected = std::collections::BTreeSet::new();
            if !selections.iter().all(|card| {
                selected.insert(*card) && original.contains(card) && current.contains(card)
            }) {
                return Err(EffectError::Illegal(
                    "dynamic discard card is duplicated or no longer legal".into(),
                ));
            }
            if let Some((target, amount)) = recover_target {
                if amount == 0
                    || !tx.state().players.get(target).is_some_and(|participant| {
                        participant.alive && participant.hp < participant.max_hp as i32
                    })
                {
                    return Err(EffectError::Illegal(
                        "dynamic discard recovery target is no longer legal".into(),
                    ));
                }
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(EffectRequest::RecoverHp {
                        source: player,
                        player: target,
                        amount,
                    });
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

        ContinuationKind::ChooseTakeLargerHandUntilMax => {
            resolve_choose_take_larger_hand_until_max(tx, observer, &payload, selections, cancelled)
        }

        ContinuationKind::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark => {
            resolve_choose_draw_or_increase_current_card_damage_or_both_mark(
                tx, observer, &payload, selections, cancelled,
            )
        }

        _ => unreachable!("choice routed to a different continuation group"),
    }
}
