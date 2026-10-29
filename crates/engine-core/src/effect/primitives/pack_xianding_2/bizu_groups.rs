//! Effect primitives for `resolve_stdgaobian` and related helpers.

use super::*;

pub(crate) fn bizu_groups(
    instance: &crate::state::SkillInstance,
    round: u32,
) -> Vec<Vec<PlayerId>> {
    if instance.state_blob.len() < 8
        || instance.state_blob.get(..4) != Some(b"BZY1")
        || u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()) != round
    {
        return Vec::new();
    }
    let mut groups = Vec::new();
    let mut offset = 8;
    while let Some(&count) = instance.state_blob.get(offset) {
        offset += 1;
        let bytes = usize::from(count) * 4;
        let Some(slice) = instance.state_blob.get(offset..offset + bytes) else {
            return Vec::new();
        };
        groups.push(
            slice
                .chunks_exact(4)
                .map(|chunk| PlayerId::new(u32::from_le_bytes(chunk.try_into().unwrap())))
                .collect(),
        );
        offset += bytes;
    }
    groups
}

pub(crate) fn record_bizu_group(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    targets: &[PlayerId],
) -> Result<bool, EffectError> {
    let round = tx.state().turn.round;
    let instance = tx
        .state()
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("Bizu skill missing".into()))?;
    let mut canonical = targets.to_vec();
    canonical.sort_by_key(|target| target.raw());
    canonical.dedup();
    if canonical.len() > u8::MAX as usize {
        return Err(EffectError::ContentFault(
            "Bizu target group exceeds choice protocol".into(),
        ));
    }
    let repeated = bizu_groups(instance, round).contains(&canonical);
    let instance = tx.state_mut().skills.get_mut(skill).unwrap();
    if instance.state_blob.get(..4) != Some(b"BZY1")
        || instance.state_blob.get(4..8) != Some(round.to_le_bytes().as_slice())
    {
        instance.state_blob = b"BZY1".to_vec();
        instance.state_blob.extend_from_slice(&round.to_le_bytes());
    }
    instance.state_blob.push(canonical.len() as u8);
    for target in canonical {
        instance
            .state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
    }
    Ok(repeated)
}

pub(crate) fn bianyue_wuxie_targets(tx: &Transaction, player: PlayerId) -> Vec<PlayerId> {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| *target != player && tx.state().hand_count(*target) > 0)
        .collect()
}

pub(crate) fn resolve_bianyue(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if matches!(action, 0 | 10) && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_count(player);
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| tx.state().hand_count(*target) == hand)
                .collect();
            let repeated = record_bizu_group(tx, skill, &targets)?;
            let mut effects: Vec<_> = targets
                .into_iter()
                .map(|player| EffectRequest::DrawCards { player, count: 1 })
                .collect();
            if repeated {
                effects.push(EffectRequest::BanActiveUntilTurnEnd { player, skill });
                effects.push(EffectRequest::RecoverHp {
                    source: player,
                    player,
                    amount: 1,
                });
            }
            run_effects(tx, effects)
        }
        10 => {
            if tx.state().hand_count(player) == 0 {
                return Ok(Vec::new());
            }
            let options = bianyue_wuxie_targets(tx, player);
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
                ContinuationKind::Custom,
                vec![RESOLVE_BIANYUE_CHOICE, player.raw(), skill.raw()],
            )
        }
        11 => {
            if tx.state().hand_count(player) == 0
                || !bianyue_wuxie_targets(tx, player).contains(&target)
            {
                return Err(EffectError::Illegal(
                    "Wuxie hand-swap target is stale".into(),
                ));
            }
            let player_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Wuxie owner hand missing".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("Wuxie target hand missing".into()))?;
            let player_cards = tx.state().hand_cards(player);
            let target_cards = tx.state().hand_cards(target);
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: player_cards,
                        to: target_hand,
                        from: Some(player_hand),
                    },
                    EffectRequest::MoveCards {
                        cards: target_cards,
                        to: player_hand,
                        from: Some(target_hand),
                    },
                    EffectRequest::ResolveBianyue {
                        player,
                        target,
                        action: 12,
                        skill,
                    },
                ],
            )
        }
        12 => {
            let cards: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|value| {
                        let kind = crate::query::effective_card_kind(tx.state(), player, value);
                        kind.is_sha() || kind.is_damage_trick()
                    })
                })
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let hand = tx.state().cards.get(cards[0]).unwrap().zone;
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            let events = move_cards(tx, cards.clone(), draw, Some(hand))?;
            let zone = tx.state_mut().zones.get_mut(draw).unwrap();
            zone.cards.retain(|candidate| !cards.contains(candidate));
            zone.cards.splice(0..0, cards);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown Bianyue action".into())),
    }
}

