//! Command operations for `resolve_choose_gain_sha_for_opening_extra_play` and related helpers.

use super::*;

pub(crate) fn resolve_choose_gain_resolved_sha_or_shan_then_optional_use_sha(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    match stage {
        0 | 4 => {
            let owner = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
            if cancelled || observer != owner || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose exactly one resolved Sha gift recipient".into(),
                ));
            }
            let (sha, shan, excluded) = resolved_sha_gift_groups(payload, 3)?;
            let recipient = PlayerId::new(selections[0]);
            if excluded.contains(&recipient)
                || !tx
                    .state()
                    .players
                    .get(recipient)
                    .is_some_and(|participant| participant.alive)
                || !tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == owner && tx.state().skill_effectively_enabled(instance)
                })
            {
                return Err(EffectError::Illegal(
                    "resolved Sha gift recipient is no longer legal".into(),
                ));
            }
            let sha = current_discard_group(tx, &sha);
            let shan = current_discard_group(tx, &shan);
            if stage == 4 {
                let cards: Vec<_> = sha.into_iter().chain(shan).collect();
                return if cards.is_empty() {
                    Ok(())
                } else {
                    give_resolved_sha_group(tx, owner, recipient, skill, &cards, true)
                };
            }
            match (sha.is_empty(), shan.is_empty()) {
                (true, true) => Ok(()),
                (false, true) => give_resolved_sha_group(tx, owner, recipient, skill, &sha, false),
                (true, false) => give_resolved_sha_group(tx, owner, recipient, skill, &shan, false),
                (false, false) => {
                    let mut next = vec![
                        1,
                        owner.raw(),
                        recipient.raw(),
                        skill.raw(),
                        sha.len() as u32,
                    ];
                    next.extend(sha.iter().map(|card| card.raw()));
                    next.push(shan.len() as u32);
                    next.extend(shan.iter().map(|card| card.raw()));
                    next.push(excluded.len() as u32);
                    next.extend(excluded.iter().map(|target| target.raw()));
                    crate::effect::open_choice(
                        tx,
                        owner,
                        1,
                        1,
                        false,
                        vec![1, 2],
                        engine_api::ChoiceKind::Custom,
                        ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha,
                        next,
                    )
                    .map(|_| ())
                }
            }
        }
        1 => {
            let owner = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let recipient = PlayerId::new(*payload.get(2).unwrap_or(&0));
            let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
            if cancelled || observer != owner || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose the resolved Sha or Shan group".into(),
                ));
            }
            let (sha, shan, _) = resolved_sha_gift_groups(payload, 4)?;
            match selections[0] {
                1 => give_resolved_sha_group(tx, owner, recipient, skill, &sha, false),
                2 => give_resolved_sha_group(tx, owner, recipient, skill, &shan, false),
                _ => Err(EffectError::Illegal("unknown resolved card group".into())),
            }
        }
        2 => {
            let owner = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
            let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
            if observer != player {
                return Err(EffectError::Illegal(
                    "follow-up Sha choice belongs to another player".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled follow-up Sha choice has selections".into(),
                    ));
                }
                return Ok(());
            }
            if selections.is_empty() {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose at most one follow-up Sha source".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            let targets = crate::command::play::prompted_sha_targets(tx, owner, player, card);
            if targets.is_empty()
                || !crate::command::play::prompted_sha_source_cards(tx.state(), player)
                    .contains(&card)
            {
                return Err(EffectError::Illegal(
                    "follow-up Sha source is no longer legal".into(),
                ));
            }
            let max = targets
                .len()
                .min(crate::query::sha_target_limit_for(tx.state(), player) as usize)
                .min(u8::MAX as usize) as u8;
            crate::effect::open_choice(
                tx,
                player,
                1,
                max,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha,
                vec![3, owner.raw(), player.raw(), skill.raw(), card.raw()],
            )
            .map(|_| ())
        }
        3 => {
            let owner = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            if observer != player {
                return Err(EffectError::Illegal(
                    "follow-up Sha target choice belongs to another player".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            crate::command::play::play_prompted_sha_source(tx, owner, player, card, &targets)
        }
        _ => Err(EffectError::EngineFault(
            "unknown resolved Sha gift continuation stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_other_targets_by_skipped_phase_count_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 && payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad skipped-phase damage continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let amount = payload[1];
    let skill = engine_api::SkillId::new(payload[2]);
    let draw_per_damage = payload.get(3) == Some(&1);
    let max = tx.state().turn.skipped_phase_count() as usize;
    if observer != player
        || cancelled
        || amount == 0
        || selections.is_empty()
        || selections.len() > max
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "skipped-phase damage choice context changed".into(),
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut targets = Vec::with_capacity(selections.len());
    for raw in selections {
        let target = PlayerId::new(*raw);
        if target == player
            || !seen.insert(*raw)
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "skipped-phase damage target is no longer legal".into(),
            ));
        }
        targets.push(target);
    }
    let events = run_effects(
        tx,
        targets
            .iter()
            .map(|target| EffectRequest::DealDamage {
                source: player,
                target: *target,
                amount,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            })
            .collect(),
    )?;
    if draw_per_damage {
        let damaged: std::collections::BTreeSet<u32> = events
            .iter()
            .filter_map(|event| match event {
                engine_api::DomainEvent::DamageDealt { source, target, .. }
                    if *source == player && targets.contains(target) =>
                {
                    Some(target.raw())
                }
                _ => None,
            })
            .collect();
        if !damaged.is_empty() {
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player,
                    count: damaged.len() as u32,
                }],
            )?;
        }
    }
    Ok(())
}

