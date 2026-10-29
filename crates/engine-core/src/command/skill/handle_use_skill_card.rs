//! Command operations for `ddd_fengzheng_provider_exists` and related helpers.

use super::*;

/// Use a package-declared source-free virtual card conversion during the play phase.
pub(crate) fn handle_use_skill_card(
    tx: &mut Transaction,
    actor: PlayerId,
    skill: SkillId,
    card_kind: u8,
    targets: &[PlayerId],
    equip_slot: u8,
) -> Result<(), EffectError> {
    if tx.state().revision == 0 {
        return Err(EffectError::Illegal("not started".into()));
    }
    if tx.state().has_pending_choice() {
        return Err(EffectError::Illegal("choice pending".into()));
    }
    if tx.state().turn.current != actor || tx.state().turn.phase != Phase::Play {
        return Err(EffectError::Illegal(
            "source-free conversion requires the play phase".into(),
        ));
    }
    let kind = CardKind::from_tag(card_kind)
        .ok_or_else(|| EffectError::Illegal("unknown virtual card kind".into()))?;
    let inst = tx
        .state()
        .skills
        .get(skill)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("unknown skill".into()))?;
    if inst.owner != actor || !tx.state().skill_effectively_enabled(&inst) {
        return Err(EffectError::Illegal(
            "source-free conversion skill unavailable".into(),
        ));
    }
    let rule = source_free_rule_for(tx.state(), &inst, kind, false)
        .ok_or_else(|| EffectError::Illegal("source-free conversion unavailable".into()))?;
    if let Some(group) = delegated_response_group(&rule) {
        super::super::play::validate_source_free_sha_use(tx, actor, targets)?;
        let mut action_payload = vec![targets.len() as u32];
        action_payload.extend(targets.iter().map(|target| target.raw()));
        return super::super::resolve_choice::begin_delegated_group_response(
            tx,
            actor,
            skill,
            kind,
            6,
            &action_payload,
            group,
        );
    }
    let slot = validate_source_free_cost(tx, actor, &rule, equip_slot)?;

    record_source_free_use(tx, skill, &rule)?;
    let mut resume = vec![
        actor.raw(),
        skill.raw(),
        kind.tag() as u32,
        targets.len() as u32,
    ];
    resume.extend(targets.iter().map(|target| target.raw()));
    {
        let resolution = &mut tx.state_mut().resolution;
        resolution.resume_kind = crate::state::ContinuationKind::SourceFreeViewAsPlay;
        resolution.resume_payload = resume.clone();
    }
    match pay_source_free_cost(tx, actor, skill, kind, &rule, slot) {
        Ok(()) => {
            tx.state_mut().resolution.resume_kind = crate::state::ContinuationKind::None;
            tx.state_mut().resolution.resume_payload.clear();
            resume_source_free_play(tx, &resume)
        }
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Err(error) => {
            tx.state_mut().resolution.resume_kind = crate::state::ContinuationKind::None;
            tx.state_mut().resolution.resume_payload.clear();
            Err(error)
        }
    }
}

/// Continue proactive source-free card use after cost-trigger choices finish.
pub(crate) fn resume_source_free_play(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    let actor = PlayerId::new(*payload.first().unwrap_or(&0));
    let skill = SkillId::new(*payload.get(1).unwrap_or(&0));
    let kind = payload
        .get(2)
        .and_then(|tag| CardKind::from_tag(*tag as u8))
        .ok_or_else(|| EffectError::EngineFault("bad source-free play kind".into()))?;
    let target_count = *payload.get(3).unwrap_or(&0) as usize;
    let targets: Vec<PlayerId> = payload
        .iter()
        .skip(4)
        .take(target_count)
        .copied()
        .map(PlayerId::new)
        .collect();
    let inst = tx
        .state()
        .skills
        .get(skill)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("unknown skill".into()))?;
    use_materialized_virtual_card(tx, actor, skill, &inst, kind, &targets)
}

