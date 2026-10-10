//! Effect primitive implementations — bingshi 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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

pub(crate) fn random_hand_card(tx: &mut Transaction, player: PlayerId) -> Result<CardId, EffectError> {
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
    super::run_effects(tx, effects)
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
    super::run_effects(
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
    super::run_effects(
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
    super::run_effects(
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
    super::run_effects(
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

pub(crate) fn choose_current_card_use_enhancements(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if helper_skill_def.is_none()
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::ContentFault(
            "card-use enhancements require their originating skill and helper definition".into(),
        ));
    }
    let Some(extra_targets) =
        crate::command::play::current_card_use_enhancement_extra_targets(tx, player, card)?
    else {
        return Ok(Vec::new());
    };
    let mut options = Vec::with_capacity(4);
    if !extra_targets.is_empty() {
        options.push(0);
    }
    options.extend([1, 2, 3]);
    open_choice(
        tx,
        player,
        1,
        2,
        false,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ChooseCurrentCardUseEnhancements,
        vec![
            player.raw(),
            card.raw(),
            helper_skill_def.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn add_card_use_policy_skill(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    policy_mask: u8,
    skill_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let policy_mask = policy_mask & 0b111;
    if policy_mask == 0 || card.is_none() || skill_def.is_none() {
        return Ok(Vec::new());
    }
    let mut entries = tx
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
                .and_then(|instance| instance.card_use_policies())
        })
        .unwrap_or_default();
    if let Some((_, mask)) = entries.iter_mut().find(|(entry, _)| *entry == card) {
        *mask |= policy_mask;
    } else {
        entries.push((card, policy_mask));
    }
    add_skill_with_state(
        tx,
        player,
        skill_def,
        Some(crate::state::SkillInstance::card_use_policy_state(&entries)),
    )
}

pub(crate) fn skill_pile_def(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<u32, EffectError> {
    let state = tx.state();
    state
        .skills
        .get(skill)
        .filter(|instance| instance.owner == player && state.skill_effectively_enabled(instance))
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::ContentFault("invalid skill-pile owner or skill".into()))
}

pub(crate) fn hand_sha_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    tx.state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                crate::query::effective_card_kind(tx.state(), player, card).is_sha()
            })
        })
        .collect()
}

pub(crate) fn choose_store_hand_sha_in_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !tx.state().skill_pile_cards(player, skill_def).is_empty()
    {
        return Ok(Vec::new());
    }
    let cards = hand_sha_cards(tx, player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        cards.len().min(u8::MAX as usize) as u8,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseStoreHandShaInSkillPile,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn choose_spend_skill_pile_for_dying_jiu(
    tx: &mut Transaction,
    player: PlayerId,
    dying: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !tx.state().players.get(dying).is_some_and(|participant| {
            participant.alive && participant.dying && participant.hp <= 0
        })
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, skill_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseSpendSkillPileForDyingJiu,
        vec![player.raw(), dying.raw(), skill.raw()],
    )
}

pub(crate) fn ensure_skill_pile_zone(
    tx: &mut Transaction,
    player: PlayerId,
    skill_def: u32,
) -> Result<ZoneId, EffectError> {
    if let Some(zone) = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == skill_def)
        .map(|pile| pile.zone)
    {
        return tx
            .state()
            .zones
            .get(zone)
            .filter(|zone| zone.kind == ZoneKind::Expansion && zone.owner == player)
            .map(|zone| zone.id)
            .ok_or_else(|| EffectError::EngineFault("invalid skill-pile zone".into()));
    }
    let zone = tx.state_mut().zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner: player,
        cards: Vec::new(),
    });
    tx.state_mut().skill_piles.push(crate::state::SkillPileRef {
        owner: player,
        skill_def,
        zone,
        usable_as_hand: false,
        wants_lost_count: false,
    });
    tx.state_mut()
        .skill_piles
        .sort_by_key(|pile| (pile.owner.raw(), pile.skill_def));
    Ok(zone)
}

pub(crate) fn commit_store_hand_sha_in_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if cards.is_empty() || !tx.state().skill_pile_cards(player, skill_def).is_empty() {
        return Err(EffectError::Illegal(
            "skill-pile storage requires selected cards and an empty pile".into(),
        ));
    }
    let legal: std::collections::BTreeSet<_> = hand_sha_cards(tx, player).into_iter().collect();
    let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    if selected.len() != cards.len() || !selected.is_subset(&legal) {
        return Err(EffectError::Illegal(
            "skill-pile storage must select distinct current hand Sha".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    move_cards(tx, cards, pile, Some(hand))
}

pub(crate) fn store_selected_in_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if cards.is_empty() {
        // 死腿修复（批 215）：DSL 直排语句没有前置选择阶段，cards 恒空导致该腿
        // 在已部署件中静默无效——改为自开 SelectCards 1/1（从当前手牌，必选）。
        if !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        {
            return Ok(Vec::new());
        }
        let hand_cards: Vec<u32> = tx
            .state()
            .hand_cards(player)
            .iter()
            .map(|card| card.raw())
            .collect();
        if hand_cards.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            hand_cards,
            ChoiceKind::SelectCards,
            ContinuationKind::StoreSelectedInSkillPile,
            vec![player.raw(), skill.raw()],
        );
    }
    let legal: std::collections::BTreeSet<_> =
        tx.state().hand_cards(player).into_iter().collect();
    let selected: std::collections::BTreeSet<_> = cards.iter().copied().collect();
    if selected.len() != cards.len() || !selected.is_subset(&legal) {
        return Err(EffectError::Illegal(
            "skill-pile storage must select distinct current hand cards".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    move_cards(tx, cards, pile, Some(hand))
}

pub(crate) fn gain_top_card_to_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile is missing".into()))?;
    if tx
        .state()
        .zones
        .get(draw)
        .is_none_or(|zone| zone.cards.is_empty())
    {
        reshuffle_discard_into_draw(tx)?;
    }
    let Some(card) = tx
        .state()
        .zones
        .get(draw)
        .and_then(|zone| zone.cards.last())
        .copied()
    else {
        return Ok(Vec::new());
    };
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    move_cards(tx, vec![card], pile, Some(draw))
}

pub(crate) fn choose_discard_skill_pile_card_add_marked_skill(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: engine_api::SkillDefId,
    mut selected: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if player == target
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("invalid marked-skill target".into()));
    }
    let cards = tx.state().skill_pile_cards(player, skill_def);
    if cards.is_empty() {
        return Err(EffectError::Illegal("current skill pile is empty".into()));
    }
    if selected.is_none() && cards.len() > 1 {
        open_choice(
            tx,
            player,
            1,
            1,
            false,
            cards.iter().map(|card| card.raw()).collect(),
            ChoiceKind::Custom,
            ContinuationKind::ChooseDiscardSkillPileCardAddMarkedSkill,
            vec![
                player.raw(),
                target.raw(),
                helper_skill_def.raw(),
                skill.raw(),
            ],
        )?;
        return Err(EffectError::ChoiceRequested);
    }
    if selected.is_none() {
        selected = cards[0];
    }
    if !cards.contains(&selected) {
        return Err(EffectError::Illegal(
            "selected card is not in the current skill pile".into(),
        ));
    }
    let from = tx
        .state()
        .cards
        .get(selected)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("selected pile card is missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile is missing".into()))?;
    tx.state_mut().resolution.deferred_tail_effects.extend([
        EffectRequest::AddSkill {
            player: target,
            skill_def: helper_skill_def,
        },
        EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(helper_skill_def.raw()),
            delta: 1,
        },
    ]);
    move_cards_with_reason(
        tx,
        vec![selected],
        discard,
        Some(from),
        CardMoveReason::Discard,
    )
}

pub(crate) fn end_match_for_camp_if_skill_pile_at_least(
    tx: &mut Transaction,
    player: PlayerId,
    pile_skill_def: engine_api::SkillDefId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().outcome.is_finished()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || tx
            .state()
            .skill_pile_cards(player, pile_skill_def.raw())
            .len()
            < count as usize
    {
        return Ok(Vec::new());
    }
    let identity = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad camp winner".into()))?
        .identity;
    let outcome = match identity {
        Identity::Lord | Identity::Loyalist => MatchOutcome::LoyalistWin,
        Identity::Rebel => MatchOutcome::RebelWin,
        Identity::Renegade => MatchOutcome::RenegadeWin { player },
    };
    tx.journal(JournalEntry::OutcomeCheckpoint);
    tx.state_mut().outcome = outcome.clone();
    Ok(vec![DomainEvent::MatchEnded { outcome }])
}

pub(crate) fn commit_spend_skill_pile_for_dying_jiu(
    tx: &mut Transaction,
    player: PlayerId,
    dying: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(dying)
        .is_some_and(|participant| participant.alive && participant.dying && participant.hp <= 0)
        || !tx
            .state()
            .skill_pile_cards(player, skill_def)
            .contains(&card)
    {
        return Err(EffectError::Illegal(
            "dying Jiu requires the current dying player and one current skill-pile card".into(),
        ));
    }
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == skill_def)
        .map(|pile| pile.zone)
        .ok_or_else(|| EffectError::EngineFault("skill-pile zone missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut events = move_cards(tx, vec![card], discard, Some(pile))?;
    events.extend(use_virtual_card_on(tx, dying, CardKind::Jiu.tag(), dying)?);
    Ok(events)
}

pub(crate) fn potweiyan_status(tx: &Transaction, skill: engine_api::SkillId) -> u8 {
    tx.state()
        .skills
        .get(skill)
        .and_then(|instance| instance.state_blob.strip_prefix(b"PWY1"))
        .and_then(|rest| rest.first())
        .copied()
        .unwrap_or(0)
}

pub(crate) fn set_potweiyan_status(tx: &mut Transaction, skill: engine_api::SkillId, status: u8) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"PWY1".as_slice(), &[status]].concat();
    }
}

pub(crate) fn set_potkuanggu_upgraded(tx: &mut Transaction, skill: engine_api::SkillId) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = b"PKG1".to_vec();
    }
}

pub(crate) fn potkuanggu_upgraded(tx: &Transaction, skill: engine_api::SkillId) -> bool {
    tx.state()
        .skills
        .get(skill)
        .is_some_and(|instance| instance.state_blob == b"PKG1")
}

pub(crate) fn ensure_marked_skill(
    tx: &mut Transaction,
    player: PlayerId,
    def: engine_api::SkillDefId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = if skill_instance_by_def(tx, player, def).is_none() {
        add_skill(tx, player, def)?
    } else {
        Vec::new()
    };
    let skill = skill_instance_by_def(tx, player, def)
        .ok_or_else(|| EffectError::EngineFault("marked helper was not added".into()))?;
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.mark = instance.mark.saturating_add(count);
    }
    events.push(DomainEvent::RuleNote {
        code: 558,
        payload: vec![player.raw(), def.raw(), count],
    });
    Ok(events)
}

pub(crate) fn fail_potweiyan_mission(
    tx: &mut Transaction,
    player: PlayerId,
    mission: engine_api::SkillId,
    zhuangshi_def: engine_api::SkillDefId,
    kunfen_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if potweiyan_status(tx, mission) != 0 {
        return Ok(Vec::new());
    }
    set_potweiyan_status(tx, mission, 2);
    let mut events = remove_skill_by_def(tx, player, zhuangshi_def)?;
    if skill_instance_by_def(tx, player, kunfen_def).is_none() {
        events.extend(add_skill(tx, player, kunfen_def)?);
    }
    events.push(DomainEvent::RuleNote {
        code: 558,
        payload: vec![player.raw(), 2],
    });
    Ok(events)
}

pub(crate) fn potyinzhan_entries(
    tx: &Transaction,
    skill: engine_api::SkillId,
) -> (CardId, Vec<(PlayerId, bool)>) {
    let Some(blob) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.state_blob.as_slice())
        .filter(|blob| blob.starts_with(b"PYZ1") && blob.len() >= 9)
    else {
        return (CardId::NONE, Vec::new());
    };
    let card = CardId::new(u32::from_le_bytes(blob[4..8].try_into().unwrap()));
    let count = blob[8] as usize;
    if blob.len() != 9 + count * 5 {
        return (CardId::NONE, Vec::new());
    }
    let entries = blob[9..]
        .chunks_exact(5)
        .map(|chunk| {
            (
                PlayerId::new(u32::from_le_bytes(chunk[..4].try_into().unwrap())),
                chunk[4] != 0,
            )
        })
        .collect();
    (card, entries)
}

pub(crate) fn set_potyinzhan_entries(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    card: CardId,
    entries: &[(PlayerId, bool)],
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        let mut blob = b"PYZ1".to_vec();
        blob.extend_from_slice(&card.raw().to_le_bytes());
        blob.push(entries.len().min(u8::MAX as usize) as u8);
        for (target, both) in entries.iter().take(u8::MAX as usize) {
            blob.extend_from_slice(&target.raw().to_le_bytes());
            blob.push(u8::from(*both));
        }
        instance.state_blob = blob;
    }
}

pub(crate) fn resolve_potweiyan(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kuanggu_def = engine_api::SkillDefId::new(POTWEIYAN_KUANGGU_DEF);
    let zhuangshi_def = engine_api::SkillDefId::new(POTWEIYAN_ZHUANGSHI_DEF);
    let kunfen_def = engine_api::SkillDefId::new(POTWEIYAN_KUNFEN_DEF);
    let direct_def = engine_api::SkillDefId::new(POTWEIYAN_DIRECT_DEF);
    let limit_def = engine_api::SkillDefId::new(POTWEIYAN_LIMIT_DEF);
    let kuanggu_limit_def = engine_api::SkillDefId::new(POTWEIYAN_KUANGGU_LIMIT_DEF);
    if action <= 10 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if potweiyan_status(tx, skill) == 0
                && skill_instance_by_def(tx, player, kuanggu_def).is_none()
            {
                add_skill(tx, player, kuanggu_def)
            } else {
                Ok(Vec::new())
            }
        }
        1 => {
            if potweiyan_status(tx, skill) != 0 {
                return Ok(Vec::new());
            }
            set_potweiyan_status(tx, skill, 1);
            if let Some(kuanggu) = skill_instance_by_def(tx, player, kuanggu_def) {
                set_potkuanggu_upgraded(tx, kuanggu);
            }
            let direct_left = skill_instance_by_def(tx, player, direct_def)
                .and_then(|id| tx.state().skills.get(id))
                .map_or(0, |instance| instance.mark);
            let limit_left = skill_instance_by_def(tx, player, limit_def)
                .and_then(|id| tx.state().skills.get(id))
                .map_or(0, |instance| instance.mark);
            let mut effects = Vec::new();
            if limit_left > 0 {
                effects.push(EffectRequest::DrawCards { player, count: 1 });
            }
            if direct_left > 0 {
                let damaged = tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|participant| participant.hp < participant.max_hp as i32);
                effects.push(if damaged {
                    EffectRequest::RecoverHp {
                        player,
                        amount: 1,
                        source: PlayerId::NONE,
                    }
                } else {
                    EffectRequest::DrawCards { player, count: 1 }
                });
            }
            run_effects(tx, effects)?;
            Ok(vec![DomainEvent::RuleNote {
                code: 558,
                payload: vec![player.raw(), 1],
            }])
        }
        2 => {
            if potweiyan_status(tx, skill) != 0
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || skill_instance_by_def(tx, player, zhuangshi_def).is_none()
            {
                return Ok(Vec::new());
            }
            let hand = crate::query::discardable_hand_equip_cards(tx.state(), player)
                .into_iter()
                .filter(|card| tx.state().hand_cards(player).contains(card))
                .collect::<Vec<_>>();
            if hand.is_empty() {
                return open_potweiyan_hp_choice(
                    tx,
                    player,
                    skill,
                    kuanggu_def,
                    zhuangshi_def,
                    kunfen_def,
                    direct_def,
                    limit_def,
                    kuanggu_limit_def,
                    0,
                );
            }
            open_choice(
                tx,
                player,
                1,
                hand.len().min(u8::MAX as usize) as u8,
                true,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    POTWEIYAN_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    skill.raw(),
                    kuanggu_def.raw(),
                    zhuangshi_def.raw(),
                    kunfen_def.raw(),
                    direct_def.raw(),
                    limit_def.raw(),
                    kuanggu_limit_def.raw(),
                ],
            )
        }
        3 => fail_potweiyan_mission(tx, player, skill, zhuangshi_def, kunfen_def),
        4 | 5 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.mark = instance.mark.saturating_sub(1);
                if instance.mark == 0 {
                    return remove_skill(tx, skill);
                }
            }
            Ok(Vec::new())
        }
        6 => remove_skill(tx, skill),
        7 => {
            let owner = tx
                .state()
                .players
                .get(player)
                .ok_or_else(|| EffectError::Illegal("bad 饮战 owner".into()))?;
            let victim = tx
                .state()
                .players
                .get(target)
                .ok_or_else(|| EffectError::Illegal("bad 饮战 target".into()))?;
            let hp_branch = owner.hp <= victim.hp;
            let hand_branch = tx.state().hand_count(player) <= tx.state().hand_count(target);
            if hp_branch {
                increase_bound_damage_by_one(tx, tx.state().resolution.current_damage_id)?;
            }
            if hand_branch {
                let (stored_card, mut entries) = potyinzhan_entries(tx, skill);
                if entries.is_empty() || stored_card == card {
                    entries.push((target, hp_branch));
                    set_potyinzhan_entries(tx, skill, card, &entries);
                }
            }
            Ok(Vec::new())
        }
        8 => continue_potyinzhan(tx, player, card, skill),
        9 => {
            let damaged = tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.hp < participant.max_hp as i32);
            let mut options = vec![1];
            if damaged {
                options.insert(0, 0);
            }
            if potkuanggu_upgraded(tx, skill)
                && !crate::query::discardable_hand_equip_cards(tx.state(), player).is_empty()
            {
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
                    POTWEIYAN_CUSTOM_CHOICE,
                    3,
                    player.raw(),
                    skill.raw(),
                    kuanggu_limit_def.raw(),
                ],
            )
        }
        10 => {
            run_effects(
                tx,
                vec![
                    EffectRequest::DealDamage {
                        source: PlayerId::NONE,
                        target: player,
                        amount: 1,
                        is_lose_hp: true,
                        nature: 0,
                    },
                    EffectRequest::DrawCards { player, count: 2 },
                ],
            )?;
            Ok(Vec::new())
        }
        11 => finish_potyinzhan_entry(tx, player, target, card, skill),
        12 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return ensure_marked_skill(tx, player, kuanggu_limit_def, 1);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    POTWEIYAN_CUSTOM_CHOICE,
                    4,
                    player.raw(),
                    skill.raw(),
                    kuanggu_limit_def.raw(),
                ],
            )
        }
        13 => ensure_marked_skill(tx, player, kuanggu_limit_def, 1),
        14 => {
            let count = card.raw();
            let mut events = ensure_marked_skill(tx, player, direct_def, count)?;
            events.extend(open_potweiyan_hp_choice(
                tx,
                player,
                skill,
                kuanggu_def,
                zhuangshi_def,
                kunfen_def,
                direct_def,
                limit_def,
                kuanggu_limit_def,
                count,
            )?);
            Ok(events)
        }
        15 => ensure_marked_skill(tx, player, limit_def, card.raw()),
        _ => Err(EffectError::ContentFault("unknown 势魏延 action".into())),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn open_potweiyan_hp_choice(
    tx: &mut Transaction,
    player: PlayerId,
    mission: engine_api::SkillId,
    kuanggu_def: engine_api::SkillDefId,
    zhuangshi_def: engine_api::SkillDefId,
    kunfen_def: engine_api::SkillDefId,
    direct_def: engine_api::SkillDefId,
    limit_def: engine_api::SkillDefId,
    kuanggu_limit_def: engine_api::SkillDefId,
    discarded: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hp = tx
        .state()
        .players
        .get(player)
        .map_or(0, |participant| participant.hp.max(0) as u32);
    if hp == 0 {
        return fail_potweiyan_mission(tx, player, mission, zhuangshi_def, kunfen_def);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        (1..=hp).collect(),
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            POTWEIYAN_CUSTOM_CHOICE,
            1,
            player.raw(),
            mission.raw(),
            kuanggu_def.raw(),
            zhuangshi_def.raw(),
            kunfen_def.raw(),
            direct_def.raw(),
            limit_def.raw(),
            kuanggu_limit_def.raw(),
            discarded,
        ],
    )
}

pub(crate) fn continue_potyinzhan(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let (stored_card, entries) = potyinzhan_entries(tx, skill);
    if entries.is_empty() || stored_card != card {
        return Ok(Vec::new());
    }
    let (target, _) = entries[0];
    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
    if options.is_empty() {
        return finish_potyinzhan_entry(tx, player, target, CardId::NONE, skill);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            POTWEIYAN_CUSTOM_CHOICE,
            2,
            player.raw(),
            skill.raw(),
            target.raw(),
        ],
    )
}

pub(crate) fn finish_potyinzhan_entry(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    discarded: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let (card, mut entries) = potyinzhan_entries(tx, skill);
    let Some((expected, both)) = entries.first().copied() else {
        return Ok(Vec::new());
    };
    if expected != target {
        return Err(EffectError::Illegal("饮战 target changed".into()));
    }
    entries.remove(0);
    set_potyinzhan_entries(tx, skill, card, &entries);
    let mut effects = Vec::new();
    if both {
        effects.push(EffectRequest::RecoverHp {
            player,
            amount: 1,
            source: PlayerId::NONE,
        });
        if discarded.is_some()
            && tx.state().cards.get(discarded).is_some_and(|card| {
                tx.state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .is_some_and(|discard| card.zone == discard)
            })
        {
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("hand missing".into()))?;
            effects.push(EffectRequest::MoveCards {
                cards: vec![discarded],
                to: hand,
                from: tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE),
            });
        }
    }
    effects.push(EffectRequest::ResolvePotweiyan {
        player,
        target: PlayerId::NONE,
        card,
        action: 8,
        skill,
    });
    run_effects(tx, effects)?;
    Ok(Vec::new())
}

pub(crate) const POTZHOUYU_CHIYUN_DEF: u32 = 4_200_000_566;
pub(crate) const POTZHOUYU_YANHUI_DEF: u32 = 4_200_000_567;
pub(crate) const POTZHOUYU_FENTAO_DEF: u32 = 4_200_000_568;
pub(crate) const POTZHOUYU_DISABLE_ONE: u64 = 1 << 1;
pub(crate) const POTZHOUYU_DISABLE_TWO: u64 = 1 << 2;

