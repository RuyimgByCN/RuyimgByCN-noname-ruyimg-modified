//! Choice-continuation handlers (post 2).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

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

pub(crate) fn resolve_tuxi_gain_other_hands_replace_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("truncated Tuxi payload".into()))?;
    let player = PlayerId::new(
        *payload
            .get(1)
            .ok_or_else(|| EffectError::EngineFault("truncated Tuxi player".into()))?,
    );
    let skill = engine_api::SkillId::new(
        *payload
            .get(2)
            .ok_or_else(|| EffectError::EngineFault("truncated Tuxi skill".into()))?,
    );
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "Tuxi choice cannot be cancelled".into(),
        ));
    }

    match stage {
        0 | 2 => {
            let reduce_draw = stage == 2;
            let draw_count = if reduce_draw {
                *payload
                    .get(3)
                    .ok_or_else(|| EffectError::EngineFault("truncated Tuxi draw count".into()))?
            } else {
                0
            };
            let option_count_index = if reduce_draw { 4 } else { 3 };
            let option_count = *payload
                .get(option_count_index)
                .ok_or_else(|| EffectError::EngineFault("truncated Tuxi targets".into()))?
                as usize;
            let first_option = option_count_index + 1;
            let max_targets = if reduce_draw {
                option_count.min(draw_count as usize)
            } else {
                option_count.min(2)
            };
            if payload.len() != first_option + option_count
                || selections.is_empty()
                || selections.len() > max_targets
            {
                return Err(EffectError::Illegal(
                    "Tuxi target count is outside the pending draw bound".into(),
                ));
            }
            let snapshotted = &payload[first_option..];
            let mut seen = std::collections::BTreeSet::new();
            let mut targets: Vec<_> = selections
                .iter()
                .map(|raw| {
                    let target = PlayerId::new(*raw);
                    if !seen.insert(*raw)
                        || !snapshotted.contains(raw)
                        || target == player
                        || !tx
                            .state()
                            .players
                            .get(target)
                            .is_some_and(|participant| participant.alive)
                        || tx.state().hand_count(target) == 0
                    {
                        return Err(EffectError::Illegal(
                            "Tuxi target is invalid or stale".into(),
                        ));
                    }
                    Ok(target)
                })
                .collect::<Result<_, _>>()?;
            targets.sort_by_key(|target| {
                tx.state()
                    .players
                    .get(*target)
                    .map_or(u8::MAX, |participant| participant.seat.raw())
            });
            run_effects(
                tx,
                vec![if reduce_draw {
                    EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
                        player,
                        targets,
                        index: 0,
                        selected: CardId::NONE,
                        draw_count,
                        skill,
                    }
                } else {
                    EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
                        player,
                        targets,
                        index: 0,
                        selected: CardId::NONE,
                        skill,
                    }
                }],
            )
            .map(|_| ())
        }
        1 | 3 => {
            let reduce_draw = stage == 3;
            let draw_count = if reduce_draw {
                *payload
                    .get(3)
                    .ok_or_else(|| EffectError::EngineFault("truncated Tuxi draw count".into()))?
            } else {
                0
            };
            let index_index = if reduce_draw { 4 } else { 3 };
            let index = *payload
                .get(index_index)
                .ok_or_else(|| EffectError::EngineFault("truncated Tuxi index".into()))?;
            let target_count = *payload
                .get(index_index + 1)
                .ok_or_else(|| EffectError::EngineFault("truncated Tuxi target count".into()))?
                as usize;
            let first_target = index_index + 2;
            let option_count_index = first_target
                .checked_add(target_count)
                .ok_or_else(|| EffectError::EngineFault("oversized Tuxi payload".into()))?;
            let option_count = *payload
                .get(option_count_index)
                .ok_or_else(|| EffectError::EngineFault("truncated Tuxi card options".into()))?
                as usize;
            if payload.len() != option_count_index + 1 + option_count || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Tuxi requires exactly one target hand card".into(),
                ));
            }
            let targets: Vec<_> = payload[first_target..option_count_index]
                .iter()
                .map(|raw| PlayerId::new(*raw))
                .collect();
            let target = targets
                .get(index as usize)
                .copied()
                .ok_or_else(|| EffectError::Illegal("Tuxi target index is stale".into()))?;
            let selected = CardId::new(selections[0]);
            if !payload[option_count_index + 1..].contains(&selected.raw())
                || !tx.state().hand_cards(target).contains(&selected)
            {
                return Err(EffectError::Illegal(
                    "Tuxi hand-card selection is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![if reduce_draw {
                    EffectRequest::ChooseOtherHandTargetsGainOneReduceDraw {
                        player,
                        targets,
                        index,
                        selected,
                        draw_count,
                        skill,
                    }
                } else {
                    EffectRequest::ChooseOtherHandTargetsGainOneReplaceDraw {
                        player,
                        targets,
                        index,
                        selected,
                        skill,
                    }
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown Tuxi stage".into())),
    }
}

pub(crate) fn resolve_gain_one_hand_from_selected_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(
        *payload
            .first()
            .ok_or_else(|| EffectError::EngineFault("truncated Cuorui player".into()))?,
    );
    let skill = engine_api::SkillId::new(
        *payload
            .get(1)
            .ok_or_else(|| EffectError::EngineFault("truncated Cuorui skill".into()))?,
    );
    let index = *payload
        .get(2)
        .ok_or_else(|| EffectError::EngineFault("truncated Cuorui index".into()))?;
    let target_count = *payload
        .get(3)
        .ok_or_else(|| EffectError::EngineFault("truncated Cuorui target count".into()))?
        as usize;
    let option_count_index = 4usize
        .checked_add(target_count)
        .ok_or_else(|| EffectError::EngineFault("oversized Cuorui payload".into()))?;
    let option_count = *payload
        .get(option_count_index)
        .ok_or_else(|| EffectError::EngineFault("truncated Cuorui card options".into()))?
        as usize;
    if observer != player
        || cancelled
        || selections.len() != 1
        || payload.len() != option_count_index + 1 + option_count
    {
        return Err(EffectError::Illegal(
            "Cuorui requires exactly one target hand card".into(),
        ));
    }
    let targets: Vec<_> = payload[4..option_count_index]
        .iter()
        .map(|raw| PlayerId::new(*raw))
        .collect();
    let target = targets
        .get(index as usize)
        .copied()
        .ok_or_else(|| EffectError::Illegal("Cuorui target index is stale".into()))?;
    let selected = CardId::new(selections[0]);
    if !payload[option_count_index + 1..].contains(&selected.raw())
        || !tx.state().hand_cards(target).contains(&selected)
    {
        return Err(EffectError::Illegal(
            "Cuorui hand-card selection is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::GainOneHandFromSelectedTargets {
            player,
            targets,
            index,
            selected,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_yiji_distribute_top_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("truncated Yiji payload".into()))?;
    let player = PlayerId::new(
        *payload
            .get(1)
            .ok_or_else(|| EffectError::EngineFault("truncated Yiji player".into()))?,
    );
    let skill = engine_api::SkillId::new(
        *payload
            .get(2)
            .ok_or_else(|| EffectError::EngineFault("truncated Yiji skill".into()))?,
    );
    let card_count = *payload
        .get(3)
        .ok_or_else(|| EffectError::EngineFault("truncated Yiji card count".into()))?
        as usize;
    let cards_end = 4usize
        .checked_add(card_count)
        .ok_or_else(|| EffectError::EngineFault("oversized Yiji payload".into()))?;
    if observer != player || cancelled || payload.len() < cards_end {
        return Err(EffectError::Illegal(
            "Yiji choice cannot be cancelled".into(),
        ));
    }
    let cards: Vec<_> = payload[4..cards_end]
        .iter()
        .map(|raw| CardId::new(*raw))
        .collect();

    match stage {
        0 => {
            if payload.len() != cards_end || selections.is_empty() || selections.len() > cards.len()
            {
                return Err(EffectError::Illegal(
                    "Yiji requires a nonempty remaining-card subset".into(),
                ));
            }
            let mut seen = std::collections::BTreeSet::new();
            let selected: Vec<_> = selections
                .iter()
                .map(|raw| {
                    let card = CardId::new(*raw);
                    if !seen.insert(*raw) || !cards.contains(&card) {
                        return Err(EffectError::Illegal(
                            "Yiji subset is invalid or duplicated".into(),
                        ));
                    }
                    Ok(card)
                })
                .collect::<Result<_, _>>()?;
            run_effects(
                tx,
                vec![EffectRequest::DistributeTopTwoCards {
                    player,
                    cards,
                    selected,
                    target: PlayerId::NONE,
                    skill,
                    ready_to_choose: true,
                }],
            )
            .map(|_| ())
        }
        1 => {
            let selected_count = *payload
                .get(cards_end)
                .ok_or_else(|| EffectError::EngineFault("truncated Yiji subset".into()))?
                as usize;
            if payload.len() != cards_end + 1 + selected_count || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Yiji requires exactly one recipient".into(),
                ));
            }
            let selected: Vec<_> = payload[cards_end + 1..]
                .iter()
                .map(|raw| CardId::new(*raw))
                .collect();
            let target = PlayerId::new(selections[0]);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "Yiji recipient is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DistributeTopTwoCards {
                    player,
                    cards,
                    selected,
                    target,
                    skill,
                    ready_to_choose: true,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown Yiji stage".into())),
    }
}

