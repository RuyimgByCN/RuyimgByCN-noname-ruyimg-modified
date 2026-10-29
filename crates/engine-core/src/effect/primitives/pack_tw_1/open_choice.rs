//! Effect primitives for `finish_turn_begin` and related helpers.

use super::*;

/// Public helper to open a typed choice with continuation already set.
pub fn open_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    min_count: u8,
    max_count: u8,
    allow_cancel: bool,
    option_tokens: Vec<u32>,
    kind: ChoiceKind,
    cont: ContinuationKind,
    payload: Vec<u32>,
) -> Result<Vec<DomainEvent>, EffectError> {
    {
        let r = &mut tx.state_mut().resolution;
        r.continuation_kind = cont;
        r.continuation_payload = payload;
    }
    request_choice(
        tx,
        observer,
        min_count,
        max_count,
        allow_cancel,
        option_tokens,
        kind,
    )
}

pub(crate) fn resolve_twfupan(
    tx: &mut Transaction,
    player: PlayerId,
    damage_amount: u32,
    card: CardId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    if !alive || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let eligible_targets = |tx: &Transaction| {
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|candidate| *candidate != player)
            .filter(|candidate| {
                tx.state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.twfupan_target_state(*candidate) < 2)
            })
            .collect::<Vec<_>>()
    };
    match stage {
        0 => {
            if damage_amount == 0 {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwfupan {
                    player,
                    damage_amount,
                    card: CardId::NONE,
                    target: PlayerId::NONE,
                    stage: 1,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DrawCardsFromSkill {
                    player,
                    count: damage_amount,
                    skill,
                }],
            )
        }
        1 => {
            let cards: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip]
                .into_iter()
                .filter_map(|kind| tx.state().zones.find(kind, player))
                .filter_map(|zone| tx.state().zones.get(zone))
                .flat_map(|zone| zone.cards.iter().copied())
                .collect();
            if cards.is_empty() || eligible_targets(tx).is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwfupan,
                vec![1, player.raw(), damage_amount, 0, 0, skill.raw()],
            )
        }
        2 => {
            if card.is_none() {
                return Err(EffectError::Illegal("twfupan selected card missing".into()));
            }
            let options = eligible_targets(tx);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwfupan,
                vec![2, player.raw(), damage_amount, card.raw(), 0, skill.raw()],
            )
        }
        3 => {
            let previous = tx
                .state()
                .skills
                .get(skill)
                .map_or(2, |instance| instance.twfupan_target_state(target));
            if previous == 0 {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .expect("validated twfupan skill")
                    .set_twfupan_target_state(target, 1);
                return run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 2,
                        skill,
                    }],
                );
            }
            if previous == 1 {
                return open_choice(
                    tx,
                    player,
                    0,
                    1,
                    true,
                    vec![1],
                    ChoiceKind::Confirm,
                    ContinuationKind::ResolveTwfupan,
                    vec![3, player.raw(), damage_amount, 0, target.raw(), skill.raw()],
                );
            }
            Ok(Vec::new())
        }
        4 => {
            if tx
                .state()
                .skills
                .get(skill)
                .is_none_or(|instance| instance.twfupan_target_state(target) != 1)
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated twfupan skill")
                .set_twfupan_target_state(target, 2);
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("unknown twfupan stage".into())),
    }
}

