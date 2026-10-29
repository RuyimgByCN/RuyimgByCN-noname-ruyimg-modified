//! Command operations for `resolve_choose_give_two_different_colors_or_recover` and related helpers.

use super::*;

pub(crate) fn resolve_resolve_tianxiang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 10 && payload.len() != 11 {
        return Err(EffectError::EngineFault(
            "bad Tianxiang continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let draw_skill_def = engine_api::SkillDefId::new(payload[2]);
    let source = PlayerId::new(payload[3]);
    let amount = payload[4];
    let nature = payload[5] as u8;
    let card = engine_api::CardId::new(payload[6]);
    let card_kind = payload[7] as u8;
    let card_flags = payload[8] as u8;
    let skill = engine_api::SkillId::new(payload[9]);
    if observer != player
        || cancelled
        || tx.state().players.get(player).is_none()
        || draw_skill_def.raw() == 0
        || source.is_none()
        || amount == 0
    {
        return Err(EffectError::Illegal(
            "Tianxiang redirect context changed".into(),
        ));
    }
    if payload.len() == 10 {
        let [selected] = selections else {
            return Err(EffectError::Illegal(
                "Tianxiang requires exactly one heart card".into(),
            ));
        };
        let selected = engine_api::CardId::new(*selected);
        let valid_heart = tx.state().cards.get(selected).is_some_and(|card_state| {
            card_state.suit == engine_api::Suit::Heart
                && crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .contains(&selected)
        });
        if !valid_heart {
            return Err(EffectError::Illegal(
                "Tianxiang heart discard cost is invalid or stale".into(),
            ));
        }
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != player)
            .map(PlayerId::raw)
            .collect();
        if targets.is_empty() {
            return Err(EffectError::Illegal(
                "Tianxiang has no redirect targets".into(),
            ));
        }
        let mut next = payload.to_vec();
        next.push(selected.raw());
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            engine_api::ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            next,
        )
        .map(|_| ());
    }
    let [selected_target] = selections else {
        return Err(EffectError::Illegal(
            "Tianxiang requires exactly one redirect target".into(),
        ));
    };
    let target = PlayerId::new(*selected_target);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Tianxiang redirect target is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveTianxiang {
            player,
            target,
            discard_card: engine_api::CardId::new(payload[10]),
            source,
            amount,
            nature,
            card,
            card_kind,
            card_flags,
            draw_skill_def,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_retianxiang(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if !(4..=6).contains(&payload.len()) {
        return Err(EffectError::EngineFault(
            "bad Retianxiang continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let source = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player || cancelled || tx.state().players.get(player).is_none() {
        return Err(EffectError::Illegal(
            "Retianxiang branch context changed".into(),
        ));
    }
    if payload.len() == 4 {
        let [selected] = selections else {
            return Err(EffectError::Illegal(
                "Retianxiang requires exactly one heart card".into(),
            ));
        };
        let selected = engine_api::CardId::new(*selected);
        let valid_heart = tx.state().cards.get(selected).is_some_and(|card_state| {
            card_state.suit == engine_api::Suit::Heart
                && crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .contains(&selected)
        });
        if !valid_heart {
            return Err(EffectError::Illegal(
                "Retianxiang heart discard cost is invalid or stale".into(),
            ));
        }
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| *target != player)
            .map(PlayerId::raw)
            .collect();
        if targets.is_empty() {
            return Err(EffectError::Illegal(
                "Retianxiang has no branch targets".into(),
            ));
        }
        let mut next = payload.to_vec();
        next.push(selected.raw());
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            engine_api::ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            next,
        )
        .map(|_| ());
    }
    if payload.len() == 5 {
        let [selected_target] = selections else {
            return Err(EffectError::Illegal(
                "Retianxiang requires exactly one branch target".into(),
            ));
        };
        let target = PlayerId::new(*selected_target);
        if target == player
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "Retianxiang branch target is invalid or stale".into(),
            ));
        }
        let mut next = payload.to_vec();
        next.push(target.raw());
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            engine_api::ChoiceKind::Custom,
            ContinuationKind::Custom,
            next,
        )
        .map(|_| ());
    }
    let [selected_branch] = selections else {
        return Err(EffectError::Illegal(
            "Retianxiang requires exactly one branch".into(),
        ));
    };
    let branch = (*selected_branch).min(1) as u8;
    let target = PlayerId::new(payload[5]);
    if target == player {
        return Err(EffectError::Illegal(
            "Retianxiang branch target is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveRetianxiang {
            player,
            target,
            discard_card: engine_api::CardId::new(payload[4]),
            branch,
            source,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_lianzhou(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("bad Lianzhou continuation".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player
        || cancelled
        || selections.is_empty()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("Lianzhou link context changed".into()));
    }
    let owner_hp = tx.state().players.get(player).unwrap().hp;
    let targets: Vec<_> = selections.iter().map(|raw| PlayerId::new(*raw)).collect();
    let valid = targets.iter().all(|&target| {
        target != player
            && tx.state().players.get(target).is_some_and(|participant| {
                participant.alive
                    && participant.hp == owner_hp
                    && !participant
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
            })
    });
    if !valid {
        return Err(EffectError::Illegal(
            "Lianzhou link targets are invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveLianzhou {
            player,
            targets,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_resolve_stddechong(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad Stddechong continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player
        || cancelled
        || selections.is_empty()
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
        || player == target
    {
        return Err(EffectError::Illegal(
            "Stddechong gift context changed".into(),
        ));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(|card| card.raw())
            .collect();
    let cards: Vec<_> = selections
        .iter()
        .map(|raw| engine_api::CardId::new(*raw))
        .collect();
    let valid = cards.iter().all(|card| {
        allowed.contains(&card.raw())
            && tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player)
    });
    if !valid {
        return Err(EffectError::Illegal(
            "Stddechong gift cards are invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveStddechong {
            player,
            target,
            cards,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_discard_hand_equip_count(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad discard-count continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let count = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player
        || cancelled
        || count == 0
        || selections.len() as u32 != count
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "discard-count choice context changed".into(),
        ));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(|card| card.raw())
            .collect();
    let cards: Vec<_> = selections
        .iter()
        .map(|raw| engine_api::CardId::new(*raw))
        .collect();
    let valid = cards.iter().all(|card| {
        allowed.contains(&card.raw())
            && tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|state| state.owner == player)
    });
    if !valid {
        return Err(EffectError::Illegal(
            "discard-count cards are invalid or stale".into(),
        ));
    }
    let _ = skill;
    run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).map(|_| ())
}

pub(crate) fn resolve_choose_discard_hand_range(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 {
        return Err(EffectError::EngineFault(
            "bad discard-hand-range continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let min = payload[2];
    let max = payload[3];
    let count = selections.len() as u32;
    if observer != player
        || cancelled
        || min == 0
        || max < min
        || count < min
        || count > max
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "discard-hand-range choice context changed".into(),
        ));
    }
    let allowed: std::collections::BTreeSet<_> =
        crate::effect::standard_ganglie_discardable_hand_cards(tx, player)
            .into_iter()
            .map(|card| card.raw())
            .collect();
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if selected.len() != selections.len()
        || count > allowed.len() as u32
        || !selected.iter().all(|card| allowed.contains(card))
    {
        return Err(EffectError::Illegal(
            "discard-hand-range cards are invalid or stale".into(),
        ));
    }
    let cards = selections
        .iter()
        .copied()
        .map(engine_api::CardId::new)
        .collect();
    run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).map(|_| ())
}

pub(crate) fn resolve_dcctjiuxian_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .get(1)
        .ok_or_else(|| EffectError::EngineFault("truncated Dcctjiuxian choice".into()))?;
    match stage {
        0 if payload.len() == 5 => {
            let player = PlayerId::new(payload[2]);
            let count = payload[3] as usize;
            let skill = engine_api::SkillId::new(payload[4]);
            let hand: std::collections::BTreeSet<_> =
                tx.state().hand_cards(player).into_iter().collect();
            let cards: Vec<_> = selections
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect();
            let distinct: std::collections::BTreeSet<_> = cards.iter().copied().collect();
            if observer != player
                || cancelled
                || count == 0
                || cards.len() != count
                || distinct.len() != cards.len()
                || !cards.iter().all(|card| hand.contains(card))
            {
                return Err(EffectError::Illegal(
                    "Dcctjiuxian recast choice is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcctjiuxian {
                    player,
                    event_player: player,
                    target: PlayerId::NONE,
                    card: engine_api::CardId::NONE,
                    action: 1,
                    cards,
                    skill,
                }],
            )
            .map(|_| ())
        }
        2 if payload.len() == 5 => {
            let player = PlayerId::new(payload[2]);
            let damaged = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            if observer != player {
                return Err(EffectError::Illegal(
                    "only the Dcctjiuxian owner may choose recovery".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "Dcctjiuxian recovery chooses at most one target".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            if !crate::effect::dcctjiuxian_recovery_targets(tx, player, damaged).contains(&target) {
                return Err(EffectError::Illegal(
                    "Dcctjiuxian recovery target is invalid or stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcctjiuxian {
                    player,
                    event_player: damaged,
                    target,
                    card: engine_api::CardId::NONE,
                    action: 3,
                    cards: Vec::new(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "bad Dcctjiuxian continuation".into(),
        )),
    }
}

pub(crate) fn resolve_bianyue_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault("truncated Bianyue choice".into()));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "Bianyue target choice is invalid".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    if !crate::effect::bianyue_wuxie_targets(tx, player).contains(&target) {
        return Err(EffectError::Illegal(
            "Bianyue target choice is stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveBianyue {
            player,
            target,
            action: 11,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_zhupeilan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let mode = *payload.get(1).unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal("Zhupeilan choice is invalid".into()));
    }
    match mode {
        0 => {
            if payload.len() < 7 {
                return Err(EffectError::EngineFault("truncated Cilv choice".into()));
            }
            let count = payload[6] as usize;
            if payload.len() != 7 + count || selections[0] > 2 {
                return Err(EffectError::Illegal("malformed Cilv choice".into()));
            }
            let card = engine_api::CardId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let helper_skill_def = engine_api::SkillDefId::new(payload[5]);
            let event_cards = payload[7..]
                .iter()
                .copied()
                .map(engine_api::CardId::new)
                .collect();
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhupeilan {
                    player,
                    target: PlayerId::NONE,
                    card,
                    helper_skill_def,
                    action: 3 + selections[0] as u8,
                    event_cards,
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() != 4 {
                return Err(EffectError::EngineFault("truncated Tongdao choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            if !tx.state().players.alive_by_seat().contains(&target) {
                return Err(EffectError::Illegal("Tongdao target is stale".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveZhupeilan {
                    player,
                    target,
                    card: engine_api::CardId::NONE,
                    helper_skill_def: engine_api::SkillDefId::NONE,
                    action: 11,
                    event_cards: Vec::new(),
                    skill: engine_api::SkillId::new(payload[3]),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault("unknown Zhupeilan choice".into())),
    }
}

pub(crate) fn resolve_resolve_jieliang_gain(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad Jieliang gain continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    if observer != player
        || cancelled
        || selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Jieliang gain choice context changed".into(),
        ));
    }
    let card = engine_api::CardId::new(selections[0]);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let in_discard = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card_state| card_state.zone == discard);
    if !in_discard {
        return Err(EffectError::Illegal(
            "Jieliang gain card is no longer in the discard pile".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("gainer hand zone missing".into()))?;
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

pub(crate) fn resolve_resolve_stdjujian(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "bad Stdjujian continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let cards: Vec<_> = payload[3..]
        .iter()
        .map(|raw| engine_api::CardId::new(*raw))
        .collect();
    if observer != player
        || cancelled
        || selections.len() != 1
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Stdjujian gain context changed".into(),
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
            "Stdjujian gain target is invalid or stale".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ResolveStdjujian {
            player,
            target,
            cards,
            skill,
        }],
    )
    .map(|_| ())
}