pub(crate) fn cilv_decode(blob: &[u8]) -> (u8, Vec<(CardId, u8, Vec<CardId>)>) {
    if blob.len() < 5 || blob.get(..4) != Some(CILV_STATE_TAG.as_slice()) {
        return (0, Vec::new());
    }
    let mut records = Vec::new();
    let mut offset = 5usize;
    while offset + 7 <= blob.len() {
        let card = CardId::new(u32::from_le_bytes(
            blob[offset..offset + 4].try_into().unwrap(),
        ));
        let mode = blob[offset + 4];
        let count = u16::from_le_bytes(blob[offset + 5..offset + 7].try_into().unwrap()) as usize;
        offset += 7;
        let Some(end) = offset.checked_add(count.saturating_mul(4)) else {
            return (blob[4] & 0b111, Vec::new());
        };
        if end > blob.len() {
            return (blob[4] & 0b111, Vec::new());
        }
        let cards = blob[offset..end]
            .chunks_exact(4)
            .map(|bytes| CardId::new(u32::from_le_bytes(bytes.try_into().unwrap())))
            .collect();
        records.push((card, mode, cards));
        offset = end;
    }
    if offset != blob.len() {
        return (blob[4] & 0b111, Vec::new());
    }
    (blob[4] & 0b111, records)
}

pub(crate) fn cilv_encode(used: u8, records: &[(CardId, u8, Vec<CardId>)]) -> Vec<u8> {
    let mut blob = CILV_STATE_TAG.to_vec();
    blob.push(used & 0b111);
    for (card, mode, cards) in records {
        blob.extend_from_slice(&card.raw().to_le_bytes());
        blob.push(*mode & 0b110);
        blob.extend_from_slice(&(cards.len().min(u16::MAX as usize) as u16).to_le_bytes());
        for card in cards.iter().take(u16::MAX as usize) {
            blob.extend_from_slice(&card.raw().to_le_bytes());
        }
    }
    blob
}

pub(crate) fn cilv_used_mask(instance: &crate::state::SkillInstance) -> u8 {
    cilv_decode(&instance.state_blob).0
}

pub(crate) fn cilv_tracks_card(
    instance: &crate::state::SkillInstance,
    card: CardId,
    mode: u8,
) -> bool {
    cilv_decode(&instance.state_blob)
        .1
        .iter()
        .any(|(tracked, tracked_mode, _)| *tracked == card && *tracked_mode & mode != 0)
}

