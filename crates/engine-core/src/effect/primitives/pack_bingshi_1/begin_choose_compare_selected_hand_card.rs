//! Effect primitives for `begin_choose_compare_selected_hand_card` and related helpers.

use super::*;

pub(crate) fn begin_choose_compare_selected_hand_card(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    win_skill_def: engine_api::SkillDefId,
    lose_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<(Vec<DomainEvent>, Vec<EffectRequest>), EffectError> {
    validate_hand_compare_start(tx.state(), player, target, card, skill)?;
    let resolution = &mut tx.state_mut().resolution;
    resolution.current_hand_compare_player = player.raw();
    resolution.current_hand_compare_target = target.raw();
    resolution.current_hand_compare_random_mask = 0;
    Ok((
        vec![DomainEvent::HandCompareStarted {
            player,
            target,
            player_card: card,
        }],
        vec![EffectRequest::ContinueChooseCompareTargetHandCard {
            player,
            target,
            card,
            win_skill_def,
            lose_skill_def,
            skill,
        }],
    ))
}

pub(crate) fn validate_second_failure_compare_contract(
    state: &crate::state::GameState,
    player: PlayerId,
    skill: engine_api::SkillId,
    draw_count: u32,
    failure_count: u32,
    lose_hp: u32,
    damage: u32,
) -> Result<(), EffectError> {
    let instance = state
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("comparison active skill missing".into()))?;
    let contract = state
        .content
        .first_active_for_skill(instance.def)
        .ok_or_else(|| EffectError::ContentFault("comparison active contract missing".into()))?;
    if instance.owner != player
        || !state.skill_effectively_enabled(instance)
        || state.turn.current != player
        || state.turn.phase != Phase::Play
        || draw_count == 0
        || failure_count != 2
        || lose_hp == 0
        || damage == 0
        || !contract.targets.as_ref().is_some_and(|targets| {
            targets.min == 1
                && targets.max == 1
                && !targets.allow_self
                && targets.can_compare
                && targets.unused_by_skill_this_turn
        })
        || !contract.selection.as_ref().is_some_and(|selection| {
            selection.zone == "hand" && selection.min == 1 && selection.max == 1
        })
    {
        return Err(EffectError::ContentFault(
            "comparison failure path requires its declared active contract".into(),
        ));
    }
    Ok(())
}

pub(crate) fn record_second_failure_compare_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    draw_count: u32,
    failure_count: u32,
    lose_hp: u32,
    damage: u32,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    validate_second_failure_compare_contract(
        tx.state(),
        player,
        skill,
        draw_count,
        failure_count,
        lose_hp,
        damage,
    )?;
    validate_hand_compare_start(tx.state(), player, target, card, skill)?;
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| instance.record_active_compare_target(target))
    {
        return Err(EffectError::Illegal(
            "comparison target was already selected this turn".into(),
        ));
    }
    Ok(())
}

pub(crate) fn choose_compare_selected_hand_card_second_failure_seat_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    draw_count: u32,
    failure_count: u32,
    lose_hp: u32,
    damage: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    record_second_failure_compare_target(
        tx,
        player,
        target,
        card,
        draw_count,
        failure_count,
        lose_hp,
        damage,
        skill,
    )?;
    let options: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .into_iter()
        .map(CardId::raw)
        .collect();
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseCompareTargetHandCardSecondFailureSeatDamage,
        vec![
            player.raw(),
            target.raw(),
            card.raw(),
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill.raw(),
        ],
    )
}

