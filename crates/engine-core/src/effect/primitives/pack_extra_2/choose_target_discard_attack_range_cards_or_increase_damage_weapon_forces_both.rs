//! Effect primitives for `reveal_uniform_hand_draw_self_choose_others` and related helpers.

use super::*;

pub(crate) fn choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
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
        || !bound_sha_damage_matches(tx, damage_id, player, target)
    {
        return Err(EffectError::Illegal(
            "attack-range damage choice context changed".into(),
        ));
    }
    let attack_range = current_attack_range(tx, player);
    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
    let weapons = physical_equipped_weapons(tx, player);
    if !weapons.is_empty() {
        if options.is_empty() || attack_range == 0 {
            return super::super::run_effects(
                tx,
                vec![
                    EffectRequest::ContinueIncreaseBoundDamage {
                        player,
                        damage_id,
                        amount: 1,
                        skill,
                    },
                    EffectRequest::DiscardCards {
                        player,
                        cards: weapons,
                    },
                ],
            );
        }
        return open_attack_range_discard_choice(
            tx,
            2,
            player,
            target,
            damage_id,
            skill,
            attack_range,
            options,
        );
    }
    if options.is_empty() || attack_range == 0 {
        increase_bound_damage_by_one(tx, damage_id)?;
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth,
        vec![
            0,
            player.raw(),
            target.raw(),
            damage_id,
            skill.raw(),
            attack_range.min(u32::MAX as usize) as u32,
        ],
    )
}

pub(crate) fn choose_hand_count_different_target_draw_or_random_discard(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "hand-count target choice context changed".into(),
        ));
    }
    let hand_count = tx.state().hand_count(player);
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| tx.state().hand_count(target) != hand_count)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![player.raw(), skill.raw(), options.len() as u32];
    payload.extend(options.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseHandCountDifferentTargetDrawOrRandomDiscard,
        payload,
    )
}

pub(crate) fn rezhenjun_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Prepare
}

pub(crate) fn choose_target_discard_excess_then_self_discard_or_target_draw(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !rezhenjun_context_is_current(tx, player, skill) {
        return Err(EffectError::Illegal(
            "prepare-phase target discard context changed".into(),
        ));
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty()
        })
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![0, player.raw(), skill.raw(), options.len() as u32];
    payload.extend(options.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
        payload,
    )
}

pub(crate) fn choose_self_discard_or_target_draw_after_target_discard(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if !rezhenjun_context_is_current(tx, player, skill)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "prepare-phase self-discard branch context changed".into(),
        ));
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let required = usize::try_from(count).unwrap_or(usize::MAX);
    if options.len() < required || count > u32::from(u8::MAX) {
        return super::super::run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player: target,
                count,
                skill,
            }],
        );
    }
    let mut payload = vec![
        2,
        player.raw(),
        target.raw(),
        skill.raw(),
        count,
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        count as u8,
        count as u8,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
        payload,
    )
}

pub(crate) fn decadezhenjun_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && matches!(tx.state().turn.phase, Phase::Prepare | Phase::End)
}

pub(crate) fn resolve_decadezhenjun(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !decadezhenjun_context_is_current(tx, player, skill) {
                return Err(EffectError::Illegal(
                    "Decade Zhenjun phase context changed".into(),
                ));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    !crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![3, player.raw(), skill.raw(), options.len() as u32];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
                payload,
            )
        }
        1 => {
            if count == 0
                || !decadezhenjun_context_is_current(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return super::super::run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player: target,
                        count,
                        skill,
                    }],
                );
            }
            let mut payload = vec![
                5,
                player.raw(),
                target.raw(),
                skill.raw(),
                count,
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
                payload,
            )
        }
        _ => Err(EffectError::Illegal(
            "invalid Decade Zhenjun internal stage".into(),
        )),
    }
}

pub(crate) fn sp_xiaoguo_initial_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && player != target
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == target
        && tx.state().turn.phase == Phase::End
}

pub(crate) fn sp_xiaoguo_participants_alive(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> bool {
    tx.state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
}

pub(crate) fn resolve_sp_xiaoguo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !sp_xiaoguo_initial_context_is_current(tx, player, target, skill) {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo phase context changed".into(),
                ));
            }
            let options = crate::query::sp_xiaoguo_hand_cards(tx.state(), player);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveSpXiaoguo,
                vec![0, player.raw(), target.raw(), skill.raw()],
            )
        }
        1 => {
            if !sp_xiaoguo_participants_alive(tx, player, target) {
                return Ok(Vec::new());
            }
            let options = crate::query::sp_xiaoguo_equipment_cards(tx.state(), target);
            if options.is_empty() {
                return super::super::run_effects(
                    tx,
                    vec![EffectRequest::ResolveSpXiaoguo {
                        player,
                        target,
                        stage: 2,
                        skill,
                    }],
                );
            }
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveSpXiaoguo,
                vec![2, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            if !sp_xiaoguo_participants_alive(tx, player, target) {
                return Ok(Vec::new());
            }
            super::super::run_effects(
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
        3 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            super::super::run_effects(
                tx,
                vec![EffectRequest::DrawCardsFromSkill {
                    player,
                    count: 1,
                    skill,
                }],
            )
        }
        _ => Err(EffectError::Illegal(
            "invalid SP Xiaoguo internal stage".into(),
        )),
    }
}

