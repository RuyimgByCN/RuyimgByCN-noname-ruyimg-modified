//! Effect primitive implementations — bingshi 3 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

use super::*;

pub(crate) fn twyuyan_gift_cards(
    tx: &Transaction,
    player: PlayerId,
    source: PlayerId,
    sha: CardId,
) -> Vec<CardId> {
    let Some(sha_state) = tx.state().cards.get(sha) else {
        return Vec::new();
    };
    crate::query::gainable_target_cards(tx.state(), player, source, TargetCardScope::HandEquip)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                if sha_state.rank > 0 {
                    card.rank > sha_state.rank
                } else {
                    !crate::query::effective_card_kind(tx.state(), source, card).is_basic()
                }
            })
        })
        .collect()
}

pub(crate) fn resolve_twyuyan(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || crate::command::play::validate_saved_sha_use(tx, source, card).is_err()
    {
        return Ok(Vec::new());
    }
    let mut probe = tx.state().resolution.clone();
    if !probe.exclude_saved_sha_target(card, player) {
        return Err(EffectError::Illegal(
            "TW Yuyan target is not in the current Sha".into(),
        ));
    }
    let options = twyuyan_gift_cards(tx, player, source, card);
    if options.is_empty() {
        tx.state_mut()
            .resolution
            .exclude_saved_sha_target(card, player);
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        source,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveTwyuyan,
        vec![player.raw(), source.raw(), card.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_twguoyi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    hand_lock_skill_def: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets = crate::command::play::saved_targeted_card_targets(tx, player, card)?;
    if !targets.contains(&target) {
        return Err(EffectError::Illegal(
            "TW Guoyi target left the current card use".into(),
        ));
    }
    let owner = tx.state().players.get(player).unwrap();
    let target_is_max_hp = tx.state().players.alive_by_seat().iter().all(|candidate| {
        tx.state()
            .players
            .get(*candidate)
            .map_or(true, |candidate| {
                tx.state()
                    .players
                    .get(target)
                    .is_some_and(|target| target.hp >= candidate.hp)
            })
    });
    let target_is_max_hand = tx
        .state()
        .players
        .alive_by_seat()
        .iter()
        .all(|candidate| tx.state().hand_count(target) >= tx.state().hand_count(*candidate));
    let x = (owner.max_hp as i32 - owner.hp).max(0) as u32 + 1;
    let owner_low_hand = tx.state().hand_count(player) as u32 <= x;
    if !(target_is_max_hp || target_is_max_hand || owner_low_hand) {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        0,
        player.raw(),
        target.raw(),
        card.raw(),
        hand_lock_skill_def,
        skill.raw(),
        u32::from(target_is_max_hp || target_is_max_hand),
        u32::from(owner_low_hand),
        x,
    ];
    let can_discard = !crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty();
    if !can_discard {
        payload.push(0);
        return commit_twguoyi_choice(tx, &payload).map(|effects| {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .extend(effects);
            Vec::new()
        });
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        vec![0, 1],
        ChoiceKind::Custom,
        ContinuationKind::ResolveTwguoyi,
        payload,
    )
}

pub(crate) fn commit_twguoyi_choice(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<Vec<EffectRequest>, EffectError> {
    let [0, player, target, card, hand_lock_skill_def, skill, max_condition, low_hand, _x, choice] =
        payload
    else {
        return Err(EffectError::EngineFault(
            "bad TW Guoyi commit payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let target = PlayerId::new(*target);
    let card = CardId::new(*card);
    let skill = engine_api::SkillId::new(*skill);
    let choice = (*choice as u8).min(1);
    let mask = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("TW Guoyi skill disappeared".into()))?
        .mark_twguoyi_choice(choice);
    if (*max_condition != 0 && *low_hand != 0) || mask == 0b11 {
        let card_state = tx
            .state()
            .cards
            .get(card)
            .ok_or_else(|| EffectError::Illegal("TW Guoyi source card disappeared".into()))?;
        let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
        let targets = crate::command::play::saved_targeted_card_targets(tx, player, card)?;
        tx.state_mut()
            .resolution
            .register_card_replay(crate::state::PendingCardReplay {
                source_card: card,
                player,
                kind: kind.tag(),
                targets,
                skill,
            });
    }
    Ok((choice == 0)
        .then_some(EffectRequest::AddSkillUntilTurnEnd {
            player: target,
            skill_def: engine_api::SkillDefId::new(*hand_lock_skill_def),
        })
        .into_iter()
        .collect())
}

pub(crate) fn replay_virtual_card_on_targets(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    targets: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(kind) = CardKind::from_tag(kind) else {
        return Err(EffectError::ContentFault("bad TW Guoyi replay kind".into()));
    };
    if !kind.is_sha()
        && !matches!(kind, CardKind::Tao | CardKind::Jiu)
        && !(kind.is_trick() && !kind.is_delayed_trick() && kind != CardKind::TrickWuxie)
    {
        // Extra settlement of equipment, delayed tricks, and response-only cards has no second
        // independently playable effect after the original physical card has moved.
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || targets.iter().any(|target| {
            !tx.state()
                .players
                .get(*target)
                .is_some_and(|participant| participant.alive)
        })
    {
        return Ok(Vec::new());
    }
    let card = materialize_virtual_card(tx, player, kind)?;
    crate::command::play::play_as_kind(
        tx, player, card, kind, &targets, false, false, true, false,
    )?;
    Ok(Vec::new())
}

pub(crate) fn resolve_twchuhai(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    stage: u8,
    index: u32,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    const MISSION_SUCCEEDED_FLAG: u8 = 0;
    if action == 1 {
        if !originating_skill_is_enabled(tx, player, skill) {
            return Ok(Vec::new());
        }
        let hand = tx.state().zones.find(ZoneKind::Hand, player);
        let options: Vec<_> = event_cards
            .into_iter()
            .filter(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| Some(card.zone) == hand && card.owner == player)
            })
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        let mut payload = vec![1, player.raw(), skill.raw(), options.len() as u32];
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(|card| card.raw()).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ResolveTwchuhai,
            payload,
        );
    }
    if action != 0 {
        return Err(EffectError::ContentFault("unknown TW Chuhai action".into()));
    }
    if stage == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.content_flag(MISSION_SUCCEEDED_FLAG))
                .unwrap_or(false)
            || tx.state().turn.dying_caused_count(player) < 2
        {
            return Ok(Vec::new());
        }
        tx.state_mut()
            .skills
            .get_mut(skill)
            .expect("validated TW Chuhai skill")
            .set_content_flag(MISSION_SUCCEEDED_FLAG, true);
        tx.state_mut().disabled_judge_zones.insert(player);
        let judge = tx
            .state()
            .zones
            .find(ZoneKind::Judge, player)
            .ok_or_else(|| EffectError::EngineFault("TW Chuhai judge zone missing".into()))?;
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::EngineFault("TW Chuhai discard pile missing".into()))?;
        let judge_cards = tx
            .state()
            .zones
            .get(judge)
            .map(|zone| zone.cards.clone())
            .unwrap_or_default();
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::ResolveTwchuhai {
                player,
                action: 0,
                stage: 1,
                index: 0,
                event_cards: Vec::new(),
                skill,
            });
        let mut effects = Vec::new();
        if !judge_cards.is_empty() {
            effects.push(EffectRequest::MoveCardsWithReason {
                cards: judge_cards,
                to: discard,
                from: Some(judge),
                reason: engine_api::CardMoveReason::Discard,
            });
        }
        return run_effects(tx, effects).map(|events| events);
    }
    if stage != 1 {
        return Err(EffectError::EngineFault("bad TW Chuhai stage".into()));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let order = tx.state().players.alive_by_seat();
    let Some(owner_index) = order.iter().position(|candidate| *candidate == player) else {
        return Ok(Vec::new());
    };
    let others: Vec<_> = (1..order.len())
        .map(|offset| order[(owner_index + offset) % order.len()])
        .collect();
    let mut next = index as usize;
    while next < others.len()
        && crate::query::discardable_hand_equip_cards(tx.state(), others[next]).is_empty()
    {
        next += 1;
    }
    let Some(giver) = others.get(next).copied() else {
        return Ok(Vec::new());
    };
    let options = crate::query::discardable_hand_equip_cards(tx.state(), giver);
    let mut payload = vec![0, player.raw(), giver.raw(), skill.raw(), (next + 1) as u32];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        giver,
        1,
        1,
        false,
        options.into_iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveTwchuhai,
        payload,
    )
}

