//! Command operations for `resolve_draw_give_or_discard_hand_discard` and related helpers.

use super::*;

pub(crate) fn resolve_draw_give_or_discard_hand_discard(
    tx: &mut Transaction,
    player: PlayerId,
    payment_count: usize,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let options = draw_give_or_discard_hand_cards(tx, player, true);
    let required = options.len().min(payment_count);
    if required == 0 {
        return Ok(());
    }
    if required == options.len() {
        return run_effects(
            tx,
            vec![EffectRequest::DiscardCards {
                player,
                cards: options,
            }],
        )
        .map(|_| ());
    }
    crate::effect::open_choice(
        tx,
        player,
        required as u8,
        required as u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::DrawThenChooseGiveOrDiscardHand,
        vec![3, player.raw(), required as u32, skill.raw()],
    )
    .map(|_| ())
}

pub(crate) fn resolve_draw_then_choose_give_or_discard_hand(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill_raw = match stage {
        0 | 1 | 3 => *payload.get(3).unwrap_or(&0),
        2 => *payload.get(4).unwrap_or(&0),
        _ => 0,
    };
    let skill = engine_api::SkillId::new(skill_raw);
    let recipient_scope = match stage {
        0 | 1 => payload.get(4),
        2 => payload.get(5),
        _ => None,
    }
    .copied()
    .map(|tag| {
        u8::try_from(tag)
            .ok()
            .and_then(DrawTargetScope::from_tag)
            .ok_or_else(|| EffectError::EngineFault("bad give recipient scope".into()))
    })
    .transpose()?
    .unwrap_or(DrawTargetScope::Other);
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if observer != player
        || cancelled
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !skill_enabled
    {
        return Err(EffectError::Illegal(
            "draw/give-or-discard hand choice context changed".into(),
        ));
    }

    match (stage, payload, selections) {
        (0, [_, _, payment_count, _], [1]) | (0, [_, _, payment_count, _, _], [1]) => {
            if *payment_count == 0 || *payment_count > u8::MAX as u32 {
                return Err(EffectError::EngineFault(
                    "bad draw/give hand payment count".into(),
                ));
            }
            if draw_give_or_discard_hand_cards(tx, player, false).len() < *payment_count as usize {
                return Err(EffectError::Illegal("not enough hand cards to give".into()));
            }
            let mut targets: Vec<_> = tx
                .state()
                .players
                .iter_live()
                .filter(|target| {
                    crate::effect::player_target_matches(
                        tx.state(),
                        player,
                        target.id,
                        recipient_scope,
                    )
                })
                .map(|target| target.id.raw())
                .collect();
            targets.sort_unstable();
            targets.dedup();
            if targets.is_empty() {
                return Err(EffectError::Illegal(
                    "give branch has no living recipient".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets,
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::DrawThenChooseGiveOrDiscardHand,
                vec![
                    1,
                    player.raw(),
                    *payment_count,
                    skill.raw(),
                    recipient_scope.tag() as u32,
                ],
            )
            .map(|_| ())
        }
        (0, [_, _, payment_count, _], [2]) | (0, [_, _, payment_count, _, _], [2]) => {
            if *payment_count == 0 || *payment_count > u8::MAX as u32 {
                return Err(EffectError::EngineFault(
                    "bad draw/discard hand payment count".into(),
                ));
            }
            resolve_draw_give_or_discard_hand_discard(tx, player, *payment_count as usize, skill)
        }
        (1, [_, _, payment_count, _], [target]) | (1, [_, _, payment_count, _, _], [target]) => {
            if *payment_count == 0 || *payment_count > u8::MAX as u32 {
                return Err(EffectError::EngineFault(
                    "bad draw/give hand payment count".into(),
                ));
            }
            let target = PlayerId::new(*target);
            if !crate::effect::player_target_matches(tx.state(), player, target, recipient_scope) {
                return Err(EffectError::Illegal(
                    "draw/give hand recipient is no longer legal".into(),
                ));
            }
            let cards = draw_give_or_discard_hand_cards(tx, player, false);
            if cards.len() < *payment_count as usize {
                return Err(EffectError::Illegal("not enough hand cards to give".into()));
            }
            if cards.len() == *payment_count as usize {
                let from = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::EngineFault("giver hand zone missing".into()))?;
                let to = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| {
                        EffectError::EngineFault("recipient hand zone missing".into())
                    })?;
                return run_effects(tx, vec![EffectRequest::GiveCards { cards, to, from }])
                    .map(|_| ());
            }
            crate::effect::open_choice(
                tx,
                player,
                *payment_count as u8,
                *payment_count as u8,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::DrawThenChooseGiveOrDiscardHand,
                vec![
                    2,
                    player.raw(),
                    target.raw(),
                    *payment_count,
                    skill.raw(),
                    recipient_scope.tag() as u32,
                ],
            )
            .map(|_| ())
        }
        (2, [_, _, target, payment_count, _], selected)
        | (2, [_, _, target, payment_count, _, _], selected)
            if selected.len() == *payment_count as usize =>
        {
            let target = PlayerId::new(*target);
            if *payment_count == 0
                || *payment_count > u8::MAX as u32
                || !crate::effect::player_target_matches(
                    tx.state(),
                    player,
                    target,
                    recipient_scope,
                )
            {
                return Err(EffectError::Illegal(
                    "draw/give hand recipient context changed".into(),
                ));
            }
            let current: std::collections::BTreeSet<_> =
                draw_give_or_discard_hand_cards(tx, player, false)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            let mut distinct = std::collections::BTreeSet::new();
            if !selected
                .iter()
                .all(|card| distinct.insert(*card) && current.contains(card))
            {
                return Err(EffectError::Illegal(
                    "draw/give hand cards are duplicated or stale".into(),
                ));
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("giver hand zone missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("recipient hand zone missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: selected.iter().copied().map(CardId::new).collect(),
                    to,
                    from,
                }],
            )
            .map(|_| ())
        }
        (3, [_, _, required, _], selected) if selected.len() == *required as usize => {
            if *required == 0 || *required > u8::MAX as u32 {
                return Err(EffectError::EngineFault(
                    "bad draw/discard hand payment count".into(),
                ));
            }
            let current: std::collections::BTreeSet<_> =
                draw_give_or_discard_hand_cards(tx, player, true)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            let mut distinct = std::collections::BTreeSet::new();
            if !selected
                .iter()
                .all(|card| distinct.insert(*card) && current.contains(card))
            {
                return Err(EffectError::Illegal(
                    "draw/discard hand cards are duplicated or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: selected.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "invalid draw/give-or-discard hand selection".into(),
        )),
    }
}

pub(crate) fn shenduan_card_is_eligible(tx: &Transaction, player: PlayerId, card: CardId) -> bool {
    let Some(discard) = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE) else {
        return false;
    };
    tx.state().cards.get(card).is_some_and(|card_state| {
        card_state.zone == discard
            && card_state.owner.is_none()
            && crate::query::effective_card_kind(tx.state(), player, card_state).is_basic()
            && crate::query::effective_card_color_mask(tx.state(), player, card) == 0b10
    })
}

