//! Effect primitives for `replace_draw_with_distinct_suit_selection` and related helpers.

use super::*;

pub(crate) fn suppress_non_locked_skills_until_turn_end(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().turn.current.is_none()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || tx
            .state()
            .turn
            .non_locked_skill_suppressed_players
            .contains(&player)
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .turn
        .non_locked_skill_suppressed_players
        .push(player);
    tx.state_mut()
        .turn
        .non_locked_skill_suppressed_players
        .sort_by_key(|player| player.raw());
    tx.state_mut().rebuild_derived_indexes();
    Ok(vec![DomainEvent::RuleNote {
        code: 356,
        payload: vec![player.raw()],
    }])
}

pub(crate) fn decode_yuheng_state(blob: &[u8]) -> YuhengState {
    if blob.len() != 16 || &blob[..4] != b"YHG1" {
        return YuhengState::default();
    }
    YuhengState {
        history: u32::from_le_bytes(blob[4..8].try_into().unwrap()),
        full_remaining: u32::from_le_bytes(blob[8..12].try_into().unwrap()),
        current_remaining: u32::from_le_bytes(blob[12..16].try_into().unwrap()),
    }
}

pub(crate) fn encode_yuheng_state(state: YuhengState) -> Vec<u8> {
    let mut blob = b"YHG1".to_vec();
    blob.extend_from_slice(&state.history.to_le_bytes());
    blob.extend_from_slice(&state.full_remaining.to_le_bytes());
    blob.extend_from_slice(&state.current_remaining.to_le_bytes());
    blob
}

pub(crate) fn player_skill_by_def(
    tx: &Transaction,
    player: PlayerId,
    skill_def: u32,
) -> Option<engine_api::SkillId> {
    tx.state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def.raw() == skill_def)
        })
        .copied()
}

pub(crate) fn current_yuheng_mask(tx: &Transaction, player: PlayerId) -> u32 {
    DILI_LINES
        .iter()
        .enumerate()
        .filter(|(_, line)| player_skill_by_def(tx, player, **line).is_some())
        .fold(0, |mask, (index, _)| mask | (0b111 << (index * 3)))
}

pub(crate) fn yuheng_state(
    tx: &Transaction,
    player: PlayerId,
) -> Option<(engine_api::SkillId, YuhengState)> {
    let skill = player_skill_by_def(tx, player, YUHENG_DEF)?;
    Some((
        skill,
        decode_yuheng_state(&tx.state().skills.get(skill)?.state_blob),
    ))
}

pub(crate) fn store_yuheng_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    state: YuhengState,
) -> Result<(), EffectError> {
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("Yuheng skill disappeared".into()))?;
    instance.state_blob = encode_yuheng_state(state);
    Ok(())
}

pub(crate) fn refresh_dili_line_enablement(tx: &mut Transaction, player: PlayerId, history: u32) {
    for (index, line) in DILI_LINES.iter().enumerate() {
        let Some(skill) = player_skill_by_def(tx, player, *line) else {
            continue;
        };
        let group = 0b111 << (index * 3);
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.enabled = history & group == group;
        }
    }
    tx.state_mut().rebuild_derived_indexes();
}

pub(crate) fn take_random_mask_bit(tx: &mut Transaction, mask: u32) -> Option<(usize, u32)> {
    let choices: Vec<_> = (0..YUHENG_POOL.len())
        .filter(|index| mask & (1 << index) != 0)
        .collect();
    if choices.is_empty() {
        return None;
    }
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let choice = choices[rng.gen_range_u32(choices.len() as u32) as usize];
    tx.state_mut().rng = rng.into_state();
    Some((choice, mask & !(1 << choice)))
}

pub(crate) fn add_dili_line(
    tx: &mut Transaction,
    player: PlayerId,
    line_index: usize,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = add_skill(
        tx,
        player,
        engine_api::SkillDefId::new(DILI_LINES[line_index]),
    )?;
    if let Some((yuheng, mut state)) = yuheng_state(tx, player) {
        state.current_remaining |= 0b111 << (line_index * 3);
        store_yuheng_state(tx, yuheng, state)?;
        refresh_dili_line_enablement(tx, player, state.history);
    }
    events.push(DomainEvent::RuleNote {
        code: 240,
        payload: vec![player.raw(), DILI_LINES[line_index]],
    });
    Ok(events)
}