pub(crate) fn potzhouyu_turn_allowed(tx: &Transaction, skill: engine_api::SkillId) -> bool {
    tx.state().skills.get(skill).is_some_and(|instance| {
        instance.content_flags & (POTZHOUYU_DISABLE_ONE | POTZHOUYU_DISABLE_TWO) == 0
            || tx.state().turn.current == instance.owner
    })
}

pub(crate) fn potzhouyu_branch_allowed(tx: &Transaction, skill: engine_api::SkillId, branch: u8) -> bool {
    tx.state().skills.get(skill).is_some_and(|instance| {
        let flag = if branch == 1 {
            POTZHOUYU_DISABLE_ONE
        } else {
            POTZHOUYU_DISABLE_TWO
        };
        instance.content_flags & flag == 0
    })
}

#[derive(Default)]
pub(crate) struct PotyanhuiState {
    armed: bool,
    revealed: Vec<(CardId, PlayerId)>,
    discarded: Vec<PlayerId>,
}

pub(crate) fn potyanhui_state(tx: &Transaction, skill: engine_api::SkillId) -> PotyanhuiState {
    let Some(blob) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.state_blob.as_slice())
    else {
        return PotyanhuiState::default();
    };
    if !blob.starts_with(b"PYH1") || blob.len() < 6 {
        return PotyanhuiState::default();
    }
    let armed = blob[4] != 0;
    let revealed_len = blob[5] as usize;
    let mut offset = 6;
    if blob.len() < offset + revealed_len * 8 + 1 {
        return PotyanhuiState::default();
    }
    let mut revealed = Vec::with_capacity(revealed_len);
    for chunk in blob[offset..offset + revealed_len * 8].chunks_exact(8) {
        revealed.push((
            CardId::new(u32::from_le_bytes(chunk[..4].try_into().unwrap())),
            PlayerId::new(u32::from_le_bytes(chunk[4..].try_into().unwrap())),
        ));
    }
    offset += revealed_len * 8;
    let discarded_len = blob[offset] as usize;
    offset += 1;
    if blob.len() != offset + discarded_len * 4 {
        return PotyanhuiState::default();
    }
    let discarded = blob[offset..]
        .chunks_exact(4)
        .map(|raw| PlayerId::new(u32::from_le_bytes(raw.try_into().unwrap())))
        .collect();
    PotyanhuiState {
        armed,
        revealed,
        discarded,
    }
}

pub(crate) fn set_potyanhui_state(tx: &mut Transaction, skill: engine_api::SkillId, state: &PotyanhuiState) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        let mut blob = b"PYH1".to_vec();
        blob.push(u8::from(state.armed));
        blob.push(state.revealed.len().min(u8::MAX as usize) as u8);
        for (card, player) in state.revealed.iter().take(u8::MAX as usize) {
            blob.extend_from_slice(&card.raw().to_le_bytes());
            blob.extend_from_slice(&player.raw().to_le_bytes());
        }
        blob.push(state.discarded.len().min(u8::MAX as usize) as u8);
        for player in state.discarded.iter().take(u8::MAX as usize) {
            blob.extend_from_slice(&player.raw().to_le_bytes());
        }
        instance.state_blob = blob;
    }
}

pub(crate) fn set_potfentao_pending(tx: &mut Transaction, skill: engine_api::SkillId, target: PlayerId) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"PFT1".as_slice(), &target.raw().to_le_bytes()].concat();
    }
}

pub(crate) fn add_potfentao_damage(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    propagated: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let delta = if propagated {
        let instance = tx
            .state_mut()
            .skills
            .get_mut(skill)
            .ok_or_else(|| EffectError::Illegal("焚涛技能已失效".into()))?;
        instance.mark = instance.mark.saturating_add(1);
        instance.mark
    } else {
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.mark = 0;
        }
        1
    };
    increase_bound_damage(tx, tx.state().resolution.current_damage_id, delta)?;
    Ok(vec![DomainEvent::RuleNote {
        code: 561,
        payload: vec![skill.raw(), delta],
    }])
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_potzhouyu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    card_flags: u8,
    targets: &[PlayerId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action <= 7 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if !potzhouyu_turn_allowed(tx, skill) {
                return Ok(Vec::new());
            }
            let Some(instance) = tx.state_mut().skills.get_mut(skill) else {
                return Ok(Vec::new());
            };
            if instance.content_flags & 1 != 0 {
                return Ok(Vec::new());
            }
            instance.content_flags |= 1;
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                cards.len().min(u8::MAX as usize) as u8,
                true,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![POTZHOUYU_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if !potzhouyu_turn_allowed(tx, skill) {
                return Ok(Vec::new());
            }
            let eligible: Vec<_> = targets
                .iter()
                .copied()
                .filter(|candidate| {
                    tx.state()
                        .players
                        .get(*candidate)
                        .is_some_and(|role| role.alive)
                        && tx.state().hand_count(*candidate) > 0
                })
                .collect();
            if eligible.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                eligible.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![POTZHOUYU_CUSTOM_CHOICE, 3, player.raw(), skill.raw()],
            )
        }
        2 => {
            let mut state = potyanhui_state(tx, skill);
            if !state.armed {
                return Ok(Vec::new());
            }
            state.armed = false;
            let damage_targets: Vec<_> = state
                .discarded
                .iter()
                .copied()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|role| role.alive)
                })
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect();
            state.discarded.clear();
            let draw_count = state
                .revealed
                .iter()
                .map(|(_, role)| *role)
                .collect::<std::collections::BTreeSet<_>>()
                .len() as u32;
            set_potyanhui_state(tx, skill, &state);
            let mut options = Vec::new();
            if draw_count > 0 && potzhouyu_branch_allowed(tx, skill, 2) {
                options.push(0);
            }
            if potzhouyu_branch_allowed(tx, skill, 1) {
                options.extend(damage_targets.iter().map(|target| target.raw()));
            }
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![
                POTZHOUYU_CUSTOM_CHOICE,
                5,
                player.raw(),
                skill.raw(),
                draw_count,
                damage_targets.len() as u32,
            ];
            payload.extend(damage_targets.iter().map(|target| target.raw()));
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
        3 => {
            let propagated = card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED != 0;
            if !propagated {
                if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                    instance.mark = 0;
                }
            }
            if !potzhouyu_turn_allowed(tx, skill) {
                return Ok(Vec::new());
            }
            let linked_other = tx.state().players.get(target).is_some_and(|role| {
                role.alive
                    && target != player
                    && role
                        .marks
                        .iter()
                        .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
            });
            if !linked_other {
                return Ok(Vec::new());
            }
            let allow_one = potzhouyu_branch_allowed(tx, skill, 1);
            let allow_two = potzhouyu_branch_allowed(tx, skill, 2);
            let discardable = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let equip_count = tx
                .state()
                .zones
                .find(ZoneKind::Equip, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .map_or(0, |zone| zone.cards.len());
            let required = (tx.state().hand_count(target) + equip_count as u32).div_ceil(2);
            let can_discard = required > 0 && discardable.len() as u32 >= required;
            if !allow_two || !can_discard {
                return if allow_one {
                    add_potfentao_damage(tx, skill, propagated)
                } else {
                    Ok(Vec::new())
                };
            }
            let required_u8 = required.min(u8::MAX as u32) as u8;
            open_choice(
                tx,
                target,
                required_u8,
                required_u8,
                allow_one,
                discardable.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    POTZHOUYU_CUSTOM_CHOICE,
                    6,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                    u32::from(propagated),
                    required as u32,
                ],
            )
        }
        4 => {
            let pending = tx.state().skills.get(skill).is_some_and(|instance| {
                instance.state_blob.starts_with(b"PFT1")
                    && instance.state_blob.get(4..8).is_some_and(|raw| {
                        u32::from_le_bytes(raw.try_into().unwrap()) == target.raw()
                    })
            });
            if !pending {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob.clear();
            }
            let linked = tx.state().players.get(target).is_some_and(|role| {
                role.marks
                    .iter()
                    .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
            });
            if linked {
                Ok(Vec::new())
            } else {
                add_mark(tx, target, crate::content::MARK_CHAIN, 1)
            }
        }
        5 => {
            let mut state = potyanhui_state(tx, skill);
            state.revealed.clear();
            state.discarded.clear();
            state.armed = false;
            set_potyanhui_state(tx, skill, &state);
            Ok(Vec::new())
        }
        6 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Prepare
                || tx
                    .state()
                    .skills
                    .get(skill)
                    .is_none_or(|instance| instance.lifetime_fire_count > 0)
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![POTZHOUYU_CUSTOM_CHOICE, 8, player.raw(), skill.raw()],
            )
        }
        7 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.content_flags &= !1;
            }
            Ok(Vec::new())
        }
        8 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|role| role.alive)
            {
                return Ok(Vec::new());
            }
            let color_mask = card.raw() as u8;
            let matching: Vec<_> = tx
                .state()
                .hand_cards(target)
                .into_iter()
                .filter(|candidate| {
                    crate::query::effective_card_color_mask(tx.state(), target, *candidate)
                        & color_mask
                        != 0
                })
                .collect();
            let mut options = Vec::new();
            if !matching.is_empty() && potzhouyu_branch_allowed(tx, skill, 1) {
                options.push(1);
            }
            if potzhouyu_branch_allowed(tx, skill, 2) {
                options.push(2);
            }
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    POTZHOUYU_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                    u32::from(color_mask),
                ],
            )
        }
        _ => Err(EffectError::ContentFault("unknown 势周瑜 action".into())),
    }
}

pub(crate) fn resolve_potzhouyu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 4 || payload[0] != POTZHOUYU_CUSTOM_CHOICE {
        return Err(EffectError::EngineFault("bad 势周瑜 payload".into()));
    }
    let player = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal("势周瑜技能已失效".into()));
    }
    match payload[1] {
        0 => {
            if observer != player {
                return Err(EffectError::Illegal("炽沄选牌者变化".into()));
            }
            if cancelled {
                return Ok(());
            }
            if selections.is_empty() {
                return Err(EffectError::Illegal("炽沄至少交一张牌".into()));
            }
            let hand: std::collections::BTreeSet<_> =
                tx.state().hand_cards(player).into_iter().collect();
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != cards.len()
                || cards.iter().any(|card| !hand.contains(card))
            {
                return Err(EffectError::Illegal("炽沄手牌已变化".into()));
            }
            let color_mask = cards.iter().fold(0, |mask, card| {
                mask | crate::query::effective_card_color_mask(tx.state(), player, *card)
            });
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .collect();
            let mut next = vec![
                POTZHOUYU_CUSTOM_CHOICE,
                1,
                player.raw(),
                skill.raw(),
                u32::from(color_mask),
                cards.len() as u32,
            ];
            next.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        1 => {
            if observer != player || cancelled || selections.len() != 1 || payload.len() < 6 {
                return Err(EffectError::Illegal("炽沄目标选择无效".into()));
            }
            let count = payload[5] as usize;
            if payload.len() != 6 + count {
                return Err(EffectError::EngineFault("bad 炽沄 cards payload".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|role| role.alive)
            {
                return Err(EffectError::Illegal("炽沄目标已变化".into()));
            }
            let cards: Vec<_> = payload[6..].iter().copied().map(CardId::new).collect();
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if cards.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_none_or(|value| Some(value.zone) != hand || value.owner != player)
            }) {
                return Err(EffectError::Illegal("炽沄手牌已变化".into()));
            }
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("target hand missing".into()))?;
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards,
                        to: target_hand,
                        from: hand,
                    },
                    EffectRequest::ResolvePotzhouyu {
                        player,
                        target,
                        card: CardId::new(payload[4]),
                        action: 8,
                        card_flags: 0,
                        targets: Vec::new(),
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        2 => {
            if payload.len() != 6 || observer.raw() != payload[4] || cancelled {
                return Err(EffectError::Illegal("炽沄分支选择无效".into()));
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal("炽沄分支缺失".into()));
            };
            let target = PlayerId::new(payload[4]);
            match *choice {
                1 if potzhouyu_branch_allowed(tx, skill, 1) => {
                    let mask = payload[5] as u8;
                    let cards: Vec<_> = tx
                        .state()
                        .hand_cards(target)
                        .into_iter()
                        .filter(|card| {
                            crate::query::effective_card_color_mask(tx.state(), target, *card)
                                & mask
                                != 0
                        })
                        .collect();
                    if cards.is_empty() {
                        return Err(EffectError::Illegal("炽沄同色手牌已变化".into()));
                    }
                    run_effects(
                        tx,
                        vec![
                            EffectRequest::RevealCards {
                                cards,
                                to: Vec::new(),
                            },
                            EffectRequest::DealDamage {
                                source: player,
                                target,
                                amount: 1,
                                is_lose_hp: false,
                                nature: rule_api::damage_nature::FIRE,
                            },
                        ],
                    )
                    .map(|_| ())
                }
                2 if potzhouyu_branch_allowed(tx, skill, 2) => {
                    let mut effects = vec![EffectRequest::DrawCards { player, count: 2 }];
                    let linked = tx.state().players.get(target).is_some_and(|role| {
                        role.marks
                            .iter()
                            .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
                    });
                    if !linked {
                        effects.push(EffectRequest::AddMark {
                            player: target,
                            mark: engine_api::MarkDefId::new(crate::content::MARK_CHAIN),
                            delta: 1,
                        });
                    }
                    run_effects(tx, effects).map(|_| ())
                }
                _ => Err(EffectError::Illegal("炽沄分支已禁用".into())),
            }
        }
        3 => {
            if observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("焰洄目标选择无效".into()));
            }
            let target = PlayerId::new(selections[0]);
            let cards = tx.state().hand_cards(target);
            if cards.is_empty() {
                return Err(EffectError::Illegal("焰洄目标已无手牌".into()));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    POTZHOUYU_CUSTOM_CHOICE,
                    4,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                ],
            )
            .map(|_| ())
        }
        4 => {
            if payload.len() != 5 || observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("焰洄展示选择无效".into()));
            }
            let target = PlayerId::new(payload[4]);
            let card = CardId::new(selections[0]);
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal("焰洄手牌已变化".into()));
            }
            let mut state = potyanhui_state(tx, skill);
            let seen = state.revealed.iter().any(|(shown, _)| *shown == card);
            if !state.revealed.iter().any(|entry| *entry == (card, target)) {
                state.revealed.push((card, target));
            }
            state.armed = true;
            if seen && !state.discarded.contains(&target) {
                state.discarded.push(target);
            }
            set_potyanhui_state(tx, skill, &state);
            let mut effects = vec![EffectRequest::RevealCards {
                cards: vec![card],
                to: Vec::new(),
            }];
            if seen {
                effects.push(EffectRequest::DiscardCards {
                    player: target,
                    cards: vec![card],
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        5 => {
            if payload.len() < 6 || observer != player || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("焰洄阶段结算选择无效".into()));
            }
            let target_count = payload[5] as usize;
            if payload.len() != 6 + target_count {
                return Err(EffectError::EngineFault("bad 焰洄阶段 payload".into()));
            }
            let choice = selections[0];
            if choice == 0 {
                if !potzhouyu_branch_allowed(tx, skill, 2) || payload[4] == 0 {
                    return Err(EffectError::Illegal("焰洄摸牌分支无效".into()));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DrawCards {
                        player,
                        count: payload[4],
                    }],
                )
                .map(|_| ())
            } else {
                let target = PlayerId::new(choice);
                if !potzhouyu_branch_allowed(tx, skill, 1)
                    || !payload[6..].contains(&choice)
                    || !tx
                        .state()
                        .players
                        .get(target)
                        .is_some_and(|role| role.alive)
                {
                    return Err(EffectError::Illegal("焰洄伤害目标无效".into()));
                }
                run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::FIRE,
                    }],
                )
                .map(|_| ())
            }
        }
        6 => {
            if payload.len() != 7 || observer.raw() != payload[4] {
                return Err(EffectError::Illegal("焚涛弃牌选择无效".into()));
            }
            let target = PlayerId::new(payload[4]);
            let propagated = payload[5] != 0;
            if cancelled {
                return add_potfentao_damage(tx, skill, propagated).map(|_| ());
            }
            let required = payload[6] as usize;
            if selections.len() != required {
                return Err(EffectError::Illegal("焚涛弃牌数量无效".into()));
            }
            let legal: std::collections::BTreeSet<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), target)
                    .into_iter()
                    .collect();
            let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            if cards
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != cards.len()
                || cards.iter().any(|card| !legal.contains(card))
            {
                return Err(EffectError::Illegal("焚涛弃牌已变化".into()));
            }
            set_potfentao_pending(tx, skill, target);
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards,
                }],
            )
            .map(|_| ())
        }
        8 => {
            if observer != player {
                return Err(EffectError::Illegal("雄姿选择者变化".into()));
            }
            if cancelled {
                return Ok(());
            }
            let [choice] = selections else {
                return Err(EffectError::Illegal("雄姿分支缺失".into()));
            };
            if !matches!(*choice, 1 | 2) {
                return Err(EffectError::Illegal("雄姿分支无效".into()));
            }
            let disable = if *choice == 1 {
                POTZHOUYU_DISABLE_TWO
            } else {
                POTZHOUYU_DISABLE_ONE
            };
            for def in [
                POTZHOUYU_CHIYUN_DEF,
                POTZHOUYU_YANHUI_DEF,
                POTZHOUYU_FENTAO_DEF,
            ] {
                if let Some(id) =
                    skill_instance_by_def(tx, player, engine_api::SkillDefId::new(def))
                {
                    if let Some(instance) = tx.state_mut().skills.get_mut(id) {
                        instance.content_flags |= disable;
                    }
                }
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.lifetime_fire_count = 1;
            }
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 2 }]).map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown 势周瑜 choice stage".into(),
        )),
    }
}

/// 置入 (expansion P2, ADR 0641, ABI 650): the owner selects one hand card
/// (any kind) and places it onto their skill pile, reusing the
/// CommitStoreHandShaInSkillPile pile-insertion machinery.
pub(crate) fn place_hand_card_to_expansion(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards: Vec<u32> = tx.state().hand_cards(player).iter().map(|c| c.raw()).collect();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        cards,
        ChoiceKind::SelectCards,
        ContinuationKind::PlaceHandCardToExpansion,
        vec![player.raw(), skill.raw()],
    )
}

/// 选他人授标记 (ABI 652): the chooser picks one other alive player and the
/// chosen player's mark is adjusted by `delta`.
pub(crate) fn choose_other_add_mark(
    tx: &mut Transaction,
    player: PlayerId,
    mark: u32,
    delta: i32,
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
    // Negative deltas are removal flows (jsrgpingrong removing 猎): only
    // targets that actually hold the mark are offered, mirroring upstream
    // hasMark-gated chooseTarget filters. Positive grants keep offering every
    // other living player.
    let clamp_negative = delta < 0;
    let current_count =
        |state: &crate::state::GameState, target: PlayerId| -> i32 {
            state
                .players
                .get(target)
                .and_then(|p| p.marks.iter().find(|(m, _)| *m == mark))
                .map_or(0, |(_, c)| *c)
        };
    let mut options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .filter(|&target| !clamp_negative || current_count(tx.state(), target) > 0)
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
        ContinuationKind::ChooseOtherAddMark,
        vec![player.raw(), mark, delta as u32, skill.raw()],
    )
}

/// 取用 (expansion P1, ADR 0641, ABI 651): the owner selects one card from
/// their own skill pile (武将牌上) and gains it into hand.
pub(crate) fn place_top_deck_to_expansion(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    // Provenance gate on the originating skill instance; the pile is addressed by
    // the `pile_def` literal so sibling piles can be supplied.
    let _ = skill_pile_def(tx, player, skill)?;
    if count == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let top: Vec<CardId> = {
        let cards = &tx.state().zones.get(draw).unwrap().cards;
        cards.iter().take(count as usize).copied().collect()
    };
    if top.is_empty() {
        return Ok(Vec::new());
    }
    let pile = ensure_skill_pile_zone(tx, player, pile_def)?;
    move_cards_with_reason(tx, top, pile, Some(draw), CardMoveReason::Unspecified)
}

pub(crate) fn choose_one_from_skill_pile_discard(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, pile_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        cards.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseOneFromSkillPileDiscard,
        vec![player.raw(), pile_def, skill.raw()],
    )
}

pub(crate) fn place_hand_cards_to_deck_top(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill_pile_def(tx, player, skill)?;
    if count == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let hand_cards: Vec<u32> = {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
        tx.state()
            .zones
            .get(hand)
            .unwrap()
            .cards
            .iter()
            .map(|c| c.raw())
            .collect()
    };
    if hand_cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        count as u8,
        count as u8,
        true,
        hand_cards,
        ChoiceKind::SelectCards,
        ContinuationKind::PlaceHandCardsToDeckTop,
        vec![player.raw(), skill.raw(), count],
    )
}

pub(crate) fn gain_whole_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, pile_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|p| p.owner == player && p.skill_def == pile_def)
        .map(|p| p.zone)
        .ok_or_else(|| EffectError::Illegal("skill pile missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    move_cards_with_reason(tx, cards, hand, Some(pile), CardMoveReason::Unspecified)
}

pub(crate) fn choose_target_hand_card_to_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && !tx.state().hand_cards(target).is_empty())
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetHandCardToPile,
        vec![0, player.raw(), skill.raw(), pile_def],
    )
}

const RESOLVE_DCBINGJI_CUSTOM_CHOICE: u32 = 0xDCB0_0001;
/// 专属状态机 (批 239, ADR 0641 §8.34)：度望——游戏开始时，将牌堆顶前 5 张
/// 非【杀】牌置于自身技能堆（「刺」）。距离修正面保守省略（上游为有刺即 +1，
/// DSL 无 min(1,count) 修饰源，+N 超集不建）。
pub(crate) fn resolve_duwang(
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
    let skill_def = crate::effect::skill_pile_def(tx, player, skill)?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    // 牌堆顶 = 向量尾部；取前 5 张非【杀】。
    let top_cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let mut picked = Vec::new();
    for card in top_cards.iter().rev() {
        if picked.len() >= 5 {
            break;
        }
        if let Some(state) = tx.state().cards.get(*card) {
            if state.kind != crate::state::CardKind::Sha {
                picked.push(*card);
            }
        }
    }
    if picked.is_empty() {
        return Ok(Vec::new());
    }
    let pile = crate::effect::ensure_skill_pile_zone(tx, player, skill_def)?;
    crate::effect::move_cards(tx, picked, pile, Some(draw))?;
    Ok(Vec::new())
}

