//! Command operations for `resolve_resolve_lulve` and related helpers.

use super::*;

pub(crate) fn resolve_resolve_lulve(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 && payload.len() != 4 {
        return Err(EffectError::EngineFault("bad Lulve continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if cancelled
        || selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Lulve choice context changed".into()));
    }
    if payload.len() == 3 {
        // Stage 0: owner picked the target.
        if observer != player {
            return Err(EffectError::Illegal(
                "Lulve target choice observer mismatch".into(),
            ));
        }
        let target = PlayerId::new(selections[0]);
        if target == player
            || tx.state().hand_count(target) == 0
            || tx.state().hand_count(target) >= tx.state().hand_count(player)
        {
            return Err(EffectError::Illegal(
                "Lulve target is invalid or stale".into(),
            ));
        }
        return run_effects(
            tx,
            vec![EffectRequest::ResolveLulve {
                player,
                target,
                branch: 2,
                skill,
            }],
        )
        .map(|_| ());
    }
    // Stage 1: the target picked the branch.
    if observer != PlayerId::new(payload[3]) {
        return Err(EffectError::Illegal(
            "Lulve branch choice observer mismatch".into(),
        ));
    }
    let branch = (*selections.first().unwrap()).min(1) as u8;
    run_effects(
        tx,
        vec![EffectRequest::ResolveLulve {
            player,
            target: PlayerId::new(payload[3]),
            branch,
            skill,
        }],
    )
    .map(|_| ())
}

/// 临界 (sp/ollinjie) continuation: the damage source discards the selected hand card; if
/// the discarded card is effectively a Sha, the skill owner draws one. The discard and the
/// conditional draw are scheduled in one queue so interrupts serialize correctly.
pub(crate) fn resolve_ollinjie(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, source, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Ollinjie discard continuation".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let source = PlayerId::new(*source);
    let skill = engine_api::SkillId::new(*skill);
    if observer != source {
        return Err(EffectError::Illegal(
            "Ollinjie discard chooser mismatch".into(),
        ));
    }
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Ollinjie source must discard exactly one hand card".into(),
        ));
    }
    let cost = CardId::new(selections[0]);
    if !tx.state().hand_cards(source).contains(&cost) {
        return Err(EffectError::Illegal(
            "Ollinjie discard card is no longer in the source's hand".into(),
        ));
    }
    let discarded_sha =
        tx.state().cards.get(cost).is_some_and(|card| {
            crate::query::effective_card_kind(tx.state(), source, card).is_sha()
        });
    let mut effects = vec![EffectRequest::DiscardCards {
        player: source,
        cards: vec![cost],
    }];
    if discarded_sha {
        effects.push(EffectRequest::DrawCards { player, count: 1 });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_target_gain_max_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, _skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Shouchun gain-max-hp continuation".into(),
        ));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal(
            "Shouchun gain-max-hp chooser mismatch".into(),
        ));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled Shouchun target choice has selections".into(),
            ));
        }
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Shouchun must select exactly one target".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Shouchun target is no longer alive".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ChangeMaxHp {
            player: target,
            delta: 1,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_target_hand_card_to_pile(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    match stage {
        0 => {
            let [_, player, skill, pile_def] = payload else {
                return Err(EffectError::EngineFault(
                    "bad choose-target-hand-card-to-pile continuation".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let skill = engine_api::SkillId::new(*skill);
            let pile_def = *pile_def;
            if observer != player {
                return Err(EffectError::Illegal("chooser mismatch".into()));
            }
            if cancelled {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "must select exactly one target".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            let target_alive = tx.state().players.get(target).is_some_and(|p| p.alive);
            if target == player || !target_alive {
                return Err(EffectError::Illegal(
                    "target must be another living player".into(),
                ));
            }
            let options = tx.state().hand_cards(target);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "target no longer has hand cards".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.iter().map(|c| c.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseTargetHandCardToPile,
                vec![1, player.raw(), skill.raw(), pile_def, target.raw()],
            )?;
            Ok(())
        }
        1 => {
            let [_, player, skill, pile_def, target] = payload else {
                return Err(EffectError::EngineFault(
                    "bad choose-target-hand-card-to-pile continuation".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let skill = engine_api::SkillId::new(*skill);
            let pile_def = *pile_def;
            let target = PlayerId::new(*target);
            if observer != player {
                return Err(EffectError::Illegal("chooser mismatch".into()));
            }
            if cancelled {
                return Err(EffectError::Illegal(
                    "cancelled choose-target-hand-card-to-pile".into(),
                ));
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("must select exactly one card".into()));
            }
            let card = CardId::new(selections[0]);
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
            if !tx
                .state()
                .zones
                .get(target_hand)
                .unwrap()
                .cards
                .contains(&card)
            {
                return Err(EffectError::Illegal(
                    "selected card not in target hand".into(),
                ));
            }
            // ensure (player, pile_def) pile exists
            let existing = tx
                .state()
                .skill_piles
                .iter()
                .find(|p| p.owner == player && p.skill_def == pile_def)
                .map(|p| p.zone);
            let pile = match existing {
                Some(z) => z,
                None => crate::effect::ensure_skill_pile_zone(tx, player, pile_def)?,
            };
            tx.state_mut()
                .zones
                .get_mut(target_hand)
                .unwrap()
                .cards
                .retain(|c| *c != card);
            tx.state_mut().zones.get_mut(pile).unwrap().cards.push(card);
            if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
                card_state.zone = pile;
                card_state.owner = player;
            }
            Ok(())
        }
        _ => Err(EffectError::EngineFault(
            "bad choose-target-hand-card-to-pile stage".into(),
        )),
    }
}

pub(crate) fn resolve_gain_target_hand_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    match stage {
        0 => {
            let [_, player, skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Sizhou target continuation".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let skill = engine_api::SkillId::new(*skill);
            if observer != player {
                return Err(EffectError::Illegal("Sizhou chooser mismatch".into()));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled Sizhou target choice has selections".into(),
                    ));
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Sizhou must select exactly one target".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            let target_alive = tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive);
            if target == player || !target_alive {
                return Err(EffectError::Illegal(
                    "Sizhou target must be another living player".into(),
                ));
            }
            let options = tx.state().hand_cards(target);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "Sizhou target no longer has hand cards".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::GainTargetHandCard,
                vec![1, player.raw(), target.raw(), skill.raw()],
            )
            .map(|_| ())
        }
        1 => {
            let [_, player, target, _skill] = payload else {
                return Err(EffectError::EngineFault(
                    "bad Sizhou card continuation".into(),
                ));
            };
            let player = PlayerId::new(*player);
            let target = PlayerId::new(*target);
            if observer != player {
                return Err(EffectError::Illegal("Sizhou card chooser mismatch".into()));
            }
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Sizhou must take exactly one hand card".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal(
                    "Sizhou card is no longer in the target's hand".into(),
                ));
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("Sizhou target hand missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Sizhou player hand missing".into()))?;
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
            "bad Sizhou continuation stage".into(),
        )),
    }
}

