//! Command operations for `resolve_choose_card_user_or_add_target` and related helpers.

use super::*;

pub(crate) fn resolve_choose_card_user_or_add_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3
        || observer != PlayerId::new(payload[0])
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal(
            "choose exactly one alternate card user or target".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let card = CardId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let target = PlayerId::new(selections[0]);
    if tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player
                && instance.play_phase_fire_count == 0
                && tx.state().skill_effectively_enabled(instance)
        })
        || !crate::command::play::saved_card_extra_target_is_legal(tx, player, card, target)
    {
        return Err(EffectError::Illegal(
            "alternate card user or target context changed".into(),
        ));
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .play_phase_fire_count = 1;
    let cards = tx.state().hand_cards(target);
    if cards.is_empty() {
        return run_effects(
            tx,
            vec![EffectRequest::CommitAddSkillBoundCardTarget {
                player,
                card,
                target,
                skill,
            }],
        )
        .map(|_| ());
    }
    crate::effect::open_choice(
        tx,
        target,
        0,
        1,
        true,
        cards.into_iter().map(CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::ChooseGiveCardOrAddTarget,
        vec![
            player.raw(),
            card.raw(),
            skill.raw(),
            target.raw(),
            tx.state().turn.current.raw(),
            u32::from(tx.state().turn.phase.to_u8()),
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_give_card_or_add_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6 || selections.len() > 1 || (cancelled && !selections.is_empty()) {
        return Err(EffectError::Illegal(
            "give one hand card or become an extra target".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let card = CardId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let current = PlayerId::new(payload[4]);
    let phase = Phase::from_u8(payload[5] as u8);
    if observer != target
        || tx.state().turn.current != current
        || tx.state().turn.phase != phase
        || current != player
        || phase != Phase::Play
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player
                && instance.play_phase_fire_count == 1
                && tx.state().skill_effectively_enabled(instance)
        })
        || !crate::command::play::saved_card_extra_target_is_legal(tx, player, card, target)
    {
        return Err(EffectError::Illegal(
            "give-card or extra-target context changed".into(),
        ));
    }
    let Some(raw) = selections.first().copied() else {
        return run_effects(
            tx,
            vec![EffectRequest::CommitAddSkillBoundCardTarget {
                player,
                card,
                target,
                skill,
            }],
        )
        .map(|_| ());
    };
    let gift = CardId::new(raw);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("gift source hand missing".into()))?;
    if !tx
        .state()
        .cards
        .get(gift)
        .is_some_and(|card| card.zone == hand && card.owner == target)
    {
        return Err(EffectError::Illegal(
            "selected card is no longer in the alternate user's hand".into(),
        ));
    }
    let recipient = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("gift recipient hand missing".into()))?;
    run_effects(
        tx,
        vec![
            EffectRequest::GiveCards {
                cards: vec![gift],
                to: recipient,
                from: hand,
            },
            EffectRequest::CommitTransferSavedCardUser {
                previous: player,
                new_user: target,
                card,
                skill,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_discard_equipment_reduce_current_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if !(3..=4).contains(&payload.len()) || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one equipment card to reduce damage".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let damage_id = payload[1];
    let skill = engine_api::SkillId::new(payload[2]);
    let prevent = payload.get(3).copied().unwrap_or(0) != 0;
    let card = CardId::new(selections[0]);
    if observer != player
        || tx.state().resolution.current_damage_id != damage_id
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .any(|candidate| {
                candidate == card
                    && tx.state().cards.get(candidate).is_some_and(|card_state| {
                        crate::query::effective_card_kind(tx.state(), player, card_state).is_equip()
                    })
            })
    {
        return Err(EffectError::Illegal(
            "equipment damage-reduction context changed".into(),
        ));
    }
    let discard = EffectRequest::DiscardCards {
        player,
        cards: vec![card],
    };
    if prevent {
        run_effects(
            tx,
            vec![discard, EffectRequest::PreventDamage { damage_id }],
        )
        .map(|_| ())
    } else {
        crate::effect::reduce_bound_damage_by_one(tx, damage_id)?;
        run_effects(tx, vec![discard]).map(|_| ())
    }
}

pub(crate) fn resolve_choose_discard_damage_endpoint_card_increase_if_suit(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 7 || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one damage-endpoint card".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let damage_id = payload[2];
    let suit = payload[3];
    let amount = payload[4];
    let skill = engine_api::SkillId::new(payload[5]);
    let option_count = payload[6] as usize;
    let card = CardId::new(selections[0]);
    if payload.len() != 7 + option_count
        || observer != player
        || !(1..=4).contains(&suit)
        || amount == 0
        || !payload[7..].contains(&card.raw())
        || !crate::effect::bound_sha_damage_matches_any_segment(tx, damage_id, player, target)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !crate::effect::damage_endpoint_discard_cards(tx, player, target).contains(&card)
    {
        return Err(EffectError::Illegal(
            "damage-endpoint discard context changed".into(),
        ));
    }
    let card_owner = tx
        .state()
        .cards
        .get(card)
        .map(|state| state.owner)
        .ok_or_else(|| EffectError::Illegal("damage-endpoint card is missing".into()))?;
    let matches_suit = tx.state().cards.get(card).is_some_and(|state| {
        crate::query::effective_card_suit(tx.state(), card_owner, state)
            == crate::command::judge::suit_from_tag(suit)
    });
    if matches_suit {
        tx.state_mut().resolution.deferred_effects.push(
            EffectRequest::ContinueIncreaseBoundDamage {
                player,
                damage_id,
                amount,
                skill,
            },
        );
    }
    run_effects(
        tx,
        vec![EffectRequest::DiscardTargetCards {
            actor: player,
            target: card_owner,
            cards: vec![card],
            scope: rule_api::TargetCardScope::HandEquip,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_move_target_equipment_or_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player
        || player == target
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
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
            "move-equipment-or-draw context changed".into(),
        ));
    }
    match stage {
        0 | 4 => {
            if payload.len() != 4 {
                return Err(EffectError::EngineFault(
                    "bad move-equipment-or-draw branch payload".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            let [branch] = selections else {
                return Err(EffectError::Illegal(
                    "choose exactly one move-equipment-or-draw branch".into(),
                ));
            };
            match *branch {
                1 => run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 1,
                        skill,
                    }],
                )
                .map(|_| ()),
                0 => {
                    let cards =
                        crate::query::movable_target_equipment_cards(tx.state(), player, target);
                    if cards.is_empty() {
                        return Err(EffectError::Illegal(
                            "no compatible target equipment remains".into(),
                        ));
                    }
                    let mut next = vec![
                        1,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        cards.len().min(u32::MAX as usize) as u32,
                    ];
                    next.extend(cards.iter().map(|card| card.raw()));
                    crate::effect::open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        cards.into_iter().map(CardId::raw).collect(),
                        engine_api::ChoiceKind::SelectCards,
                        ContinuationKind::ChooseMoveTargetEquipmentOrDraw,
                        next,
                    )
                    .map(|_| ())
                }
                2 if stage == 4 => {
                    let mut cards = tx.state().hand_cards(target);
                    if cards.is_empty() {
                        return Err(EffectError::Illegal(
                            "target no longer has hand cards to inspect".into(),
                        ));
                    }
                    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
                    rng.shuffle(&mut cards);
                    tx.state_mut().rng = rng.into_state();
                    cards.truncate(2);
                    let mut next = vec![
                        5,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        cards.len() as u32,
                    ];
                    next.extend(cards.iter().map(|card| card.raw()));
                    crate::effect::open_choice(
                        tx,
                        player,
                        1,
                        1,
                        false,
                        cards.into_iter().map(CardId::raw).collect(),
                        engine_api::ChoiceKind::SelectCards,
                        ContinuationKind::ChooseMoveTargetEquipmentOrDraw,
                        next,
                    )
                    .map(|_| ())
                }
                _ => Err(EffectError::Illegal(
                    "unknown move-equipment-or-draw branch".into(),
                )),
            }
        }
        1 | 2 => {
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated equipment-move card options".into())
            })? as usize;
            if payload.len() != 5 + option_count || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose exactly one compatible equipment card".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            let current = crate::query::movable_target_equipment_cards(tx.state(), player, target);
            if !payload[5..].contains(&card.raw()) || !current.contains(&card) {
                return Err(EffectError::Illegal(
                    "selected equipment card is no longer movable".into(),
                ));
            }
            let mut effects = crate::effect::move_target_equipment_to_player_effects(
                tx.state(),
                player,
                target,
                card,
            )?;
            if stage == 2 {
                effects.push(EffectRequest::DrawCardsFromSkill {
                    player,
                    count: 1,
                    skill,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        3 => {
            if payload.len() != 4 {
                return Err(EffectError::EngineFault(
                    "bad move-equipment-and-or-draw branch payload".into(),
                ));
            }
            if cancelled {
                return Ok(());
            }
            if selections.is_empty()
                || selections.len() > 2
                || selections
                    .iter()
                    .any(|selection| !matches!(selection, 0 | 1))
                || (selections.len() == 2 && selections[0] == selections[1])
            {
                return Err(EffectError::Illegal(
                    "choose one or two distinct move-equipment-and-or-draw branches".into(),
                ));
            }
            let move_selected = selections.contains(&0);
            let draw_selected = selections.contains(&1);
            if !move_selected {
                return run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 1,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            let cards = crate::query::movable_target_equipment_cards(tx.state(), player, target);
            if cards.is_empty() {
                return Err(EffectError::Illegal(
                    "no compatible target equipment remains".into(),
                ));
            }
            let mut next = vec![
                if draw_selected { 2 } else { 1 },
                player.raw(),
                target.raw(),
                skill.raw(),
                cards.len().min(u32::MAX as usize) as u32,
            ];
            next.extend(cards.iter().map(|card| card.raw()));
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseMoveTargetEquipmentOrDraw,
                next,
            )
            .map(|_| ())
        }
        5 => {
            let option_count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated inspected-hand card options".into())
            })? as usize;
            if payload.len() != 5 + option_count || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose exactly one inspected hand card".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !payload[5..].contains(&card.raw()) || !tx.state().hand_cards(target).contains(&card)
            {
                return Err(EffectError::Illegal(
                    "selected inspected hand card is invalid or stale".into(),
                ));
            }
            if crate::query::card_gain_protected_from(tx.state(), player, target, card) {
                return Ok(());
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("target hand zone is missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand zone is missing".into()))?;
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
            "unknown move-equipment-or-draw continuation stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_recover_owner_or_damage_by_given_equipment(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad equipment-gift branch payload".into(),
        ));
    }
    let owner = PlayerId::new(payload[0]);
    let recipient = PlayerId::new(payload[1]);
    let count = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != recipient
        || count == 0
        || !tx
            .state()
            .players
            .get(recipient)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == owner)
    {
        return Err(EffectError::Illegal(
            "equipment-gift branch context changed".into(),
        ));
    }
    if cancelled {
        if tx
            .state()
            .players
            .get(owner)
            .is_some_and(|participant| participant.alive)
        {
            return run_effects(
                tx,
                vec![EffectRequest::RecoverHp {
                    source: recipient,
                    player: owner,
                    amount: count,
                }],
            )
            .map(|_| ());
        }
        return Ok(());
    }
    let max = count.min(u32::from(u8::MAX)) as usize;
    if selections.is_empty() || selections.len() > max {
        return Err(EffectError::Illegal(
            "equipment-gift damage target count is invalid".into(),
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut targets = Vec::with_capacity(selections.len());
    for raw in selections {
        let target = PlayerId::new(*raw);
        if !seen.insert(*raw)
            || !crate::query::player_in_attack_range(tx.state(), recipient, target)
        {
            return Err(EffectError::Illegal(
                "equipment-gift damage target is no longer in attack range".into(),
            ));
        }
        targets.push(target);
    }
    run_effects(
        tx,
        targets
            .into_iter()
            .map(|target| EffectRequest::DealDamage {
                source: recipient,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            })
            .collect(),
    )
    .map(|_| ())
}