const RESOLVE_QIANLONG_MAGIC: u32 = 0x51_4C_4E_01;

const RESOLVE_ZUOZHAN_CUSTOM_CHOICE: u32 = 0x5A55_0001;

/// 「坐瞻」标记（记于所选角色，对应上游 markAuto("jsrgzuozhan") 的目标集合）。
pub(crate) const ZUOZHAN_SIT_MARK: u32 = 0x5A55_0002;

/// 专属状态机 (批 242, ADR 0641 §8.37)：坐瞻——游戏开始时，选择至多两名其他
/// 角色，获得 min(所选角色最高体力, 5) 枚「射程」标记（attack_range
/// owner_skill_mark 消费），所选角色各记一枚「坐瞻」标记。
pub(crate) fn resolve_jsrgzuozhan(
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
    let targets: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player)
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        2,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveJsrgzuozhanStage,
        vec![RESOLVE_ZUOZHAN_CUSTOM_CHOICE, player.raw(), skill.raw()],
    )
}

/// 续体：selected = 「坐瞻」角色（1..=2）；射程标记 = min(所选最高体力, 5)
/// 加于自身（标记 id = 技能 def，owner_skill_mark 修饰源按 def 读取；
/// self_skill 传入为实例 id，须经 skill_pile_def 解析）。
pub(crate) fn resolve_jsrgzuozhan_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    let mut best_hp = 0i32;
    for raw in selected {
        let hp = tx
            .state()
            .players
            .get(PlayerId::new(*raw))
            .filter(|participant| participant.alive)
            .map(|participant| participant.hp)
            .unwrap_or(0);
        best_hp = best_hp.max(hp);
    }
    let range = best_hp.clamp(0, 5) as u32;
    let mut events = Vec::new();
    if range > 0 {
        events.extend(crate::effect::add_mark(tx, player, skill_def, range as i32)?);
    }
    for raw in selected {
        events.extend(crate::effect::add_mark(
            tx,
            PlayerId::new(*raw),
            ZUOZHAN_SIT_MARK,
            1,
        )?);
    }
    Ok(events)
}

const RESOLVE_ZUOZHAN_GAIN_CUSTOM_CHOICE: u32 = 0x5A55_0003;

/// 基本牌名分组（上游「牌名各不相同」口径：杀/火杀/雷杀同名 sha）。
fn zuozhan_basic_name_group(kind: crate::state::CardKind) -> Option<u8> {
    match kind {
        crate::state::CardKind::Sha
        | crate::state::CardKind::ShaFire
        | crate::state::CardKind::ShaThunder => Some(1),
        crate::state::CardKind::Tao => Some(2),
        crate::state::CardKind::Shan => Some(3),
        crate::state::CardKind::Jiu => Some(4),
        _ => None,
    }
}

/// 从弃牌堆顶（向量尾）向堆底扫描，取至多 n 张牌名互不相同的基本牌。
fn zuozhan_distinct_basics_from_discard(
    tx: &Transaction,
    n: usize,
) -> Vec<engine_api::CardId> {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE);
    let Some(discard) = discard else {
        return Vec::new();
    };
    let cards = tx.state().zones.get(discard).unwrap().cards.clone();
    let mut picked = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for card in cards.iter().rev() {
        if picked.len() >= n {
            break;
        }
        let Some(group) = tx
            .state()
            .cards
            .get(*card)
            .and_then(|state| zuozhan_basic_name_group(state.kind))
        else {
            continue;
        };
        if seen.insert(group) {
            picked.push(*card);
        }
    }
    picked
}

/// 专属状态机 (批 243, ADR 0641 §8.38)：坐瞻组技补牌——「坐瞻」标记角色死亡后
/// （全局 dieAfter），令一名存活的「坐瞻」角色从弃牌堆获得至多 X（射程标记）
/// 张牌名各不相同的基本牌。
pub(crate) fn resolve_jsrgzuozhan_gain(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 权威校验：owner 存活；死亡者记有「坐瞻」标记；存在存活的坐瞻角色；
    // 弃牌堆存在可取的基本牌。
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || event_player == PlayerId::NONE
        || !tx
            .state()
            .players
            .get(event_player)
            .is_some_and(|participant| participant.marks.iter().any(|(m, c)| {
                *m == ZUOZHAN_SIT_MARK && *c > 0
            }))
    {
        return Ok(Vec::new());
    }
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            candidate != event_player
                && tx
                    .state()
                    .players
                    .get(candidate)
                    .is_some_and(|participant| {
                        participant.marks.iter().any(|(m, c)| {
                            *m == ZUOZHAN_SIT_MARK && *c > 0
                        })
                    })
        })
        .map(PlayerId::raw)
        .collect();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let skill_def = skill_pile_def(tx, player, skill)?;
    let quota = tx
        .state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(m, _)| *m == skill_def))
        .map_or(0, |(_, c)| *c)
        .max(0) as usize;
    if quota == 0 || zuozhan_distinct_basics_from_discard(tx, quota).is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveJsrgzuozhanGainStage,
        vec![
            RESOLVE_ZUOZHAN_GAIN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            event_player.raw(),
        ],
    )
}

/// 续体：selected[0] = 获得补牌的存活「坐瞻」角色。
pub(crate) fn resolve_jsrgzuozhan_gain_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    _event_player: PlayerId,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    let quota = tx
        .state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(m, _)| *m == skill_def))
        .map_or(0, |(_, c)| *c)
        .max(0) as usize;
    let cards = zuozhan_distinct_basics_from_discard(tx, quota);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let target = PlayerId::new(*selected.first().ok_or_else(|| {
        EffectError::Illegal("resolve-jsrgzuozhan-gain target required".into())
    })?);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut events = Vec::new();
    for card in cards {
        tx.state_mut()
            .zones
            .get_mut(discard)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = hand;
            card_state.owner = target;
        }
        events.push(engine_api::DomainEvent::CardsDrawn {
            player: target,
            cards: vec![card],
        });
    }
    Ok(events)
}

/// 专属状态机 (批 241, ADR 0641 §8.36)：潜龙——受到伤害后，观看牌堆顶三张牌，
/// 获得至多已损失体力值张牌，其余置于牌堆底。
pub(crate) fn resolve_qianlong(
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
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let cards = tx.state().zones.get(draw).unwrap().cards.clone();
    if cards.len() < 3 {
        return Ok(Vec::new());
    }
    let top3 = [cards[cards.len() - 1], cards[cards.len() - 2], cards[cards.len() - 3]];
    let participant = tx.state().players.get(player).unwrap();
    let lost = (participant.max_hp as i32 - participant.hp).max(0) as usize;
    let max_gain = lost.min(3);
    if max_gain == 0 {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        0,
        max_gain as u8,
        true,
        top3.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveQianlongStage,
        vec![
            RESOLVE_QIANLONG_MAGIC,
            player.raw(),
            skill.raw(),
            top3[0].raw(),
            top3[1].raw(),
            top3[2].raw(),
        ],
    )
}

/// 续体：selected = 获得牌；顶三张中未选的移至牌堆底（向量头部）。
pub(crate) fn resolve_qianlong_stage(
    tx: &mut Transaction,
    player: PlayerId,
    top3: [u32; 3],
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let chosen: std::collections::BTreeSet<u32> = selected.iter().copied().collect();
    let mut events = Vec::new();
    // 获得牌：从牌堆移入手牌。
    for raw in selected {
        let card = CardId::new(*raw);
        tx.state_mut()
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = hand;
            card_state.owner = player;
        }
        events.push(engine_api::DomainEvent::CardsDrawn {
            player,
            cards: vec![card],
        });
    }
    // 未选的顶三张按原次序压入牌堆底（向量头部，先压最顶者使末选最靠底侧）。
    for raw in top3 {
        let card = CardId::new(raw);
        if chosen.contains(&raw) {
            continue;
        }
        if let Some(pos) = tx
            .state()
            .zones
            .get(draw)
            .unwrap()
            .cards
            .iter()
            .position(|c| *c == card)
        {
            let card_id = tx.state().zones.get(draw).unwrap().cards[pos];
            tx.state_mut()
                .zones
                .get_mut(draw)
                .unwrap()
                .cards
                .remove(pos);
            tx.state_mut()
                .zones
                .get_mut(draw)
                .unwrap()
                .cards
                .insert(0, card_id);
        }
    }
    Ok(events)
}

const RESOLVE_JSRGDAOREN_CUSTOM_CHOICE: u32 = 0x4A53_0001;

/// 专属状态机 (批 240, ADR 0641 §8.35)：蹈仁——出牌阶段限一次，将一张手牌交给
/// 一名其他角色，然后对处于你与其攻击范围交集内的所有角色各造成 1 点无来源
/// 伤害。载荷 6 槽：[magic, player, skill, stage, card, target]。
pub(crate) fn resolve_jsrgdaoren(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.phase != crate::state::Phase::Play
        || tx.state().hand_cards(player).is_empty()
        || !tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .any(|candidate| candidate != player)
    {
        return Ok(Vec::new());
    }
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveJsrgdaorenStage,
        vec![RESOLVE_JSRGDAOREN_CUSTOM_CHOICE, player.raw(), skill.raw(), 1, 0, 0],
    )
}

#[allow(clippy::too_many_arguments)]
fn jsrgdaoren_step(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    card: u32,
    target: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let payload = |stage: u32, card: u32, target: u32| {
        vec![
            RESOLVE_JSRGDAOREN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            stage,
            card,
            target,
        ]
    };
    match stage {
        1 => {
            // 手牌已选：选目标（其他存活角色）。
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveJsrgdaorenStage,
                payload(2, card, 0),
            )
        }
        2 => {
            // 目标已选：交牌 + 射程交集伤害。
            let target = PlayerId::new(target);
            let card_id = CardId::new(card);
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
            let target_hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
            if !tx.state().zones.get(hand).unwrap().cards.contains(&card_id) {
                return Err(EffectError::Illegal(
                    "jsrgdaoren card not in hand".into(),
                ));
            }
            tx.state_mut()
                .zones
                .get_mut(hand)
                .unwrap()
                .cards
                .retain(|c| *c != card_id);
            tx.state_mut()
                .zones
                .get_mut(target_hand)
                .unwrap()
                .cards
                .push(card_id);
            if let Some(card_state) = tx.state_mut().cards.get_mut(card_id) {
                card_state.zone = target_hand;
                card_state.owner = target;
            }
            // 射程交集：双方攻击范围均可及的所有存活角色（含自己/目标若在范围内）。
            let victims: Vec<PlayerId> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| {
                    crate::query::player_in_attack_range(tx.state(), player, candidate)
                        && crate::query::player_in_attack_range(tx.state(), target, candidate)
                })
                .collect();
            let mut events = Vec::new();
            for victim in victims {
                events.extend(super::pack_mobile::deal_damage(
                    tx,
                    PlayerId::NONE,
                    victim,
                    1,
                    false,
                    0,
                    false,
                    CardId::NONE,
                    0,
                    0,
                )?);
            }
            Ok(events)
        }
        _ => Ok(Vec::new()),
    }
}

/// 状态机续体入口（resolver 经由 crate::effect 重导出调用）。
pub(crate) fn jsrgdaoren_entry(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    card: u32,
    target: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    jsrgdaoren_step(tx, player, skill, stage, card, target)
}

const RESOLVE_PXQIXIAN_CUSTOM_CHOICE: u32 = 0x5078_0001;

const QUANJIA_MARK: u32 = 0x51A_0001;

/// 专属状态机 (批 237, ADR 0641 §8.32)：拳甲——一名其他角色对另一角色造成伤害后
/// （来源为当前回合角色且有手牌），随机观看来源 (2+标记数) 张手牌：其中有【杀】
/// 则对其使用虚拟【杀】；否则获得 1 枚标记（下次多看一张）。
pub(crate) fn resolve_quanjia(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    source: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 权威校验：来源非自己、目标非来源、来源为当前回合角色、来源存活且有手牌。
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || source == player
        || source == target
        || tx.state().turn.current != source
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive && !tx.state().hand_cards(source).is_empty())
    {
        return Ok(Vec::new());
    }
    let marks = tx
        .state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(m, _)| *m == QUANJIA_MARK))
        .map_or(0, |(_, c)| *c) as u32;
    let num = 2 + marks;
    let mut hand = tx.state().hand_cards(source);
    // RNG 随机取样（ Fisher–Yates 局部 + 取前 num 张）。
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    rng.shuffle(&mut hand);
    let sampled: Vec<engine_api::CardId> = hand.into_iter().take(num as usize).collect();
    tx.state_mut().rng = rng.into_state();
    // 其中有【杀】则对其使用虚拟【杀】；否则获得 1 枚标记。
    let has_sha = sampled.iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|state| state.kind == crate::state::CardKind::Sha)
    });
    if has_sha {
        let card = crate::effect::materialize_virtual_card(tx, player, crate::state::CardKind::Sha)?;
        crate::command::play::play_as_kind(
            tx,
            player,
            card,
            crate::state::CardKind::Sha,
            &[source],
            false,
            false,
            true,
            false,
        )?;
    } else {
        crate::effect::add_mark(tx, player, QUANJIA_MARK, 1)?;
    }
    Ok(Vec::new())
}

/// 变张置入（A0 置入面，ADR 0641 §8.12）：从自身手牌选 [min_count, max_count]

/// 专属状态机 (批 236, ADR 0641 §8.31)：祁县——出牌阶段限一次，重铸一张装备牌
/// （置入弃牌堆）并视为对一名其他角色使用一张【杀】或【过河拆桥】。
/// 载荷 7 槽：[magic, player, skill, stage, kind, material, target]。
pub(crate) fn resolve_pxqixian(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.phase != crate::state::Phase::Play
    {
        return Ok(Vec::new());
    }
    if pxqixian_equip_materials(tx, player).is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ResolvePxqixianStage,
        vec![RESOLVE_PXQIXIAN_CUSTOM_CHOICE, player.raw(), skill.raw(), 1, 0, 0, 0],
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn pxqixian_step(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    kind: u32,
    material: u32,
    target: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let payload = |stage: u32, kind: u32, material: u32, target: u32| {
        vec![
            RESOLVE_PXQIXIAN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            stage,
            kind,
            material,
            target,
        ]
    };
    match stage {
        2 => {
            // kind 已选（1=sha, 2=guohe）：选重铸素材（装备牌）。
            let materials = pxqixian_equip_materials(tx, player);
            if materials.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                materials.into_iter().map(|card| card.raw()).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolvePxqixianStage,
                payload(2, kind, 0, 0),
            )
        }
        3 => {
            // 素材已选（material）：选目标。
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolvePxqixianStage,
                payload(3, kind, material, 0),
            )
        }
        4 => {
            // 目标已选：重铸素材置入弃牌堆 + 物化虚拟牌按规则使用。
            let material = CardId::new(material);
            let target = PlayerId::new(target);
            let card_kind = if kind == 1 {
                crate::state::CardKind::Sha
            } else {
                crate::state::CardKind::TrickGuohe
            };
            if let Some(from) = tx.state().cards.get(material).map(|state| state.zone) {
                let discard = tx
                    .state()
                    .zones
                    .find(crate::state::ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
                tx.state_mut()
                    .zones
                    .get_mut(from)
                    .unwrap()
                    .cards
                    .retain(|c| *c != material);
                tx.state_mut()
                    .zones
                    .get_mut(discard)
                    .unwrap()
                    .cards
                    .push(material);
                if let Some(card_state) = tx.state_mut().cards.get_mut(material) {
                    card_state.zone = discard;
                    card_state.owner = PlayerId::NONE;
                }
            }
            let card = crate::effect::materialize_virtual_card(tx, player, card_kind)?;
            crate::command::play::play_as_kind(
                tx,
                player,
                card,
                card_kind,
                &[target],
                false,
                false,
                true,
                false,
            )?;
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

/// 手牌+装备区中的装备牌（重铸素材）。
pub(crate) fn pxqixian_equip_materials(tx: &Transaction, player: PlayerId) -> Vec<engine_api::CardId> {
    let mut materials = Vec::new();
    for zone_kind in [crate::state::ZoneKind::Hand, crate::state::ZoneKind::Equip] {
        if let Some(zone) = tx.state().zones.find(zone_kind, player) {
            for card in tx.state().zones.get(zone).unwrap().cards.clone() {
                if let Some(state) = tx.state().cards.get(card) {
                    if matches!(
                        state.kind,
                        crate::state::CardKind::Weapon
                            | crate::state::CardKind::Armor
                            | crate::state::CardKind::HorseOff
                            | crate::state::CardKind::HorseDef
                            | crate::state::CardKind::Treasure
                    ) {
                        materials.push(card);
                    }
                }
            }
        }
    }
    materials
}

/// 状态机续体入口（resolver 经由 crate::effect 重导出调用）。
pub(crate) fn resolve_dcbingji_entry(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    kind: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    resolve_dcbingji(tx, player, skill, stage, kind)
}

/// 专属状态机 (批 235, ADR 0641 §8.30)：秉纪——出牌阶段，若所有手牌同花色且
/// 本回合未使用过该花色，可展示所有手牌并视为使用一张【杀】或【桃】（该花色
/// 本回合记为已用）。
pub(crate) fn resolve_dcbingji(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    kind: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.phase != crate::state::Phase::Play
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            // 校验：手牌非空且全部同花色。
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let first = crate::query::effective_card_suit(
                tx.state(),
                player,
                &tx.state().cards.get(hand[0]).expect("card"),
            );
            if hand.iter().any(|card| {
                crate::query::effective_card_suit(
                    tx.state(),
                    player,
                    &tx.state().cards.get(*card).expect("card"),
                ) != first
            }) {
                return Ok(Vec::new());
            }
            // 校验：该花色本回合未使用过。
            let bit = suit_history_bit(first);
            let history = tx
                .state()
                .turn
                .flag(crate::state::turn_flag::used_card_suit_history_by_player(
                    player,
                ));
            if history & bit != 0 {
                return Ok(Vec::new());
            }
            let suit_tag = suit_tag(first);
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveDcbingjiStage,
                vec![
                    RESOLVE_DCBINGJI_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    1,
                    suit_tag,
                ],
            )
        }
        1 => {
            // kind=1 (sha)：选目标；kind=2 (tao)：对自己使用。
            if kind == 2 {
                use_dcbingji_virtual(tx, player, 2, player)?;
                Ok(Vec::new())
            } else {
                let targets: Vec<u32> = tx
                    .state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|&candidate| candidate != player)
                    .map(PlayerId::raw)
                    .collect();
                if targets.is_empty() {
                    return Ok(Vec::new());
                }
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    targets,
                    ChoiceKind::SelectTargets,
                    ContinuationKind::ResolveDcbingjiStage,
                    vec![
                        RESOLVE_DCBINGJI_CUSTOM_CHOICE,
                        player.raw(),
                        skill.raw(),
                        2,
                        1,
                    ],
                )
            }
        }
        2 => {
            // kind=1 (sha)：对所选目标使用。
            use_dcbingji_virtual(tx, player, 1, PlayerId::new(kind))?;
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

fn suit_history_bit(suit: engine_api::Suit) -> i32 {
    match suit {
        engine_api::Suit::Spade => 1,
        engine_api::Suit::Heart => 1 << 1,
        engine_api::Suit::Club => 1 << 2,
        engine_api::Suit::Diamond => 1 << 3,
        engine_api::Suit::None => 0,
    }
}

fn suit_tag(suit: engine_api::Suit) -> u32 {
    match suit {
        engine_api::Suit::Spade => 1,
        engine_api::Suit::Heart => 2,
        engine_api::Suit::Club => 3,
        engine_api::Suit::Diamond => 4,
        engine_api::Suit::None => 0,
    }
}

fn suit_from_tag(tag: u32) -> engine_api::Suit {
    match tag {
        1 => engine_api::Suit::Spade,
        2 => engine_api::Suit::Heart,
        3 => engine_api::Suit::Club,
        4 => engine_api::Suit::Diamond,
        _ => engine_api::Suit::None,
    }
}

fn use_dcbingji_virtual(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u32,
    target: PlayerId,
) -> Result<(), EffectError> {
    // kind: 1 = sha, 2 = tao（物化虚拟牌并按规则使用）。
    let card_kind = if kind == 1 {
        crate::state::CardKind::Sha
    } else {
        crate::state::CardKind::Tao
    };
    let card = crate::effect::materialize_virtual_card(tx, player, card_kind)?;
    crate::command::play::play_as_kind(
        tx,
        player,
        card,
        card_kind,
        &[target],
        false,
        false,
        true,
        false,
    )?;
    // 记录该花色本回合已用（批 230 入册缺口的面内实现）：虚拟牌无花色，
    // 取持有者当前统一手牌花色（虚拟使用不消耗手牌，同花色不变）。
    let hand = tx.state().hand_cards(player);
    if let Some(first) = hand.first() {
        let suit = crate::query::effective_card_suit(
            tx.state(),
            player,
            &tx.state().cards.get(*first).expect("card"),
        );
        if hand.iter().all(|card| {
            crate::query::effective_card_suit(
                tx.state(),
                player,
                &tx.state().cards.get(*card).expect("card"),
            ) == suit
        } && suit != engine_api::Suit::None)
        {
            tx.state_mut().turn.remember_used_card_suit(player, suit);
        }
    }
    Ok(())
}

/// 变张置入（A0 置入面，ADR 0641 §8.12）：从自身手牌选 [min_count, max_count]
/// 张置入自身技能堆（允许取消=不置入）。上限以当前手牌数钳位；max==0 或空手
/// no-op。
pub(crate) fn choose_hand_cards_to_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    min_count: u32,
    max_count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill_pile_def(tx, player, skill)?;
    if max_count == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let hand_cards: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    if hand_cards.is_empty() {
        return Ok(Vec::new());
    }
    let n = hand_cards.len() as u32;
    let lo = min_count.min(n) as u8;
    let hi = max_count.min(n) as u8;
    open_choice(
        tx,
        player,
        lo,
        hi,
        true,
        hand_cards,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseHandCardsToSkillPile,
        vec![player.raw(), skill.raw(), min_count, max_count],
    )
}