pub(crate) fn resolve_twyizhu(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    stage: u8,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            let count = tx.state().players.alive_by_seat().len().min(4) as u32;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwyizhu {
                    player,
                    source,
                    card,
                    action,
                    stage: 1,
                    event_cards: Vec::new(),
                    skill,
                });
            run_effects(tx, vec![EffectRequest::DrawCards { player, count }])
        }
        (0, 1) => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let need = options.len().min(2);
            if need == 0 {
                return Ok(Vec::new());
            }
            if options.len() <= 2 {
                return plant_twyizhu_cards(tx, player, options, skill);
            }
            let mut payload = vec![0, player.raw(), skill.raw(), need as u32];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                need as u8,
                need as u8,
                false,
                options.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwyizhu,
                payload,
            )
        }
        (0, 2) => plant_twyizhu_cards(tx, player, event_cards, skill),
        (1, 0) => {
            if source == player || card.is_none() {
                return Ok(Vec::new());
            }
            let tracked = tx
                .state()
                .skills
                .get(skill)
                .expect("validated TW Yizhu skill")
                .twkujian_cards();
            let marked: Vec<_> = event_cards
                .into_iter()
                .filter(|candidate| tracked.contains(candidate))
                .collect();
            if marked.is_empty() {
                return Ok(Vec::new());
            }
            let existing = crate::command::play::saved_targeted_card_targets(tx, source, card)?;
            if existing.len() != 1 {
                return Ok(Vec::new());
            }
            let (candidates, _) =
                crate::command::play::current_card_target_adjustments(tx, source, card)?;
            if candidates.is_empty() {
                return finish_twyizhu_used_cards(tx, player, &marked, skill);
            }
            let mut options = vec![existing[0].raw()];
            options.extend(candidates.iter().map(|target| target.raw()));
            let mut payload = vec![
                1,
                player.raw(),
                source.raw(),
                card.raw(),
                skill.raw(),
                existing[0].raw(),
                marked.len() as u32,
            ];
            payload.extend(marked.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                2,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwyizhu,
                payload,
            )
        }
        (2, 0) => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("validated TW Yizhu skill")
                .take_twkujian_cards(&event_cards);
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Yizhu resolver stage".into(),
        )),
    }
}

pub(crate) fn plant_twyizhu_cards(
    tx: &mut Transaction,
    player: PlayerId,
    mut cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    cards.sort_unstable_by_key(|card| card.raw());
    cards.dedup();
    if cards.is_empty() || cards.len() > 2 {
        return Err(EffectError::Illegal(
            "TW Yizhu requires one or two distinct cards".into(),
        ));
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    if cards.iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_none_or(|state| state.owner != player || ![hand, equip].contains(&Some(state.zone)))
    }) {
        return Err(EffectError::Illegal(
            "TW Yizhu selected card left hand/equipment".into(),
        ));
    }
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .expect("validated TW Yizhu skill")
        .add_twkujian_cards(&cards)
    {
        return Err(EffectError::ContentFault(
            "TW Yizhu tracking state is occupied".into(),
        ));
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("TW Yizhu draw pile missing".into()))?;
    let mut events = Vec::new();
    for from in [hand, equip].into_iter().flatten() {
        let group: Vec<_> = cards
            .iter()
            .copied()
            .filter(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|state| state.zone == from)
            })
            .collect();
        if !group.is_empty() {
            events.extend(move_cards_with_reason(
                tx,
                group,
                draw,
                Some(from),
                engine_api::CardMoveReason::Unspecified,
            )?);
        }
    }
    let depth_cap = tx
        .state()
        .players
        .alive_by_seat()
        .len()
        .min(4)
        .saturating_mul(2)
        .saturating_sub(2);
    let pile_len = tx
        .state()
        .zones
        .get(draw)
        .map_or(0, |zone| zone.cards.len());
    let available = pile_len.saturating_sub(cards.len());
    let max_depth = depth_cap.min(available);
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let depth = if max_depth == 0 {
        0
    } else {
        rng.gen_range_u32((max_depth + 1) as u32) as usize
    };
    tx.state_mut().rng = rng.into_state();
    let zone = tx
        .state_mut()
        .zones
        .get_mut(draw)
        .expect("validated draw pile");
    zone.cards.retain(|card| !cards.contains(card));
    let at = zone.cards.len().saturating_sub(depth);
    zone.cards.splice(at..at, cards);
    Ok(events)
}

pub(crate) fn finish_twyizhu_used_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: &[CardId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    tx.state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::Illegal("TW Yizhu skill disappeared".into()))?
        .take_twkujian_cards(cards);
    run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twluanchou(
    tx: &mut Transaction,
    player: PlayerId,
    helper_skill_def: u32,
    action: u8,
    mut targets: Vec<PlayerId>,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
    card: CardId,
    card_kind: u8,
    card_flags: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            targets.sort_unstable_by_key(|target| target.raw());
            targets.dedup();
            if targets.len() != 2
                || targets.iter().any(|target| {
                    !tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|participant| participant.alive)
                })
            {
                return Err(EffectError::Illegal(
                    "TW Luanchou requires two distinct living targets".into(),
                ));
            }
            let mut effects: Vec<_> = tx
                .state()
                .skills
                .sorted_live()
                .into_iter()
                .filter(|instance| instance.def.raw() == helper_skill_def)
                .map(|instance| EffectRequest::RemoveSkill { skill: instance.id })
                .collect();
            effects.extend(targets.into_iter().map(|target| EffectRequest::AddSkill {
                player: target,
                skill_def: engine_api::SkillDefId::new(helper_skill_def),
            }));
            run_effects(tx, effects)
        }
        1 => {
            let damage_id = tx.state().resolution.current_damage_id;
            if damage_id == 0
                || tx.state().resolution.damage_redirect_in_progress
                || target == player
                || amount == 0
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .prevented_damage_ids
                .push(damage_id);
            tx.state_mut().resolution.damage_redirect_in_progress = true;
            let result = run_effects(
                tx,
                vec![EffectRequest::DealCardDamage {
                    source,
                    target: player,
                    amount,
                    is_lose_hp: false,
                    nature,
                    card,
                    card_kind,
                    card_flags,
                }],
            );
            tx.state_mut().resolution.damage_redirect_in_progress = false;
            result
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Luanchou action".into(),
        )),
    }
}

/// Custom-continuation marker for ABI 456 `choose_discard_heart_redirect_current_damage`.
pub(crate) const RESOLVE_TIANXIANG_CUSTOM_CHOICE: u32 = 0xDCD8_4560;

pub(crate) fn resolve_tianxiang(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    discard_card: CardId,
    source: PlayerId,
    amount: u32,
    nature: u8,
    card: CardId,
    card_kind: u8,
    card_flags: u8,
    draw_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if draw_skill_def.raw() == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        // Stage 0: choose a discardable heart hand card.
        let hearts: Vec<_> = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .into_iter()
            .filter_map(|zone| tx.state().zones.get(zone))
            .flat_map(|zone| zone.cards.iter().copied().collect::<Vec<_>>())
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card_state| {
                    card_state.suit == engine_api::Suit::Heart
                        && crate::query::discardable_hand_equip_cards(tx.state(), player)
                            .contains(card)
                })
            })
            .collect();
        if hearts.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            hearts.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_TIANXIANG_CUSTOM_CHOICE,
                player.raw(),
                draw_skill_def.raw(),
                source.raw(),
                amount,
                u32::from(nature),
                card.raw(),
                u32::from(card_kind),
                u32::from(card_flags),
                skill.raw(),
            ],
        );
    }
    // Stage 2: discard the heart cost, redirect the current damage, grant the draw skill.
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id == 0
        || tx.state().resolution.damage_redirect_in_progress
        || target == player
        || amount == 0
        || source.is_none()
    {
        return Ok(Vec::new());
    }
    let discard = discard_cards(tx, player, vec![discard_card])?;
    let mut events = discard;
    events.extend(run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: draw_skill_def,
        }],
    )?);
    tx.state_mut()
        .resolution
        .prevented_damage_ids
        .push(damage_id);
    tx.state_mut().resolution.damage_redirect_in_progress = true;
    let result = run_effects(
        tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature,
            card,
            card_kind,
            card_flags,
        }],
    );
    tx.state_mut().resolution.damage_redirect_in_progress = false;
    if let Ok(redirected) = result {
        events.extend(redirected);
    }
    Ok(events)
}

