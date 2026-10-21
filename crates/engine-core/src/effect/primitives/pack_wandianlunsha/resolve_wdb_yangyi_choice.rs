//! Effect primitives for `resolve_wdb_yangyi_choice` and related helpers.

use super::*;

pub(crate) fn resolve_wdb_yangyi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_WDB_YANGYI_CHOICE)
        || payload.len() < 2
        || (cancelled && !selections.is_empty())
    {
        return Err(EffectError::Illegal("stale 杨仪/调兵遣将 choice".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() != 5 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("bad 筹度 target choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let cost = CardId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let target = PlayerId::new(selections[0]);
            let cost_state = tx
                .state()
                .cards
                .get(cost)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("筹度 cost disappeared".into()))?;
            if observer != player
                || !originating_skill_is_enabled(tx, player, skill)
                || cost_state.owner != player
                || !tx
                    .state()
                    .zones
                    .get(cost_state.zone)
                    .is_some_and(|zone| matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip))
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive && !participant.dying)
            {
                return Err(EffectError::Illegal("invalid 筹度 target choice".into()));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveWdbYangyi {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 3,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::MoveCardsWithReason {
                    cards: vec![cost],
                    to: discard,
                    from: Some(cost_state.zone),
                    reason: CardMoveReason::Discard,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() != 6 || selections.len() > 1 {
                return Err(EffectError::Illegal("bad 立断 choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let gained = CardId::new(payload[4]);
            let skill = engine_api::SkillId::new(payload[5]);
            if observer != target || !originating_skill_is_enabled(tx, player, skill) {
                return Err(EffectError::Illegal("invalid 立断 choice observer".into()));
            }
            if cancelled || selections.is_empty() {
                crate::command::play::use_equipment_from_hand(tx, target, gained)
            } else {
                let selected = CardId::new(selections[0]);
                let selected_state = tx
                    .state()
                    .cards
                    .get(selected)
                    .cloned()
                    .ok_or_else(|| EffectError::Illegal("立断 gift disappeared".into()))?;
                if selected_state.owner != target
                    || !tx
                        .state()
                        .zones
                        .get(selected_state.zone)
                        .is_some_and(|zone| zone.kind == ZoneKind::Hand && zone.owner == target)
                {
                    return Err(EffectError::Illegal("invalid 立断 gift".into()));
                }
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::EngineFault("立断 owner hand missing".into()))?;
                run_effects(
                    tx,
                    vec![EffectRequest::GiveCards {
                        cards: vec![selected],
                        to: hand,
                        from: selected_state.zone,
                    }],
                )
                .map(|_| ())
            }
        }
        2 => {
            if selections.len() > 1 || payload.len() < 4 {
                return Err(EffectError::Illegal("bad 调兵遣将 pool choice".into()));
            }
            let queue_len = payload[2] as usize;
            if payload.len() != 3 + queue_len {
                return Err(EffectError::Illegal("bad 调兵遣将 queue payload".into()));
            }
            let queue = &payload[3..];
            let (_, _, target, _, revealed) = diaobing_queue_parts(queue)?;
            if observer != target {
                return Err(EffectError::Illegal("调兵遣将 target changed".into()));
            }
            if cancelled || selections.is_empty() {
                let mut next = queue.to_vec();
                next[5] = next[5].saturating_add(1);
                return crate::command::play::advance_independent_trick_queue(tx, &next);
            }
            let selected = CardId::new(selections[0]);
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            if !revealed.contains(&selected)
                || !tx
                    .state()
                    .cards
                    .get(selected)
                    .is_some_and(|card| card.zone == processing)
            {
                return Err(EffectError::Illegal(
                    "invalid 调兵遣将 revealed card".into(),
                ));
            }
            let hand = tx.state().hand_cards(target);
            if hand.is_empty() {
                return Err(EffectError::Illegal(
                    "调兵遣将 target hand disappeared".into(),
                ));
            }
            let mut next = vec![
                RESOLVE_WDB_YANGYI_CHOICE,
                3,
                selected.raw(),
                queue_len.min(u32::MAX as usize) as u32,
            ];
            next.extend_from_slice(queue);
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        3 => {
            if cancelled || selections.len() != 1 || payload.len() < 5 {
                return Err(EffectError::Illegal(
                    "bad 调兵遣将 replacement choice".into(),
                ));
            }
            let selected = CardId::new(payload[2]);
            let queue_len = payload[3] as usize;
            if payload.len() != 4 + queue_len {
                return Err(EffectError::Illegal(
                    "invalid 调兵遣将 replacement payload".into(),
                ));
            }
            let queue = payload[4..].to_vec();
            let (actor, source_card, target, targets, mut revealed) = diaobing_queue_parts(&queue)?;
            if observer != target {
                return Err(EffectError::Illegal("调兵遣将 target changed".into()));
            }
            let replacement = CardId::new(selections[0]);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("target hand missing".into()))?;
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            if !tx
                .state()
                .cards
                .get(replacement)
                .is_some_and(|card| card.owner == target && card.zone == hand)
                || !revealed.contains(&selected)
                || !tx
                    .state()
                    .cards
                    .get(selected)
                    .is_some_and(|card| card.zone == processing)
            {
                return Err(EffectError::Illegal("stale 调兵遣将 swap".into()));
            }
            revealed.retain(|card| *card != selected);
            revealed.push(replacement);
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ContinueDiaobingqianjiang {
                    action: 1,
                    actor,
                    source_card,
                    targets,
                    revealed,
                },
            );
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCardsWithReason {
                        cards: vec![replacement],
                        to: processing,
                        from: Some(hand),
                        reason: CardMoveReason::Unspecified,
                    },
                    EffectRequest::MoveCardsWithReason {
                        cards: vec![selected],
                        to: hand,
                        from: Some(processing),
                        reason: CardMoveReason::Unspecified,
                    },
                ],
            )
            .map(|_| ())
        }
        4 => {
            if cancelled || selections.is_empty() || payload.len() < 5 {
                return Err(EffectError::Illegal("bad 调兵遣将 top-deck choice".into()));
            }
            let actor = PlayerId::new(payload[2]);
            let count = payload[4] as usize;
            if observer != actor || payload.len() != 5 + count {
                return Err(EffectError::Illegal(
                    "invalid 调兵遣将 top-deck payload".into(),
                ));
            }
            let revealed: Vec<_> = payload[5..].iter().copied().map(CardId::new).collect();
            let selected: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let unique: std::collections::BTreeSet<_> = selected.iter().copied().collect();
            if unique.len() != selected.len()
                || selected.iter().any(|card| !revealed.contains(card))
            {
                return Err(EffectError::Illegal(
                    "invalid 调兵遣将 top-deck cards".into(),
                ));
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            if revealed.iter().any(|card| {
                !tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|state| state.zone == processing)
            }) {
                return Err(EffectError::Illegal("调兵遣将 pool changed".into()));
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let rest: Vec<_> = revealed
                .into_iter()
                .filter(|card| !unique.contains(card))
                .collect();
            let mut effects = Vec::new();
            if !selected.is_empty() {
                effects.push(EffectRequest::MoveCardsWithReason {
                    cards: selected.into_iter().rev().collect(),
                    to: draw,
                    from: Some(processing),
                    reason: CardMoveReason::Unspecified,
                });
            }
            if !rest.is_empty() {
                effects.push(EffectRequest::MoveCardsWithReason {
                    cards: rest,
                    to: discard,
                    from: Some(processing),
                    reason: CardMoveReason::Discard,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "unknown 杨仪/调兵遣将 choice stage".into(),
        )),
    }
}