pub(crate) fn open_changandajian_target_choice(
    tx: &mut Transaction,
    player: PlayerId,
    remaining: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if remaining == 0 {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            [ZoneKind::Equip, ZoneKind::Judge].into_iter().any(|kind| {
                tx.state()
                    .zones
                    .find(kind, target)
                    .and_then(|zone| tx.state().zones.get(zone))
                    .is_some_and(|zone| !zone.cards.is_empty())
            })
        })
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseChangandajianTarget,
        vec![player.raw(), remaining, skill.raw()],
    )
}

pub(crate) fn finalize_tiaojiyanmei(
    tx: &mut Transaction,
    actor: PlayerId,
    left: PlayerId,
    right: PlayerId,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive)
        || tx.state().hand_count(left) != tx.state().hand_count(right)
    {
        return Ok(Vec::new());
    }
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card| Some(card.zone) == discard)
    {
        return Ok(Vec::new());
    }
    let targets = tx.state().players.alive_by_seat();
    open_choice(
        tx,
        actor,
        1,
        1,
        true,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTiaojiyanmeiGainTarget,
        vec![actor.raw(), card.raw()],
    )
}

pub(crate) fn begin_tiaojiyanmei_discard(
    tx: &mut Transaction,
    actor: PlayerId,
    left: PlayerId,
    right: PlayerId,
    higher: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), higher);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        higher,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DiscardTiaojiyanmeiCard,
        vec![actor.raw(), left.raw(), right.raw(), higher.raw()],
    )
}

