//! Choice continuations for `DyingSave` and related requests.

use super::*;

pub(super) fn resolve_choice(
    tx: &mut Transaction,
    _actor: PlayerId,
    observer: PlayerId,
    kind: ContinuationKind,
    payload: Vec<u32>,
    selections: &[u32],
    cancelled: bool,
    _vm_cont: (Vec<u8>, String, u32, u32, [u32; 4], u32, [u32; 3]),
    _optional_trigger: Option<crate::trigger::PendingTrigger>,
) -> Result<(), EffectError> {
    match kind {
        ContinuationKind::DyingSave => {
            let dying = PlayerId::new(*payload.first().unwrap_or(&0));
            let killer = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let responder = PlayerId::new(*payload.get(2).unwrap_or(&0));
            let len = *payload.get(3).unwrap_or(&u32::MAX) as usize;
            let remaining = payload
                .get(4..)
                .filter(|rest| rest.len() == len)
                .ok_or_else(|| EffectError::EngineFault("truncated dying-save queue".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            if observer != responder {
                return Err(EffectError::Illegal(
                    "dying save must be answered by the current responder".into(),
                ));
            }
            let pick = if cancelled {
                0
            } else {
                selections.first().copied().unwrap_or(0)
            };
            if pick == 0 {
                run_effects(
                    tx,
                    vec![EffectRequest::ContinueDyingSave {
                        player: dying,
                        source: killer,
                        responders: remaining,
                    }],
                )?;
            } else if pick == 1 {
                let (save_card, save_kind) =
                    crate::query::dying_save_card_and_kind_for(tx.state(), responder, dying)
                        .ok_or_else(|| EffectError::Illegal("card save disabled".into()))?;
                let from = tx
                    .state()
                    .cards
                    .get(save_card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::Illegal("missing save card".into()))?;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                let mut responders = vec![responder];
                responders.extend(remaining);
                let mut effects = vec![EffectRequest::MoveCardsWithReason {
                    cards: vec![save_card],
                    to: discard,
                    from: Some(from),
                    reason: CardMoveReason::Respond,
                }];
                append_pothaoshi_refills(tx.state(), responder, &[save_card], &mut effects);
                effects.push(EffectRequest::CardResponded {
                    player: responder,
                    respond_to: dying,
                    card: save_card,
                    response_kind: save_kind.tag(),
                });
                effects.extend([
                    EffectRequest::RecoverHp {
                        source: responder,
                        player: dying,
                        amount: 1,
                    },
                    EffectRequest::ContinueDyingSave {
                        player: dying,
                        source: killer,
                        responders,
                    },
                ]);
                run_effects(tx, effects)?;
            } else if let Some((skill, kind)) = [CardKind::Tao, CardKind::Jiu]
                .into_iter()
                .filter(|kind| *kind != CardKind::Jiu || responder == dying)
                .find_map(|kind| {
                    super::super::skill::source_free_response_skill(
                        tx.state(),
                        responder,
                        kind,
                        pick,
                    )
                    .map(|skill| (skill, kind))
                })
            {
                return super::super::skill::begin_source_free_response(
                    tx, responder, skill, kind, 5, &payload,
                );
            } else if let Some((skill, kind)) = [CardKind::Tao, CardKind::Jiu]
                .into_iter()
                .filter(|kind| *kind != CardKind::Jiu || responder == dying)
                .find_map(|kind| {
                    super::super::skill::multi_card_response_skill(
                        tx.state(),
                        responder,
                        kind,
                        pick,
                    )
                    .map(|skill| (skill, kind))
                })
            {
                return super::super::skill::begin_multi_card_response(
                    tx, responder, skill, kind, 5, &payload,
                );
            } else {
                return Err(EffectError::Illegal("invalid dying-save option".into()));
            }
            resume_interrupted_continuation(tx)
        }

        ContinuationKind::DiscardToLimit => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let need = *payload.get(1).unwrap_or(&0) as usize;
            if cancelled {
                return Err(EffectError::Illegal("discard cannot cancel".into()));
            }
            if selections.len() != need {
                return Err(EffectError::Illegal(format!(
                    "must discard exactly {need} cards"
                )));
            }
            let legal: std::collections::BTreeSet<u32> =
                crate::query::discard_to_limit_tokens(tx.state(), player)
                    .into_iter()
                    .map(|card| card.raw())
                    .collect();
            let mut cards = Vec::new();
            for token in selections {
                let cid = CardId::new(*token);
                let c = tx
                    .state()
                    .cards
                    .get(cid)
                    .ok_or_else(|| EffectError::Illegal("bad discard card".into()))?;
                if c.owner != player {
                    return Err(EffectError::Illegal(
                        "discard card not owned by player".into(),
                    ));
                }
                if !legal.contains(token) {
                    return Err(EffectError::Illegal(
                        "card cannot be discarded for hand limit".into(),
                    ));
                }
                cards.push(cid);
            }
            let mut seen = std::collections::BTreeSet::new();
            for c in &cards {
                if !seen.insert(c.raw()) {
                    return Err(EffectError::Illegal("duplicate discard".into()));
                }
            }
            run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])?;
            use crate::state::Phase;
            tx.state_mut().turn.phase = Phase::End;
            let current = tx.state().turn.current;
            tx.push_event(engine_api::DomainEvent::TurnStateChanged {
                current,
                phase: Phase::End.to_tag(),
            });
            Ok(())
        }

        ContinuationKind::ReorderDrawPile => {
            // payload: [player, take, c0..] where c0.. are the peeked top cards
            // listed bottom→top (draw order last). Selected tokens sink to the
            // bottom (in selection order); the rest keep their original order
            // on top. Cancel = keep all on top unchanged.
            let take = *payload.get(1).unwrap_or(&0) as usize;
            let peeked: Vec<u32> = payload.iter().skip(2).take(take).copied().collect();
            let selected: std::collections::BTreeSet<u32> = if cancelled {
                Default::default()
            } else {
                selections.iter().copied().collect()
            };
            // Validate selections belong to the peeked set.
            for s in &selected {
                if !peeked.contains(s) {
                    return Err(EffectError::Illegal("scry: card not among peeked".into()));
                }
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
            let z = tx
                .state()
                .zones
                .get(draw)
                .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
            let total = z.cards.len();
            if total < take {
                return Err(EffectError::EngineFault("scry: pile shrank".into()));
            }
            // `peeked` is bottom→top; iterate top→bottom for stable "keep on top".
            let mut keep_top: Vec<u32> = Vec::new(); // will be applied top-most first
            let mut to_bottom: Vec<u32> = Vec::new();
            for &cid in peeked.iter().rev() {
                if selected.contains(&cid) {
                    to_bottom.push(cid);
                } else {
                    keep_top.push(cid);
                }
            }
            // Rebuild (stored bottom→top): [ selected (new bottom) ][ untouched
            // middle ][ unselected (top, original order preserved) ]. Both
            // keep_top and to_bottom are currently top→bottom, so reverse each to
            // store bottom→top; the original top-most card ends up last (drawn
            // first). Selected-card relative order follows the peek, not raw
            // click order — a deterministic simplification of full free reorder.
            let z = tx.state_mut().zones.get_mut(draw).unwrap();
            let middle: Vec<engine_api::CardId> = z.cards[..total - take].to_vec();
            let mut rebuilt: Vec<engine_api::CardId> = Vec::with_capacity(total);
            for cid in to_bottom.iter().rev() {
                rebuilt.push(engine_api::CardId::new(*cid));
            }
            rebuilt.extend(middle);
            for cid in keep_top.iter().rev() {
                rebuilt.push(engine_api::CardId::new(*cid));
            }
            z.cards = rebuilt;
            use crate::state::Phase;
            tx.state_mut().turn.phase = Phase::Judge;
            let current = tx.state().turn.current;
            tx.push_event(engine_api::DomainEvent::TurnStateChanged {
                current,
                phase: Phase::Judge.to_tag(),
            });
            Ok(())
        }

        ContinuationKind::ReorderDrawPileFourSplitTwoThenDraw => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let take = *payload.get(1).unwrap_or(&0) as usize;
            let peeked: Vec<u32> = payload.iter().skip(2).take(take).copied().collect();
            if !cancelled {
                if take < 2 || selections.len() != take {
                    return Err(EffectError::Illegal(
                        "draw-phase scry: must order every offered card".into(),
                    ));
                }
                let selected: std::collections::BTreeSet<u32> =
                    selections.iter().copied().collect();
                let offered: std::collections::BTreeSet<u32> = peeked.iter().copied().collect();
                if selected.len() != take || selected != offered {
                    return Err(EffectError::Illegal(
                        "draw-phase scry: selection must match offered cards".into(),
                    ));
                }
                let draw = tx
                    .state()
                    .zones
                    .find(ZoneKind::DrawPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
                let zone = tx
                    .state()
                    .zones
                    .get(draw)
                    .ok_or_else(|| EffectError::EngineFault("draw zone missing".into()))?;
                let total = zone.cards.len();
                if total < take
                    || zone.cards[total - take..]
                        .iter()
                        .map(|card| card.raw())
                        .ne(peeked.iter().copied())
                {
                    return Err(EffectError::EngineFault(
                        "draw-phase scry: draw pile changed while choice was open".into(),
                    ));
                }
                let top_count = take - 2;
                let middle = zone.cards[..total - take].to_vec();
                let mut rebuilt = Vec::with_capacity(total);
                rebuilt.extend(selections[top_count..].iter().copied().map(CardId::new));
                rebuilt.extend(middle);
                rebuilt.extend(
                    selections[..top_count]
                        .iter()
                        .rev()
                        .copied()
                        .map(CardId::new),
                );
                tx.state_mut().zones.get_mut(draw).unwrap().cards = rebuilt;
            }
            super::super::turn::continue_draw_phase(tx, player)
        }

        ContinuationKind::DrawReplacement => {
            use crate::state::turn_flag;
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let draw_n = *payload.get(1).unwrap_or(&2);
            let forgo = !cancelled && selections.contains(&1);
            if forgo {
                tx.state_mut().turn.set_flag(turn_flag::DRAW_REPLACED, 1);
            }
            let draw_n = draw_n.saturating_sub(u32::from(forgo));
            if super::super::turn::open_optional_draw_bonus(tx, player, draw_n, true)? {
                return Err(EffectError::ChoiceRequested);
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player,
                        count: draw_n,
                    }],
                )?;
            }
            super::super::turn::finish_replaced_draw_phase(tx);
            Ok(())
        }

        ContinuationKind::OptionalDrawBonus => {
            let player = PlayerId::new(*payload.first().unwrap_or(&0));
            let mut draw_n = *payload.get(1).unwrap_or(&2);
            let advance_after = *payload.get(2).unwrap_or(&0) != 0;
            let bonuses = payload.get(3..).unwrap_or_default();
            if !cancelled && selections.first().copied() == Some(1) {
                draw_n = draw_n.saturating_add(*bonuses.first().unwrap_or(&0));
            }
            if bonuses.len() > 1 {
                crate::effect::open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    engine_api::ChoiceKind::Confirm,
                    ContinuationKind::OptionalDrawBonus,
                    [player.raw(), draw_n, u32::from(advance_after)]
                        .into_iter()
                        .chain(bonuses[1..].iter().copied())
                        .collect(),
                )?;
                return Err(EffectError::ChoiceRequested);
            }
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player,
                    count: draw_n,
                }],
            )?;
            if advance_after {
                super::super::turn::finish_replaced_draw_phase(tx);
            }
            Ok(())
        }

        ContinuationKind::PickTargetCard => {
            // mode 0=过河 discard (hand/equip/judge), 1=take (hand/equip),
            // 2=take (equip), 3=take (hand), 10..14=typed take scope.
            let target = PlayerId::new(*payload.first().unwrap_or(&0));
            let actor = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let mode = *payload.get(2).unwrap_or(&0);
            if cancelled {
                return Err(EffectError::Illegal("must pick a card".into()));
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("pick exactly one card".into()));
            }
            let cid = CardId::new(selections[0]);
            let c = tx
                .state()
                .cards
                .get(cid)
                .ok_or_else(|| EffectError::Illegal("bad card".into()))?
                .clone();
            if c.owner != target {
                return Err(EffectError::Illegal("card not owned by target".into()));
            }
            let from_zone = c.zone;
            // Validate zone allowed
            let in_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .map(|z| z == from_zone)
                .unwrap_or(false);
            let in_equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .map(|z| z == from_zone)
                .unwrap_or(false);
            let in_judge = tx
                .state()
                .zones
                .find(ZoneKind::Judge, target)
                .map(|z| z == from_zone)
                .unwrap_or(false);
            let take_scope = match mode {
                1 => Some(rule_api::TargetCardScope::HandEquip),
                2 => Some(rule_api::TargetCardScope::Equip),
                3 => Some(rule_api::TargetCardScope::Hand),
                10..=14 => rule_api::TargetCardScope::from_tag((mode - 10) as u8),
                _ => None,
            };
            let result = if let Some(scope) = take_scope {
                if !crate::query::gainable_target_cards(tx.state(), actor, target, scope)
                    .contains(&cid)
                {
                    return Err(EffectError::Illegal("cannot take this card".into()));
                }
                let to_hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, actor)
                    .ok_or_else(|| EffectError::Illegal("no actor hand".into()))?;
                super::super::play::run_effects_then_finish_independent_trick_target(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: vec![cid],
                        to: to_hand,
                        from: Some(from_zone),
                    }],
                    payload.get(3..).unwrap_or_default(),
                )
            } else if mode == 20 {
                // store to actor's skill pile (ABI 567): hand only
                if !in_hand {
                    return Err(EffectError::Illegal(
                        "skill-pile storage requires a hand card".into(),
                    ));
                }
                let skill_id = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
                let skill_def = crate::effect::skill_pile_def(tx, actor, skill_id)?;
                let pile = crate::effect::ensure_skill_pile_zone(tx, actor, skill_def)?;
                super::super::play::run_effects_then_finish_independent_trick_target(
                    tx,
                    vec![EffectRequest::MoveCards {
                        cards: vec![cid],
                        to: pile,
                        from: Some(from_zone),
                    }],
                    payload.get(4..).unwrap_or_default(),
                )
            } else {
                // discard: hand / equip / judge
                if !in_hand && !in_equip && !in_judge {
                    return Err(EffectError::Illegal("cannot discard this card".into()));
                }
                if crate::query::card_discard_protected_from(tx.state(), actor, target, cid) {
                    return Err(EffectError::Illegal(
                        "card is protected from discard".into(),
                    ));
                }
                if in_judge {
                    let discard = tx
                        .state()
                        .zones
                        .find(ZoneKind::DiscardPile, PlayerId::NONE)
                        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
                    super::super::play::run_effects_then_finish_independent_trick_target(
                        tx,
                        vec![EffectRequest::MoveCards {
                            cards: vec![cid],
                            to: discard,
                            from: Some(from_zone),
                        }],
                        payload.get(3..).unwrap_or_default(),
                    )
                } else {
                    super::super::play::run_effects_then_finish_independent_trick_target(
                        tx,
                        vec![EffectRequest::DiscardCards {
                            player: target,
                            cards: vec![cid],
                        }],
                        payload.get(3..).unwrap_or_default(),
                    )
                }
            };
            result
        }

        ContinuationKind::HuogongShow => {
            // payload: [actor, target, optional queued-trick tail...]
            let actor = PlayerId::new(*payload.first().unwrap_or(&0));
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            if cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("must show one card".into()));
            }
            let shown = CardId::new(selections[0]);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
            let c = tx
                .state()
                .cards
                .get(shown)
                .ok_or_else(|| EffectError::Illegal("bad card".into()))?
                .clone();
            if c.zone != hand || c.owner != target {
                return Err(EffectError::Illegal("shown card not in hand".into()));
            }
            let suit = crate::query::effective_card_suit(tx.state(), target, &c);
            // Actor discards a same-suit card or cancels
            let actor_hand = tx.state().hand_cards(actor);
            let tokens: Vec<u32> = actor_hand
                .iter()
                .filter(|cid| {
                    tx.state()
                        .cards
                        .get(**cid)
                        .map(|x| crate::query::effective_card_suit(tx.state(), actor, x) == suit)
                        .unwrap_or(false)
                })
                .map(|c| c.raw())
                .collect();
            run_effects(
                tx,
                vec![EffectRequest::EmitRuleEvent {
                    code: 210, // huogong show
                    payload: vec![
                        actor.raw(),
                        target.raw(),
                        shown.raw(),
                        super::super::judge::suit_tag(suit),
                    ],
                }],
            )?;
            if tokens.is_empty() {
                // Cannot discard matching suit — fire fails
                return super::super::play::finish_independent_trick_target(
                    tx,
                    payload.get(2..).unwrap_or_default(),
                );
            }
            let mut next_payload = vec![
                actor.raw(),
                target.raw(),
                super::super::judge::suit_tag(suit),
            ];
            next_payload.extend_from_slice(payload.get(2..).unwrap_or_default());
            crate::effect::open_choice(
                tx,
                actor,
                0,
                1,
                true, // cancel = no damage
                tokens,
                engine_api::ChoiceKind::SelectCards,
                crate::state::ContinuationKind::HuogongDiscard,
                next_payload,
            )?;
            Err(EffectError::ChoiceRequested)
        }

        ContinuationKind::HuogongDiscard => {
            // payload: [actor, target, suit_tag, optional queued-trick tail...]
            let actor = PlayerId::new(*payload.first().unwrap_or(&0));
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let suit = super::super::judge::suit_from_tag(*payload.get(2).unwrap_or(&1));
            if cancelled || selections.is_empty() {
                return super::super::play::finish_independent_trick_target(
                    tx,
                    payload.get(3..).unwrap_or_default(),
                );
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("discard one card".into()));
            }
            let cid = CardId::new(selections[0]);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, actor)
                .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
            let c = tx
                .state()
                .cards
                .get(cid)
                .ok_or_else(|| EffectError::Illegal("bad card".into()))?
                .clone();
            if c.zone != hand
                || c.owner != actor
                || crate::query::effective_card_suit(tx.state(), actor, &c) != suit
            {
                return Err(EffectError::Illegal("must discard same suit".into()));
            }
            super::super::play::run_effects_then_finish_independent_trick_target(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: actor,
                        cards: vec![cid],
                    },
                    {
                        let (card, bonus) =
                            super::super::play::saved_independent_trick_damage_card_and_bonus(
                                payload.get(3..).unwrap_or_default(),
                            );
                        if card.is_some() {
                            EffectRequest::DealCardDamage {
                                source: actor,
                                target,
                                amount: 1u32.saturating_add(bonus),
                                is_lose_hp: false,
                                nature: rule_api::damage_nature::FIRE,
                                card,
                                card_kind: CardKind::TrickHuogong.tag(),
                                card_flags: 0,
                            }
                        } else {
                            EffectRequest::DealDamage {
                                source: actor,
                                target,
                                amount: 1,
                                is_lose_hp: false,
                                nature: rule_api::damage_nature::FIRE,
                            }
                        }
                    },
                    EffectRequest::EmitRuleEvent {
                        code: 211, // huogong hit
                        payload: vec![actor.raw(), target.raw(), cid.raw()],
                    },
                ],
                payload.get(3..).unwrap_or_default(),
            )
        }

        ContinuationKind::JuedouDuel => {
            let source = PlayerId::new(*payload.first().unwrap_or(&0));
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let next = PlayerId::new(*payload.get(2).unwrap_or(&0));
            let exchanges = *payload.get(3).unwrap_or(&0);
            let need = *payload.get(4).unwrap_or(&1);
            let have = *payload.get(5).unwrap_or(&0);
            let tail = payload.get(6..).unwrap_or_default();
            if cancelled || selections.is_empty() {
                let dmg_source = if next == target { source } else { target };
                let amount = crate::query::sha_damage_for(tx.state(), dmg_source, 1);
                return super::super::play::finish_juedou_with_tail(
                    tx, source, target, next, amount, tail,
                );
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("play one 杀".into()));
            }
            if let Some(skill) = super::super::skill::source_free_response_skill(
                tx.state(),
                next,
                CardKind::Sha,
                selections[0],
            ) {
                return super::super::skill::begin_source_free_response(
                    tx,
                    next,
                    skill,
                    CardKind::Sha,
                    3,
                    &payload,
                );
            }
            if let Some(skill) = super::super::skill::multi_card_response_skill(
                tx.state(),
                next,
                CardKind::Sha,
                selections[0],
            ) {
                return super::super::skill::begin_multi_card_response(
                    tx,
                    next,
                    skill,
                    CardKind::Sha,
                    3,
                    &payload,
                );
            }
            let cid = CardId::new(selections[0]);
            let legal = super::super::play::response_sha_tokens(tx.state(), next);
            if !legal.contains(&cid.raw()) {
                return Err(EffectError::Illegal("not a legal 杀".into()));
            }
            let from = response_card_zone(tx, next, cid)?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
            let resume = super::super::play::juedou_payload_after_response(
                source, target, next, exchanges, need, have, tail,
            );
            run_physical_response_effects(
                tx,
                vec![
                    EffectRequest::MoveCardsWithReason {
                        cards: vec![cid],
                        to: discard,
                        from: Some(from),
                        reason: CardMoveReason::Respond,
                    },
                    EffectRequest::CardResponded {
                        player: next,
                        respond_to: source,
                        card: cid,
                        response_kind: CardKind::Sha.tag(),
                    },
                    EffectRequest::EmitRuleEvent {
                        code: 124,
                        payload: vec![source.raw(), target.raw(), next.raw(), cid.raw()],
                    },
                ],
                ContinuationKind::JuedouDuel,
                resume.clone(),
                next,
                cid,
                CardKind::Sha,
            )?;
            super::super::play::continue_juedou_payload(tx, &resume)
        }

        ContinuationKind::JiedaoChoice => {
            // payload: [actor, holder, victim]; selection 0=give weapon, 1=sha
            let actor = PlayerId::new(*payload.first().unwrap_or(&0));
            let holder = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let victim = PlayerId::new(*payload.get(2).unwrap_or(&0));
            if cancelled || selections.is_empty() {
                return Err(EffectError::Illegal("jiedao must choose".into()));
            }
            let pick = selections[0];
            if pick == 1 {
                // Holder uses 杀 on victim (with 闪 response)
                return super::super::play::begin_sha_response(
                    tx, holder, victim, 1, 0, 0, true, None,
                );
            }
            jiedao_give_weapon(tx, actor, holder)
        }

        ContinuationKind::WuxieWindow => {
            // payload: [nullified, ask_idx, ask_len, p0.., effect_tag, actor, respond_to,
            // response_kind, response_suit, ...]
            let ask_idx = *payload.get(1).unwrap_or(&0) as usize;
            let ask_len = *payload.get(2).unwrap_or(&0) as usize;
            let observer = if ask_len > 0 && 3 + ask_idx < payload.len() {
                PlayerId::new(payload[3 + ask_idx])
            } else {
                PlayerId::NONE
            };
            if cancelled || selections.is_empty() {
                // Pass → next asker
                let mut p = payload.clone();
                if p.len() > 1 {
                    p[1] = (ask_idx + 1) as u32;
                }
                tx.state_mut().resolution.continuation_kind = ContinuationKind::WuxieWindow;
                tx.state_mut().resolution.continuation_payload = p;
                return super::super::play::advance_wuxie_window(tx);
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("play one 无懈".into()));
            }
            if let Some(skill) = super::super::skill::source_free_response_skill(
                tx.state(),
                observer,
                CardKind::TrickWuxie,
                selections[0],
            ) {
                return super::super::skill::begin_source_free_response(
                    tx,
                    observer,
                    skill,
                    CardKind::TrickWuxie,
                    4,
                    &payload,
                );
            }
            if let Some(skill) = super::super::skill::multi_card_response_skill(
                tx.state(),
                observer,
                CardKind::TrickWuxie,
                selections[0],
            ) {
                return super::super::skill::begin_multi_card_response(
                    tx,
                    observer,
                    skill,
                    CardKind::TrickWuxie,
                    4,
                    &payload,
                );
            }
            let cid = CardId::new(selections[0]);
            let legal =
                super::super::play::wuxie_tokens_for_payload(tx.state(), observer, &payload);
            if !legal.contains(&cid.raw()) {
                return Err(EffectError::Illegal("not a 无懈 in hand".into()));
            }
            let response_suit = tx
                .state()
                .cards
                .get(cid)
                .map(|card| crate::query::effective_card_suit(tx.state(), observer, card))
                .ok_or_else(|| EffectError::Illegal("bad 无懈 card".into()))?;
            let from = response_card_zone(tx, observer, cid)?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
            // Precompute the next wuxie window so an optional response trigger can
            // pause here without losing the parity/seat continuation.
            let nullified = payload.first().copied().unwrap_or(0) ^ 1;
            let effect_off = 3 + ask_len;
            let effect_tag = *payload.get(effect_off).unwrap_or(&0);
            let actor = PlayerId::new(*payload.get(effect_off + 1).unwrap_or(&0));
            let respond_to = PlayerId::new(*payload.get(effect_off + 2).unwrap_or(&0));
            let effect_args: Vec<u32> = if payload.len() > effect_off + 5 {
                payload[effect_off + 5..].to_vec()
            } else {
                vec![]
            };
            let start = {
                let order = tx.state().players.alive_by_seat();
                let pos = order.iter().position(|p| *p == observer).unwrap_or(0);
                if order.is_empty() {
                    observer
                } else {
                    order[(pos + 1) % order.len()]
                }
            };
            let mut resume_payload = vec![
                start.raw(),
                nullified,
                effect_tag,
                actor.raw(),
                observer.raw(),
                CardKind::TrickWuxie.tag() as u32,
                super::super::judge::suit_tag(response_suit),
            ];
            resume_payload.extend_from_slice(&effect_args);
            let mut effects = vec![EffectRequest::MoveCardsWithReason {
                cards: vec![cid],
                to: discard,
                from: Some(from),
                reason: CardMoveReason::Respond,
            }];
            append_pothaoshi_refills(tx.state(), observer, &[cid], &mut effects);
            effects.push(EffectRequest::CardResponded {
                player: observer,
                respond_to,
                card: cid,
                response_kind: CardKind::TrickWuxie.tag(),
            });
            effects.push(EffectRequest::EmitRuleEvent {
                code: 221, // 无懈 used
                payload: vec![observer.raw(), cid.raw()],
            });
            run_response_effects(tx, effects, ContinuationKind::WuxieWindow, resume_payload)?;
            // Toggle nullify parity and re-open ask from next seat after 无懈 user
            super::super::play::begin_wuxie_window_with_parity(
                tx,
                start,
                nullified,
                effect_tag,
                actor,
                observer,
                CardKind::TrickWuxie,
                response_suit,
                &effect_args,
            )
        }

        ContinuationKind::ActiveSkillSelection => {
            if cancelled {
                return Ok(());
            }
            let skill = engine_api::SkillId::new(*payload.first().unwrap_or(&0));
            let target_count = *payload.get(1).unwrap_or(&0) as usize;
            let targets: Vec<PlayerId> = payload
                .iter()
                .skip(2)
                .take(target_count)
                .copied()
                .map(PlayerId::new)
                .collect();
            let cards: Vec<CardId> = selections.iter().copied().map(CardId::new).collect();
            super::super::skill::handle_activate_skill(tx, observer, skill, &targets, &cards)
        }
        _ => unreachable!("choice routed to a different continuation group"),
    }
}