pub(crate) fn begin_choose_compare_selected_hand_card_second_failure_seat_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    draw_count: u32,
    failure_count: u32,
    lose_hp: u32,
    damage: u32,
    skill: engine_api::SkillId,
) -> Result<(Vec<DomainEvent>, Vec<EffectRequest>), EffectError> {
    record_second_failure_compare_target(
        tx,
        player,
        target,
        card,
        draw_count,
        failure_count,
        lose_hp,
        damage,
        skill,
    )?;
    let resolution = &mut tx.state_mut().resolution;
    resolution.current_hand_compare_player = player.raw();
    resolution.current_hand_compare_target = target.raw();
    resolution.current_hand_compare_random_mask = 0;
    Ok((
        vec![DomainEvent::HandCompareStarted {
            player,
            target,
            player_card: card,
        }],
        vec![
            EffectRequest::ContinueChooseCompareTargetHandCardSecondFailureSeatDamage {
                player,
                target,
                card,
                draw_count,
                failure_count,
                lose_hp,
                damage,
                skill,
            },
        ],
    ))
}

pub(crate) fn validate_hand_compare_start(
    state: &crate::state::GameState,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    let valid_skill = state.skills.get(skill).is_some_and(|instance| {
        state.skill_effectively_enabled(instance)
            && (instance.owner == player
                || instance.twjianwei_compare() == Some((player, target))
                || (instance.owner == target && instance.zhiba_compare() == Some(player)))
    });
    if !valid_skill {
        return Err(EffectError::ContentFault(
            "hand comparison requires its active skill instance".into(),
        ));
    }
    if !state
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || player == target
    {
        return Err(EffectError::Illegal(
            "hand comparison requires two different living players".into(),
        ));
    }
    let hand = state
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("comparison owner has no hand zone".into()))?;
    if !state
        .cards
        .get(card)
        .is_some_and(|value| value.owner == player && value.zone == hand)
    {
        return Err(EffectError::Illegal(
            "selected comparison card is no longer in hand".into(),
        ));
    }
    if state.hand_cards(target).is_empty() {
        return Err(EffectError::Illegal(
            "comparison target has no hand card".into(),
        ));
    }
    Ok(())
}

pub(crate) fn randomize_current_hand_compare_opponent(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "hand comparison randomizer requires its skill instance".into(),
        ));
    }
    let resolution = &mut tx.state_mut().resolution;
    if player.raw() == resolution.current_hand_compare_player {
        resolution.current_hand_compare_random_mask |= 0b10;
    } else if player.raw() == resolution.current_hand_compare_target {
        resolution.current_hand_compare_random_mask |= 0b01;
    } else {
        return Err(EffectError::Illegal(
            "randomizer owner is not a current comparison participant".into(),
        ));
    }
    Ok(Vec::new())
}