pub(crate) fn rearrange_pile_to_deck_top(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill_pile_def(tx, player, skill)?;
    if count == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, pile_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let n = (count as usize).min(cards.len()) as u8;
    open_choice(
        tx,
        player,
        n,
        n,
        true,
        cards.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::RearrangePileToDeckTop,
        vec![player.raw(), skill.raw(), pile_def, count],
    )
}

pub(crate) fn discard_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    pile_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    // Gate on the originating skill being live (provenance); the pile itself is
    // addressed by the `pile_def` literal so sibling piles (e.g. a parent skill's
    // marker pile) can be discarded.
    let _ = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, pile_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let pile_zone = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == player && pile.skill_def == pile_def)
        .map(|pile| pile.zone)
        .ok_or_else(|| EffectError::Illegal("skill pile zone missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    move_cards_with_reason(tx, cards, discard, Some(pile_zone), CardMoveReason::Discard)
}

pub(crate) fn gain_from_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().skill_pile_cards(player, skill_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        cards.iter().map(|card| card.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::GainFromSkillPile,
        vec![player.raw(), skill.raw()],
    )
}

/// §7 技能夺取设施 S0（ABI 675，第一百四十三批）：选择一名其他存活角色，
/// 再从其可获得技能名单中获得一个。名单=目标存活技能 def 元数据
/// `locked==false && non_gainable==false`，按 def raw 升序去重。
/// 分段共用 Cont 283：payload [owner, skill, stage(0|1), (stage1: target, defs...)]。
pub(crate) fn choose_gain_other_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mut targets: Vec<u32> = Vec::new();
    for candidate in tx.state().players.alive_by_seat() {
        if candidate == player {
            continue;
        }
        if !gainable_skill_defs(tx, candidate).is_empty() {
            targets.push(candidate.raw());
        }
    }
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::Custom,
        ContinuationKind::ChooseGainOtherSkill,
        vec![player.raw(), skill.raw(), 0],
    )
}

/// 目标可获得技能 def 名单（升序、去重、过滤 locked/non_gainable）。
fn gainable_skill_defs(tx: &Transaction, target: PlayerId) -> Vec<u32> {
    use std::collections::BTreeSet;
    let mut defs: BTreeSet<u32> = BTreeSet::new();
    for instance in tx.state().skills.sorted_live() {
        if instance.owner != target || !instance.enabled {
            continue;
        }
        let Some(meta) = tx.state().content.skill_metadata(instance.def) else {
            continue;
        };
        if meta.locked || meta.non_gainable {
            continue;
        }
        defs.insert(instance.def.raw());
    }
    defs.into_iter().collect()
}

/// Cont 283 resolver：stage0 选目标 → stage1 名单选技并 AddSkill。
pub(crate) fn resolve_choose_gain_other_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 {
        return Err(EffectError::EngineFault("bad gain-skill payload".into()));
    }
    let (owner_raw, skill_raw, stage) = (payload[0], payload[1], payload[2]);
    let player = PlayerId::new(owner_raw);
    let skill = engine_api::SkillId::new(skill_raw);
    if observer != player {
        return Err(EffectError::Illegal("gain-skill chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    if stage == 0 {
        let Some(&target_raw) = selections.first() else {
            return Ok(());
        };
        let target = PlayerId::new(target_raw);
        let defs = gainable_skill_defs(tx, target);
        if defs.is_empty() {
            return Ok(());
        }
        let mut payload: Vec<u32> = vec![owner_raw, skill_raw, 1, target_raw];
        payload.extend_from_slice(&defs);
        open_choice(
            tx,
            player,
            1,
            1,
            true,
            defs,
            ChoiceKind::Custom,
            ContinuationKind::ChooseGainOtherSkill,
            payload,
        )?;
        return Ok(());
    }
    let Some(&target_raw) = payload.get(3) else {
        return Err(EffectError::EngineFault("bad gain-skill stage1".into()));
    };
    let defs = &payload[4..];
    let Some(&index) = selections.first() else {
        return Ok(());
    };
    let Some(def) = defs.get(index as usize).copied() else {
        return Ok(());
    };
    let _ = target_raw;
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: engine_api::SkillDefId::new(def),
        }],
    )?;
    tx.state_mut()
        .facility_gained_skills
        .entry(owner_raw)
        .or_default()
        .push(def);
    Ok(())
}

/// §7 技能夺取设施 S1（ABI 676）：从本设施获得清单中选择一个技能移除
/// （RemoveSkill+出清登记）。清单空则静默。Cont 284，payload [player, skill, defs...]。
pub(crate) fn lose_gained_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let Some(defs) = tx.state().facility_gained_skills.get(&player.raw()) else {
        return Ok(Vec::new());
    };
    if defs.is_empty() {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = defs.clone();
    let mut payload = vec![player.raw(), skill.raw()];
    payload.extend(options.iter().copied());
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::LoseGainedSkill,
        payload,
    )
}

/// Cont 284 resolver：移除所选已获得技能并出清登记。
pub(crate) fn resolve_lose_gained_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 2 {
        return Err(EffectError::EngineFault("bad lose-skill payload".into()));
    }
    let (owner_raw, skill_raw) = (payload[0], payload[1]);
    let player = PlayerId::new(owner_raw);
    let skill = engine_api::SkillId::new(skill_raw);
    if observer != player {
        return Err(EffectError::Illegal("lose-skill chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let defs = &payload[2..];
    let Some(&index) = selections.first() else {
        return Ok(());
    };
    let Some(def) = defs.get(index as usize).copied() else {
        return Ok(());
    };
    // 找到该 def 的存活实例（可能被移除/刷新过，找不到则仅出清登记）。
    let instance_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.owner == player && s.def.raw() == def)
        .map(|s| s.id);
    if let Some(id) = instance_id {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill: id }])?;
    }
    if let Some(list) = tx.state_mut().facility_gained_skills.get_mut(&owner_raw) {
        list.retain(|d| *d != def);
        if list.is_empty() {
            tx.state_mut().facility_gained_skills.remove(&owner_raw);
        }
    }
    let _ = skill;
    Ok(())
}

/// §7 技能夺取设施 S2（ABI 677）：合韵延伸。三段链（共用 Cont 285，stage 位区分）：
/// 段0 选一名共享「可获得技能」的其他存活角色 → 段1 选一个共同技能 def →
/// 段2 持有者失去该技能+目标摸 2。
/// 可共享判定：!locked && !non_gainable && !lord_only（上游 charlotte 排除在
/// 元数据中无对应标志，以此近似，已注明）。
fn eligible_shared_defs(tx: &Transaction, owner: PlayerId) -> Vec<u32> {
    use std::collections::BTreeSet;
    let mut defs: BTreeSet<u32> = BTreeSet::new();
    for instance in tx.state().skills.sorted_live() {
        if instance.owner != owner || !instance.enabled {
            continue;
        }
        let Some(meta) = tx.state().content.skill_metadata(instance.def) else {
            continue;
        };
        if meta.locked || meta.non_gainable || meta.lord_only {
            continue;
        }
        defs.insert(instance.def.raw());
    }
    defs.into_iter().collect()
}

pub(crate) fn resolve_potheyun(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let mine = eligible_shared_defs(tx, player);
    if mine.is_empty() {
        return Ok(Vec::new());
    }
    let mine_set: std::collections::BTreeSet<u32> = mine.iter().copied().collect();
    let mut targets: Vec<u32> = Vec::new();
    for candidate in tx.state().players.alive_by_seat() {
        if candidate == player {
            continue;
        }
        let theirs: std::collections::BTreeSet<u32> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .filter(|s| s.owner == candidate && s.enabled)
            .map(|s| s.def.raw())
            .collect();
        if !mine_set.intersection(&theirs).next().is_none() {
            targets.push(candidate.raw());
        }
    }
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::Custom,
        ContinuationKind::ResolvePotheyun,
        vec![player.raw(), skill.raw(), 0],
    )
}

/// Cont 285 resolver：stage1 选共同技能；stage2 失去技能+目标摸 2。
pub(crate) fn resolve_potheyun_pick(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 3 {
        return Err(EffectError::EngineFault("bad potheyun payload".into()));
    }
    let (owner_raw, skill_raw, stage) = (payload[0], payload[1], payload[2]);
    let player = PlayerId::new(owner_raw);
    let skill = engine_api::SkillId::new(skill_raw);
    if observer != player {
        return Err(EffectError::Illegal("potheyun chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    if stage == 0 {
        let Some(&target_raw) = selections.first() else {
            return Ok(());
        };
        let target = PlayerId::new(target_raw);
        let mine: std::collections::BTreeSet<u32> = eligible_shared_defs(tx, player).into_iter().collect();
        let theirs: std::collections::BTreeSet<u32> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .filter(|s| s.owner == target && s.enabled)
            .map(|s| s.def.raw())
            .collect();
        let shared: Vec<u32> = mine.intersection(&theirs).copied().collect();
        if shared.is_empty() {
            return Ok(());
        }
        let mut payload: Vec<u32> = vec![owner_raw, skill_raw, 1, target_raw];
        payload.extend_from_slice(&shared);
        open_choice(
            tx,
            player,
            1,
            1,
            true,
            shared,
            ChoiceKind::Custom,
            ContinuationKind::ResolvePotheyun,
            payload,
        )?;
        return Ok(());
    }
    let Some(&target_raw) = payload.get(3) else {
        return Err(EffectError::EngineFault("bad potheyun stage1".into()));
    };
    let defs = &payload[4..];
    let Some(&index) = selections.first() else {
        return Ok(());
    };
    let Some(def) = defs.get(index as usize).copied() else {
        return Ok(());
    };
    let instance_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.owner == player && s.def.raw() == def)
        .map(|s| s.id);
    if let Some(id) = instance_id {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill: id }])?;
    }
    run_effects(
        tx,
        vec![EffectRequest::DrawCards {
            player: PlayerId::new(target_raw),
            count: 2,
        }],
    )?;
    Ok(())
}

const RESOLVE_TWMOUZHI_MAGIC: u32 = 0x4D4F_0001;

const TWMOUZHI_COLOR_KEY: u32 = 1;

/// 事件牌颜色（上游 get.color 口径）：1=红（♥♦），2=黑（♠♣），None=无牌/无色。
fn twmouzhi_card_color(tx: &Transaction, raw: u32) -> Option<u32> {
    if raw == 0 {
        return None;
    }
    tx.state().cards.get(CardId::new(raw)).and_then(|card| match card.suit {
        engine_api::Suit::Heart | engine_api::Suit::Diamond => Some(1),
        engine_api::Suit::Spade | engine_api::Suit::Club => Some(2),
        _ => None,
    })
}

/// 专属状态机 (批 245, ADR 0641 §8.39)：谋制——锁定技。mode 0（damage_before）：
/// 事件牌有颜色且与记录的上次受伤牌颜色相同时防止本次伤害（防止不更新记录）；
/// mode 1（damage）：记录事件牌颜色（无牌/无色清除记录）。
pub(crate) fn resolve_twmouzhi(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    event_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match mode {
        0 => {
            let Some(color) = twmouzhi_card_color(tx, event_card) else {
                return Ok(Vec::new());
            };
            let last = tx.state().skills.storage_get(skill, TWMOUZHI_COLOR_KEY);
            if last != 0 && last == color {
                let damage_id = tx.state().resolution.current_damage_id;
                if damage_id != 0
                    && !tx
                        .state()
                        .resolution
                        .prevented_damage_ids
                        .contains(&damage_id)
                {
                    tx.state_mut()
                        .resolution
                        .prevented_damage_ids
                        .push(damage_id);
                }
            }
        }
        1 => {
            let color = twmouzhi_card_color(tx, event_card).unwrap_or(0);
            if !tx.state_mut().skills.storage_set(skill, TWMOUZHI_COLOR_KEY, color) {
                return Err(EffectError::Illegal("twmouzhi: bad skill".into()));
            }
        }
        _ => {}
    }
    Ok(Vec::new())
}

const RESOLVE_TWDAIGUI_MAGIC: u32 = 0x5444_0001;

/// stage 3 轮次载荷：[magic, player, skill, 3, k, m, targets(k), revealed(m),
/// picks(2×r)]——picks 按 (chooser_raw, card_raw) 记录，card=0 表示该轮跳过
/// （目标死亡或余牌为空）。
fn twdaigui_round_payload(
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: &[u32],
    revealed: &[u32],
    picks: &[(u32, u32)],
) -> Vec<u32> {
    let mut payload = vec![
        RESOLVE_TWDAIGUI_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        targets.len() as u32,
        revealed.len() as u32,
    ];
    payload.extend_from_slice(targets);
    payload.extend_from_slice(revealed);
    for (chooser, card) in picks {
        payload.push(*chooser);
        payload.push(*card);
    }
    payload
}

/// 专属状态机 (批 247/248, ADR 0641 §8.40)：待归——出牌阶段结束时，若你的
/// 手牌颜色均相同，选择至多 X（手牌数）名角色，亮出牌堆底等量的牌，这些角色
/// 按座次依次选获其中一张（slice 2）。
pub(crate) fn resolve_twdaigui(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            // 手牌同色门控（机内权威校验，上游 filter：所有手牌 get.color 一致）。
            let mut red = 0usize;
            let mut black = 0usize;
            for card in &hand {
                if let Some(card_state) = tx.state().cards.get(*card) {
                    match card_state.suit {
                        engine_api::Suit::Heart | engine_api::Suit::Diamond => red += 1,
                        engine_api::Suit::Spade | engine_api::Suit::Club => black += 1,
                        _ => {}
                    }
                }
            }
            if red > 0 && black > 0 {
                return Ok(Vec::new());
            }
            if red == 0 && black == 0 {
                return Ok(Vec::new());
            }
            // 选 [1, 手牌数] 名任意存活角色（可含自己，可取消）。
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            let max_n = hand.len().min(u8::MAX as usize) as u8;
            open_choice(
                tx,
                player,
                1,
                max_n,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwdaiguiStage,
                vec![RESOLVE_TWDAIGUI_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 目标选择已确认（resolver 经由 crate::effect 重导出调用）：
/// selected = 「待归」目标（1..=手牌数）。亮出等量牌堆底牌（公开）并按座次
/// 打开第一轮选获。
pub(crate) fn resolve_twdaigui_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    // 目标按座次排序（上游 sortBySeat）。
    let seat: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    let mut targets: Vec<u32> = selected.to_vec();
    targets.sort_by_key(|raw| seat.iter().position(|s| s == raw).unwrap_or(usize::MAX));

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let cards = tx.state().zones.get(draw).unwrap().cards.clone();
    let n = targets.len().min(cards.len());
    if n == 0 {
        return Ok(Vec::new());
    }
    let revealed: Vec<u32> = cards[..n].iter().map(|card| card.raw()).collect();
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::RevealCards {
            cards: revealed.iter().map(|raw| CardId::new(*raw)).collect(),
            to: Vec::new(),
        }],
    )?;
    open_twdaigui_round(tx, player, skill, targets, revealed, Vec::new())?;
    Ok(Vec::new())
}

/// 打开第 r 轮选获（r = picks.len()）。已死亡目标与余牌为空的轮次按上游
/// `continue` 语义跳过；全部轮次完成后余牌归弃牌堆。
fn open_twdaigui_round(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<u32>,
    revealed: Vec<u32>,
    mut picks: Vec<(u32, u32)>,
) -> Result<(), EffectError> {
    let k = targets.len();
    loop {
        let r = picks.len();
        if r >= k {
            return twdaigui_finish(tx, revealed, &picks);
        }
        let chooser_raw = targets[r];
        let chooser = PlayerId::new(chooser_raw);
        if !tx
            .state()
            .players
            .get(chooser)
            .is_some_and(|participant| participant.alive)
        {
            picks.push((chooser_raw, 0));
            continue;
        }
        let picked: std::collections::BTreeSet<u32> =
            picks.iter().map(|(_, card)| *card).collect();
        let remaining: Vec<u32> = revealed
            .iter()
            .copied()
            .filter(|card| !picked.contains(card))
            .collect();
        if remaining.is_empty() {
            picks.push((chooser_raw, 0));
            continue;
        }
        let payload = twdaigui_round_payload(player, skill, &targets, &revealed, &picks);
        open_choice(
            tx,
            chooser,
            1,
            1,
            false,
            remaining,
            ChoiceKind::SelectCards,
            ContinuationKind::ResolveTwdaiguiStage,
            payload,
        )?;
        return Ok(());
    }
}

/// 轮次选获已确认（resolver 调用）：observer = 当前选获目标，selections[0] =
/// 选获牌。入手后开下一轮（或终局）。
pub(crate) fn resolve_twdaigui_round(
    tx: &mut Transaction,
    observer: PlayerId,
    player: PlayerId,
    skill: engine_api::SkillId,
    payload: &[u32],
    selections: &[u32],
) -> Result<(), EffectError> {
    if payload.len() < 7 || payload[0] != RESOLVE_TWDAIGUI_MAGIC || payload[3] != 3 {
        return Err(EffectError::EngineFault(
            "bad resolve-twdaigui round payload".into(),
        ));
    }
    let k = payload[4] as usize;
    let m = payload[5] as usize;
    if payload.len() < 6 + k + m || (payload.len() - 6 - k - m) % 2 != 0 {
        return Err(EffectError::EngineFault(
            "bad resolve-twdaigui round payload layout".into(),
        ));
    }
    let targets: Vec<u32> = payload[6..6 + k].to_vec();
    let revealed: Vec<u32> = payload[6 + k..6 + k + m].to_vec();
    let picks_rest = &payload[6 + k + m..];
    let mut picks: Vec<(u32, u32)> = picks_rest
        .chunks_exact(2)
        .map(|pair| (pair[0], pair[1]))
        .collect();
    let r = picks.len();
    let chooser_raw = targets.get(r).copied().ok_or_else(|| {
        EffectError::EngineFault("resolve-twdaigui round index out of range".into())
    })?;
    let chooser = PlayerId::new(chooser_raw);
    if observer != chooser {
        return Err(EffectError::Illegal(
            "resolve-twdaigui round chooser mismatch".into(),
        ));
    }
    let picked: std::collections::BTreeSet<u32> = picks.iter().map(|(_, card)| *card).collect();
    let card_raw = *selections.first().ok_or_else(|| {
        EffectError::Illegal("resolve-twdaigui pick required".into())
    })?;
    let revealed_set: std::collections::BTreeSet<u32> = revealed.iter().copied().collect();
    if !revealed_set.contains(&card_raw)
        || picked.contains(&card_raw)
        || !tx.state().players.get(chooser).is_some_and(|p| p.alive)
    {
        return Err(EffectError::Illegal(
            "resolve-twdaigui pick outside revealed pool".into(),
        ));
    }
    // 选获入手：牌堆 → 选获者手牌。
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, chooser)
        .ok_or_else(|| EffectError::Illegal("no chooser hand".into()))?;
    let card = CardId::new(card_raw);
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = hand;
        card_state.owner = chooser;
    }
    picks.push((chooser_raw, card_raw));
    open_twdaigui_round(tx, player, skill, targets, revealed, picks)?;
    Ok(())
}

