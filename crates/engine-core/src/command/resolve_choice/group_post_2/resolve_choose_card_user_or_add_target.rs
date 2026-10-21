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

pub(crate) fn resolve_choose_target_discard_weapon_range_hand_or_increase_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let option_count = *payload
        .get(6)
        .ok_or_else(|| EffectError::EngineFault("truncated damage discard payload".into()))?
        as usize;
    if payload.len() != 7 + option_count {
        return Err(EffectError::EngineFault(
            "bad damage discard payload".into(),
        ));
    }
    let mode = payload[0];
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let damage_id = payload[3];
    let skill = engine_api::SkillId::new(payload[4]);
    let required = payload[5] as usize;
    let rule_matches = match mode {
        0 => {
            crate::effect::bound_sha_damage_matches(tx, damage_id, player, target)
                && crate::effect::physical_weapon_range(tx, player).map(usize::from)
                    == Some(required)
        }
        1 => {
            crate::effect::bound_damage_matches(tx, damage_id, player, target)
                && crate::effect::distance_hand_discard_count(tx, player, target) == Some(required)
        }
        _ => false,
    };
    if observer != target
        || player == target
        || required == 0
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
        || !rule_matches
    {
        return Err(EffectError::Illegal(
            "damage discard context changed".into(),
        ));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled damage discard contains cards".into(),
            ));
        }
        crate::effect::increase_bound_damage_by_one(tx, damage_id)?;
        return Ok(());
    }
    if selections.len() != required {
        return Err(EffectError::Illegal(format!(
            "choose exactly {required} hand cards"
        )));
    }
    let current_hand = tx.state().hand_cards(target);
    let mut seen = std::collections::BTreeSet::new();
    let mut cards = Vec::with_capacity(required);
    for raw in selections {
        let card = CardId::new(*raw);
        if !seen.insert(*raw) || !payload[7..].contains(raw) || !current_hand.contains(&card) {
            return Err(EffectError::Illegal(
                "damage discard card is invalid or stale".into(),
            ));
        }
        cards.push(card);
    }
    let mut effects = vec![EffectRequest::DiscardCards {
        player: target,
        cards,
    }];
    if mode == 0 {
        let weapons = crate::effect::physical_equipped_weapons(tx, player);
        if weapons.is_empty() {
            return Err(EffectError::Illegal(
                "weapon-range discard source weapon is missing".into(),
            ));
        }
        effects.push(EffectRequest::DiscardCards {
            player,
            cards: weapons,
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = payload.first().copied().ok_or_else(|| {
        EffectError::EngineFault("truncated attack-range damage choice payload".into())
    })?;
    if !matches!(stage, 0..=2) || payload.len() < 6 {
        return Err(EffectError::EngineFault(
            "bad attack-range damage choice payload".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let damage_id = payload[3];
    let skill = engine_api::SkillId::new(payload[4]);
    let attack_range = payload[5] as usize;
    let weapons = crate::effect::physical_equipped_weapons(tx, player);
    let context_valid = observer == target
        && player != target
        && !cancelled
        && tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        && crate::effect::bound_sha_damage_matches(tx, damage_id, player, target)
        && crate::effect::current_attack_range(tx, player) == attack_range
        && (stage == 2) == !weapons.is_empty();
    if !context_valid {
        return Err(EffectError::Illegal(
            "attack-range damage choice context changed".into(),
        ));
    }
    if stage == 0 {
        if payload.len() != 6 || selections.len() != 1 || !matches!(selections[0], 1 | 2) {
            return Err(EffectError::Illegal(
                "attack-range damage branch requires one choice".into(),
            ));
        }
        if selections[0] == 2 {
            return run_effects(
                tx,
                vec![EffectRequest::ContinueIncreaseBoundDamage {
                    player,
                    damage_id,
                    amount: 1,
                    skill,
                }],
            )
            .map(|_| ());
        }
        let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
        if options.is_empty() || attack_range == 0 {
            return Err(EffectError::Illegal(
                "attack-range discard branch is no longer available".into(),
            ));
        }
        return crate::effect::open_attack_range_discard_choice(
            tx,
            1,
            player,
            target,
            damage_id,
            skill,
            attack_range,
            options,
        )
        .map(|_| ());
    }

    let option_count = *payload
        .get(6)
        .ok_or_else(|| EffectError::EngineFault("truncated attack-range discard payload".into()))?
        as usize;
    if payload.len() != 7 + option_count {
        return Err(EffectError::EngineFault(
            "bad attack-range discard payload".into(),
        ));
    }
    let required = attack_range.min(option_count).min(usize::from(u8::MAX));
    if required == 0 || selections.len() != required {
        return Err(EffectError::Illegal(format!(
            "choose exactly {required} hand/equipment cards"
        )));
    }
    let current = crate::query::discardable_hand_equip_cards(tx.state(), target);
    let mut seen = std::collections::BTreeSet::new();
    let mut cards = Vec::with_capacity(required);
    for raw in selections {
        let card = CardId::new(*raw);
        if !seen.insert(*raw) || !payload[7..].contains(raw) || !current.contains(&card) {
            return Err(EffectError::Illegal(
                "attack-range discard card is invalid or stale".into(),
            ));
        }
        cards.push(card);
    }
    let mut effects = vec![EffectRequest::DiscardCards {
        player: target,
        cards,
    }];
    if stage == 2 {
        effects.extend([
            EffectRequest::ContinueIncreaseBoundDamage {
                player,
                damage_id,
                amount: 1,
                skill,
            },
            EffectRequest::DiscardCards {
                player,
                cards: weapons,
            },
        ]);
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_hand_count_different_target_draw_or_random_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let option_count = *payload
        .get(2)
        .ok_or_else(|| EffectError::EngineFault("truncated hand-count target payload".into()))?
        as usize;
    if payload.len() != 3 + option_count {
        return Err(EffectError::EngineFault(
            "bad hand-count target payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    if observer != player
        || cancelled
        || selections.len() != 1
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "hand-count target choice context changed".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    let player_hand_count = tx.state().hand_count(player);
    let target_hand_count = tx.state().hand_count(target);
    if !payload[3..].contains(&target.raw())
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || player_hand_count == target_hand_count
    {
        return Err(EffectError::Illegal(
            "selected hand-count target is invalid or stale".into(),
        ));
    }
    if target_hand_count < player_hand_count {
        return run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player: target,
                count: 1,
                skill,
            }],
        )
        .map(|_| ());
    }
    let hand = tx.state().hand_cards(target);
    let max = u32::try_from(hand.len())
        .map_err(|_| EffectError::EngineFault("hand exceeds deterministic RNG range".into()))?;
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let card = hand[rng.gen_range_u32(max) as usize];
    tx.state_mut().rng = rng.into_state();
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: target,
            cards: vec![card],
        }],
    )
    .map(|_| ())
}

pub(crate) fn rezhenjun_choice_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) && tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Prepare
}

pub(crate) fn decadezhenjun_choice_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) && tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && matches!(tx.state().turn.phase, Phase::Prepare | Phase::End)
}