pub(crate) fn source_free_rule_for(
    state: &crate::state::GameState,
    inst: &crate::state::SkillInstance,
    kind: CardKind,
    response: bool,
) -> Option<ViewAsDef> {
    state
        .content
        .view_as_defs_for_skill(inst.def)
        .into_iter()
        .find(|rule| {
            let delegated_group = delegated_response_group(rule);
            let delegated = delegated_group.is_some();
            ((delegated && map_as_kind(&rule.as_kind) == Some(kind))
                || (rule.source_zone.as_deref() == Some("none")
                    && (matches!(
                        rule.cost.as_deref(),
                        Some(
                            "guhuo"
                                | "face_down_hand"
                                | "dczhaowen"
                                | "swap_lower_hand"
                                | "draw_one_outside_turn_empty_hand"
                                | "hand_equip_mark"
                                | "lose_max_hp_1"
                                | "all_hand"
                                | "potdaozhuan"
                                | "dddshilie"
                                | "dddfusi"
                                | "turn_over"
                        )
                    ) || rule.branch == Some(inst.view_as_branch))
                    && source_free_family_contains(&rule.as_kind, kind)))
                && if response {
                    matches!(rule.window.as_str(), "any" | "response")
                } else {
                    matches!(rule.window.as_str(), "any" | "play_phase")
                }
                && (delegated
                    || rule.cost.as_deref() != Some("guhuo")
                    || !state.hand_cards(inst.owner).is_empty())
                && (delegated
                    || rule.cost.as_deref() != Some("face_down_hand")
                    || face_down_view_as_kind_available(state, inst, kind))
                && (delegated
                    || rule.cost.as_deref() != Some("dczhaowen")
                    || dczhaowen_view_as_kind_available(state, inst, kind))
                && (delegated
                    || !rule.once_per_round
                    || inst.view_as_use_round != state.turn.round
                    || rule
                        .branch
                        .is_none_or(|branch| inst.view_as_used_branches & (1u32 << branch) == 0))
                && !rule
                    .uses_per_turn
                    .is_some_and(|limit| inst.use_count >= u32::from(limit))
                && (rule.cost.as_deref() != Some("swap_lower_hand")
                    || !swap_lower_hand_targets(state, inst.owner).is_empty())
                && (rule.cost.as_deref() != Some("draw_one_outside_turn_empty_hand")
                    || (state.turn.current != inst.owner
                        && state.hand_cards(inst.owner).is_empty()))
                && (rule.cost.as_deref() != Some("all_hand")
                    || !state.hand_cards(inst.owner).is_empty())
                && (rule.cost.as_deref() != Some("dddshilie")
                    || (matches!(kind, CardKind::Sha | CardKind::Shan)
                        && !dddshilie_sources(state, inst.owner).is_empty()
                        && dddshilie_threshold(state, inst.owner).is_some_and(|threshold| {
                            dddshilie_sources(state, inst.owner)
                                .iter()
                                .filter_map(|card| state.cards.get(*card))
                                .map(|card| {
                                    u16::from(crate::query::effective_card_number(
                                        state, inst.owner, card,
                                    ))
                                })
                                .sum::<u16>()
                                >= threshold
                        })))
                && (rule.cost.as_deref() != Some("dddfusi")
                    || !dddfusi_sources(state, inst.owner, inst.id, kind).is_empty())
                && (rule.cost.as_deref() != Some("potdaozhuan")
                    || (inst.potdaozhuan_available(
                        state.turn.round,
                        state.turn.current,
                        potdaozhuan_name_bit(kind),
                    ) && !potdaozhuan_cost_cards(state, inst.owner).is_empty()))
                && (rule.cost.as_deref() != Some("hand_equip_mark")
                    || (!hand_equip_view_as_sources(state, inst.owner).is_empty()
                        && rule.mark_cost.is_some_and(|mark| {
                            state.players.get(inst.owner).is_some_and(|player| {
                                player
                                    .marks
                                    .iter()
                                    .any(|(existing, count)| *existing == mark && *count > 0)
                            })
                        })))
                && delegated_group.is_none_or(|group| {
                    !delegated_response_candidates(state, inst.owner, group, kind).is_empty()
                        && (response
                            || inst.delegated_retry_lock_revision() != Some(state.revision))
                })
                && (rule.cost.as_deref() != Some("disable_equip_slot")
                    || state.players.get(inst.owner).is_some_and(|player| {
                        (1..=5).any(|tag| {
                            EquipSlot::from_tag(tag)
                                .is_some_and(|slot| player.equip_slot_enabled(slot))
                        })
                    }))
                && (rule.cost.as_deref() != Some("lose_max_hp_1")
                    || state
                        .players
                        .get(inst.owner)
                        .is_some_and(|player| player.alive && player.max_hp > 0))
        })
}

