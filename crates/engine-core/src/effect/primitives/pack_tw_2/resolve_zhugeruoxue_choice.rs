//! Effect primitives for `resolve_zhugeruoxue_choice` and related helpers.

use super::*;

pub(crate) fn resolve_zhugeruoxue_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload.first() != Some(&RESOLVE_ZHUGERUOXUE_CHOICE) {
        return Err(EffectError::EngineFault(
            "truncated Zhugeruoxue choice".into(),
        ));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    match stage {
        0 => {
            let skill = engine_api::SkillId::new(payload[3]);
            if observer != player {
                return Err(EffectError::Illegal("Qiongying observer changed".into()));
            }
            if cancelled && selections.is_empty() {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.use_count = instance.use_count.saturating_sub(1);
                    instance.play_phase_use_count = instance.play_phase_use_count.saturating_sub(1);
                }
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Qiongying requires one field card".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            if !zhugeruoxue_movable_field_cards(tx).contains(&card) {
                return Err(EffectError::Illegal("Qiongying field card changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugeruoxue {
                    player,
                    target: PlayerId::NONE,
                    card,
                    action: 1,
                    remaining: 0,
                    used_mask: 0,
                    repeated: false,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() != 5 || observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Qiongying destination".into()));
            }
            let card = CardId::new(payload[3]);
            let target = PlayerId::new(selections[0]);
            if !crate::query::field_card_move_destinations(tx.state(), card).contains(&target) {
                return Err(EffectError::Illegal("Qiongying destination changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugeruoxue {
                    player,
                    target,
                    card,
                    action: 2,
                    remaining: 0,
                    used_mask: 0,
                    repeated: false,
                    skill: engine_api::SkillId::new(payload[4]),
                }],
            )
            .map(|_| ())
        }
        3 => {
            if payload.len() != 5 || observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Qiongying discard".into()));
            }
            let card = CardId::new(selections[0]);
            let suit = crate::command::judge::suit_from_tag(payload[3]);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card)
                || !tx.state().cards.get(card).is_some_and(|state| {
                    Some(state.zone) == hand
                        && crate::query::effective_card_suit(tx.state(), player, state) == suit
                })
            {
                return Err(EffectError::Illegal("Qiongying discard changed".into()));
            }
            discard_cards(tx, player, vec![card]).map(|_| ())
        }
        10 => {
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Nuanhui target".into()));
            }
            let target = PlayerId::new(selections[0]);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Nuanhui target changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugeruoxue {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 11,
                    remaining: 0,
                    used_mask: 0,
                    repeated: false,
                    skill: engine_api::SkillId::new(payload[3]),
                }],
            )
            .map(|_| ())
        }
        12 => {
            if payload.len() < 9 {
                return Err(EffectError::EngineFault(
                    "truncated Nuanhui card choice".into(),
                ));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[7]);
            if observer != target {
                return Err(EffectError::Illegal("Nuanhui chooser changed".into()));
            }
            if cancelled && selections.is_empty() {
                return run_effects(
                    tx,
                    vec![EffectRequest::ResolveZhugeruoxue {
                        player,
                        target,
                        card: CardId::NONE,
                        action: 12,
                        remaining: 0,
                        used_mask: payload[5],
                        repeated: payload[6] != 0,
                        skill,
                    }],
                )
                .map(|_| ());
            }
            if selections.len() != 1 || !payload[8..].contains(&selections[0]) {
                return Err(EffectError::Illegal("invalid Nuanhui basic choice".into()));
            }
            if !zhugeruoxue_basic_options(tx, target)?.contains(&selections[0]) {
                return Err(EffectError::Illegal("Nuanhui basic choice changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugeruoxue {
                    player,
                    target,
                    card: CardId::new(selections[0]),
                    action: 13,
                    remaining: payload[4],
                    used_mask: payload[5],
                    repeated: payload[6] != 0,
                    skill,
                }],
            )
            .map(|_| ())
        }
        13 => {
            if payload.len() < 13 || cancelled {
                return Err(EffectError::Illegal("invalid Nuanhui target choice".into()));
            }
            let target = PlayerId::new(payload[3]);
            let card = CardId::new(payload[4]);
            let kind = u8::try_from(payload[5])
                .ok()
                .and_then(CardKind::from_tag)
                .filter(|kind| zhugeruoxue_basic_name_bit(*kind).is_some())
                .ok_or_else(|| EffectError::EngineFault("bad Nuanhui target kind".into()))?;
            let min = usize::try_from(payload[10]).unwrap_or(usize::MAX);
            let max = usize::try_from(payload[11]).unwrap_or(usize::MAX);
            let offered: Vec<_> = payload[12..].iter().copied().map(PlayerId::new).collect();
            if observer != target
                || selections.len() < min
                || selections.len() > max
                || selections
                    .iter()
                    .any(|selection| !payload[12..].contains(selection))
            {
                return Err(EffectError::Illegal(
                    "Nuanhui target selection changed".into(),
                ));
            }
            let crate::command::play::SkillCardTargetSpec::Select {
                min: current_min,
                max: current_max,
                options,
            } = crate::command::play::skill_card_target_spec(tx, target, card)?
            else {
                return Err(EffectError::Illegal("Nuanhui target shape changed".into()));
            };
            let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let mut distinct = std::collections::BTreeSet::new();
            if selected.len() < usize::from(current_min)
                || selected.len() > usize::from(current_max)
                || selected.iter().any(|candidate| {
                    !distinct.insert(*candidate)
                        || !options.contains(candidate)
                        || !offered.contains(candidate)
                })
            {
                return Err(EffectError::Illegal(
                    "Nuanhui targets became illegal".into(),
                ));
            }
            play_zhugeruoxue_virtual_basic(
                tx,
                player,
                target,
                card,
                kind,
                &selected,
                payload[6],
                payload[7],
                payload[8] != 0,
                engine_api::SkillId::new(payload[9]),
            )
        }
        _ => Err(EffectError::EngineFault(
            "unknown Zhugeruoxue choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_twxiawei(
    tx: &mut Transaction,
    player: PlayerId,
    wangxing_skill_def: u32,
    action: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("TW Xiawei skill disappeared".into()))?;
    match action {
        0 => {
            let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
            if let Some(reference) = tx
                .state_mut()
                .skill_piles
                .iter_mut()
                .find(|reference| reference.zone == pile)
            {
                reference.usable_as_hand = true;
            }
            let mut selected = Vec::new();
            for kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
                let Some(zone) = tx.state().zones.find(kind, PlayerId::NONE) else {
                    continue;
                };
                let cards = tx
                    .state()
                    .zones
                    .get(zone)
                    .map(|zone| zone.cards.clone())
                    .unwrap_or_default();
                for card in cards.into_iter().rev() {
                    if selected.len() == 2 {
                        break;
                    }
                    if tx
                        .state()
                        .cards
                        .get(card)
                        .is_some_and(|card| card.kind.is_basic())
                    {
                        selected.push((zone, card));
                    }
                }
                if selected.len() == 2 {
                    break;
                }
            }
            let mut events = Vec::new();
            for from in selected
                .iter()
                .map(|(zone, _)| *zone)
                .collect::<std::collections::BTreeSet<_>>()
            {
                let cards = selected
                    .iter()
                    .filter_map(|(zone, card)| (*zone == from).then_some(*card))
                    .collect();
                events.extend(move_cards_with_reason(
                    tx,
                    cards,
                    pile,
                    Some(from),
                    CardMoveReason::Unspecified,
                )?);
            }
            Ok(events)
        }
        1 => {
            let cards = tx.state().skill_pile_cards(player, skill_def);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("TW Xiawei discard pile missing".into()))?;
            move_cards_with_reason(tx, cards, discard, Some(pile), CardMoveReason::Discard)
        }
        2 if stage == 0 => open_choice(
            tx,
            player,
            0,
            1,
            true,
            vec![1, 2, 3, 4],
            ChoiceKind::Custom,
            ContinuationKind::ResolveTwxiawei,
            vec![2, player.raw(), skill.raw(), wangxing_skill_def],
        ),
        2 if (1..=4).contains(&stage) => {
            let count = u32::from(stage) + 1;
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("TW Xiawei draw pile missing".into()))?;
            while tx
                .state()
                .zones
                .get(draw)
                .map_or(0, |zone| zone.cards.len())
                < count as usize
            {
                let before = tx
                    .state()
                    .zones
                    .get(draw)
                    .map_or(0, |zone| zone.cards.len());
                reshuffle_discard_into_draw(tx)?;
                if tx
                    .state()
                    .zones
                    .get(draw)
                    .map_or(0, |zone| zone.cards.len())
                    == before
                {
                    break;
                }
            }
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .map(|zone| {
                    zone.cards
                        .iter()
                        .rev()
                        .take(count as usize)
                        .copied()
                        .collect()
                })
                .unwrap_or_default();
            let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
            if let Some(reference) = tx
                .state_mut()
                .skill_piles
                .iter_mut()
                .find(|reference| reference.zone == pile)
            {
                reference.usable_as_hand = true;
            }
            let mut events =
                move_cards_with_reason(tx, cards, pile, Some(draw), CardMoveReason::Unspecified)?;
            events.extend(run_effects(
                tx,
                vec![
                    EffectRequest::AddSkill {
                        player,
                        skill_def: engine_api::SkillDefId::new(wangxing_skill_def),
                    },
                    EffectRequest::AddMark {
                        player,
                        mark: engine_api::MarkDefId::new(wangxing_skill_def),
                        delta: i32::from(stage),
                    },
                ],
            )?);
            Ok(events)
        }
        3 => {
            increase_bound_damage_by_one(tx, tx.state().resolution.current_damage_id)?;
            Ok(Vec::new())
        }
        4 => {
            let count = tx
                .state()
                .players
                .get(player)
                .and_then(|participant| {
                    participant
                        .marks
                        .iter()
                        .find(|(mark, _)| *mark == wangxing_skill_def)
                        .map(|(_, count)| (*count).max(0) as usize)
                })
                .unwrap_or(0);
            if count == 0 {
                return remove_skill(tx, skill);
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.len() < count {
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::RemoveSkill { skill },
                        EffectRequest::RemoveMark {
                            player,
                            mark: engine_api::MarkDefId::new(wangxing_skill_def),
                            amount: count as u32,
                        },
                        EffectRequest::ChangeMaxHp { player, delta: -1 },
                    ],
                );
            }
            let mut payload = vec![
                4,
                player.raw(),
                skill.raw(),
                wangxing_skill_def,
                count as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                count as u8,
                count as u8,
                true,
                options.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwxiawei,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Xiawei resolver action/stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twbianfuren(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: u32,
    action: u8,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            let owner_is_max_max_hp = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter_map(|candidate| tx.state().players.get(candidate).map(|p| p.max_hp))
                .max()
                .is_some_and(|max_hp| {
                    tx.state()
                        .players
                        .get(player)
                        .is_some_and(|owner| owner.max_hp == max_hp)
                });
            let mut effects = Vec::new();
            if target != player {
                let damage_id = tx.state().resolution.current_damage_id;
                if damage_id == 0 {
                    return Err(EffectError::ContentFault(
                        "TW Wanwei used outside damage_before".into(),
                    ));
                }
                if !tx
                    .state()
                    .resolution
                    .prevented_damage_ids
                    .contains(&damage_id)
                {
                    tx.state_mut()
                        .resolution
                        .prevented_damage_ids
                        .push(damage_id);
                }
                effects.push(EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                });
            }
            if target == player || owner_is_max_max_hp {
                effects.push(EffectRequest::AddSkillUntilTurnEnd {
                    player,
                    skill_def: engine_api::SkillDefId::new(helper_skill_def),
                });
            }
            run_effects(tx, effects)
        }
        (1, 0) => {
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveTwbianfuren {
                    player,
                    target: PlayerId::NONE,
                    helper_skill_def,
                    action,
                    stage: 1,
                    cards: Vec::new(),
                    skill,
                },
            );
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
        }
        (1, 1) => use_twbianfuren_bottom_card(tx, player, skill),
        (2, 0) => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let hand_count = tx.state().hand_cards(player).len() as i32;
            let hand_limit = match crate::query::query(
                tx.state(),
                &rule_api::QueryRequest::HandLimit { player },
                &mut crate::query::QueryContext::new(),
            ) {
                rule_api::QueryResponse::Int(value) => value.max(0),
                _ => return Err(EffectError::Illegal("hand-limit query failed".into())),
            };
            let max = (hand_count - hand_limit).max(1) as usize;
            let max = options.len().min(max).min(u8::MAX as usize) as u8;
            let mut payload = vec![0, player.raw(), helper_skill_def, skill.raw()];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                max,
                true,
                options.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwbianfuren,
                payload,
            )
        }
        (2, 1) => finish_twyuejian_cards(tx, player, cards, Vec::new(), helper_skill_def),
        _ => Err(EffectError::ContentFault(
            "unknown TW Bianfuren resolver action/stage".into(),
        )),
    }
}

