//! Command operations for `skill_card_target_spec_with_options` and related helpers.

use super::*;

/// Play a view-as card while attaching engine-owned post-use damage flags.
pub(crate) fn play_as_kind_with_extra_damage_flags(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    as_kind: CardKind,
    targets: &[PlayerId],
    ignore_distance: bool,
    force_unrespondable: bool,
    forced_unrespondable_targets: &[PlayerId],
    ignore_use_count: bool,
    source_was_hand: bool,
    extra_damage_flags: u8,
    extra_cards: &[CardId],
    multi_card_bonus: u32,
) -> Result<(), EffectError> {
    play_as_kind_with_extra_damage_flags_target_bonus(
        tx,
        actor,
        card,
        as_kind,
        targets,
        ignore_distance,
        force_unrespondable,
        forced_unrespondable_targets,
        ignore_use_count,
        source_was_hand,
        extra_damage_flags,
        extra_cards,
        multi_card_bonus,
        0,
    )
}

/// Play a view-as card with one content-authorized target-limit bonus.
pub(crate) fn play_as_kind_with_extra_damage_flags_target_bonus(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    as_kind: CardKind,
    targets: &[PlayerId],
    ignore_distance: bool,
    force_unrespondable: bool,
    forced_unrespondable_targets: &[PlayerId],
    ignore_use_count: bool,
    source_was_hand: bool,
    extra_damage_flags: u8,
    extra_cards: &[CardId],
    multi_card_bonus: u32,
    target_limit_bonus: u8,
) -> Result<(), EffectError> {
    let player = tx
        .state()
        .players
        .get(actor)
        .ok_or_else(|| EffectError::Illegal("bad actor".into()))?
        .clone();
    let source_card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad card".into()))?
        .clone();
    if crate::query::card_use_cancelled_for_logical_kind(tx.state(), actor, &source_card, as_kind) {
        return Err(EffectError::Illegal("card use disabled".into()));
    }
    validate_explicit_target_policy(tx, actor, &source_card, as_kind, targets)?;
    let converted_use_unlimited = extra_cards.is_empty()
        && crate::query::card_use_unlimited_for_logical_kind(tx.state(), actor, card, as_kind);
    remember_hand_card_suit_use(tx, actor, card, source_was_hand);
    let mut queue: Vec<EffectRequest> = Vec::new();
    if !extra_cards.is_empty() {
        let hand = tx.state().zones.find(ZoneKind::Hand, actor);
        let mut source_cards = Vec::with_capacity(extra_cards.len() + 1);
        source_cards.push(card);
        source_cards.extend_from_slice(extra_cards);
        let all_sources_were_hand = source_was_hand
            && extra_cards.iter().all(|source| {
                tx.state().cards.get(*source).is_some_and(|source_card| {
                    Some(source_card.zone) == hand
                        || crate::query::borrowed_hand_source(tx.state(), actor, *source).is_some()
                })
            });
        tx.state_mut().resolution.plan_card_source_cards(
            actor,
            card,
            source_cards,
            all_sources_were_hand,
        );
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
        let refills = crate::effect::pothaoshi_refill_effects(tx.state(), actor, extra_cards);
        for extra in extra_cards {
            let from = tx
                .state()
                .cards
                .get(*extra)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("view-as source disappeared".into()))?;
            queue.push(EffectRequest::MoveCardsWithReason {
                cards: vec![*extra],
                to: discard,
                from: Some(from),
                reason: CardMoveReason::Use,
            });
        }
        queue.extend(refills);
    }
    match as_kind {
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => {
            let ignores_use_count = ignore_use_count
                || crate::query::sha_use_does_not_count_for_card(tx.state(), actor, card)
                || converted_use_unlimited;
            let respondable = !force_unrespondable
                && !crate::query::sha_unrespondable_for_logical_kind(
                    tx.state(),
                    actor,
                    &source_card,
                    as_kind,
                );
            if !ignores_use_count && !sha_use_count_allows_targets(tx.state(), actor, card, targets)
            {
                return Err(EffectError::Illegal("sha use limit reached".into()));
            }
            validate_sha_targets_with_options_target_bonus(
                tx,
                actor,
                card,
                targets,
                ignore_distance,
                target_limit_bonus,
            )?;
            let mut amount = 1u32.saturating_add(multi_card_bonus);
            let jiu = player
                .marks
                .iter()
                .any(|(m, c)| *m == crate::content::MARK_JIU && *c > 0);
            let card_flags = (if jiu {
                rule_api::damage_card_flag::JIU_BOOSTED
            } else {
                0
            }) | extra_damage_flags;
            if jiu {
                amount = amount.saturating_add(1);
                queue.push(EffectRequest::RemoveMark {
                    player: actor,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_JIU),
                    amount: 1,
                });
            }
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: targets.to_vec(),
            });
            // Proactive 杀 (view-as) during own turn counts for 克己.
            if tx.state().turn.current == actor && !ignores_use_count {
                record_counted_sha_use(tx);
            }
            let counted = tx.state().turn.current == actor && !ignores_use_count;
            let resume = if targets.len() == 1 {
                let resume_flags = saved_sha_flags(
                    tx,
                    respondable && !forced_unrespondable_targets.contains(&targets[0]),
                    counted,
                );
                (
                    crate::state::ContinuationKind::ResumePlayedSha,
                    vec![
                        actor.raw(),
                        targets[0].raw(),
                        amount,
                        pack_damage_card_context(as_kind.sha_nature(), card_flags),
                        resume_flags,
                        card.raw(),
                    ],
                )
            } else {
                let resume_flags = saved_sha_flags(tx, respondable, counted);
                let mut payload = vec![
                    actor.raw(),
                    amount,
                    pack_damage_card_context(as_kind.sha_nature(), card_flags),
                    resume_flags,
                    card.raw(),
                    targets.len() as u32,
                ];
                payload.extend(targets.iter().map(|target| target.raw()));
                payload.push(u32::MAX);
                payload.extend(
                    targets
                        .iter()
                        .map(|target| u32::from(!forced_unrespondable_targets.contains(target))),
                );
                (
                    crate::state::ContinuationKind::ResumePlayedShaQueue,
                    payload,
                )
            };
            if !extra_cards.is_empty() {
                let mut source_cards = Vec::with_capacity(extra_cards.len() + 1);
                source_cards.push(card);
                source_cards.extend_from_slice(extra_cards);
                let resolution = &mut tx.state_mut().resolution;
                resolution.register_sha_resolution(actor, card, targets, counted);
                debug_assert!(resolution.register_sha_source_cards(actor, card, &source_cards));
            }
            let (kind, payload) = run_effects_with_saved_resume(tx, queue, resume.0, resume.1)?;
            return resume_played_sha_from_payload(tx, kind, &payload);
        }
        CardKind::Tao => {
            if crate::query::card_target_cancelled_for_logical_kind(
                tx.state(),
                actor,
                actor,
                &source_card,
                CardKind::Tao,
            ) {
                return Err(EffectError::Illegal("tao target disabled".into()));
            }
            if player.hp >= player.max_hp as i32 {
                return Err(EffectError::Illegal("hp full".into()));
            }
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: vec![actor],
            });
            queue.push(EffectRequest::RecoverHp {
                source: actor,
                player: actor,
                amount: 1u32.saturating_add(multi_card_bonus),
            });
        }
        CardKind::Jiu => {
            queue.push(EffectRequest::UseCard {
                player: actor,
                card,
                targets: vec![actor],
            });
            if player.hp < player.max_hp as i32 {
                queue.push(EffectRequest::RecoverHp {
                    source: actor,
                    player: actor,
                    amount: 1,
                });
            } else {
                queue.push(EffectRequest::AddMark {
                    player: actor,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_JIU),
                    delta: 1,
                });
            }
        }
        CardKind::Shan => {
            return Err(EffectError::Illegal(
                "view_as shan not usable proactively (no response phase yet)".into(),
            ));
        }
        // Trick view-as (奇袭 → 过河拆桥, 国色 → 乐不思蜀, …): retag the source
        // card to the trick kind and delegate to the full play pipeline, reusing
        // its target validation and wuxie windows. No effect logic duplicated.
        CardKind::TrickGuohe
        | CardKind::TrickShunshou
        | CardKind::TrickJuedou
        | CardKind::TrickWuzhong
        | CardKind::TrickLe
        | CardKind::TrickBingliang
        | CardKind::TrickHuogong
        | CardKind::TrickJiedao
        | CardKind::TrickTiesuo
        | CardKind::TrickNanman
        | CardKind::TrickWanjian
        | CardKind::TrickTaoyuan
        | CardKind::TrickWugu
        | CardKind::TrickWangmeizhike
        | CardKind::TrickDiaobingqianjiang => {
            run_effects(tx, queue)?;
            if let Some(c) = tx.state_mut().cards.get_mut(card) {
                c.kind = as_kind;
            }
            return handle_card_from_hand_with_options(
                tx,
                actor,
                card,
                targets,
                ignore_distance,
                source_was_hand,
            );
        }
        _ => {
            return Err(EffectError::Illegal("unsupported view_as kind".into()));
        }
    }
    run_effects(tx, queue).map(|_| ())
}