pub(crate) fn resolve_twjiaohua(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    event_cards: &[CardId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target_is_min_hp = tx
        .state()
        .players
        .get(target)
        .filter(|target| target.alive)
        .is_some_and(|target_state| {
            tx.state()
                .players
                .iter_live()
                .filter(|other| other.alive)
                .all(|other| target_state.hp <= other.hp)
        });
    if !originating_skill_is_enabled(tx, player, skill)
        || target.is_none()
        || (target != player && !target_is_min_hp)
    {
        return Err(EffectError::Illegal("invalid TW Jiaohua context".into()));
    }

    let category = |card: &crate::state::Card| {
        if card.kind.is_basic() {
            0b001
        } else if card.kind.is_trick() {
            0b010
        } else if card.kind.is_equip() {
            0b100
        } else {
            0
        }
    };
    let drawn_mask = event_cards.iter().fold(0u8, |mask, card| {
        mask | tx.state().cards.get(*card).map_or(0, category)
    });
    let key = crate::state::turn_flag::twjiaohua_category_by_player(target);
    let gained_mask = tx.state().turn.flag(key) as u8;
    let available = !(drawn_mask | gained_mask) & 0b111;
    if available == 0 {
        return Err(EffectError::Illegal(
            "TW Jiaohua has no available category".into(),
        ));
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
    let candidate = [draw, discard].into_iter().find_map(|zone| {
        tx.state().zones.get(zone).and_then(|zone_state| {
            zone_state.cards.iter().rev().copied().find(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| category(card) & available != 0)
            })
        })
    });
    let Some(card) = candidate else {
        return Ok(Vec::new());
    };
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("TW Jiaohua card disappeared".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("TW Jiaohua target hand missing".into()))?;
    let gained_category = tx
        .state()
        .cards
        .get(card)
        .map(category)
        .ok_or_else(|| EffectError::Illegal("TW Jiaohua card disappeared".into()))?;
    tx.state_mut()
        .turn
        .set_flag(key, i32::from(gained_mask | gained_category));
    move_cards(tx, vec![card], hand, Some(from))
}

pub(crate) fn resolve_twkujian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: &[CardId],
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("invalid TW Kujian context".into()));
    }
    match action {
        0 => {
            if !(1..=3).contains(&cards.len()) {
                return Err(EffectError::Illegal(
                    "TW Kujian requires one to three cards".into(),
                ));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("TW Kujian owner hand missing".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("TW Kujian target hand missing".into()))?;
            let mut selected = cards.to_vec();
            selected.sort_unstable_by_key(|card| card.raw());
            selected.dedup();
            if selected.len() != cards.len()
                || selected.iter().any(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_none_or(|card| card.zone != hand)
                })
            {
                return Err(EffectError::Illegal("invalid TW Kujian gift cards".into()));
            }
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Kujian skill")
                .add_twkujian_cards(&selected)
            {
                return Err(EffectError::ContentFault(
                    "TW Kujian state is occupied".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: selected,
                    to: target_hand,
                    from: hand,
                }],
            )
        }
        1 | 2 => {
            let tracked = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Kujian skill")
                .take_twkujian_cards(cards);
            if tracked.is_empty() {
                return Ok(Vec::new());
            }
            let count = tracked.len() as u32;
            if action == 1 {
                return run_effects(
                    tx,
                    vec![EffectRequest::DrawEachBySeat {
                        first: player,
                        second: target,
                        count,
                        skill,
                    }],
                );
            }
            let seats = tx.state().players.alive_by_seat();
            let start = seats
                .iter()
                .position(|candidate| *candidate == tx.state().turn.current)
                .unwrap_or(0);
            let mut participants = [player, target];
            participants.sort_by_key(|candidate| {
                seats
                    .iter()
                    .position(|seated| seated == candidate)
                    .map_or(usize::MAX, |index| {
                        (index + seats.len() - start) % seats.len()
                    })
            });
            let mut effects = Vec::with_capacity(tracked.len() * 2);
            for _ in tracked {
                effects.extend(participants.iter().map(|participant| {
                    EffectRequest::ChooseDiscardOneHandOrEquip {
                        player: *participant,
                    }
                }));
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("unknown TW Kujian action".into())),
    }
}