pub(crate) fn random_hand_card(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<CardId, EffectError> {
    let cards = tx.state().hand_cards(player);
    let max = u32::try_from(cards.len())
        .map_err(|_| EffectError::EngineFault("hand exceeds deterministic RNG range".into()))?;
    if max == 0 {
        return Err(EffectError::Illegal(
            "random comparison participant has no hand card".into(),
        ));
    }
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let card = cards[rng.gen_range_u32(max) as usize];
    tx.state_mut().rng = rng.into_state();
    Ok(card)
}

pub(crate) fn continue_choose_compare_target_hand_card(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    mut player_card: CardId,
    win_skill_def: engine_api::SkillDefId,
    lose_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    validate_hand_compare_start(tx.state(), player, target, player_card, skill)?;
    let resolution = &mut tx.state_mut().resolution;
    if resolution.current_hand_compare_player != player.raw()
        || resolution.current_hand_compare_target != target.raw()
    {
        return Err(EffectError::Illegal(
            "hand comparison start context changed".into(),
        ));
    }
    let random_mask = resolution.current_hand_compare_random_mask;
    resolution.current_hand_compare_player = 0;
    resolution.current_hand_compare_target = 0;
    resolution.current_hand_compare_random_mask = 0;

    if random_mask & 0b01 != 0 {
        player_card = random_hand_card(tx, player)?;
    }
    if random_mask & 0b10 != 0 {
        let target_card = random_hand_card(tx, target)?;
        return commit_hand_compare(
            tx,
            player,
            target,
            player_card,
            target_card,
            win_skill_def,
            lose_skill_def,
            skill,
        );
    }
    let options = tx.state().hand_cards(target);
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseCompareTargetHandCard,
        vec![
            player.raw(),
            target.raw(),
            player_card.raw(),
            win_skill_def.raw(),
            lose_skill_def.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn continue_choose_compare_target_hand_card_second_failure_seat_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    mut player_card: CardId,
    draw_count: u32,
    failure_count: u32,
    lose_hp: u32,
    damage: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    validate_hand_compare_start(tx.state(), player, target, player_card, skill)?;
    let resolution = &mut tx.state_mut().resolution;
    if resolution.current_hand_compare_player != player.raw()
        || resolution.current_hand_compare_target != target.raw()
    {
        return Err(EffectError::Illegal(
            "hand comparison start context changed".into(),
        ));
    }
    let random_mask = resolution.current_hand_compare_random_mask;
    resolution.current_hand_compare_player = 0;
    resolution.current_hand_compare_target = 0;
    resolution.current_hand_compare_random_mask = 0;
    if random_mask & 0b01 != 0 {
        player_card = random_hand_card(tx, player)?;
    }
    if random_mask & 0b10 != 0 {
        let target_card = random_hand_card(tx, target)?;
        return commit_hand_compare_second_failure_seat_damage(
            tx,
            player,
            target,
            player_card,
            target_card,
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill,
        );
    }
    let options = tx.state().hand_cards(target);
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseCompareTargetHandCardSecondFailureSeatDamage,
        vec![
            player.raw(),
            target.raw(),
            player_card.raw(),
            draw_count,
            failure_count,
            lose_hp,
            damage,
            skill.raw(),
        ],
    )
}

/// ABI 678 (batch 166): grant-flow commit — on the chooser's strict win the target is
/// granted `win_skill_def`; tie/loss grants nothing. `grant_to_target=false` delegates
/// to the classic [`commit_hand_compare`].
pub(crate) fn commit_hand_compare_ext(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    player_card: CardId,
    target_card: CardId,
    win_skill_def: engine_api::SkillDefId,
    lose_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
    grant_to_target: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !grant_to_target {
        return commit_hand_compare(
            tx,
            player,
            target,
            player_card,
            target_card,
            win_skill_def,
            lose_skill_def,
            skill,
        );
    }
    let (player_hand, target_hand, discard, winner) =
        hand_compare_commit_context(tx.state(), player, target, player_card, target_card, skill)?;
    let mut effects = vec![
        EffectRequest::RevealCards {
            cards: vec![player_card, target_card],
            to: Vec::new(),
        },
        EffectRequest::MoveCards {
            cards: vec![player_card],
            to: discard,
            from: Some(player_hand),
        },
        EffectRequest::MoveCards {
            cards: vec![target_card],
            to: discard,
            from: Some(target_hand),
        },
    ];
    if winner == player {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: win_skill_def,
        });
    }
    effects.push(EffectRequest::EmitHandCompareFinished {
        player,
        target,
        player_card,
        target_card,
        winner,
    });
    super::super::run_effects(tx, effects)
}