pub(crate) fn resolve_choose_discarded_black_basic_as_bingliang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "discard-to-Bingliang choice must be resolved by the skill owner".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal(
            "discard-to-Bingliang skill context changed".into(),
        ));
    }
    if cancelled {
        return if stage == 0 {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "Bingliang target choice cannot cancel".into(),
            ))
        };
    }

    match stage {
        0 => {
            let len = *payload.get(3).unwrap_or(&0) as usize;
            let offered = payload
                .get(4..4 + len)
                .ok_or_else(|| EffectError::EngineFault("truncated Shenduan cards".into()))?;
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "choose exactly one discarded black basic card".into(),
                ));
            };
            if !offered.contains(selected) {
                return Err(EffectError::Illegal(
                    "discarded card was not offered".into(),
                ));
            }
            let card = CardId::new(*selected);
            if !shenduan_card_is_eligible(tx, player, card) {
                return Err(EffectError::Illegal(
                    "discarded card is no longer an effective black basic card".into(),
                ));
            }
            let targets = crate::command::play::view_as_bingliang_targets(tx, player, card);
            if targets.is_empty() {
                return Err(EffectError::Illegal(
                    "discarded card has no legal Bingliang target".into(),
                ));
            }
            let remaining: Vec<_> = offered
                .iter()
                .copied()
                .filter(|candidate| *candidate != card.raw())
                .collect();
            let mut next = vec![
                1,
                player.raw(),
                skill.raw(),
                card.raw(),
                remaining.len() as u32,
            ];
            next.extend(remaining);
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::ChooseDiscardedBlackBasicAsBingliang,
                next,
            )
            .map(|_| ())
        }
        1 => {
            let card = CardId::new(*payload.get(3).unwrap_or(&0));
            let len = *payload.get(4).unwrap_or(&0) as usize;
            let remaining = payload
                .get(5..5 + len)
                .ok_or_else(|| EffectError::EngineFault("truncated Shenduan remainder".into()))?;
            let [target] = selections else {
                return Err(EffectError::Illegal(
                    "choose exactly one Bingliang target".into(),
                ));
            };
            let target = PlayerId::new(*target);
            if !shenduan_card_is_eligible(tx, player, card)
                || !crate::command::play::view_as_bingliang_targets(tx, player, card)
                    .contains(&target)
            {
                return Err(EffectError::Illegal(
                    "Bingliang card or target is no longer legal".into(),
                ));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("missing discard pile".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("missing skill owner hand".into()))?;
            crate::effect::move_one_card(tx, card, discard, hand, player, false)?;
            tx.state_mut().resolution.deferred_effects.push(
                EffectRequest::ChooseDiscardedBlackBasicAsBingliang {
                    player,
                    cards: remaining.iter().copied().map(CardId::new).collect(),
                    skill,
                },
            );
            match crate::command::play::play_as_kind(
                tx,
                player,
                card,
                CardKind::TrickBingliang,
                &[target],
                true,
                false,
                false,
                false,
            ) {
                Ok(()) => crate::effect::resume_deferred_resolution(tx),
                Err(error) => Err(error),
            }
        }
        _ => Err(EffectError::Illegal("unknown Shenduan choice stage".into())),
    }
}