/// Custom-continuation marker for ABI 457 `choose_discard_heart_cancel_damage_branch`.
pub(crate) const RESOLVE_RETIANXIANG_CUSTOM_CHOICE: u32 = 0xDCE1_4570;

pub(crate) fn resolve_retianxiang(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    discard_card: CardId,
    branch: u8,
    source: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        // Stage 0: choose a discardable heart hand card.
        let hearts: Vec<_> = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .into_iter()
            .filter_map(|zone| tx.state().zones.get(zone))
            .flat_map(|zone| zone.cards.iter().copied().collect::<Vec<_>>())
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|card_state| {
                    card_state.suit == engine_api::Suit::Heart
                        && crate::query::discardable_hand_equip_cards(tx.state(), player)
                            .contains(card)
                })
            })
            .collect();
        if hearts.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            hearts.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_RETIANXIANG_CUSTOM_CHOICE,
                player.raw(),
                source.raw(),
                skill.raw(),
            ],
        );
    }
    // Apply: cancel the current damage, discard the heart cost, run the chosen branch.
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id == 0 || target == player {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .resolution
        .prevented_damage_ids
        .push(damage_id);
    let mut events = discard_cards(tx, player, vec![discard_card])?;
    if branch == 0 {
        events.extend(run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )?);
        let lost = tx
            .state()
            .players
            .get(target)
            .map(|participant| {
                participant
                    .max_hp
                    .saturating_sub(participant.hp.max(0) as u32)
            })
            .unwrap_or(0)
            .min(5);
        if lost > 0 {
            events.extend(run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player: target,
                    count: lost,
                }],
            )?);
        }
    } else {
        events.extend(run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            }],
        )?);
        let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
        if let Some(discard) = discard {
            events.extend(run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards: vec![discard_card],
                    to: tx
                        .state()
                        .zones
                        .find(ZoneKind::Hand, target)
                        .ok_or_else(|| EffectError::Illegal("target hand zone missing".into()))?,
                    from: Some(discard),
                }],
            )?);
        }
    }
    Ok(events)
}

/// Custom-continuation marker for ABI 458 `choose_link_other_same_hp`.
pub(crate) const RESOLVE_LIANZHOU_CUSTOM_CHOICE: u32 = 0xDCE3_4580;

pub(crate) fn resolve_lianzhou(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if targets.is_empty() {
        // Stage 0: choose any number of other players with equal HP that are not linked.
        let owner_hp = tx
            .state()
            .players
            .get(player)
            .map(|participant| participant.hp)
            .unwrap_or(0);
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&target| {
                target != player
                    && tx.state().players.get(target).is_some_and(|participant| {
                        participant.hp == owner_hp
                            && !participant.marks.iter().any(|(mark, count)| {
                                *mark == crate::content::MARK_CHAIN && *count > 0
                            })
                    })
            })
            .map(PlayerId::raw)
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        let max = u8::try_from(options.len()).unwrap_or(u8::MAX);
        return open_choice(
            tx,
            player,
            1,
            max,
            false,
            options,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![RESOLVE_LIANZHOU_CUSTOM_CHOICE, player.raw(), skill.raw()],
        );
    }
    // Apply: link every selected target.
    let mut events = Vec::new();
    for target in targets {
        events.extend(run_effects(
            tx,
            vec![EffectRequest::AddMark {
                player: target,
                mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                delta: 1,
            }],
        )?);
    }
    Ok(events)
}

/// Custom-continuation marker for ABI 459 `choose_give_hand_equip_to_event_player`.
pub(crate) const RESOLVE_STDDECHONG_CUSTOM_CHOICE: u32 = 0xDCE7_4590;

pub(crate) fn resolve_stddechong(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
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
    if cards.is_empty() {
        // Stage 0: choose any number of discardable hand/equipment cards.
        let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
        if options.is_empty() {
            return Ok(Vec::new());
        }
        let max = u8::try_from(options.len()).unwrap_or(u8::MAX);
        return open_choice(
            tx,
            player,
            1,
            max,
            false,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_STDDECHONG_CUSTOM_CHOICE,
                player.raw(),
                target.raw(),
                skill.raw(),
            ],
        );
    }
    // Apply: give every selected card to the target's hand.
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("recipient hand zone missing".into()))?;
    let mut by_zone = std::collections::BTreeMap::<u32, Vec<CardId>>::new();
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(CardId::raw)
            .collect();
    let mut events = Vec::new();
    for card in &cards {
        let card_state = tx
            .state()
            .cards
            .get(*card)
            .ok_or_else(|| EffectError::Illegal("bad gift card".into()))?;
        if card_state.owner != player || !allowed.contains(&card.raw()) {
            return Err(EffectError::Illegal(
                "gift card not in player hand or equipment".into(),
            ));
        }
        by_zone
            .entry(card_state.zone.raw())
            .or_default()
            .push(*card);
    }
    for (from, cards) in by_zone {
        events.extend(run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards,
                to: hand,
                from: Some(engine_api::ZoneId::new(from)),
            }],
        )?);
    }
    Ok(events)
}

/// Custom-continuation marker for ABI 460 `discard_hand_equip_count`.
pub(crate) const DISCARD_HAND_EQUIP_COUNT_CUSTOM_CHOICE: u32 = 0xDCE9_4600;

/// Custom-continuation marker for ABI 596 `choose_discard_hand_range`.
pub(crate) const CHOOSE_DISCARD_HAND_RANGE_CHOICE: u32 = 0xDCDC_5960;

/// Custom-continuation marker for ABI 597 `resolve_dcctjiuxian`.
pub(crate) const RESOLVE_DCCTJIUXIAN_CHOICE: u32 = 0xDCDC_5970;
/// Custom-continuation marker for ABI 598 `resolve_bianyue`.
pub(crate) const RESOLVE_BIANYUE_CHOICE: u32 = 0xDCDC_5980;
/// Custom-continuation marker for ABI 600 `resolve_zhupeilan`.
pub(crate) const RESOLVE_ZHUPEILAN_CHOICE: u32 = 0xDCDC_6000;
pub const RESOLVE_DC_SB_ZHANGXIU_CHOICE: u32 = 0xDCDC_6010;
pub const RESOLVE_DC_SB_ZHOUYU_CHOICE: u32 = 0xDCDC_6020;
pub const RESOLVE_DC_SB_LUSU_CHOICE: u32 = 0xDCDC_6030;
pub const RESOLVE_ZHANGJIAN_CHOICE: u32 = 0xDCDC_6040;

/// Custom-continuation marker for ABI 461 `resolve_jieliang_gain`.
pub(crate) const RESOLVE_JIELIANG_GAIN_CUSTOM_CHOICE: u32 = 0xDCEB_4610;

/// Custom-continuation marker for ABI 462 `choose_other_gain_event_cards`.
pub(crate) const RESOLVE_STDJUJIAN_CUSTOM_CHOICE: u32 = 0xDCED_4620;

/// Custom-continuation marker for ABI 463 `resolve_lulve`.
pub(crate) const RESOLVE_LULVE_CUSTOM_CHOICE: u32 = 0xDCEF_4630;

/// Custom-continuation marker for ABI 464 `resolve_dctingxian`.
pub(crate) const RESOLVE_DCTINGXIAN_CUSTOM_CHOICE: u32 = 0xDCF1_4640;

/// Custom-continuation marker for ABI 465 `choose_other_add_skill`.
pub(crate) const RESOLVE_HFJIEYING_CUSTOM_CHOICE: u32 = 0xDCF3_4650;

/// Custom-continuation marker for ABI 466 `resolve_dcnuchen`.
pub(crate) const RESOLVE_DCNUCHEN_CUSTOM_CHOICE: u32 = 0xDCF5_4660;

