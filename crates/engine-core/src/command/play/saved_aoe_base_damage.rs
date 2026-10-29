//! Command operations for `apply_deferred_trick` and related helpers.

use super::*;

pub(crate) fn saved_aoe_base_damage(args: &[u32], target_count: usize) -> u32 {
    let extension_at = 1usize.saturating_add(target_count);
    if args.get(extension_at) == Some(&SAVED_CARD_BASE_DAMAGE_MAGIC) {
        1u32.saturating_add(*args.get(extension_at + 1).unwrap_or(&0))
    } else {
        1
    }
}

pub(crate) fn optional_trick_targets(tx: &Transaction, args: &[u32]) -> Option<Vec<PlayerId>> {
    let len = *args.first()? as usize;
    Some(
        args.get(1..1 + len)?
            .iter()
            .map(|raw| PlayerId::new(*raw))
            .filter(|player| {
                tx.state()
                    .players
                    .get(*player)
                    .map(|state| state.alive)
                    .unwrap_or(false)
            })
            .collect(),
    )
}

/// Collect card tokens from target zones for 过河/顺手.
pub(crate) fn target_zone_card_tokens(
    tx: &Transaction,
    target: PlayerId,
    hand: bool,
    equip: bool,
    judge: bool,
) -> Vec<u32> {
    let mut out = Vec::new();
    if hand {
        out.extend(tx.state().hand_cards(target).iter().map(|c| c.raw()));
    }
    if equip {
        if let Some(eq) = tx.state().zones.find(ZoneKind::Equip, target) {
            if let Some(z) = tx.state().zones.get(eq) {
                out.extend(z.cards.iter().map(|c| c.raw()));
            }
        }
    }
    if judge {
        if let Some(jz) = tx.state().zones.find(ZoneKind::Judge, target) {
            if let Some(z) = tx.state().zones.get(jz) {
                out.extend(z.cards.iter().map(|c| c.raw()));
            }
        }
    }
    out
}

pub(crate) fn discardable_target_zone_card_tokens(
    tx: &Transaction,
    actor: PlayerId,
    target: PlayerId,
) -> Vec<u32> {
    target_zone_card_tokens(tx, target, true, true, true)
        .into_iter()
        .filter(|raw| {
            !crate::query::card_discard_protected_from(tx.state(), actor, target, CardId::new(*raw))
        })
        .collect()
}

/// Equipment-typed cards in a player's hand/equip zones (上游
/// `hasCards("he", { type: "equip" })`), used by 敕令's discard-equipment option.
pub(crate) fn equip_typed_zone_cards(tx: &Transaction, player: PlayerId) -> Vec<u32> {
    target_zone_card_tokens(tx, player, true, true, false)
        .into_iter()
        .filter(|raw| {
            tx.state()
                .cards
                .get(CardId::new(*raw))
                .is_some_and(|card| card.kind.is_equip())
        })
        .collect()
}

pub(crate) fn sha_in_range(tx: &Transaction, from: PlayerId, to: PlayerId) -> bool {
    if crate::query::card_distance_unlimited_for(tx.state(), from) {
        return true;
    }
    let mut ctx = QueryContext::new();
    let dist = query(tx.state(), &QueryRequest::Distance { from, to }, &mut ctx);
    let range = query(
        tx.state(),
        &QueryRequest::AttackRange { player: from },
        &mut ctx,
    );
    matches!(
        (dist, range),
        (QueryResponse::Int(d), QueryResponse::Int(r)) if d <= r
    )
}

pub(crate) fn prompted_sha_source(
    state: &crate::state::GameState,
    player: PlayerId,
    card: CardId,
) -> Option<(CardKind, Option<(SkillId, crate::content::ViewAsDef)>)> {
    let participant = state.players.get(player)?;
    if !participant.alive || participant.dying {
        return None;
    }
    let card_state = state.cards.get(card)?;
    if crate::query::card_color_locked_for_card(state, player, card) {
        return None;
    }
    if state.playable_hand_cards(player).contains(&card) {
        let kind = crate::query::effective_card_kind(state, player, card_state);
        if kind.is_sha() && !crate::query::card_use_cancelled_for_card(state, player, card) {
            return Some((kind, None));
        }
    }
    for skill in &participant.skills {
        let Some(instance) = state.skills.get(*skill) else {
            continue;
        };
        if !state.skill_effectively_enabled(instance) {
            continue;
        }
        for rule in state.content.view_as_defs_for_skill(instance.def) {
            if rule.as_kind != "sha"
                || rule.source_zone.as_deref() == Some("none")
                || !matches!(rule.window.as_str(), "any" | "play_phase")
                || rule
                    .uses_per_turn
                    .is_some_and(|limit| instance.use_count >= u32::from(limit))
                || rule
                    .branch
                    .is_some_and(|branch| branch != instance.view_as_branch)
                || (rule.once_per_round
                    && instance.view_as_use_round == state.turn.round
                    && rule.branch.is_some_and(|branch| {
                        instance.view_as_used_branches & (1u32 << branch) != 0
                    }))
                || !crate::query::view_as_source_zone_matches(
                    state,
                    player,
                    rule.source_zone.as_deref(),
                    card_state,
                )
                || !crate::query::view_as_filter_matches(state, player, &rule.filter, card_state)
                || crate::query::card_use_cancelled_for_logical_kind(
                    state,
                    player,
                    card_state,
                    CardKind::Sha,
                )
            {
                continue;
            }
            return Some((CardKind::Sha, Some((*skill, rule))));
        }
    }
    None
}

