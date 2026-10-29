//! Effect primitives for `choose_other_bizheng` and related helpers.

use super::*;

pub(crate) fn rejieyin_can_equip(tx: &Transaction, target: PlayerId, card: CardId) -> bool {
    tx.state()
        .cards
        .get(card)
        .and_then(|card| card.kind.equip_slot())
        .is_some_and(|slot| {
            tx.state()
                .players
                .get(target)
                .is_some_and(|target| target.alive && target.equip_slot_enabled(slot))
        })
}

pub(crate) fn resolve_rejieyin(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    card: CardId,
    equip: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "Rejieyin selected card became illegal".into(),
                ));
            }
            let source_is_equip = tx.state().cards.get(card).is_some_and(|card| {
                tx.state()
                    .zones
                    .get(card.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Equip)
            });
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    let male = tx
                        .state()
                        .players
                        .get(target)
                        .and_then(|target| tx.state().content.character(target.character_def))
                        .is_some_and(|character| {
                            matches!(character.sex.as_str(), "male" | "double")
                        });
                    male && (!source_is_equip || rejieyin_can_equip(tx, target, card))
                })
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveRejieyin,
                vec![0, player.raw(), card.raw(), skill.raw()],
            )
        }
        1 => {
            let source_is_equip = tx.state().cards.get(card).is_some_and(|card| {
                tx.state()
                    .zones
                    .get(card.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Equip)
            });
            if source_is_equip {
                return resolve_rejieyin(tx, player, target, 2, card, true, skill);
            }
            if rejieyin_can_equip(tx, target, card) {
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Custom,
                    ContinuationKind::ResolveRejieyin,
                    vec![1, player.raw(), target.raw(), card.raw(), skill.raw()],
                );
            }
            resolve_rejieyin(tx, player, target, 2, card, false, skill)
        }
        2 => {
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveRejieyin {
                    player,
                    target,
                    stage: 3,
                    card: CardId::NONE,
                    equip: false,
                    skill,
                },
            );
            if !equip {
                return discard_cards(tx, player, vec![card]);
            }
            if !rejieyin_can_equip(tx, target, card) {
                return Err(EffectError::Illegal(
                    "Rejieyin equipment target became illegal".into(),
                ));
            }
            let card_state = tx
                .state()
                .cards
                .get(card)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("missing Rejieyin card".into()))?;
            let slot = card_state.kind.equip_slot().unwrap();
            let target_equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .ok_or_else(|| EffectError::EngineFault("target equip zone missing".into()))?;
            let existing: Vec<_> = tx
                .state()
                .zones
                .get(target_equip)
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .copied()
                .filter(|existing| {
                    tx.state()
                        .cards
                        .get(*existing)
                        .and_then(|card| card.kind.equip_slot())
                        == Some(slot)
                })
                .collect();
            let mut events = Vec::new();
            if !existing.is_empty() {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                events.extend(move_cards(tx, existing, discard, Some(target_equip))?);
            }
            events.extend(move_cards(
                tx,
                vec![card],
                target_equip,
                Some(card_state.zone),
            )?);
            Ok(events)
        }
        3 => {
            let (drawer, recovering) = match (
                tx.state().players.get(player).map(|player| player.hp),
                tx.state().players.get(target).map(|target| target.hp),
            ) {
                (Some(left), Some(right)) if left > right => (player, target),
                (Some(left), Some(right)) if left < right => (target, player),
                _ => return Ok(Vec::new()),
            };
            if tx
                .state()
                .players
                .get(recovering)
                .is_some_and(|participant| participant.hp < participant.max_hp as i32)
            {
                tx.state_mut().resolution.deferred_effects.insert(
                    0,
                    EffectRequest::RecoverHp {
                        source: drawer,
                        player: recovering,
                        amount: 1,
                    },
                );
            }
            draw_cards(tx, drawer, 1)
        }
        _ => Err(EffectError::ContentFault("invalid Rejieyin stage".into())),
    }
}