pub(crate) fn potdaozhuan_name_bit(kind: CardKind) -> u32 {
    let tag = if kind.is_sha() {
        CardKind::Sha.tag()
    } else {
        kind.tag()
    };
    (tag < 32).then_some(1u32 << tag).unwrap_or(0)
}

pub(crate) fn potdaozhuan_cost_cards(
    state: &crate::state::GameState,
    actor: PlayerId,
) -> Vec<CardId> {
    let mut owners = vec![actor];
    if state.turn.current != actor
        && state
            .players
            .get(state.turn.current)
            .is_some_and(|player| player.alive)
    {
        owners.push(state.turn.current);
    }
    let mut cards = Vec::new();
    for owner in owners {
        cards.extend(state.hand_cards(owner));
        if let Some(equip) = state.zones.find(ZoneKind::Equip, owner) {
            cards.extend(
                state
                    .zones
                    .get(equip)
                    .into_iter()
                    .flat_map(|zone| zone.cards.iter().copied()),
            );
        }
    }
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn source_free_post_draw(
    state: &crate::state::GameState,
    skill: SkillId,
    kind: CardKind,
) -> u8 {
    state
        .skills
        .get(skill)
        .into_iter()
        .flat_map(|instance| state.content.view_as_defs_for_skill(instance.def))
        .find(|rule| {
            rule.source_zone.as_deref() == Some("none")
                && source_free_family_contains(&rule.as_kind, kind)
                || rule.cost.as_deref() == Some("dddzhijie")
                    && response_rule_matches_kind(&rule, kind)
        })
        .map_or(0, |rule| rule.post_draw)
}

/// Deterministic opaque choice tokens for eligible source-free response conversions.
pub(crate) fn source_free_response_tokens(
    state: &crate::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> Vec<u32> {
    source_free_response_entries(state, player, kind)
        .into_iter()
        .map(|(token, _)| token)
        .collect()
}

/// Resolve an opaque source-free response token to its runtime skill instance.
pub(crate) fn source_free_response_skill(
    state: &crate::state::GameState,
    player: PlayerId,
    kind: CardKind,
    token: u32,
) -> Option<SkillId> {
    source_free_response_entries(state, player, kind)
        .into_iter()
        .find_map(|(candidate, skill)| (candidate == token).then_some(skill))
}

pub(crate) fn source_free_response_entries(
    state: &crate::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> Vec<(u32, SkillId)> {
    if matches!(kind, CardKind::Tao | CardKind::Jiu)
        && crate::query::card_save_cancelled_for_kind(state, player, kind)
    {
        return Vec::new();
    }
    if kind.is_sha()
        && (crate::query::card_response_cancelled_for(state, player)
            || crate::query::sha_response_cancelled_for(state, player))
    {
        return Vec::new();
    }
    if matches!(kind, CardKind::Shan | CardKind::TrickWuxie)
        && crate::query::card_response_cancelled_for(state, player)
    {
        return Vec::new();
    }
    let Some(owner) = state.players.get(player) else {
        return Vec::new();
    };
    let occupied: std::collections::BTreeSet<u32> = state
        .cards
        .sorted_live()
        .into_iter()
        .map(|card| card.id.raw())
        .chain([0, 1, crate::content::BAGUA_RESPONSE_TOKEN])
        .collect();
    let mut token = u32::MAX;
    let mut out = Vec::new();
    for skill_id in &owner.skills {
        let Some(inst) = state.skills.get(*skill_id) else {
            continue;
        };
        if !state.skill_effectively_enabled(inst) {
            continue;
        }
        let Some(rule) = source_free_rule_for(state, inst, kind, true) else {
            continue;
        };
        if delegated_response_group(&rule).is_some_and(|group| {
            delegated_response_candidates(state, player, group, kind).is_empty()
        }) {
            continue;
        }
        while occupied.contains(&token) || out.iter().any(|(used, _)| *used == token) {
            token = token.saturating_sub(1);
        }
        out.push((token, *skill_id));
        token = token.saturating_sub(1);
    }
    out
}

pub(crate) fn response_rule_matches_kind(rule: &ViewAsDef, kind: CardKind) -> bool {
    match rule.as_kind.as_str() {
        "sha" => kind.is_sha(),
        "shan" => kind == CardKind::Shan,
        "tao" => kind == CardKind::Tao,
        "jiu" => kind == CardKind::Jiu,
        "wuxie" => kind == CardKind::TrickWuxie,
        _ => false,
    }
}

pub(crate) fn multi_card_response_rule(
    state: &crate::state::GameState,
    player: PlayerId,
    skill: SkillId,
    kind: CardKind,
) -> Option<ViewAsDef> {
    let instance = state.skills.get(skill)?;
    (instance.owner == player && state.skill_effectively_enabled(instance))
        .then(|| {
            state
                .content
                .view_as_defs_for_skill(instance.def)
                .into_iter()
                .find(|rule| {
                    (rule.max_cards > 1
                        || rule.owner_hp_min_one_cards
                        || rule.owner_basic_uses_plus_one_cards)
                        && matches!(rule.window.as_str(), "any" | "response")
                        && response_rule_matches_kind(rule, kind)
                        && !rule
                            .uses_per_turn
                            .is_some_and(|limit| instance.use_count >= u32::from(limit))
                        && (rule.cost.as_deref() != Some("potqingyan")
                            || !potqingyan_has_tracked_hand_card(state, player, instance))
                })
        })
        .flatten()
}

pub(crate) fn multi_card_response_sources(
    state: &crate::state::GameState,
    player: PlayerId,
    rule: &ViewAsDef,
) -> Vec<u32> {
    if rule.cost.as_deref() == Some("dddzhijie") {
        return state
            .skill_pile_cards(player, crate::effect::DDD_DONGCHA_DEF)
            .into_iter()
            .filter(|card| {
                !crate::query::card_color_locked_for_card(state, player, *card)
                    && !crate::query::response_color_locked_for_current_card(state, player, *card)
            })
            .map(CardId::raw)
            .collect();
    }
    state
        .cards
        .sorted_live()
        .into_iter()
        .filter(|card| {
            !crate::query::card_color_locked_for_card(state, player, card.id)
                && !crate::query::response_color_locked_for_current_card(state, player, card.id)
                && view_as_source_matches(state, player, rule, card.id)
        })
        .map(|card| card.id.raw())
        .collect()
}

pub(crate) fn multi_card_response_entries(
    state: &crate::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> Vec<(u32, SkillId)> {
    if matches!(kind, CardKind::Tao | CardKind::Jiu)
        && crate::query::card_save_cancelled_for_kind(state, player, kind)
        || kind.is_sha()
            && (crate::query::card_response_cancelled_for(state, player)
                || crate::query::sha_response_cancelled_for(state, player))
        || matches!(kind, CardKind::Shan | CardKind::TrickWuxie)
            && crate::query::card_response_cancelled_for(state, player)
    {
        return Vec::new();
    }
    let Some(owner) = state.players.get(player) else {
        return Vec::new();
    };
    let occupied: std::collections::BTreeSet<u32> = state
        .cards
        .sorted_live()
        .into_iter()
        .map(|card| card.id.raw())
        .chain([0, 1, crate::content::BAGUA_RESPONSE_TOKEN])
        .chain(source_free_response_tokens(state, player, kind))
        .collect();
    let mut token = u32::MAX;
    let mut out = Vec::new();
    for skill in &owner.skills {
        let Some(rule) = multi_card_response_rule(state, player, *skill, kind) else {
            continue;
        };
        let sources = multi_card_response_sources(state, player, &rule);
        let (min_cards, _) = view_as_card_count(state, player, &rule);
        if sources.len() < usize::from(min_cards)
            || rule.cost.as_deref() == Some("dddzhijie")
                && !sources.iter().enumerate().any(|(index, first)| {
                    sources[index + 1..].iter().any(|second| {
                        let first = CardId::new(*first);
                        let second = CardId::new(*second);
                        let same = crate::query::effective_card_color_mask(state, player, first)
                            == crate::query::effective_card_color_mask(state, player, second);
                        (rule.as_kind == "shan") == same
                    })
                })
            || rule.same_suit
                && !sources.iter().any(|source| {
                    let Some(card) = state.cards.get(CardId::new(*source)) else {
                        return false;
                    };
                    let suit = crate::query::effective_card_suit(state, player, card);
                    sources
                        .iter()
                        .filter(|candidate| {
                            state
                                .cards
                                .get(CardId::new(**candidate))
                                .is_some_and(|candidate| {
                                    crate::query::effective_card_suit(state, player, candidate)
                                        == suit
                                })
                        })
                        .count()
                        >= usize::from(min_cards)
                })
        {
            continue;
        }
        while occupied.contains(&token) || out.iter().any(|(used, _)| *used == token) {
            token = token.saturating_sub(1);
        }
        out.push((token, *skill));
        token = token.saturating_sub(1);
    }
    out
}

pub(crate) fn multi_card_response_tokens(
    state: &crate::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> Vec<u32> {
    multi_card_response_entries(state, player, kind)
        .into_iter()
        .map(|(token, _)| token)
        .collect()
}

pub(crate) fn multi_card_response_skill(
    state: &crate::state::GameState,
    player: PlayerId,
    kind: CardKind,
    token: u32,
) -> Option<SkillId> {
    multi_card_response_entries(state, player, kind)
        .into_iter()
        .find_map(|(candidate, skill)| (candidate == token).then_some(skill))
}

pub(crate) fn begin_multi_card_response(
    tx: &mut Transaction,
    player: PlayerId,
    skill: SkillId,
    kind: CardKind,
    action: u32,
    action_payload: &[u32],
) -> Result<(), EffectError> {
    let rule = multi_card_response_rule(tx.state(), player, skill, kind)
        .ok_or_else(|| EffectError::Illegal("multi-card response conversion unavailable".into()))?;
    let options = multi_card_response_sources(tx.state(), player, &rule);
    if options.is_empty() {
        return Err(EffectError::Illegal(
            "multi-card response conversion has no source".into(),
        ));
    }
    let mut payload = vec![player.raw(), skill.raw(), kind.tag() as u32, action];
    payload.extend_from_slice(action_payload);
    let (min_cards, max_cards) = view_as_card_count(tx.state(), player, &rule);
    crate::effect::open_choice(
        tx,
        player,
        min_cards,
        max_cards,
        false,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::MultiCardViewAsResponse,
        payload,
    )?;
    Err(EffectError::ChoiceRequested)
}

/// Begin a source-free response/save conversion, pausing for an equip-slot cost if needed.
pub(crate) fn begin_source_free_response(
    tx: &mut Transaction,
    player: PlayerId,
    skill: SkillId,
    kind: CardKind,
    action: u32,
    action_payload: &[u32],
) -> Result<(), EffectError> {
    let inst = tx
        .state()
        .skills
        .get(skill)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("unknown response conversion skill".into()))?;
    if inst.owner != player || !tx.state().skill_effectively_enabled(&inst) {
        return Err(EffectError::Illegal(
            "response conversion skill unavailable".into(),
        ));
    }
    let rule = source_free_rule_for(tx.state(), &inst, kind, true)
        .ok_or_else(|| EffectError::Illegal("response conversion unavailable".into()))?;
    if let Some(group) = delegated_response_group(&rule) {
        return super::super::resolve_choice::begin_delegated_group_response(
            tx,
            player,
            skill,
            kind,
            action,
            action_payload,
            group,
        );
    }
    if rule.cost.as_deref() == Some("disable_equip_slot") {
        let slots: Vec<u32> = tx
            .state()
            .players
            .get(player)
            .into_iter()
            .flat_map(|owner| {
                (1..=5).filter_map(move |tag| {
                    EquipSlot::from_tag(tag)
                        .filter(|slot| owner.equip_slot_enabled(*slot))
                        .map(|slot| slot as u32)
                })
            })
            .collect();
        if slots.is_empty() {
            return Err(EffectError::Illegal("no enabled equipment slot".into()));
        }
        let mut payload = vec![skill.raw(), kind.tag() as u32, action];
        payload.extend_from_slice(action_payload);
        crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            slots,
            engine_api::ChoiceKind::Custom,
            crate::state::ContinuationKind::SourceFreeViewAsEquipCost,
            payload,
        )?;
        return Err(EffectError::ChoiceRequested);
    }
    complete_source_free_response_cost(tx, player, skill, kind, 0, action, action_payload)
}

/// Pay a selected response-conversion cost and continue the suspended response action.
pub(crate) fn complete_source_free_response_cost(
    tx: &mut Transaction,
    player: PlayerId,
    skill: SkillId,
    kind: CardKind,
    equip_slot: u8,
    action: u32,
    action_payload: &[u32],
) -> Result<(), EffectError> {
    let inst = tx
        .state()
        .skills
        .get(skill)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("unknown response conversion skill".into()))?;
    let rule = source_free_rule_for(tx.state(), &inst, kind, true)
        .ok_or_else(|| EffectError::Illegal("response conversion unavailable".into()))?;
    let slot = validate_source_free_cost(tx, player, &rule, equip_slot)?;
    record_source_free_use(tx, skill, &rule)?;
    let mut resume = vec![player.raw(), skill.raw(), kind.tag() as u32, action];
    resume.extend_from_slice(action_payload);
    {
        let resolution = &mut tx.state_mut().resolution;
        resolution.resume_kind = crate::state::ContinuationKind::SourceFreeViewAsResponse;
        resolution.resume_payload = resume.clone();
    }
    match pay_source_free_cost(tx, player, skill, kind, &rule, slot) {
        Ok(()) => {
            tx.state_mut().resolution.resume_kind = crate::state::ContinuationKind::None;
            tx.state_mut().resolution.resume_payload.clear();
            super::super::resolve_choice::resume_source_free_response(tx, &resume)
        }
        Err(EffectError::ChoiceRequested) => Err(EffectError::ChoiceRequested),
        Err(error) => {
            tx.state_mut().resolution.resume_kind = crate::state::ContinuationKind::None;
            tx.state_mut().resolution.resume_payload.clear();
            Err(error)
        }
    }
}

pub(crate) fn source_free_family_contains(family: &str, kind: CardKind) -> bool {
    match family {
        "basic" => kind.is_basic(),
        "trick" => kind.is_trick(),
        "ordinary_trick" => kind.is_trick() && !kind.is_delayed_trick(),
        "tao" => kind == CardKind::Tao,
        "jiu" => kind == CardKind::Jiu,
        _ => false,
    }
}

pub(crate) fn delegated_response_group(rule: &ViewAsDef) -> Option<&str> {
    rule.source_zone.as_deref()?.strip_prefix("group_")
}

pub(crate) fn delegated_response_candidates(
    state: &crate::state::GameState,
    owner: PlayerId,
    group: &str,
    kind: CardKind,
) -> Vec<PlayerId> {
    if !matches!(kind, CardKind::Sha | CardKind::Shan) {
        return Vec::new();
    }
    let mut order = state.players.alive_by_seat();
    let Some(owner_index) = order.iter().position(|player| *player == owner) else {
        return Vec::new();
    };
    order.rotate_left(owner_index + 1);
    order
        .into_iter()
        .filter(|player| {
            *player != owner
                && state.player_group(*player) == Some(group)
                && !match kind {
                    CardKind::Sha => super::super::play::delegated_sha_options(state, *player),
                    CardKind::Shan => super::super::play::delegated_shan_options(state, *player),
                    _ => Vec::new(),
                }
                .is_empty()
        })
        .collect()
}