pub(crate) fn validate_alive_other(
    tx: &Transaction,
    actor: PlayerId,
    target: PlayerId,
) -> Result<(), EffectError> {
    if target == actor {
        return Err(EffectError::Illegal("cannot target self".into()));
    }
    let t = tx
        .state()
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if !t.alive {
        return Err(EffectError::Illegal("target dead".into()));
    }
    Ok(())
}

/// Legal targets when one physical card is used as a distance-unlimited Bingliang.
pub(crate) fn view_as_bingliang_targets(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    let Some(actor_state) = tx.state().players.get(actor) else {
        return Vec::new();
    };
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    if !actor_state.alive
        || actor_state.dying
        || crate::query::card_use_cancelled_for_logical_kind(
            tx.state(),
            actor,
            card_state,
            CardKind::TrickBingliang,
        )
    {
        return Vec::new();
    }
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| validate_alive_other(tx, actor, *target).is_ok())
        .filter(|target| {
            validate_explicit_target_policy(
                tx,
                actor,
                card_state,
                CardKind::TrickBingliang,
                &[*target],
            )
            .is_ok()
        })
        .filter(|target| !has_delayed_of_kind(tx, *target, CardKind::TrickBingliang))
        .collect()
}

pub(crate) fn validate_sha_target(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
) -> Result<(), EffectError> {
    validate_sha_target_with_options(tx, actor, card, target, false)
}