pub(crate) fn resolve_xiaoguo_discard_basic_then_equip_or_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, target, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Xiaoguo continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let target = PlayerId::new(*target);
    let skill = engine_api::SkillId::new(*skill);
    match stage {
        0 => {
            let current = crate::query::standard_xiaoguo_basic_hand_cards(tx.state(), player);
            let valid = observer == player
                && tx.state().turn.current == target
                && tx.state().turn.phase == Phase::End
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|state| state.alive)
                && tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                });
            if !valid {
                return Err(EffectError::Illegal("Xiaoguo cost context changed".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled Xiaoguo cost contains a card".into(),
                    ));
                }
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Xiaoguo requires exactly one basic hand card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            if !current.contains(&selected) {
                return Err(EffectError::Illegal(
                    "Xiaoguo basic-card cost is invalid or stale".into(),
                ));
            }
            match run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![selected],
                }],
            ) {
                Ok(_) => crate::effect::open_xiaoguo_target_equipment_or_damage(
                    tx, player, target, skill,
                ),
                Err(EffectError::ChoiceRequested) => {
                    tx.state_mut().resolution.resume_kind =
                        ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage;
                    tx.state_mut().resolution.resume_payload =
                        vec![1, player.raw(), target.raw(), skill.raw()];
                    Err(EffectError::ChoiceRequested)
                }
                Err(error) => Err(error),
            }
        }
        2 => {
            if observer != target
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|state| state.alive)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|state| state.alive)
            {
                return Err(EffectError::Illegal(
                    "Xiaoguo consequence context changed".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled Xiaoguo equipment choice contains a card".into(),
                    ));
                }
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )
                .map(|_| ());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "Xiaoguo requires exactly one equipment card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            if !crate::query::standard_xiaoguo_equipment_cards(tx.state(), target)
                .contains(&selected)
            {
                return Err(EffectError::Illegal(
                    "Xiaoguo equipment choice is invalid or stale".into(),
                ));
            }
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
            "invalid Xiaoguo choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_sp_xiaoguo(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [stage, player, target, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad SP Xiaoguo continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let target = PlayerId::new(*target);
    let skill = engine_api::SkillId::new(*skill);
    match stage {
        0 => {
            let valid = observer == player
                && player != target
                && tx.state().turn.current == target
                && tx.state().turn.phase == Phase::End
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
                && tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                });
            if !valid {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo cost context changed".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled SP Xiaoguo cost contains a card".into(),
                    ));
                }
                return Ok(());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo requires exactly one hand card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            if !crate::query::sp_xiaoguo_hand_cards(tx.state(), player).contains(&selected) {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo hand-card cost is invalid or stale".into(),
                ));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveSpXiaoguo {
                    player,
                    target,
                    stage: 1,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![selected],
                }],
            )
            .map(|_| ())
        }
        2 => {
            if observer != target
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
                    "SP Xiaoguo consequence context changed".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled SP Xiaoguo equipment choice contains a card".into(),
                    ));
                }
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveSpXiaoguo {
                        player,
                        target,
                        stage: 2,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo requires exactly one equipment card".into(),
                ));
            };
            let selected = CardId::new(*selected);
            if !crate::query::sp_xiaoguo_equipment_cards(tx.state(), target).contains(&selected) {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo equipment choice is invalid or stale".into(),
                ));
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveSpXiaoguo {
                    player,
                    target,
                    stage: 3,
                    skill,
                });
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
            "invalid SP Xiaoguo choice stage".into(),
        )),
    }
}

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

pub(crate) fn draw_give_or_discard_hand_cards(
    tx: &Transaction,
    player: PlayerId,
    discardable_only: bool,
) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let mut cards = if discardable_only {
        crate::query::discardable_hand_equip_cards(tx.state(), player)
    } else {
        tx.state().hand_cards(player)
    };
    cards.retain(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
    });
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}
