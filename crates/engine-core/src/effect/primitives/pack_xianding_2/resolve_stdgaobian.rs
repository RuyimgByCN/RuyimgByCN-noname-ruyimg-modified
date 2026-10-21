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

pub(crate) fn bizu_groups(
    instance: &crate::state::SkillInstance,
    round: u32,
) -> Vec<Vec<PlayerId>> {
    if instance.state_blob.len() < 8
        || instance.state_blob.get(..4) != Some(b"BZY1")
        || u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()) != round
    {
        return Vec::new();
    }
    let mut groups = Vec::new();
    let mut offset = 8;
    while let Some(&count) = instance.state_blob.get(offset) {
        offset += 1;
        let bytes = usize::from(count) * 4;
        let Some(slice) = instance.state_blob.get(offset..offset + bytes) else {
            return Vec::new();
        };
        groups.push(
            slice
                .chunks_exact(4)
                .map(|chunk| PlayerId::new(u32::from_le_bytes(chunk.try_into().unwrap())))
                .collect(),
        );
        offset += bytes;
    }
    groups
}

pub(crate) fn record_bizu_group(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    targets: &[PlayerId],
) -> Result<bool, EffectError> {
    let round = tx.state().turn.round;
    let instance = tx
        .state()
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("Bizu skill missing".into()))?;
    let mut canonical = targets.to_vec();
    canonical.sort_by_key(|target| target.raw());
    canonical.dedup();
    if canonical.len() > u8::MAX as usize {
        return Err(EffectError::ContentFault(
            "Bizu target group exceeds choice protocol".into(),
        ));
    }
    let repeated = bizu_groups(instance, round).contains(&canonical);
    let instance = tx.state_mut().skills.get_mut(skill).unwrap();
    if instance.state_blob.get(..4) != Some(b"BZY1")
        || instance.state_blob.get(4..8) != Some(round.to_le_bytes().as_slice())
    {
        instance.state_blob = b"BZY1".to_vec();
        instance.state_blob.extend_from_slice(&round.to_le_bytes());
    }
    instance.state_blob.push(canonical.len() as u8);
    for target in canonical {
        instance
            .state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
    }
    Ok(repeated)
}

pub(crate) fn bianyue_wuxie_targets(tx: &Transaction, player: PlayerId) -> Vec<PlayerId> {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| *target != player && tx.state().hand_count(*target) > 0)
        .collect()
}

pub(crate) fn resolve_bianyue(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if matches!(action, 0 | 10) && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_count(player);
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| tx.state().hand_count(*target) == hand)
                .collect();
            let repeated = record_bizu_group(tx, skill, &targets)?;
            let mut effects: Vec<_> = targets
                .into_iter()
                .map(|player| EffectRequest::DrawCards { player, count: 1 })
                .collect();
            if repeated {
                effects.push(EffectRequest::BanActiveUntilTurnEnd { player, skill });
                effects.push(EffectRequest::RecoverHp {
                    source: player,
                    player,
                    amount: 1,
                });
            }
            run_effects(tx, effects)
        }
        10 => {
            if tx.state().hand_count(player) == 0 {
                return Ok(Vec::new());
            }
            let options = bianyue_wuxie_targets(tx, player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![RESOLVE_BIANYUE_CHOICE, player.raw(), skill.raw()],
            )
        }
        11 => {
            if tx.state().hand_count(player) == 0
                || !bianyue_wuxie_targets(tx, player).contains(&target)
            {
                return Err(EffectError::Illegal(
                    "Wuxie hand-swap target is stale".into(),
                ));
            }
            let player_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Wuxie owner hand missing".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("Wuxie target hand missing".into()))?;
            let player_cards = tx.state().hand_cards(player);
            let target_cards = tx.state().hand_cards(target);
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: player_cards,
                        to: target_hand,
                        from: Some(player_hand),
                    },
                    EffectRequest::MoveCards {
                        cards: target_cards,
                        to: player_hand,
                        from: Some(target_hand),
                    },
                    EffectRequest::ResolveBianyue {
                        player,
                        target,
                        action: 12,
                        skill,
                    },
                ],
            )
        }
        12 => {
            let cards: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|value| {
                        let kind = crate::query::effective_card_kind(tx.state(), player, value);
                        kind.is_sha() || kind.is_damage_trick()
                    })
                })
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let hand = tx.state().cards.get(cards[0]).unwrap().zone;
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            let events = move_cards(tx, cards.clone(), draw, Some(hand))?;
            let zone = tx.state_mut().zones.get_mut(draw).unwrap();
            zone.cards.retain(|candidate| !cards.contains(candidate));
            zone.cards.splice(0..0, cards);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown Bianyue action".into())),
    }
}