pub(crate) fn reanxu_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(player);
    if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
        cards.extend(
            tx.state()
                .zones
                .get(equip)
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .copied(),
        );
    }
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_reanxu(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    first: PlayerId,
    second: PlayerId,
    card: CardId,
    from_hand: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let alive = tx.state().players.alive_by_seat();
            if alive.len() <= 2 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = alive
                .into_iter()
                .filter(|&target| target != player && !reanxu_cards(tx, target).is_empty())
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveReanxu,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 if second.is_none() => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player && target != first)
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() || reanxu_cards(tx, first).is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveReanxu,
                vec![1, player.raw(), first.raw(), skill.raw()],
            )
        }
        1 => {
            if player == first
                || player == second
                || first == second
                || ![first, second].into_iter().all(|target| {
                    tx.state()
                        .players
                        .get(target)
                        .is_some_and(|participant| participant.alive)
                })
            {
                return Err(EffectError::Illegal(
                    "Reanxu target pair became illegal".into(),
                ));
            }
            let options = reanxu_cards(tx, first);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                second,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveReanxu,
                vec![2, player.raw(), first.raw(), second.raw(), skill.raw()],
            )
        }
        2 => {
            let card_state = tx
                .state()
                .cards
                .get(card)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("missing Reanxu card".into()))?;
            if card_state.owner != first || !reanxu_cards(tx, first).contains(&card) {
                return Err(EffectError::Illegal(
                    "Reanxu selected card became illegal".into(),
                ));
            }
            let from_hand = tx
                .state()
                .zones
                .get(card_state.zone)
                .is_some_and(|zone| zone.kind == ZoneKind::Hand);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, second)
                .ok_or_else(|| EffectError::EngineFault("Reanxu target hand missing".into()))?;
            tx.state_mut().resolution.deferred_effects.insert(
                0,
                EffectRequest::ResolveReanxu {
                    player,
                    stage: 3,
                    first,
                    second,
                    card: CardId::NONE,
                    from_hand,
                    skill,
                },
            );
            move_cards(tx, vec![card], hand, Some(card_state.zone))
        }
        3 => {
            if from_hand {
                tx.state_mut().resolution.deferred_effects.insert(
                    0,
                    EffectRequest::ResolveReanxu {
                        player,
                        stage: 4,
                        first,
                        second,
                        card: CardId::NONE,
                        from_hand: false,
                        skill,
                    },
                );
                return draw_cards(tx, player, 1);
            }
            resolve_reanxu(tx, player, 4, first, second, CardId::NONE, false, skill)
        }
        4 => {
            if ![first, second].into_iter().all(|target| {
                tx.state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            }) {
                return Ok(Vec::new());
            }
            let first_hand = tx.state().hand_count(first);
            let second_hand = tx.state().hand_count(second);
            if first_hand == second_hand {
                return Ok(Vec::new());
            }
            let lower = if first_hand < second_hand {
                first
            } else {
                second
            };
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Confirm,
                ContinuationKind::ResolveReanxu,
                vec![3, player.raw(), lower.raw(), skill.raw()],
            )
        }
        _ => Err(EffectError::ContentFault("invalid Reanxu stage".into())),
    }
}