pub(crate) fn resolve_dili_yuheng(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    card: CardId,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(origin) = tx.state().skills.get(skill).cloned() else {
        return Ok(Vec::new());
    };
    if origin.owner != player || !tx.state().skill_effectively_enabled(&origin) {
        return Ok(Vec::new());
    }
    match action {
        0 if origin.def.raw() == DILI_DEF => {
            if DILI_LINES
                .iter()
                .any(|line| player_skill_by_def(tx, player, *line).is_some())
            {
                return Ok(Vec::new());
            }
            let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
            let line = rng.gen_range_u32(DILI_LINES.len() as u32) as usize;
            tx.state_mut().rng = rng.into_state();
            add_dili_line(tx, player, line)
        }
        1 if origin.def.raw() == YUHENG_DEF => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let used_other_non_locked = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|player| player.skills.iter())
                .filter_map(|skill| tx.state().skills.get(*skill))
                .any(|instance| {
                    instance.id != skill
                        && instance.play_phase_use_count > 0
                        && !tx
                            .state()
                            .content
                            .skill_metadata(instance.def)
                            .is_some_and(|metadata| metadata.locked)
                });
            let removable: Vec<_> = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|player| player.skills.iter())
                .filter(|candidate| **candidate != skill)
                .filter(|candidate| {
                    tx.state().skills.get(**candidate).is_some_and(|instance| {
                        !tx.state()
                            .content
                            .skill_metadata(instance.def)
                            .is_some_and(|metadata| metadata.locked)
                    })
                })
                .copied()
                .collect();
            let mut events = Vec::new();
            for candidate in removable {
                events.extend(remove_skill(tx, candidate)?);
            }

            let mut state = decode_yuheng_state(&origin.state_blob);
            let full = if state.full_remaining == 0 {
                YUHENG_ALL_MASK
            } else {
                state.full_remaining
            };
            let current = if state.current_remaining == 0 {
                current_yuheng_mask(tx, player)
            } else {
                state.current_remaining
            };
            let (full_index, full_remaining) = take_random_mask_bit(tx, full)
                .ok_or_else(|| EffectError::EngineFault("Yuheng full pool is empty".into()))?;
            let mut selected = vec![full_index];
            let mut current_remaining = current;
            if !used_other_non_locked && current != 0 {
                let distinct = current & !(1 << full_index);
                let selection_mask = if distinct == 0 { current } else { distinct };
                if let Some((current_index, _)) = take_random_mask_bit(tx, selection_mask) {
                    selected.push(current_index);
                    current_remaining &= !(1 << current_index);
                }
            }
            for index in selected {
                let selected_def = YUHENG_POOL[index];
                state.history |= 1 << index;
                events.extend(add_skill(
                    tx,
                    player,
                    engine_api::SkillDefId::new(selected_def),
                )?);
            }
            state.full_remaining = full_remaining;
            state.current_remaining = current_remaining;
            store_yuheng_state(tx, skill, state)?;
            refresh_dili_line_enablement(tx, player, state.history);

            let qionglan = player_skill_by_def(tx, player, DILI_LINES[2]);
            if state.history & (0b111 << 6) == 0b111 << 6
                && qionglan.is_some_and(|helper| {
                    tx.state()
                        .skills
                        .get(helper)
                        .is_some_and(|instance| instance.lifetime_fire_count == 0)
                })
            {
                let mut candidates = vec![0usize, 1, 3, 4, 5];
                let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
                rng.shuffle(&mut candidates);
                if candidates[..2].contains(&3) && candidates[..2].contains(&4) {
                    candidates.swap(1, 2);
                }
                tx.state_mut().rng = rng.into_state();
                for line in candidates.into_iter().take(2) {
                    events.extend(add_dili_line(tx, player, line)?);
                }
                if let Some(helper) = qionglan {
                    tx.state_mut()
                        .skills
                        .get_mut(helper)
                        .expect("Qionglan helper exists")
                        .lifetime_fire_count = 1;
                }
                refresh_dili_line_enablement(tx, player, state.history);
            }
            Ok(events)
        }
        2 if origin.def.raw() == YUHENG_DEF => {
            if origin.play_phase_use_count != 0 {
                return Ok(Vec::new());
            }
            deal_damage(
                tx,
                PlayerId::NONE,
                player,
                1,
                true,
                0,
                false,
                CardId::NONE,
                0,
                0,
            )
        }
        3 if origin.def.raw() == DILI_LINES[0] => {
            let Some((_, state)) = yuheng_state(tx, player) else {
                return Ok(Vec::new());
            };
            let rank = tx.state().cards.get(card).map_or(0, |card| card.rank);
            if state.history & 0b111 != 0b111 || !matches!(rank, 2 | 3 | 5 | 7 | 11 | 13) {
                return Ok(Vec::new());
            }
            add_card_use_policy_skill(tx, player, card, 1 << 1, origin.def)
        }
        4 if origin.def.raw() == DILI_LINES[1] => {
            let Some((_, state)) = yuheng_state(tx, player) else {
                return Ok(Vec::new());
            };
            if state.history & (0b111 << 3) != 0b111 << 3
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Judge
            {
                return Ok(Vec::new());
            }
            let events = skip_next_judge_phase(tx, player)?;
            tx.state_mut()
                .turn
                .set_flag(turn_flag::PRE_NORMAL_EXTRA_DRAW, 1);
            Ok(events)
        }
        5 if origin.def.raw() == DILI_LINES[5] => {
            let Some((_, state)) = yuheng_state(tx, player) else {
                return Ok(Vec::new());
            };
            let Some(source) = tx.state().cards.get(card).cloned() else {
                return Ok(Vec::new());
            };
            let Some(slot) = source.kind.equip_slot() else {
                return Ok(Vec::new());
            };
            if state.history & (0b111 << 15) != 0b111 << 15
                || source.owner != player
                || !tx
                    .state()
                    .zones
                    .get(source.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Equip)
                || CHANGANDAJIAN_DEFS.contains(&source.def.raw())
            {
                return Ok(Vec::new());
            }
            let ship_def = engine_api::CardDefId::new(CHANGANDAJIAN_DEFS[slot.index()]);
            let metadata = tx
                .state()
                .content
                .card_metadata(ship_def)
                .cloned()
                .ok_or_else(|| EffectError::ContentFault("Changandajian card missing".into()))?;
            let kind = match metadata.kind.as_str() {
                "weapon" => CardKind::Weapon,
                "armor" => CardKind::Armor,
                "horse_def" => CardKind::HorseDef,
                "horse_off" => CardKind::HorseOff,
                "treasure" => CardKind::Treasure,
                _ => {
                    return Err(EffectError::ContentFault(
                        "Changandajian card kind is invalid".into(),
                    ));
                }
            };
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            let equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .ok_or_else(|| EffectError::EngineFault("equipment zone missing".into()))?;
            let mut events = move_cards(tx, vec![card], discard, Some(source.zone))?;
            let spade = origin.state_blob.first().copied().unwrap_or(0) != 0;
            let ship = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: ship_def,
                kind,
                suit: if spade { Suit::Spade } else { Suit::Heart },
                rank: 10,
                weapon_range: metadata.weapon_range,
                zone: processing,
                owner: PlayerId::NONE,
                face_up: true,
            });
            tx.state_mut()
                .zones
                .get_mut(processing)
                .expect("processing zone exists")
                .cards
                .push(ship);
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("Yuanlv skill exists")
                .state_blob = vec![u8::from(!spade)];
            events.extend(move_cards(tx, vec![ship], equip, Some(processing))?);
            Ok(events)
        }
        128 if origin.def.raw() == CHANGANDAJIAN_DESTROY_DEF => {
            let ships: Vec<_> = event_cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .and_then(|card| tx.state().content.card_metadata(card.def))
                        .is_some_and(|card| {
                            card.on_leave_equip_skill_def == CHANGANDAJIAN_DESTROY_DEF
                        })
                })
                .collect();
            if ships.is_empty() {
                return Ok(Vec::new());
            }
            let recover = ships
                .iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(**card)
                        .is_some_and(|card| card.def.raw() == CHANGANDAJIAN_EQUIP2_DEF)
                })
                .count()
                .min(u32::MAX as usize) as u32;
            let removed = tx
                .state()
                .zones
                .find(ZoneKind::Removed, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("removed zone missing".into()))?;
            let mut events = Vec::new();
            for card in &ships {
                let from = tx
                    .state()
                    .cards
                    .get(*card)
                    .map(|card| card.zone)
                    .ok_or_else(|| EffectError::EngineFault("ship card missing".into()))?;
                events.extend(move_cards(tx, vec![*card], removed, Some(from))?);
            }
            events.extend(recover_hp(tx, player, player, recover)?);
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDiliYuheng {
                    player,
                    action: 129,
                    card: CardId::NONE,
                    event_cards: ships,
                    skill,
                },
            );
            Ok(events)
        }
        129 if origin.def.raw() == CHANGANDAJIAN_DESTROY_DEF => open_changandajian_target_choice(
            tx,
            player,
            event_cards.len().min(u32::MAX as usize) as u32,
            skill,
        ),
        _ => Ok(Vec::new()),
    }
}