pub(crate) fn cilv_decode(blob: &[u8]) -> (u8, Vec<(CardId, u8, Vec<CardId>)>) {
    if blob.len() < 5 || blob.get(..4) != Some(CILV_STATE_TAG.as_slice()) {
        return (0, Vec::new());
    }
    let mut records = Vec::new();
    let mut offset = 5usize;
    while offset + 7 <= blob.len() {
        let card = CardId::new(u32::from_le_bytes(
            blob[offset..offset + 4].try_into().unwrap(),
        ));
        let mode = blob[offset + 4];
        let count = u16::from_le_bytes(blob[offset + 5..offset + 7].try_into().unwrap()) as usize;
        offset += 7;
        let Some(end) = offset.checked_add(count.saturating_mul(4)) else {
            return (blob[4] & 0b111, Vec::new());
        };
        if end > blob.len() {
            return (blob[4] & 0b111, Vec::new());
        }
        let cards = blob[offset..end]
            .chunks_exact(4)
            .map(|bytes| CardId::new(u32::from_le_bytes(bytes.try_into().unwrap())))
            .collect();
        records.push((card, mode, cards));
        offset = end;
    }
    if offset != blob.len() {
        return (blob[4] & 0b111, Vec::new());
    }
    (blob[4] & 0b111, records)
}

pub(crate) fn cilv_encode(used: u8, records: &[(CardId, u8, Vec<CardId>)]) -> Vec<u8> {
    let mut blob = CILV_STATE_TAG.to_vec();
    blob.push(used & 0b111);
    for (card, mode, cards) in records {
        blob.extend_from_slice(&card.raw().to_le_bytes());
        blob.push(*mode & 0b110);
        blob.extend_from_slice(&(cards.len().min(u16::MAX as usize) as u16).to_le_bytes());
        for card in cards.iter().take(u16::MAX as usize) {
            blob.extend_from_slice(&card.raw().to_le_bytes());
        }
    }
    blob
}

pub(crate) fn cilv_used_mask(instance: &crate::state::SkillInstance) -> u8 {
    cilv_decode(&instance.state_blob).0
}

pub(crate) fn cilv_tracks_card(
    instance: &crate::state::SkillInstance,
    card: CardId,
    mode: u8,
) -> bool {
    cilv_decode(&instance.state_blob)
        .1
        .iter()
        .any(|(tracked, tracked_mode, _)| *tracked == card && *tracked_mode & mode != 0)
}

pub(crate) fn cilv_apply_choice(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    choice: u8,
    event_cards: Vec<CardId>,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state().skills.get(skill) else {
        return Err(EffectError::Illegal("Cilv skill disappeared".into()));
    };
    if instance.owner != player || choice > 2 {
        return Err(EffectError::Illegal("invalid Cilv choice".into()));
    }
    let (used, _) = cilv_decode(&instance.state_blob);
    let bit = 1u8 << choice;
    if used & bit != 0 {
        return Err(EffectError::Illegal(
            "Cilv choice was already removed".into(),
        ));
    }
    let current = tx
        .state()
        .resolution
        .pending_card_resolutions
        .iter()
        .rev()
        .find(|record| record.card == card && record.targets.contains(&player));
    if current.is_none() {
        return Err(EffectError::Illegal("Cilv trick context changed".into()));
    }
    tx.state_mut().skills.get_mut(skill).unwrap().state_blob = cilv_encode(used | bit, &[]);
    if choice == 0 {
        crate::command::play::exclude_saved_card_target(tx, card, player)?;
        return Ok(Vec::new());
    }
    if helper_skill_def.is_none()
        || !tx
            .state()
            .content
            .skill_metadata(helper_skill_def)
            .is_some_and(|metadata| metadata.non_gainable)
    {
        return Err(EffectError::ContentFault(
            "Cilv requires a non-gainable helper".into(),
        ));
    }
    let events = add_skill(tx, player, helper_skill_def)?;
    let helper = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|owner| owner.skills.iter().copied())
        .find(|candidate| {
            tx.state()
                .skills
                .get(*candidate)
                .is_some_and(|instance| instance.def == helper_skill_def)
        })
        .ok_or_else(|| EffectError::EngineFault("Cilv helper was not added".into()))?;
    let (_, mut records) = cilv_decode(&tx.state().skills.get(helper).unwrap().state_blob);
    let mut cards = event_cards;
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    records.push((card, bit, cards));
    tx.state_mut().skills.get_mut(helper).unwrap().state_blob = cilv_encode(0, &records);
    Ok(events)
}

