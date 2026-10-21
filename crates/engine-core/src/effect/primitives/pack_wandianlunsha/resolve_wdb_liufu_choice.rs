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

pub(crate) fn resolve_wdb_xizhenxihong(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let owner_alive = tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive);
            let source_alive = tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive);
            let mut probe = tx.state().resolution.clone();
            if !owner_alive || !source_alive || !probe.exclude_saved_sha_target(card, player) {
                return Ok(Vec::new());
            }
            let mut options = tx.state().hand_cards(player);
            if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, player) {
                options.extend(
                    tx.state()
                        .zones
                        .get(equip)
                        .into_iter()
                        .flat_map(|zone| zone.cards.iter().copied()),
                );
            }
            options.sort_by_key(|card| card.raw());
            options.dedup();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_WDB_FUCHOU_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let sources = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| wdb_fuchou_sources(&instance.state_blob))
                .ok_or_else(|| EffectError::ContentFault("invalid 负仇 state".into()))?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("负仇 skill disappeared".into()))?
                .state_blob = b"WFU1".to_vec();
            continue_wdb_fuchou(tx, 0, player, PlayerId::NONE, sources, skill)
        }
        _ => Err(EffectError::ContentFault(
            "unknown 习珍习宏 resolver action".into(),
        )),
    }
}

pub(crate) fn continue_wdb_fuchou(
    tx: &mut Transaction,
    action: u8,
    player: PlayerId,
    target: PlayerId,
    mut remaining: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let Some(next) = remaining
                .iter()
                .position(|source| {
                    tx.state()
                        .players
                        .get(*source)
                        .is_some_and(|participant| participant.alive)
                })
                .map(|index| remaining.remove(index))
            else {
                return Ok(Vec::new());
            };
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ContinueWdbFuchou {
                    action: 1,
                    player,
                    target: next,
                    remaining,
                    skill,
                },
            );
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
        }
        1 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return continue_wdb_fuchou(tx, 0, player, PlayerId::NONE, remaining, skill);
            }
            let options: Vec<_> =
                crate::command::play::prompted_sha_source_cards(tx.state(), player)
                    .into_iter()
                    .filter(|card| {
                        crate::command::play::prompted_sha_targets_unlimited(tx, player, *card)
                            .contains(&target)
                    })
                    .collect();
            if options.is_empty() {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ContinueWdbFuchou {
                        action: 0,
                        player,
                        target: PlayerId::NONE,
                        remaining,
                        skill,
                    },
                );
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                );
            }
            let mut payload = vec![
                RESOLVE_WDB_FUCHOU_CHOICE,
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                remaining.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(remaining.iter().map(|source| source.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 负仇 continuation action".into(),
        )),
    }
}