pub(crate) fn resolve_twruilian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    event_cards: &[CardId],
    action: u8,
    category: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let round = tx.state().turn.round;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Ruilian skill")
                .set_twruilian_prompt_round(round);
            let targets = tx.state().players.alive_by_seat();
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwruilian,
                vec![0, player.raw(), 0, skill.raw(), 0],
            )
        }
        1 => {
            let mask = event_cards.iter().fold(0, |mask, card| {
                mask | tx.state().cards.get(*card).map_or(0, |card| {
                    if card.kind.is_basic() {
                        1
                    } else if card.kind.is_trick() {
                        2
                    } else if card.kind.is_equip() {
                        4
                    } else {
                        0
                    }
                })
            });
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Ruilian skill")
                .record_twruilian_discard(target, event_cards.len() as u32, mask);
            Ok(Vec::new())
        }
        2 => {
            let Some((count, mask)) = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Ruilian skill")
                .take_twruilian_summary(target)
            else {
                return Ok(Vec::new());
            };
            if count < 2 || mask == 0 {
                return Ok(Vec::new());
            }
            let options: Vec<u32> = [(1, 1), (2, 2), (3, 4)]
                .into_iter()
                .filter_map(|(token, bit)| (mask & bit != 0).then_some(token))
                .collect();
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwruilian,
                vec![1, player.raw(), target.raw(), skill.raw(), u32::from(mask)],
            )
        }
        3 => {
            let matches = |kind: CardKind| match category {
                1 => kind.is_basic(),
                2 => kind.is_trick(),
                3 => kind.is_equip(),
                _ => false,
            };
            if !matches!(category, 1..=3) {
                return Err(EffectError::ContentFault(
                    "unknown TW Ruilian category".into(),
                ));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(discard)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev())
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| matches(card.kind))
                })
                .take(2)
                .collect();
            let effects = [target, player]
                .into_iter()
                .zip(cards)
                .filter_map(|(recipient, card)| {
                    tx.state()
                        .zones
                        .find(ZoneKind::Hand, recipient)
                        .map(|hand| EffectRequest::GiveCards {
                            cards: vec![card],
                            to: hand,
                            from: discard,
                        })
                })
                .collect();
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Ruilian action".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twjianwei(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    target: PlayerId,
    _card: CardId,
    winner: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let can_compare = |tx: &Transaction, left: PlayerId, right: PlayerId| {
        left != right
            && tx
                .state()
                .players
                .get(left)
                .is_some_and(|participant| participant.alive)
            && tx
                .state()
                .players
                .get(right)
                .is_some_and(|participant| participant.alive)
            && !tx.state().hand_cards(left).is_empty()
            && !tx.state().hand_cards(right).is_empty()
    };
    match stage {
        0 => {
            if actor != player || physical_equipped_weapons(tx, player).is_empty() {
                return Ok(Vec::new());
            }
            let range = match crate::query::query(
                tx.state(),
                &QueryRequest::AttackRange { player },
                &mut crate::query::QueryContext::new(),
            ) {
                QueryResponse::Int(value) => value,
                _ => 0,
            };
            let options: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| can_compare(tx, player, *target))
                .filter(|target| {
                    matches!(
                        crate::query::query(
                            tx.state(),
                            &QueryRequest::Distance {
                                from: player,
                                to: *target,
                            },
                            &mut crate::query::QueryContext::new(),
                        ),
                        QueryResponse::Int(distance) if distance <= range
                    )
                })
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwjianwei,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if actor == player || !can_compare(tx, actor, player) {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                actor,
                0,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwjianwei,
                vec![1, player.raw(), actor.raw(), skill.raw()],
            )
        }
        2 => {
            if !can_compare(tx, actor, target) {
                return Ok(Vec::new());
            }
            let options = tx.state().hand_cards(actor);
            open_choice(
                tx,
                actor,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwjianwei,
                vec![2, player.raw(), actor.raw(), target.raw(), skill.raw()],
            )
        }
        3 => {
            let comparison = tx
                .state()
                .skills
                .get(skill)
                .and_then(|skill| skill.twjianwei_compare());
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Jianwei skill")
                .clear_twjianwei_compare();
            let Some((actor, target)) = comparison else {
                return Ok(Vec::new());
            };
            if winner == player {
                let loser = if player == actor { target } else { actor };
                let options = crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    loser,
                    TargetCardScope::HandEquipJudge,
                );
                let zone_count = [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                    .into_iter()
                    .filter(|kind| {
                        tx.state().zones.find(*kind, loser).is_some_and(|zone| {
                            options.iter().any(|card| {
                                tx.state()
                                    .cards
                                    .get(*card)
                                    .is_some_and(|card| card.zone == zone)
                            })
                        })
                    })
                    .count();
                if zone_count == 0 {
                    return Ok(Vec::new());
                }
                return open_choice(
                    tx,
                    player,
                    zone_count as u8,
                    zone_count as u8,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::ResolveTwjianwei,
                    vec![3, player.raw(), loser.raw(), skill.raw()],
                );
            }
            if winner.is_none() || player != actor && player != target {
                return Ok(Vec::new());
            }
            let weapons = physical_equipped_weapons(tx, player);
            if weapons.is_empty() {
                return Ok(Vec::new());
            }
            let from = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, winner)
                .ok_or_else(|| EffectError::EngineFault("winner hand missing".into()))?;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: weapons,
                    to,
                    from,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("unknown TW Jianwei stage".into())),
    }
}