pub(crate) fn reset_player_starting_skills(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participant = tx
        .state()
        .players
        .get(player)
        .cloned()
        .ok_or_else(|| EffectError::Illegal("Tongdao target missing".into()))?;
    let character = tx
        .state()
        .content
        .character(participant.character_def)
        .cloned()
        .ok_or_else(|| EffectError::ContentFault("Tongdao target character missing".into()))?;
    let removable: Vec<_> = participant
        .skills
        .iter()
        .copied()
        .filter(|skill| {
            tx.state().skills.get(*skill).is_some_and(|instance| {
                !tx.state()
                    .content
                    .skill_metadata(instance.def)
                    .is_some_and(|metadata| metadata.non_gainable)
            })
        })
        .collect();
    let removed_defs: std::collections::BTreeSet<_> = removable
        .iter()
        .filter_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .map(|instance| instance.def.raw())
        })
        .collect();
    let piles: Vec<_> = tx
        .state()
        .skill_piles
        .iter()
        .copied()
        .filter(|pile| pile.owner == player && removed_defs.contains(&pile.skill_def))
        .collect();
    let mut events = Vec::new();
    for skill in removable {
        events.extend(remove_skill(tx, skill)?);
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    for pile in piles {
        let cards = tx
            .state()
            .zones
            .get(pile.zone)
            .map(|zone| zone.cards.clone())
            .unwrap_or_default();
        if !cards.is_empty() {
            events.extend(move_cards_with_reason(
                tx,
                cards,
                discard,
                Some(pile.zone),
                CardMoveReason::Discard,
            )?);
        }
        tx.state_mut()
            .skill_piles
            .retain(|candidate| candidate.zone != pile.zone);
    }
    for raw in character.skills {
        let def = engine_api::SkillDefId::new(raw);
        let allowed = tx
            .state()
            .content
            .skill_metadata(def)
            .is_some_and(|metadata| {
                !metadata.non_gainable
                    && (!metadata.lord_only || participant.identity == Identity::Lord)
            });
        if allowed {
            events.extend(add_skill(tx, player, def)?);
        }
    }
    Ok(events)
}

