//! Command operations for `resolve_reveal_category_distribution` and related helpers.

use super::*;

pub(crate) fn resolve_resolve_hfjieying(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad Hfjieying continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let grant_def = engine_api::SkillDefId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player
        || cancelled
        || selections.len() != 1
        || grant_def.is_none()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Hfjieying grant context changed".into(),
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
            "Hfjieying grant target is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveHfjieying {
            player,
            target,
            grant_def,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_pothuanjie_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5
        || observer != PlayerId::new(payload[1])
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("invalid 共谋 target choice".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolvePothuanjie {
            player: PlayerId::new(payload[1]),
            target: PlayerId::new(selections[0]),
            stage: 1,
            qice_def: engine_api::SkillDefId::new(payload[2]),
            kanpo_def: engine_api::SkillDefId::new(payload[3]),
            skill: engine_api::SkillId::new(payload[4]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_potnigu_gift(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 {
        return Err(EffectError::EngineFault("bad 逆固 continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let index = u8::try_from(payload[3])
        .map_err(|_| EffectError::EngineFault("bad 逆固 target index".into()))?;
    let declined = u8::try_from(payload[4])
        .map_err(|_| EffectError::EngineFault("bad 逆固 refusal count".into()))?;
    let count = usize::try_from(payload[5])
        .map_err(|_| EffectError::EngineFault("bad 逆固 target count".into()))?;
    if payload.len() != 6 + count || usize::from(index) >= count {
        return Err(EffectError::EngineFault("bad 逆固 target payload".into()));
    }
    let targets: Vec<_> = payload[6..].iter().copied().map(PlayerId::new).collect();
    let target = targets[usize::from(index)];
    if observer != target || selections.len() > 1 || (!cancelled && selections.len() != 1) {
        return Err(EffectError::Illegal("invalid 逆固 gift choice".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolvePotnigu {
            player,
            target,
            card: selections
                .first()
                .copied()
                .map(CardId::new)
                .unwrap_or(CardId::NONE),
            targets,
            index,
            declined,
            stage: 1,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_potlulian_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4
        || observer != PlayerId::new(payload[1])
        || cancelled
        || selections.len() != 1
    {
        return Err(EffectError::Illegal("invalid 戮连 damage target".into()));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolvePotlulian {
            player: PlayerId::new(payload[1]),
            card: CardId::new(payload[2]),
            target: PlayerId::new(selections[0]),
            stage: 2,
            skill: engine_api::SkillId::new(payload[3]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_potdimeng_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [marker, player_raw, lower_raw, lost_hp, discard_count, mode, skill_raw] = payload else {
        return Err(EffectError::EngineFault(
            "bad 缔盟 continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let lower = PlayerId::new(*lower_raw);
    let skill = SkillId::new(*skill_raw);
    if *marker != crate::effect::POTDIMENG_CUSTOM_CHOICE
        || observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("invalid 缔盟 chooser".into()));
    }
    let draw_lower = |tx: &mut Transaction| -> Result<(), EffectError> {
        if lower.is_none()
            || !tx
                .state()
                .players
                .get(lower)
                .is_some_and(|participant| participant.alive)
        {
            return Ok(());
        }
        run_effects(
            tx,
            vec![EffectRequest::DrawCards {
                player: lower,
                count: *lost_hp,
            }],
        )
        .map(|_| ())
    };
    if cancelled {
        return if *mode == 1 { draw_lower(tx) } else { Ok(()) };
    }
    if *mode == 2 {
        if selections != [1] {
            return Err(EffectError::Illegal("invalid 缔盟 draw choice".into()));
        }
        return draw_lower(tx);
    }
    if !matches!(*mode, 0 | 1) || selections.len() != *discard_count as usize {
        return Err(EffectError::Illegal("invalid 缔盟 discard count".into()));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(CardId::raw)
            .collect();
    let mut unique = std::collections::BTreeSet::new();
    if !selections
        .iter()
        .all(|card| allowed.contains(card) && unique.insert(*card))
    {
        return Err(EffectError::Illegal(
            "缔盟 discard cards became illegal".into(),
        ));
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

#[allow(unused_variables)]

pub(crate) fn resolve_pothaoshi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [marker, player_raw, skill_raw] = payload else {
        return Err(EffectError::EngineFault(
            "bad 好施 continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let skill = SkillId::new(*skill_raw);
    if *marker != crate::effect::POTHAOSHI_CUSTOM_CHOICE
        || observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal("invalid 好施 chooser".into()));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled 好施 selected a target".into(),
            ))
        };
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal("好施 requires one target".into()));
    };
    let target = PlayerId::new(*target_raw);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("好施 target became illegal".into()));
    }
    let updated = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| instance.set_pothaoshi_grant(target));
    if !updated {
        return Err(EffectError::EngineFault(
            "好施 authorization state is incompatible".into(),
        ));
    }
    Ok(())
}

pub(crate) fn resolve_give_selected_then_target_choose_discard_same_suit_or_lose_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 2 {
        return Err(EffectError::EngineFault(
            "bad selected-card gift continuation".into(),
        ));
    }
    let target = PlayerId::new(payload[0]);
    let suit = crate::command::judge::suit_from_tag(payload[1]);
    if observer != target
        || cancelled
        || selections.len() != 1
        || !matches!(selections[0], 0 | 1)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|state| state.alive)
    {
        return Err(EffectError::Illegal(
            "selected-card gift choice context changed".into(),
        ));
    }
    if selections[0] == 1 {
        return run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            }],
        )
        .map(|_| ());
    }

    let revealed = tx.state().hand_cards(target);
    tx.push_event(engine_api::DomainEvent::CardsRevealed {
        cards: revealed,
        to: Vec::new(),
    });
    let cards: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), target)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| state.suit == suit)
        })
        .collect();
    if cards.is_empty() {
        return Ok(());
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

pub(crate) fn resolve_fanjian_choose_suit_or_hand(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let valid_selection = !cancelled && selections.len() == 1;
    match payload {
        [0, player, target, skill] => {
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            let skill = engine_api::SkillId::new(*skill);
            let suit = selections.first().copied().unwrap_or_default();
            let valid = valid_selection
                && observer == target
                && matches!(suit, 1..=4)
                && tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
                && player != target
                && [player, target].into_iter().all(|participant| {
                    tx.state()
                        .players
                        .get(participant)
                        .is_some_and(|state| state.alive)
                });
            let options = tx.state().hand_cards(player);
            if !valid || options.is_empty() {
                return Err(EffectError::Illegal(
                    "fanjian suit choice context changed".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                target,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::FanjianChooseSuitOrHand,
                vec![1, player.raw(), target.raw(), skill.raw(), suit],
            )
            .map(|_| ())
        }
        [1, player, target, skill, suit] => {
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            let skill = engine_api::SkillId::new(*skill);
            let card = CardId::new(selections.first().copied().unwrap_or_default());
            let from = tx.state().zones.find(ZoneKind::Hand, player);
            let to = tx.state().zones.find(ZoneKind::Hand, target);
            let card_suit = from.and_then(|from| {
                tx.state()
                    .cards
                    .get(card)
                    .filter(|state| state.owner == player && state.zone == from)
                    .map(|state| crate::query::effective_card_suit(tx.state(), player, state))
            });
            let valid = valid_selection
                && observer == target
                && tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == player && tx.state().skill_effectively_enabled(instance)
                })
                && player != target
                && [player, target].into_iter().all(|participant| {
                    tx.state()
                        .players
                        .get(participant)
                        .is_some_and(|state| state.alive)
                });
            let (Some(from), Some(to), Some(card_suit)) = (from, to, card_suit) else {
                return Err(EffectError::Illegal(
                    "fanjian hand choice context changed".into(),
                ));
            };
            if !valid {
                return Err(EffectError::Illegal(
                    "fanjian hand choice context changed".into(),
                ));
            }
            let mut effects = vec![EffectRequest::GiveCards {
                cards: vec![card],
                to,
                from,
            }];
            if card_suit != crate::command::judge::suit_from_tag(*suit) {
                effects.push(EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: 0,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad fanjian continuation payload".into(),
        )),
    }
}

pub(crate) fn resolve_choose_other_draw_owner_lost_hp_turn_over(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(
        *payload
            .first()
            .ok_or_else(|| EffectError::EngineFault("missing lost-HP draw chooser".into()))?,
    );
    let skill = engine_api::SkillId::new(
        *payload
            .get(1)
            .ok_or_else(|| EffectError::EngineFault("missing lost-HP draw skill".into()))?,
    );
    if payload.len() != 2 {
        return Err(EffectError::EngineFault(
            "bad lost-HP draw continuation".into(),
        ));
    }
    if observer != player {
        return Err(EffectError::Illegal("lost-HP draw chooser mismatch".into()));
    }
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one other draw target".into(),
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
            "selected lost-HP draw target is not a living other player".into(),
        ));
    }
    let count = tx
        .state()
        .players
        .get(player)
        .map(|owner| owner.max_hp.saturating_sub(owner.hp.max(0) as u32))
        .unwrap_or(0);
    run_effects(
        tx,
        vec![
            EffectRequest::DrawCardsFromSkill {
                player: target,
                count,
                skill,
            },
            EffectRequest::TurnOver { player: target },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_move_equipment_then_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "move equipment choice requires exactly one selection".into(),
        ));
    }
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing move-equipment stage".into()))?;
    let chooser = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != chooser {
        return Err(EffectError::Illegal(
            "move equipment choice actor changed".into(),
        ));
    }
    match stage {
        0 => {
            let option_count = *payload.get(3).ok_or_else(|| {
                EffectError::EngineFault("truncated move-equipment card options".into())
            })? as usize;
            if payload.len() != 4 + option_count {
                return Err(EffectError::EngineFault(
                    "invalid move-equipment card option count".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !payload[4..].contains(&card.raw())
                || !crate::query::movable_equipment_cards(tx.state(), chooser).contains(&card)
            {
                return Err(EffectError::Illegal(
                    "equipment card is no longer movable".into(),
                ));
            }
            let source = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.owner)
                .ok_or_else(|| EffectError::Illegal("missing equipment card".into()))?;
            let destinations = crate::query::equipment_move_destinations(tx.state(), chooser, card);
            let mut next = vec![
                1,
                chooser.raw(),
                skill.raw(),
                source.raw(),
                card.raw(),
                destinations.len().min(u32::MAX as usize) as u32,
            ];
            next.extend(destinations.iter().map(|player| player.raw()));
            crate::effect::open_choice(
                tx,
                chooser,
                1,
                1,
                false,
                destinations.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::MoveEquipmentThenDraw,
                next,
            )?;
            Ok(())
        }
        1 => {
            let source = PlayerId::new(*payload.get(3).unwrap_or(&0));
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let option_count = *payload.get(5).ok_or_else(|| {
                EffectError::EngineFault("truncated move-equipment target options".into())
            })? as usize;
            if payload.len() != 6 + option_count {
                return Err(EffectError::EngineFault(
                    "invalid move-equipment target option count".into(),
                ));
            }
            let destination = PlayerId::new(selections[0]);
            let legal = crate::query::equipment_move_destinations(tx.state(), chooser, card);
            if !payload[6..].contains(&destination.raw()) || !legal.contains(&destination) {
                return Err(EffectError::Illegal(
                    "equipment destination is no longer legal".into(),
                ));
            }
            let card_state = tx
                .state()
                .cards
                .get(card)
                .ok_or_else(|| EffectError::Illegal("missing equipment card".into()))?;
            if card_state.owner != source {
                return Err(EffectError::Illegal(
                    "equipment source changed during choice".into(),
                ));
            }
            let slot = card_state
                .kind
                .equip_slot()
                .ok_or_else(|| EffectError::Illegal("selected card is not equipment".into()))?;
            let from = card_state.zone;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Equip, destination)
                .ok_or_else(|| EffectError::Illegal("missing destination equipment zone".into()))?;
            let replaced: Vec<_> = tx
                .state()
                .zones
                .get(to)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
                .into_iter()
                .filter(|existing| {
                    tx.state()
                        .cards
                        .get(*existing)
                        .and_then(|card| card.kind.equip_slot())
                        == Some(slot)
                })
                .collect();
            let mut effects = Vec::new();
            if !replaced.is_empty() {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("missing discard pile".into()))?;
                effects.push(EffectRequest::MoveCards {
                    cards: replaced,
                    to: discard,
                    from: Some(to),
                });
            }
            effects.push(EffectRequest::MoveCards {
                cards: vec![card],
                to,
                from: Some(from),
            });
            effects.push(EffectRequest::DrawCardsFromSkill {
                player: source,
                count: 1,
                skill,
            });
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown move-equipment continuation stage".into(),
        )),
    }
}
