//! Command operations for `source_free_wuzhong_has_legal_use` and related helpers.

use super::*;

/// Whether a living actor may use the source-free, self-resolving Wuzhong needed by 颂谳.
pub(crate) fn source_free_wuzhong_has_legal_use(
    state: &crate::state::GameState,
    actor: PlayerId,
) -> bool {
    let Some(player) = state.players.get(actor) else {
        return false;
    };
    let Some(hand) = state.zones.find(ZoneKind::Hand, actor) else {
        return false;
    };
    if !player.alive || player.dying {
        return false;
    }
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::TrickWuzhong,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    !crate::query::card_use_cancelled_for_logical_kind(state, actor, &card, CardKind::TrickWuzhong)
}

/// Use one suitless virtual Duel on the active skill's already-authoritative target list.
///
/// The active target contract replaces Duel's ordinary one-target cap; all other card-use,
/// target, Wuxie, and per-target Duel rules remain on the ordinary resolution path.
pub(crate) fn play_source_free_multi_target_juedou(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
    {
        return Err(EffectError::Illegal(
            "source-free card user is not able to play".into(),
        ));
    }
    let source_card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad source-free card".into()))?
        .clone();
    if crate::query::card_use_cancelled_for_logical_kind(
        tx.state(),
        actor,
        &source_card,
        CardKind::TrickJuedou,
    ) {
        return Err(EffectError::Illegal("source-free card use disabled".into()));
    }
    if targets.is_empty() {
        return Err(EffectError::Illegal(
            "source-free duel needs at least one target".into(),
        ));
    }
    ensure_distinct_targets(targets)?;
    validate_explicit_target_policy(tx, actor, &source_card, CardKind::TrickJuedou, targets)?;
    for &target in targets {
        validate_alive_other(tx, actor, target)?;
    }
    use_independent_trick_targets(tx, actor, card, targets, trick_effect::JUEDOU, false)
}

/// Use one suitless virtual Shunshou on an authoritative target list while bypassing only its
/// ordinary distance and one-target caps. Each target still receives an independent Wuxie window
/// and must expose a gainable hand/equipment card when the use starts.
pub(crate) fn play_source_free_multi_target_shunshou(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
        || targets.is_empty()
    {
        return Err(EffectError::Illegal(
            "source-free Shunshou user or targets are invalid".into(),
        ));
    }
    let source_card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad source-free card".into()))?
        .clone();
    if crate::query::card_use_cancelled_for_logical_kind(
        tx.state(),
        actor,
        &source_card,
        CardKind::TrickShunshou,
    ) {
        return Err(EffectError::Illegal("source-free card use disabled".into()));
    }
    ensure_distinct_targets(targets)?;
    validate_explicit_target_policy(tx, actor, &source_card, CardKind::TrickShunshou, targets)?;
    for &target in targets {
        validate_alive_other(tx, actor, target)?;
        if crate::query::gainable_target_cards(
            tx.state(),
            actor,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .is_empty()
        {
            return Err(EffectError::Illegal(
                "source-free Shunshou target has no gainable card".into(),
            ));
        }
    }
    use_independent_trick_targets(tx, actor, card, targets, trick_effect::SHUNSHOU, false)
}

/// Use one exact suitless virtual Sha on a fixed target and draw for `skill` if that use
/// commits no damage. Ordinary distance and target policies apply; the play-phase use count does
/// not because the use is produced by a triggered skill.
pub(crate) fn use_virtual_sha_on_draw_if_no_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no removed zone".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
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
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?
        .cards
        .push(card);

    let player_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive && !participant.dying);
    let legal = player_alive
        && tx.state().cards.get(card).is_some_and(|card_state| {
            !crate::query::card_use_cancelled_for_logical_kind(
                tx.state(),
                player,
                card_state,
                CardKind::Sha,
            ) && validate_explicit_target_policy(tx, player, card_state, CardKind::Sha, &[target])
                .is_ok()
        })
        && validate_sha_targets_with_options(tx, player, card, &[target], false).is_ok();
    if !legal {
        crate::effect::move_one_card(tx, card, processing, removed, PlayerId::NONE, true)?;
        if !player_alive {
            return Ok(());
        }
        return run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player,
                count: 1,
                skill,
            }],
        )
        .map(|_| ());
    }

    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no virtual Sha user hand".into()))?;
    crate::effect::move_one_card(tx, card, processing, hand, player, false)?;
    tx.state_mut()
        .resolution
        .register_sha_no_damage_draw(card, skill);
    play_as_kind_with_extra_damage_flags(
        tx,
        player,
        card,
        CardKind::Sha,
        &[target],
        false,
        false,
        &[],
        true,
        false,
        rule_api::damage_card_flag::DRAW_IF_NO_DAMAGE,
        &[],
        0,
    )
}