pub(crate) fn resolve_wdb_fuchou_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_WDB_FUCHOU_CHOICE)
        || payload.len() < 6
        || (cancelled && !selections.is_empty())
        || selections.len() > 1
    {
        return Err(EffectError::Illegal("stale 负仇 choice".into()));
    }
    let stage = payload[1];
    let player = PlayerId::new(payload[2]);
    if observer != player {
        return Err(EffectError::Illegal("负仇 choice observer changed".into()));
    }
    match stage {
        0 => {
            if payload.len() != 6 {
                return Err(EffectError::Illegal("bad 负仇 gift payload".into()));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let target = PlayerId::new(payload[3]);
            let incoming = CardId::new(payload[4]);
            let skill = engine_api::SkillId::new(payload[5]);
            let selected = CardId::new(selections[0]);
            let selected_state = tx
                .state()
                .cards
                .get(selected)
                .cloned()
                .ok_or_else(|| EffectError::Illegal("负仇 gift disappeared".into()))?;
            let owned_zone = [ZoneKind::Hand, ZoneKind::Equip]
                .into_iter()
                .filter_map(|kind| tx.state().zones.find(kind, player))
                .any(|zone| zone == selected_state.zone);
            if !originating_skill_is_enabled(tx, player, skill)
                || !owned_zone
                || selected_state.owner != player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid 负仇 gift".into()));
            }
            crate::command::play::exclude_saved_card_target(tx, incoming, player)?;
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::EngineFault("负仇 skill disappeared".into()))?;
            if instance.state_blob.is_empty() {
                instance.state_blob = b"WFU1".to_vec();
            }
            let sources = wdb_fuchou_sources(&instance.state_blob)
                .ok_or_else(|| EffectError::ContentFault("invalid 负仇 state".into()))?;
            if !sources.contains(&target) {
                instance
                    .state_blob
                    .extend_from_slice(&target.raw().to_le_bytes());
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("负仇 source hand missing".into()))?;
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
        1 => {
            let count = payload[5] as usize;
            if payload.len() != 6 + count {
                return Err(EffectError::Illegal("bad 负仇 settlement payload".into()));
            }
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let remaining: Vec<_> = payload[6..].iter().copied().map(PlayerId::new).collect();
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ContinueWdbFuchou {
                    action: 0,
                    player,
                    target: PlayerId::NONE,
                    remaining,
                    skill,
                },
            );
            if cancelled || selections.is_empty() {
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                )
                .map(|_| ());
            }
            let selected = CardId::new(selections[0]);
            if !crate::command::play::prompted_sha_source_cards(tx.state(), player)
                .contains(&selected)
                || !crate::command::play::prompted_sha_targets_unlimited(tx, player, selected)
                    .contains(&target)
            {
                return Err(EffectError::Illegal("invalid 负仇 Sha".into()));
            }
            crate::command::play::play_prompted_sha_source_unlimited(
                tx,
                player,
                selected,
                &[target],
            )
        }
        _ => Err(EffectError::Illegal("bad 负仇 choice stage".into())),
    }
}

pub(crate) fn resolve_wdb_yangyi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let owned =
                tx.state().cards.get(card).is_some_and(|state| {
                    state.owner == player
                        && tx.state().zones.get(state.zone).is_some_and(|zone| {
                            matches!(zone.kind, ZoneKind::Hand | ZoneKind::Equip)
                        })
                });
            if !owned {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    tx.state()
                        .players
                        .get(*candidate)
                        .is_some_and(|participant| participant.alive && !participant.dying)
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
                ContinuationKind::Custom,
                vec![
                    RESOLVE_WDB_YANGYI_CHOICE,
                    0,
                    player.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive && !participant.dying)
            {
                return Ok(Vec::new());
            }
            let usable = tx.state().cards.get(card).is_some_and(|state| {
                state.owner == target
                    && state.kind.is_equip()
                    && tx
                        .state()
                        .zones
                        .get(state.zone)
                        .is_some_and(|zone| zone.kind == ZoneKind::Hand && zone.owner == target)
                    && state.kind.equip_slot().is_some_and(|slot| {
                        tx.state()
                            .players
                            .get(target)
                            .is_some_and(|participant| participant.equip_slot_enabled(slot))
                    })
                    && !crate::query::card_use_cancelled_for_card(tx.state(), target, card)
            });
            if !usable {
                return Ok(Vec::new());
            }
            let options = tx
                .state()
                .hand_cards(target)
                .into_iter()
                .map(CardId::raw)
                .collect();
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_WDB_YANGYI_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }]),
        3 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive && !participant.dying)
            {
                return Ok(Vec::new());
            }
            use_virtual_card_on_selected_targets(
                tx,
                target,
                CardKind::TrickDiaobingqianjiang.tag(),
                Vec::new(),
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 杨仪 resolver action".into(),
        )),
    }
}

pub(crate) fn current_diaobing_revealed(tx: &Transaction, cards: Vec<CardId>) -> Vec<CardId> {
    cards
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|state| {
                tx.state()
                    .zones
                    .get(state.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Processing)
            })
        })
        .collect()
}

