//! Effect primitives for `resolve_zhugeruoxue_choice` and related helpers.

use super::*;

pub(crate) fn finish_twyuejian_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    top: Vec<CardId>,
    helper_skill_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    if selected.len() != cards.len() {
        return Err(EffectError::Illegal(
            "duplicate TW Yuejian selected card".into(),
        ));
    }
    let mut seen_top = std::collections::BTreeSet::new();
    if top
        .iter()
        .any(|card| !selected.contains(card) || !seen_top.insert(*card))
    {
        return Err(EffectError::Illegal("invalid TW Yuejian top cards".into()));
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    if cards.iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_none_or(|state| state.owner != player || ![hand, equip].contains(&Some(state.zone)))
    }) {
        return Err(EffectError::Illegal(
            "TW Yuejian selected card moved".into(),
        ));
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("TW Yuejian draw pile missing".into()))?;
    let mut events = Vec::new();
    for from in [hand, equip].into_iter().flatten() {
        let group: Vec<_> = cards
            .iter()
            .copied()
            .filter(|card| tx.state().cards.get(*card).is_some_and(|c| c.zone == from))
            .collect();
        if !group.is_empty() {
            events.extend(move_cards(tx, group, draw, Some(from))?);
        }
    }
    let top_set: std::collections::BTreeSet<_> = top.iter().copied().collect();
    let bottom: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| !top_set.contains(card))
        .collect();
    let zone = tx
        .state_mut()
        .zones
        .get_mut(draw)
        .expect("validated draw pile");
    zone.cards.retain(|card| !selected.contains(card));
    zone.cards.splice(0..0, bottom);
    zone.cards.extend(top);
    let mut effects = Vec::new();
    if cards.len() >= 3 {
        effects.push(EffectRequest::ChangeMaxHp { player, delta: 1 });
    }
    if cards.len() >= 2 {
        effects.push(EffectRequest::RecoverHp {
            source: player,
            player,
            amount: 1,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player,
        skill_def: engine_api::SkillDefId::new(helper_skill_def),
    });
    effects.push(EffectRequest::AddMark {
        player,
        mark: engine_api::MarkDefId::new(helper_skill_def),
        delta: 1,
    });
    events.extend(run_effects(tx, effects)?);
    Ok(events)
}