/// 穆荫 continuation: apply the persistent +1 hand-limit bonus to the chosen
/// same-clan-or-self, not-field-max target.
pub(crate) fn resolve_choose_clan_not_max_hand_limit_gain(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, _skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Muyin hand-limit continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("Muyin chooser mismatch".into()));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled Muyin choice has selections".into(),
            ));
        }
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Muyin must select exactly one target".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
        return Err(EffectError::Illegal("Muyin target not alive".into()));
    }
    let bonus = tx
        .state()
        .hand_limit_bonus
        .get(&target.raw())
        .copied()
        .unwrap_or(0);
    tx.state_mut()
        .hand_limit_bonus
        .insert(target.raw(), bonus + 1);
    Ok(())
}

/// 置入 continuation: commit the chosen hand card onto the skill pile via the
/// existing CommitStoreHandShaInSkillPile insertion effect.
pub(crate) fn resolve_place_hand_card_to_expansion(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad expansion-place continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player {
        return Err(EffectError::Illegal("expansion placer mismatch".into()));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled expansion place has selections".into(),
            ));
        }
        return Ok(());
    }
    let [card] = selections else {
        return Err(EffectError::Illegal(
            "expansion place must select exactly one card".into(),
        ));
    };
    let card = CardId::new(*card);
    if !tx.state().hand_cards(player).contains(&card) {
        return Err(EffectError::Illegal(
            "expansion place card not in hand".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::CommitStoreHandShaInSkillPile {
            player,
            cards: vec![card],
            skill,
        }],
    )?;
    Ok(())
}

/// 选他人授标记 continuation (ABI 652): apply the mark delta to the chosen target.
pub(crate) fn resolve_choose_other_add_mark(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, mark, delta, _skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad choose-other-add-mark payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let delta = *delta as i32;
    if observer != player {
        return Err(EffectError::Illegal("add-mark chooser mismatch".into()));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled add-mark has selections".into(),
            ));
        }
        return Ok(());
    }
    let [target] = selections else {
        return Err(EffectError::Illegal(
            "choose exactly one add-mark target".into(),
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
        return Err(EffectError::Illegal("bad add-mark target".into()));
    }
    // Negative deltas remove presence-based marks (jsrgpingrong 猎): clamp at
    // the target's current count so the counter never goes below zero.
    let delta = if delta < 0 {
        let current = tx
            .state()
            .players
            .get(target)
            .and_then(|p| p.marks.iter().find(|(m, _)| *m == *mark))
            .map_or(0, |(_, c)| *c);
        (-current).max(delta)
    } else {
        delta
    };
    run_effects(
        tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(*mark),
            delta,
        }],
    )
    .map(|_| ())
}

/// 取用 continuation: move the selected card from the owner's skill pile to hand.
pub(crate) fn resolve_gain_from_skill_pile(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad gain-from-pile continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player {
        return Err(EffectError::Illegal(
            "gain-from-pile chooser mismatch".into(),
        ));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled gain-from-pile has selections".into(),
            ));
        }
        return Ok(());
    }
    let [card] = selections else {
        return Err(EffectError::Illegal(
            "gain-from-pile must select exactly one card".into(),
        ));
    };
    let card = CardId::new(*card);
    let skill_def = crate::effect::skill_pile_def(tx, player, skill)?;
    let pile_cards = tx.state().skill_pile_cards(player, skill_def);
    if !pile_cards.contains(&card) {
        return Err(EffectError::Illegal(
            "gain-from-pile card not in the skill pile".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile_zone = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == skill_def)
        .map(|pile| pile.zone)
        .ok_or_else(|| EffectError::EngineFault("missing skill-pile zone".into()))?;
    crate::effect::move_cards(tx, vec![card], hand, Some(pile_zone))?;
    Ok(())
}

/// start_debate_targets continuation (ABI 656): forward to the pack_jsrg resolver.
pub(crate) fn resolve_start_debate_targets(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    crate::effect::resolve_start_debate_targets(tx, observer, payload, selections, cancelled)
}

/// discard_basic_draw_equal continuation (ABI 657): forward to pack_jsrg.
pub(crate) fn resolve_discard_basic_draw_equal(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    crate::effect::resolve_discard_basic_draw_equal(tx, observer, payload, selections, cancelled)
}