/// Custom-continuation marker for ABI 467 `resolve_dczhiheng`.
pub(crate) const RESOLVE_DCZHIHENG_CUSTOM_CHOICE: u32 = 0xDCF7_4670;

/// Custom-continuation marker for ABI 468 `resolve_stdgaobian`.
pub(crate) const RESOLVE_STDGAOBIAN_CUSTOM_CHOICE: u32 = 0xDCF9_4680;

/// Custom-continuation marker for ABI 469 `resolve_spjiedao`.
pub(crate) const RESOLVE_SPJIEDAO_CUSTOM_CHOICE: u32 = 0xDCFA_4690;

/// Custom-continuation marker for ABI 470 `resolve_sibian`.
pub(crate) const RESOLVE_SIBIAN_CUSTOM_CHOICE: u32 = 0xDCFB_4700;

/// Custom-continuation marker for ABI 491 category-card distribution.
pub(crate) const REVEAL_CATEGORY_DISTRIBUTION_CUSTOM_CHOICE: u32 = 0xDCFB_4910;

/// Custom-continuation marker for ABI 492 deterministic region-card gain.
pub(crate) const RANDOM_GAIN_REGION_COMPARE_COLOR_CUSTOM_CHOICE: u32 = 0xDCD2_4920;

/// Custom-continuation marker for ABI 493 `resolve_refuhai`.
pub(crate) const REFUHAI_CUSTOM_CHOICE: u32 = 0xDCD5_4930;

/// Custom-continuation marker for ABI 494 `choose_gain_target_hand_ceil_half_difference`.
pub(crate) const YANGWU_GAIN_HAND_CUSTOM_CHOICE: u32 = 0xDCD6_4940;

/// Custom-continuation marker for ABI 495 `resolve_gzjinfa`.
pub(crate) const GZJINFA_CUSTOM_CHOICE: u32 = 0xDCD7_4950;

/// Custom-continuation marker for ABI 497 `yuanlve`.
pub(crate) const YUANLVE_CUSTOM_CHOICE: u32 = 0xDCD9_4970;

/// Custom-continuation marker for ABI 499 `jiebing`.
pub(crate) const JIEBING_CUSTOM_CHOICE: u32 = 0xDCDA_4990;

/// Custom-continuation marker for ABI 500 `mobiledanshou`.
pub(crate) const MOBILEDANSHOU_CUSTOM_CHOICE: u32 = 0xDCDB_5000;

/// Custom-continuation marker for ABI 501 `jsrgchiying`.
pub(crate) const RESOLVE_JSRGCHIYING_CUSTOM_CHOICE: u32 = 0xDCDC_5010;

/// Custom-continuation markers for ABI 503 国渊 skills.
pub(crate) const MBQINGDAO_CUSTOM_CHOICE: u32 = 0xDCDD_5031;
pub(crate) const MBXIUGENG_CUSTOM_CHOICE: u32 = 0xDCDD_5032;
pub(crate) const MBCHENSHE_CUSTOM_CHOICE: u32 = 0xDCDD_5033;
pub(crate) const MBRENXING_CUSTOM_CHOICE: u32 = 0xDCDD_5041;
pub(crate) const MBCHIZHANG_CUSTOM_CHOICE: u32 = 0xDCDD_5051;
pub(crate) const MBDUANYANG_CUSTOM_CHOICE: u32 = 0xDCDD_5052;
/// Custom-continuation marker for ABI 529 势桓阶「共谋」。
pub(crate) const POTHUANJIE_CUSTOM_CHOICE: u32 = 0xDCDD_5291;
pub(crate) const POTNIGU_CUSTOM_CHOICE: u32 = 0xDCDD_5321;
pub(crate) const POTLULIAN_CUSTOM_CHOICE: u32 = 0xDCDD_5322;
/// Custom-continuation marker for ABI 535 势鲁肃「缔盟」。
pub(crate) const POTDIMENG_CUSTOM_CHOICE: u32 = 0xDCDD_5351;
pub(crate) const POTHAOSHI_CUSTOM_CHOICE: u32 = 0xDCDD_5361;

/// One refill settlement per distinct 好施 lender represented by these physical sources.
pub(crate) fn pothaoshi_refill_effects(
    state: &crate::state::GameState,
    borrower: PlayerId,
    cards: &[CardId],
) -> Vec<EffectRequest> {
    let mut grants = std::collections::BTreeSet::new();
    for card in cards {
        if let Some((lender, skill)) = crate::query::borrowed_hand_source(state, borrower, *card) {
            grants.insert((lender.raw(), skill.raw()));
        }
    }
    grants
        .into_iter()
        .map(|(lender, skill)| EffectRequest::ResolvePothaoshi {
            player: PlayerId::new(lender),
            target: borrower,
            action: 2,
            skill: engine_api::SkillId::new(skill),
        })
        .collect()
}

pub(crate) fn resolve_jsrgchiying(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    started: bool,
    mut remaining: Vec<PlayerId>,
    accum: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if !started {
        let mut order = tx.state().players.alive_by_seat();
        let Some(index) = order.iter().position(|candidate| *candidate == player) else {
            return Ok(Vec::new());
        };
        order.rotate_left(index);
        remaining = order
            .into_iter()
            .filter(|candidate| {
                *candidate != player && dcchiying_in_range(tx.state(), target, *candidate)
            })
            .collect();
    }
    while let Some(discarder) = remaining.first().copied() {
        remaining.remove(0);
        if !tx
            .state()
            .players
            .get(discarder)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let cards = crate::query::discardable_hand_equip_cards(tx.state(), discarder);
        if cards.is_empty() {
            continue;
        }
        let mut payload = vec![
            RESOLVE_JSRGCHIYING_CUSTOM_CHOICE,
            player.raw(),
            target.raw(),
            skill.raw(),
            discarder.raw(),
            remaining.len() as u32,
        ];
        payload.extend(remaining.iter().map(|participant| participant.raw()));
        payload.push(accum.len() as u32);
        payload.extend(accum.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            discarder,
            1,
            1,
            false,
            cards.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            payload,
        );
    }
    let hp = tx
        .state()
        .players
        .get(target)
        .map(|participant| participant.hp.max(0) as usize)
        .unwrap_or(0);
    let basic_count = accum
        .iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(**card)
                .is_some_and(|card| card.kind.is_basic())
        })
        .count();
    if basic_count <= hp {
        dcchiying_gain_basics(tx, target, accum)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn resolve_mbqingdao(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    damaged: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == source
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut options = vec![1];
    let second_available = if damaged {
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .any(|target| {
                !crate::command::play::discardable_target_zone_card_tokens(tx, player, target)
                    .is_empty()
            })
    } else {
        tx.state().hand_cards(player).into_iter().any(|candidate| {
            crate::command::play::skill_card_target_spec_with_options(
                tx, player, candidate, true, true,
            )
            .is_ok()
        })
    };
    if second_available {
        options.push(2);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            MBQINGDAO_CUSTOM_CHOICE,
            0,
            player.raw(),
            source.raw(),
            card.raw(),
            u32::from(damaged),
            skill.raw(),
        ],
    )
}

pub(crate) fn resolve_mbxiugeng(
    tx: &mut Transaction,
    player: PlayerId,
    helper_def: engine_api::SkillDefId,
    hand_limit_def: engine_api::SkillDefId,
    stage: u8,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let options = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            open_choice(
                tx,
                player,
                1,
                2,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    MBXIUGENG_CUSTOM_CHOICE,
                    player.raw(),
                    helper_def.raw(),
                    hand_limit_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let baseline = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| instance.state_blob.get(..4))
                .map(|bytes| u32::from_le_bytes(bytes.try_into().expect("four bytes")))
                .ok_or_else(|| EffectError::ContentFault("missing 修耕 hand record".into()))?;
            let hand = tx.state().hand_count(player) as u32;
            let mut effects = Vec::new();
            if hand <= baseline {
                effects.push(EffectRequest::DrawCards { player, count: 2 });
            }
            if hand >= baseline {
                effects.push(EffectRequest::AddSkill {
                    player,
                    skill_def: hand_limit_def,
                });
                effects.push(EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(hand_limit_def.raw()),
                    delta: 1,
                });
            }
            effects.push(EffectRequest::RemoveSkill { skill });
            run_effects(tx, effects)
        }
        2 => {
            let mut events = Vec::new();
            for target in targets.into_iter().take(2) {
                if !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                {
                    continue;
                }
                let baseline = u32::try_from(tx.state().hand_count(target)).unwrap_or(u32::MAX);
                events.extend(remove_skill_by_def(tx, target, helper_def)?);
                events.extend(add_skill_with_state(
                    tx,
                    target,
                    helper_def,
                    Some(baseline.to_le_bytes().to_vec()),
                )?);
            }
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown 修耕 stage".into())),
    }
}