pub(crate) fn resolve_choose_discard_judge_use_virtual_sha_draw_if_no_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "judgment card must be chosen by the skill owner".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let [selected] = selections else {
        return Err(EffectError::Illegal(
            "choose exactly one judgment card".into(),
        ));
    };
    let selected = CardId::new(*selected);
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || !crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::Judge,
    )
    .contains(&selected)
    {
        return Err(EffectError::Illegal(
            "judgment discard context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::DiscardTargetCards {
                actor: player,
                target,
                cards: vec![selected],
                scope: rule_api::TargetCardScope::Judge,
            },
            EffectRequest::UseVirtualShaDrawIfNoDamage {
                player,
                target,
                skill,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_store_hand_sha_in_skill_pile(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(1).unwrap_or(&0));
    if observer != player || cancelled || selections.is_empty() {
        return Err(EffectError::Illegal(
            "store at least one hand Sha as the skill owner".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::CommitStoreHandShaInSkillPile {
            player,
            cards: selections.iter().copied().map(CardId::new).collect(),
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_spend_skill_pile_for_dying_jiu(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let dying = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "spend exactly one skill-pile card as the skill owner".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::CommitSpendSkillPileForDyingJiu {
            player,
            dying,
            card: CardId::new(selections[0]),
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_discard_one_then_gain_target_equipment(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let slot = payload
        .get(2)
        .copied()
        .and_then(|tag| u8::try_from(tag).ok())
        .and_then(crate::state::EquipSlot::from_tag)
        .ok_or_else(|| EffectError::Illegal("bad equipment slot payload".into()))?;
    if observer != player {
        return Err(EffectError::Illegal("discard chooser mismatch".into()));
    }
    if cancelled {
        if !selections.is_empty() {
            return Err(EffectError::Illegal(
                "cancelled discard must not select a card".into(),
            ));
        }
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one hand or equipment card".into(),
        ));
    }
    let cost = CardId::new(selections[0]);
    if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&cost) {
        return Err(EffectError::Illegal(
            "discard card not in player hand or equipment".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player,
            cards: vec![cost],
        }],
    )?;

    let Some(from) = tx.state().zones.find(ZoneKind::Equip, target) else {
        return Ok(());
    };
    let cards: Vec<_> = tx
        .state()
        .zones
        .get(from)
        .map(|zone| {
            zone.cards
                .iter()
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .and_then(|card| card.kind.equip_slot())
                        == Some(slot)
                })
                .collect()
        })
        .unwrap_or_default();
    if cards.is_empty() {
        return Ok(());
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("discard chooser has no hand zone".into()))?;
    run_effects(tx, vec![EffectRequest::GiveCards { cards, to, from }]).map(|_| ())
}

pub(crate) fn resolve_choose_discard_one_for_sha_refund_and_draw_if_red(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let (player, sha_user, sha, skill, restore_skill_def) = match payload {
        [player, sha_user, sha, skill] => (
            PlayerId::new(*player),
            PlayerId::new(*sha_user),
            CardId::new(*sha),
            engine_api::SkillId::new(*skill),
            None,
        ),
        [player, sha_user, sha, skill, restore_skill_def] => (
            PlayerId::new(*player),
            PlayerId::new(*sha_user),
            CardId::new(*sha),
            engine_api::SkillId::new(*skill),
            Some(engine_api::SkillDefId::new(*restore_skill_def)),
        ),
        _ => {
            return Err(EffectError::EngineFault(
                "bad Sha-refund discard continuation".into(),
            ));
        }
    };
    if observer != player {
        return Err(EffectError::Illegal(
            "Sha-refund discard chooser mismatch".into(),
        ));
    }
    if cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "must pay the Sha-refund cost with exactly one hand or equipment card".into(),
        ));
    }
    let cost = CardId::new(selections[0]);
    if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&cost) {
        return Err(EffectError::Illegal(
            "Sha-refund cost card is no longer discardable".into(),
        ));
    }
    let restore_skill_def = if let Some(restore_skill_def) = restore_skill_def {
        crate::command::play::validate_saved_sha_use(tx, sha_user, sha)?;
        let effective_rank = |card| {
            crate::query::query(
                tx.state(),
                &rule_api::QueryRequest::CardAttribute { card, attr: 1 },
                &mut crate::query::QueryContext::new(),
            )
        };
        let cost_rank = effective_rank(cost);
        let sha_rank = effective_rank(sha);
        match (cost_rank, sha_rank) {
            (rule_api::QueryResponse::Int(cost_rank), rule_api::QueryResponse::Int(sha_rank)) => {
                (cost_rank == sha_rank).then_some(restore_skill_def)
            }
            _ => {
                return Err(EffectError::ContentFault(
                    "Sha-refund equal-rank comparison requires valid card ranks".into(),
                ));
            }
        }
    } else {
        None
    };
    // Keep payment and benefit in one scheduler queue. If discarding opens an interrupting
    // choice, the scheduler serializes the refund/draw as deferred work instead of losing it.
    let mut effects = vec![
        EffectRequest::DiscardCards {
            player,
            cards: vec![cost],
        },
        EffectRequest::RefundCurrentShaUseAndDrawIfRed {
            player,
            sha_user,
            card: sha,
            skill,
        },
    ];
    if let Some(skill_def) = restore_skill_def {
        effects.push(EffectRequest::AddSkill { player, skill_def });
    }
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_choose_repolu_targets_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Repolu target continuation".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player {
        return Err(EffectError::Illegal("Repolu chooser mismatch".into()));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Repolu choice must not select targets".into(),
            ))
        };
    }
    if selections.is_empty()
        || selections.len() > usize::from(u8::MAX)
        || selections
            .iter()
            .enumerate()
            .any(|(index, target)| selections[..index].contains(target))
    {
        return Err(EffectError::Illegal(
            "Repolu requires distinct living targets".into(),
        ));
    }
    let mark = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("Repolu skill context changed".into()))?;
    let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if selected.iter().any(|target| {
        !tx.state()
            .players
            .get(*target)
            .is_some_and(|participant| participant.alive)
    }) {
        return Err(EffectError::Illegal(
            "Repolu target is no longer alive".into(),
        ));
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| selected.contains(target))
        .collect();
    if targets.len() != selected.len() {
        return Err(EffectError::Illegal(
            "Repolu target seat order could not be resolved".into(),
        ));
    }
    let previous = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("Repolu owner is missing".into()))?
        .marks
        .iter()
        .find(|(existing, _)| *existing == mark)
        .map(|(_, count)| *count)
        .unwrap_or(0)
        .max(0) as u32;
    let draw_count = previous.saturating_add(1);
    let mut effects = vec![EffectRequest::AddMark {
        player,
        mark: engine_api::MarkDefId::new(mark),
        delta: 1,
    }];
    effects.extend(
        targets
            .into_iter()
            .map(|target| EffectRequest::DrawCardsFromSkill {
                player: target,
                count: draw_count,
                skill,
            }),
    );
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolve_zhuiyi(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, killer, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad Zhuiyi target continuation".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let killer = PlayerId::new(*killer);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player {
        return Err(EffectError::Illegal("Zhuiyi chooser mismatch".into()));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled Zhuiyi choice must not select a target".into(),
            ))
        };
    }
    let [target] = selections else {
        return Err(EffectError::Illegal(
            "Zhuiyi requires exactly one target".into(),
        ));
    };
    let target = PlayerId::new(*target);
    if target == player
        || target == killer
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Zhuiyi target is no longer legal".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal("Zhuiyi skill context changed".into()));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::RecoverHp {
                source: PlayerId::NONE,
                player: target,
                amount: 1,
            },
            EffectRequest::DrawCardsFromSkill {
                player: target,
                count: 3,
                skill,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_xinfu_guolun(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Guolun choice stage".into()))?;
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal("Guolun chooser mismatch".into()));
    }
    match stage {
        0 => {
            let [card] = selections else {
                return Err(EffectError::Illegal(
                    "Guolun requires one target hand card".into(),
                ));
            };
            if cancelled {
                return Err(EffectError::Illegal(
                    "Guolun target-hand choice cannot be cancelled".into(),
                ));
            }
            let target_card = CardId::new(*card);
            if !tx.state().hand_cards(target).contains(&target_card) {
                return Err(EffectError::Illegal("Guolun target hand card moved".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::RevealCards {
                    cards: vec![target_card],
                    to: Vec::new(),
                }],
            )?;
            let mut options = tx.state().hand_cards(player);
            if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
                options.extend(
                    tx.state()
                        .zones
                        .get(equip)
                        .map(|zone| zone.cards.clone())
                        .unwrap_or_default(),
                );
            }
            if options.is_empty() {
                return Ok(());
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ResolveXinfuGuolun,
                vec![
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    target_card.raw(),
                ],
            )
            .map(|_| ())
        }
        1 => {
            if cancelled {
                return if selections.is_empty() {
                    Ok(())
                } else {
                    Err(EffectError::Illegal(
                        "cancelled Guolun choice must not select a card".into(),
                    ))
                };
            }
            let [player_card] = selections else {
                return Err(EffectError::Illegal(
                    "Guolun exchange requires one owner card".into(),
                ));
            };
            let player_card = CardId::new(*player_card);
            let target_card =
                CardId::new(*payload.get(4).ok_or_else(|| {
                    EffectError::EngineFault("missing Guolun target card".into())
                })?);
            run_effects(
                tx,
                vec![
                    EffectRequest::RevealCards {
                        cards: vec![player_card],
                        to: Vec::new(),
                    },
                    EffectRequest::ResolveXinfuGuolun {
                        player,
                        target,
                        player_card,
                        target_card,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Guolun choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_target_draw(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(
        *payload
            .first()
            .ok_or_else(|| EffectError::EngineFault("missing target-draw chooser".into()))?,
    );
    if observer != player {
        return Err(EffectError::Illegal("target-draw chooser mismatch".into()));
    }
    if (cancelled && payload.len() != 6) || (payload.len() == 3 && selections.len() != 1) {
        return Err(EffectError::Illegal(
            "choose the required number of draw targets".into(),
        ));
    }
    if payload.len() == 3 {
        let cap = payload[1];
        let skill = engine_api::SkillId::new(payload[2]);
        let target = PlayerId::new(selections[0]);
        let participant = tx
            .state()
            .players
            .get(target)
            .filter(|participant| participant.alive)
            .ok_or_else(|| EffectError::Illegal("selected draw target is not alive".into()))?;
        if cap == 0 {
            return Err(EffectError::EngineFault(
                "maximum-HP draw cap must be positive".into(),
            ));
        }
        let count = participant
            .max_hp
            .min(cap)
            .saturating_sub(tx.state().hand_count(target));
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
    let (min_count, max_count, scope_index, skill_index, optional) = match payload.len() {
        4 => (1usize, 1usize, 2usize, 3usize, false),
        5 => (
            payload[2] as usize,
            payload[2] as usize,
            3usize,
            4usize,
            false,
        ),
        6 => (
            payload[2] as usize,
            payload[3] as usize,
            4usize,
            5usize,
            true,
        ),
        _ => {
            return Err(EffectError::EngineFault(
                "bad target-draw continuation".into(),
            ));
        }
    };
    let count = payload[1];
    let scope = u8::try_from(payload[scope_index])
        .ok()
        .and_then(DrawTargetScope::from_tag)
        .ok_or_else(|| EffectError::EngineFault("bad target-draw scope".into()))?;
    let skill = engine_api::SkillId::new(payload[skill_index]);
    if cancelled && optional {
        return Ok(());
    }
    if min_count == 0
        || min_count > max_count
        || cancelled
        || selections.len() < min_count
        || selections.len() > max_count
    {
        return Err(EffectError::Illegal(
            "choose the required number of draw targets".into(),
        ));
    }
    if selections
        .iter()
        .enumerate()
        .any(|(index, token)| selections[..index].contains(token))
    {
        return Err(EffectError::Illegal("draw targets must be distinct".into()));
    }
    let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if selected
        .iter()
        .any(|target| !crate::effect::player_target_matches(tx.state(), player, *target, scope))
    {
        return Err(EffectError::Illegal(
            "selected player is outside the draw target scope".into(),
        ));
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| selected.contains(target))
        .collect();
    if targets.len() != selections.len() {
        return Err(EffectError::Illegal(
            "draw target seat order could not be resolved".into(),
        ));
    }
    run_effects(
        tx,
        targets
            .into_iter()
            .map(|target| EffectRequest::DrawCardsFromSkill {
                player: target,
                count,
                skill,
            })
            .collect(),
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_targets_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 6 {
        return Err(EffectError::EngineFault(
            "bad target-damage continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let min_count = u8::try_from(payload[1])
        .map_err(|_| EffectError::EngineFault("bad target-damage minimum".into()))?;
    let max_count = u8::try_from(payload[2])
        .map_err(|_| EffectError::EngineFault("bad target-damage maximum".into()))?;
    let scope = u8::try_from(payload[3])
        .ok()
        .and_then(DrawTargetScope::from_tag)
        .ok_or_else(|| EffectError::EngineFault("bad target-damage scope".into()))?;
    let amount = payload[4];
    let skill = engine_api::SkillId::new(payload[5]);
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if observer != player
        || cancelled
        || min_count == 0
        || min_count > max_count
        || amount == 0
        || !skill_enabled
        || selections.len() < usize::from(min_count)
        || selections.len() > usize::from(max_count)
    {
        return Err(EffectError::Illegal(
            "choose the required number of damage targets".into(),
        ));
    }
    if selections
        .iter()
        .enumerate()
        .any(|(index, token)| selections[..index].contains(token))
    {
        return Err(EffectError::Illegal(
            "damage targets must be distinct".into(),
        ));
    }
    let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if selected
        .iter()
        .any(|target| !crate::effect::player_target_matches(tx.state(), player, *target, scope))
    {
        return Err(EffectError::Illegal(
            "selected player is outside the damage target scope".into(),
        ));
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| selected.contains(target))
        .collect();
    if targets.len() != selections.len() {
        return Err(EffectError::Illegal(
            "damage target seat order could not be resolved".into(),
        ));
    }
    run_effects(
        tx,
        targets
            .into_iter()
            .map(|target| EffectRequest::DealDamage {
                source: player,
                target,
                amount,
                is_lose_hp: false,
                nature: 0,
            })
            .collect(),
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_target_draw_then_owner_if_below_max_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad target-draw-follow-up continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let count = payload[1];
    let skill = engine_api::SkillId::new(payload[2]);
    let target = selections.first().copied().map(PlayerId::new);
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if observer != player
        || cancelled
        || selections.len() != 1
        || count == 0
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
        })
    {
        return Err(EffectError::Illegal(
            "target-draw-follow-up choice context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
            player,
            target: target.expect("validated target selection"),
            count,
            skill,
            ready_to_check: false,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_target_self_discard_then_owner_lose_hp_if_hand_ne_hp(
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
                            rule_api::TargetCardScope::HandEquip,
                        )
                        .is_empty()
                })
            {
                return Err(EffectError::Illegal(
                    "self-discard target choice context changed".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                        player,
                        target: target.expect("validated self-discard target"),
                        skill,
                        ready_to_check: false,
                    },
                ],
            )
            .map(|_| ())
        }
        Some(1) if payload.len() == 4 => {
            let player = PlayerId::new(payload[1]);
            let target = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let selected = selections.first().copied().map(CardId::new);
            let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player && tx.state().skill_effectively_enabled(instance)
            });
            if observer != target
                || cancelled
                || selections.len() != 1
                || !skill_enabled
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
                || !selected.is_some_and(|card| {
                    crate::query::discardable_target_cards(
                        tx.state(),
                        target,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .contains(&card)
                })
            {
                return Err(EffectError::Illegal(
                    "self-discard card choice context changed".into(),
                ));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ChooseTargetSelfDiscardThenOwnerLoseHpIfHandNeHp {
                    player,
                    target,
                    skill,
                    ready_to_check: true,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: vec![selected.expect("validated self-discard card")],
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad self-discard hand-vs-HP continuation".into(),
        )),
    }
}

pub(crate) fn resolve_choose_give_one_hand_equip_or_recover(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::EngineFault(
            "truncated give-or-recover continuation".into(),
        ));
    }
    let owner = PlayerId::new(payload[0]);
    let giver = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    let option_count = payload[4] as usize;
    if payload.len() != 5 + option_count {
        return Err(EffectError::EngineFault(
            "bad give-or-recover option snapshot".into(),
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
        || selections.len() > 1
        || (cancelled && !selections.is_empty())
    {
        return Err(EffectError::Illegal(
            "give-or-recover choice context changed".into(),
        ));
    }

    if cancelled || selections.is_empty() {
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

    let card = CardId::new(selections[0]);
    if !payload[5..].contains(&card.raw()) {
        return Err(EffectError::Illegal(
            "card was not offered for give-or-recover".into(),
        ));
    }
    let from = response_card_zone(tx, giver, card)?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no recipient hand".into()))?;
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