pub(crate) fn resolve_twfeiyi(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    stage: u8,
    _card: CardId,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            if first_shared_pile_card_by_kind(tx, CardKind::TrickTiaojiyanmei)?.is_some() {
                return gain_card_by_kind(tx, player, CardKind::TrickTiaojiyanmei.tag());
            }
            let used: Vec<_> = tx
                .state()
                .cards
                .sorted_live()
                .into_iter()
                .filter(|card| card.kind == CardKind::TrickTiaojiyanmei)
                .map(|card| card.suit)
                .collect();
            let missing: Vec<_> = [
                engine_api::Suit::Spade,
                engine_api::Suit::Heart,
                engine_api::Suit::Club,
                engine_api::Suit::Diamond,
            ]
            .into_iter()
            .filter(|suit| !used.contains(suit))
            .collect();
            let Some(suit) = (!missing.is_empty()).then(|| {
                let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
                let suit = missing[rng.gen_range_u32(missing.len() as u32) as usize];
                tx.state_mut().rng = rng.into_state();
                suit
            }) else {
                return Ok(Vec::new());
            };
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("TW Shengxi hand missing".into()))?;
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::NONE,
                kind: CardKind::TrickTiaojiyanmei,
                suit,
                rank: 6,
                weapon_range: 0,
                zone: hand,
                owner: player,
                face_up: false,
            });
            tx.state_mut()
                .zones
                .get_mut(hand)
                .expect("validated hand")
                .cards
                .push(card);
            Ok(vec![DomainEvent::CardsMoved {
                cards: vec![card],
                from: ZoneId::NONE,
                to: hand,
                reason: engine_api::CardMoveReason::Unspecified,
            }])
        }
        (1, 0) => open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![
                CardKind::TrickWuzhong.tag().into(),
                CardKind::TrickGuohe.tag().into(),
                CardKind::TrickWuxie.tag().into(),
            ],
            ChoiceKind::Custom,
            ContinuationKind::ResolveTwfeiyi,
            vec![1, player.raw(), skill.raw()],
        ),
        (2, 0) => {
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("TW Kuanji discard pile missing".into()))?;
            let mut cards: Vec<_> = event_cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.zone == discard)
                })
                .collect();
            cards.sort_unstable_by_key(|card| card.raw());
            cards.dedup();
            if cards.is_empty()
                || !tx
                    .state()
                    .players
                    .iter_live()
                    .any(|target| target.alive && target.id != player)
            {
                return Ok(Vec::new());
            }
            let mut payload = vec![2, player.raw(), skill.raw()];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                cards.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwfeiyi,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Feiyi resolver action/stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twchenzhen(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    helper_skill_def: u32,
    record_skill_def: u32,
    action: u8,
    stage: u8,
    card_kind: u8,
    card: CardId,
    branch: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => open_choice(
            tx,
            player,
            1,
            1,
            false,
            twmuyue_card_kinds()
                .into_iter()
                .map(|kind| kind.tag().into())
                .collect(),
            ChoiceKind::Custom,
            ContinuationKind::ResolveTwchenzhen,
            vec![0, player.raw(), helper_skill_def, skill.raw()],
        ),
        (0, 1) => {
            let has_helper = player_has_skill_def(
                tx,
                player,
                engine_api::SkillDefId::new(helper_skill_def),
                true,
            );
            if has_helper {
                run_effects(
                    tx,
                    vec![EffectRequest::RemoveSkillByDef {
                        player,
                        skill_def: engine_api::SkillDefId::new(helper_skill_def),
                    }],
                )?;
                return open_twmuyue_target(tx, player, card_kind, false, helper_skill_def, skill);
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                1,
                player.raw(),
                helper_skill_def,
                skill.raw(),
                card_kind.into(),
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwchenzhen,
                payload,
            )
        }
        (0, 2) => {
            let declared = CardKind::from_tag(card_kind)
                .ok_or_else(|| EffectError::Illegal("bad TW Muyue card kind".into()))?;
            let selected = tx
                .state()
                .cards
                .get(card)
                .ok_or_else(|| EffectError::Illegal("TW Muyue discard card missing".into()))?;
            let matches =
                crate::query::effective_card_kind(tx.state(), player, selected) == declared;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveTwchenzhen {
                    player,
                    target: PlayerId::NONE,
                    helper_skill_def,
                    record_skill_def: 0,
                    action: 0,
                    stage: 3,
                    card_kind,
                    card: CardId::NONE,
                    branch: u8::from(matches),
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                }],
            )
        }
        (0, 3) => open_twmuyue_target(tx, player, card_kind, branch != 0, helper_skill_def, skill),
        (1, 0) => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.iter().map(|target| target.raw()).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwchenzhen,
                vec![
                    10,
                    player.raw(),
                    helper_skill_def,
                    record_skill_def,
                    skill.raw(),
                ],
            )
        }
        (2, 0) => {
            let count = tx
                .state()
                .players
                .get(player)
                .and_then(|participant| {
                    participant
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == helper_skill_def)
                        .map(|(_, count)| (*count).max(0) as usize)
                })
                .unwrap_or(0);
            let mut effects = vec![
                EffectRequest::RemoveSkill { skill },
                EffectRequest::RemoveMark {
                    player,
                    mark: engine_api::MarkDefId::new(helper_skill_def),
                    amount: count as u32,
                },
            ];
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let need = count.min(options.len());
            if need == 0 {
                return run_effects(tx, effects);
            }
            if options.len() <= need {
                effects.push(EffectRequest::DiscardCards {
                    player,
                    cards: options,
                });
                return run_effects(tx, effects);
            }
            let mut payload = vec![12, player.raw(), skill.raw(), helper_skill_def, need as u32];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                need as u8,
                need as u8,
                false,
                options.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwchenzhen,
                payload,
            )
        }
        (3, 0) => {
            let Some((previous_branch, hand_count)) = tx
                .state()
                .skills
                .get(skill)
                .and_then(|skill| skill.twchayi_state())
            else {
                return remove_skill(tx, skill);
            };
            let changed = tx.state().hand_count(player) != hand_count;
            let mut events = Vec::new();
            if changed {
                if previous_branch == 0 {
                    events.extend(run_effects(
                        tx,
                        vec![
                            EffectRequest::AddSkill {
                                player,
                                skill_def: engine_api::SkillDefId::new(helper_skill_def),
                            },
                            EffectRequest::AddMark {
                                player,
                                mark: engine_api::MarkDefId::new(helper_skill_def),
                                delta: 1,
                            },
                        ],
                    )?);
                } else {
                    let cards = tx.state().hand_cards(player);
                    if !cards.is_empty() {
                        events.push(DomainEvent::CardsRevealed {
                            cards,
                            to: Vec::new(),
                        });
                    }
                }
            }
            events.extend(remove_skill(tx, skill)?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Chenzhen resolver action/stage".into(),
        )),
    }
}

