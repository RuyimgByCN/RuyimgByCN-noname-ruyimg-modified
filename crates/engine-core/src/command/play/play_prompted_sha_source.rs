//! Command operations for `play_prompted_sha_source` and related helpers.

use super::*;

/// Use one revalidated physical source as a logical Sha without consuming the ordinary use count.
pub(crate) fn play_prompted_sha_source(
    tx: &mut Transaction,
    owner: PlayerId,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let legal = prompted_sha_targets(tx, owner, player, card);
    play_prompted_sha_source_with_legal_targets(tx, player, card, targets, &legal)
}

/// Use one revalidated physical or view-as source for 乱武 without consuming the ordinary Sha
/// use count.
pub(crate) fn play_luanwu_sha_source(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let legal = luanwu_sha_targets(tx, player, card);
    play_prompted_sha_source_with_legal_targets(tx, player, card, targets, &legal)
}

/// List legal targets for a prompted physical or view-as Sha while ignoring ordinary distance.
pub(crate) fn prompted_sha_targets_unlimited(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| validate_sha_target_with_options(tx, player, card, *target, true).is_ok())
        .collect()
}

/// Legal targets for a prompted physical or one-card view-as Sha under ordinary distance and
/// use-count rules.
pub(crate) fn prompted_sha_targets_normal(
    tx: &Transaction,
    player: PlayerId,
    card: CardId,
) -> Vec<PlayerId> {
    let Some((kind, view_as)) = prompted_sha_source(tx.state(), player, card) else {
        return Vec::new();
    };
    let Some(card_state) = tx.state().cards.get(card) else {
        return Vec::new();
    };
    let ignore_distance = view_as
        .as_ref()
        .is_some_and(|(_, rule)| rule.ignore_distance);
    let within_use_count = crate::query::sha_play_phase_use_count(tx.state())
        < crate::query::sha_use_limit_for_card(tx.state(), player, card);
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            within_use_count
                || crate::query::sha_use_unlimited_for_target(tx.state(), player, *target)
        })
        .filter(|target| {
            validate_explicit_target_policy(tx, player, card_state, kind, &[*target]).is_ok()
                && validate_sha_target_with_options(tx, player, card, *target, ignore_distance)
                    .is_ok()
        })
        .collect()
}

/// Use a prompted physical or one-card view-as Sha with ordinary distance and use-count rules.
pub(crate) fn play_prompted_sha_source_normal(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let legal = prompted_sha_targets_normal(tx, player, card);
    if targets.is_empty()
        || targets.len() > crate::query::sha_target_limit_for(tx.state(), player) as usize
        || targets.iter().any(|target| !legal.contains(target))
        || targets
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != targets.len()
    {
        return Err(EffectError::Illegal(
            "invalid ordinary prompted Sha targets".into(),
        ));
    }
    let ignore_distance = prompted_sha_source(tx.state(), player, card)
        .and_then(|(_, view_as)| view_as.map(|(_, rule)| rule.ignore_distance))
        .unwrap_or(false);
    play_prompted_sha_source_impl(tx, player, card, targets, ignore_distance, false, 0)
}