/// ABI 680 (batch 174): third-party comparison commit — reveal, both cards to discard,
/// then on non-tie the **winner** uses a distance-free virtual Sha against the loser
/// (jun_zhaoshou; target fixed to the loser — upstream lets the winner aim freely).
pub(crate) fn commit_hand_compare_between(
    tx: &mut Transaction,
    player: PlayerId,
    first: PlayerId,
    second: PlayerId,
    first_card: CardId,
    second_card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 自包含校验（不复用 hand_compare_commit_context——其 skill-instance 校验要求
    // 参与者持有技能实例，而 between 的技能在旁观者 owner 身上）。
    let first_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, first)
        .ok_or_else(|| EffectError::Illegal("between first hand missing".into()))?;
    let second_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, second)
        .ok_or_else(|| EffectError::Illegal("between second hand missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
    let first_rank = tx
        .state()
        .cards
        .get(first_card)
        .filter(|card| card.owner == first && card.zone == first_hand)
        .map(|card| crate::query::effective_card_number(tx.state(), first, card))
        .ok_or_else(|| EffectError::Illegal("first comparison card left hand".into()))?;
    let second_rank = tx
        .state()
        .cards
        .get(second_card)
        .filter(|card| card.owner == second && card.zone == second_hand)
        .map(|card| crate::query::effective_card_number(tx.state(), second, card))
        .ok_or_else(|| EffectError::Illegal("second comparison card left hand".into()))?;
    let winner = if first_rank > second_rank {
        first
    } else if second_rank > first_rank {
        second
    } else {
        // 平局：无虚拟杀，仅弃牌（Emit 以下方统一路径发出）。
        PlayerId::NONE
    };
    let loser = if winner == first { second } else { first };
    super::super::run_effects(
        tx,
        vec![
            EffectRequest::RevealCards {
                cards: vec![first_card, second_card],
                to: Vec::new(),
            },
            EffectRequest::MoveCards {
                cards: vec![first_card],
                to: discard,
                from: Some(first_hand),
            },
            EffectRequest::MoveCards {
                cards: vec![second_card],
                to: discard,
                from: Some(second_hand),
            },
            EffectRequest::EmitHandCompareFinished {
                player: first,
                target: second,
                player_card: first_card,
                target_card: second_card,
                winner,
            },
        ],
    )?;
    if winner == PlayerId::NONE || winner == loser {
        return Ok(Vec::new());
    }
    // 胜方对败方方向使用无距离虚拟【杀】（批 178 尾债消除：目标由胜方经
    // VirtualShaTargets 选择链自选，与上游 chooseUseTarget(nodistance) 对齐；
    // 若无可选目标，choose_use_virtual_sha 自行回收虚拟牌）。
    choose_use_virtual_sha(tx, winner, true, false, false, skill, 0, 0)
}

pub(crate) fn commit_hand_compare(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    player_card: CardId,
    target_card: CardId,
    win_skill_def: engine_api::SkillDefId,
    lose_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let (player_hand, target_hand, discard, winner) =
        hand_compare_commit_context(tx.state(), player, target, player_card, target_card, skill)?;
    let twjianwei_owner = tx.state().skills.get(skill).and_then(|instance| {
        instance
            .twjianwei_compare()
            .is_some_and(|pair| pair == (player, target))
            .then_some(instance.owner)
    });
    let twzhibian_owner = tx.state().skills.get(skill).and_then(|instance| {
        (instance.owner == player && instance.twzhibian_compare() == Some(target))
            .then_some(instance.owner)
    });
    let zhiba_owner = tx.state().skills.get(skill).and_then(|instance| {
        (instance.owner == target && instance.zhiba_compare() == Some(player))
            .then_some(instance.owner)
    });
    let result = if let Some(owner) = twjianwei_owner {
        EffectRequest::ResolveTwjianwei {
            player: owner,
            actor: player,
            target,
            card: player_card,
            winner,
            stage: 3,
            skill,
        }
    } else if let Some(owner) = zhiba_owner {
        EffectRequest::ResolveZhibaSettle {
            player: owner,
            actor: player,
            winner,
            player_card,
            target_card,
            stage: 0,
            skill,
        }
    } else if let Some(owner) = twzhibian_owner {
        EffectRequest::ResolveTwzhibian {
            player: owner,
            target,
            winner,
            stage: 1,
            skill,
        }
    } else if winner == player {
        EffectRequest::AddBoundSkill {
            player,
            skill_def: win_skill_def,
            bound_target: target,
        }
    } else {
        EffectRequest::AddSkill {
            player,
            skill_def: lose_skill_def,
        }
    };
    super::super::run_effects(
        tx,
        vec![
            EffectRequest::RevealCards {
                cards: vec![player_card, target_card],
                to: Vec::new(),
            },
            EffectRequest::MoveCards {
                cards: vec![player_card],
                to: discard,
                from: Some(player_hand),
            },
            EffectRequest::MoveCards {
                cards: vec![target_card],
                to: discard,
                from: Some(target_hand),
            },
            result,
            EffectRequest::EmitHandCompareFinished {
                player,
                target,
                player_card,
                target_card,
                winner,
            },
        ],
    )
}

pub(crate) fn hand_compare_commit_context(
    state: &crate::state::GameState,
    player: PlayerId,
    target: PlayerId,
    player_card: CardId,
    target_card: CardId,
    skill: engine_api::SkillId,
) -> Result<(ZoneId, ZoneId, ZoneId, PlayerId), EffectError> {
    let instance = state.skills.get(skill);
    let twjianwei = instance.and_then(|instance| instance.twjianwei_compare());
    let zhiba = instance.and_then(|instance| instance.zhiba_compare());
    if !instance.is_some_and(|instance| {
        instance.owner == player
            || twjianwei == Some((player, target))
            || (instance.owner == target && zhiba == Some(player))
    }) {
        return Err(EffectError::ContentFault(
            "comparison active skill no longer belongs to its player".into(),
        ));
    }
    let player_hand = state
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("comparison owner hand missing".into()))?;
    let target_hand = state
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("comparison target hand missing".into()))?;
    let mut player_rank = state
        .cards
        .get(player_card)
        .filter(|card| card.owner == player && card.zone == player_hand)
        .map(|card| crate::query::effective_card_number(state, player, card))
        .ok_or_else(|| EffectError::Illegal("owner comparison card left hand".into()))?;
    let mut target_rank = state
        .cards
        .get(target_card)
        .filter(|card| card.owner == target && card.zone == target_hand)
        .map(|card| crate::query::effective_card_number(state, target, card))
        .ok_or_else(|| EffectError::Illegal("target comparison card left hand".into()))?;
    if twjianwei == Some((player, target)) {
        let owner = instance.unwrap().owner;
        let has_weapon = state
            .zones
            .find(ZoneKind::Equip, owner)
            .and_then(|zone| state.zones.get(zone))
            .is_some_and(|zone| {
                zone.cards.iter().any(|card| {
                    state
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind.equip_slot() == Some(EquipSlot::Weapon))
                })
            });
        if has_weapon {
            let bonus = match crate::query::query(
                state,
                &QueryRequest::AttackRange { player: owner },
                &mut crate::query::QueryContext::new(),
            ) {
                QueryResponse::Int(value) => value.clamp(0, 13) as u8,
                _ => 0,
            };
            if owner == player {
                player_rank = player_rank.saturating_add(bonus).min(13);
            } else if owner == target {
                target_rank = target_rank.saturating_add(bonus).min(13);
            }
        }
    }
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let winner = if player_rank > target_rank {
        player
    } else if target_rank > player_rank {
        target
    } else {
        PlayerId::NONE
    };
    Ok((player_hand, target_hand, discard, winner))
}

