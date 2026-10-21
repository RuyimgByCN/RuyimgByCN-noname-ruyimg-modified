//! Effect primitives for `twyuyan_gift_cards` and related helpers.

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