/// 终局：余牌（亮出未选获）归弃牌堆。
fn twdaigui_finish(tx: &mut Transaction, revealed: Vec<u32>, picks: &[(u32, u32)]) -> Result<(), EffectError> {
    let picked: std::collections::BTreeSet<u32> = picks.iter().map(|(_, card)| *card).collect();
    let leftover: Vec<CardId> = revealed
        .iter()
        .copied()
        .filter(|card| !picked.contains(card))
        .map(CardId::new)
        .collect();
    if leftover.is_empty() {
        return Ok(());
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    crate::effect::move_cards(tx, leftover, discard, None)?;
    Ok(())
}

const RESOLVE_TWMITU_MAGIC: u32 = 0x544D_0001;

/// 「迷途」gaintag（content_card_tags，对应上游 gaintag "twmitu"）。
pub(crate) const TWMITU_TAG: u32 = 3_195_907_057;

/// 专属状态机 (批 249, ADR 0641 §8.41 slice 1)：迷途——准备阶段，你选择至多
/// 3 名角色，这些角色各摸 2 张牌、摸得牌打上「迷途」标记并公开亮出
/// （slice 2 将接续拼点轮与崩溃面）。
pub(crate) fn resolve_twmitu(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let _ = count;
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                3,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveTwmituStage,
                vec![RESOLVE_TWMITU_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 目标选择已确认（resolver 调用）：各目标按座次摸 2 张、打「迷途」标记并
/// 公开亮出，然后进入拼点轮推进（slice 2）。
pub(crate) fn resolve_twmitu_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    let seat: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    let mut targets: Vec<u32> = selected.to_vec();
    targets.sort_by_key(|raw| seat.iter().position(|s| s == raw).unwrap_or(usize::MAX));

    let mut events = Vec::new();
    let mut shown: Vec<CardId> = Vec::new();
    let mut shown_by_target: Vec<u32> = Vec::new();
    for raw in targets.clone() {
        let target = PlayerId::new(raw);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            shown_by_target.push(0);
            continue;
        }
        let drawn = crate::effect::draw_cards(tx, target, 2)?;
        let mut first = 0u32;
        for event in &drawn {
            if let engine_api::DomainEvent::CardsDrawn { cards, .. } = event {
                for card in cards {
                    tx.state_mut().add_content_card_tag(*card, TWMITU_TAG);
                    shown.push(*card);
                    if first == 0 {
                        first = card.raw();
                    }
                }
            }
        }
        shown_by_target.push(first);
        events.extend(drawn);
    }
    if !shown.is_empty() {
        crate::effect::run_effects(
            tx,
            vec![EffectRequest::RevealCards {
                cards: shown,
                to: Vec::new(),
            }],
        )?;
    }
    // slice 2：拼点轮推进（第 0 轮；upstream compare 轮按座次逐目标）。
    twmitu_advance(
        tx,
        player,
        skill,
        &targets,
        &shown_by_target,
        0,
        &[],
        &[],
        &[],
    )?;
    Ok(events)
}

/// 拼点轮载荷：[magic, player, skill, stage, k, done, targets(k), opponents(d),
/// consents(d), pend]——stage 3=owner 选对手；4=目标知情（Custom 1/2）；
/// 5=目标选拼点牌；6=对手选拼点牌。pend 为 5→6 之间暂存的目标牌。
fn twmitu_round_payload(
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    targets: &[u32],
    shown: &[u32],
    done: usize,
    opponents: &[u32],
    consents: &[u32],
    pend: &[u32],
) -> Vec<u32> {
    let mut payload = vec![
        RESOLVE_TWMITU_MAGIC,
        player.raw(),
        skill.raw(),
        stage,
        targets.len() as u32,
        done as u32,
    ];
    payload.extend_from_slice(targets);
    payload.extend_from_slice(shown);
    payload.extend_from_slice(opponents);
    payload.extend_from_slice(consents);
    payload.extend_from_slice(pend);
    payload
}

/// 轮次推进：跳过死亡/无手牌/无合法对手的轮次；done == k 时收口（slice 3
/// 崩溃面将在此接续）。
fn twmitu_advance(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: &[u32],
    shown: &[u32],
    done: usize,
    opponents: &[u32],
    consents: &[u32],
    pend: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let k = targets.len();
    if done >= k {
        return Ok(Vec::new());
    }
    let target_raw = targets[done];
    let target = PlayerId::new(target_raw);
    // 上游 !isIn / canCompare 不可满足 → 跳过该轮。
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != target)
        .filter(|&candidate| !tx.state().hand_cards(candidate).is_empty())
        .map(PlayerId::raw)
        .collect();
    if !target_alive || tx.state().hand_cards(target).is_empty() || candidates.is_empty() {
        return twmitu_advance(
            tx,
            player,
            skill,
            targets,
            shown,
            done + 1,
            opponents,
            consents,
            pend,
        );
    }
    // stage 3：owner 为该目标指定拼点对手（上游 forced chooseTarget）。
    let payload =
        twmitu_round_payload(player, skill, 3, targets, shown, done, opponents, consents, pend);
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveTwmituStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// 拼点轮各阶段确认（resolver 调用）。返回事件（虚拟杀等）。
pub(crate) fn resolve_twmitu_round(
    tx: &mut Transaction,
    observer: PlayerId,
    player: PlayerId,
    skill: engine_api::SkillId,
    payload: &[u32],
    selections: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    const HEADER: usize = 6;
    if payload.len() < HEADER || payload[0] != RESOLVE_TWMITU_MAGIC {
        return Err(EffectError::EngineFault(
            "bad resolve-twmitu round payload".into(),
        ));
    }
    let stage = payload[3];
    let k = payload[4] as usize;
    let done = payload[5] as usize;
    if k == 0 || done >= k {
        return Err(EffectError::EngineFault(
            "resolve-twmitu round index out of range".into(),
        ));
    }
    let targets: Vec<u32> = payload[HEADER..HEADER + k].to_vec();
    let shown: Vec<u32> = payload[HEADER + k..HEADER + 2 * k].to_vec();
    let consents_off = HEADER + 2 * k;
    let d_opponents = match stage {
        3 => done,
        4..=6 => done + 1,
        _ => return Err(EffectError::EngineFault("bad twmitu stage".into())),
    };
    let d_consents = match stage {
        3..=4 => done,
        5..=6 => done + 1,
        _ => return Err(EffectError::EngineFault("bad twmitu stage".into())),
    };
    let opponents_end = consents_off + d_opponents;
    let consents_end = opponents_end + d_consents;
    if payload.len() < consents_end {
        return Err(EffectError::EngineFault(
            "bad resolve-twmitu round payload layout".into(),
        ));
    }
    let opponents: Vec<u32> = payload[consents_off..opponents_end].to_vec();
    let consents: Vec<u32> = payload[opponents_end..consents_end].to_vec();
    let pend: Vec<u32> = payload[consents_end..].to_vec();
    let _ = &shown;
    let target_raw = targets[done];
    let target = PlayerId::new(target_raw);

    match stage {
        3 => {
            // owner 指定对手：须存活、有手牌、非该目标。
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-twmitu opponent pick requires owner".into(),
                ));
            }
            let opponent_raw = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-twmitu opponent required".into())
            })?;
            let opponent = PlayerId::new(opponent_raw);
            if opponent == target
                || !tx
                    .state()
                    .players
                    .get(opponent)
                    .is_some_and(|participant| {
                        participant.alive && !tx.state().hand_cards(opponent).is_empty()
                    })
            {
                return Err(EffectError::Illegal(
                    "resolve-twmitu invalid opponent".into(),
                ));
            }
            let mut opponents2 = opponents;
            opponents2.push(opponent_raw);
            // stage 4：目标知情（1=同意 2=拒绝）。
            let payload = twmitu_round_payload(
                player,
                skill,
                4,
                &targets,
                &shown,
                done,
                &opponents2,
                &consents,
                &[],
            );
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwmituStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        4 => {
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-twmitu consent requires the target".into(),
                ));
            }
            let consent = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-twmitu consent required".into())
            })?;
            let mut consents2 = consents;
            consents2.push(consent);
            if consent != 1 {
                // 拒绝拼点：轮次完成。
                return twmitu_advance(
                    tx,
                    player,
                    skill,
                    &targets,
                    &shown,
                    done + 1,
                    &opponents,
                    &consents2,
                    &[],
                );
            }
            // stage 5：目标选拼点牌。
            let hand: Vec<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            if hand.is_empty() {
                return Err(EffectError::Illegal(
                    "resolve-twmitu target has no hand card".into(),
                ));
            }
            let payload = twmitu_round_payload(
                player,
                skill,
                5,
                &targets,
                &shown,
                done,
                &opponents,
                &consents2,
                &[],
            );
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                hand,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwmituStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        5 => {
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-twmitu compare card requires the target".into(),
                ));
            }
            let card_raw = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-twmitu compare card required".into())
            })?;
            let target_hand = tx.state().hand_cards(target);
            if !target_hand.iter().any(|card| card.raw() == card_raw) {
                return Err(EffectError::Illegal(
                    "resolve-twmitu compare card outside target hand".into(),
                ));
            }
            let payload = twmitu_round_payload(
                player,
                skill,
                6,
                &targets,
                &shown,
                done,
                &opponents,
                &consents,
                &[card_raw],
            );
            let opponent = PlayerId::new(opponents[done]);
            let ohand: Vec<u32> = tx
                .state()
                .hand_cards(opponent)
                .iter()
                .map(|card| card.raw())
                .collect();
            open_choice(
                tx,
                opponent,
                1,
                1,
                false,
                ohand,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveTwmituStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        6 => {
            if observer != PlayerId::new(opponents[done]) {
                return Err(EffectError::Illegal(
                    "resolve-twmitu compare card requires the opponent".into(),
                ));
            }
            if pend.len() != 1 {
                return Err(EffectError::EngineFault(
                    "resolve-twmitu missing target compare card".into(),
                ));
            }
            let t_card = CardId::new(pend[0]);
            let o_card = CardId::new(*selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-twmitu compare card required".into())
            })?);
            let opponent = PlayerId::new(opponents[done]);
            let opponent_hand = tx.state().hand_cards(opponent);
            if !opponent_hand.iter().any(|card| *card == o_card) {
                return Err(EffectError::Illegal(
                    "resolve-twmitu compare card outside opponent hand".into(),
                ));
            }
            let t_rank = tx.state().cards.get(t_card).map(|c| c.rank).unwrap_or(0);
            let o_rank = tx.state().cards.get(o_card).map(|c| c.rank).unwrap_or(0);
            let mut events: Vec<DomainEvent> = Vec::new();
            if t_rank != o_rank {
                let (winner, loser) = if t_rank > o_rank {
                    (target, opponent)
                } else {
                    (opponent, target)
                };
                // 赢家视为对输家使用虚拟【杀】：机内直调伤害结算（登记债：目标
                // 无应闪机会——可暂停的用牌管线无法在机内中段拼接）。
                events.extend(super::pack_mobile::deal_damage(
                    tx,
                    winner,
                    loser,
                    1,
                    false,
                    0,
                    false,
                    CardId::NONE,
                    0,
                    0,
                )?);
            }
            // slice 3 崩溃面：每个参与拼点的「迷途」目标，若其拼点用牌不是自己
            // 的展示牌（首张摸得牌），owner 减 1 点体力上限。
            let mut penalty = 0i32;
            for (participant_raw, used_card) in [(target_raw, t_card), (opponents[done], o_card)] {
                if let Some(idx) = targets.iter().position(|t| *t == participant_raw) {
                    let showed = shown.get(idx).copied().unwrap_or(0);
                    if showed != 0 && showed != used_card.raw() {
                        penalty += 1;
                    }
                }
            }
            if penalty > 0 {
                events.extend(super::change_max_hp(tx, player, -penalty)?);
            }
            twmitu_advance(
                tx,
                player,
                skill,
                &targets,
                &shown,
                done + 1,
                &opponents,
                &consents,
                &[],
            )?;
            Ok(events)
        }
        _ => Err(EffectError::EngineFault("bad twmitu stage".into())),
    }
}

const RESOLVE_QIANTUN_MAGIC: u32 = 0x514E_0001;

/// 专属状态机 (批 252, ADR 0641 §8.42)：谦吞——出牌阶段限一次，令一名其他
/// 角色展示任意张手牌（其拼点只能用展示牌），与其拼点：你赢则获得其展示的
/// 手牌；没赢（含平点）则获得其其余手牌。
pub(crate) fn resolve_qiantun(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player)
        .filter(|&candidate| !tx.state().hand_cards(candidate).is_empty())
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveQiantunStage,
        vec![RESOLVE_QIANTUN_MAGIC, player.raw(), skill.raw(), 0],
    )
}

/// stage 2：目标展示任意张手牌（强制，≥1）。
pub(crate) fn resolve_qiantun_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    open_choice(
        tx,
        target,
        1,
        hand.len().min(u8::MAX as usize) as u8,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveQiantunStage,
        vec![RESOLVE_QIANTUN_MAGIC, player.raw(), skill.raw(), 2, target_raw],
    )
}

/// stage 3：展示已定 → owner 选自己的拼点牌（不受限）。
pub(crate) fn resolve_qiantun_shown(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    shown: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    let mut payload = vec![
        RESOLVE_QIANTUN_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        target_raw,
        shown.len() as u32,
    ];
    payload.extend_from_slice(shown);
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveQiantunStage,
        payload,
    )
}

/// stage 4：owner 拼点牌已定 → 目选拼点牌（手中有展示牌时只能从展示牌选）。
pub(crate) fn resolve_qiantun_player_card(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    shown: &[u32],
    player_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let target_hand: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    let shown_set: std::collections::BTreeSet<u32> = shown.iter().copied().collect();
    let has_shown_in_hand = target_hand.iter().any(|card| shown_set.contains(card));
    let options: Vec<u32> = if has_shown_in_hand {
        target_hand
            .iter()
            .copied()
            .filter(|card| shown_set.contains(card))
            .collect()
    } else {
        target_hand
    };
    let mut payload = vec![
        RESOLVE_QIANTUN_MAGIC,
        player.raw(),
        skill.raw(),
        4,
        target_raw,
        shown.len() as u32,
    ];
    payload.extend_from_slice(shown);
    payload.push(player_card);
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveQiantunStage,
        payload,
    )
}

pub(crate) fn resolve_qiantun_resolve(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    shown: &[u32],
    player_card: u32,
    target_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let target = PlayerId::new(target_raw);
    let p_rank = tx
        .state()
        .cards
        .get(CardId::new(player_card))
        .map(|c| c.rank)
        .unwrap_or(0);
    let t_rank = tx
        .state()
        .cards
        .get(CardId::new(target_card))
        .map(|c| c.rank)
        .unwrap_or(0);
    let player_won = p_rank > t_rank;

    // 拼点牌置入弃牌堆（chooseToCompare 规则）。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    for raw in [player_card, target_card] {
        let card = CardId::new(raw);
        if let Some(hand_zone) = {
            let holder = tx
                .state()
                .cards
                .get(card)
                .map(|card_state| card_state.owner)
                .unwrap_or(PlayerId::NONE);
            tx.state().zones.find(ZoneKind::Hand, holder)
        } {
            tx.state_mut()
                .zones
                .get_mut(hand_zone)
                .unwrap()
                .cards
                .retain(|c| *c != card);
        }
        tx.state_mut().zones.get_mut(discard).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = discard;
        }
    }

    let target_hand: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    let shown_set: std::collections::BTreeSet<u32> = shown.iter().copied().collect();
    let taken: Vec<u32> = if player_won {
        target_hand
            .iter()
            .copied()
            .filter(|card| shown_set.contains(card))
            .collect()
    } else {
        target_hand
            .iter()
            .copied()
            .filter(|card| !shown_set.contains(card))
            .collect()
    };
    let mut events = Vec::new();
    if taken.is_empty() {
        return Ok(events);
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no player hand".into()))?;
    for raw in taken {
        let card = CardId::new(raw);
        tx.state_mut()
            .zones
            .get_mut(from)
            .unwrap()
            .cards
            .retain(|c| *c != card);
        tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = to;
            card_state.owner = player;
        }
        events.push(engine_api::DomainEvent::CardsDrawn {
            player,
            cards: vec![card],
        });
    }
    Ok(events)
}

const RESOLVE_YANGMING_MAGIC: u32 = 0x594D_0001;

/// 专属状态机 (批 253, ADR 0641 §8.43)：养名——出牌阶段限一次，与一名角色
/// 拼点：目标赢 → 目标摸「已输次数」张 + owner 回复 1 点体力，结束；否则
/// 记一次输，owner 可选择重复此流程（目标无法再拼点时自动结束）。
pub(crate) fn resolve_yangming(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_cards(player).is_empty()
    {
        return Ok(Vec::new());
    }
    let targets: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| {
            candidate != player && !tx.state().hand_cards(candidate).is_empty()
        })
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveYangmingStage,
        vec![RESOLVE_YANGMING_MAGIC, player.raw(), skill.raw(), 0],
    )
}

/// 载荷：[magic, player, skill, stage, target, losses, pend...]
/// stage 2=owner 选拼点牌（pend 空）；stage 3=目选拼点牌（pend=[c_p]）；
/// stage 4=owner 知情（Custom 1=重复 2=结束）。
fn yangming_payload(
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    target_raw: u32,
    losses: u32,
    pend: &[u32],
) -> Vec<u32> {
    let mut payload = vec![
        RESOLVE_YANGMING_MAGIC,
        player.raw(),
        skill.raw(),
        stage,
        target_raw,
        losses,
    ];
    payload.extend_from_slice(pend);
    payload
}

/// 目标选择已确认（resolver 调用）：打开第一轮 owner 选拼点牌。
pub(crate) fn resolve_yangming_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    losses: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    let payload = yangming_payload(player, skill, 2, target_raw, losses, &[]);
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveYangmingStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// 两侧都能选拼点牌时才可继续。
fn yangming_can_compare(tx: &Transaction, player: PlayerId, target: PlayerId) -> bool {
    tx.state()
        .players
        .get(player)
        .is_some_and(|p| p.alive && !tx.state().hand_cards(player).is_empty())
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|p| p.alive && !tx.state().hand_cards(target).is_empty())
}

/// 拼点轮各阶段确认（resolver 调用）。
pub(crate) fn resolve_yangming_round(
    tx: &mut Transaction,
    observer: PlayerId,
    player: PlayerId,
    skill: engine_api::SkillId,
    payload: &[u32],
    selections: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    const HEADER: usize = 6;
    if payload.len() < HEADER || payload[0] != RESOLVE_YANGMING_MAGIC {
        return Err(EffectError::EngineFault(
            "bad resolve-yangming continuation payload".into(),
        ));
    }
    let stage = payload[3];
    let target_raw = payload[4];
    let losses = payload[5];
    let pend: Vec<u32> = payload[HEADER..].to_vec();
    let target = PlayerId::new(target_raw);
    match stage {
        2 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card requires owner".into(),
                ));
            }
            let c_p = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-yangming compare card required".into())
            })?;
            let owner_hand: std::collections::BTreeSet<u32> = tx
                .state()
                .hand_cards(player)
                .iter()
                .map(|card| card.raw())
                .collect();
            if !owner_hand.contains(&c_p) {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card outside owner hand".into(),
                ));
            }
            let t_hand: Vec<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            let payload = yangming_payload(player, skill, 3, target_raw, losses, &[c_p]);
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                t_hand,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveYangmingStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        3 => {
            if observer != target {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card requires the target".into(),
                ));
            }
            if pend.len() != 1 {
                return Err(EffectError::EngineFault(
                    "resolve-yangming missing owner compare card".into(),
                ));
            }
            let c_p = pend[0];
            let c_t = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-yangming compare card required".into())
            })?;
            let target_hand: std::collections::BTreeSet<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            if !target_hand.contains(&c_t) {
                return Err(EffectError::Illegal(
                    "resolve-yangming compare card outside target hand".into(),
                ));
            }
            // 拼点牌置入弃牌堆（chooseToCompare 规则）。
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let mut events = Vec::new();
            for (holder, raw) in [(player, c_p), (target, c_t)] {
                let card = CardId::new(raw);
                if let Some(hand_zone) = tx.state().zones.find(ZoneKind::Hand, holder) {
                    tx.state_mut()
                        .zones
                        .get_mut(hand_zone)
                        .unwrap()
                        .cards
                        .retain(|c| *c != card);
                }
                tx.state_mut()
                    .zones
                    .get_mut(discard)
                    .unwrap()
                    .cards
                    .push(card);
                if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
                    card_state.zone = discard;
                }
            }
            let p_rank = tx
                .state()
                .cards
                .get(CardId::new(c_p))
                .map(|c| c.rank)
                .unwrap_or(0);
            let t_rank = tx
                .state()
                .cards
                .get(CardId::new(c_t))
                .map(|c| c.rank)
                .unwrap_or(0);
            if t_rank > p_rank {
                // 目标赢：摸已输次数张 + owner 回复 1，结束。
                if losses > 0 {
                    events.extend(crate::effect::draw_cards(tx, target, losses)?);
                }
                events.extend(super::recover_hp(tx, player, player, 1)?);
                return Ok(events);
            }
            // owner 赢或平：记一次输。
            let losses2 = losses + 1;
            if !yangming_can_compare(tx, player, target) {
                return Ok(events);
            }
            // stage 4：owner 选择是否重复。
            let payload = yangming_payload(player, skill, 4, target_raw, losses2, &[]);
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveYangmingStage,
                payload,
            )?;
            Ok(events)
        }
        4 => {
            if observer != player {
                return Err(EffectError::Illegal(
                    "resolve-yangming repeat requires owner".into(),
                ));
            }
            let repeat = *selections.first().ok_or_else(|| {
                EffectError::Illegal("resolve-yangming repeat choice required".into())
            })?;
            if repeat == 1 && yangming_can_compare(tx, player, target) {
                resolve_yangming_stage(tx, player, skill, target_raw, losses)?;
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::EngineFault("bad resolve-yangming stage".into())),
    }
}

const RESOLVE_CANYUN_MAGIC: u32 = 0x4359_0001;

/// 装备区牌数。
fn canyun_equip_count(tx: &Transaction, player: PlayerId) -> usize {
    tx.state()
        .zones
        .find(ZoneKind::Equip, player)
        .map(|zone| tx.state().zones.get(zone).unwrap().cards.len())
        .unwrap_or(0)
}

/// 专属状态机 (批 255, ADR 0641 §8.44)：残殒——出牌阶段，可弃置一张牌并
/// 选择 1-2 名其他角色，比较装备数：owner 多则其回复 1 体力，相等则其摸
/// 1 张牌，其多则其失去 1 体力；目标体力为 1 时 owner 摸一张牌。
pub(crate) fn resolve_canyun(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 {
        return Ok(Vec::new());
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_cards(player).is_empty()
    {
        return Ok(Vec::new());
    }
    let targets: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player)
        .map(PlayerId::raw)
        .collect();
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    // stage 0：弃置一张手牌（费用）。
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveCanyunStage,
        vec![RESOLVE_CANYUN_MAGIC, player.raw(), skill.raw(), 2],
    )
}

/// stage 2：弃牌已定 → 选 1-2 名其他角色。
pub(crate) fn resolve_canyun_discarded(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    discard_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player)
        .map(PlayerId::raw)
        .collect();
    let mut payload = vec![
        RESOLVE_CANYUN_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        discard_card,
    ];
    open_choice(
        tx,
        player,
        1,
        2,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveCanyunStage,
        payload,
    )
}

/// stage 3：目标已定 → 逐目标比较装备数结算；目标体力为 1 时 owner 摸 1。
pub(crate) fn resolve_canyun_resolve(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    discard_card: u32,
    targets: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let owner_equip = canyun_equip_count(tx, player) as i32;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let player_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no player hand".into()))?;
    let mut events = Vec::new();
    let mut owner_draws = 0i32;

    // 弃置费用牌。
    let cost = CardId::new(discard_card);
    tx.state_mut()
        .zones
        .get_mut(player_hand)
        .unwrap()
        .cards
        .retain(|c| *c != cost);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(cost);
    if let Some(card_state) = tx.state_mut().cards.get_mut(cost) {
        card_state.zone = discard;
    }

    for raw in targets {
        let target = PlayerId::new(*raw);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let target_equip = canyun_equip_count(tx, target) as i32;
        if owner_equip > target_equip {
            events.extend(super::recover_hp(tx, player, target, 1)?);
        } else if owner_equip == target_equip {
            events.extend(crate::effect::draw_cards(tx, target, 1)?);
        } else {
            // 目标多：其失去 1 体力（体力流失通道）。
            events.extend(super::deal_damage(
                tx,
                PlayerId::NONE,
                target,
                1,
                true,
                0,
                false,
                CardId::NONE,
                0,
                0,
            )?);
        }
        if hp_of_player(tx, target) == 1 {
            owner_draws += 1;
        }
    }
    if owner_draws > 0 {
        events.extend(crate::effect::draw_cards(tx, player, owner_draws as u32)?);
    }
    Ok(events)
}

fn hp_of_player(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .map(|participant| participant.hp)
        .unwrap_or(0)
}