/// Use one revalidated physical or view-as Sha against arbitrary legal targets while ignoring
/// ordinary distance and use count.
pub(crate) fn play_prompted_sha_source_unlimited(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> Result<(), EffectError> {
    let legal = prompted_sha_targets_unlimited(tx, player, card);
    play_prompted_sha_source_with_legal_targets(tx, player, card, targets, &legal)
}

/// Force one exact physical Sha from the discard pile, with only the supplied targets unable to
/// respond. Used by 破袭; ordinary distance and use count are ignored.
pub(crate) fn play_discard_sha_unlimited(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    unrespondable: &[PlayerId],
) -> Result<(), EffectError> {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("shared discard pile missing".into()))?;
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("discarded Sha missing".into()))?
        .clone();
    let kind = crate::query::effective_card_kind(tx.state(), player, &card_state);
    let legal = prompted_sha_targets_unlimited(tx, player, card);
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|player| player.alive && !player.dying)
        || card_state.zone != discard
        || !kind.is_sha()
        || targets.is_empty()
        || targets.len() > crate::query::sha_target_limit_for(tx.state(), player) as usize
        || targets.iter().any(|target| !legal.contains(target))
        || targets
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != targets.len()
        || crate::query::card_use_cancelled_for_card(tx.state(), player, card)
    {
        return Err(EffectError::Illegal("invalid 破袭 Sha use".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Sha user hand missing".into()))?;
    crate::effect::move_one_card(tx, card, discard, hand, player, false)?;
    play_as_kind_with_extra_damage_flags(
        tx,
        player,
        card,
        kind,
        targets,
        true,
        false,
        unrespondable,
        true,
        false,
        0,
        &[],
        0,
    )
}

/// Use one exact hand Sha forced by 备诛, ignoring distance and the ordinary use count.
pub(crate) fn play_beizhu_sha_source(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    if !beizhu_sha_usable(tx, player, target, card) {
        return Err(EffectError::Illegal(
            "Beizhu Sha is no longer usable on the skill owner".into(),
        ));
    }
    play_prompted_sha_source_impl(
        tx,
        player,
        card,
        &[target],
        true,
        true,
        rule_api::damage_card_flag::BEIZHU,
    )
}

pub(crate) fn beizhu_sha_usable(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
) -> bool {
    let Some(hand) = tx.state().zones.find(ZoneKind::Hand, player) else {
        return false;
    };
    let Some(card_state) = tx.state().cards.get(card) else {
        return false;
    };
    if card_state.zone != hand
        || card_state.owner != player
        || !crate::query::effective_card_kind(tx.state(), player, card_state).is_sha()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|state| state.alive && !state.dying)
        || crate::query::card_color_locked_for_card(tx.state(), player, card)
        || crate::query::card_use_cancelled_for_card(tx.state(), player, card)
    {
        return false;
    }
    validate_explicit_target_policy(tx, player, card_state, CardKind::Sha, &[target]).is_ok()
        && validate_sha_targets_with_options(tx, player, card, &[target], true).is_ok()
}

pub(crate) fn play_prompted_sha_source_with_legal_targets(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    legal: &[PlayerId],
) -> Result<(), EffectError> {
    if targets.is_empty()
        || targets.len() > crate::query::sha_target_limit_for(tx.state(), player) as usize
        || targets.iter().any(|target| !legal.contains(target))
        || targets
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != targets.len()
    {
        return Err(EffectError::Illegal(
            "invalid prompted Sha target selection".into(),
        ));
    }
    play_prompted_sha_source_impl(tx, player, card, targets, true, true, 0)
}

pub(crate) fn play_prompted_sha_source_impl(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    ignore_distance: bool,
    ignore_use_count: bool,
    extra_damage_flags: u8,
) -> Result<(), EffectError> {
    let (kind, view_as) = prompted_sha_source(tx.state(), player, card)
        .ok_or_else(|| EffectError::Illegal("prompted Sha source is no longer legal".into()))?;
    if view_as.is_none() {
        return handle_card_from_hand_with_policies_and_flags(
            tx,
            player,
            card,
            targets,
            ignore_distance,
            ignore_use_count,
            true,
            extra_damage_flags,
        );
    }
    let (skill, rule) = view_as.unwrap();
    let card_state = tx
        .state()
        .cards
        .get(card)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("missing prompted Sha source".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no prompted Sha user hand".into()))?;
    let borrowed = crate::query::borrowed_hand_source(tx.state(), player, card).is_some();
    let source_was_hand = card_state.zone == hand || borrowed;
    if !source_was_hand {
        crate::effect::move_one_card(tx, card, card_state.zone, hand, player, false)?;
    }
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def)
        .ok_or_else(|| EffectError::Illegal("missing prompted Sha view-as skill".into()))?;
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.use_count = instance.use_count.saturating_add(1);
    }
    let source_suit = crate::query::effective_card_suit(
        tx.state(),
        player,
        tx.state()
            .cards
            .get(card)
            .expect("prompted source preserved"),
    );
    let force_unrespondable =
        rule.red_unrespondable && matches!(source_suit, Suit::Heart | Suit::Diamond);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, card.raw() as i32);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
    tx.state_mut().turn.set_flag(
        crate::state::turn_flag::VIEW_AS_LOGICAL_KIND,
        i32::from(kind.tag()),
    );
    let result = (|| {
        run_effects(
            tx,
            vec![EffectRequest::EmitRuleEvent {
                code: 120,
                payload: vec![
                    skill.raw(),
                    skill_def.raw(),
                    player.raw(),
                    card.raw(),
                    kind.tag() as u32,
                ],
            }],
        )?;
        play_as_kind_with_extra_damage_flags(
            tx,
            player,
            card,
            kind,
            targets,
            ignore_distance,
            force_unrespondable,
            &[],
            ignore_use_count,
            source_was_hand,
            extra_damage_flags,
            &[],
            0,
        )
    })();
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_CARD, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_SKILL, 0);
    tx.state_mut()
        .turn
        .set_flag(crate::state::turn_flag::VIEW_AS_LOGICAL_KIND, 0);
    result
}

/// Cards that can answer 杀 as 闪 (natural 闪 or view_as 闪).
pub(crate) fn response_shan_tokens(state: &crate::state::GameState, player: PlayerId) -> Vec<u32> {
    if crate::query::hand_card_use_response_cancelled_for(state, player) {
        return super::super::skill::source_free_response_tokens(state, player, CardKind::Shan);
    }
    response_tokens(state, player, "shan", |kind| kind == CardKind::Shan)
}

