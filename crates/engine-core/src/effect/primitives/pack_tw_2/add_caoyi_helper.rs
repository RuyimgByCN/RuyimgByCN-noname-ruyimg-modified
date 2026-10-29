//! Effect primitives for `ddd_bailei_damage` and related helpers.

use super::*;

pub(crate) fn add_caoyi_helper(
    tx: &mut Transaction,
    target: PlayerId,
    source: PlayerId,
    mode: u8,
    helper_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let existing = tx.state().players.get(target).and_then(|player| {
        player.skills.iter().copied().find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def == helper_def)
        })
    });
    let (skill, events) = match existing {
        Some(skill) => (skill, Vec::new()),
        None => grant_skill_instance(tx, target, helper_def, b"MYI1".to_vec())?,
    };
    let current = tx.state().turn.current;
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::EngineFault("Miyi helper disappeared".into()))?;
    if caoyi_helper_entries(&instance.state_blob).is_none() {
        return Err(EffectError::ContentFault(
            "invalid Miyi helper state".into(),
        ));
    }
    instance
        .state_blob
        .extend_from_slice(&source.raw().to_le_bytes());
    instance.state_blob.push(mode);
    instance.temporary_until_turn_end_of = current;
    Ok(events)
}

pub(crate) fn resolve_caoyi(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    action: u8,
    event_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Prepare
                || helper_skill_def.is_none()
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_CAOYI_CHOICE,
                    0,
                    player.raw(),
                    helper_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) || card.raw() > 1 {
                return Ok(Vec::new());
            }
            let alive = tx.state().players.alive_by_seat();
            let selected: std::collections::BTreeSet<_> = event_targets.iter().copied().collect();
            if selected.is_empty()
                || selected.len() != event_targets.len()
                || !selected.iter().all(|target| alive.contains(target))
            {
                return Err(EffectError::Illegal("stale Miyi targets".into()));
            }
            let mut events = Vec::new();
            let mut effects = Vec::new();
            for target in alive.into_iter().filter(|target| selected.contains(target)) {
                events.extend(add_caoyi_helper(
                    tx,
                    target,
                    player,
                    card.raw() as u8,
                    helper_skill_def,
                )?);
                effects.push(if card.raw() == 0 {
                    EffectRequest::RecoverHp {
                        source: player,
                        player: target,
                        amount: 1,
                    }
                } else {
                    EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }
                });
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .extend(effects);
            Ok(events)
        }
        10 => {
            let Some(instance) = tx.state().skills.get(skill).cloned() else {
                return Ok(Vec::new());
            };
            if instance.owner != player
                || instance.def != helper_skill_def
                || tx.state().turn.phase != Phase::End
            {
                return Ok(Vec::new());
            }
            let entries = caoyi_helper_entries(&instance.state_blob)
                .ok_or_else(|| EffectError::ContentFault("invalid Miyi helper state".into()))?;
            let events = remove_skill(tx, skill)?;
            if tx
                .state()
                .players
                .get(player)
                .is_some_and(|target| target.alive)
            {
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .extend(entries.into_iter().map(|(source, initial)| {
                        if initial == 0 {
                            EffectRequest::DealDamage {
                                source,
                                target: player,
                                amount: 1,
                                is_lose_hp: false,
                                nature: rule_api::damage_nature::NORMAL,
                            }
                        } else {
                            EffectRequest::RecoverHp {
                                source,
                                player,
                                amount: 1,
                            }
                        }
                    }));
            }
            Ok(events)
        }
        11 => {
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            if instance.owner == player && instance.def == helper_skill_def {
                remove_skill(tx, skill)
            } else {
                Ok(Vec::new())
            }
        }
        20 => {
            if !originating_skill_is_enabled(tx, player, skill) || event_targets.len() != 1 {
                return Ok(Vec::new());
            }
            let target = event_targets[0];
            if !crate::command::play::source_free_sha_can_target_without_distance(
                tx.state(),
                player,
                target,
            ) {
                return Ok(Vec::new());
            }
            let virtual_card = allocate_virtual_card_kind(tx, CardKind::Sha)?;
            tx.state_mut().resolution.register_sha_resolution(
                player,
                virtual_card,
                &[target],
                false,
            );
            if !tx
                .state_mut()
                .resolution
                .mark_sha_damage_source_none(player, virtual_card)
            {
                return Err(EffectError::EngineFault(
                    "Yinjun Sha resolution missing".into(),
                ));
            }
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("Yinjun skill disappeared".into()))?;
            instance.state_blob = b"CYJ1".to_vec();
            instance
                .state_blob
                .extend_from_slice(&virtual_card.raw().to_le_bytes());
            let result = crate::command::play::handle_virtual_sha_with_policies_and_bonus(
                tx,
                player,
                virtual_card,
                &[target],
                true,
                false,
                0,
            );
            if result
                .as_ref()
                .is_err_and(|error| *error != EffectError::ChoiceRequested)
            {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.state_blob.clear();
                }
            }
            result.map(|_| Vec::new())
        }
        21 => {
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            if instance.owner != player
                || instance.state_blob.len() != 8
                || !instance.state_blob.starts_with(b"CYJ1")
                || u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()) != card.raw()
            {
                return Ok(Vec::new());
            }
            let hp = tx.state().players.get(player).map_or(0, |owner| owner.hp);
            let instance = tx.state_mut().skills.get_mut(skill).unwrap();
            instance.state_blob.clear();
            instance.turn_fire_count = instance.turn_fire_count.saturating_add(1);
            if instance.turn_fire_count as i32 > hp {
                instance.turn_disabled = true;
                tx.state_mut().rebuild_derived_indexes();
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown Caoyi resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_caoyi_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 || payload.first() != Some(&RESOLVE_CAOYI_CHOICE) {
        return Err(EffectError::EngineFault("truncated Caoyi choice".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    let helper_skill_def = engine_api::SkillDefId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    if observer != player {
        return Err(EffectError::Illegal("Caoyi choice observer changed".into()));
    }
    match stage {
        0 => {
            if cancelled || selections.len() != 1 || selections[0] > 1 {
                return Err(EffectError::Illegal("invalid Miyi branch".into()));
            }
            let targets = tx.state().players.alive_by_seat();
            open_choice(
                tx,
                player,
                1,
                targets.len().min(u8::MAX as usize) as u8,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_CAOYI_CHOICE,
                    1,
                    player.raw(),
                    helper_skill_def.raw(),
                    skill.raw(),
                    selections[0],
                ],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() != 6 {
                return Err(EffectError::EngineFault("truncated Miyi targets".into()));
            }
            if cancelled && selections.is_empty() {
                return Ok(());
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveCaoyi {
                    player,
                    card: CardId::new(payload[5]),
                    helper_skill_def,
                    action: 1,
                    event_targets: selections.iter().copied().map(PlayerId::new).collect(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Caoyi choice stage".into(),
        )),
    }
}

pub(crate) fn zhugeruoxue_movable_field_cards(tx: &Transaction) -> Vec<CardId> {
    let mut cards: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .flat_map(|source| crate::query::movable_field_cards_from(tx.state(), source))
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn allocate_virtual_card_kind(
    tx: &mut Transaction,
    kind: CardKind,
) -> Result<CardId, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .expect("validated processing zone")
        .cards
        .push(card);
    Ok(card)
}

pub(crate) fn zhugeruoxue_basic_options(
    tx: &mut Transaction,
    target: PlayerId,
) -> Result<Vec<u32>, EffectError> {
    let mut options = Vec::new();
    for kind in [
        CardKind::Sha,
        CardKind::Tao,
        CardKind::Jiu,
        CardKind::ShaFire,
        CardKind::ShaThunder,
    ] {
        let card = allocate_virtual_card_kind(tx, kind)?;
        let legal = matches!(
            crate::command::play::skill_card_target_spec(tx, target, card),
            Ok(crate::command::play::SkillCardTargetSpec::Immediate(_))
                | Ok(crate::command::play::SkillCardTargetSpec::Select { .. })
        );
        remove_unused_virtual_card(tx, card)?;
        if legal {
            options.push(u32::from(kind.tag()));
        }
    }
    Ok(options)
}

pub(crate) fn zhugeruoxue_basic_name_bit(kind: CardKind) -> Option<u32> {
    match kind {
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => Some(1),
        CardKind::Tao => Some(2),
        CardKind::Jiu => Some(4),
        _ => None,
    }
}

pub(crate) fn play_zhugeruoxue_virtual_basic(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
    targets: &[PlayerId],
    remaining: u32,
    used_mask: u32,
    repeated: bool,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveZhugeruoxue {
            player,
            target,
            card: CardId::NONE,
            action: 12,
            remaining,
            used_mask,
            repeated,
            skill,
        });
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, i32::from(kind.tag()));
    let result = crate::command::play::handle_skill_card(tx, target, card, targets);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_zhugeruoxue(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    remaining: u32,
    used_mask: u32,
    repeated: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let options = zhugeruoxue_movable_field_cards(tx);
            if options.is_empty() {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.use_count = instance.use_count.saturating_sub(1);
                    instance.play_phase_use_count = instance.play_phase_use_count.saturating_sub(1);
                }
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![RESOLVE_ZHUGERUOXUE_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let destinations = crate::query::field_card_move_destinations(tx.state(), card);
            if destinations.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                destinations.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_ZHUGERUOXUE_CHOICE,
                    1,
                    player.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            if !crate::query::field_card_move_destinations(tx.state(), card).contains(&target) {
                return Err(EffectError::Illegal(
                    "Qiongying field-card destination changed".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Qiongying field card missing".into()))?;
            let zone_kind = tx
                .state()
                .zones
                .get(from)
                .map(|zone| zone.kind)
                .ok_or_else(|| EffectError::Illegal("Qiongying source zone missing".into()))?;
            let to =
                tx.state().zones.find(zone_kind, target).ok_or_else(|| {
                    EffectError::Illegal("Qiongying destination zone missing".into())
                })?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to,
                        from: Some(from),
                    },
                    EffectRequest::ResolveZhugeruoxue {
                        player,
                        target,
                        card,
                        action: 3,
                        remaining: 0,
                        used_mask: 0,
                        repeated: false,
                        skill,
                    },
                ],
            )
        }
        3 => {
            let Some(card_state) = tx.state().cards.get(card) else {
                return Ok(Vec::new());
            };
            let suit = crate::query::effective_card_suit(tx.state(), player, card_state);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .filter(|candidate| {
                    tx.state().cards.get(*candidate).is_some_and(|state| {
                        Some(state.zone) == hand
                            && crate::query::effective_card_suit(tx.state(), player, state) == suit
                    })
                })
                .collect();
            if options.is_empty() {
                let cards = tx.state().hand_cards(player);
                return if cards.is_empty() {
                    Ok(Vec::new())
                } else {
                    Ok(vec![DomainEvent::CardsRevealed {
                        cards,
                        to: Vec::new(),
                    }])
                };
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_ZHUGERUOXUE_CHOICE,
                    3,
                    player.raw(),
                    crate::command::judge::suit_tag(suit),
                    skill.raw(),
                ],
            )
        }
        10 => {
            let targets = tx.state().players.alive_by_seat();
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
                ContinuationKind::Custom,
                vec![RESOLVE_ZHUGERUOXUE_CHOICE, 10, player.raw(), skill.raw()],
            )
        }
        11 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let total = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len())
                .max(1)
                .min(u32::MAX as usize) as u32;
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhugeruoxue {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 12,
                    remaining: total,
                    used_mask: 0,
                    repeated: false,
                    skill,
                }],
            )
        }
        12 => {
            if remaining == 0
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                if !repeated {
                    return Ok(Vec::new());
                }
                let equip = tx.state().zones.find(ZoneKind::Equip, target);
                let cards: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), target)
                    .into_iter()
                    .filter(|card| {
                        tx.state()
                            .cards
                            .get(*card)
                            .is_some_and(|state| Some(state.zone) == equip)
                    })
                    .collect();
                return if cards.is_empty() {
                    Ok(Vec::new())
                } else {
                    discard_cards(tx, target, cards)
                };
            }
            let options = zhugeruoxue_basic_options(tx, target)?;
            if options.is_empty() {
                return resolve_zhugeruoxue(
                    tx,
                    player,
                    target,
                    CardId::NONE,
                    12,
                    0,
                    used_mask,
                    repeated,
                    skill,
                );
            }
            let mut payload = vec![
                RESOLVE_ZHUGERUOXUE_CHOICE,
                12,
                player.raw(),
                target.raw(),
                remaining,
                used_mask,
                u32::from(repeated),
                skill.raw(),
            ];
            payload.extend(options.iter().copied());
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )
        }
        13 => {
            let kind = u8::try_from(card.raw())
                .ok()
                .and_then(CardKind::from_tag)
                .filter(|kind| zhugeruoxue_basic_name_bit(*kind).is_some())
                .ok_or_else(|| EffectError::ContentFault("bad Nuanhui basic kind".into()))?;
            if !zhugeruoxue_basic_options(tx, target)?.contains(&u32::from(kind.tag())) {
                return Err(EffectError::Illegal(
                    "Nuanhui basic kind became illegal".into(),
                ));
            }
            let bit = zhugeruoxue_basic_name_bit(kind).unwrap();
            let repeated = repeated || used_mask & bit != 0;
            let used_mask = used_mask | bit;
            let remaining = remaining.saturating_sub(1);
            let virtual_card = allocate_virtual_card_kind(tx, kind)?;
            match crate::command::play::skill_card_target_spec(tx, target, virtual_card)? {
                crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
                    play_zhugeruoxue_virtual_basic(
                        tx,
                        player,
                        target,
                        virtual_card,
                        kind,
                        &targets,
                        remaining,
                        used_mask,
                        repeated,
                        skill,
                    )?;
                    Ok(Vec::new())
                }
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    let mut payload = vec![
                        RESOLVE_ZHUGERUOXUE_CHOICE,
                        13,
                        player.raw(),
                        target.raw(),
                        virtual_card.raw(),
                        u32::from(kind.tag()),
                        remaining,
                        used_mask,
                        u32::from(repeated),
                        skill.raw(),
                        u32::from(min),
                        u32::from(max),
                    ];
                    payload.extend(options.iter().map(|target| target.raw()));
                    open_choice(
                        tx,
                        target,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        payload,
                    )
                }
                crate::command::play::SkillCardTargetSpec::Pairs(_) => {
                    Err(EffectError::ContentFault(
                        "Nuanhui basic card cannot require paired targets".into(),
                    ))
                }
            }
        }
        _ => Err(EffectError::ContentFault(
            "unknown Zhugeruoxue resolver action".into(),
        )),
    }
}
