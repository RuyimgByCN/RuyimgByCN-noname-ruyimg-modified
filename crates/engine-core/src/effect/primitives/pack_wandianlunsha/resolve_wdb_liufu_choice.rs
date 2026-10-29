//! Effect primitives for `resolve_wdb_liufu_choice` and related helpers.

use super::*;

pub(crate) fn resolve_wdb_liufu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 8 || payload.first() != Some(&RESOLVE_WDB_LIUFU_CHOICE) {
        return Err(EffectError::Illegal("invalid Wdb Liufu choice".into()));
    }
    let action = payload[1] as u8;
    let player = PlayerId::new(payload[2]);
    let target = PlayerId::new(payload[3]);
    let card = CardId::new(payload[4]);
    let zhucheng_skill_def = engine_api::SkillDefId::new(payload[5]);
    let skill = engine_api::SkillId::new(payload[6]);
    let required = payload[7] as usize;
    match action {
        1 => {
            if observer != target
                || !originating_skill_is_enabled(tx, player, skill)
                || crate::command::play::validate_saved_sha_target(tx, target, card, player)
                    .is_err()
            {
                return Err(EffectError::Illegal("stale 筑城 Sha choice".into()));
            }
            if cancelled {
                return run_effects(
                    tx,
                    vec![EffectRequest::ExcludeSavedShaTarget {
                        card,
                        target: player,
                    }],
                )
                .map(|_| ());
            }
            let selected: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let distinct: std::collections::BTreeSet<_> = selected.iter().copied().collect();
            let legal: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), target)
                    .into_iter()
                    .collect();
            if selected.len() != required
                || distinct.len() != selected.len()
                || !distinct.is_subset(&legal)
            {
                return Err(EffectError::Illegal("invalid 筑城 discard payment".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selected,
                }],
            )
            .map(|_| ())
        }
        2 => {
            if observer != player
                || !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != target
                || tx.state().turn.phase != Phase::Play
            {
                return Err(EffectError::Illegal("stale 夺气 choice".into()));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            if selections.len() != required || required != 1 {
                return Err(EffectError::Illegal("invalid 夺气 pile selection".into()));
            }
            let selected = CardId::new(selections[0]);
            if !tx
                .state()
                .skill_pile_cards(player, zhucheng_skill_def.raw())
                .contains(&selected)
            {
                return Err(EffectError::Illegal("selected card is not a 筑".into()));
            }
            let from = tx
                .state()
                .cards
                .get(selected)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("selected 筑 is missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("discard pile is missing".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveWdbLiufu {
                    player,
                    target,
                    card: CardId::NONE,
                    zhucheng_skill_def,
                    action: 3,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::MoveCardsWithReason {
                    cards: vec![selected],
                    to: discard,
                    from: Some(from),
                    reason: CardMoveReason::Discard,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "unknown Wdb Liufu choice action".into(),
        )),
    }
}