pub(crate) fn resolve_mbchenshe(
    tx: &mut Transaction,
    player: PlayerId,
    dying: PlayerId,
    source: PlayerId,
    stage: u8,
    mut roles: Vec<PlayerId>,
    suits: Vec<u8>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let dying_state = tx.state().players.get(dying);
    if !originating_skill_is_enabled(tx, player, skill)
        || player == dying
        || !dying_state
            .is_some_and(|participant| participant.alive && (stage != 0 || participant.dying))
    {
        return Ok(Vec::new());
    }
    if stage == 0 {
        roles = [player, dying, source]
            .into_iter()
            .filter(|target| {
                target.is_some()
                    && tx
                        .state()
                        .players
                        .get(*target)
                        .is_some_and(|participant| participant.alive)
                    && if *target == player {
                        !crate::query::discardable_hand_equip_cards(tx.state(), player).is_empty()
                    } else {
                        !crate::command::play::discardable_target_zone_card_tokens(
                            tx, player, *target,
                        )
                        .is_empty()
                    }
            })
            .collect();
    }
    while let Some(target) = roles.first().copied() {
        let options = if target == player {
            crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .map(CardId::raw)
                .collect()
        } else {
            crate::command::play::discardable_target_zone_card_tokens(tx, player, target)
        };
        if options.is_empty() {
            roles.remove(0);
            continue;
        }
        let mut payload = vec![
            MBCHENSHE_CUSTOM_CHOICE,
            player.raw(),
            dying.raw(),
            source.raw(),
            skill.raw(),
            roles.len() as u32,
        ];
        payload.extend(roles.iter().map(|target| target.raw()));
        payload.push(suits.len() as u32);
        payload.extend(suits.iter().map(|suit| u32::from(*suit)));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            payload,
        );
    }
    if suits.len() == 3 && suits.iter().all(|suit| *suit == suits[0]) {
        let amount = tx
            .state()
            .players
            .get(dying)
            .map(|target| (target.max_hp as i32 - target.hp).max(0) as u32)
            .unwrap_or(0);
        run_effects(
            tx,
            vec![
                EffectRequest::RecoverHp {
                    source: player,
                    player: dying,
                    amount,
                },
                EffectRequest::RemoveSkill { skill },
            ],
        )
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn resolve_mbquanchong(
    tx: &mut Transaction,
    player: PlayerId,
    helper_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.extra_turn_queue.len() >= 8
        || crate::query::discardable_hand_equip_cards(tx.state(), player).is_empty()
    {
        return Ok(Vec::new());
    }
    let unique_max = tx
        .state()
        .players
        .get(player)
        .filter(|owner| owner.alive)
        .is_some_and(|owner| {
            tx.state()
                .players
                .iter_live()
                .filter(|other| other.alive && other.id != player)
                .all(|other| owner.hp > other.hp)
        });
    let mut effects = vec![EffectRequest::DiscardAllHandAndEquip { player }];
    if !unique_max {
        effects.push(EffectRequest::AddSkill {
            player,
            skill_def: helper_def,
        });
    }
    tx.state_mut().turn.extra_turn_queue.push(player);
    run_effects(tx, effects)
}

pub(crate) fn resolve_mbrenxing(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if stage == 1 {
        return run_effects(
            tx,
            vec![EffectRequest::ChooseDiscardTargetCard {
                player,
                target,
                scope: TargetCardScope::HandEquipJudge,
            }],
        );
    }
    let round = tx.state().turn.round;
    let count = tx
        .state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 12 && &instance.state_blob[..4] == b"RNX1")
                .then(|| {
                    let stored = u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap());
                    (stored == round)
                        .then(|| u32::from_le_bytes(instance.state_blob[8..12].try_into().unwrap()))
                })
                .flatten()
        })
        .unwrap_or(0);
    if count >= 2 {
        return Ok(Vec::new());
    }
    let current = tx.state().turn.current;
    let draw_available = tx
        .state()
        .players
        .get(current)
        .is_some_and(|participant| participant.alive);
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            tx.state().turn.flag(turn_flag::sha_used_by_player(*target)) == 0
                && !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    *target,
                    TargetCardScope::HandEquipJudge,
                )
                .is_empty()
        })
        .collect();
    let mut options = Vec::new();
    if draw_available {
        options.push(1);
    }
    if !targets.is_empty() {
        options.push(2);
    }
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut blob = b"RNX1".to_vec();
    blob.extend_from_slice(&round.to_le_bytes());
    blob.extend_from_slice(&count.saturating_add(1).to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
    let mut payload = vec![
        MBRENXING_CUSTOM_CHOICE,
        0,
        player.raw(),
        current.raw(),
        skill.raw(),
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
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

pub(crate) fn resolve_mbchizhang(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if stage == 1 {
        let hand = tx.state().zones.find(ZoneKind::Hand, player);
        if cards.is_empty()
            || cards.iter().any(|selected| {
                !tx.state().cards.get(*selected).is_some_and(|state| {
                    state.owner == player
                        && Some(state.zone) == hand
                        && !crate::query::card_discard_protected_from(
                            tx.state(),
                            player,
                            player,
                            *selected,
                        )
                })
            })
        {
            return Err(EffectError::Illegal("invalid 鸱张 discard".into()));
        }
        let mask = cards.iter().fold(0, |mask, selected| {
            mask | crate::query::effective_card_color_mask(tx.state(), player, *selected)
        });
        tx.state_mut()
            .resolution
            .lock_card_response_colors(card, mask);
        return run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]);
    }
    let valid = tx
        .state()
        .resolution
        .pending_card_resolutions
        .iter()
        .rev()
        .find(|record| record.card == card)
        .is_some_and(|record| record.source == player && record.source_was_hand);
    let options =
        crate::query::discardable_target_cards(tx.state(), player, player, TargetCardScope::Hand);
    if !valid || options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        options.len().min(usize::from(u8::MAX)) as u8,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            MBCHIZHANG_CUSTOM_CHOICE,
            player.raw(),
            card.raw(),
            skill.raw(),
        ],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_mbduanyang(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    mode: u8,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let skill_def = tx.state().skills.get(skill).unwrap().def.raw();
    match (mode, stage) {
        (0, 0) => {
            let mut candidates: Vec<_> = cards
                .into_iter()
                .filter(|candidate| {
                    tx.state()
                        .cards
                        .get(*candidate)
                        .is_some_and(|state| state.owner.is_none() && state.kind.is_sha())
                })
                .collect();
            candidates.sort_by_key(|candidate| candidate.raw());
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            let mut rng = RuleRng::from_state(tx.state().rng);
            let selected = candidates[rng.gen_range_u32(candidates.len() as u32) as usize];
            tx.journal(JournalEntry::RngCheckpoint);
            tx.state_mut().rng = rng.into_state();
            let from = tx.state().cards.get(selected).unwrap().zone;
            let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
            if let Some(reference) = tx
                .state_mut()
                .skill_piles
                .iter_mut()
                .find(|reference| reference.owner == player && reference.skill_def == skill_def)
            {
                reference.usable_as_hand = true;
            }
            move_cards_with_reason(
                tx,
                vec![selected],
                pile,
                Some(from),
                CardMoveReason::Unspecified,
            )
        }
        (1, 0) => {
            let pile_cards = tx.state().skill_pile_cards(player, skill_def);
            let playable = pile_cards.into_iter().find_map(|candidate| {
                crate::command::play::skill_card_target_spec_with_options(
                    tx, player, candidate, true, false,
                )
                .ok()
                .map(|spec| (candidate, spec))
            });
            let Some((candidate, spec)) = playable else {
                let cards = tx.state().skill_pile_cards(player, skill_def);
                if cards.is_empty() {
                    return Ok(Vec::new());
                }
                let from = ensure_skill_pile_zone(tx, player, skill_def)?;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("missing discard pile".into()))?;
                return move_cards_with_reason(
                    tx,
                    cards,
                    discard,
                    Some(from),
                    CardMoveReason::Discard,
                );
            };
            match spec {
                crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
                    open_choice(
                        tx,
                        player,
                        min,
                        max,
                        false,
                        options.into_iter().map(PlayerId::raw).collect(),
                        ChoiceKind::SelectTargets,
                        ContinuationKind::Custom,
                        vec![
                            MBDUANYANG_CUSTOM_CHOICE,
                            1,
                            player.raw(),
                            candidate.raw(),
                            skill.raw(),
                        ],
                    )
                }
                _ => Err(EffectError::ContentFault(
                    "断鞅 only stores physical Sha".into(),
                )),
            }
        }
        (1, 1) => {
            let mut blob = b"DYG1".to_vec();
            blob.extend_from_slice(&card.raw().to_le_bytes());
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveMbduanyang {
                    player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    mode: 1,
                    stage: 0,
                    cards: Vec::new(),
                    skill,
                },
            );
            crate::command::play::play_as_kind(
                tx,
                player,
                card,
                CardKind::Sha,
                &[target],
                true,
                false,
                false,
                false,
            )?;
            Ok(Vec::new())
        }
        (2, 0) => {
            let options =
                crate::command::play::target_zone_card_tokens(tx, target, true, true, true)
                    .into_iter()
                    .map(CardId::new)
                    .collect::<Vec<_>>();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                options.len().min(2) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    MBDUANYANG_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        (2, 1) => run_effects(
            tx,
            vec![
                EffectRequest::RecastCards {
                    player: target,
                    cards,
                },
                EffectRequest::DrawCards { player, count: 4 },
            ],
        ),
        _ => Err(EffectError::ContentFault("bad 断鞅 stage".into())),
    }
}