pub(crate) fn resolve_choose_target_discard_excess_then_self_discard_or_target_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    match payload.first().copied() {
        Some(0) => {
            let option_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated prepare target payload".into())
            })? as usize;
            if payload.len() != 4 + option_count {
                return Err(EffectError::EngineFault(
                    "bad prepare target payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = engine_api::SkillId::new(payload[2]);
            if observer != player
                || cancelled
                || selections.len() != 1
                || !rezhenjun_choice_context_is_current(tx, player, skill)
            {
                return Err(EffectError::Illegal(
                    "prepare target choice context changed".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if !payload[4..].contains(&target.raw())
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || options.is_empty()
            {
                return Err(EffectError::Illegal(
                    "selected prepare target is invalid or stale".into(),
                ));
            }
            let target_hand = i32::try_from(tx.state().hand_count(target)).unwrap_or(i32::MAX);
            let target_hp = tx
                .state()
                .players
                .get(target)
                .map(|participant| participant.hp)
                .unwrap_or(0);
            let requested = target_hand.saturating_sub(target_hp).max(1) as usize;
            let required = requested.min(options.len()).min(usize::from(u8::MAX));
            let mut next = vec![
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                required as u32,
                options.len() as u32,
            ];
            next.extend(options.iter().map(|card| card.raw()));
            crate::effect::open_choice(
                tx,
                player,
                required as u8,
                required as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
                next,
            )?;
            Ok(())
        }
        Some(1) => {
            let required = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated target-card discard payload".into())
            })? as usize;
            let option_count = *payload.get(5).ok_or_else(|| {
                EffectError::EngineFault("truncated target-card discard options".into())
            })? as usize;
            if payload.len() != 6 + option_count {
                return Err(EffectError::EngineFault(
                    "bad target-card discard payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            if observer != player
                || cancelled
                || selections.len() != required
                || required == 0
                || !rezhenjun_choice_context_is_current(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "target-card discard choice context changed".into(),
                ));
            }
            let current = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let mut seen = std::collections::BTreeSet::new();
            let mut cards = Vec::with_capacity(required);
            let mut non_equipment = 0u32;
            for raw in selections {
                let card = CardId::new(*raw);
                if !seen.insert(*raw) || !payload[6..].contains(raw) || !current.contains(&card) {
                    return Err(EffectError::Illegal(
                        "target-card discard selection is invalid or stale".into(),
                    ));
                }
                let card_state =
                    tx.state().cards.get(card).ok_or_else(|| {
                        EffectError::Illegal("missing selected target card".into())
                    })?;
                if !card_state.kind.is_equip() {
                    non_equipment = non_equipment.saturating_add(1);
                }
                cards.push(card);
            }
            let mut effects = vec![EffectRequest::DiscardTargetCards {
                actor: player,
                target,
                cards,
                scope: rule_api::TargetCardScope::HandEquip,
            }];
            if non_equipment > 0 {
                effects.push(
                    EffectRequest::ChooseSelfDiscardOrTargetDrawAfterTargetDiscard {
                        player,
                        target,
                        count: non_equipment,
                        skill,
                    },
                );
            }
            run_effects(tx, effects).map(|_| ())
        }
        Some(2) => {
            let count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated self-discard branch payload".into())
            })?;
            let option_count = *payload.get(5).ok_or_else(|| {
                EffectError::EngineFault("truncated self-discard branch options".into())
            })? as usize;
            if payload.len() != 6 + option_count {
                return Err(EffectError::EngineFault(
                    "bad self-discard branch payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            if observer != player
                || count == 0
                || !rezhenjun_choice_context_is_current(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "self-discard branch context changed".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled self-discard branch has selections".into(),
                    ));
                }
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
            if selections.len() != count as usize {
                return Err(EffectError::Illegal(
                    "self-discard branch requires the exact cost".into(),
                ));
            }
            let current = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let mut seen = std::collections::BTreeSet::new();
            let mut cards = Vec::with_capacity(selections.len());
            for raw in selections {
                let card = CardId::new(*raw);
                if !seen.insert(*raw) || !payload[6..].contains(raw) || !current.contains(&card) {
                    return Err(EffectError::Illegal(
                        "self-discard cost is invalid or stale".into(),
                    ));
                }
                cards.push(card);
            }
            run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).map(|_| ())
        }
        Some(3) => {
            let option_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated Decade Zhenjun target payload".into())
            })? as usize;
            if payload.len() != 4 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Decade Zhenjun target payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let skill = engine_api::SkillId::new(payload[2]);
            if observer != player
                || cancelled
                || selections.len() != 1
                || !decadezhenjun_choice_context_is_current(tx, player, skill)
            {
                return Err(EffectError::Illegal(
                    "Decade Zhenjun target choice context changed".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            if !payload[4..].contains(&target.raw())
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || options.is_empty()
            {
                return Err(EffectError::Illegal(
                    "selected Decade Zhenjun target is invalid or stale".into(),
                ));
            }
            let target_hand = i32::try_from(tx.state().hand_count(target)).unwrap_or(i32::MAX);
            let target_hp = tx
                .state()
                .players
                .get(target)
                .map(|participant| participant.hp)
                .unwrap_or(0);
            let requested = target_hand.saturating_sub(target_hp).max(1) as usize;
            let required = requested.min(options.len()).min(usize::from(u8::MAX));
            let mut next = vec![
                4,
                player.raw(),
                target.raw(),
                skill.raw(),
                required as u32,
                options.len() as u32,
            ];
            next.extend(options.iter().map(|card| card.raw()));
            crate::effect::open_choice(
                tx,
                player,
                required as u8,
                required as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
                next,
            )?;
            Ok(())
        }
        Some(4) => {
            let required = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Decade Zhenjun card payload".into())
            })? as usize;
            let option_count = *payload.get(5).ok_or_else(|| {
                EffectError::EngineFault("truncated Decade Zhenjun card options".into())
            })? as usize;
            if payload.len() != 6 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Decade Zhenjun card payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            if observer != player
                || cancelled
                || selections.len() != required
                || required == 0
                || !decadezhenjun_choice_context_is_current(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Decade Zhenjun card choice context changed".into(),
                ));
            }
            let current = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
            let mut seen = std::collections::BTreeSet::new();
            let mut cards = Vec::with_capacity(required);
            let mut has_equipment = false;
            for raw in selections {
                let card = CardId::new(*raw);
                if !seen.insert(*raw) || !payload[6..].contains(raw) || !current.contains(&card) {
                    return Err(EffectError::Illegal(
                        "Decade Zhenjun card selection is invalid or stale".into(),
                    ));
                }
                let card_state =
                    tx.state().cards.get(card).ok_or_else(|| {
                        EffectError::Illegal("missing Decade Zhenjun card".into())
                    })?;
                has_equipment |= card_state.kind.is_equip();
                cards.push(card);
            }
            if !has_equipment {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveDecadezhenjun {
                        player,
                        target,
                        stage: 1,
                        count: required as u32,
                        skill,
                    },
                );
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardTargetCards {
                    actor: player,
                    target,
                    cards,
                    scope: rule_api::TargetCardScope::HandEquip,
                }],
            )
            .map(|_| ())
        }
        Some(5) => {
            let count = *payload.get(4).ok_or_else(|| {
                EffectError::EngineFault("truncated Decade Zhenjun owner branch".into())
            })?;
            let option_count = *payload.get(5).ok_or_else(|| {
                EffectError::EngineFault("truncated Decade Zhenjun owner options".into())
            })? as usize;
            if payload.len() != 6 + option_count {
                return Err(EffectError::EngineFault(
                    "bad Decade Zhenjun owner payload".into(),
                ));
            }
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            if observer != player
                || count == 0
                || !decadezhenjun_choice_context_is_current(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Decade Zhenjun owner branch context changed".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled Decade Zhenjun branch has selections".into(),
                    ));
                }
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
            let [raw] = selections else {
                return Err(EffectError::Illegal(
                    "Decade Zhenjun owner branch requires exactly one card".into(),
                ));
            };
            let card = CardId::new(*raw);
            let current = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if !payload[6..].contains(raw) || !current.contains(&card) {
                return Err(EffectError::Illegal(
                    "Decade Zhenjun owner discard is invalid or stale".into(),
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
            "unknown prepare target discard stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_gain_one_from_damage_target_and_neighbors(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(
        *payload
            .first()
            .ok_or_else(|| EffectError::EngineFault("truncated sequential gain payload".into()))?,
    );
    let skill = engine_api::SkillId::new(
        *payload
            .get(1)
            .ok_or_else(|| EffectError::EngineFault("truncated sequential gain skill".into()))?,
    );
    let index = *payload
        .get(2)
        .ok_or_else(|| EffectError::EngineFault("truncated sequential gain index".into()))?;
    let target_count = *payload
        .get(3)
        .ok_or_else(|| EffectError::EngineFault("truncated sequential gain targets".into()))?
        as usize;
    let option_count_index = 4usize
        .checked_add(target_count)
        .ok_or_else(|| EffectError::EngineFault("oversized sequential gain payload".into()))?;
    let option_count = *payload
        .get(option_count_index)
        .ok_or_else(|| EffectError::EngineFault("truncated sequential gain options".into()))?
        as usize;
    if payload.len() != option_count_index + 1 + option_count {
        return Err(EffectError::EngineFault(
            "bad sequential gain payload".into(),
        ));
    }
    if observer != player
        || cancelled
        || selections.len() != 1
        || !crate::effect::eu_tuxi_context_is_current(tx, player, skill)
    {
        return Err(EffectError::Illegal(
            "sequential gain choice context changed".into(),
        ));
    }
    let targets: Vec<_> = payload[4..option_count_index]
        .iter()
        .map(|raw| PlayerId::new(*raw))
        .collect();
    let current = targets
        .get(index as usize)
        .copied()
        .ok_or_else(|| EffectError::Illegal("sequential gain index is stale".into()))?;
    let selected = CardId::new(selections[0]);
    let snapshotted_options = &payload[option_count_index + 1..];
    let current_options = crate::effect::eu_tuxi_gain_options(tx, player, current);
    if !snapshotted_options.contains(&selected.raw()) || !current_options.contains(&selected) {
        return Err(EffectError::Illegal(
            "sequential gain selection is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors {
                player,
                targets,
                index,
                selected,
                skill,
            },
        ],
    )
    .map(|_| ())
}