/// 专属状态机 (批 256, ADR 0641 §8.45)：横骛——使用/打出牌后，若手牌无此
/// 花色且场上有同花色装备：摸「场上该花色装备数」张（frequent 锁定自动）。
pub(crate) fn resolve_hengwu(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_card: u32,
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
    // 事件牌花色（无牌/无色 → 结束）。
    let Some(suit) = tx
        .state()
        .cards
        .get(CardId::new(event_card))
        .map(|card| card.suit)
        .filter(|suit| {
            matches!(
                suit,
                engine_api::Suit::Spade
                    | engine_api::Suit::Heart
                    | engine_api::Suit::Club
                    | engine_api::Suit::Diamond
            )
        })
    else {
        return Ok(Vec::new());
    };
    // 门控：手牌无此花色。
    if tx
        .state()
        .hand_cards(player)
        .iter()
        .any(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card_state| card_state.suit == suit)
        })
    {
        return Ok(Vec::new());
    }
    // 场上同花色装备数。
    let mut count = 0usize;
    for zone in tx.state().zones.sorted_live() {
        if zone.kind != ZoneKind::Equip {
            continue;
        }
        for card in &zone.cards {
            if tx
                .state()
                .cards
                .get(*card)
                .is_some_and(|card_state| card_state.suit == suit)
            {
                count += 1;
            }
        }
    }
    if count == 0 {
        return Ok(Vec::new());
    }
    crate::effect::draw_cards(tx, player, count as u32)
}

const RESOLVE_BIAOZHAO3_MAGIC: u32 = 0x425A_0001;

/// 表召族「表」堆 def（biaozhao 自身堆，跨技 def 字面量）。
const BIAOZHAO_PILE_DEF: u32 = 3_640_810_944;

/// 专属状态机 (批 257, ADR 0641 §8.46)：表召·归——准备阶段若「表」堆有牌：
/// 弃置首张，令一名角色（可选自己，可取消）摸至全场最大手牌数（至多 5）并
/// 回复 1 点体力。全场最大手牌数机内计算。
pub(crate) fn resolve_biaozhao3(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            // 门控：「表」堆非空（forced 触发体内由 DSL 门控保证，机内再校验）。
            let pile = tx.state().skill_pile_cards(player, BIAOZHAO_PILE_DEF);
            if pile.is_empty() {
                return Ok(Vec::new());
            }
            // 弃置首张「表」。
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let card = pile[0];
            if let Some(pile_zone) = tx
                .state()
                .skill_piles
                .iter()
                .find(|p| p.owner == player && p.skill_def == BIAOZHAO_PILE_DEF)
                .map(|p| p.zone)
            {
                tx.state_mut()
                    .zones
                    .get_mut(pile_zone)
                    .unwrap()
                    .cards
                    .retain(|c| *c != card);
            }
            tx.state_mut()
                .zones
                .get_mut(discard)
                .unwrap()
                .cards
                .push(card);
            if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
                card_state.zone = discard;
            }
            // 全场最大手牌数（机内计算）。
            let mut num = 0usize;
            for participant in tx.state().players.alive_by_seat() {
                num = num.max(tx.state().hand_cards(participant).len());
            }
            let targets: Vec<u32> = tx
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
                1,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveBiaozhao3Stage,
                vec![
                    RESOLVE_BIAOZHAO3_MAGIC,
                    player.raw(),
                    skill.raw(),
                    2,
                    num as u32,
                ],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 续体：selections[0] = 受益角色。摸至 num（至多补 5 张）并回复 1 点体力。
pub(crate) fn resolve_biaozhao3_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    num: u32,
    selected: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    if selected.is_empty() {
        return Ok(Vec::new());
    }
    let target = PlayerId::new(selected[0]);
    let mut events = Vec::new();
    if num > tx.state().hand_cards(target).len() as u32 {
        let draw = (num - tx.state().hand_cards(target).len() as u32).min(5);
        if draw > 0 {
            events.extend(crate::effect::draw_cards(tx, target, draw)?);
        }
    }
    events.extend(super::recover_hp(tx, player, target, 1)?);
    Ok(events)
}

/// 专属状态机 (批 258, ADR 0641 §8.47)：穿云——「传术」衍生技。杀指定目标后
/// 若其装备区有牌：随机弃置其中一张。
pub(crate) fn resolve_chuanyun(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_player: PlayerId,
    event_card: u32,
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
    // 事件牌须为【杀】（普/火/雷）。
    let is_sha = tx
        .state()
        .cards
        .get(CardId::new(event_card))
        .is_some_and(|card| {
            matches!(
                card.kind,
                crate::state::CardKind::Sha
                    | crate::state::CardKind::ShaFire
                    | crate::state::CardKind::ShaThunder
            )
        });
    if !is_sha || event_player == PlayerId::NONE || event_player == player {
        return Ok(Vec::new());
    }
    // 目标装备区有牌：RNG 随机弃置一张。
    let Some(equip_zone) = tx.state().zones.find(ZoneKind::Equip, event_player) else {
        return Ok(Vec::new());
    };
    let equips = tx.state().zones.get(equip_zone).unwrap().cards.clone();
    if equips.is_empty() {
        return Ok(Vec::new());
    }
    let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
    let idx = rng.gen_range_u32(equips.len() as u32) as usize;
    tx.state_mut().rng = rng.into_state();
    let card = equips[idx];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    tx.state_mut()
        .zones
        .get_mut(equip_zone)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = discard;
    }
    Ok(Vec::new())
}

const RESOLVE_BIAOZHAO2_MAGIC: u32 = 0x425A_0002;

/// 专属状态机 (批 259, ADR 0641 §8.48)：表召·应——任意角色弃置与「表」
/// 同花色同点数的牌时（cards_moved 过滤器已判定），将「表」交给弃牌者，
/// owner 失去 1 点体力。无弃牌者（来源不可考）时「表」入弃牌堆。
pub(crate) fn resolve_biaozhao2(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    event_player: PlayerId,
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
    let pile = tx.state().skill_pile_cards(player, BIAOZHAO_PILE_DEF);
    if pile.is_empty() {
        return Ok(Vec::new());
    }
    let card = pile[0];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    // 从「表」堆移除。
    if let Some(pile_zone) = tx
        .state()
        .skill_piles
        .iter()
        .find(|p| p.owner == player && p.skill_def == BIAOZHAO_PILE_DEF)
        .map(|p| p.zone)
    {
        tx.state_mut()
            .zones
            .get_mut(pile_zone)
            .unwrap()
            .cards
            .retain(|c| *c != card);
    }
    // 有存活弃牌者 → 「表」交给弃牌者；否则「表」入弃牌堆。
    let giver_target = if event_player != PlayerId::NONE
        && tx
            .state()
            .players
            .get(event_player)
            .is_some_and(|participant| participant.alive)
    {
        event_player
    } else {
        player
    };
    if giver_target == player {
        // 无可交付对象 → 「表」入弃牌堆。
        tx.state_mut()
            .zones
            .get_mut(discard)
            .unwrap()
            .cards
            .push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = discard;
        }
    } else {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, giver_target)
            .ok_or_else(|| EffectError::Illegal("no discarder hand".into()))?;
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
            card_state.zone = hand;
            card_state.owner = giver_target;
        }
    }
    // owner 失去 1 点体力。
    let lose = super::deal_damage(tx, PlayerId::NONE, player, 1, true, 0, false, CardId::NONE, 0, 0)?;
    Ok(lose)
}

const RESOLVE_DCNIJI_MAGIC: u32 = 0x4E4A_0001;

/// 「逆击」gaintag（content_card_tags，对应上游 gaintag "dcniji"）。
pub(crate) const DCNIJI_TAG: u32 = 3_628_735_017;

/// 专属状态机 (批 260, ADR 0641 §8.49)：逆击——成为非装备牌目标后摸 1 张牌
/// 并打上「逆击」标记；结束阶段弃置手牌中全部「逆击」牌。
pub(crate) fn resolve_dcniji(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    event_card: u32,
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
    match mode {
        0 => {
            // 非装备牌目标：事件牌须存在且非装备。
            let Some(targeting) = tx.state().cards.get(CardId::new(event_card)) else {
                return Ok(Vec::new());
            };
            if targeting.kind.equip_slot().is_some() {
                return Ok(Vec::new());
            }
            let mut events = crate::effect::draw_cards(tx, player, 1)?;
            if let Some(engine_api::DomainEvent::CardsDrawn { cards, .. }) = events.last() {
                for card in cards {
                    tx.state_mut().add_content_card_tag(*card, DCNIJI_TAG);
                }
            }
            Ok(events)
        }
        1 => {
            // 结束阶段：弃置手牌中全部「逆击」牌。
            let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            let tagged: Vec<CardId> = tx
                .state()
                .zones
                .get(hand)
                .unwrap()
                .cards
                .iter()
                .copied()
                .filter(|card| tx.state().card_has_content_tag(*card, DCNIJI_TAG))
                .collect();
            for card in tagged {
                tx.state_mut()
                    .zones
                    .get_mut(hand)
                    .unwrap()
                    .cards
                    .retain(|c| *c != card);
                tx.state_mut()
                    .zones
                    .get_mut(discard)
                    .unwrap()
                    .cards
                    .push(card);
                if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
                    card_state.zone = discard;
                }
            }
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

const RESOLVE_DCMOUKUI_MAGIC: u32 = 0x4D4B_0001;

/// 谋溃「双选已定」标记的 storage 键（slice 2 追罚用）。
const DCMOUKUI_FLAG_KEY: u32 = 1;

/// 专属状态机 (批 261, ADR 0641 §8.50)：谋溃——杀指定目标后，chooseButton
/// [1,2]：摸 1 张牌／弃置目标一张手牌（owner 从目标手牌选），可同选两项。
/// 未命中追罚面留 slice 2。
pub(crate) fn resolve_dcmoukui(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            // 新一次杀指定：清除上一杀可能残留的追罚标记。
            tx.state_mut()
                .skills
                .storage_set(skill, DCMOUKUI_FLAG_KEY, 0);
            // Custom 双选：1=摸一张牌 2=弃目标一张牌；可选 1-2 项，不可取消。
            open_choice(
                tx,
                player,
                1,
                2,
                false,
                vec![1, 2],
                ChoiceKind::Custom,
                ContinuationKind::ResolveDcmoukuiStage,
                vec![
                    RESOLVE_DCMOUKUI_MAGIC,
                    player.raw(),
                    skill.raw(),
                    2,
                    target.raw(),
                ],
            )
        }
        5 => {
            // 追罚入口：杀未命中且双选标记在 → 闪避者选 owner 一张手牌弃置。
            resolve_dcmoukui_conseq(tx, player, skill, target)
        }
        _ => Ok(Vec::new()),
    }
}

/// stage 2：知情选择已定 → 应用。
pub(crate) fn resolve_dcmoukui_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target: PlayerId,
    selections: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = Vec::new();
    let draw = selections.contains(&1);
    let discard = selections.contains(&2);
    if draw && discard {
        // 双选：记录追罚标记（此杀未命中 → 目标弃 owner 一张手牌）。
        tx.state_mut()
            .skills
            .storage_set(skill, DCMOUKUI_FLAG_KEY, 1);
    }
    if draw {
        events.extend(crate::effect::draw_cards(tx, player, 1)?);
    }
    if discard {
        // owner 从目标手牌选一张弃置。
        let options: Vec<u32> = tx
            .state()
            .hand_cards(target)
            .iter()
            .map(|card| card.raw())
            .collect();
        if options.is_empty() {
            return Ok(events);
        }
        let payload = vec![
            RESOLVE_DCMOUKUI_MAGIC,
            player.raw(),
            skill.raw(),
            3,
            target.raw(),
        ];
        open_choice(
            tx,
            player,
            1,
            1,
            false,
            options,
            ChoiceKind::SelectCards,
            ContinuationKind::ResolveDcmoukuiStage,
            payload,
        )?;
    }
    Ok(events)
}

/// stage 3：弃置目标手牌 selections[0]。
pub(crate) fn resolve_dcmoukui_discard(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target: PlayerId,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let card = CardId::new(card_raw);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
    if !tx
        .state()
        .zones
        .get(hand)
        .unwrap()
        .cards
        .contains(&card)
    {
        return Err(EffectError::Illegal(
            "dcmoukui card outside target hand".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = discard;
    }
    Ok(Vec::new())
}

const RESOLVE_ZHONGYAN_MAGIC: u32 = 0x5A5A_0001;

/// 专属状态机 (批 262, ADR 0641 §8.51)：忠言——出牌阶段限一次，亮出牌堆顶
/// 三张牌，一名有手牌的角色以其中一张与其一张手牌交换（chooseToMove 族）。
pub(crate) fn resolve_zhongyan(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| !tx.state().hand_cards(candidate).is_empty())
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveZhongyanStage,
                vec![RESOLVE_ZHONGYAN_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// 目标已选（resolver 调用）：亮出牌堆顶三张，目标选其中一张。
pub(crate) fn resolve_zhongyan_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    if tx.state().hand_cards(target).is_empty() {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let top = tx.state().zones.get(draw).unwrap().cards.clone();
    if top.is_empty() {
        return Ok(Vec::new());
    }
    // 牌堆顶 = 向量尾部。
    let take = top.len().min(3);
    let revealed: Vec<CardId> = top[top.len() - take..].to_vec();
    crate::effect::run_effects(
        tx,
        vec![EffectRequest::RevealCards {
            cards: revealed.clone(),
            to: Vec::new(),
        }],
    )?;
    let mut payload = vec![
        RESOLVE_ZHONGYAN_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        target_raw,
        revealed.len() as u32,
    ];
    payload.extend(revealed.iter().map(|c| c.raw()));
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        revealed.iter().map(|c| c.raw()).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveZhongyanStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// 目标已选亮牌（resolver 调用）：目标再选自己一张手牌交换。
pub(crate) fn resolve_zhongyan_pick(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    revealed: &[u32],
    revealed_pick: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    let mut payload = vec![
        RESOLVE_ZHONGYAN_MAGIC,
        player.raw(),
        skill.raw(),
        4,
        target_raw,
        revealed.len() as u32,
    ];
    payload.extend_from_slice(revealed);
    payload.push(revealed_pick);
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveZhongyanStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// 交换结算：所选亮牌入手牌，所选手牌置回牌堆顶（原亮牌位置）。
pub(crate) fn resolve_zhongyan_swap(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    revealed: &[u32],
    revealed_pick: u32,
    hand_card: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = (player, skill);
    let target = PlayerId::new(target_raw);
    let pick = CardId::new(revealed_pick);
    let hand_card = CardId::new(hand_card);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
    // 亮牌 → 目标手牌。
    if let Some(pos) = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .position(|c| *c == pick)
    {
        tx.state_mut()
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .remove(pos);
    }
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(pick);
    if let Some(card_state) = tx.state_mut().cards.get_mut(pick) {
        card_state.zone = hand;
        card_state.owner = target;
    }
    // 目标手牌 → 牌堆顶（原亮牌位置；找不到则置顶）。
    let insert_pos = {
        let cards = &tx.state().zones.get(draw).unwrap().cards;
        cards
            .iter()
            .position(|c| revealed.contains(&c.raw()))
            .unwrap_or(0)
    };
    if let Some(pos) = {
        let cards = &tx.state().zones.get(hand).unwrap().cards;
        cards.iter().position(|c| *c == hand_card)
    } {
        tx.state_mut()
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .remove(pos);
    }
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .insert(insert_pos, hand_card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(hand_card) {
        card_state.zone = draw;
        card_state.owner = PlayerId::NONE;
    }
    Ok(Vec::new())
}

const RESOLVE_CHUMEI_MAGIC: u32 = 0x4348_0001;

/// 专属状态机 (批 263, ADR 0641 §8.52)：狐魅——出牌阶段，选择一名体力不超过
/// owner 本回合造成伤害总数的角色，令其执行一项：摸一张牌／交给 owner 一张
/// 手牌／回复 1 点体力（受伤时）。
pub(crate) fn resolve_chumei(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            // 门控：本回合造成伤害总数 > 0（否则无目标满足体力 ≤ 该值）。
            let dealt = tx.state().turn.damage_dealt_count(player);
            if dealt == 0 {
                return Ok(Vec::new());
            }
            let dealt = dealt as i32;
            let targets: Vec<u32> = tx
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
                            .is_some_and(|p| p.alive && p.hp <= dealt)
                })
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveChumeiStage,
                vec![RESOLVE_CHUMEI_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// stage 2：目标已定 → 三选一（1=目标摸 1；2=目标交 owner 一张手牌；
/// 3=目标回复 1，仅目标受伤时可选）。
pub(crate) fn resolve_chumei_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let target_damaged = tx
        .state()
        .players
        .get(target)
        .is_some_and(|p| p.hp < p.max_hp as i32);
    let mut options = vec![1u32];
    if !tx.state().hand_cards(target).is_empty() {
        options.push(2);
    }
    if target_damaged {
        options.push(3);
    }
    let payload = vec![
        RESOLVE_CHUMEI_MAGIC,
        player.raw(),
        skill.raw(),
        2,
        target_raw,
        options.len() as u32,
    ]
    .into_iter()
    .chain(options.iter().copied())
    .collect::<Vec<u32>>();
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ResolveChumeiStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// stage 2 选「给牌」后的子选择：目标从手牌选一张交给 owner。
pub(crate) fn resolve_chumei_give_pick(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let hand: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    let payload = vec![
        RESOLVE_CHUMEI_MAGIC,
        player.raw(),
        skill.raw(),
        4,
        target_raw,
    ];
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        hand,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveChumeiStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// stage 4：目标选定的手牌交给 owner。
pub(crate) fn resolve_chumei_give(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let target = PlayerId::new(target_raw);
    let card = CardId::new(card_raw);
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
    if !tx
        .state()
        .zones
        .get(from)
        .unwrap()
        .cards
        .contains(&card)
    {
        return Err(EffectError::Illegal(
            "chumei card outside target hand".into(),
        ));
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no owner hand".into()))?;
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = to;
        card_state.owner = player;
    }
    Ok(Vec::new())
}


/// 追罚入口（stage 5）：杀未命中（ShaDodged）且双选标记在 → 目标（闪避者）
/// 弃置 owner 一张手牌。
fn resolve_dcmoukui_conseq(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    dodger: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx
        .state()
        .skills
        .storage_get(skill, DCMOUKUI_FLAG_KEY)
        != 1
    {
        return Ok(Vec::new());
    }
    // 消耗标记。
    tx.state_mut()
        .skills
        .storage_set(skill, DCMOUKUI_FLAG_KEY, 0);
    if !tx
        .state()
        .players
        .get(dodger)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_cards(player).is_empty()
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .iter()
        .map(|card| card.raw())
        .collect();
    let payload = vec![
        RESOLVE_DCMOUKUI_MAGIC,
        player.raw(),
        skill.raw(),
        6,
        dodger.raw(),
    ];
    open_choice(
        tx,
        dodger,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveDcmoukuiStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// stage 6：闪避者选定的 owner 手牌弃置。
pub(crate) fn resolve_dcmoukui_conseq_discard(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    dodger: PlayerId,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let card = CardId::new(card_raw);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no owner hand".into()))?;
    if !tx
        .state()
        .zones
        .get(hand)
        .unwrap()
        .cards
        .contains(&card)
    {
        return Err(EffectError::Illegal(
            "dcmoukui conseq card outside owner hand".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = discard;
    }
    Ok(Vec::new())
}

const RESOLVE_QINMIAN_MAGIC: u32 = 0x514D_0001;

/// 「勤勉」gaintag（content_card_tags，对应上游 gaintag "qinmian"）。
pub(crate) const QINMIAN_TAG: u32 = 3_958_551_236;

/// 专属状态机 (批 265, ADR 0641 §8.53)：勤勉——mode 0：准备阶段摸 1 张或
/// 受到伤害后按伤害点数摸牌，摸得牌打「勤勉」标记；mode 1：全场结束阶段，
/// 可弃置手牌中全部标记牌并对当前回合角色造成等量伤害（若手牌全部为标记牌
/// 则等值为其张数，否则 1 点）。
pub(crate) fn resolve_qinmian(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    amount: u32,
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
    let mut events: Vec<DomainEvent> = Vec::new();
    match mode {
        0 => {
            let draw_n = amount.max(1);
            let mut drawn = crate::effect::draw_cards(tx, player, draw_n)?;
            if let Some(engine_api::DomainEvent::CardsDrawn { cards, .. }) = drawn.last() {
                for card in cards {
                    tx.state_mut().add_content_card_tag(*card, QINMIAN_TAG);
                }
            }
            events.extend(drawn);
            Ok(events)
        }
        1 => {
            // 全场结束阶段（当前回合角色 = state.turn.current）。
            let current = tx.state().turn.current;
            let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
            let tagged: Vec<CardId> = tx
                .state()
                .zones
                .get(hand)
                .unwrap()
                .cards
                .iter()
                .copied()
                .filter(|card| tx.state().card_has_content_tag(*card, QINMIAN_TAG))
                .collect();
            if tagged.is_empty() {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
            for card in &tagged {
                tx.state_mut()
                    .zones
                    .get_mut(hand)
                    .unwrap()
                    .cards
                    .retain(|c| *c != *card);
                tx.state_mut()
                    .zones
                    .get_mut(discard)
                    .unwrap()
                    .cards
                    .push(*card);
                if let Some(card_state) = tx.state_mut().cards.get_mut(*card) {
                    card_state.zone = discard;
                }
            }
            // 伤害值：手牌全部为标记牌 → 等于张数；否则 1 点（上游 isAllQinmian）。
            let hand_now = tx.state().hand_cards(player).len();
            let damage = if hand_now == 0 { tagged.len() as u32 } else { 1 };
            if current != PlayerId::NONE && damage > 0 {
                events.extend(super::pack_mobile::deal_damage(
                    tx,
                    player,
                    current,
                    damage,
                    false,
                    0,
                    false,
                    CardId::NONE,
                    0,
                    0,
                )?);
            }
            Ok(events)
        }
        _ => Ok(Vec::new()),
    }
}

const RESOLVE_SHUAIYAN_MAGIC: u32 = 0x5348_0001;

/// 专属状态机 (批 266, ADR 0641 §8.54)：率言——弃牌阶段开始，若手牌数大于
/// 1：选择一名其他角色，其交给你一张牌（手牌或装备区）。
pub(crate) fn resolve_shuaiyan(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
                || tx.state().hand_cards(player).len() <= 1
            {
                return Ok(Vec::new());
            }
            let candidates: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| {
                    candidate != player
                        && (tx.state().zones.find(ZoneKind::Hand, candidate).is_some()
                            && !tx.state().hand_cards(candidate).is_empty()
                            || tx
                                .state()
                                .zones
                                .find(ZoneKind::Equip, candidate)
                                .is_some_and(|zone| {
                                    !tx.state().zones.get(zone).unwrap().cards.is_empty()
                                }))
                })
                .map(PlayerId::raw)
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                candidates,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveShuaiyanStage,
                vec![RESOLVE_SHUAIYAN_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        2 => {
            // 目标已选：其从手牌/装备区选一张交给 owner。
            let target = tx
                .state()
                .resolution
                .continuation_payload
                .get(4)
                .map(|raw| PlayerId::new(*raw))
                .unwrap_or(PlayerId::NONE);
            let mut options: Vec<u32> = tx
                .state()
                .hand_cards(target)
                .iter()
                .map(|card| card.raw())
                .collect();
            if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, target) {
                options.extend(
                    tx.state()
                        .zones
                        .get(equip)
                        .unwrap()
                        .cards
                        .iter()
                        .map(|card| card.raw()),
                );
            }
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let payload = vec![
                RESOLVE_SHUAIYAN_MAGIC,
                player.raw(),
                skill.raw(),
                3,
                target.raw(),
            ];
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveShuaiyanStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

/// stage 3：目标选定的牌交给 owner。
pub(crate) fn resolve_shuaiyan_give(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let target = PlayerId::new(target_raw);
    let card = CardId::new(card_raw);
    // 从目标的手牌或装备区移除。
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .filter(|zone| {
            tx.state()
                .zones
                .get(*zone)
                .unwrap()
                .cards
                .contains(&card)
        })
        .or_else(|| tx.state().zones.find(ZoneKind::Equip, target))
        .ok_or_else(|| EffectError::Illegal("shuaiyan card not found".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no owner hand".into()))?;
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = to;
        card_state.owner = player;
    }
    Ok(vec![engine_api::DomainEvent::CardsDrawn {
        player,
        cards: vec![card],
    }])
}

/// stage 2：目标已选 → 目标从手牌/装备区选一张交给 owner。
pub(crate) fn resolve_shuaiyan_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    target_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = PlayerId::new(target_raw);
    let mut options: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .iter()
        .map(|card| card.raw())
        .collect();
    if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, target) {
        options.extend(
            tx.state()
                .zones
                .get(equip)
                .unwrap()
                .cards
                .iter()
                .map(|card| card.raw()),
        );
    }
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let payload = vec![
        RESOLVE_SHUAIYAN_MAGIC,
        player.raw(),
        skill.raw(),
        3,
        target_raw,
    ];
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveShuaiyanStage,
        payload,
    )?;
    Ok(Vec::new())
}

/// 专属状态机 (批 275, ADR 0641 §8.63)：鄙位——出牌阶段限一次，弃手牌中
/// 点数最大的一张，令一名有手牌的其他角色弃置所有点数≥该牌的手牌；若其
/// 未因此弃牌，重置技能的出牌阶段使用计数（上游 delete getStat("skill")）。
pub(crate) fn resolve_friendbiwei(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if target == player {
        return Err(EffectError::Illegal(
            "resolve-friendbiwei target must be another player".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_count(target) == 0
    {
        return Ok(Vec::new());
    }
    // 成本牌必须是 owner 手牌中点数最大的一张。
    let hand = tx.state().hand_cards(player);
    if !hand.contains(&card) {
        return Err(EffectError::Illegal(
            "resolve-friendbiwei card is not in the owner hand".into(),
        ));
    }
    let cost_rank = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.rank)
        .ok_or_else(|| EffectError::Illegal("resolve-friendbiwei card is missing".into()))?;
    let max_other = hand
        .iter()
        .filter(|other| **other != card)
        .filter_map(|other| tx.state().cards.get(*other).map(|card| card.rank))
        .max()
        .unwrap_or(0);
    if cost_rank < max_other {
        return Err(EffectError::Illegal(
            "resolve-friendbiwei card is not the highest rank".into(),
        ));
    }
    // 弃成本牌。
    let mut events = run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player,
            cards: vec![card],
        }],
    )?;
    // 目标弃置所有点数 ≥ 成本点数的手牌。
    let to_discard: Vec<CardId> = tx
        .state()
        .hand_cards(target)
        .into_iter()
        .filter(|other| {
            tx.state()
                .cards
                .get(*other)
                .is_some_and(|card| card.rank >= cost_rank)
        })
        .collect();
    if to_discard.is_empty() {
        // 未因此弃牌 → 重置技能使用计数（可再次发动）。
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.use_count = 0;
            instance.play_phase_use_count = 0;
        }
        return Ok(events);
    }
    events.extend(run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: target,
            cards: to_discard,
        }],
    )?);
    Ok(events)
}

const RESOLVE_CHULAO_MAGIC: u32 = 0x434C_0001;

/// 专属状态机 (批 274, ADR 0641 §8.62)：黜老——出牌阶段限一次，弃一张牌
/// （黑桃则 owner 摸 1），对任意名势力各异角色逐个由 owner 选弃其一张牌
/// （被弃牌黑桃则该目标摸 1）。stage 0=弃成本牌 → 1=成本结算+对首目标开
/// 选牌 → 2=弃目标牌+黑桃摸 1+循环下一目标。
pub(crate) fn resolve_chulao(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    // 势力各异且非 unknown。
    let mut seen: Vec<String> = Vec::new();
    for target in targets {
        let group = tx.state().player_group(*target);
        match group {
            None => return Ok(Vec::new()),
            Some(group) => {
                if seen.iter().any(|existing| existing == group) {
                    return Err(EffectError::Illegal(
                        "resolve-chulao targets must have distinct groups".into(),
                    ));
                }
                seen.push(group.to_string());
            }
        }
        if !tx
            .state()
            .players
            .get(*target)
            .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "resolve-chulao targets must be alive".into(),
            ));
        }
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        RESOLVE_CHULAO_MAGIC,
        1u32,
        player.raw(),
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveChulaoStage,
        payload,
    )
}

const RESOLVE_DINGCE_MAGIC: u32 = 0x4443_0002;

/// 专属状态机 (批 281, ADR 0641 §8.69)：定策——受到伤害后，弃自己一张手牌
/// （可取消），再令伤害来源弃一张手牌（owner 代选，强制）。
/// stage 0=owner 选自己的手牌 → 1=弃置+对来源开选牌 → 2=弃置来源牌。
/// 同色分支（虚拟【洞烛先机】）因卡牌不在牌池登记债，暂不结算。
pub(crate) fn resolve_dingce(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(source)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = crate::query::discardable_hand_equip_cards(tx.state(), player)
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
        1,
        true,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveDingceStage,
        vec![RESOLVE_DINGCE_MAGIC, 1, player.raw(), source.raw()],
    )
}

const RESOLVE_DCQIAOLVE_MAGIC: u32 = 0x514C_0001;

/// 专属状态机 (批 286, ADR 0641 §8.71)：巧略弃牌代价补全——用牌指定目标后，
/// 先弃自己一张牌（强制），再取事件目标一张随机手牌。
/// stage 0=owner 选成本牌（1..1 强制）→ 1=弃成本 + TakeOneHandCard。
pub(crate) fn resolve_dcqiaolve(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_count(target) == 0
    {
        return Ok(Vec::new());
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveDcqiaolveStage,
        vec![
            RESOLVE_DCQIAOLVE_MAGIC,
            1,
            player.raw(),
            target.raw(),
        ],
    )
}

/// 专属状态机 (批 287, ADR 0641 §8.72)：雄军——锁定技，owner 造成伤害后，
/// 每名拥有〖雄军〗的角色各摸一张牌（按 def 遍历存活持有者）。单发无续体。
/// 雄军去重键：记录已处理的 damage seq（DealDamage 对 DamageDealt 存在
/// 双分派——批 287 实测；同一伤害只结算一次）。
const RESOLVE_TW_XIONGJUN_LAST_DAMAGE: u32 = 0x584A_0002;

pub(crate) fn resolve_twxiongjun(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let damage_token = tx.state().resolution.current_damage_id;
    if damage_token != 0 {
        if let Some(instance) = tx
            .state()
            .players
            .get(player)
            .and_then(|p| p.skills.iter().find_map(|skill| {
                tx.state().skills.get(*skill).and_then(|instance| {
                    (instance.def.raw() == 2_909_562_544).then_some(instance.id)
                })
            }))
        {
            if tx.state().skills.storage_get(instance, RESOLVE_TW_XIONGJUN_LAST_DAMAGE) == damage_token
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .skills
                .storage_set(instance, RESOLVE_TW_XIONGJUN_LAST_DAMAGE, damage_token);
        }
    }
    let mut events = Vec::new();
    for holder in tx.state().players.alive_by_seat() {
        let owns = tx.state().players.get(holder).is_some_and(|p| {
            p.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def.raw() == 2_909_562_544)
            })
        });
        if !owns {
            continue;
        }
        events.extend(crate::effect::draw_cards(tx, holder, 1)?);
    }
    let _ = player;
    Ok(events)
}