pub(crate) fn commit_hand_compare_second_failure_seat_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    player_card: CardId,
    target_card: CardId,
    draw_count: u32,
    failure_count: u32,
    lose_hp: u32,
    damage: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let (player_hand, target_hand, discard, winner) =
        hand_compare_commit_context(tx.state(), player, target, player_card, target_card, skill)?;
    let result = if winner == player {
        EffectRequest::ChooseDrawOrDiscardTargetCard {
            player,
            target,
            draw_count,
            skill,
        }
    } else {
        EffectRequest::RecordCompareFailureSeatDamage {
            player,
            target,
            failure_count,
            lose_hp,
            damage,
            skill,
        }
    };
    super::super::run_effects(
        tx,
        vec![
            EffectRequest::RevealCards {
                cards: vec![player_card, target_card],
                to: Vec::new(),
            },
            EffectRequest::MoveCards {
                cards: vec![player_card],
                to: discard,
                from: Some(player_hand),
            },
            EffectRequest::MoveCards {
                cards: vec![target_card],
                to: discard,
                from: Some(target_hand),
            },
            EffectRequest::EmitHandCompareFinished {
                player,
                target,
                player_card,
                target_card,
                winner,
            },
            result,
        ],
    )
}

pub(crate) fn choose_draw_or_discard_target_card(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    draw_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if draw_count == 0
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == player)
    {
        return Err(EffectError::ContentFault(
            "comparison win choice requires its originating skill".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let can_discard = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        && !crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .is_empty();
    if !can_discard {
        return draw_cards(tx, player, draw_count);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseDrawOrDiscardTargetCard,
        vec![player.raw(), target.raw(), draw_count, skill.raw()],
    )
}

pub(crate) fn record_compare_failure_seat_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    failure_count: u32,
    lose_hp: u32,
    damage: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<EffectRequest>, EffectError> {
    if failure_count != 2 || lose_hp == 0 || damage == 0 {
        return Err(EffectError::ContentFault(
            "comparison failure path requires positive second-failure values".into(),
        ));
    }
    let instance = tx
        .state()
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("comparison active skill missing".into()))?;
    if instance.owner != player {
        return Err(EffectError::ContentFault(
            "comparison active skill owner mismatch".into(),
        ));
    }
    let recorded = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .and_then(|instance| instance.record_active_compare_failure(target))
        .ok_or_else(|| EffectError::Illegal("comparison failure history changed".into()))?;
    if recorded < failure_count as usize {
        return Ok(Vec::new());
    }
    if recorded != failure_count as usize {
        return Err(EffectError::Illegal(
            "comparison failure threshold was already reached".into(),
        ));
    }
    let failures = tx
        .state()
        .skills
        .get(skill)
        .and_then(|instance| instance.active_compare_failures())
        .ok_or_else(|| EffectError::EngineFault("comparison failure history missing".into()))?;
    let [first, second] = failures.as_slice() else {
        return Err(EffectError::EngineFault(
            "comparison failure path requires exactly two targets".into(),
        ));
    };
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated comparison skill")
        .active_turn_banned = true;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ChooseSeatPathDamage {
            player,
            first: *first,
            second: *second,
            amount: damage,
            skill,
        });
    Ok(vec![EffectRequest::DealDamage {
        source: PlayerId::NONE,
        target: player,
        amount: lose_hp,
        is_lose_hp: true,
        nature: rule_api::damage_nature::NORMAL,
    }])
}

