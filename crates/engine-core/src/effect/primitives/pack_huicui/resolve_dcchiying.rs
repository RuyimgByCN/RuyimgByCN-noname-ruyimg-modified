//! Effect primitives for `resolve_dcchiying` and related helpers.

use super::*;

pub(crate) fn resolve_dcchiying(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    accum: Vec<CardId>,
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
    if stage == 1 {
        let owner_hp = tx.state().players.get(player).map(|p| p.hp).unwrap_or(0);
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| {
                candidate != player
                    && tx
                        .state()
                        .players
                        .get(candidate)
                        .map(|p| p.hp <= owner_hp)
                        .unwrap_or(false)
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
            true,
            options,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                RESOLVE_DCCHIYING_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                target.raw(),
                1,
                0,
            ],
        );
    }
    // Stage 2: the target picks a player in range to discard from.
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let picked: Vec<PlayerId> = accum
        .iter()
        .filter_map(|card| tx.state().cards.get(*card).map(|card| card.owner))
        .collect();
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            candidate != target
                && !picked.contains(&candidate)
                && dcchiying_in_range(tx.state(), target, candidate)
                && !crate::query::discardable_target_cards(
                    tx.state(),
                    target,
                    candidate,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        })
        .map(PlayerId::raw)
        .collect();
    if options.is_empty() {
        // No more players: gain the accumulated basic cards.
        return dcchiying_gain_basics(tx, target, accum);
    }
    let mut payload = vec![
        RESOLVE_DCCHIYING_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        target.raw(),
        2,
        accum.len() as u32,
    ];
    payload.extend(accum.iter().map(|card| card.raw()));
    open_choice(
        tx,
        target,
        1,
        1,
        true,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn dcchiying_gain_basics(
    tx: &mut Transaction,
    target: PlayerId,
    accum: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let basics: Vec<CardId> = accum
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.zone == discard && card.kind.is_basic())
        })
        .collect();
    let mut moves = Vec::new();
    for card in basics {
        let from = tx
            .state()
            .cards
            .get(card)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("Dcchiying card missing".into()))?;
        moves.push(EffectRequest::MoveCards {
            cards: vec![card],
            to: tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("target hand missing".into()))?,
            from: Some(from),
        });
    }
    run_effects(tx, moves)
}

pub(crate) fn adjacent_players(tx: &Transaction, player: PlayerId) -> Vec<PlayerId> {
    use rule_api::QueryRequest;
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            if candidate == player {
                return false;
            }
            let mut ctx = crate::query::QueryContext::new();
            match crate::query::query(
                tx.state(),
                &QueryRequest::Distance {
                    from: player,
                    to: candidate,
                },
                &mut ctx,
            ) {
                rule_api::QueryResponse::Int(distance) => distance <= 1,
                _ => false,
            }
        })
        .collect()
}