pub(crate) fn add_source_turn_bound_skill(
    tx: &mut Transaction,
    target: PlayerId,
    skill_def: engine_api::SkillDefId,
    source: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let events = add_skill_with_state(tx, target, skill_def, None)?;
    let skill = tx
        .state()
        .players
        .get(target)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .find(|skill| {
            tx.state()
                .skills
                .get(**skill)
                .is_some_and(|instance| instance.def == skill_def)
        })
        .copied()
        .ok_or_else(|| EffectError::EngineFault("source-turn-bound helper was not added".into()))?;
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| instance.add_source_turn_bound(source))
    {
        return Err(EffectError::EngineFault(
            "source-turn-bound helper source could not be recorded".into(),
        ));
    }
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_qixing_weather(
    tx: &mut Transaction,
    player: PlayerId,
    qixing_skill_def: engine_api::SkillDefId,
    helper_skill_def: engine_api::SkillDefId,
    max_targets: u8,
    stage: u8,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if max_targets == 0
        || qixing_skill_def.is_none()
        || helper_skill_def.is_none()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill) {
            return Ok(Vec::new());
        }
        let cards = tx.state().skill_pile_cards(player, qixing_skill_def.raw());
        let max = cards
            .len()
            .min(tx.state().players.alive_by_seat().len())
            .min(usize::from(max_targets))
            .min(u8::MAX as usize) as u8;
        if max == 0 {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            max,
            true,
            cards.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseQixingWeatherCards,
            vec![
                player.raw(),
                skill.raw(),
                qixing_skill_def.raw(),
                helper_skill_def.raw(),
                u32::from(max_targets),
            ],
        );
    }
    if stage != 1 || cards.len() != targets.len() || cards.is_empty() {
        return Err(EffectError::Illegal("invalid Qixing weather state".into()));
    }
    let mut unique_targets = std::collections::BTreeSet::new();
    if !targets.iter().all(|&target| {
        unique_targets.insert(target)
            && tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
    }) {
        return Err(EffectError::Illegal(
            "Qixing weather targets are no longer legal".into(),
        ));
    }
    let mut events = Vec::new();
    for target in targets {
        events.extend(add_source_turn_bound_skill(
            tx,
            target,
            helper_skill_def,
            player,
        )?);
    }
    Ok(events)
}