pub(crate) fn validate_independent_trick_target_count(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    base: u32,
    label: &str,
) -> Result<(), EffectError> {
    let limit = crate::query::card_target_limit_for_card(tx.state(), actor, card, base);
    if targets.is_empty() || targets.len() > limit as usize {
        return Err(EffectError::Illegal(format!(
            "{label} needs 1 to {limit} targets"
        )));
    }
    ensure_distinct_targets(targets)
}

pub(crate) fn ensure_distinct_targets(targets: &[PlayerId]) -> Result<(), EffectError> {
    let mut seen = std::collections::BTreeSet::new();
    if targets
        .iter()
        .any(|target| target.is_none() || !seen.insert(target.raw()))
    {
        return Err(EffectError::Illegal(
            "card targets must be distinct players".into(),
        ));
    }
    Ok(())
}

pub(crate) fn reject_other_targeting_card_if_self_only(
    tx: &Transaction,
    actor: PlayerId,
) -> Result<(), EffectError> {
    if crate::query::card_targets_self_only_for(tx.state(), actor) {
        Err(EffectError::Illegal(
            "card requires another player as target".into(),
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn run_effects_with_saved_resume(
    tx: &mut Transaction,
    effects: Vec<EffectRequest>,
    resume_kind: crate::state::ContinuationKind,
    resume_payload: Vec<u32>,
) -> Result<(crate::state::ContinuationKind, Vec<u32>), EffectError> {
    let sha = match resume_kind {
        crate::state::ContinuationKind::ResumePlayedSha => Some((
            PlayerId::new(*resume_payload.first().unwrap_or(&0)),
            CardId::new(*resume_payload.get(5).unwrap_or(&0)),
            vec![PlayerId::new(*resume_payload.get(1).unwrap_or(&0))],
            resume_payload
                .get(4)
                .is_some_and(|flags| flags & SHA_RESUME_COUNTED != 0),
        )),
        crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let len = *resume_payload.get(5).unwrap_or(&0) as usize;
            resume_payload
                .get(6..6usize.saturating_add(len))
                .map(|targets| {
                    (
                        PlayerId::new(*resume_payload.first().unwrap_or(&0)),
                        CardId::new(*resume_payload.get(4).unwrap_or(&0)),
                        targets.iter().copied().map(PlayerId::new).collect(),
                        resume_payload
                            .get(3)
                            .is_some_and(|flags| flags & SHA_RESUME_COUNTED != 0),
                    )
                })
        }
        _ => None,
    };
    {
        let resolution = &mut tx.state_mut().resolution;
        resolution.resume_kind = resume_kind;
        resolution.resume_payload = resume_payload;
        if let Some((source, card, targets, use_counted)) = sha {
            resolution.register_sha_resolution(source, card, &targets, use_counted);
        }
    }
    match run_effects(tx, effects) {
        Ok(_) => {
            let resolution = &mut tx.state_mut().resolution;
            let kind = resolution.resume_kind;
            resolution.resume_kind = crate::state::ContinuationKind::None;
            Ok((kind, std::mem::take(&mut resolution.resume_payload)))
        }
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Err(error) => {
            tx.state_mut().resolution.resume_kind = crate::state::ContinuationKind::None;
            tx.state_mut().resolution.resume_payload.clear();
            Err(error)
        }
    }
}

pub(crate) fn resume_played_sha_from_payload(
    tx: &mut Transaction,
    kind: crate::state::ContinuationKind,
    payload: &[u32],
) -> Result<(), EffectError> {
    let (source, card) = match kind {
        crate::state::ContinuationKind::ResumePlayedSha => (
            PlayerId::new(*payload.first().unwrap_or(&0)),
            CardId::new(*payload.get(5).unwrap_or(&0)),
        ),
        crate::state::ContinuationKind::ResumePlayedShaQueue => (
            PlayerId::new(*payload.first().unwrap_or(&0)),
            CardId::new(*payload.get(4).unwrap_or(&0)),
        ),
        _ => (PlayerId::NONE, CardId::NONE),
    };
    if crate::effect::pottaishici_response_cost_armed(tx, source, card) {
        {
            let resolution = &mut tx.state_mut().resolution;
            resolution.resume_kind = kind;
            resolution.resume_payload = payload.to_vec();
        }
        crate::effect::begin_pottaishici_response_cost(tx, source, card)?;
        if tx.state().has_pending_choice() {
            return Err(EffectError::ChoiceRequested);
        }
        let resolution = &mut tx.state_mut().resolution;
        let next_kind = resolution.resume_kind;
        resolution.resume_kind = crate::state::ContinuationKind::None;
        let next = std::mem::take(&mut resolution.resume_payload);
        return resume_played_sha_from_payload(tx, next_kind, &next);
    }
    match kind {
        crate::state::ContinuationKind::ResumePlayedSha => {
            let source = PlayerId::new(*payload.first().unwrap_or(&0));
            let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let card = CardId::new(*payload.get(5).unwrap_or(&0));
            tx.state_mut().resolution.register_sha_resolution(
                source,
                card,
                &[target],
                payload
                    .get(4)
                    .is_some_and(|flags| flags & SHA_RESUME_COUNTED != 0),
            );
            if payload.iter().skip(6).any(|value| *value == target.raw()) {
                return finish_played_sha(tx, source, card, *payload.get(3).unwrap_or(&0));
            }
            let amount =
                payload
                    .get(2)
                    .copied()
                    .unwrap_or(1)
                    .saturating_add(sha_target_damage_bonus(
                        payload.get(4).copied().unwrap_or(1),
                    ));
            let (nature, card_flags) = unpack_damage_card_context(*payload.get(3).unwrap_or(&0));
            let respondable = *payload.get(4).unwrap_or(&1) & SHA_RESUME_RESPONDABLE != 0;
            begin_sha_response(
                tx,
                source,
                target,
                amount,
                nature,
                card_flags,
                respondable,
                Some(card),
            )
        }
        crate::state::ContinuationKind::ResumePlayedShaQueue => {
            let source = PlayerId::new(*payload.first().unwrap_or(&0));
            let amount = *payload.get(1).unwrap_or(&1);
            let (nature, card_flags) = unpack_damage_card_context(*payload.get(2).unwrap_or(&0));
            let respondable = *payload.get(3).unwrap_or(&1) & SHA_RESUME_RESPONDABLE != 0;
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let len = *payload.get(5).unwrap_or(&0) as usize;
            let declared_targets: Vec<_> = payload
                .iter()
                .skip(6)
                .take(len)
                .copied()
                .map(PlayerId::new)
                .collect();
            tx.state_mut().resolution.register_sha_resolution(
                source,
                card,
                &declared_targets,
                payload
                    .get(3)
                    .is_some_and(|flags| flags & SHA_RESUME_COUNTED != 0),
            );
            let targets_end = 6usize.saturating_add(len);
            let has_response_flags = payload.get(targets_end) == Some(&u32::MAX)
                && payload.len() >= targets_end.saturating_add(1).saturating_add(len);
            let flags_start = targets_end.saturating_add(1);
            let exclusions_start = if has_response_flags {
                flags_start.saturating_add(len)
            } else {
                targets_end
            };
            let exclusions = payload.get(exclusions_start..).unwrap_or_default();
            let forced_unrespondable: Vec<_> = payload
                .iter()
                .skip(6)
                .take(len)
                .enumerate()
                .filter(|(index, target)| {
                    !exclusions.contains(target)
                        && has_response_flags
                        && payload
                            .get(flags_start + index)
                            .is_some_and(|flags| flags & SHA_RESUME_RESPONDABLE == 0)
                })
                .map(|(_, target)| PlayerId::new(*target))
                .collect();
            let target_damage_bonuses: Vec<_> = payload
                .iter()
                .skip(6)
                .take(len)
                .enumerate()
                .filter(|(_, target)| !exclusions.contains(target))
                .map(|(index, _)| {
                    has_response_flags
                        .then(|| payload.get(flags_start + index).copied())
                        .flatten()
                        .map(sha_target_damage_bonus)
                        .unwrap_or(0)
                })
                .collect();
            let targets: Vec<_> = payload
                .iter()
                .skip(6)
                .take(len)
                .copied()
                .filter(|target| !exclusions.contains(target))
                .map(PlayerId::new)
                .collect();
            if targets.is_empty() {
                return finish_played_sha(tx, source, card, *payload.get(2).unwrap_or(&0));
            }
            begin_sha_queue(
                tx,
                source,
                &targets,
                amount,
                nature,
                card_flags,
                respondable,
                Some(card),
                &forced_unrespondable,
                &target_damage_bonuses,
            )
        }
        _ => Err(EffectError::EngineFault(
            "invalid played Sha continuation".into(),
        )),
    }
}

pub(crate) fn run_effects_then_wuxie(
    tx: &mut Transaction,
    effects: Vec<EffectRequest>,
    ask_start: PlayerId,
    effect_tag: u32,
    actor: PlayerId,
    effect_args: &[u32],
) -> Result<(), EffectError> {
    let card = effects
        .iter()
        .find_map(|effect| match effect {
            EffectRequest::UseCard { player, card, .. }
            | EffectRequest::UseFieldCard { player, card, .. }
                if *player == actor =>
            {
                Some(*card)
            }
            _ => None,
        })
        .ok_or_else(|| EffectError::EngineFault("wuxie window missing source card".into()))?;
    let source_card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::EngineFault("wuxie window source card missing".into()))?;
    let response_kind = crate::query::effective_card_kind(tx.state(), actor, source_card);
    let response_suit = crate::query::effective_card_suit(tx.state(), actor, source_card);
    let mut resume_payload = vec![
        ask_start.raw(),
        0,
        effect_tag,
        actor.raw(),
        actor.raw(),
        response_kind.tag() as u32,
        super::super::judge::suit_tag(response_suit),
    ];
    resume_payload.extend_from_slice(effect_args);
    resume_payload.push(SAVED_WUXIE_SOURCE_CARD_MAGIC);
    resume_payload.push(card.raw());
    let (resume_kind, resume_payload) = run_effects_with_saved_resume(
        tx,
        effects,
        crate::state::ContinuationKind::WuxieWindow,
        resume_payload,
    )?;
    if resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return Err(EffectError::EngineFault(
            "card-used triggers changed the Wuxie resume kind".into(),
        ));
    }
    let saved_args = resume_payload
        .get(7..)
        .ok_or_else(|| EffectError::EngineFault("truncated Wuxie resume payload".into()))?;
    resume_wuxie_after_card_used_triggers(
        tx,
        ask_start,
        *resume_payload.get(1).unwrap_or(&0),
        effect_tag,
        actor,
        actor,
        response_kind,
        response_suit,
        saved_args,
    )?;
    crate::effect::finish_pending_card_replays_if_idle(tx)
}

pub(crate) fn remove_saved_target(
    payload: &mut Vec<u32>,
    count_index: usize,
    start: usize,
    target: PlayerId,
) {
    let count = payload.get(count_index).copied().unwrap_or(0) as usize;
    let Some(offset) = payload
        .get(start..start.saturating_add(count))
        .and_then(|targets| targets.iter().position(|value| *value == target.raw()))
    else {
        return;
    };
    payload.remove(start + offset);
    payload[count_index] = payload[count_index].saturating_sub(1);
}

/// Exclude one target from the currently suspended physical card resolution.
pub(crate) fn exclude_saved_card_target(
    tx: &mut Transaction,
    card: CardId,
    target: PlayerId,
) -> Result<(), EffectError> {
    if tx
        .state_mut()
        .resolution
        .exclude_saved_sha_target(card, target)
    {
        return Ok(());
    }
    let resolution = &tx.state().resolution;
    if resolution.resume_kind != crate::state::ContinuationKind::WuxieWindow {
        return Err(EffectError::Illegal(
            "no matching currently resolving card target".into(),
        ));
    }
    let marker = resolution
        .resume_payload
        .iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC)
        .ok_or_else(|| EffectError::Illegal("saved card marker is missing".into()))?;
    if resolution.resume_payload.get(marker + 1) != Some(&card.raw()) {
        return Err(EffectError::Illegal(
            "saved card target does not match triggering card".into(),
        ));
    }
    let effect_tag = *resolution.resume_payload.get(2).unwrap_or(&0);
    let default_global_targets = (marker == 7
        && matches!(effect_tag, trick_effect::TAOYUAN | trick_effect::WUGU))
    .then(|| {
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != target)
            .map(PlayerId::raw)
            .collect::<Vec<_>>()
    });
    let payload = &mut tx.state_mut().resolution.resume_payload;
    if payload.get(7) == Some(&INDEPENDENT_TRICK_QUEUE_MAGIC) {
        remove_saved_target(payload, 14, 15, target);
    } else {
        match effect_tag {
            trick_effect::NANMAN | trick_effect::WANJIAN | trick_effect::TIESUO => {
                let (count, start) = if effect_tag == trick_effect::TIESUO
                    && payload.get(7) == Some(&TIESUO_SAVED_TARGETS_MAGIC)
                {
                    (9, 10)
                } else {
                    (7, 8)
                };
                remove_saved_target(payload, count, start, target);
            }
            trick_effect::TAOYUAN | trick_effect::WUGU => {
                if let Some(targets) = default_global_targets {
                    payload.insert(7, targets.len() as u32);
                    for (offset, player) in targets.into_iter().enumerate() {
                        payload.insert(8 + offset, player);
                    }
                } else {
                    remove_saved_target(payload, 7, 8, target);
                }
            }
            trick_effect::JIEDAO => {
                for index in 7..=8 {
                    if payload.get(index) == Some(&target.raw()) {
                        payload[index] = PlayerId::NONE.raw();
                    }
                }
            }
            trick_effect::TIAOJIYANMEI => {
                for index in 7..=8 {
                    if payload
                        .get(index)
                        .map(|raw| raw & !SAVED_TARGET_EXCLUDED_MASK)
                        == Some(target.raw())
                    {
                        payload[index] |= SAVED_TARGET_EXCLUDED_MASK;
                    }
                }
            }
            trick_effect::DELAYED_PLACE => {
                if payload.get(8) == Some(&target.raw()) {
                    payload[8] = PlayerId::NONE.raw();
                }
            }
            _ => {
                return Err(EffectError::Illegal(
                    "current card target cannot be excluded".into(),
                ));
            }
        }
    }
    Ok(())
}