pub(crate) const MBRUNWEI_CUSTOM_CHOICE: u32 = 0xDCDD_5061;
pub(crate) const MBSHUANGHUAI_CUSTOM_CHOICE: u32 = 0xDCDD_5062;
pub(crate) const MBXIONGTU_CUSTOM_CHOICE: u32 = 0xDCDD_5071;
pub(crate) const MBSIJIAN_CUSTOM_CHOICE: u32 = 0xDCDD_5081;
pub(crate) const MBJIANJI_CUSTOM_CHOICE: u32 = 0xDCDD_5091;
pub(crate) const MBFEIJING_CUSTOM_CHOICE: u32 = 0xDCDD_5101;
pub(crate) const MBXUYE_CUSTOM_CHOICE: u32 = 0xDCDD_5141;
pub(crate) const POTCHENGPU_CUSTOM_CHOICE: u32 = 0xDCDD_5151;
pub(crate) const POTJUGU_CUSTOM_CHOICE: u32 = 0xDCDD_5161;
pub(crate) const POTSIFENG_CUSTOM_CHOICE: u32 = 0xDCDD_5171;
pub(crate) const POTCHENQUN_CUSTOM_CHOICE: u32 = 0xDCDD_5181;
pub(crate) const POTCHENDAO_CUSTOM_CHOICE: u32 = 0xDCDD_5191;
pub(crate) const MBZHUJI_CUSTOM_CHOICE: u32 = 0xDCDD_5211;
pub(crate) const POTPOXI_CUSTOM_CHOICE: u32 = 0xDCDD_5221;
pub(crate) const POTFUAN_CUSTOM_CHOICE: u32 = 0xDCDD_5251;
pub(crate) const MBYUANMO_CUSTOM_CHOICE: u32 = 0xDCDD_5092;

pub(crate) fn mbrunwei_state(tx: &Transaction, skill: engine_api::SkillId) -> (u32, bool, Vec<CardId>) {
    let Some(blob) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| &instance.state_blob)
    else {
        return (0, false, Vec::new());
    };
    if blob.len() < 16 || &blob[..4] != b"RNW1" {
        return (0, false, Vec::new());
    }
    let remaining = u32::from_le_bytes(blob[4..8].try_into().unwrap());
    let enabled = u32::from_le_bytes(blob[8..12].try_into().unwrap()) != 0;
    let count = u32::from_le_bytes(blob[12..16].try_into().unwrap()) as usize;
    if blob.len() != 16 + count * 4 {
        return (0, false, Vec::new());
    }
    let cards = blob[16..]
        .chunks_exact(4)
        .map(|raw| CardId::new(u32::from_le_bytes(raw.try_into().unwrap())))
        .collect();
    (remaining, enabled, cards)
}

pub(crate) fn set_mbrunwei_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    remaining: u32,
    enabled: bool,
    cards: &[CardId],
) {
    let mut blob = b"RNW1".to_vec();
    blob.extend_from_slice(&remaining.to_le_bytes());
    blob.extend_from_slice(&u32::from(enabled).to_le_bytes());
    blob.extend_from_slice(&(cards.len() as u32).to_le_bytes());
    for card in cards {
        blob.extend_from_slice(&card.raw().to_le_bytes());
    }
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn resolve_mbrunwei(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    value: u32,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let (_, enabled, _) = mbrunwei_state(tx, skill);
            let uses = tx.state().skills.get(skill).unwrap().play_phase_use_count;
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || uses == 0
                || uses > 2
                || (uses == 2 && !enabled)
            {
                return Err(EffectError::Illegal("润微 is not available".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                vec![1, 2, 3, 4, 5],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![MBRUNWEI_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let count = value.clamp(1, 5) as usize;
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("missing draw pile".into()))?;
            while tx
                .state()
                .zones
                .get(draw)
                .map_or(0, |zone| zone.cards.len())
                < count
            {
                let before = tx
                    .state()
                    .zones
                    .get(draw)
                    .map_or(0, |zone| zone.cards.len());
                reshuffle_discard_into_draw(tx)?;
                if tx
                    .state()
                    .zones
                    .get(draw)
                    .map_or(0, |zone| zone.cards.len())
                    == before
                {
                    break;
                }
            }
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .map(|zone| zone.cards.iter().rev().take(count).copied().collect())
                .unwrap_or_default();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let mut colors: Vec<u32> = cards
                .iter()
                .map(|card| {
                    u32::from(crate::query::effective_card_color_mask(
                        tx.state(),
                        player,
                        *card,
                    ))
                })
                .filter(|color| *color != 0)
                .collect();
            colors.sort_unstable();
            colors.dedup();
            let mut events = vec![DomainEvent::CardsRevealed {
                cards: cards.clone(),
                to: Vec::new(),
            }];
            if colors.is_empty() {
                events.extend(discard_revealed_mbrunwei_cards(tx, &cards)?);
                return Ok(events);
            }
            let mut payload = vec![MBRUNWEI_CUSTOM_CHOICE, 1, player.raw(), skill.raw()];
            payload.extend(cards.iter().map(|card| card.raw()));
            events.extend(open_choice(
                tx,
                player,
                1,
                1,
                false,
                colors,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                payload,
            )?);
            Ok(events)
        }
        2 => {
            let (_, enabled, _) = mbrunwei_state(tx, skill);
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| !enabled || tx.state().turn.card_gain_count(*target) == 0)
                .collect();
            if targets.is_empty() {
                return discard_revealed_mbrunwei_cards(tx, &cards);
            }
            let mut payload = vec![MBRUNWEI_CUSTOM_CHOICE, 2, player.raw(), skill.raw(), value];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        3 => {
            let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE);
            let valid: Vec<_> = cards
                .iter()
                .copied()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|state| {
                        Some(state.zone) == draw
                            && crate::query::effective_card_color_mask(tx.state(), player, *card)
                                == value as u8
                    })
                })
                .collect();
            if valid.is_empty()
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|target| target.alive)
            {
                return Err(EffectError::Illegal("润微 selection changed".into()));
            }
            let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            let mut events =
                move_cards_with_reason(tx, valid.clone(), hand, draw, CardMoveReason::Unspecified)?;
            let rest: Vec<_> = cards
                .into_iter()
                .filter(|card| !valid.contains(card))
                .collect();
            if !rest.is_empty() {
                events.extend(move_cards_with_reason(
                    tx,
                    rest,
                    discard,
                    draw,
                    CardMoveReason::Unspecified,
                )?);
            }
            let (remaining, enabled, mut tracked) = mbrunwei_state(tx, skill);
            let remaining = if remaining == 0 && !enabled {
                valid.len() as u32
            } else {
                remaining
            };
            if target == player {
                tracked.extend(valid);
                tracked.sort_by_key(|card| card.raw());
                tracked.dedup();
            }
            set_mbrunwei_state(tx, skill, remaining, enabled, &tracked);
            Ok(events)
        }
        4 => {
            let (remaining, enabled, tracked) = mbrunwei_state(tx, skill);
            let remaining = remaining.saturating_sub(cards.len() as u32);
            set_mbrunwei_state(tx, skill, remaining, enabled || remaining == 0, &tracked);
            Ok(Vec::new())
        }
        5 => {
            let (_, _, tracked) = mbrunwei_state(tx, skill);
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let cards: Vec<_> = tracked
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| Some(card.zone) == hand)
                })
                .collect();
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            if cards.is_empty() {
                Ok(Vec::new())
            } else {
                run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])
            }
        }
        _ => Err(EffectError::ContentFault("bad 润微 stage".into())),
    }
}