/// Physical 诱敌深入 cards usable before an incoming respondable Sha takes effect.
pub(crate) fn response_youdishenru_tokens(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<u32> {
    if crate::query::card_response_cancelled_for(state, player)
        || crate::query::hand_card_use_response_cancelled_for(state, player)
    {
        return Vec::new();
    }
    response_card_tokens(state, player, "youdishenru", |kind| {
        kind == CardKind::TrickYoudishenru
    })
    .into_iter()
    .filter(|token| !crate::query::card_use_cancelled_for_card(state, player, CardId::new(*token)))
    .collect()
}

/// Cards that can answer 南蛮/决斗 as 杀 (natural 杀 or view_as 杀).
pub(crate) fn response_sha_tokens(state: &crate::state::GameState, player: PlayerId) -> Vec<u32> {
    if crate::query::card_response_cancelled_for(state, player)
        || crate::query::sha_response_cancelled_for(state, player)
    {
        return Vec::new();
    }
    if crate::query::hand_card_use_response_cancelled_for(state, player) {
        return super::super::skill::source_free_response_tokens(state, player, CardKind::Sha);
    }
    response_tokens(state, player, "sha", CardKind::is_sha)
}

pub(crate) fn response_tokens(
    state: &crate::state::GameState,
    player: PlayerId,
    as_kind: &str,
    natural: impl Fn(CardKind) -> bool,
) -> Vec<u32> {
    let mut out = response_card_tokens(state, player, as_kind, natural);
    let dynamic_kind = match as_kind {
        "sha" => CardKind::Sha,
        "shan" => CardKind::Shan,
        "wuxie" => CardKind::TrickWuxie,
        _ => CardKind::Other,
    };
    if dynamic_kind != CardKind::Other {
        out.extend(super::super::skill::source_free_response_tokens(
            state,
            player,
            dynamic_kind,
        ));
        out.extend(super::super::skill::multi_card_response_tokens(
            state,
            player,
            dynamic_kind,
        ));
    }
    out
}

pub(crate) fn response_card_tokens(
    state: &crate::state::GameState,
    player: PlayerId,
    as_kind: &str,
    natural: impl Fn(CardKind) -> bool,
) -> Vec<u32> {
    let mut out = Vec::new();
    for cid in state.playable_hand_cards(player) {
        let Some(c) = state.cards.get(cid) else {
            continue;
        };
        let natural_kind = crate::query::effective_card_kind(state, player, c);
        if !crate::query::card_color_locked_for_card(state, player, cid)
            && !crate::query::response_color_locked_for_current_card(state, player, cid)
            && (natural(natural_kind)
                && !crate::query::card_response_cancelled_for_logical_kind(
                    state,
                    player,
                    c,
                    natural_kind,
                )
                || view_as_response_skill(state, player, c, as_kind).is_some())
        {
            out.push(cid.raw());
        }
    }
    if let Some(equip) = state.zones.find(ZoneKind::Equip, player) {
        if let Some(zone) = state.zones.get(equip) {
            for cid in &zone.cards {
                if state.cards.get(*cid).is_some_and(|card| {
                    !crate::query::card_color_locked_for_card(state, player, *cid)
                        && !crate::query::response_color_locked_for_current_card(
                            state, player, *cid,
                        )
                        && view_as_response_skill(state, player, card, as_kind).is_some()
                }) {
                    out.push(cid.raw());
                }
            }
        }
    }
    out
}

pub(crate) fn view_as_response_skill(
    state: &crate::state::GameState,
    player: PlayerId,
    c: &crate::state::Card,
    as_kind: &str,
) -> Option<engine_api::SkillId> {
    let Some(logical_kind) = (match as_kind {
        "sha" => Some(CardKind::Sha),
        "shan" => Some(CardKind::Shan),
        "wuxie" => Some(CardKind::TrickWuxie),
        _ => None,
    }) else {
        return None;
    };
    let Some(p) = state.players.get(player) else {
        return None;
    };
    for sid in &p.skills {
        let Some(inst) = state.skills.get(*sid) else {
            continue;
        };
        if !state.skill_effectively_enabled(inst) {
            continue;
        }
        for v in state.content.view_as_defs_for_skill(inst.def) {
            if v.as_kind != as_kind || !matches!(v.window.as_str(), "any" | "response") {
                continue;
            }
            if crate::query::view_as_source_zone_matches(state, player, v.source_zone.as_deref(), c)
                && crate::query::view_as_filter_matches(state, player, &v.filter, c)
                && !crate::query::card_response_cancelled_for_logical_kind(
                    state,
                    player,
                    c,
                    logical_kind,
                )
            {
                return Some(*sid);
            }
        }
    }
    None
}
