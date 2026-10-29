//! Command operations for `resolve_new_yajiao` and related helpers.

use super::*;

pub(crate) fn resolve_choose_gain_card_from_each_other_then_turn_over(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 {
        return Err(EffectError::Illegal(
            "bad gain-from-each continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let option_count = payload[3] as usize;
    let [card_raw] = selections else {
        return Err(EffectError::Illegal(
            "gain-from-each requires exactly one card".into(),
        ));
    };
    let card = CardId::new(*card_raw);
    let card_state = tx.state().cards.get(card);
    let gainable = crate::query::gainable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquipJudge,
    );
    if cancelled
        || observer != player
        || payload.len() != 4 + option_count
        || !payload[4..].contains(card_raw)
        || !gainable.contains(&card)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "gain-from-each card is no longer legal".into(),
        ));
    }
    let from = card_state.unwrap().zone;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("Guixin owner hand zone missing".into()))?;
    run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![card],
                to: hand,
                from: Some(from),
            },
            EffectRequest::RevealCards {
                cards: vec![card],
                to: Vec::new(),
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_hidden_skill_pile_hand_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, pile_def, _action, _card, _skill] = payload else {
        return Err(EffectError::Illegal(
            "bad hidden skill-pile hand payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal(
            "hidden skill-pile hand observer mismatch".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled hidden skill-pile hand choice must be empty".into(),
            ))
        };
    }
    let current_hand: std::collections::BTreeSet<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    let hidden = tx.state().hidden_skill_pile_cards(player, *pile_def);
    if selections.is_empty()
        || selections.len() > hidden.len()
        || selected.len() != selections.len()
        || !selected.is_subset(&current_hand)
    {
        return Err(EffectError::Illegal(
            "hidden skill-pile hand cards are no longer legal".into(),
        ));
    }
    let mut next = payload.to_vec();
    next.push(selections.len() as u32);
    next.extend_from_slice(selections);
    crate::effect::open_choice(
        tx,
        player,
        selections.len() as u8,
        selections.len() as u8,
        false,
        hidden.into_iter().map(CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::ChooseHiddenSkillPileCards,
        next,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_hidden_skill_pile_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal(
            "bad hidden skill-pile card payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let action = payload[2] as u8;
    let (count, hand_start) = if action == 2 {
        (payload.get(5).copied().unwrap_or(0) as usize, 6)
    } else {
        (1, payload.len())
    };
    if observer != player
        || cancelled
        || selections.len() != count
        || (action == 2 && payload.len() != hand_start + count)
    {
        return Err(EffectError::Illegal(
            "hidden skill-pile card count mismatch".into(),
        ));
    }
    let current: std::collections::BTreeSet<_> = tx
        .state()
        .hidden_skill_pile_cards(player, payload[1])
        .into_iter()
        .map(CardId::raw)
        .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if selected.len() != selections.len() || !selected.is_subset(&current) {
        return Err(EffectError::Illegal(
            "hidden skill-pile cards are no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveHiddenSkillPile {
            player,
            pile_skill_def: engine_api::SkillDefId::new(payload[1]),
            action,
            card: CardId::new(payload[3]),
            stage: 1,
            hand_cards: if action == 2 {
                payload[hand_start..]
                    .iter()
                    .copied()
                    .map(CardId::new)
                    .collect()
            } else {
                Vec::new()
            },
            pile_cards: selections.iter().copied().map(CardId::new).collect(),
            event_cards: Vec::new(),
            skill: engine_api::SkillId::new(payload[4]),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_mark_parity_control_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mark_raw, mass_threshold, damage_amount, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad mark parity target continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal(
            "mark parity target observer mismatch".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled mark parity target must be empty".into(),
            ))
        };
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "mark parity control requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "mark parity control target is no longer alive".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveMarkParityControl {
            player,
            mark: engine_api::MarkDefId::new(*mark_raw),
            mass_threshold: *mass_threshold,
            damage_amount: *damage_amount,
            stage: 1,
            index: 0,
            target,
            targets: Vec::new(),
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_discard_mark_parity_control_target_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, mark_raw, mass_threshold, damage_amount, skill_raw] = payload
    else {
        return Err(EffectError::Illegal(
            "bad mark parity discard continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let target = PlayerId::new(*target_raw);
    let [card_raw] = selections else {
        return Err(EffectError::Illegal(
            "mark parity discard requires one card".into(),
        ));
    };
    let card = CardId::new(*card_raw);
    if cancelled
        || observer != player
        || !crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquipJudge,
        )
        .contains(&card)
    {
        return Err(EffectError::Illegal(
            "mark parity discard card is no longer legal".into(),
        ));
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveMarkParityControl {
            player,
            mark: engine_api::MarkDefId::new(*mark_raw),
            mass_threshold: *mass_threshold,
            damage_amount: *damage_amount,
            stage: 3,
            index: 0,
            target,
            targets: Vec::new(),
            skill: engine_api::SkillId::new(*skill_raw),
        });
    run_effects(
        tx,
        vec![EffectRequest::DiscardTargetCards {
            actor: player,
            target,
            cards: vec![card],
            scope: rule_api::TargetCardScope::HandEquipJudge,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_confirm_mark_parity_mass_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, mark_raw, mass_threshold, damage_amount, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad mark parity mass continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    let [choice] = selections else {
        return Err(EffectError::Illegal(
            "mark parity mass confirmation requires one option".into(),
        ));
    };
    if cancelled || observer != player || *choice > 1 {
        return Err(EffectError::Illegal(
            "invalid mark parity mass confirmation".into(),
        ));
    }
    if *choice == 0 {
        return Ok(());
    }
    let mark = engine_api::MarkDefId::new(*mark_raw);
    let marks = tx
        .state()
        .players
        .get(player)
        .and_then(|owner| {
            owner
                .marks
                .iter()
                .find(|(definition, _)| *definition == mark.raw())
                .map(|(_, count)| (*count).max(0) as u32)
        })
        .unwrap_or(0);
    if marks <= *mass_threshold {
        return Err(EffectError::Illegal(
            "mark parity mass threshold is no longer met".into(),
        ));
    }
    let targets = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| *target != player)
        .collect();
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveMarkParityControl {
            player,
            mark,
            mass_threshold: *mass_threshold,
            damage_amount: *damage_amount,
            stage: 4,
            index: 0,
            target: PlayerId::NONE,
            targets,
            skill: engine_api::SkillId::new(*skill_raw),
        });
    run_effects(
        tx,
        vec![EffectRequest::RemoveMark {
            player,
            mark,
            amount: 0,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_selected_target_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal(
            "bad selected-target damage continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let damage_amount = payload[1];
    let nature = payload[2] as u8;
    let target_count = payload[4] as usize;
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "selected-target damage requires one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if cancelled
        || observer != player
        || payload.len() != 5 + target_count
        || !payload[5..].contains(target_raw)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "selected-target damage choice is no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target,
            amount: damage_amount,
            is_lose_hp: false,
            nature,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_distinct_suit_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 {
        return Err(EffectError::Illegal(
            "bad distinct-suit continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let count = payload[2] as usize;
    if cancelled || observer != player || payload.len() != 3 + count {
        return Err(EffectError::Illegal(
            "invalid distinct-suit selection context".into(),
        ));
    }
    let original: std::collections::BTreeSet<_> = payload[3..].iter().copied().collect();
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    let all_suits: std::collections::BTreeSet<_> = payload[3..]
        .iter()
        .filter_map(|card| tx.state().cards.get(CardId::new(*card)))
        .filter(|card| card.zone == processing)
        .map(|card| {
            crate::command::judge::suit_tag(crate::query::effective_card_suit(
                tx.state(),
                player,
                card,
            ))
        })
        .collect();
    let mut selected_cards = std::collections::BTreeSet::new();
    let selected_suits: std::collections::BTreeSet<_> = selections
        .iter()
        .filter_map(|card| {
            let card = tx.state().cards.get(CardId::new(*card))?;
            (original.contains(&card.id.raw())
                && card.zone == processing
                && selected_cards.insert(card.id.raw()))
            .then(|| {
                crate::command::judge::suit_tag(crate::query::effective_card_suit(
                    tx.state(),
                    player,
                    card,
                ))
            })
        })
        .collect();
    if selected_cards.len() != selections.len()
        || selections.len() != all_suits.len()
        || selected_suits != all_suits
    {
        return Err(EffectError::Illegal(
            "selection must contain exactly one revealed card per distinct suit".into(),
        ));
    }
    let selected: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let rest: Vec<_> = payload[3..]
        .iter()
        .copied()
        .filter(|card| !selected_cards.contains(card))
        .map(CardId::new)
        .collect();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let mut effects = vec![EffectRequest::MoveCards {
        cards: selected,
        to: hand,
        from: Some(processing),
    }];
    if !rest.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards: rest,
            to: discard,
            from: Some(processing),
        });
    }
    run_effects(tx, effects).map(|_| ())
}

/// Resolve the 「试探」 (scskuiji) cross-hand distinct-suit discard. The chooser may pick up to
/// `count` cards from its own and `target`'s hands combined; the selection must have distinct
/// effective suits, and all selected cards are then discarded (split by owner).
pub(crate) fn resolve_choose_scskuiji_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal(
            "bad scskuiji continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let count = payload[2] as usize;
    let offered_len = payload[4] as usize;
    if observer != player || payload.len() != 5 + offered_len || selections.len() > count {
        return Err(EffectError::Illegal(
            "invalid scskuiji selection context".into(),
        ));
    }
    let offered: std::collections::BTreeSet<_> = payload[5..].iter().copied().collect();
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    // Each selected card must be one of the offered cards, still in either participant's hand, and
    // no two may share an effective suit.
    let mut seen_cards = std::collections::BTreeSet::new();
    let mut seen_suits = std::collections::BTreeSet::new();
    let hand_zones = [
        tx.state().zones.find(ZoneKind::Hand, player),
        tx.state().zones.find(ZoneKind::Hand, target),
    ];
    for raw in selections {
        let card = tx
            .state()
            .cards
            .get(CardId::new(*raw))
            .ok_or_else(|| EffectError::Illegal("scskuiji selected card missing".into()))?;
        if !offered.contains(&card.id.raw())
            || !hand_zones.iter().flatten().any(|zone| *zone == card.zone)
            || !seen_cards.insert(card.id.raw())
        {
            return Err(EffectError::Illegal(
                "scskuiji selection must reference distinct offered hand cards".into(),
            ));
        }
        let suit = crate::command::judge::suit_tag(crate::query::effective_card_suit(
            tx.state(),
            player,
            card,
        ));
        if !seen_suits.insert(suit) {
            return Err(EffectError::Illegal(
                "scskuiji selection must use distinct effective suits".into(),
            ));
        }
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let effects = cards
        .iter()
        .map(|card| {
            let owner = tx
                .state()
                .cards
                .get(*card)
                .map(|card| card.owner)
                .unwrap_or(PlayerId::NONE);
            EffectRequest::MoveCards {
                cards: vec![*card],
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, owner),
            }
        })
        .collect::<Vec<_>>();
    run_effects(tx, effects).map(|_| ())
}

/// Resolve the 「破试探」 (pskuiji) cross-hand distinct-suit discard with an asymmetric penalty:
/// identical validation/discard to scskuiji, and if the owner and target discarded different
/// numbers of cards, the side that discarded fewer loses 1 HP. (The legacy `rechouai` grant is a
/// dangling no-op in upstream — the skill is never defined — and is intentionally not modeled.)
pub(crate) fn resolve_choose_pskuiji_discard(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::Illegal(
            "bad pskuiji continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let count = payload[2] as usize;
    let offered_len = payload[4] as usize;
    if observer != player || payload.len() != 5 + offered_len || selections.len() > count {
        return Err(EffectError::Illegal(
            "invalid pskuiji selection context".into(),
        ));
    }
    let offered: std::collections::BTreeSet<_> = payload[5..].iter().copied().collect();
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let mut seen_cards = std::collections::BTreeSet::new();
    let mut seen_suits = std::collections::BTreeSet::new();
    let hand_zones = [
        tx.state().zones.find(ZoneKind::Hand, player),
        tx.state().zones.find(ZoneKind::Hand, target),
    ];
    let mut owner_counts: [(PlayerId, u32); 2] = [(player, 0), (target, 0)];
    for raw in selections {
        let card = tx
            .state()
            .cards
            .get(CardId::new(*raw))
            .ok_or_else(|| EffectError::Illegal("pskuiji selected card missing".into()))?;
        if !offered.contains(&card.id.raw())
            || !hand_zones.iter().flatten().any(|zone| *zone == card.zone)
            || !seen_cards.insert(card.id.raw())
        {
            return Err(EffectError::Illegal(
                "pskuiji selection must reference distinct offered hand cards".into(),
            ));
        }
        let suit = crate::command::judge::suit_tag(crate::query::effective_card_suit(
            tx.state(),
            player,
            card,
        ));
        if !seen_suits.insert(suit) {
            return Err(EffectError::Illegal(
                "pskuiji selection must use distinct effective suits".into(),
            ));
        }
        for (owner, cnt) in owner_counts.iter_mut() {
            if *owner == card.owner {
                *cnt += 1;
            }
        }
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let mut effects: Vec<EffectRequest> = selections
        .iter()
        .copied()
        .map(CardId::new)
        .map(|card| {
            let owner = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.owner)
                .unwrap_or(PlayerId::NONE);
            EffectRequest::MoveCards {
                cards: vec![card],
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, owner),
            }
        })
        .collect();
    // Asymmetric penalty: the side that discarded fewer loses 1 HP.
    let [(_owner_a, cnt_a), (_owner_b, cnt_b)] = owner_counts;
    if cnt_a != cnt_b {
        let under_discarder = if cnt_a < cnt_b { player } else { target };
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: under_discarder,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_inspected_heart_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player_raw, target_raw, skill_raw] = payload else {
        return Err(EffectError::Illegal(
            "bad inspected-heart continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player_raw);
    if observer != player {
        return Err(EffectError::Illegal(
            "inspected-heart observer mismatch".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled inspected-heart selection must be empty".into(),
            ))
        };
    }
    let [card_raw] = selections else {
        return Err(EffectError::Illegal(
            "inspected-heart selection requires one card".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    let card = CardId::new(*card_raw);
    let hand = tx.state().zones.find(ZoneKind::Hand, target);
    if !tx.state().cards.get(card).is_some_and(|state| {
        hand == Some(state.zone)
            && crate::query::effective_card_suit(tx.state(), target, state)
                == engine_api::Suit::Heart
    }) {
        return Err(EffectError::Illegal(
            "selected inspected card is no longer a heart hand card".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::InspectTargetHandChooseHeartDiscardOrTop {
            player,
            target,
            stage: 1,
            card,
            skill: engine_api::SkillId::new(*skill_raw),
        }],
    )
    .map(|_| ())
}
