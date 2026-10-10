//! Choice-continuation handlers (post 3).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

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
                || !crate::command::play::view_as_bingliang_targets(tx, player, card).contains(&target)
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
                || crate::command::play::validate_sha_target_with_options(tx, target, card, victim, true)
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

pub(crate) fn resolve_resolve_tianxiang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 10 && payload.len() != 11 {
        return Err(EffectError::EngineFault(
            "bad Tianxiang continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let draw_skill_def = engine_api::SkillDefId::new(payload[2]);
    let source = PlayerId::new(payload[3]);
    let amount = payload[4];
    let nature = payload[5] as u8;
    let card = engine_api::CardId::new(payload[6]);
    let card_kind = payload[7] as u8;
    let card_flags = payload[8] as u8;
    let skill = engine_api::SkillId::new(payload[9]);
    if observer != player
        || cancelled
        || tx.state().players.get(player).is_none()
        || draw_skill_def.raw() == 0
        || source.is_none()
        || amount == 0
    {
        return Err(EffectError::Illegal(
            "Tianxiang redirect context changed".into(),
        ));
    }
    if payload.len() == 10 {
        let [selected] = selections else {
            return Err(EffectError::Illegal(
                "Tianxiang requires exactly one heart card".into(),
            ));
        };
        let selected = engine_api::CardId::new(*selected);
        let valid_heart = tx.state().cards.get(selected).is_some_and(|card_state| {
            card_state.suit == engine_api::Suit::Heart
                && crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .contains(&selected)
        });
        if !valid_heart {
            return Err(EffectError::Illegal(
                "Tianxiang heart discard cost is invalid or stale".into(),
            ));
        }
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != player)
            .map(PlayerId::raw)
            .collect();
        if targets.is_empty() {
            return Err(EffectError::Illegal(
                "Tianxiang has no redirect targets".into(),
            ));
        }
        let mut next = payload.to_vec();
        next.push(selected.raw());
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            engine_api::ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            next,
        )
        .map(|_| ());
    }
    let [selected_target] = selections else {
        return Err(EffectError::Illegal(
            "Tianxiang requires exactly one redirect target".into(),
        ));
    };
    let target = PlayerId::new(*selected_target);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Tianxiang redirect target is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveTianxiang {
            player,
            target,
            discard_card: engine_api::CardId::new(payload[10]),
            source,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            draw_skill_def,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_retianxiang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if !(4..=6).contains(&payload.len()) {
        return Err(EffectError::EngineFault(
            "bad Retianxiang continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let source = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player || cancelled || tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal(
            "Retianxiang branch context changed".into(),
        ));
    }
    if payload.len() == 4 {
        let [selected] = selections else {
            return Err(EffectError::Illegal(
                "Retianxiang requires exactly one heart card".into(),
            ));
        };
        let selected = engine_api::CardId::new(*selected);
        let valid_heart = tx.state().cards.get(selected).is_some_and(|card_state| {
            card_state.suit == engine_api::Suit::Heart
                && crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .contains(&selected)
        });
        if !valid_heart {
            return Err(EffectError::Illegal(
                "Retianxiang heart discard cost is invalid or stale".into(),
            ));
        }
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != player)
            .map(PlayerId::raw)
            .collect();
        if targets.is_empty() {
            return Err(EffectError::Illegal(
                "Retianxiang has no branch targets".into(),
            ));
        }
        let mut next = payload.to_vec();
        next.push(selected.raw());
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            engine_api::ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            next,
        )
        .map(|_| ());
    }
    if payload.len() == 5 {
        let [selected_target] = selections else {
            return Err(EffectError::Illegal(
                "Retianxiang requires exactly one branch target".into(),
            ));
        };
        let target = PlayerId::new(*selected_target);
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "Retianxiang branch target is invalid or stale".into(),
            ));
        }
        let mut next = payload.to_vec();
        next.push(target.raw());
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            engine_api::ChoiceKind::Custom,
            ContinuationKind::Custom,
            next,
        )
        .map(|_| ());
    }
    let [selected_branch] = selections else {
        return Err(EffectError::Illegal(
            "Retianxiang requires exactly one branch".into(),
        ));
    };
    let branch = (*selected_branch).min(1) as u8;
    let target = PlayerId::new(payload[5]);
    if target == player {
        return Err(EffectError::Illegal(
            "Retianxiang branch target is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveRetianxiang {
            player,
            target,
            discard_card: engine_api::CardId::new(payload[4]),
            branch,
            source,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_lianzhou(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("bad Lianzhou continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player
        || cancelled
        || selections.is_empty()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Lianzhou link context changed".into()));
    }
    let owner_hp = tx.state().players.get(player).unwrap().hp;
    let targets: Vec<_> = selections.iter().map(|raw| PlayerId::new(*raw)).collect();
    let valid = targets.iter().all(|&target| {
        target != player
            && tx.state().players.get(target).is_some_and(|participant| {
                participant.alive
                    && participant.hp == owner_hp
                    && !participant
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
            })
    });
    if !valid {
        return Err(EffectError::Illegal(
            "Lianzhou link targets are invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveLianzhou {
            player,
            targets,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_stddechong(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad Stddechong continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player
        || cancelled
        || selections.is_empty()
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
        || player == target
    {
        return Err(EffectError::Illegal(
            "Stddechong gift context changed".into(),
        ));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(|card| card.raw())
            .collect();
    let cards: Vec<_> = selections
        .iter()
        .map(|raw| engine_api::CardId::new(*raw))
        .collect();
    let valid = cards.iter().all(|card| {
        allowed.contains(&card.raw())
            && tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player)
    });
    if !valid {
        return Err(EffectError::Illegal(
            "Stddechong gift cards are invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveStddechong {
            player,
            target,
            cards,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_discard_hand_equip_count(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad discard-count continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let count = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player
        || cancelled
        || count == 0
        || selections.len() as u32 != count
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "discard-count choice context changed".into(),
        ));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(|card| card.raw())
            .collect();
    let cards: Vec<_> = selections
        .iter()
        .map(|raw| engine_api::CardId::new(*raw))
        .collect();
    let valid = cards.iter().all(|card| {
        allowed.contains(&card.raw())
            && tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player)
    });
    if !valid {
        return Err(EffectError::Illegal(
            "discard-count cards are invalid or stale".into(),
        ));
    }
    let _ = skill;
    run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).map(|_| ())
}

pub(crate) fn resolve_choose_discard_hand_range(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 {
        return Err(EffectError::EngineFault(
            "bad discard-hand-range continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let min = payload[2];
    let max = payload[3];
    let count = selections.len() as u32;
    if observer != player
        || cancelled
        || min == 0
        || max < min
        || count < min
        || count > max
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "discard-hand-range choice context changed".into(),
        ));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::effect::standard_ganglie_discardable_hand_cards(tx, player)
            .into_iter()
            .map(|card| card.raw())
            .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if selected.len() != selections.len()
        || count > allowed.len() as u32
        || !selected.iter().all(|card| allowed.contains(card))
    {
        return Err(EffectError::Illegal(
            "discard-hand-range cards are invalid or stale".into(),
        ));
    }
    let cards = selections
        .iter()
        .copied()
        .map(engine_api::CardId::new)
        .collect();
    run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).map(|_| ())
}

pub(crate) fn resolve_dcctjiuxian_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .get(1)
        .ok_or_else(|| EffectError::EngineFault("truncated Dcctjiuxian choice".into()))?;
    match stage {
        0 if payload.len() == 5 => {
            let player = PlayerId::new(payload[2]);
            let count = payload[3] as usize;
            let skill = engine_api::SkillId::new(payload[4]);
            let hand: std::collections::BTreeSet<_> =
                tx.state().hand_cards(player).into_iter().collect();
            let cards: Vec<_> = selections
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect();
            let distinct: std::collections::BTreeSet<_> = cards.iter().copied().collect();
            if observer != player
                || cancelled
                || count == 0
                || cards.len() != count
                || distinct.len() != cards.len()
                || !cards.iter().all(|card| hand.contains(card))
            {
                return Err(EffectError::Illegal(
                    "Dcctjiuxian recast choice is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcctjiuxian {
                    player,
                    event_player: player,
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    action: 1,
                    cards,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 if payload.len() == 5 => {
            let player = PlayerId::new(payload[2]);
            let damaged = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            if observer != player {
                return Err(EffectError::Illegal(
                    "only the Dcctjiuxian owner may choose recovery".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Dcctjiuxian recovery chooses at most one target".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            if !crate::effect::dcctjiuxian_recovery_targets(tx, player, damaged).contains(&target) {
                return Err(EffectError::Illegal(
                    "Dcctjiuxian recovery target is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcctjiuxian {
                    player,
                    event_player: damaged,
                    target,
                    card: engine_api::CardId::NONE,
                    action: 3,
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad Dcctjiuxian continuation".into(),
        )),
    }
}

pub(crate) fn resolve_bianyue_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("truncated Bianyue choice".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Bianyue target choice is invalid".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    if !crate::effect::bianyue_wuxie_targets(tx, player).contains(&target) {
        return Err(EffectError::Illegal(
            "Bianyue target choice is stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveBianyue {
            player,
            target,
            action: 11,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_zhupeilan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let mode = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("Zhupeilan choice is invalid".into()));
    }
    match mode {
        0 => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault("truncated Cilv choice".into()));
            }
            let count = payload[6] as usize;
            if payload.len() != 7 + count || selections[0] > 2 {
                return Err(EffectError::Illegal("malformed Cilv choice".into()));
            }
            let card = engine_api::CardId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let helper_skill_def = engine_api::SkillDefId::new(payload[5]);
            let event_cards = payload[7..]
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect();
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhupeilan {
                    player,
                    target: PlayerId::NONE,
                    card,
                    helper_skill_def,
                    action: 3 + selections[0] as u8,
                    event_cards,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() != 4 {
                return Err(EffectError::EngineFault("truncated Tongdao choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            if !tx.state().players.alive_by_seat().contains(&target) {
                return Err(EffectError::Illegal("Tongdao target is stale".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhupeilan {
                    player,
                    target,
                    card: engine_api::CardId::NONE,
                    helper_skill_def: engine_api::SkillDefId::NONE,
                    action: 11,
                    event_cards: Vec::new(),
                    skill: engine_api::SkillId::new(payload[3]),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown Zhupeilan choice".into())),
    }
}

pub(crate) fn resolve_resolve_jieliang_gain(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Jieliang gain continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player
        || cancelled
        || selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Jieliang gain choice context changed".into(),
        ));
    }
    let card = engine_api::CardId::new(selections[0]);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let in_discard = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card_state| card_state.zone == discard);
    if !in_discard {
        return Err(EffectError::Illegal(
            "Jieliang gain card is no longer in the discard pile".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("gainer hand zone missing".into()))?;
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

pub(crate) fn resolve_resolve_stdjujian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "bad Stdjujian continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let cards: Vec<_> = payload[3..]
        .iter()
        .map(|raw| engine_api::CardId::new(*raw))
        .collect();
    if observer != player
        || cancelled
        || selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Stdjujian gain context changed".into(),
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
            "Stdjujian gain target is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveStdjujian {
            player,
            target,
            cards,
            skill,
        }],
    )
    .map(|_| ())
}

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
    let discarded_sha = tx
        .state()
        .cards
        .get(cost)
        .is_some_and(|card| crate::query::effective_card_kind(tx.state(), source, card).is_sha());
    let mut effects = vec![EffectRequest::DiscardCards {
        player: source,
        cards: vec![cost],
    }];
    if discarded_sha {
        effects.push(EffectRequest::DrawCards {
            player,
            count: 1,
        });
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
                return Err(EffectError::Illegal("must select exactly one target".into()));
            }
            let target = PlayerId::new(selections[0]);
            let target_alive = tx
                .state()
                .players
                .get(target)
                .is_some_and(|p| p.alive);
            if target == player || !target_alive {
                return Err(EffectError::Illegal(
                    "target must be another living player".into(),
                ));
            }
            let options = tx.state().hand_cards(target);
            if options.is_empty() {
                return Err(EffectError::Illegal("target no longer has hand cards".into()));
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
                return Err(EffectError::Illegal(
                    "must select exactly one card".into(),
                ));
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
        return Err(EffectError::Illegal("expansion place card not in hand".into()));
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
        return Err(EffectError::Illegal("gain-from-pile chooser mismatch".into()));
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