/// Physical cards the player can currently supply as a prompted logical Sha.
pub(crate) fn prompted_sha_source_cards(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<CardId> {
    let mut cards: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| state.zones.find(kind, player))
        .flat_map(|zone| {
            state
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .filter(|card| prompted_sha_source(state, player, *card).is_some())
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn twtanfeng_sha_source_is_owned(
    state: &crate::state::GameState,
    player: PlayerId,
    card: CardId,
) -> bool {
    let Some(card_state) = state.cards.get(card) else {
        return false;
    };
    let in_hand_or_equip = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| state.zones.find(kind, player))
        .any(|zone| zone == card_state.zone);
    in_hand_or_equip
        && card_state.owner == player
        && state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && !participant.dying)
        && !crate::query::card_use_cancelled_for_logical_kind(
            state,
            player,
            card_state,
            CardKind::Sha,
        )
}

/// Physical hand/equipment cards 探锋 may convert to Sha against `required_target`.
pub(crate) fn twtanfeng_sha_source_cards(
    tx: &Transaction,
    player: PlayerId,
    required_target: PlayerId,
) -> Vec<CardId> {
    let mut cards: Vec<_> = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, player))
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default()
        })
        .filter(|card| !twtanfeng_sha_targets(tx, player, required_target, *card).is_empty())
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

/// Legal normal-distance Sha targets for one 探锋 physical source; empty unless the owner is legal.
pub(crate) fn twtanfeng_sha_targets(
    tx: &Transaction,
    player: PlayerId,
    required_target: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    if !twtanfeng_sha_source_is_owned(tx.state(), player, card) {
        return Vec::new();
    }
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    let legal: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            validate_explicit_target_policy(tx, player, card_state, CardKind::Sha, &[*target])
                .is_ok()
                && validate_sha_target_with_options(tx, player, card, *target, false).is_ok()
        })
        .collect();
    legal
        .contains(&required_target)
        .then_some(legal)
        .unwrap_or_default()
}