pub(crate) fn resolve_wdb_liuyan(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let linked = tx.state().players.get(player).is_some_and(|participant| {
                participant
                    .marks
                    .iter()
                    .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
            });
            let selected_hand = tx.state().hand_cards(player).contains(&card);
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || linked
                || !selected_hand
            {
                return Ok(Vec::new());
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![card],
                    },
                    EffectRequest::AddMark {
                        player,
                        mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                        delta: 1,
                    },
                ],
            )
        }
        1 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Prepare {
                return Ok(Vec::new());
            }
            let count =
                crate::query::other_group_count_outside_owner_attack_range(tx.state(), player);
            if count == 0 {
                Ok(Vec::new())
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player,
                        count,
                        skill,
                    }],
                )
            }
        }
        _ => Err(EffectError::ContentFault(
            "unknown Wdb Liuyan resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_wdb_liuzan(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: engine_api::SkillDefId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action == 0 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if helper_skill_def.is_none() {
        return Err(EffectError::ContentFault(
            "Wdb Liuzan requires a helper skill definition".into(),
        ));
    }
    match action {
        0 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    target,
                    TargetCardScope::HandEquip,
                )
                .is_empty()
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveWdbLiuzan {
                    player,
                    target,
                    helper_skill_def,
                    action: 1,
                    skill,
                });
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
        }
        1 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return Ok(Vec::new());
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
                    RESOLVE_WDB_LIUZAN_CHOICE,
                    player.raw(),
                    target.raw(),
                    helper_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let Some(owner) = tx
                .state()
                .players
                .get(player)
                .filter(|participant| participant.alive)
            else {
                return Ok(Vec::new());
            };
            let lost_hp = (owner.max_hp as i32 - owner.hp).max(0) as u32;
            if lost_hp == 0 {
                return Ok(Vec::new());
            }
            choose_optional_targets_draw(
                tx,
                player,
                1,
                1,
                lost_hp.min(u32::from(u8::MAX)) as u8,
                DrawTargetScope::Any,
                skill,
            )
        }
        3 => {
            let Some(owner) = tx
                .state()
                .players
                .get(player)
                .filter(|participant| participant.alive)
            else {
                return Ok(Vec::new());
            };
            let lost_hp = (owner.max_hp as i32 - owner.hp).max(0) as u32;
            if lost_hp == 0 {
                return Ok(Vec::new());
            }
            let existing = owner.skills.iter().copied().find(|candidate| {
                tx.state()
                    .skills
                    .get(*candidate)
                    .is_some_and(|instance| instance.def == helper_skill_def)
            });
            let (helper, events) = match existing {
                Some(helper) => (helper, Vec::new()),
                None => grant_skill_instance(tx, player, helper_skill_def, Vec::new())?,
            };
            let current = tx.state().turn.current;
            let instance = tx
                .state_mut()
                .skills
                .get_mut(helper)
                .ok_or_else(|| EffectError::EngineFault("亢音 helper disappeared".into()))?;
            instance.state_blob = lost_hp.to_le_bytes().to_vec();
            instance.temporary_until_turn_end_of = current;
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "unknown Wdb Liuzan resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_wdb_liuzan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || payload.first() != Some(&RESOLVE_WDB_LIUZAN_CHOICE) {
        return Err(EffectError::Illegal("invalid Wdb Liuzan choice".into()));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let helper_skill_def = engine_api::SkillDefId::new(payload[3]);
    let skill = engine_api::SkillId::new(payload[4]);
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("stale 亢音 discard choice".into()));
    }
    let selected = CardId::new(selections[0]);
    if !crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        TargetCardScope::HandEquip,
    )
    .contains(&selected)
    {
        return Err(EffectError::Illegal("invalid 亢音 target card".into()));
    }
    let is_basic = tx
        .state()
        .cards
        .get(selected)
        .is_some_and(|card| card.kind.is_basic());
    let from = tx
        .state()
        .cards
        .get(selected)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("selected 亢音 card is missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile is missing".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveWdbLiuzan {
            player,
            target,
            helper_skill_def,
            action: if is_basic { 2 } else { 3 },
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![selected],
            to: discard,
            from: Some(from),
            reason: CardMoveReason::Discard,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_wdb_tianyu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action != 0 {
        return Err(EffectError::ContentFault(
            "unknown Wdb Tianyu resolver action".into(),
        ));
    }
    let source = tx
        .state()
        .resolution
        .sha_resolution_records
        .iter()
        .rev()
        .find(|record| record.card == card && record.targets.contains(&target))
        .map_or(PlayerId::NONE, |record| record.source);
    if !originating_skill_is_enabled(tx, player, skill)
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
        || source.is_none()
        || !tx.state().cards.get(card).is_some_and(|incoming| {
            crate::query::effective_card_kind(tx.state(), source, incoming).is_sha()
        })
    {
        return Ok(Vec::new());
    }
    let resolution = &tx.state().resolution;
    let saved_kind = resolution.resume_kind;
    let saved_payload = resolution.resume_payload.clone();
    if source.is_none()
        || source == target
        || !resolution.sha_target_resolving(source, card, target)
        || !matches!(
            saved_kind,
            ContinuationKind::ResumePlayedSha | ContinuationKind::ResumePlayedShaQueue
        )
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind.is_equip())
        })
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let saved = crate::command::play::encode_youdishenru_saved(
        target,
        source,
        card,
        saved_kind,
        &saved_payload,
    );
    let mut payload = vec![
        RESOLVE_WDB_TIANYU_CHOICE,
        player.raw(),
        target.raw(),
        source.raw(),
        card.raw(),
        skill.raw(),
        saved.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend_from_slice(&saved);
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn resolve_wdb_tianyu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let source = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let incoming = CardId::new(*payload.get(4).unwrap_or(&0));
    let len = *payload.get(6).unwrap_or(&u32::MAX) as usize;
    let saved = payload
        .get(7..7usize.saturating_add(len))
        .ok_or_else(|| EffectError::Illegal("bad 诱战 discard choice payload".into()))?;
    if payload.first() != Some(&RESOLVE_WDB_TIANYU_CHOICE)
        || payload.len() != 7 + len
        || observer != player
        || (cancelled && !selections.is_empty())
        || selections.len() > 1
    {
        return Err(EffectError::Illegal("stale 诱战 discard choice".into()));
    }
    if cancelled || selections.is_empty() {
        return Ok(());
    }
    let selected = CardId::new(selections[0]);
    if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&selected) {
        return Err(EffectError::Illegal("invalid 诱战 equipment cost".into()));
    }
    let selected_state = tx
        .state()
        .cards
        .get(selected)
        .ok_or_else(|| EffectError::Illegal("诱战 equipment cost disappeared".into()))?;
    if selected_state.owner != player || !selected_state.kind.is_equip() {
        return Err(EffectError::Illegal(
            "诱战 requires an equipment card".into(),
        ));
    }
    let from = selected_state.zone;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile is missing".into()))?;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ContinueYoudishenru {
            action: 0,
            protected: target,
            source,
            forced_card: incoming,
            saved: saved.to_vec(),
        });
    run_effects(
        tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![selected],
            to: discard,
            from: Some(from),
            reason: CardMoveReason::Discard,
        }],
    )
    .map(|_| ())
}

pub(crate) fn wdb_fuchou_sources(blob: &[u8]) -> Option<Vec<PlayerId>> {
    if !blob.starts_with(b"WFU1") || (blob.len() - 4) % 4 != 0 {
        return None;
    }
    Some(
        blob[4..]
            .chunks_exact(4)
            .map(|raw| PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap())))
            .collect(),
    )
}