pub(crate) fn cilv_apply_choice(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    choice: u8,
    event_cards: Vec<CardId>,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state().skills.get(skill) else {
        return Err(EffectError::Illegal("Cilv skill disappeared".into()));
    };
    if instance.owner != player || choice > 2 {
        return Err(EffectError::Illegal("invalid Cilv choice".into()));
    }
    let (used, _) = cilv_decode(&instance.state_blob);
    let bit = 1u8 << choice;
    if used & bit != 0 {
        return Err(EffectError::Illegal(
            "Cilv choice was already removed".into(),
        ));
    }
    let current = tx
        .state()
        .resolution
        .pending_card_resolutions
        .iter()
        .rev()
        .find(|record| record.card == card && record.targets.contains(&player));
    if current.is_none() {
        return Err(EffectError::Illegal("Cilv trick context changed".into()));
    }
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = cilv_encode(used | bit, &[]);
    if choice == 0 {
        crate::command::play::exclude_saved_card_target(tx, card, player)?;
        return Ok(Vec::new());
    }
    if helper_skill_def.is_none()
        || !tx
            .state()
            .content
            .skill_metadata(helper_skill_def)
            .is_some_and(|metadata| metadata.non_gainable)
    {
        return Err(EffectError::ContentFault(
            "Cilv requires a non-gainable helper".into(),
        ));
    }
    let events = add_skill(tx, player, helper_skill_def)?;
    let helper = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|owner| owner.skills.iter().copied())
        .find(|candidate| {
            tx.state()
                .skills
                .get(*candidate)
                .is_some_and(|instance| instance.def == helper_skill_def)
        })
        .ok_or_else(|| EffectError::EngineFault("Cilv helper was not added".into()))?;
    let (_, mut records) = cilv_decode(&tx.state().skills.get(helper).unwrap().state_blob);
    let mut cards = event_cards;
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    records.push((card, bit, cards));
    tx.state_mut().skills.get_mut(helper).unwrap().state_blob = cilv_encode(0, &records);
    Ok(events)
}

pub(crate) fn reset_player_starting_skills(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participant = tx
        .state()
        .players
        .get(player)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("Tongdao target missing".into()))?;
    let character = tx
        .state()
        .content
        .character(participant.character_def)
        .cloned()
        .ok_or_else(|| EffectError::ContentFault("Tongdao target character missing".into()))?;
    let removable: Vec<_> = participant
        .skills
        .iter()
        .copied()
        .filter(|skill| {
            tx.state().skills.get(*skill).is_some_and(|instance| {
                !tx.state()
                    .content
                    .skill_metadata(instance.def)
                    .is_some_and(|metadata| metadata.non_gainable)
            })
        })
        .collect();
    let removed_defs: std::collections::BTreeSet<_> = removable
        .iter()
        .filter_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .map(|instance| instance.def.raw())
        })
        .collect();
    let piles: Vec<_> = tx
        .state()
        .skill_piles
        .iter()
        .copied()
        .filter(|pile| pile.owner == player && removed_defs.contains(&pile.skill_def))
        .collect();
    let mut events = Vec::new();
    for skill in removable {
        events.extend(remove_skill(tx, skill)?);
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    for pile in piles {
        let cards = tx
            .state()
            .zones
            .get(pile.zone)
            .map(|zone| zone.cards.clone())
            .unwrap_or_default();
        if !cards.is_empty() {
            events.extend(move_cards_with_reason(
                tx,
                cards,
                discard,
                Some(pile.zone),
                CardMoveReason::Discard,
            )?);
        }
        tx.state_mut()
            .skill_piles
            .retain(|candidate| candidate.zone != pile.zone);
    }
    for raw in character.skills {
        let def = engine_api::SkillDefId::new(raw);
        let allowed = tx
            .state()
            .content
            .skill_metadata(def)
            .is_some_and(|metadata| {
                !metadata.non_gainable
                    && (!metadata.lord_only || participant.identity == Identity::Lord)
            });
        if allowed {
            events.extend(add_skill(tx, player, def)?);
        }
    }
    Ok(events)
}