/// Validate a Sha target while preserving every policy except ordinary range.
pub(crate) fn validate_sha_target_with_options(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    ignore_distance: bool,
) -> Result<(), EffectError> {
    let t = tx
        .state()
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if !t.alive {
        return Err(EffectError::Illegal("target dead".into()));
    }
    if target == actor {
        return Err(EffectError::Illegal("cannot sha self".into()));
    }
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad card".into()))?;
    if crate::query::card_target_cancelled_for_logical_kind(
        tx.state(),
        actor,
        target,
        card_state,
        CardKind::Sha,
    ) {
        return Err(EffectError::Illegal("sha target disabled".into()));
    }
    if ignore_distance
        || crate::query::sha_distance_unlimited_for(tx.state(), actor)
        || crate::query::card_distance_unlimited_for_target(tx.state(), actor, card, target)
    {
        return Ok(());
    }
    let mut ctx = QueryContext::new();
    let dist = query(
        tx.state(),
        &QueryRequest::Distance {
            from: actor,
            to: target,
        },
        &mut ctx,
    );
    let range = query(
        tx.state(),
        &QueryRequest::AttackRange { player: actor },
        &mut ctx,
    );
    match (dist, range) {
        (QueryResponse::Int(d), QueryResponse::Int(r)) => {
            if d > r {
                return Err(EffectError::Illegal("target out of range".into()));
            }
        }
        _ => return Err(EffectError::Illegal("distance query failed".into())),
    }
    Ok(())
}

/// Whether one exact physical hand card can be used by `actor` so that `target` is affected,
/// ignoring only ordinary distance. Used by XIANDING 英谋's forced serial card uses.
pub(crate) fn physical_damage_card_can_target_without_distance(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
) -> bool {
    if actor == target
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|participant| participant.alive && !participant.dying)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx.state().hand_cards(actor).contains(&card)
    {
        return false;
    }
    let Some(card_state) = tx.state().cards.get(card) else {
        return false;
    };
    if crate::query::card_use_cancelled_for_card(tx.state(), actor, card) {
        return false;
    }
    let kind = crate::query::effective_card_kind(tx.state(), actor, card_state);
    match kind {
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => {
            sha_use_count_allows_targets(tx.state(), actor, card, &[target])
                && validate_explicit_target_policy(tx, actor, card_state, kind, &[target]).is_ok()
                && validate_sha_target_with_options(tx, actor, card, target, true).is_ok()
        }
        CardKind::TrickJuedou => {
            crate::query::card_target_limit_for_card(tx.state(), actor, card, 1) > 0
                && validate_explicit_target_policy(tx, actor, card_state, kind, &[target]).is_ok()
        }
        CardKind::TrickHuogong => {
            crate::query::card_target_limit_for_card(tx.state(), actor, card, 1) > 0
                && tx.state().hand_count(target) > 0
                && validate_explicit_target_policy(tx, actor, card_state, kind, &[target]).is_ok()
        }
        CardKind::TrickNanman | CardKind::TrickWanjian => {
            !crate::query::card_targets_self_only_for(tx.state(), actor)
        }
        _ => false,
    }
}

