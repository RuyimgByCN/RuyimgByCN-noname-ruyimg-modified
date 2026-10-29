//! Effect primitives for `resolve_stdgaobian` and related helpers.

use super::*;

pub(crate) fn resolve_stdgaobian(
    tx: &mut Transaction,
    player: PlayerId,
    chooser: PlayerId,
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
    if chooser.is_none() {
        // Exactly one alive player received damage this turn.
        let damaged: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| tx.state().turn.player_received_damage(candidate))
            .collect();
        if damaged.len() != 1 {
            return Ok(Vec::new());
        }
        let chooser = damaged[0];
        return open_choice(
            tx,
            chooser,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                RESOLVE_STDGAOBIAN_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                chooser.raw(),
                0,
                0,
            ],
        );
    }
    if !tx
        .state()
        .players
        .get(chooser)
        .is_some_and(|participant| participant.alive)
        || !tx.state().turn.player_received_damage(chooser)
    {
        return Err(EffectError::Illegal(
            "Stdgaobian chooser context changed".into(),
        ));
    }
    // Stage 1: the damaged player picks a Sha target.
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != chooser)
        .map(PlayerId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        chooser,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        vec![
            RESOLVE_STDGAOBIAN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            chooser.raw(),
            1,
            0,
        ],
    )
}

pub(crate) fn resolve_dczhiheng(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        options.len().min(u8::MAX as usize) as u8,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![RESOLVE_DCZHIHENG_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_dcdanji(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state().skills.get(skill) else {
        return Err(EffectError::Illegal(
            "Dcdanji skill instance vanished".into(),
        ));
    };
    if instance.owner != player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Dcdanji awakening context invalid".into(),
        ));
    }
    run_effects(
        tx,
        vec![
            EffectRequest::RemoveSkill { skill },
            EffectRequest::ChangeMaxHp { player, delta: -1 },
        ],
    )?;
    let num = {
        let p = tx
            .state()
            .players
            .get(player)
            .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
        (p.max_hp as i32 - p.hp).max(0) as u32
    };
    if num > 0 {
        run_effects(
            tx,
            vec![EffectRequest::RecoverHp {
                source: player,
                player,
                amount: num,
            }],
        )?;
    }
    run_effects(
        tx,
        vec![
            EffectRequest::DrawCardsFromSkill {
                player,
                count: num,
                skill,
            },
            EffectRequest::AddSkill {
                player,
                skill_def: engine_api::SkillDefId::new(DCDANJI_MASHU_DEF),
            },
            EffectRequest::AddSkill {
                player,
                skill_def: engine_api::SkillDefId::new(DCDANJI_DCNUCHEN_DEF),
            },
        ],
    )
}

pub(crate) fn resolve_dcnuchen(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| candidate != player && tx.state().hand_count(candidate) > 0)
            .map(PlayerId::raw)
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                RESOLVE_DCNUCHEN_CUSTOM_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                0,
                0,
            ],
        );
    }
    let target_hand = tx.state().hand_cards(target);
    if target_hand.is_empty() {
        return Err(EffectError::Illegal("Dcnuchen target has no hand".into()));
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        target_hand.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            RESOLVE_DCNUCHEN_CUSTOM_CHOICE,
            1,
            player.raw(),
            skill.raw(),
            target.raw(),
            0,
        ],
    )
}

pub(crate) fn resolve_hfjieying(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    grant_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if grant_def.is_none()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| candidate != player)
            .map(PlayerId::raw)
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                RESOLVE_HFJIEYING_CUSTOM_CHOICE,
                player.raw(),
                grant_def.raw(),
                skill.raw(),
            ],
        );
    }
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: grant_def,
        }],
    )
}

pub(crate) fn resolve_dctingxian(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || card.is_none()
        || targets.is_empty()
    {
        return Ok(Vec::new());
    }
    let equip_count = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| tx.state().zones.get(zone))
        .map(|zone| zone.cards.len() as u32)
        .unwrap_or(0);
    let max = equip_count + 1;
    let options: Vec<_> = targets
        .into_iter()
        .filter(|&target| {
            tx.state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        })
        .map(PlayerId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let cap = (options.len() as u32).min(max);
    let mut events = run_effects(tx, vec![EffectRequest::DrawCards { player, count: max }])?;
    let choice_max = u8::try_from(cap).unwrap_or(u8::MAX);
    events.extend(open_choice(
        tx,
        player,
        1,
        choice_max,
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        vec![
            RESOLVE_DCTINGXIAN_CUSTOM_CHOICE,
            player.raw(),
            card.raw(),
            skill.raw(),
        ],
    )?);
    Ok(events)
}

pub(crate) fn resolve_lulve(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    branch: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.current != player
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        // Stage 0: choose another player with fewer (non-empty) hand cards.
        let owner_hand = tx.state().hand_count(player);
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| {
                candidate != player
                    && tx.state().hand_count(candidate) > 0
                    && tx.state().hand_count(candidate) < owner_hand
            })
            .map(PlayerId::raw)
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![RESOLVE_LULVE_CUSTOM_CHOICE, player.raw(), skill.raw()],
        );
    }
    if branch == 2 {
        // Stage 1: the target chooses the branch.
        return open_choice(
            tx,
            target,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                RESOLVE_LULVE_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                target.raw(),
            ],
        );
    }
    // Stage 2: apply the chosen branch.
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("owner hand zone missing".into()))?;
    let mut events = Vec::new();
    if branch == 0 {
        let target_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::Illegal("target hand zone missing".into()))?;
        let cards: Vec<_> = tx
            .state()
            .zones
            .get(target_hand)
            .map(|zone| zone.cards.clone())
            .unwrap_or_default();
        events.extend(run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards,
                to: hand,
                from: Some(target_hand),
            }],
        )?);
        events.extend(run_effects(tx, vec![EffectRequest::TurnOver { player }])?);
    } else {
        events.extend(run_effects(
            tx,
            vec![EffectRequest::TurnOver { player: target }],
        )?);
        events.extend(run_effects(
            tx,
            vec![EffectRequest::UseVirtualCardOn {
                player: target,
                kind: CardKind::Sha.tag(),
                target: player,
            }],
        )?);
    }
    Ok(events)
}