/// Exclude every target of the currently suspended card use.
pub(crate) fn exclude_all_saved_card_targets(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let mut excluded = 0usize;
    for target in targets {
        if exclude_saved_card_target(tx, card, *target).is_ok() {
            excluded += 1;
        }
    }
    if excluded == targets.len() {
        return Ok(());
    }
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), player, card))
        .ok_or_else(|| EffectError::Illegal("missing current card".into()))?;
    let before = tx.state().resolution.deferred_tail_effects.len();
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .retain(|effect| match effect {
            EffectRequest::RecoverHp {
                source,
                player: target,
                ..
            } if matches!(kind, CardKind::Tao | CardKind::Jiu)
                && *source == player
                && targets.contains(target) =>
            {
                false
            }
            EffectRequest::AddMark {
                player: target,
                mark,
                ..
            } if kind == CardKind::Jiu
                && *target == player
                && mark.raw() == crate::content::MARK_JIU =>
            {
                false
            }
            _ => true,
        });
    if excluded == 0 && before == tx.state().resolution.deferred_tail_effects.len() {
        return Err(EffectError::ContentFault(
            "current card targets cannot be cancelled".into(),
        ));
    }
    Ok(())
}

/// Resume the original trick after card-used triggers, suppressing its Wuxie window when an
/// exact-card policy was installed while those triggers were resolving.
pub(crate) fn resume_wuxie_after_card_used_triggers(
    tx: &mut Transaction,
    start: PlayerId,
    nullified: u32,
    effect_tag: u32,
    actor: PlayerId,
    respond_to: PlayerId,
    response_kind: CardKind,
    response_suit: engine_api::Suit,
    args: &[u32],
) -> Result<(), EffectError> {
    let marker = args
        .iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC);
    let (effect_args, card) = marker.map_or((args, CardId::NONE), |index| {
        (
            &args[..index],
            CardId::new(*args.get(index + 1).unwrap_or(&0)),
        )
    });
    if card.is_some()
        && crate::query::card_use_unrespondable_for_card(
            tx.state(),
            respond_to,
            card,
            response_kind,
            response_suit,
        )
    {
        return apply_deferred_trick_with_extras(tx, effect_tag, actor, effect_args, card);
    }
    begin_wuxie_window_with_parity(
        tx,
        start,
        nullified,
        effect_tag,
        actor,
        respond_to,
        response_kind,
        response_suit,
        args,
    )
}