pub(crate) fn use_twbianfuren_bottom_card(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("TW Wanwei draw pile missing".into()))?;
    if tx
        .state()
        .zones
        .get(draw)
        .is_none_or(|zone| zone.cards.is_empty())
    {
        reshuffle_discard_into_draw(tx)?;
    }
    let Some(card) = tx
        .state()
        .zones
        .get(draw)
        .and_then(|zone| zone.cards.first().copied())
    else {
        return Ok(Vec::new());
    };
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("TW Wanwei hand missing".into()))?;
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: vec![card],
        to: Vec::new(),
    }];
    events.extend(move_cards(tx, vec![card], hand, Some(draw))?);
    match crate::command::play::skill_card_target_spec(tx, player, card) {
        Ok(crate::command::play::SkillCardTargetSpec::Immediate(targets)) => {
            events.extend(use_card(tx, player, card, targets)?);
        }
        Ok(crate::command::play::SkillCardTargetSpec::Select { min, max, options }) => {
            let mut payload = vec![
                3,
                player.raw(),
                card.raw(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            events.extend(open_choice(
                tx,
                player,
                min,
                max,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwbianfuren,
                payload,
            )?);
        }
        _ => {
            move_cards(tx, vec![card], draw, Some(hand))?;
            let zone = tx
                .state_mut()
                .zones
                .get_mut(draw)
                .expect("validated draw pile");
            zone.cards.retain(|candidate| *candidate != card);
            zone.cards.insert(0, card);
        }
    }
    Ok(events)
}