pub(crate) fn resolve_stdjujian(
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
            .skills
            .get(skill)
            .is_some_and(|instance| instance.turn_fire_count == 0)
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| candidate != player)
            .map(PlayerId::raw)
            .collect();
        if options.is_empty() {
            return Ok(Vec::new());
        }
        let mut payload = vec![RESOLVE_STDJUJIAN_CUSTOM_CHOICE, player.raw(), skill.raw()];
        payload.extend(cards.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            payload,
        );
    }
    // Apply: the selected target gains the used cards still in the discard pile.
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.turn_fire_count = instance.turn_fire_count.saturating_add(1);
    }
    gain_event_cards(tx, target, &cards)
}

pub(crate) fn resolve_jieliang_gain(
    tx: &mut Transaction,
    player: PlayerId,
    mover: PlayerId,
    cards: Vec<CardId>,
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
    // Only the current phase player's discard-phase discards qualify.
    if mover != tx.state().turn.current
        || mover == player
        || tx.state().turn.phase != Phase::Discard
    {
        return Ok(Vec::new());
    }
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    let options: Vec<_> = cards
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card_state| discard.is_some_and(|zone| card_state.zone == zone))
        })
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            RESOLVE_JIELIANG_GAIN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn discard_hand_equip_count(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if (options.len() as u32) < count {
        return Ok(Vec::new());
    }
    let max = u8::try_from(count).unwrap_or(u8::MAX);
    open_choice(
        tx,
        player,
        max,
        max,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            DISCARD_HAND_EQUIP_COUNT_CUSTOM_CHOICE,
            player.raw(),
            count,
            skill.raw(),
        ],
    )
}

pub(crate) fn choose_discard_hand_range(
    tx: &mut Transaction,
    player: PlayerId,
    min: u32,
    max: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if min == 0
        || max < min
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options = standard_ganglie_discardable_hand_cards(tx, player);
    if (options.len() as u32) < min {
        return Ok(Vec::new());
    }
    let actual_max = max.min(options.len() as u32).min(u8::MAX as u32) as u8;
    open_choice(
        tx,
        player,
        min.min(u8::MAX as u32) as u8,
        actual_max,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            CHOOSE_DISCARD_HAND_RANGE_CHOICE,
            player.raw(),
            min,
            max,
            skill.raw(),
        ],
    )
}

pub(crate) fn dcctjiuxian_recovery_targets(
    tx: &Transaction,
    player: PlayerId,
    damaged: PlayerId,
) -> Vec<PlayerId> {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| {
            *candidate != player
                && tx
                    .state()
                    .players
                    .get(*candidate)
                    .is_some_and(|participant| {
                        participant.alive && participant.hp < participant.max_hp as i32
                    })
                && crate::query::player_in_attack_range(tx.state(), damaged, *candidate)
        })
        .collect()
}

pub(crate) fn resolve_dcctjiuxian(
    tx: &mut Transaction,
    player: PlayerId,
    event_player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action != 6 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let count = hand.len().div_ceil(2);
            let exact = u8::try_from(count).map_err(|_| {
                EffectError::ContentFault("Dcctjiuxian hand exceeds choice protocol".into())
            })?;
            open_choice(
                tx,
                player,
                exact,
                exact,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DCCTJIUXIAN_CHOICE,
                    0,
                    player.raw(),
                    count as u32,
                    skill.raw(),
                ],
            )
        }
        1 => run_effects(
            tx,
            vec![
                EffectRequest::RecastCards { player, cards },
                EffectRequest::ResolveDcctjiuxian {
                    player,
                    event_player: player,
                    target: PlayerId::NONE,
                    card: CardId::NONE,
                    action: 5,
                    cards: Vec::new(),
                    skill,
                },
            ],
        ),
        2 => {
            if !tx
                .state()
                .players
                .get(event_player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let candidates = dcctjiuxian_recovery_targets(tx, player, event_player);
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                candidates.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DCCTJIUXIAN_CHOICE,
                    2,
                    player.raw(),
                    event_player.raw(),
                    skill.raw(),
                ],
            )
        }
        3 => {
            if !dcctjiuxian_recovery_targets(tx, player, event_player).contains(&target) {
                return Err(EffectError::Illegal(
                    "Dcctjiuxian recovery target is stale".into(),
                ));
            }
            recover_hp(tx, player, target, 1)
        }
        5 => choose_use_virtual_card_impl(
            tx,
            player,
            CardKind::TrickJuedou.tag(),
            skill,
            VirtualCardTracking::Dcctjiuxian,
            false,
        ),
        6 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.take_tracked_cards(&[card]);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown Dcctjiuxian action".into(),
        )),
    }
}