pub(crate) fn eu_tuxi_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
}

/// Reproduce legacy `trigger.player`, `getPrevious()`, `getNext()` after the draw barrier.
/// Previous and next intentionally remain duplicated in a two-player game.
pub(crate) fn damage_target_and_legacy_neighbors(
    tx: &Transaction,
    target: PlayerId,
) -> Option<Vec<PlayerId>> {
    let target_seat = tx.state().players.get(target)?.seat.raw();
    let alive = tx.state().players.alive_by_seat();
    let mut targets = vec![target];
    if alive.len() <= 1 {
        return Some(targets);
    }

    if let Some(position) = alive.iter().position(|&player| player == target) {
        targets.push(alive[(position + alive.len() - 1) % alive.len()]);
        targets.push(alive[(position + 1) % alive.len()]);
        return Some(targets);
    }

    let previous = alive
        .iter()
        .rev()
        .copied()
        .find(|player| {
            tx.state()
                .players
                .get(*player)
                .is_some_and(|participant| participant.seat.raw() < target_seat)
        })
        .or_else(|| alive.last().copied());
    let next = alive
        .iter()
        .copied()
        .find(|player| {
            tx.state()
                .players
                .get(*player)
                .is_some_and(|participant| participant.seat.raw() > target_seat)
        })
        .or_else(|| alive.first().copied());
    targets.extend(previous);
    targets.extend(next);
    Some(targets)
}

pub(crate) fn eu_tuxi_gain_options(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Vec::new();
    }
    let hand = (target != player)
        .then(|| tx.state().zones.find(ZoneKind::Hand, target))
        .flatten();
    let equip = tx.state().zones.find(ZoneKind::Equip, target);
    let mut options: Vec<_> = [hand, equip]
        .into_iter()
        .flatten()
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect();
    options.sort_by_key(|card| card.raw());
    options.dedup();
    options
}

pub(crate) fn draw_then_choose_gain_one_from_damage_target_and_neighbors(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !eu_tuxi_context_is_current(tx, player, skill) {
        return if ready_to_choose {
            Ok(Vec::new())
        } else {
            Err(EffectError::Illegal(
                "damage-source sequential gain context changed".into(),
            ))
        };
    }
    if tx.state().players.get(target).is_none() {
        return if ready_to_choose {
            Ok(Vec::new())
        } else {
            Err(EffectError::Illegal("missing damaged player".into()))
        };
    }
    if !ready_to_choose {
        let events = draw_cards(tx, player, 1)?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
                player,
                target,
                skill,
                ready_to_choose: true,
            },
        );
        return Ok(events);
    }

    let Some(targets) = damage_target_and_legacy_neighbors(tx, target) else {
        return Ok(Vec::new());
    };
    continue_choose_gain_one_from_damage_target_and_neighbors(
        tx,
        player,
        targets,
        0,
        CardId::NONE,
        skill,
    )
}

pub(crate) fn continue_choose_gain_one_from_damage_target_and_neighbors(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    mut index: u32,
    selected: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !eu_tuxi_context_is_current(tx, player, skill) {
        return Ok(Vec::new());
    }

    if selected.is_some() {
        let current = targets
            .get(index as usize)
            .copied()
            .ok_or_else(|| EffectError::Illegal("sequential gain index changed".into()))?;
        if !eu_tuxi_gain_options(tx, player, current).contains(&selected) {
            return Err(EffectError::Illegal(
                "sequential gain selection is invalid or stale".into(),
            ));
        }
        let from = tx
            .state()
            .cards
            .get(selected)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("missing selected gain card".into()))?;
        let to = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("missing skill owner hand".into()))?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors {
                player,
                targets,
                index: index.saturating_add(1),
                selected: CardId::NONE,
                skill,
            },
        );
        return move_cards(tx, vec![selected], to, Some(from));
    }

    while let Some(current) = targets.get(index as usize).copied() {
        let options = eu_tuxi_gain_options(tx, player, current);
        if options.is_empty() {
            index = index.saturating_add(1);
            continue;
        }
        let mut payload = vec![player.raw(), skill.raw(), index, targets.len() as u32];
        payload.extend(targets.iter().map(|target| target.raw()));
        payload.push(options.len() as u32);
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseGainOneFromDamageTargetAndNeighbors,
            payload,
        );
    }
    Ok(Vec::new())
}

pub(crate) fn modify_current_damage(
    tx: &mut Transaction,
    player: PlayerId,
    delta: i32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if delta == 0 {
        return Ok(Vec::new());
    }
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id == 0 {
        return Err(EffectError::ContentFault(
            "current damage modification used outside its damage_before event".into(),
        ));
    }
    if delta > 0 {
        increase_bound_damage(tx, damage_id, delta as u32)?;
    } else {
        for _ in 0..(-delta) {
            reduce_bound_damage_by_one(tx, damage_id)?;
        }
    }
    let _ = player;
    Ok(Vec::new())
}