/// 专属状态机 (批 288, ADR 0641 §8.73)：义襄——每名角色回合限一次，成为
/// 使用牌目标时随机获得牌堆中一张自己没有的基本牌。单发无续体。
pub(crate) fn resolve_yixiang(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let hand_kinds: Vec<crate::state::CardKind> = tx
        .state()
        .hand_cards(player)
        .iter()
        .filter_map(|card| tx.state().cards.get(*card))
        .map(|card| card.kind)
        .collect();
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let candidate = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .copied()
        .find(|card| {
            tx.state().cards.get(*card).is_some_and(|c| {
                c.kind.is_basic()
                    && !hand_kinds.contains(&c.kind)
                    && !tx.state().hand_cards(player).contains(card)
            })
        });
    let Some(card) = candidate else {
        return Ok(Vec::new());
    };
    move_cards(tx, vec![card], hand, Some(draw))
}

const RESOLVE_MEIYONG_MAGIC: u32 = 0x4D59_0001;

/// 求援 giver 扫描：座位序从 `after` 之后环绕，跳过 owner、已死与无
/// 手+装备牌者（上游 `game.filterPlayer(current != player).sortBySeat()`）。
pub(crate) fn xinfu_qiai_next_giver(
    state: &crate::state::GameState,
    owner: PlayerId,
    after: PlayerId,
) -> Option<PlayerId> {
    let alive = state.players.alive_by_seat();
    let start = alive
        .iter()
        .position(|id| *id == after)
        .map(|index| index + 1)
        .unwrap_or(0);
    for offset in 0..alive.len() {
        let candidate = alive[(start + offset) % alive.len()];
        if candidate == owner {
            return None;
        }
        if !crate::query::discardable_hand_equip_cards(state, candidate).is_empty() {
            return Some(candidate);
        }
    }
    None
}

const RESOLVE_XINFU_QIAI_MAGIC: u32 = 0x5149_0001;
/// 专属状态机 (批 291, ADR 0641 §8.74)：求援——限定技，濒死时其他角色依
/// 座次各交给 owner 一张牌（手+装备，强制，跳过无牌者）。stage 1=对 giver
/// 开选牌（强制 1..1）→ 续体=移交 owner 手牌区+循环下一 giver。
pub(crate) fn resolve_xinfu_qiai(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(giver) = xinfu_qiai_next_giver(tx.state(), player, player) else {
        return Ok(Vec::new());
    };
    let options: Vec<u32> = crate::query::discardable_hand_equip_cards(tx.state(), giver)
        .into_iter()
        .map(CardId::raw)
        .collect();
    open_choice(
        tx,
        giver,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveXinfuQiaiStage,
        vec![
            RESOLVE_XINFU_QIAI_MAGIC,
            1,
            player.raw(),
            giver.raw(),
        ],
    )
}

/// 专属状态机 (批 292, ADR 0641 §8.76)：温仁——出牌阶段限一次，任选多名
/// 目标，每名目标无手牌摸 1、手牌数不大于 owner 再摸 1（可叠 2，可摸 0）。
/// 单发无续体；目标按座位序去重结算（上游逐目标 await draw 同序）。
pub(crate) fn resolve_olwenren(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    let order = tx.state().players.alive_by_seat();
    let mut sorted: Vec<PlayerId> = Vec::with_capacity(targets.len());
    for &target in targets {
        if sorted.contains(&target) {
            continue;
        }
        if !order.contains(&target) {
            continue;
        }
        sorted.push(target);
    }
    sorted.sort_by_key(|target| {
        order
            .iter()
            .position(|candidate| candidate == target)
            .expect("validated living target")
    });
    let owner_hand = tx.state().hand_count(player);
    let mut events = Vec::new();
    for target in sorted {
        let mut num: u32 = 0;
        if tx.state().hand_count(target) == 0 {
            num += 1;
        }
        if tx.state().hand_count(target) <= owner_hand {
            num += 1;
        }
        if num > 0 {
            events.extend(draw_cards(tx, target, num)?);
        }
    }
    Ok(events)
}

const SPOLZHUBEI_DEF: u32 = 3_309_571_556;
const SPOLXIXIANG_DEF: u32 = 2_531_785_939;

/// 专属状态机 (批 293, ADR 0641 §8.77)：霸业 spolaige——觉醒技：全局第二次
/// 濒死结算后，失去〖西向〗获得〖朔北〗，摸牌至体力上限并回复体力至体力
/// 上限（差值在机内计算：上游 drawTo/recoverTo 的等价量表达）。
/// 单发无续体；一次性由 owner_alive_second_global_dying（计数==2）承载。
pub(crate) fn resolve_spolaige(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut events = run_effects(
        tx,
        vec![
            EffectRequest::AddSkill {
                player,
                skill_def: engine_api::SkillDefId::new(SPOLZHUBEI_DEF),
            },
            EffectRequest::RemoveSkillByDef {
                player,
                skill_def: engine_api::SkillDefId::new(SPOLXIXIANG_DEF),
            },
        ],
    )?;
    let max_hp = tx
        .state()
        .players
        .get(player)
        .map(|participant| participant.max_hp)
        .unwrap_or(0);
    let draw = max_hp.saturating_sub(tx.state().hand_count(player));
    if draw > 0 {
        events.extend(draw_cards(tx, player, draw)?);
    }
    let hp = tx
        .state()
        .players
        .get(player)
        .map(|participant| participant.hp)
        .unwrap_or(0);
    let recover = i64::from(max_hp) - i64::from(hp);
    if recover > 0 {
        let amount = u32::try_from(recover).unwrap_or(0);
        events.extend(recover_hp(tx, PlayerId::NONE, player, amount)?);
    }
    Ok(events)
}

const RESOLVE_OLZHIJIE_MAGIC: u32 = 0x5A4A_0001;

/// 专属状态机 (批 294, ADR 0641 §8.78)：直谏——任意角色结束阶段，选射程
/// 内一名角色对其造成 1 点伤害（forced）。stage 1=选射程内目标（强制
/// SelectTargets）→ 续体=对目标造成 1 点伤害。
pub(crate) fn resolve_olzhijie(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| {
            *candidate != player
                && crate::query::player_in_attack_range(tx.state(), player, *candidate)
        })
        .map(PlayerId::raw)
        .collect();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveOlzhijieStage,
        vec![RESOLVE_OLZHIJIE_MAGIC, 1, player.raw()],
    )
}

/// 专属状态机 (批 295, ADR 0641 §8.79)：俗义——出牌阶段限一次，选一名
/// 其他角色：牌堆顶向下检索第一张其可装备的装备牌，令其获得并装备；
/// 然后对其视为使用一张无来源消耗的【杀】（上游 useCard isCard:true）。
/// 单发无续体；上游 get.cardPile 随机检索以顺序检索近似（登记债）。
pub(crate) fn resolve_olsuyi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let target_slot_enabled = |tx: &Transaction, card: CardId| {
        tx.state()
            .cards
            .get(card)
            .and_then(|card_state| {
                crate::query::effective_card_kind(tx.state(), target, card_state).equip_slot()
            })
            .is_some_and(|slot| {
                tx.state()
                    .players
                    .get(target)
                    .is_some_and(|p| p.equip_slot_enabled(slot))
            })
    };
    let candidate = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .copied()
        .find(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|c| c.kind.is_equip() && target_slot_enabled(tx, *card))
        });
    let mut events = Vec::new();
    if let Some(card) = candidate {
        // 获得 → 装备（上游 $gain2 + chooseUseTarget forced 的等价量）。
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::Illegal("target hand zone missing".into()))?;
        events.extend(move_cards(tx, vec![card], hand, Some(draw))?);
        crate::command::play::use_equipment_from_hand(tx, target, card)?;
    }
    // owner 对目标视为使用【杀】（CardKind::Sha tag=1；上游 isCard:true
    // 实体化杀的等价表达，完整结算管道）。
    if tx.state().players.get(target).is_some_and(|p| p.alive) {
        events.extend(run_effects(
            tx,
            vec![EffectRequest::UseVirtualCardOnSelectedTargets {
                player,
                kind: crate::state::CardKind::Sha.tag(),
                targets: vec![target],
            }],
        )?);
    }
    Ok(events)
}

/// 「巧织」gaintag（content_card_tags，对应上游 gaintag "olqiaozhi"）。
pub(crate) const OLQIAOZHI_TAG: u32 = 1_909_178_934;

const RESOLVE_OLQIAOZHI_MAGIC: u32 = 0x515A_0001;

/// 专属状态机 (批 296, ADR 0641 §8.80)：巧织——弃一张牌，亮牌堆顶两张
/// 并获得其中一张（所得打「巧织」标记；手牌有巧织牌时不可发动）。
/// stage 0=弃成本（强制 1..1 he）→ 1=弃置+亮顶 2 选 1 → 2=所得打标入手
/// 牌、未选牌入弃牌堆（上游 cardsGotoOrdering 语义）。
pub(crate) fn resolve_olqiaozhi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            // 手牌有巧织牌 → 不可发动（上游 filter 的 gaintag 门控）。
            let owns_tagged = tx
                .state()
                .hand_cards(player)
                .iter()
                .any(|card| {
                    tx.state()
                        .content_card_tags
                        .get(card)
                        .is_some_and(|tags| tags.contains(&OLQIAOZHI_TAG))
                });
            if owns_tagged {
                return Ok(Vec::new());
            }
            let options: Vec<u32> = crate::query::discardable_hand_equip_cards(tx.state(), player)
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
                1,
                false,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveOlqiaozhiStage,
                vec![RESOLVE_OLQIAOZHI_MAGIC, 1, player.raw()],
            )
        }
        _ => Err(EffectError::ContentFault("bad olqiaozhi stage".into())),
    }
}

/// 专属状态机 (批 302, ADR 0641 §8.86)：dczhee——forced。mode 0=入场/
/// 开局从牌堆顶向下检索；1=每名角色结束阶段从弃牌堆检索。找到第一张
/// 【闪电】且 owner 判定区无闪电 → 挂为判定牌（MoveCards 到 Judge 区，
/// 引擎判定管道自然结算）。canAddJudge 以「判定区无同名延迟锦囊」近似。
pub(crate) fn resolve_dczhee(
    tx: &mut Transaction,
    player: PlayerId,
    mode: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let judge = tx
        .state()
        .zones
        .find(ZoneKind::Judge, player)
        .ok_or_else(|| EffectError::Illegal("judge zone missing".into()))?;
    if tx
        .state()
        .zones
        .get(judge)
        .unwrap()
        .cards
        .iter()
        .any(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|c| c.kind == CardKind::TrickShandian)
        })
    {
        return Ok(Vec::new());
    }
    let zone_kind = if mode == 1 {
        ZoneKind::DiscardPile
    } else {
        ZoneKind::DrawPile
    };
    let Some(zone) = tx.state().zones.find(zone_kind, PlayerId::NONE) else {
        return Ok(Vec::new());
    };
    let candidate = tx
        .state()
        .zones
        .get(zone)
        .unwrap()
        .cards
        .iter()
        .rev()
        .copied()
        .find(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|c| c.kind == CardKind::TrickShandian)
        });
    let Some(card) = candidate else {
        return Ok(Vec::new());
    };
    let from = tx
        .state()
        .cards
        .get(card)
        .map(|card_state| engine_api::ZoneId::new(card_state.zone.raw()));
    move_cards(tx, vec![card], judge, from)
}

const RESOLVE_ZHONGJIE_MAGIC: u32 = 0x5A4A_0002;

/// 专属状态机 (批 304, ADR 0641 §8.88)：张介——死亡时（forceDie 语义，
/// owner 已 dead 仍可发动）选一名其他角色：其 +1 体力上限、回复 1 体力、
/// 摸一张牌。stage 0=选目标（可取消）→ 1=三连增益。
pub(crate) fn resolve_zhongjie(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| *candidate != player)
        .map(PlayerId::raw)
        .collect();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveZhongjieStage,
        vec![RESOLVE_ZHONGJIE_MAGIC, 1, player.raw()],
    )
}

/// 专属状态机 (批 305, ADR 0641 §8.90)：粗遐/未命孪生——forced。
/// ①使用牌者随机弃 1 手牌（RuleRng shuffle 工艺）；②条件仍成立则
/// owner 摸 1（starweiming 反转条件）。单发无续体。
/// 登记债：round 级伤害历史以 lifetime damage_history 放宽。
pub(crate) fn resolve_starcuxia_weiming(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    variant: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(source)
        .is_some_and(|participant| participant.alive)
        || tx.state().hand_count(source) == 0
    {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    // ① 随机弃 1 手牌。
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, source)
        .ok_or_else(|| EffectError::Illegal("source hand zone missing".into()))?;
    let mut candidates: Vec<CardId> = tx
        .state()
        .zones
        .get(hand)
        .unwrap()
        .cards
        .iter()
        .copied()
        .collect();
    let mut rng = RuleRng::from_state(tx.state().rng);
    rng.shuffle(&mut candidates);
    tx.journal(JournalEntry::RngCheckpoint);
    tx.state_mut().rng = rng.into_state();
    let discarded = candidates[0];
    events.extend(run_effects(
        tx,
        vec![EffectRequest::DiscardCards {
            player: source,
            cards: vec![discarded],
        }],
    )?);
    // ② 条件复验（弃牌不影响 hp/伤害历史）→ owner 摸 1。
    let (source_hp, owner_hp) = {
        let s = tx.state();
        (
            s.players.get(source).map(|p| p.hp).unwrap_or(0),
            s.players.get(player).map(|p| p.hp).unwrap_or(0),
        )
    };
    let dominant = if variant == 0 {
        source_hp > owner_hp
    } else {
        owner_hp > source_hp
    };
    let retaliated = tx
        .state()
        .damage_history
        .contains(&(source, player));
    if dominant && retaliated {
        events.extend(crate::effect::draw_cards(tx, player, 1)?);
    }
    Ok(events)
}