pub(crate) fn discard_revealed_mbrunwei_cards(
    tx: &mut Transaction,
    cards: &[CardId],
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE);
    let cards: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| Some(card.zone) == draw)
        })
        .collect();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    move_cards_with_reason(tx, cards, discard, draw, CardMoveReason::Unspecified)
}

pub(crate) fn resolve_mbshuanghuai(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    choice: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if stage == 0 {
        let mut options = vec![1];
        let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
        if discard
            .and_then(|zone| tx.state().zones.get(zone))
            .is_some_and(|zone| {
                zone.cards.iter().any(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind == CardKind::Tao)
                })
            })
        {
            options.push(2);
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                MBSHUANGHUAI_CUSTOM_CHOICE,
                player.raw(),
                target.raw(),
                skill.raw(),
            ],
        );
    }
    if !matches!(choice, 1 | 2) {
        return Err(EffectError::Illegal("bad 霜怀 choice".into()));
    }
    let previous = tx.state().skills.get(skill).and_then(|instance| {
        (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"SHH1").then(|| {
            PlayerId::new(u32::from_le_bytes(
                instance.state_blob[4..8].try_into().unwrap(),
            ))
        })
    });
    let mut blob = b"SHH1".to_vec();
    blob.extend_from_slice(&target.raw().to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
    let mut effects = if choice == 1 {
        vec![EffectRequest::PreventDamage {
            damage_id: tx.state().resolution.current_damage_id,
        }]
    } else {
        vec![EffectRequest::GainCardByKind {
            player: target,
            kind: CardKind::Tao.tag(),
        }]
    };
    if previous == Some(target) {
        effects.extend([
            EffectRequest::DrawCards { player, count: 1 },
            EffectRequest::DrawCards {
                player: target,
                count: 1,
            },
        ]);
    } else if previous.is_some() {
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: player,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    run_effects(tx, effects)
}

pub(crate) fn mbxiongtu_flags(tx: &Transaction, skill: engine_api::SkillId) -> u32 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"XTG1")
                .then(|| u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()))
        })
        .unwrap_or(0)
}

pub(crate) fn set_mbxiongtu_flags(tx: &mut Transaction, skill: engine_api::SkillId, flags: u32) {
    let instance = tx.state_mut().skills.get_mut(skill).unwrap();
    if flags == 0 {
        instance.state_blob.clear();
    } else {
        instance.state_blob = b"XTG1".to_vec();
        instance.state_blob.extend_from_slice(&flags.to_le_bytes());
    }
}

pub(crate) fn mbxiongtu_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    set_mbxiongtu_flags(tx, skill, mbxiongtu_flags(tx, skill) | 0b100);
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveMbxiongtu {
            player,
            target,
            stage: 4,
            card: CardId::NONE,
            cards: Vec::new(),
            skill,
        });
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
}

pub(crate) fn resolve_mbxiongtu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    card: CardId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let uses = tx.state().skills.get(skill).unwrap().play_phase_use_count;
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || player == target
                || uses == 0
                || uses > 2
                || (uses == 2 && mbxiongtu_flags(tx, skill) & 0b010 == 0)
            {
                return Err(EffectError::Illegal("凶图 is not available".into()));
            }
            let options =
                crate::command::play::target_zone_card_tokens(tx, target, true, false, false);
            if options.is_empty() {
                return Err(EffectError::Illegal("凶图 target has no hand card".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    MBXIONGTU_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            if !tx
                .state()
                .cards
                .get(card)
                .is_some_and(|state| state.owner == target && Some(state.zone) == hand)
            {
                return Err(EffectError::Illegal("凶图 revealed card changed".into()));
            }
            let count = 4usize.saturating_sub(
                (tx.state().turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK) as u32 & 0b1111)
                    .count_ones() as usize,
            );
            let mut events = vec![DomainEvent::CardsRevealed {
                cards: vec![card],
                to: Vec::new(),
            }];
            if count == 0 {
                events.extend(mbxiongtu_damage(tx, player, target, skill)?);
                return Ok(events);
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.len() < count {
                events.extend(resolve_mbxiongtu(
                    tx,
                    player,
                    target,
                    3,
                    card,
                    Vec::new(),
                    skill,
                )?);
                return Ok(events);
            }
            events.extend(open_choice(
                tx,
                player,
                count as u8,
                count as u8,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    MBXIONGTU_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )?);
            Ok(events)
        }
        2 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if cards.is_empty()
                || cards.iter().any(|selected| !options.contains(selected))
                || cards.len()
                    != 4usize.saturating_sub(
                        (tx.state().turn.flag(turn_flag::DISCARD_PILE_SUIT_MASK) as u32 & 0b1111)
                            .count_ones() as usize,
                    )
            {
                return Err(EffectError::Illegal("invalid 凶图 discard".into()));
            }
            let mut events = run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }])?;
            events.extend(mbxiongtu_damage(tx, player, target, skill)?);
            Ok(events)
        }
        3 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let valid = tx
                .state()
                .cards
                .get(card)
                .is_some_and(|state| state.owner == target && Some(state.zone) == hand);
            let mut events = Vec::new();
            if valid {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .unwrap();
                events.extend(move_cards_with_reason(
                    tx,
                    vec![card],
                    discard,
                    hand,
                    CardMoveReason::Discard,
                )?);
            }
            set_mbxiongtu_flags(tx, skill, mbxiongtu_flags(tx, skill) | 0b001);
            Ok(events)
        }
        4 => {
            set_mbxiongtu_flags(tx, skill, (mbxiongtu_flags(tx, skill) & !0b100) | 0b001);
            Ok(Vec::new())
        }
        5 => {
            let flags = mbxiongtu_flags(tx, skill);
            if flags & 0b001 == 0 || flags & 0b100 != 0 {
                return Ok(Vec::new());
            }
            set_mbxiongtu_flags(tx, skill, (flags & !0b001) | 0b010);
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
        }
        6 => {
            set_mbxiongtu_flags(tx, skill, mbxiongtu_flags(tx, skill) & !0b010);
            Ok(Vec::new())
        }
        7 => {
            set_mbxiongtu_flags(tx, skill, 0);
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad 凶图 stage".into())),
    }
}

pub(crate) fn resolve_mbganggeng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if player == target
                || cards.len() < 2
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || cards.iter().any(|card| {
                    !tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
                })
            {
                return Err(EffectError::Illegal("invalid 刚鲠 selection".into()));
            }
            let to = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
            let mut blob = b"GGG1".to_vec();
            blob.extend_from_slice(&target.raw().to_le_bytes());
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards,
                    to,
                    from: hand.unwrap(),
                }],
            )
        }
        1 => {
            let remembered = tx.state().skills.get(skill).and_then(|instance| {
                (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"GGG1").then(
                    || {
                        PlayerId::new(u32::from_le_bytes(
                            instance.state_blob[4..8].try_into().unwrap(),
                        ))
                    },
                )
            });
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .state_blob
                .clear();
            let Some(target) = remembered else {
                return Ok(Vec::new());
            };
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let target_hand = tx.state().hand_count(target);
            let is_max = tx
                .state()
                .players
                .iter_live()
                .filter(|participant| participant.alive)
                .all(|participant| tx.state().hand_count(participant.id) <= target_hand);
            if is_max {
                run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }])
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::ChooseDiscardTargetCard {
                        player,
                        target,
                        scope: TargetCardScope::HandEquipJudge,
                    }],
                )
            }
        }
        _ => Err(EffectError::ContentFault("bad 刚鲠 stage".into())),
    }
}