/// Use one 探锋 source as Sha after revalidating its mandatory owner target and normal range.
pub(crate) fn play_twtanfeng_sha_source(
    tx: &mut Transaction,
    player: PlayerId,
    required_target: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let legal = twtanfeng_sha_targets(tx, player, required_target, card);
    let valid_shape = if crate::query::sha_targets_all_for(tx.state(), player) {
        targets == legal
    } else {
        !targets.is_empty()
            && targets.contains(&required_target)
            && targets.len() <= crate::query::sha_target_limit_for(tx.state(), player) as usize
            && targets.iter().all(|target| legal.contains(target))
            && targets
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == targets.len()
    };
    if !valid_shape {
        return Err(EffectError::Illegal("invalid Tanfeng Sha targets".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no Tanfeng user hand".into()))?;
    let card_state = tx
        .state()
        .cards
        .get(card)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("Tanfeng source disappeared".into()))?;
    let borrowed = crate::query::borrowed_hand_source(tx.state(), player, card).is_some();
    let source_was_hand = card_state.zone == hand || borrowed;
    if !source_was_hand {
        crate::effect::move_one_card(tx, card, card_state.zone, hand, player, false)?;
    }
    play_as_kind(
        tx,
        player,
        card,
        CardKind::Sha,
        targets,
        true,
        false,
        true,
        source_was_hand,
    )
}

pub(crate) fn oldjiefan_sha_spec(
    state: &crate::state::GameState,
    player: PlayerId,
    current: PlayerId,
    card: CardId,
) -> Option<(bool, Vec<PlayerId>)> {
    let (_, view_as) = prompted_sha_source(state, player, card)?;
    let card_state = state.cards.get(card)?;
    if current.is_none()
        || current == player
        || crate::query::card_targets_self_only_for(state, player)
        || crate::query::card_target_cancelled_for_logical_kind(
            state,
            player,
            current,
            card_state,
            CardKind::Sha,
        )
    {
        return None;
    }
    let ignore_distance = view_as
        .as_ref()
        .is_some_and(|(_, rule)| rule.ignore_distance);
    let target_legal = |target: PlayerId| {
        target != player
            && state
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            && !crate::query::card_target_cancelled_for_logical_kind(
                state,
                player,
                target,
                card_state,
                CardKind::Sha,
            )
            && (ignore_distance
                || crate::query::sha_distance_unlimited_for(state, player)
                || crate::query::card_distance_unlimited_for_target(state, player, card, target)
                || crate::query::player_in_attack_range(state, player, target))
    };
    if !target_legal(current) {
        return None;
    }
    let targets = if crate::query::sha_targets_all_for(state, player) {
        state
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| target_legal(*target))
            .collect()
    } else if crate::query::sha_target_limit_for(state, player) > 0 {
        vec![current]
    } else {
        return None;
    };
    targets
        .contains(&current)
        .then_some((ignore_distance, targets))
}

/// Physical cards that can supply old 解烦's Sha against the current turn player.
pub(crate) fn oldjiefan_sha_source_cards(
    state: &crate::state::GameState,
    player: PlayerId,
    current: PlayerId,
) -> Vec<CardId> {
    prompted_sha_source_cards(state, player)
        .into_iter()
        .filter(|card| oldjiefan_sha_spec(state, player, current, *card).is_some())
        .collect()
}

/// Use old 解烦's selected physical or view-as Sha through the ordinary response pipeline.
pub(crate) fn play_oldjiefan_sha_source(
    tx: &mut Transaction,
    player: PlayerId,
    current: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let (ignore_distance, targets) = oldjiefan_sha_spec(tx.state(), player, current, card)
        .ok_or_else(|| EffectError::Illegal("old Jiefan Sha source became illegal".into()))?;
    let (_, view_as) = prompted_sha_source(tx.state(), player, card)
        .ok_or_else(|| EffectError::Illegal("old Jiefan Sha source disappeared".into()))?;
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("missing old Jiefan Sha source".into()))?;
    let kind = view_as.as_ref().map_or_else(
        || crate::query::effective_card_kind(tx.state(), player, card_state),
        |_| CardKind::Sha,
    );
    validate_explicit_target_policy(tx, player, card_state, kind, &targets)?;
    validate_sha_targets_with_options(tx, player, card, &targets, ignore_distance)?;
    play_prompted_sha_source_impl(
        tx,
        player,
        card,
        &targets,
        ignore_distance,
        true,
        rule_api::damage_card_flag::OLDJIEFAN,
    )
}

/// Legal targets for a Xinzhongyong-prompted Sha. Ordinary target policies apply, but ordinary
/// user distance is replaced by the original skill owner's attack range.
pub(crate) fn prompted_sha_targets(
    tx: &Transaction,
    owner: PlayerId,
    player: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    let Some((kind, _)) = prompted_sha_source(tx.state(), player, card) else {
        return Vec::new();
    };
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| *target != owner)
        .filter(|target| sha_in_range(tx, owner, *target))
        .filter(|target| {
            validate_explicit_target_policy(tx, player, card_state, kind, &[*target]).is_ok()
                && validate_sha_target_with_options(tx, player, card, *target, true).is_ok()
        })
        .collect()
}

pub(crate) fn clockwise_living_others(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<PlayerId> {
    let order = state.players.alive_by_seat();
    let Some(start) = order.iter().position(|candidate| *candidate == player) else {
        return Vec::new();
    };
    (1..order.len())
        .map(|offset| order[(start + offset) % order.len()])
        .collect()
}

/// Legal targets for a 乱武-prompted Sha. Only players at the minimum final distance from the
/// prompted user are eligible; ordinary Sha range and target policies still apply.
pub(crate) fn luanwu_sha_targets(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    let Some((kind, _)) = prompted_sha_source(tx.state(), player, card) else {
        return Vec::new();
    };
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    let candidates = clockwise_living_others(tx.state(), player);
    let distance = |target| {
        let mut context = QueryContext::new();
        match query(
            tx.state(),
            &QueryRequest::Distance {
                from: player,
                to: target,
            },
            &mut context,
        ) {
            QueryResponse::Int(value) => Some(value),
            _ => None,
        }
    };
    let Some(minimum) = candidates
        .iter()
        .filter_map(|target| distance(*target))
        .min()
    else {
        return Vec::new();
    };
    candidates
        .into_iter()
        .filter(|target| distance(*target) == Some(minimum))
        .filter(|target| {
            validate_explicit_target_policy(tx, player, card_state, kind, &[*target]).is_ok()
                && validate_sha_target_with_options(tx, player, card, *target, false).is_ok()
        })
        .collect()
}