pub(crate) fn validate_sha_targets_with_options(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
) -> Result<(), EffectError> {
    validate_sha_targets_with_options_target_bonus(tx, actor, card, targets, ignore_distance, 0)
}

pub(crate) fn validate_sha_targets_with_options_target_bonus(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    target_limit_bonus: u8,
) -> Result<(), EffectError> {
    if crate::query::sha_targets_all_for(tx.state(), actor) {
        let expected: Vec<PlayerId> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != actor)
            .filter(|target| {
                validate_sha_target_with_options(tx, actor, card, *target, ignore_distance).is_ok()
            })
            .collect();
        if targets != expected {
            return Err(EffectError::Illegal(
                "sha must target every player in range".into(),
            ));
        }
        return Ok(());
    }
    let limit = crate::query::sha_target_limit_for(tx.state(), actor)
        .saturating_add(u32::from(target_limit_bonus)) as usize;
    if targets.is_empty() || targets.len() > limit {
        return Err(EffectError::Illegal(format!(
            "sha needs 1 to {limit} targets"
        )));
    }
    let mut seen = std::collections::BTreeSet::new();
    for target in targets {
        if !seen.insert(*target) {
            return Err(EffectError::Illegal("duplicate sha target".into()));
        }
        validate_sha_target_with_options(tx, actor, card, *target, ignore_distance)?;
    }
    Ok(())
}