pub(crate) fn comparison_seat_paths(
    state: &crate::state::GameState,
    first: PlayerId,
    second: PlayerId,
) -> Option<(Vec<PlayerId>, Vec<PlayerId>)> {
    let mut order: Vec<_> = state.players.iter_live().collect();
    order.sort_by_key(|player| player.seat.raw());
    let first_index = order.iter().position(|player| player.id == first)?;
    let second_index = order.iter().position(|player| player.id == second)?;
    if first_index == second_index || order.len() < 2 {
        return None;
    }
    let (start, end) = if first_index < second_index {
        (first_index, second_index)
    } else {
        (second_index, first_index)
    };
    let build = |step: isize| {
        let mut index = start;
        let mut path = vec![order[start].id];
        while index != end {
            index = (index as isize + step).rem_euclid(order.len() as isize) as usize;
            path.insert(0, order[index].id);
        }
        path.into_iter()
            .filter(|&player| {
                player == first
                    || player == second
                    || state
                        .players
                        .get(player)
                        .is_some_and(|participant| participant.alive)
            })
            .collect::<Vec<_>>()
    };
    Some((build(-1), build(1)))
}

pub(crate) fn ordered_path_damage(
    tx: &mut Transaction,
    player: PlayerId,
    path: Vec<PlayerId>,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets = path
        .into_iter()
        .filter(|&target| {
            target != player
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
        })
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    super::super::run_effects(
        tx,
        vec![EffectRequest::DealDamageEachInOrder {
            source: player,
            targets,
            amount,
            skill,
        }],
    )
    .map(|_| Vec::new())
}