pub(crate) fn resolve_oldanxu(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    giver: PlayerId,
    receiver: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let alive = tx.state().players.alive_by_seat();
            let options = alive
                .iter()
                .copied()
                .filter(|giver| {
                    *giver != player
                        && alive.iter().copied().any(|receiver| {
                            receiver != player
                                && receiver != *giver
                                && tx.state().hand_count(receiver) < tx.state().hand_count(*giver)
                        })
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldAnxu,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    target != player
                        && target != giver
                        && tx.state().hand_count(target) < tx.state().hand_count(giver)
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldAnxu,
                vec![1, player.raw(), giver.raw(), skill.raw()],
            )
        }
        2 => {
            if player == giver
                || player == receiver
                || giver == receiver
                || tx.state().hand_count(giver) <= tx.state().hand_count(receiver)
            {
                return Err(EffectError::Illegal(
                    "old Anxu target pair became illegal".into(),
                ));
            }
            let options = tx.state().hand_cards(giver);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                receiver,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveOldAnxu,
                vec![2, player.raw(), giver.raw(), receiver.raw(), skill.raw()],
            )
        }
        3 => {
            let card_state = tx
                .state()
                .cards
                .get(card)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("missing old Anxu card".into()))?;
            let giver_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, giver)
                .ok_or_else(|| EffectError::EngineFault("old Anxu giver hand missing".into()))?;
            if card_state.owner != giver
                || card_state.zone != giver_hand
                || tx.state().hand_count(giver) <= tx.state().hand_count(receiver)
            {
                return Err(EffectError::Illegal(
                    "old Anxu selected card became illegal".into(),
                ));
            }
            let receiver_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, receiver)
                .ok_or_else(|| EffectError::EngineFault("old Anxu receiver hand missing".into()))?;
            if card_state.suit != Suit::Spade {
                tx.state_mut()
                    .resolution
                    .deferred_effects
                    .insert(0, EffectRequest::DrawCards { player, count: 1 });
            }
            move_cards(tx, vec![card], receiver_hand, Some(giver_hand))
        }
        _ => Err(EffectError::ContentFault("invalid old Anxu stage".into())),
    }
}

pub(crate) fn resolve_oldqinqing(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    lord: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let Some(lord) = tx
                .state()
                .players
                .iter_live()
                .find(|participant| participant.alive && participant.identity == Identity::Lord)
                .map(|participant| participant.id)
            else {
                return Ok(Vec::new());
            };
            let alive = tx.state().players.alive_by_seat();
            if !alive.iter().copied().any(|candidate| {
                candidate != lord
                    && candidate != player
                    && crate::query::player_in_attack_range(tx.state(), candidate, lord)
            }) {
                return Ok(Vec::new());
            }
            let options = alive
                .into_iter()
                .filter(|candidate| {
                    *candidate != player
                        && crate::query::player_in_attack_range(tx.state(), *candidate, lord)
                })
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldQinqing,
                vec![player.raw(), lord.raw(), skill.raw()],
            )
        }
        1 => {
            if ![player, target, lord].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|participant| participant.alive)
            }) || target == player
                || !crate::query::player_in_attack_range(tx.state(), target, lord)
            {
                return Err(EffectError::Illegal(
                    "old Qinqing target became illegal".into(),
                ));
            }
            let mut effects = Vec::new();
            if !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquip,
            )
            .is_empty()
            {
                effects.push(EffectRequest::ChooseDiscardTargetCard {
                    player,
                    target,
                    scope: TargetCardScope::HandEquip,
                });
            }
            effects.push(EffectRequest::DrawCards {
                player: target,
                count: 1,
            });
            effects.push(EffectRequest::ResolveOldQinqing {
                player,
                target,
                lord,
                stage: 2,
                skill,
            });
            run_effects(tx, effects)
        }
        2 => {
            if tx
                .state()
                .players
                .get(lord)
                .is_some_and(|participant| participant.alive)
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                && tx.state().hand_count(target) > tx.state().hand_count(lord)
            {
                draw_cards(tx, player, 1)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Err(EffectError::ContentFault(
            "invalid old Qinqing stage".into(),
        )),
    }
}

pub(crate) fn resolve_oldmingjian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || tx.state().hand_count(player) == 0
            {
                return Ok(Vec::new());
            }
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| target != player)
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveOldMingjian,
                vec![player.raw(), skill.raw()],
            )
        }
        1 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || tx.state().hand_count(player) == 0
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "old Mingjian target became illegal".into(),
                ));
            }
            if tx.state().turn.extra_play_phase_queue.len() >= 8 {
                return Err(EffectError::ContentFault(
                    "extra Play phase queue exceeds the eight-segment bound".into(),
                ));
            }
            tx.state_mut().turn.extra_play_phase_queue.push(target);
            run_effects(
                tx,
                vec![
                    EffectRequest::GiveAllHand { player, target },
                    EffectRequest::AdvanceTurn,
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "invalid old Mingjian stage".into(),
        )),
    }
}
