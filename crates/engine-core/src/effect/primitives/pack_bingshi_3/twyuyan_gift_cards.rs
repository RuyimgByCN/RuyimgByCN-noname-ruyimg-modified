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