pub(crate) fn resolve_zhupeilan(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    action: u8,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            let used = cilv_used_mask(instance);
            let current = tx
                .state()
                .resolution
                .pending_card_resolutions
                .iter()
                .rev()
                .find(|record| {
                    record.card == card
                        && record.source == target
                        && record.targets.contains(&player)
                        && CardKind::from_tag(record.kind)
                            .is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick())
                });
            if used == 0b111 || current.is_none() {
                return Ok(Vec::new());
            }
            let cards = if event_cards.is_empty() {
                vec![card]
            } else {
                event_cards
            };
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCards {
                        player,
                        count: 3 - used.count_ones(),
                    },
                    EffectRequest::ResolveZhupeilan {
                        player,
                        target,
                        card,
                        helper_skill_def,
                        action: 6,
                        event_cards: cards,
                        skill,
                    },
                ],
            )
        }
        1 => {
            let tracked = tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| cilv_tracks_card(instance, card, 0b010));
            let damage_id = tx.state().resolution.current_damage_id;
            if !tracked || damage_id == 0 {
                return Ok(Vec::new());
            }
            run_effects(tx, vec![EffectRequest::PreventDamage { damage_id }])
        }
        2 => {
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            let (_, records) = cilv_decode(&instance.state_blob);
            let mut gain = Vec::new();
            let mut keep = Vec::new();
            for (tracked, mode, cards) in records {
                if tracked == card {
                    if mode & 0b100 != 0 {
                        gain.extend(cards);
                    }
                } else {
                    keep.push((tracked, mode, cards));
                }
            }
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob = cilv_encode(0, &keep);
            gain.sort_by_key(|card| card.raw());
            gain.dedup();
            let mut events = gain_event_cards(tx, player, &gain)?;
            if keep.is_empty() && tx.state().skills.get(skill).is_some() {
                events.extend(remove_skill(tx, skill)?);
            }
            Ok(events)
        }
        3..=5 => cilv_apply_choice(
            tx,
            player,
            card,
            action - 3,
            event_cards,
            helper_skill_def,
            skill,
        ),
        6 => {
            let Some(owner) = tx.state().players.get(player) else {
                return Ok(Vec::new());
            };
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            let used = cilv_used_mask(instance);
            if owner.max_hp as usize >= tx.state().hand_count(player) as usize || used == 0b111 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = (0..3).filter(|choice| used & (1 << choice) == 0).collect();
            let mut payload = vec![
                RESOLVE_ZHUPEILAN_CHOICE,
                0,
                player.raw(),
                card.raw(),
                skill.raw(),
                helper_skill_def.raw(),
                event_cards.len().min(u16::MAX as usize) as u32,
            ];
            payload.extend(
                event_cards
                    .into_iter()
                    .take(u16::MAX as usize)
                    .map(CardId::raw),
            );
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        10 => {
            let valid = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player
                    && tx.state().skill_effectively_enabled(instance)
                    && instance.lifetime_fire_count == 1
            }) && tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.alive && owner.dying && owner.hp <= 0);
            if !valid {
                return Ok(Vec::new());
            }
            let options = tx.state().players.alive_by_seat();
            let payload = vec![RESOLVE_ZHUPEILAN_CHOICE, 1, player.raw(), skill.raw()];
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        11 => {
            let context_valid = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player
                    && tx.state().skill_effectively_enabled(instance)
                    && instance.lifetime_fire_count == 1
            }) && tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.alive && owner.dying && owner.hp <= 0);
            if !context_valid {
                return Err(EffectError::Illegal("Tongdao context changed".into()));
            }
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Tongdao target is stale".into()));
            }
            let target_hp = tx.state().players.get(target).unwrap().hp;
            let owner_hp = tx
                .state()
                .players
                .get(player)
                .map(|owner| owner.hp)
                .ok_or_else(|| EffectError::Illegal("Tongdao owner missing".into()))?;
            let mut events = reset_player_starting_skills(tx, target)?;
            if target != player && target_hp > owner_hp {
                events.extend(run_effects(
                    tx,
                    vec![EffectRequest::RecoverHp {
                        source: player,
                        player,
                        amount: (target_hp - owner_hp) as u32,
                    }],
                )?);
            }
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown Zhupeilan action".into())),
    }
}

pub(crate) fn dc_sb_zhangxiu_hand_equip_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(player);
    if let Some(zone) = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| tx.state().zones.get(zone))
    {
        cards.extend(zone.cards.iter().copied());
    }
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn dcsbhaoyi_cards(tx: &Transaction) -> Vec<CardId> {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    let mut cards: Vec<_> = tx
        .state()
        .turn
        .entered_discard_cards
        .iter()
        .copied()
        .filter(|card| !tx.state().turn.damage_card_materials.contains(card))
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                Some(card.zone) == discard && (card.kind.is_sha() || card.kind.is_damage_trick())
            })
        })
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards
}