pub(crate) fn mbsijian_both_count(tx: &Transaction, skill: engine_api::SkillId) -> u32 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| {
            (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"SJN1")
                .then(|| u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()))
        })
        .unwrap_or(0)
}

pub(crate) fn set_mbsijian_both_count(tx: &mut Transaction, skill: engine_api::SkillId, count: u32) {
    let mut blob = b"SJN1".to_vec();
    blob.extend_from_slice(&count.to_le_bytes());
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
}

pub(crate) fn resolve_mbsijian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    choice: u8,
    helper_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let current = tx.state().turn.current;
    let current_alive = tx
        .state()
        .players
        .get(current)
        .is_some_and(|participant| participant.alive);
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| *target != player)
        .collect();
    match stage {
        0 => {
            let mut options = Vec::new();
            if !others.is_empty() {
                options.push(1);
            }
            if current_alive {
                options.push(2);
            }
            if (!others.is_empty() || current_alive)
                && !tx
                    .state()
                    .players
                    .iter_live()
                    .any(|participant| participant.dying)
            {
                options.push(3);
            }
            if options.is_empty() {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.turn_fire_count = instance.turn_fire_count.saturating_sub(1);
                }
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    MBSIJIAN_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    helper_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 if matches!(choice, 1 | 3) && !others.is_empty() => open_choice(
            tx,
            player,
            1,
            1,
            false,
            others.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                MBSIJIAN_CUSTOM_CHOICE,
                1,
                player.raw(),
                choice.into(),
                helper_def.raw(),
                skill.raw(),
            ],
        ),
        1 => resolve_mbsijian(tx, player, PlayerId::NONE, 2, choice, helper_def, skill),
        2 => {
            let mut effects = Vec::new();
            if choice != 2
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive && target != player)
            {
                effects.push(EffectRequest::AddSkill {
                    player: target,
                    skill_def: helper_def,
                });
            }
            if choice != 1 && current_alive {
                effects.push(EffectRequest::DrawCards {
                    player: current,
                    count: 2,
                });
            }
            if choice == 3 {
                let count = mbsijian_both_count(tx, skill);
                set_mbsijian_both_count(tx, skill, count.saturating_add(1));
                if count > 0 {
                    effects.push(EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: count,
                        is_lose_hp: true,
                        nature: rule_api::damage_nature::NORMAL,
                    });
                }
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("bad 死谏 stage".into())),
    }
}

pub(crate) fn mbjianji_choice_options(tx: &Transaction, participant: PlayerId, shared: CardId) -> Vec<u32> {
    let mut options: Vec<_> = tx
        .state()
        .hand_cards(participant)
        .into_iter()
        .map(CardId::raw)
        .collect();
    options.push(shared.raw());
    options.sort_unstable();
    options.dedup();
    options
}

pub(crate) fn resolve_mbjianji(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    target: PlayerId,
    stage: u8,
    card: CardId,
    _cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if source == target
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|role| role.alive)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|role| role.alive)
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.owner == player && Some(state.zone) == hand)
            {
                return Err(EffectError::Illegal("invalid 间计 selection".into()));
            }
            open_choice(
                tx,
                source,
                1,
                1,
                false,
                mbjianji_choice_options(tx, source, card),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    MBJIANJI_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    card.raw(),
                    source.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbjianji {
                    player,
                    source: PlayerId::NONE,
                    target: PlayerId::NONE,
                    stage: 4,
                    card,
                    cards: Vec::new(),
                    targets,
                    skill,
                });
            if source == PlayerId::NONE || target == PlayerId::NONE {
                Ok(Vec::new())
            } else {
                try_use_virtual_sha_on_without_distance(tx, source, target, true, 0)
            }
        }
        4 => {
            let targets: Vec<_> = targets
                .into_iter()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|role| role.alive)
                })
                .collect();
            if targets.is_empty() {
                Ok(Vec::new())
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamageEachInOrder {
                        source: player,
                        targets,
                        amount: 1,
                        skill,
                    }],
                )
            }
        }
        _ => Err(EffectError::ContentFault("bad 间计 stage".into())),
    }
}

pub(crate) fn commit_mbjianji_compare(
    tx: &mut Transaction,
    player: PlayerId,
    first: PlayerId,
    second: PlayerId,
    shared: CardId,
    first_card: CardId,
    second_card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) || first == second {
        return Err(EffectError::Illegal("间计 context changed".into()));
    }
    let valid = |participant: PlayerId, selected: CardId| {
        selected == shared
            || tx.state().cards.get(selected).is_some_and(|card| {
                card.owner == participant
                    && tx.state().zones.get(card.zone).is_some_and(|zone| {
                        zone.kind == ZoneKind::Hand && zone.owner == participant
                    })
            })
    };
    let shared_state = tx.state().cards.get(shared).filter(|card| {
        card.owner == player
            && tx
                .state()
                .zones
                .get(card.zone)
                .is_some_and(|zone| zone.kind == ZoneKind::Hand && zone.owner == player)
    });
    if !valid(first, first_card) || !valid(second, second_card) || shared_state.is_none() {
        return Err(EffectError::Illegal("invalid 间计 comparison card".into()));
    }
    let shared_is_sha = shared_state.unwrap().kind == CardKind::Sha;
    let first_rank = tx.state().cards.get(first_card).unwrap().rank;
    let second_rank = tx.state().cards.get(second_card).unwrap().rank;
    let (winner, loser) = match first_rank.cmp(&second_rank) {
        std::cmp::Ordering::Greater => (first, second),
        std::cmp::Ordering::Less => (second, first),
        std::cmp::Ordering::Equal => (PlayerId::NONE, PlayerId::NONE),
    };
    let mut damage_targets = Vec::new();
    if shared_is_sha {
        if first_card == shared {
            damage_targets.push(first);
        }
        if second_card == shared {
            damage_targets.push(second);
        }
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
    let mut effects = vec![EffectRequest::RevealCards {
        cards: if first_card == second_card {
            vec![first_card]
        } else {
            vec![first_card, second_card]
        },
        to: Vec::new(),
    }];
    let mut moved = Vec::new();
    for selected in [first_card, second_card] {
        if moved.contains(&selected) {
            continue;
        }
        moved.push(selected);
        let from = tx.state().cards.get(selected).unwrap().zone;
        effects.push(EffectRequest::MoveCards {
            cards: vec![selected],
            to: discard,
            from: Some(from),
        });
    }
    effects.extend([
        EffectRequest::EmitHandCompareFinished {
            player: first,
            target: second,
            player_card: first_card,
            target_card: second_card,
            winner,
        },
        EffectRequest::ResolveMbjianji {
            player,
            source: winner,
            target: loser,
            stage: 3,
            card: shared,
            cards: Vec::new(),
            targets: damage_targets,
            skill,
        },
    ]);
    run_effects(tx, effects)
}

pub(crate) fn mbfeijing_seat_paths(
    state: &crate::state::GameState,
    player: PlayerId,
    target: PlayerId,
) -> Option<(Vec<PlayerId>, Vec<PlayerId>)> {
    let order = state.players.alive_by_seat();
    let start = order.iter().position(|role| *role == player)?;
    let end = order.iter().position(|role| *role == target)?;
    if start == end || order.len() < 3 {
        return None;
    }
    let walk = |step: isize| {
        let mut path = Vec::new();
        let mut index = start;
        loop {
            index = (index as isize + step).rem_euclid(order.len() as isize) as usize;
            if index == end {
                break;
            }
            path.push(order[index]);
        }
        path
    };
    Some((walk(1), walk(-1)))
}