pub(crate) fn resolve_zhupeilan(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    action: u8,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            let used = cilv_used_mask(instance);
            let current = tx
                .state()
                .resolution
                .pending_card_resolutions
                .iter()
                .rev()
                .find(|record| {
                    record.card == card
                        && record.source == target
                        && record.targets.contains(&player)
                        && CardKind::from_tag(record.kind)
                            .is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick())
                });
            if used == 0b111 || current.is_none() {
                return Ok(Vec::new());
            }
            let cards = if event_cards.is_empty() {
                vec![card]
            } else {
                event_cards
            };
            run_effects(
                tx,
                vec![
                    EffectRequest::DrawCards {
                        player,
                        count: 3 - used.count_ones(),
                    },
                    EffectRequest::ResolveZhupeilan {
                        player,
                        target,
                        card,
                        helper_skill_def,
                        action: 6,
                        event_cards: cards,
                        skill,
                    },
                ],
            )
        }
        1 => {
            let tracked = tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| cilv_tracks_card(instance, card, 0b010));
            let damage_id = tx.state().resolution.current_damage_id;
            if !tracked || damage_id == 0 {
                return Ok(Vec::new());
            }
            run_effects(tx, vec![EffectRequest::PreventDamage { damage_id }])
        }
        2 => {
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            let (_, records) = cilv_decode(&instance.state_blob);
            let mut gain = Vec::new();
            let mut keep = Vec::new();
            for (tracked, mode, cards) in records {
                if tracked == card {
                    if mode & 0b100 != 0 {
                        gain.extend(cards);
                    }
                } else {
                    keep.push((tracked, mode, cards));
                }
            }
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob = cilv_encode(0, &keep);
            gain.sort_by_key(|card| card.raw());
            gain.dedup();
            let mut events = gain_event_cards(tx, player, &gain)?;
            if keep.is_empty() && tx.state().skills.get(skill).is_some() {
                events.extend(remove_skill(tx, skill)?);
            }
            Ok(events)
        }
        3..=5 => cilv_apply_choice(
            tx,
            player,
            card,
            action - 3,
            event_cards,
            helper_skill_def,
            skill,
        ),
        6 => {
            let Some(owner) = tx.state().players.get(player) else {
                return Ok(Vec::new());
            };
            let Some(instance) = tx.state().skills.get(skill) else {
                return Ok(Vec::new());
            };
            let used = cilv_used_mask(instance);
            if owner.max_hp as usize >= tx.state().hand_count(player) as usize || used == 0b111 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = (0..3).filter(|choice| used & (1 << choice) == 0).collect();
            let mut payload = vec![
                RESOLVE_ZHUPEILAN_CHOICE,
                0,
                player.raw(),
                card.raw(),
                skill.raw(),
                helper_skill_def.raw(),
                event_cards.len().min(u16::MAX as usize) as u32,
            ];
            payload.extend(
                event_cards
                    .into_iter()
                    .take(u16::MAX as usize)
                    .map(CardId::raw),
            );
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
        10 => {
            let valid = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player
                    && tx.state().skill_effectively_enabled(instance)
                    && instance.lifetime_fire_count == 1
            }) && tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.alive && owner.dying && owner.hp <= 0);
            if !valid {
                return Ok(Vec::new());
            }
            let options = tx.state().players.alive_by_seat();
            let payload = vec![RESOLVE_ZHUPEILAN_CHOICE, 1, player.raw(), skill.raw()];
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        11 => {
            let context_valid = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.owner == player
                    && tx.state().skill_effectively_enabled(instance)
                    && instance.lifetime_fire_count == 1
            }) && tx
                .state()
                .players
                .get(player)
                .is_some_and(|owner| owner.alive && owner.dying && owner.hp <= 0);
            if !context_valid {
                return Err(EffectError::Illegal("Tongdao context changed".into()));
            }
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("Tongdao target is stale".into()));
            }
            let target_hp = tx.state().players.get(target).unwrap().hp;
            let owner_hp = tx
                .state()
                .players
                .get(player)
                .map(|owner| owner.hp)
                .ok_or_else(|| EffectError::Illegal("Tongdao owner missing".into()))?;
            let mut events = reset_player_starting_skills(tx, target)?;
            if target != player && target_hp > owner_hp {
                events.extend(run_effects(
                    tx,
                    vec![EffectRequest::RecoverHp {
                        source: player,
                        player,
                        amount: (target_hp - owner_hp) as u32,
                    }],
                )?);
            }
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("unknown Zhupeilan action".into())),
    }
}

pub(crate) fn dc_sb_zhangxiu_hand_equip_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(player);
    if let Some(zone) = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| tx.state().zones.get(zone))
    {
        cards.extend(zone.cards.iter().copied());
    }
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn dcsbhaoyi_cards(tx: &Transaction) -> Vec<CardId> {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    let mut cards: Vec<_> = tx
        .state()
        .turn
        .entered_discard_cards
        .iter()
        .copied()
        .filter(|card| !tx.state().turn.damage_card_materials.contains(card))
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                Some(card.zone) == discard && (card.kind.is_sha() || card.kind.is_damage_trick())
            })
        })
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards
}