pub(crate) fn continue_diaobingqianjiang(
    tx: &mut Transaction,
    action: u8,
    actor: PlayerId,
    source_card: CardId,
    targets: Vec<PlayerId>,
    revealed: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            let count = tx.state().players.alive_by_seat().len().div_ceil(2);
            if count == 0 {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            if tx
                .state()
                .zones
                .get(draw)
                .map_or(0, |zone| zone.cards.len())
                < count
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev())
                .take(count)
                .copied()
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ContinueDiaobingqianjiang {
                    action: 1,
                    actor,
                    source_card,
                    targets,
                    revealed: cards.clone(),
                },
            );
            let mut events = move_cards(tx, cards.clone(), processing, Some(draw))?;
            events.push(DomainEvent::CardsRevealed {
                cards,
                to: Vec::new(),
            });
            Ok(events)
        }
        1 => {
            let revealed = current_diaobing_revealed(tx, revealed);
            if !targets.is_empty() && !revealed.is_empty() {
                let source =
                    tx.state().cards.get(source_card).ok_or_else(|| {
                        EffectError::EngineFault("调兵遣将 carrier missing".into())
                    })?;
                let kind = crate::query::effective_card_kind(tx.state(), actor, source);
                let suit = crate::query::effective_card_suit(tx.state(), actor, source);
                let mut queue = vec![
                    source_card.raw(),
                    actor.raw(),
                    kind.tag() as u32,
                    crate::command::judge::suit_tag(suit),
                    crate::command::play::trick_effect::DIAOBINGQIANJIANG,
                    0,
                    targets.len().min(u32::MAX as usize) as u32,
                ];
                queue.extend(targets.iter().map(|player| player.raw()));
                queue.push(crate::command::play::DIAOBING_POOL_MAGIC);
                queue.push(revealed.len().min(u32::MAX as usize) as u32);
                queue.extend(revealed.iter().map(|card| card.raw()));
                crate::command::play::advance_independent_trick_queue(tx, &queue)?;
                return Ok(Vec::new());
            }
            if revealed.is_empty() {
                return Ok(Vec::new());
            }
            if !tx
                .state()
                .players
                .get(actor)
                .is_some_and(|participant| participant.alive)
            {
                let processing = tx
                    .state()
                    .zones
                    .find(ZoneKind::Processing, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                return move_cards(tx, revealed, discard, Some(processing));
            }
            let mut payload = vec![
                RESOLVE_WDB_YANGYI_CHOICE,
                4,
                actor.raw(),
                source_card.raw(),
                revealed.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(revealed.iter().map(|card| card.raw()));
            open_choice(
                tx,
                actor,
                1,
                revealed.len().min(u8::MAX as usize) as u8,
                false,
                revealed.iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 调兵遣将 continuation action".into(),
        )),
    }
}

pub(crate) fn diaobing_queue_parts(
    queue: &[u32],
) -> Result<(PlayerId, CardId, PlayerId, Vec<PlayerId>, Vec<CardId>), EffectError> {
    if queue.len() < 9 {
        return Err(EffectError::Illegal("truncated 调兵遣将 queue".into()));
    }
    let actor = PlayerId::new(queue[1]);
    let source_card = CardId::new(queue[0]);
    let index = queue[5] as usize;
    let count = queue[6] as usize;
    let targets_end = 7usize.saturating_add(count);
    if index >= count || queue.get(targets_end) != Some(&crate::command::play::DIAOBING_POOL_MAGIC)
    {
        return Err(EffectError::Illegal("invalid 调兵遣将 queue".into()));
    }
    let revealed_count = *queue.get(targets_end + 1).unwrap_or(&u32::MAX) as usize;
    if queue.len() != targets_end.saturating_add(2).saturating_add(revealed_count) {
        return Err(EffectError::Illegal("invalid 调兵遣将 pool".into()));
    }
    Ok((
        actor,
        source_card,
        PlayerId::new(queue[7 + index]),
        queue[7 + index + 1..targets_end]
            .iter()
            .copied()
            .map(PlayerId::new)
            .collect(),
        queue[targets_end + 2..]
            .iter()
            .copied()
            .map(CardId::new)
            .collect(),
    ))
}