pub(crate) fn twmuyue_card_kinds() -> Vec<CardKind> {
    (1..=38)
        .filter_map(CardKind::from_tag)
        .filter(|kind| kind.is_basic() || kind.is_trick())
        .collect()
}

pub(crate) fn resolve_twqianju(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::EngineFault("TW Qianju equip zone missing".into()))?;
    for zone_kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let from = tx
            .state()
            .zones
            .find(zone_kind, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("shared card zone missing".into()))?;
        let card = tx.state().zones.get(from).and_then(|zone| {
            zone.cards.iter().rev().copied().find(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .and_then(|card| card.kind.equip_slot())
                    .is_some_and(|slot| {
                        tx.state()
                            .players
                            .get(player)
                            .is_some_and(|owner| owner.equip_slot_enabled(slot))
                            && tx.state().zones.get(equip).is_some_and(|zone| {
                                zone.cards.iter().all(|existing| {
                                    tx.state()
                                        .cards
                                        .get(*existing)
                                        .and_then(|card| card.kind.equip_slot())
                                        != Some(slot)
                                })
                            })
                    })
            })
        });
        if let Some(card) = card {
            return move_cards(tx, vec![card], equip, Some(from));
        }
    }
    Ok(Vec::new())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twqingxi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    count: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    crate::command::play::validate_saved_sha_target(tx, player, card, target)?;
    if stage == 0 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let target_equips =
                crate::query::discardable_equipment_cards(tx.state(), target, target);
            if target_equips.is_empty() {
                return resolve_twqingxi(tx, player, target, card, 1, 0, Vec::new(), skill);
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwqingxi,
                vec![0, player.raw(), target.raw(), card.raw(), skill.raw()],
            )
        }
        1 => {
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(1, |zone| zone.cards.len().max(1)) as u32;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwqingxi {
                    player,
                    target,
                    card,
                    stage: 4,
                    count: 0,
                    cards: Vec::new(),
                    skill,
                });
            draw_cards(tx, player, draw)
        }
        2 => {
            let target_equips =
                crate::query::discardable_equipment_cards(tx.state(), target, target);
            let discarded = target_equips.len().min(u8::MAX as usize) as u8;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwqingxi {
                    player,
                    target,
                    card,
                    stage: 3,
                    count: discarded,
                    cards: Vec::new(),
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: target_equips,
                }],
            )
        }
        3 => {
            let source_equips =
                crate::query::discardable_equipment_cards(tx.state(), target, player);
            let need = usize::from(count).min(source_equips.len());
            if need == 0 {
                return resolve_twqingxi(tx, player, target, card, 5, count, Vec::new(), skill);
            }
            if source_equips.len() <= need {
                return resolve_twqingxi(tx, player, target, card, 6, count, source_equips, skill);
            }
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                card.raw(),
                skill.raw(),
                count.into(),
            ];
            payload.extend(source_equips.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                need as u8,
                need as u8,
                false,
                source_equips.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwqingxi,
                payload,
            )
        }
        4 => make_current_sha_target_unrespondable(tx, player, card, target),
        5 => increase_current_sha_target_damage(tx, player, card, target, 1),
        6 => {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwqingxi {
                    player,
                    target,
                    card,
                    stage: 5,
                    count,
                    cards: Vec::new(),
                    skill,
                });
            run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])
        }
        _ => Err(EffectError::ContentFault("unknown TW Qingxi stage".into())),
    }
}

