//! Command operations for `resolve_choose_give_two_different_colors_or_recover` and related helpers.

use super::*;

pub(crate) fn resolve_choose_give_two_different_colors_or_recover(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::EngineFault(
            "truncated different-color give-or-recover continuation".into(),
        ));
    }
    let owner = PlayerId::new(payload[0]);
    let giver = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let option_count = payload[4] as usize;
    if payload.len() != 5 + option_count {
        return Err(EffectError::EngineFault(
            "bad different-color give-or-recover option snapshot".into(),
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
        || !matches!(selections.len(), 0 | 2)
        || (cancelled && !selections.is_empty())
    {
        return Err(EffectError::Illegal(
            "different-color give-or-recover choice context changed".into(),
        ));
    }
    if selections.is_empty() {
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

    let cards = [CardId::new(selections[0]), CardId::new(selections[1])];
    if cards[0] == cards[1]
        || cards.iter().any(|card| !payload[5..].contains(&card.raw()))
        || crate::query::effective_card_color_mask(tx.state(), giver, cards[0])
            == crate::query::effective_card_color_mask(tx.state(), giver, cards[1])
    {
        return Err(EffectError::Illegal(
            "gift cards must be distinct offered cards of different colors".into(),
        ));
    }

    let mut by_zone = std::collections::BTreeMap::<u32, Vec<CardId>>::new();
    for card in cards {
        let zone = response_card_zone(tx, giver, card)?;
        by_zone.entry(zone.raw()).or_default().push(card);
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no recipient hand".into()))?;
    run_effects(
        tx,
        by_zone
            .into_iter()
            .map(|(zone, cards)| EffectRequest::GiveCards {
                cards,
                to,
                from: engine_api::ZoneId::new(zone),
            })
            .collect(),
    )
    .map(|_| ())
}

pub(crate) fn resolve_mbzhuguo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = payload
        .first()
        .copied()
        .ok_or_else(|| EffectError::EngineFault("missing mbzhuguo continuation stage".into()))?;
    let enabled = |state: &GameState, player: PlayerId, skill: engine_api::SkillId| {
        state.skills.get(skill).is_some_and(|instance| {
            instance.owner == player && state.skill_effectively_enabled(instance)
        })
    };
    match stage {
        0 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault(
                    "truncated mbzhuguo discard continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let need = payload[4] as usize;
            let option_count = payload[5] as usize;
            if payload.len() != 6 + option_count
                || observer != target
                || cancelled
                || selections.len() != need
                || !enabled(tx.state(), player, skill)
            {
                return Err(EffectError::Illegal(
                    "mbzhuguo discard choice context changed".into(),
                ));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("mbzhuguo target hand missing".into()))?;
            let legal: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), target)
                    .into_iter()
                    .filter(|card| {
                        tx.state()
                            .cards
                            .get(*card)
                            .is_some_and(|card| card.zone == hand)
                    })
                    .map(CardId::raw)
                    .collect();
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if selected.len() != need
                || !selected.is_subset(&legal)
                || !selected.iter().all(|card| payload[6..].contains(card))
            {
                return Err(EffectError::Illegal(
                    "mbzhuguo discard must use offered current hand cards".into(),
                ));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::RecoverHp {
                    source: PlayerId::NONE,
                    player: target,
                    amount: 1,
                });
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "truncated mbzhuguo target continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let option_count = payload[4] as usize;
            if payload.len() != 5 + option_count
                || observer != player
                || !enabled(tx.state(), player, skill)
                || (cancelled && !selections.is_empty())
                || selections.len() > 1
            {
                return Err(EffectError::Illegal(
                    "mbzhuguo target choice context changed".into(),
                ));
            }
            let Some(victim) = selections.first().copied().map(PlayerId::new) else {
                return Ok(());
            };
            if !payload[5..].contains(&victim.raw())
                || ![target, victim].into_iter().all(|id| {
                    tx.state()
                        .players
                        .get(id)
                        .is_some_and(|participant| participant.alive)
                })
                || victim == player
                || victim == target
            {
                return Err(EffectError::Illegal(
                    "mbzhuguo follow-up target is invalid".into(),
                ));
            }
            let mut cards: Vec<_> = tx
                .state()
                .hand_cards(target)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        crate::query::effective_card_kind(tx.state(), target, card) == CardKind::Sha
                    }) && crate::command::play::validate_sha_target_with_options(
                        tx, target, *card, victim, true,
                    )
                    .is_ok()
                })
                .collect();
            cards.sort_by_key(|card| card.raw());
            if cards.is_empty() {
                return Ok(());
            }
            let mut next = vec![
                2,
                player.raw(),
                target.raw(),
                victim.raw(),
                skill.raw(),
                cards.len() as u32,
            ];
            next.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveMbzhuguo,
                next,
            )
            .map(|_| ())
        }
        2 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault(
                    "truncated mbzhuguo Sha continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let victim = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let option_count = payload[5] as usize;
            if payload.len() != 6 + option_count
                || observer != target
                || !enabled(tx.state(), player, skill)
                || (cancelled && !selections.is_empty())
                || selections.len() > 1
            {
                return Err(EffectError::Illegal(
                    "mbzhuguo Sha choice context changed".into(),
                ));
            }
            let Some(card) = selections.first().copied().map(CardId::new) else {
                return Ok(());
            };
            if !payload[6..].contains(&card.raw())
                || !tx.state().hand_cards(target).contains(&card)
                || !tx.state().cards.get(card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), target, card) == CardKind::Sha
                })
                || crate::command::play::validate_sha_target_with_options(
                    tx, target, card, victim, true,
                )
                .is_err()
            {
                return Err(EffectError::Illegal(
                    "mbzhuguo selected Sha is invalid".into(),
                ));
            }
            let legal: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    crate::command::play::validate_sha_target_with_options(
                        tx, target, card, *candidate, true,
                    )
                    .is_ok()
                })
                .collect();
            let (min_count, max_count) = if crate::query::sha_targets_all_for(tx.state(), target) {
                let count = u8::try_from(legal.len()).unwrap_or(u8::MAX);
                (count, count)
            } else {
                (
                    1,
                    u8::try_from(
                        legal
                            .len()
                            .min(crate::query::sha_target_limit_for(tx.state(), target) as usize),
                    )
                    .unwrap_or(u8::MAX),
                )
            };
            let mut next = vec![
                3,
                player.raw(),
                target.raw(),
                victim.raw(),
                card.raw(),
                skill.raw(),
                legal.len() as u32,
            ];
            next.extend(legal.iter().map(|candidate| candidate.raw()));
            open_choice(
                tx,
                target,
                min_count,
                max_count,
                true,
                legal.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveMbzhuguo,
                next,
            )
            .map(|_| ())
        }
        3 => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault(
                    "truncated mbzhuguo Sha-target continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let victim = PlayerId::new(payload[3]);
            let card = CardId::new(payload[4]);
            let skill = engine_api::SkillId::new(payload[5]);
            let option_count = payload[6] as usize;
            if payload.len() != 7 + option_count
                || observer != target
                || !enabled(tx.state(), player, skill)
                || (cancelled && !selections.is_empty())
            {
                return Err(EffectError::Illegal(
                    "mbzhuguo Sha-target choice context changed".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if selected.len() != selections.len()
                || !selected.contains(&victim.raw())
                || !selected.iter().all(|id| payload[7..].contains(id))
            {
                return Err(EffectError::Illegal(
                    "mbzhuguo Sha targets must include the selected character".into(),
                ));
            }
            crate::command::play::play_prompted_sha_source_unlimited(
                tx,
                target,
                card,
                &selections
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect::<Vec<_>>(),
            )
        }
        _ => Err(EffectError::EngineFault(
            "unknown mbzhuguo continuation stage".into(),
        )),
    }
}

pub(crate) fn resolve_mbanxian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = payload
        .first()
        .copied()
        .ok_or_else(|| EffectError::EngineFault("missing mbanxian continuation stage".into()))?;
    let enabled = |state: &GameState, player: PlayerId, skill: engine_api::SkillId| {
        state.skills.get(skill).is_some_and(|instance| {
            instance.owner == player && state.skill_effectively_enabled(instance)
        })
    };
    match stage {
        0 => {
            if payload.len() < 4 {
                return Err(EffectError::EngineFault(
                    "truncated mbanxian card continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = engine_api::SkillId::new(payload[2]);
            let option_count = payload[3] as usize;
            if payload.len() != 4 + option_count
                || observer != player
                || cancelled
                || selections.len() != 1
                || !enabled(tx.state(), player, skill)
            {
                return Err(EffectError::Illegal(
                    "mbanxian card choice context changed".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
            if !payload[4..].contains(&card.raw())
                || !tx.state().cards.get(card).is_some_and(|card_state| {
                    card_state.zone == discard
                        && crate::query::effective_card_kind(tx.state(), player, card_state)
                            .is_sha()
                })
            {
                return Err(EffectError::Illegal(
                    "mbanxian selected Sha is no longer gainable".into(),
                ));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("mbanxian owner hand missing".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbanxian {
                    player,
                    cards: Vec::new(),
                    card,
                    stage: 1,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![card],
                    to: hand,
                    from: Some(discard),
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 5 {
                return Err(EffectError::EngineFault(
                    "truncated mbanxian target continuation".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let card = CardId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let option_count = payload[4] as usize;
            if payload.len() != 5 + option_count
                || observer != player
                || cancelled
                || selections.is_empty()
                || !enabled(tx.state(), player, skill)
            {
                return Err(EffectError::Illegal(
                    "mbanxian target choice context changed".into(),
                ));
            }
            let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            let legal = crate::command::play::prompted_sha_targets_unlimited(tx, player, card);
            if selected.len() != selections.len()
                || !selected.iter().all(|target| payload[5..].contains(target))
                || !selections
                    .iter()
                    .all(|target| legal.contains(&PlayerId::new(*target)))
            {
                return Err(EffectError::Illegal(
                    "mbanxian Sha targets are no longer legal".into(),
                ));
            }
            crate::command::play::play_prompted_sha_source_unlimited(
                tx,
                player,
                card,
                &selections
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect::<Vec<_>>(),
            )
        }
        _ => Err(EffectError::EngineFault(
            "unknown mbanxian continuation stage".into(),
        )),
    }
}

pub(crate) fn resolve_mbjinzu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&0) || payload.len() < 7 {
        return Err(EffectError::EngineFault(
            "truncated mbjinzu hand-card continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let owner_card = CardId::new(payload[3]);
    let helper_skill_def = engine_api::SkillDefId::new(payload[4]);
    let skill = engine_api::SkillId::new(payload[5]);
    let option_count = payload[6] as usize;
    if payload.len() != 7 + option_count
        || observer != target
        || cancelled
        || selections.len() != 2
        || selections[0] == selections[1]
        || !selections
            .iter()
            .all(|selection| payload[7..].contains(selection))
    {
        return Err(EffectError::Illegal(
            "mbjinzu target card choice context changed".into(),
        ));
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, target);
    if selections.iter().any(|selection| {
        !tx.state()
            .cards
            .get(CardId::new(*selection))
            .is_some_and(|card| Some(card.zone) == hand && card.owner == target)
    }) {
        return Err(EffectError::Illegal(
            "mbjinzu selected target card left hand".into(),
        ));
    }
    let mut cards = vec![owner_card];
    cards.extend(selections.iter().copied().map(CardId::new));
    run_effects(
        tx,
        vec![EffectRequest::ResolveMbjinzu {
            player,
            target,
            cards,
            card: CardId::NONE,
            helper_skill_def,
            stage: 1,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 2 {
        return Err(EffectError::EngineFault(
            "bad full-zone discard continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let zone = match (cancelled, selections) {
        (false, [zone @ 1..=2]) => *zone as u8,
        _ => {
            return Err(EffectError::Illegal(
                "full-zone discard requires a forced hand or equipment choice".into(),
            ));
        }
    };
    if observer != player {
        return Err(EffectError::Illegal(
            "full-zone discard choice belongs to the skill owner".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::ChooseDiscardAllHandOrEquipDrawLostHpThenLoseHp {
                player,
                skill,
                zone,
                discarded_count: 0,
                ready_to_draw: false,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_target_draw_max_hp_cap_then_discard_excess(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad capped draw/discard continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let cap = payload[1];
    let skill = engine_api::SkillId::new(payload[2]);
    let target = selections.first().copied().map(PlayerId::new);
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if observer != player
        || cancelled
        || selections.len() != 1
        || cap == 0
        || !skill_enabled
        || tx.state().players.get(player).is_none()
        || !target.is_some_and(|target| {
            tx.state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive && participant.max_hp > 0)
        })
    {
        return Err(EffectError::Illegal(
            "capped draw/discard choice context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
            player,
            target: target.expect("validated capped draw target"),
            cap,
            skill,
            ready_to_discard: false,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_target_draw_max_hp_cap_then_add_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad capped draw add-skill continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let cap = payload[2];
    let add_def = engine_api::SkillDefId::new(payload[3]);
    let target = selections.first().copied().map(PlayerId::new);
    if observer != player
        || cancelled
        || selections.len() != 1
        || cap == 0
        || add_def.raw() == 0
        || tx.state().players.get(player).is_none()
        || !target.is_some_and(|target| {
            tx.state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive && participant.max_hp > 0)
        })
    {
        return Err(EffectError::Illegal(
            "capped draw add-skill choice context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ChooseTargetDrawMaxHpCapThenAddSkill {
            player,
            target: target.expect("validated capped draw target"),
            cap,
            skill: engine_api::SkillId::NONE,
            add_def,
        }],
    )
    .map(|_| ())
}