/// 专属状态机 (批 303, ADR 0641 §8.87)：朝献——限定技（once_per_game）。
/// ①弃牌堆中全部点数为 3 的牌移入牌堆（随机插入以置牌堆底近似，zone
/// 内 reorder）；②依座位序收齐其他存活角色手牌中的 3 点牌（give 语义
/// 移入 owner 手牌区）。单发无续体。
pub(crate) fn resolve_dchaoxian(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = Vec::new();
    // ① 弃牌堆 3 点牌回牌堆底。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let threes: Vec<CardId> = tx
        .state()
        .zones
        .get(discard)
        .unwrap()
        .cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card_state| card_state.rank == 3)
        })
        .collect();
    if !threes.is_empty() {
        let from = tx
            .state()
            .cards
            .get(threes[0])
            .map(|card_state| engine_api::ZoneId::new(card_state.zone.raw()));
        events.extend(move_cards(tx, threes.clone(), draw, from)?);
        // zone 内 reorder：移到 cards 头部 = 牌堆底（上游随机插入近似）。
        let zone_cards = &mut tx.state_mut().zones.get_mut(draw).unwrap().cards;
        let len = zone_cards.len();
        let tail: Vec<CardId> = zone_cards.drain(len - threes.len()..).collect();
        for card in tail.iter().rev() {
            zone_cards.insert(0, *card);
        }
        let _ = zone_cards;
    }
    // ② 逐其他存活角色收 3 点手牌。
    for target in tx.state().players.alive_by_seat() {
        if target == player {
            continue;
        }
        let hand = match tx.state().zones.find(ZoneKind::Hand, target) {
            Some(hand) => hand,
            None => continue,
        };
        let picks: Vec<CardId> = tx
            .state()
            .zones
            .get(hand)
            .unwrap()
            .cards
            .iter()
            .copied()
            .filter(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card_state| card_state.rank == 3)
            })
            .collect();
        if picks.is_empty() {
            continue;
        }
        let from = tx
            .state()
            .cards
            .get(picks[0])
            .map(|card_state| engine_api::ZoneId::new(card_state.zone.raw()));
        let owner_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("owner hand zone missing".into()))?;
        events.extend(run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards: picks,
                to: owner_hand,
                from,
            }],
        )?);
    }
    Ok(events)
}

/// 专属状态机 (批 307, ADR 0641 §8.91)：三首——可选（上游 damageBegin4
/// confirm）。亮牌堆顶三张：若存在类型组（基本/锦囊/装备）未被本局任何
/// 使用记录收录的牌，取消本次伤害（PreventDamage，damage_before 窗口内）。
/// 亮出的牌移入弃牌堆（上游 cardsGotoOrdering 收尾近似）。
pub(crate) fn resolve_sanshou(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile missing".into()))?;
    let mask = tx.state().lifetime_used_type_groups;
    let three: Vec<CardId> = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .rev()
        .copied()
        .take(3)
        .collect();
    if three.is_empty() {
        return Ok(Vec::new());
    }
    let has_new = three.iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|c| mask & (1u8 << c.kind.card_type_group().min(2)) == 0)
    });
    let mut events = Vec::new();
    if has_new {
        // 机器侧不走 vm_host 的 damage_id=0 重写，直接读当前伤害 id。
        let damage_id = tx.state().resolution.current_damage_id;
        if damage_id == 0 {
            return Err(EffectError::ContentFault(
                "sanshou prevent outside damage window".into(),
            ));
        }
        events.extend(run_effects(
            tx,
            vec![EffectRequest::PreventDamage { damage_id }],
        )?);
    }
    // 亮出的牌移入弃牌堆（cardsGotoOrdering 收尾近似）。
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
    let from = tx
        .state()
        .cards
        .get(three[0])
        .map(|card_state| engine_api::ZoneId::new(card_state.zone.raw()));
    events.extend(move_cards(tx, three, discard, from)?);
    Ok(events)
}

const RESOLVE_NHLUNDAO_MAGIC: u32 = 0x4C44_0001;

/// 专属状态机 (批 308, ADR 0641 §8.93)：论道——受伤后若伤害来源存活且
/// 手牌数与 owner 不同：owner 手牌多则摸 1；否则 owner 从来源 he 区弃
/// 一张（上游 discardPlayerCard forced，owner 视角选牌）。
/// stage 0=gate+分支 → 摸牌直结 / 开选牌续体 → 1=弃置。
pub(crate) fn resolve_nhlundao(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if source == player
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive)
        || tx.state().hand_count(source) == tx.state().hand_count(player)
    {
        return Ok(Vec::new());
    }
    if tx.state().hand_count(player) > tx.state().hand_count(source) {
        return crate::effect::draw_cards(tx, player, 1);
    }
    // owner 手牌不多于来源 → owner 从来源 he 区弃一张（强制选牌）。
    let mut options: Vec<u32> = Vec::new();
    for kind in [ZoneKind::Hand, ZoneKind::Equip] {
        if let Some(zone) = tx.state().zones.find(kind, source) {
            options.extend(
                tx.state()
                    .zones
                    .get(zone)
                    .unwrap()
                    .cards
                    .iter()
                    .map(|card| card.raw()),
            );
        }
    }
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveNhlundaoStage,
        vec![
            RESOLVE_NHLUNDAO_MAGIC,
            1,
            player.raw(),
            source.raw(),
        ],
    )
}

/// 专属状态机 (批 309, ADR 0641 §8.94)：襄戍——限定技（once_per_game）。
/// 结束阶段，若本回合造成过伤害：令一名受伤角色回复 X 点体力并摸 X 张
/// 牌（X=min(5, 本回合造成伤害事件数，damage_dealt_count 机内直读)）。
/// 单发无续体。
pub(crate) fn resolve_xiangshu(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(target) = targets.first().copied() else {
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
    let dealt = tx.state().turn.damage_dealt_count(player);
    if dealt == 0 {
        return Ok(Vec::new());
    }
    let num = dealt.min(5);
    let mut events = Vec::new();
    let hp = tx
        .state()
        .players
        .get(target)
        .map(|participant| participant.hp)
        .unwrap_or(0);
    let max_hp = tx
        .state()
        .players
        .get(target)
        .map(|participant| participant.max_hp)
        .unwrap_or(0);
    if (i64::from(max_hp) - i64::from(hp)) > 0 {
        let recover = i64::from(max_hp) - i64::from(hp);
        let amount = u32::try_from(recover.min(i64::from(num))).unwrap_or(0);
        if amount > 0 {
            events.extend(recover_hp(tx, player, target, amount)?);
        }
    }
    events.extend(draw_cards(tx, target, num)?);
    Ok(events)
}



/// 姝勇可选取牌区：手牌+装备+判定（上游 hej）。
pub(crate) fn meiyong_hej_cards(tx: &Transaction, target: PlayerId) -> Vec<CardId> {
    let mut cards = Vec::new();
    for kind in [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge] {
        if let Some(zone) = tx.state().zones.find(kind, target) {
            cards.extend(tx.state().zones.get(zone).unwrap().cards.iter().copied());
        }
    }
    cards
}

/// 专属状态机 (批 277, ADR 0641 §8.65)：姝勇——使用/打出【杀】时，获得一名
/// 其他角色 hej 区的一张牌（owner 选牌），然后其摸 1。
/// stage 0=选目标（可取消）→ 1=选其区域一张牌（强制）→ 2=移牌+目标摸 1。
pub(crate) fn resolve_meiyong(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let candidates: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|candidate| {
            *candidate != player && !meiyong_hej_cards(tx, *candidate).is_empty()
        })
        .map(PlayerId::raw)
        .collect();
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        candidates,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveMeiyongStage,
        vec![RESOLVE_MEIYONG_MAGIC, 1, player.raw()],
    )
}

const RESOLVE_QIANGYONG_MAGIC: u32 = 0x5159_0001;

/// 羌勇防重复触发键：记录已处理过的 damage id（引擎对同一伤害可能重复
/// 交付 damage_before 事件）。
const RESOLVE_QIANGYONG_LAST_DAMAGE: u32 = 0x5159_0002;

/// 专属状态机 (批 278, ADR 0641 §8.66)：羌勇——锁定技，【杀】造成伤害时
/// 弃置受伤角色的 X 张牌（X=本回合 owner 用【杀】次数，SHA_PLAYED 旗），
/// 然后若其没有手牌，此伤害 +1。
/// stage 0=owner 从目标 he 区选 X 张（X>0 且有可弃时）→ 1=弃牌结算；
/// 无可弃或 X=0 直接做 +1 判定。
pub(crate) fn resolve_qiangyong(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 同一伤害只结算一次（引擎可能重复交付 damage_before）。
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id != 0
        && tx.state().skills.storage_get(skill, RESOLVE_QIANGYONG_LAST_DAMAGE) == damage_id
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .storage_set(skill, RESOLVE_QIANGYONG_LAST_DAMAGE, damage_id);
    let x = tx.state().turn.flag(crate::state::turn_flag::SHA_PLAYED).max(0) as u32;
    if x == 0 {
        qiangyong_bonus_if_no_hand(tx, player, target, skill)?;
        return Ok(Vec::new());
    }
    let options: Vec<u32> = crate::query::discardable_hand_equip_cards(tx.state(), target)
        .into_iter()
        .map(CardId::raw)
        .collect();
    if options.is_empty() {
        qiangyong_bonus_if_no_hand(tx, player, target, skill)?;
        return Ok(Vec::new());
    }
    let count = x.min(options.len() as u32) as u8;
    open_choice(
        tx,
        player,
        count,
        count,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveQiangyongStage,
        vec![
            RESOLVE_QIANGYONG_MAGIC,
            1,
            player.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
}

/// 目标无手牌 → 当前伤害 +1（上游 trigger.num++）。
pub(crate) fn qiangyong_bonus_if_no_hand(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<(), EffectError> {
    if tx.state().hand_count(target) == 0 {
        run_effects(
            tx,
            vec![EffectRequest::ModifyCurrentDamage {
                player,
                delta: 1,
                skill,
            }],
        )?;
    }
    Ok(())
}

const RESOLVE_MBZHENANN_MAGIC: u32 = 0x5A4E_0001;

/// 专属状态机 (批 276, ADR 0641 §8.64)：镇南——成为其他角色使用的锦囊目标
/// （目标数 > max(1, 使用者体力)）时，可弃一张牌并对任意角色造成 1 点伤害。
/// stage 0=选成本牌（可取消）→ 1=弃牌+选伤害目标（任意存活角色）→
/// 2=弃牌结算+伤害。
pub(crate) fn resolve_mbzhenann(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
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
        ContinuationKind::ResolveMbzhenannStage,
        vec![RESOLVE_MBZHENANN_MAGIC, 1, player.raw()],
    )
}

const RESOLVE_DCANXU_MAGIC: u32 = 0x4443_0001;

/// 专属状态机 (批 273, ADR 0641 §8.61)：暗蓄——出牌阶段限一次，选两名手牌
/// 数不同的角色，手牌少者从多者处选得一张手牌；所得牌非黑桃则 owner 摸 1，
/// 此后两人手牌数相等则 owner 回 1。stage 0=少者从多者手牌选 1 →
/// 1=移牌+花色/相等结算。
pub(crate) fn resolve_dcanxu(
    tx: &mut Transaction,
    player: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    if targets.len() != 2 || targets[0] == targets[1] {
        return Err(EffectError::Illegal(
            "resolve-dcanxu requires exactly two distinct targets".into(),
        ));
    }
    let (a, b) = (targets[0], targets[1]);
    let (a_count, b_count) = (tx.state().hand_count(a), tx.state().hand_count(b));
    if a_count == b_count {
        return Err(EffectError::Illegal(
            "resolve-dcanxu targets must differ in hand count".into(),
        ));
    }
    let (gainner, giver) = if a_count < b_count { (a, b) } else { (b, a) };
    let options: Vec<u32> = tx
        .state()
        .hand_cards(giver)
        .iter()
        .map(|card| card.raw())
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        gainner,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ResolveDcanxuStage,
        vec![
            RESOLVE_DCANXU_MAGIC,
            player.raw(),
            gainner.raw(),
            giver.raw(),
            1,
        ],
    )
}

const RESOLVE_DUODUAN_MAGIC: u32 = 0x4444_0001;

/// 专属状态机 (批 272, ADR 0641 §8.60)：度断——每回合一次，成为【杀】目标
/// 时可重铸一张牌；再选一项：①使用者摸 2 且此杀对 owner 无效；②使用者弃
/// 一张牌且 owner 不可响应此杀。stage 0=选重铸牌 → 1=重铸+双选（使用者无
/// 可弃牌时强制①）→ 2=分支执行/选使用者弃牌 → 3=使用者弃牌+不可响应。
pub(crate) fn resolve_duoduan(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
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
        ContinuationKind::ResolveDuoduanStage,
        vec![
            RESOLVE_DUODUAN_MAGIC,
            player.raw(),
            source.raw(),
            card.raw(),
            skill.raw(),
            1,
        ],
    )
}

/// 选项①：使用者摸 2 张，此【杀】对 owner 无效。排除在解析期内同步生效
/// （直接改 sha 挂起载荷；经 CancelIncomingCardUse 尾随提交会在响应询问
/// 之后才落地，且附带「获得实体牌」的杜达语义，与度断不符）。
pub(crate) fn resolve_duoduan_draw_and_cancel(
    tx: &mut Transaction,
    source: PlayerId,
    player: PlayerId,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = crate::effect::draw_cards(tx, source, 2)?;
    crate::command::play::exclude_saved_card_target(tx, card, player)?;
    Ok(events)
}

const RESOLVE_GAOYUAN_MAGIC: u32 = 0x4747_0001;

/// 告援候选：非 owner、非使用者的存活角色，持「诤」标，且为此【杀】在
/// 使用者射程内的合法目标（转移语义 targetEnabled）。
pub(crate) fn gaoyuan_redirect_candidates(
    tx: &Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
) -> Result<Vec<PlayerId>, EffectError> {
    let legal = crate::command::play::saved_sha_redirect_targets_in_attack_range(
        tx,
        source,
        card,
        player,
        source,
    )?;
    Ok(legal
        .into_iter()
        .filter(|candidate| {
            *candidate != source
                && tx
                    .state()
                    .players
                    .get(*candidate)
                    .is_some_and(|participant| {
                        participant
                            .marks
                            .iter()
                            .any(|(mark, count)| {
                                *mark == crate::effect::ZHENGJIAN_MARK && *count > 0
                            })
                    })
        })
        .collect())
}

/// 专属状态机 (批 271, ADR 0641 §8.59)：告援——成为【杀】目标时，可弃一张
/// 牌，将此【杀】转移给一名有「诤」标且为此杀合法目标的其他角色。
/// stage 0：选弃牌（0..1 可取消）→ stage 1：选转移目标（1..1 可取消）→
/// stage 2：弃牌 + redirect_saved_sha_target + CardTargetAdded。
pub(crate) fn resolve_gaoyuan(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let candidates = gaoyuan_redirect_candidates(tx, player, source, card)?;
    if options.is_empty() || candidates.is_empty() {
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
        ContinuationKind::ResolveGaoyuanStage,
        vec![
            RESOLVE_GAOYUAN_MAGIC,
            player.raw(),
            source.raw(),
            card.raw(),
            1,
        ],
    )
}

const RESOLVE_MBBEIMING_MAGIC: u32 = 0x4242_0001;

/// 专属状态机 (批 269, ADR 0641 §8.57)：孛明——游戏开始时，令至多两名角色各
/// 获得牌堆中一张攻击范围等于其手牌花色数的武器牌（无 distance 信息的武器
/// 视作范围 1）。
pub(crate) fn resolve_mbbeiming(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let candidates: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if candidates.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                2,
                true,
                candidates,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveMbbeimingStage,
                vec![RESOLVE_MBBEIMING_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        _ => Ok(Vec::new()),
    }
}

/// stage 2（解析器直调）：对每个选定目标，从牌堆搜索攻击范围 == 其手牌花色数
/// 的武器牌并置入其手牌区；找不到则跳过该目标。
pub(crate) fn resolve_mbbeiming_grant(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    targets: &[u32],
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = player;
    let _ = skill;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("draw pile is missing".into()))?;
    let mut events = Vec::new();
    for raw in targets {
        let target = PlayerId::new(*raw);
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        {
            continue;
        }
        let suits_count = {
            let mut seen: [bool; 4] = [false; 4];
            for card in tx.state().hand_cards(target) {
                if let Some(card) = tx.state().cards.get(card) {
                    match card.suit {
                        Suit::Spade => seen[0] = true,
                        Suit::Heart => seen[1] = true,
                        Suit::Club => seen[2] = true,
                        Suit::Diamond => seen[3] = true,
                        Suit::None => {}
                    }
                }
            }
            seen.into_iter().filter(|seen| *seen).count()
        };
        let equip = tx
            .state()
            .zones
            .get(draw)
            .and_then(|zone| {
                zone.cards.iter().rev().copied().find(|card| {
                    tx.state().cards.get(*card).is_some_and(|card| {
                        card.kind == CardKind::Weapon
                            && u32::from(card.weapon_range.max(1)) == suits_count as u32
                    })
                })
            });
        let Some(equip) = equip else {
            continue;
        };
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, target)
            .ok_or_else(|| EffectError::Illegal("no target hand".into()))?;
        events.extend(move_cards(tx, vec![equip], hand, Some(draw))?);
    }
    Ok(events)
}

const RESOLVE_ZHENGJIAN_MAGIC: u32 = 0x5A4A_0001;

/// 「诤荐」用牌计数标记（mode 1/2 累计，mode 3 结算清空）。
pub(crate) const ZHENGJIAN_MARK: u32 = 2_446_053_157;

/// 专属状态机 (批 267, ADR 0641 §8.55)：诤荐——mode 0：结束阶段选一名角色
/// 授一枚「诤」标记；mode 1/2：该角色用牌/打出牌时标记 +1（机内直读直加，
/// 无选择）；mode 3：owner 回合开始，各有标角色摸 min(5, 上限, 标数) 张并
/// 清标。
pub(crate) fn resolve_zhengjian(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
    event_player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let targets: Vec<u32> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(PlayerId::raw)
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                targets,
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveZhengjianStage,
                vec![RESOLVE_ZHENGJIAN_MAGIC, player.raw(), skill.raw(), 0],
            )
        }
        1 | 2 => {
            // 用牌/打出计数：目标带标则标数 +1（无选择，inline）。
            if event_player != PlayerId::NONE
                && tx
                    .state()
                    .players
                    .get(event_player)
                    .is_some_and(|p| p.marks.iter().any(|(m, c)| *m == ZHENGJIAN_MARK && *c > 0))
            {
                events_extend_add_mark(tx, event_player, ZHENGJIAN_MARK, 1)?;
            }
            Ok(Vec::new())
        }
        3 => {
            // owner 回合开始：各有标角色摸 min(5, 上限, 标数) 张并清标。
            let mut events = Vec::new();
            for participant in tx.state().players.alive_by_seat() {
                let marks = tx
                    .state()
                    .players
                    .get(participant)
                    .and_then(|p| p.marks.iter().find(|(m, _)| *m == ZHENGJIAN_MARK))
                    .map_or(0, |(_, c)| *c);
                if marks <= 0 {
                    continue;
                }
                let draw_n = marks.min(5).max(0) as u32;
                if draw_n > 0 {
                    events.extend(crate::effect::draw_cards(tx, participant, draw_n)?);
                }
                if let Some(p) = tx.state_mut().players.get_mut(participant) {
                    p.marks.retain(|(m, _)| *m != ZHENGJIAN_MARK);
                }
            }
            Ok(events)
        }
        _ => Ok(Vec::new()),
    }
}

fn events_extend_add_mark(
    tx: &mut Transaction,
    player: PlayerId,
    mark: u32,
    delta: i32,
) -> Result<(), EffectError> {
    crate::effect::add_mark(tx, player, mark, delta)?;
    Ok(())
}

const RESOLVE_LIANGYIN_MAGIC: u32 = 0x524C_0001;

/// 专属状态机 (批 268, ADR 0641 §8.56)：良姻——①牌被置于武将牌上时（cards_
/// moved 入 expansion 区），你可令一名角色摸一张牌；②牌从武将牌上被获得时
/// （cards_moved 出 expansion 区），你可令一名角色弃置一张手牌。
pub(crate) fn resolve_reliangyin(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<u32> = match mode {
        0 => tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(PlayerId::raw)
            .collect(),
        1 => tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|&candidate| !tx.state().hand_cards(candidate).is_empty())
            .map(PlayerId::raw)
            .collect(),
        _ => Vec::new(),
    };
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ResolveReliangyinStage,
        vec![RESOLVE_LIANGYIN_MAGIC, player.raw(), skill.raw(), mode],
    )
}

/// 续体：selections[0] = 受益/弃牌角色。
pub(crate) fn resolve_reliangyin_stage(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    mode: u32,
    who_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let who = PlayerId::new(who_raw);
    match mode {
        0 => crate::effect::draw_cards(tx, who, 1),
        1 => {
            // 该角色弃置一张手牌（其自选，强制）。
            let hand: Vec<u32> = tx
                .state()
                .hand_cards(who)
                .iter()
                .map(|card| card.raw())
                .collect();
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let payload = vec![RESOLVE_LIANGYIN_MAGIC, who.raw(), 0, 9];
            open_choice(
                tx,
                who,
                1,
                1,
                false,
                hand,
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveReliangyinStage,
                payload,
            )?;
            Ok(Vec::new())
        }
        _ => Ok(Vec::new()),
    }
}

/// mode 1 子续体：who 选定的手牌弃置。
pub(crate) fn resolve_reliangyin_discard(
    tx: &mut Transaction,
    who: PlayerId,
    card_raw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let card = CardId::new(card_raw);
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, who)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    if !tx
        .state()
        .zones
        .get(hand)
        .unwrap()
        .cards
        .contains(&card)
    {
        return Err(EffectError::Illegal(
            "liangyin card outside hand".into(),
        ));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    if let Some(card_state) = tx.state_mut().cards.get_mut(card) {
        card_state.zone = discard;
    }
    Ok(Vec::new())
}