/// Validate a source-free logical Sha before asking another player to provide its physical card.
pub(crate) fn validate_source_free_sha_use(
    tx: &Transaction,
    actor: PlayerId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
    {
        return Err(EffectError::Illegal(
            "source-free Sha user is not able to play".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no source-free Sha user hand".into()))?;
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    if crate::query::card_use_cancelled_for_logical_kind(tx.state(), actor, &card, CardKind::Sha) {
        return Err(EffectError::Illegal("source-free Sha use disabled".into()));
    }
    validate_explicit_target_policy(tx, actor, &card, CardKind::Sha, targets)?;
    let target_legal = |target: PlayerId| {
        tx.state()
            .players
            .get(target)
            .is_some_and(|player| player.alive)
            && target != actor
            && !crate::query::card_target_cancelled_for_logical_kind(
                tx.state(),
                actor,
                target,
                &card,
                CardKind::Sha,
            )
            && (crate::query::sha_distance_unlimited_for(tx.state(), actor)
                || crate::query::card_distance_unlimited_for_target(
                    tx.state(),
                    actor,
                    CardId::NONE,
                    target,
                )
                || crate::query::player_in_attack_range(tx.state(), actor, target))
    };
    if crate::query::sha_targets_all_for(tx.state(), actor) {
        let expected: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| target_legal(*target))
            .collect();
        if targets != expected {
            return Err(EffectError::Illegal(
                "sha must target every player in range".into(),
            ));
        }
    } else {
        let limit = crate::query::sha_target_limit_for(tx.state(), actor) as usize;
        if targets.is_empty() || targets.len() > limit {
            return Err(EffectError::Illegal(format!(
                "sha needs 1 to {limit} targets"
            )));
        }
        let mut seen = std::collections::BTreeSet::new();
        if targets
            .iter()
            .any(|target| !seen.insert(*target) || !target_legal(*target))
        {
            return Err(EffectError::Illegal(
                "invalid source-free Sha target".into(),
            ));
        }
    }
    if crate::query::sha_play_phase_use_count(tx.state())
        >= crate::query::sha_use_limit_for(tx.state(), actor)
        && !targets
            .iter()
            .all(|target| crate::query::sha_use_unlimited_for_target(tx.state(), actor, *target))
    {
        return Err(EffectError::Illegal("sha use limit reached".into()));
    }
    Ok(())
}

pub(crate) fn validate_explicit_target_policy(
    tx: &Transaction,
    actor: PlayerId,
    card: &crate::state::Card,
    logical_kind: CardKind,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    if logical_kind.is_sha()
        && !crate::query::luanqun_sha_targets_allowed(tx.state(), actor, targets)
    {
        return Err(EffectError::Illegal(
            "first Sha after Luanqun may only target its owners".into(),
        ));
    }
    if crate::query::card_targets_self_only_for(tx.state(), actor)
        && targets.iter().any(|target| *target != actor)
    {
        return Err(EffectError::Illegal("card may only target its user".into()));
    }
    if targets.iter().any(|target| {
        crate::query::card_target_cancelled_for_logical_kind(
            tx.state(),
            actor,
            *target,
            card,
            logical_kind,
        )
    }) {
        return Err(EffectError::Illegal("card target disabled".into()));
    }
    Ok(())
}

pub(crate) fn validate_source_free_card_on_target(
    tx: &Transaction,
    actor: PlayerId,
    kind: CardKind,
    target: PlayerId,
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
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no source-free card user hand".into()))?;
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    if crate::query::card_use_cancelled_for_logical_kind(tx.state(), actor, &card, kind) {
        return Err(EffectError::Illegal("source-free card use disabled".into()));
    }
    validate_explicit_target_policy(tx, actor, &card, kind, &[target])?;
    match kind {
        CardKind::TrickJuedou => {
            if crate::query::card_target_limit_for_kind(tx.state(), actor, kind, 1) == 0 {
                return Err(EffectError::Illegal(
                    "source-free card has no legal target".into(),
                ));
            }
            validate_alive_other(tx, actor, target)
        }
        CardKind::TrickHuogong => {
            if tx.state().hand_cards(target).is_empty() {
                Err(EffectError::Illegal("target has no hand".into()))
            } else {
                Ok(())
            }
        }
        _ => Err(EffectError::ContentFault(
            "unsupported source-free active target card kind".into(),
        )),
    }
}

/// Whether a living actor can currently use one of the source-free card kinds needed by
/// phase-end skills. This is the non-mutating counterpart of `skill_card_target_spec`.
pub(crate) fn source_free_card_has_legal_use(
    state: &crate::state::GameState,
    actor: PlayerId,
    kind: CardKind,
) -> bool {
    if !state
        .players
        .get(actor)
        .is_some_and(|player| player.alive && !player.dying)
    {
        return false;
    }
    let Some(hand) = state.zones.find(ZoneKind::Hand, actor) else {
        return false;
    };
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    if crate::query::card_use_cancelled_for_logical_kind(state, actor, &card, kind)
        || crate::query::card_targets_self_only_for(state, actor)
    {
        return false;
    }
    let legal_other = |target: PlayerId| {
        target != actor
            && state
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            && !crate::query::card_target_cancelled_for_logical_kind(
                state, actor, target, &card, kind,
            )
    };
    match kind {
        CardKind::TrickWanjian | CardKind::TrickNanman => {
            let targets: Vec<_> = state
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != actor)
                .collect();
            !targets.is_empty() && targets.iter().copied().all(legal_other)
        }
        CardKind::TrickJuedou => {
            crate::query::card_target_limit_for_kind(state, actor, kind, 1) > 0
                && state.players.alive_by_seat().into_iter().any(legal_other)
        }
        _ => false,
    }
}

/// Whether an ordinary source-free Sha may target one player while ignoring only distance.
pub(crate) fn source_free_sha_can_target_without_distance(
    state: &crate::state::GameState,
    actor: PlayerId,
    target: PlayerId,
) -> bool {
    let Some(owner) = state.players.get(actor) else {
        return false;
    };
    let Some(hand) = state.zones.find(ZoneKind::Hand, actor) else {
        return false;
    };
    if !owner.alive
        || owner.dying
        || actor == target
        || !state.players.get(target).is_some_and(|player| player.alive)
    {
        return false;
    }
    let card = Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: hand,
        owner: actor,
        face_up: true,
    };
    let use_count_allowed = crate::query::sha_play_phase_use_count(state)
        < crate::query::sha_use_limit_for(state, actor)
        || crate::query::sha_use_unlimited_for_target(state, actor, target);
    let only_target = !crate::query::sha_targets_all_for(state, actor)
        || state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != actor)
            .count()
            == 1;
    use_count_allowed
        && only_target
        && crate::query::sha_target_limit_for(state, actor) > 0
        && crate::query::luanqun_sha_targets_allowed(state, actor, &[target])
        && !crate::query::card_use_cancelled_for_logical_kind(state, actor, &card, CardKind::Sha)
        && !crate::query::card_targets_self_only_for(state, actor)
        && !crate::query::card_target_cancelled_for_logical_kind(
            state,
            actor,
            target,
            &card,
            CardKind::Sha,
        )
}