pub(crate) fn choose_seat_path_damage(
    tx: &mut Transaction,
    player: PlayerId,
    first: PlayerId,
    second: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == player)
    {
        return Err(EffectError::ContentFault(
            "seat-path damage requires its originating skill".into(),
        ));
    }
    let Some((first_path, second_path)) = comparison_seat_paths(tx.state(), first, second) else {
        return Ok(Vec::new());
    };
    if first_path.is_empty() || second_path.is_empty() {
        return Ok(Vec::new());
    }
    if first_path.len() < second_path.len() {
        return ordered_path_damage(tx, player, first_path, amount, skill);
    }
    if second_path.len() < first_path.len() {
        return ordered_path_damage(tx, player, second_path, amount, skill);
    }
    let mut payload = vec![
        player.raw(),
        amount,
        skill.raw(),
        first_path.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(first_path.into_iter().map(PlayerId::raw));
    payload.extend(second_path.into_iter().map(PlayerId::raw));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseSeatPathDamage,
        payload,
    )
}

pub(crate) fn continue_damage_each_in_order(
    tx: &mut Transaction,
    source: PlayerId,
    targets: Vec<PlayerId>,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<EffectRequest>, EffectError> {
    if amount == 0
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == source)
    {
        return Err(EffectError::ContentFault(
            "ordered damage requires its originating skill".into(),
        ));
    }
    let Some((index, target)) = targets.iter().copied().enumerate().find(|&(_, target)| {
        target != source
            && tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
    }) else {
        return Ok(Vec::new());
    };
    let remaining = targets[index + 1..].to_vec();
    if !remaining.is_empty() {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DealDamageEachInOrder {
                source,
                targets: remaining,
                amount,
                skill,
            },
        );
    }
    Ok(vec![EffectRequest::DealDamage {
        source,
        target,
        amount,
        is_lose_hp: false,
        nature: rule_api::damage_nature::NORMAL,
    }])
}

pub(crate) fn gain_highest_rank_sha_from_event_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: &[CardId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "comparison gain requires its skill instance".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let max_rank = cards
        .iter()
        .filter_map(|card| tx.state().cards.get(*card))
        .filter(|card| card.zone == discard && card.kind.is_sha())
        .map(|card| card.rank)
        .max();
    let Some(max_rank) = max_rank else {
        return Ok(Vec::new());
    };
    let gain: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                card.zone == discard && card.kind.is_sha() && card.rank == max_rank
            })
        })
        .collect();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("comparison gain hand missing".into()))?;
    move_cards(tx, gain, hand, Some(discard))
}

pub(crate) fn gain_event_cards(
    tx: &mut Transaction,
    player: PlayerId,
    cards: &[CardId],
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|state| state.alive)
    {
        return Ok(Vec::new());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    let gain: Vec<_> = cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| state.zone == discard)
        })
        .collect();
    if gain.is_empty() {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("event-card recipient hand missing".into()))?;
    move_cards(tx, gain, hand, Some(discard))
}

/// Select the one physical result card awarded by legacy two-player `zongshi` semantics.
pub(crate) fn hand_compare_result_card(
    player: PlayerId,
    initiator: PlayerId,
    target: PlayerId,
    winner: PlayerId,
    cards: &[CardId],
) -> Option<CardId> {
    let [initiator_card, target_card] = cards else {
        return None;
    };
    if winner != PlayerId::NONE && winner != initiator && winner != target {
        return None;
    }
    if player == initiator {
        Some(if winner == player {
            *target_card
        } else {
            *initiator_card
        })
    } else if player == target {
        Some(if winner == player {
            *initiator_card
        } else {
            *target_card
        })
    } else {
        None
    }
}