pub(crate) fn resolve_choose_other_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-other-damage continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let amount = payload[1];
    let skill = engine_api::SkillId::new(payload[2]);
    let target = PlayerId::new(selections[0]);
    if observer != player
        || cancelled
        || amount == 0
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
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "choose-other-damage context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_bound_source_attack_range_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad bound-source attack-range damage continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let source = PlayerId::new(payload[1]);
    let amount = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    let target = PlayerId::new(selections[0]);
    let skill_matches = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .and_then(|instance| instance.bound_target())
        == Some(source);
    if observer != player
        || cancelled
        || amount == 0
        || !skill_matches
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive)
        || target == source
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !crate::query::player_in_attack_range(tx.state(), source, target)
    {
        return Err(EffectError::Illegal(
            "bound-source attack-range damage context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_other_discard_target_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-other-discard-target-card continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let scope = u8::try_from(payload[1])
        .ok()
        .and_then(rule_api::TargetCardScope::from_tag)
        .ok_or_else(|| EffectError::EngineFault("bad target-card scope payload".into()))?;
    let skill = engine_api::SkillId::new(payload[2]);
    let target = PlayerId::new(selections[0]);
    if observer != player
        || cancelled
        || target == player
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || crate::query::discardable_target_cards(tx.state(), player, target, scope).is_empty()
    {
        return Err(EffectError::Illegal(
            "choose-other-discard-target-card context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ChooseDiscardTargetCard {
            player,
            target,
            scope,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_gain_discard_card_by_kind(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-gain-discard-card continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let kind = u8::try_from(payload[1])
        .ok()
        .and_then(CardKind::from_tag)
        .ok_or_else(|| EffectError::EngineFault("bad card-kind payload".into()))?;
    let skill = engine_api::SkillId::new(payload[2]);
    let card = CardId::new(selections[0]);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
    if observer != player
        || cancelled
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card| card.zone == discard && card.kind == kind)
    {
        return Err(EffectError::Illegal(
            "choose-gain-discard-card context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: hand,
            from: Some(discard),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_gain_own_lost_discard_card_this_turn(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 2 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-gain-own-lost-card continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let card = CardId::new(selections[0]);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
    let loss_count = tx.state().turn.card_loss_count(player);
    if observer != player
        || cancelled
        || tx.state().turn.current != player
        || !tx.state().players.get(player).is_some_and(|owner| {
            owner.alive && loss_count > 0 && loss_count <= owner.hp.max(0) as u32
        })
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .turn
            .lost_cards_for(player)
            .any(|lost| lost == card)
        || !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card| card.zone == discard)
    {
        return Err(EffectError::Illegal(
            "choose-gain-own-lost-card context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: hand,
            from: Some(discard),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_move_field_card_to(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-move-field-card continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let card = CardId::new(selections[0]);
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !crate::query::movable_field_cards_to(tx.state(), target).contains(&card)
    {
        return Err(EffectError::Illegal(
            "choose-move-field-card context changed".into(),
        ));
    }
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("selected field card is missing".into()))?;
    let from = card_state.zone;
    let from_kind = tx
        .state()
        .zones
        .get(from)
        .map(|zone| zone.kind)
        .ok_or_else(|| EffectError::Illegal("selected field card zone is missing".into()))?;
    let to_kind = match from_kind {
        ZoneKind::Equip => ZoneKind::Equip,
        ZoneKind::Judge => ZoneKind::Judge,
        _ => {
            return Err(EffectError::Illegal(
                "selected card is no longer in a field zone".into(),
            ));
        }
    };
    let to = tx
        .state()
        .zones
        .find(to_kind, target)
        .ok_or_else(|| EffectError::Illegal("field-card destination zone is missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn decode_optional_hand_distribution_plan(
    payload: &[u32],
) -> Result<
    (
        u32,
        PlayerId,
        u32,
        engine_api::SkillId,
        Vec<CardId>,
        Vec<PlayerId>,
        &[u32],
    ),
    EffectError,
> {
    if payload.len() < 5 {
        return Err(EffectError::EngineFault(
            "truncated optional hand-distribution payload".into(),
        ));
    }
    let stage = payload[0];
    let player = PlayerId::new(payload[1]);
    let give_cap = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    let plan_len = payload[4] as usize;
    let plan_end = 5usize
        .checked_add(plan_len.saturating_mul(2))
        .ok_or_else(|| {
            EffectError::EngineFault("oversized optional hand-distribution payload".into())
        })?;
    if plan_end > payload.len() || plan_len > give_cap as usize {
        return Err(EffectError::EngineFault(
            "invalid optional hand-distribution plan length".into(),
        ));
    }
    let mut cards = Vec::with_capacity(plan_len);
    let mut targets = Vec::with_capacity(plan_len);
    for pair in payload[5..plan_end].chunks_exact(2) {
        cards.push(CardId::new(pair[0]));
        targets.push(PlayerId::new(pair[1]));
    }
    Ok((
        stage,
        player,
        give_cap,
        skill,
        cards,
        targets,
        &payload[plan_end..],
    ))
}

pub(crate) fn resolve_draw_then_optionally_distribute_hand_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let (stage, player, give_cap, skill, mut cards, mut targets, tail) =
        decode_optional_hand_distribution_plan(payload)?;
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if observer != player
        || give_cap == 0
        || give_cap > u8::MAX as u32
        || !skill_enabled
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "optional hand-distribution choice context changed".into(),
        ));
    }

    let current: std::collections::BTreeSet<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let mut planned = std::collections::BTreeSet::new();
    if !cards
        .iter()
        .all(|card| planned.insert(card.raw()) && current.contains(&card.raw()))
        || targets.iter().any(|target| {
            *target == player
                || !tx
                    .state()
                    .players
                    .get(*target)
                    .is_some_and(|participant| participant.alive)
        })
    {
        return Err(EffectError::Illegal(
            "optional hand-distribution plan became stale".into(),
        ));
    }

    match stage {
        0 if tail.is_empty() => {
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled hand-distribution choice contains cards".into(),
                    ));
                }
                return run_effects(
                    tx,
                    vec![EffectRequest::DrawThenOptionallyDistributeHandCards {
                        player,
                        draw_count: 0,
                        give_cap,
                        cards,
                        targets,
                        skill,
                        ready_to_choose: true,
                        finish: true,
                    }],
                )
                .map(|_| ());
            }
            let remaining = give_cap as usize - cards.len();
            let mut selected = std::collections::BTreeSet::new();
            if selections.is_empty()
                || selections.len() > remaining
                || !selections.iter().all(|card| {
                    selected.insert(*card) && current.contains(card) && !planned.contains(card)
                })
            {
                return Err(EffectError::Illegal(
                    "optional hand-distribution card selection is invalid".into(),
                ));
            }
            let mut recipients: Vec<_> = tx
                .state()
                .players
                .iter_live()
                .filter(|target| target.alive && target.id != player)
                .map(|target| target.id.raw())
                .collect();
            recipients.sort_unstable();
            recipients.dedup();
            if recipients.is_empty() {
                return Err(EffectError::Illegal(
                    "optional hand distribution has no recipient".into(),
                ));
            }
            let mut next = vec![1, player.raw(), give_cap, skill.raw(), cards.len() as u32];
            for (card, target) in cards.iter().zip(&targets) {
                next.push(card.raw());
                next.push(target.raw());
            }
            next.push(selections.len() as u32);
            next.extend_from_slice(selections);
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                recipients,
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::DrawThenOptionallyDistributeHandCards,
                next,
            )
            .map(|_| ())
        }
        1 => {
            if cancelled || selections.len() != 1 || tail.is_empty() {
                return Err(EffectError::Illegal(
                    "optional hand-distribution recipient is required".into(),
                ));
            }
            let selected_len = tail[0] as usize;
            if selected_len == 0
                || tail.len() != selected_len + 1
                || selected_len > give_cap as usize - cards.len()
            {
                return Err(EffectError::EngineFault(
                    "invalid optional hand-distribution selected-card payload".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "optional hand-distribution recipient became stale".into(),
                ));
            }
            let mut selected = std::collections::BTreeSet::new();
            if !tail[1..].iter().all(|card| {
                selected.insert(*card) && current.contains(card) && !planned.contains(card)
            }) {
                return Err(EffectError::Illegal(
                    "optional hand-distribution cards became stale".into(),
                ));
            }
            for card in &tail[1..] {
                cards.push(CardId::new(*card));
                targets.push(target);
            }
            run_effects(
                tx,
                vec![EffectRequest::DrawThenOptionallyDistributeHandCards {
                    player,
                    draw_count: 0,
                    give_cap,
                    cards,
                    targets,
                    skill,
                    ready_to_choose: true,
                    finish: false,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown optional hand-distribution stage".into(),
        )),
    }
}