pub(crate) fn effect_args_source_card(args: &[u32]) -> CardId {
    if args.first() == Some(&INDEPENDENT_TRICK_QUEUE_MAGIC) {
        return CardId::new(*args.get(1).unwrap_or(&0));
    }
    args.iter()
        .rposition(|value| *value == SAVED_WUXIE_SOURCE_CARD_MAGIC)
        .and_then(|index| args.get(index + 1))
        .copied()
        .map(CardId::new)
        .unwrap_or(CardId::NONE)
}

pub(crate) fn use_card_then_wuxie(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    effect_tag: u32,
    effect_args: &[u32],
) -> Result<(), EffectError> {
    run_effects_then_wuxie(
        tx,
        vec![EffectRequest::UseCard {
            player: actor,
            card,
            targets: targets.to_vec(),
        }],
        actor,
        effect_tag,
        actor,
        effect_args,
    )
}

pub(crate) fn saved_independent_trick_damage_bonus(args: &[u32]) -> u32 {
    if args.first() != Some(&INDEPENDENT_TRICK_QUEUE_MAGIC) {
        return 0;
    }
    let payload = &args[1..];
    let target_count = *payload.get(6).unwrap_or(&0) as usize;
    let extension_at = 7usize.saturating_add(target_count);
    if payload.get(extension_at) == Some(&SAVED_CARD_BASE_DAMAGE_MAGIC) {
        *payload.get(extension_at + 1).unwrap_or(&0)
    } else {
        0
    }
}