pub(crate) fn resolve_twzaoli(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    mut cards: Vec<CardId>,
    slots: Vec<u8>,
    count: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 0 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let discardable = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let mandatory: Vec<_> = discardable
                .iter()
                .copied()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        crate::query::effective_card_kind(tx.state(), player, card).is_equip()
                    })
                })
                .collect();
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let optional: Vec<_> = discardable
                .iter()
                .copied()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == hand
                            && !crate::query::effective_card_kind(tx.state(), player, card_state)
                                .is_equip()
                    })
                })
                .collect();
            if mandatory.is_empty() && optional.is_empty() {
                return Ok(Vec::new());
            }
            if optional.is_empty() {
                return resolve_twzaoli(tx, player, 1, mandatory, Vec::new(), 0, skill);
            }
            let mut payload = vec![player.raw(), skill.raw(), mandatory.len() as u32];
            payload.extend(mandatory.iter().map(|card| card.raw()));
            payload.extend(optional.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                optional.len().min(u8::MAX as usize) as u8,
                false,
                optional.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwzaoli,
                payload,
            )
        }
        1 => {
            cards.sort_unstable_by_key(|card| card.raw());
            cards.dedup();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let mut equip_slots: Vec<_> = cards
                .iter()
                .filter_map(|card| tx.state().cards.get(*card))
                .filter_map(|card| card.kind.equip_slot())
                .map(|slot| slot as u8)
                .collect();
            equip_slots.sort_unstable();
            equip_slots.dedup();
            let discarded = cards.len().min(u8::MAX as usize) as u8;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwzaoli {
                    player,
                    stage: 2,
                    cards: Vec::new(),
                    slots: equip_slots,
                    count: discarded,
                    skill,
                });
            run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])
        }
        2 => {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwzaoli {
                    player,
                    stage: 3,
                    cards: Vec::new(),
                    slots,
                    count: 0,
                    skill,
                });
            draw_cards(tx, player, u32::from(count))
        }
        3 => {
            let equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .ok_or_else(|| EffectError::EngineFault("TW Zaoli equip zone missing".into()))?;
            let mut events = Vec::new();
            let mut equipped = 0u8;
            for tag in slots {
                let Some(slot) = EquipSlot::from_tag(tag) else {
                    continue;
                };
                let mut found = None;
                for zone_kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
                    let from = tx
                        .state()
                        .zones
                        .find(zone_kind, PlayerId::NONE)
                        .ok_or_else(|| {
                            EffectError::EngineFault("shared card zone missing".into())
                        })?;
                    if let Some(card) = tx.state().zones.get(from).and_then(|zone| {
                        zone.cards.iter().rev().copied().find(|card| {
                            tx.state()
                                .cards
                                .get(*card)
                                .and_then(|card| card.kind.equip_slot())
                                == Some(slot)
                        })
                    }) {
                        found = Some((from, card));
                        break;
                    }
                }
                if let Some((from, card)) = found {
                    events.extend(move_cards(tx, vec![card], equip, Some(from))?);
                    equipped = equipped.saturating_add(1);
                }
            }
            if equipped > 2 {
                events.extend(run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )?);
            }
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown TW Zaoli stage".into())),
    }
}