pub(crate) fn resolve_xuxie(
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
    run_effects(tx, vec![EffectRequest::ChangeMaxHp { player, delta: -1 }])?;
    let adjacent = adjacent_players(tx, player);
    if adjacent.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        RESOLVE_XUXIE_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        0,
        adjacent.len() as u32,
    ];
    payload.extend(adjacent.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![0, 1],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn resolve_xuxie_add(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let owner_max = tx
        .state()
        .players
        .get(player)
        .map(|p| p.max_hp)
        .unwrap_or(0);
    let someone_bigger = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .any(|candidate| {
            candidate != player
                && tx
                    .state()
                    .players
                    .get(candidate)
                    .map(|p| p.max_hp > owner_max)
                    .unwrap_or(false)
        });
    if someone_bigger {
        run_effects(tx, vec![EffectRequest::ChangeMaxHp { player, delta: 1 }])
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn resolve_xizhen(
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
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        vec![RESOLVE_XIZHEN_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_dcmoyu(
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
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            if candidate == player {
                return false;
            }
            let used = tx
                .state()
                .players
                .get(player)
                .map(|p| {
                    p.marks
                        .iter()
                        .find(|(mark, _)| *mark == candidate.raw())
                        .map(|(_, count)| *count)
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            used == 0
                && !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    candidate,
                    rule_api::TargetCardScope::HandEquipJudge,
                )
                .is_empty()
        })
        .map(PlayerId::raw)
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
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        vec![RESOLVE_DCMOYU_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_rekuanshi(
    tx: &mut Transaction,
    player: PlayerId,
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
    // Reset the once-per-turn mark at each End phase.
    let used = tx
        .state()
        .players
        .get(player)
        .map(|p| {
            p.marks
                .iter()
                .find(|(mark, _)| *mark == REKUANSHI_USED_MARK)
                .map(|(_, count)| *count)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    if used > 0 {
        run_effects(
            tx,
            vec![EffectRequest::RemoveMark {
                player,
                mark: engine_api::MarkDefId::new(REKUANSHI_USED_MARK),
                amount: used.max(0) as u32,
            }],
        )?;
    }
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
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        vec![RESOLVE_REKUANSHI_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_rekuanshi_resolve(
    tx: &mut Transaction,
    player: PlayerId,
    damaged: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(damaged)
            .is_some_and(|participant| participant.alive)
        || damaged == player
    {
        return Ok(Vec::new());
    }
    // The damaged player must carry the watch mark.
    let watched = tx
        .state()
        .players
        .get(damaged)
        .map(|p| {
            p.marks
                .iter()
                .find(|(mark, _)| *mark == REKUANSHI_WATCH_MARK)
                .map(|(_, count)| *count)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    if watched == 0 {
        return Ok(Vec::new());
    }
    let used = tx
        .state()
        .players
        .get(player)
        .map(|p| {
            p.marks
                .iter()
                .find(|(mark, _)| *mark == REKUANSHI_USED_MARK)
                .map(|(_, count)| *count)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    if used > 0 {
        return Ok(Vec::new());
    }
    let count = tx.state().turn.damage_taken_count(damaged);
    let wounded = tx
        .state()
        .players
        .get(damaged)
        .is_some_and(|p| p.hp < p.max_hp as i32);
    if count == 2 && wounded {
        run_effects(
            tx,
            vec![
                EffectRequest::RecoverHp {
                    source: player,
                    player: damaged,
                    amount: 1,
                },
                EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(REKUANSHI_USED_MARK),
                    delta: 1,
                },
            ],
        )
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn resolve_sxrmlanjiao(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    view: Vec<CardId>,
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
    if stage == 0 {
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| {
                if candidate == player || tx.state().hand_count(candidate) < 2 {
                    return false;
                }
                // Once per target per turn: mark keyed by the target id.
                let used = tx
                    .state()
                    .players
                    .get(player)
                    .map(|p| {
                        p.marks
                            .iter()
                            .find(|(mark, _)| *mark == candidate.raw())
                            .map(|(_, count)| *count)
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                used == 0
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
            true,
            options,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                RESOLVE_SXRMLANJIAO_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                0,
                0,
                target.raw(),
                0,
            ],
        );
    }
    // Stage 1: view the first two hand cards and let the target decide.
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_count(target) < 2
    {
        return Ok(Vec::new());
    }
    let hand = tx.state().hand_cards(target);
    let view: Vec<CardId> = hand.into_iter().take(2).collect();
    let mut effects = vec![EffectRequest::RevealCards {
        cards: view.clone(),
        to: Vec::new(),
    }];
    crate::effect::run_effects(tx, effects)?;
    let mut payload = vec![
        RESOLVE_SXRMLANJIAO_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        1,
        0,
        target.raw(),
        view.len() as u32,
    ];
    payload.extend(view.iter().map(|card| card.raw()));
    open_choice(
        tx,
        target,
        1,
        1,
        true,
        vec![0, 1],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn resolve_nsdingzhou(
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
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            candidate != player
                && !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    candidate,
                    rule_api::TargetCardScope::HandEquipJudge,
                )
                .is_empty()
        })
        .map(PlayerId::raw)
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
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        vec![RESOLVE_NSDINGZHOU_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_dckuizhen(
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
    let (owner_hand, owner_hp) = (
        tx.state().hand_count(player),
        tx.state().players.get(player).map(|p| p.hp).unwrap_or(0),
    );
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            candidate != player
                && (tx.state().hand_count(candidate) >= owner_hand
                    || tx
                        .state()
                        .players
                        .get(candidate)
                        .map(|p| p.hp >= owner_hp)
                        .unwrap_or(false))
        })
        .map(PlayerId::raw)
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
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        vec![RESOLVE_DCKUIZHEN_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_dckuizhen_resolve(
    tx: &mut Transaction,
    player: PlayerId,
    winner: PlayerId,
    loser: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    // Only act on the pending duel this skill opened.
    let initiator = tx
        .state()
        .players
        .get(player)
        .map(|p| {
            p.marks
                .iter()
                .find(|(mark, _)| *mark == DCKUIZHEN_PENDING_MARK)
                .map(|(_, count)| *count)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let Ok(initiator) = u32::try_from(initiator) else {
        return Ok(Vec::new());
    };
    if initiator == 0 || PlayerId::new(initiator) == player {
        return Ok(Vec::new());
    }
    let initiator = PlayerId::new(initiator);
    if !tx
        .state()
        .players
        .get(initiator)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut effects = vec![EffectRequest::RemoveMark {
        player,
        mark: engine_api::MarkDefId::new(DCKUIZHEN_PENDING_MARK),
        amount: initiator.raw(),
    }];
    if loser == player {
        // The owner lost the Duel (took damage): gain all Shas from the initiator's hand.
        let shas: Vec<CardId> = tx
            .state()
            .hand_cards(initiator)
            .into_iter()
            .filter(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| card.kind.is_sha())
            })
            .collect();
        for card in shas {
            let from = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.zone)
                .ok_or_else(|| EffectError::Illegal("Dckuizhen card missing".into()))?;
            effects.push(EffectRequest::MoveCards {
                cards: vec![card],
                to: tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
                from: Some(from),
            });
        }
    } else if winner == player {
        // The owner won: the initiator loses 1 HP.
        effects.push(EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: initiator,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    run_effects(tx, effects)
}