pub(crate) fn gain_hand_compare_result_card(
    tx: &mut Transaction,
    player: PlayerId,
    initiator: PlayerId,
    target: PlayerId,
    winner: PlayerId,
    cards: &[CardId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "comparison result gain requires its skill instance".into(),
        ));
    }
    let Some(card) = hand_compare_result_card(player, initiator, target, winner, cards) else {
        return Ok(Vec::new());
    };
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state_card| state_card.zone == discard)
    {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("comparison result gain hand missing".into()))?;
    move_cards(tx, vec![card], hand, Some(discard))
}

pub(crate) fn choose_add_skill_bound_card_target(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(bound_target) = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .and_then(|instance| instance.bound_target())
    else {
        return Ok(Vec::new());
    };
    if !tx
        .state()
        .players
        .get(bound_target)
        .is_some_and(|participant| participant.alive)
        || !crate::command::play::saved_card_extra_target_is_legal(tx, player, card, bound_target)
    {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        vec![bound_target.raw()],
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseAddSkillBoundCardTarget,
        vec![player.raw(), card.raw(), bound_target.raw(), skill.raw()],
    )
}

pub(crate) fn choose_add_or_remove_current_card_target(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let (add_targets, remove_targets) =
        crate::command::play::current_card_target_adjustments(tx, player, card)?;
    let add = !add_targets.is_empty();
    let remove = !remove_targets.is_empty();
    if !add && !remove {
        return remove_skill(tx, skill);
    }
    if add && remove {
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::ChooseAddOrRemoveCurrentCardTarget,
            vec![0, player.raw(), card.raw(), skill.raw()],
        );
    }
    let (mode, targets) = if add {
        (0, add_targets)
    } else {
        (1, remove_targets)
    };
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget,
        vec![1, player.raw(), card.raw(), skill.raw(), mode],
    )
}

pub(crate) fn choose_current_trick_targets_to_nullify(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), source, card));
    if !kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick()) {
        return Ok(Vec::new());
    }
    let (_, targets) = crate::command::play::current_card_target_adjustments(tx, source, card)?;
    if targets.len() < 2 {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        targets.len().min(u8::MAX as usize) as u8,
        true,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseAddOrRemoveCurrentCardTarget,
        vec![2, player.raw(), source.raw(), card.raw(), skill.raw()],
    )
}

pub(crate) fn choose_discard_matching_equip_color_add_skill(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    if !state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "matching-color discard requires its originating skill instance".into(),
        ));
    }
    if player == target
        || !state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || state.turn.current != target
        || state.turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let options = crate::query::discardable_non_basic_matching_equip_color_cards(state, player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardMatchingEquipColorAddSkill,
        vec![player.raw(), target.raw(), skill_def.raw(), skill.raw()],
    )
}

pub(crate) fn add_card_color_lock_skill(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    color_mask: u8,
    skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let color_mask = color_mask & 0b11;
    if color_mask == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || tx.state().players.get(source).is_none()
    {
        return Ok(Vec::new());
    }
    let existing_mask = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|participant| participant.skills.iter())
        .find_map(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .filter(|instance| instance.def == skill_def)
                .and_then(|instance| instance.card_color_lock())
                .map(|(_, mask)| mask)
        })
        .unwrap_or(0);
    add_skill_with_state(
        tx,
        player,
        skill_def,
        Some(crate::state::SkillInstance::card_color_lock_state(
            source,
            existing_mask | color_mask,
        )),
    )
}

pub(crate) fn use_virtual_sha_from_card_color_lock_source_if_no_sha(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(source) = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .and_then(|instance| instance.card_color_lock())
        .map(|(source, _)| source)
    else {
        return Ok(Vec::new());
    };
    if tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || crate::query::sha_play_phase_use_count(tx.state()) != 0
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive && !participant.dying)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    try_use_virtual_sha_on_without_distance(tx, source, player, true, 0)
}
