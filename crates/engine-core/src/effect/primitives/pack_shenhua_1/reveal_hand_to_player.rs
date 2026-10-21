//! Effect primitives for `reveal_hand_to_player` and related helpers.

use super::*;

pub(crate) fn reveal_hand_to_player(
    tx: &Transaction,
    player: PlayerId,
    observer: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ![player, observer].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    }) {
        return Err(EffectError::Illegal(
            "hand reveal participants are not alive".into(),
        ));
    }
    Ok(vec![DomainEvent::CardsRevealed {
        cards: tx.state().hand_cards(player),
        to: vec![observer],
    }])
}

pub(crate) fn reveal_hand(
    tx: &Transaction,
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
    let cards = tx.state().hand_cards(player);
    Ok((!cards.is_empty())
        .then_some(DomainEvent::CardsRevealed {
            cards,
            to: Vec::new(),
        })
        .into_iter()
        .collect())
}

pub(crate) fn reveal_target_hand_sha_or_discard_bind(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
    stage: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target || stage > 3 {
        return Err(EffectError::ContentFault(
            "Weikui branch has invalid target or stage".into(),
        ));
    }
    let participants_alive = [player, target].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    });
    if !participants_alive {
        return Ok(Vec::new());
    }

    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Err(EffectError::ContentFault(
                    "Weikui branch requires its owner's enabled skill".into(),
                ));
            }
            if tx.state().hand_cards(target).is_empty() {
                return Err(EffectError::ContentFault(
                    "Weikui target must have a hand card before the HP cost".into(),
                ));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::RevealTargetHandShaOrDiscardBind {
                    player,
                    target,
                    helper_skill_def,
                    skill,
                    stage: 1,
                },
            );
            super::super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: player,
                    amount: 1,
                    is_lose_hp: true,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .map(|_| Vec::new())
        }
        1 => {
            let hand = tx.state().hand_cards(target);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let has_shan = hand.into_iter().any(|card| {
                tx.state().cards.get(card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), target, card) == CardKind::Shan
                })
            });
            if !has_shan {
                return choose_discard_target_card(
                    tx,
                    player,
                    target,
                    rule_api::TargetCardScope::Hand,
                );
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::RevealTargetHandShaOrDiscardBind {
                    player,
                    target,
                    helper_skill_def,
                    skill,
                    stage: 2,
                },
            );
            reveal_hand_to_player(tx, target, player)
        }
        2 => {
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::RevealTargetHandShaOrDiscardBind {
                    player,
                    target,
                    helper_skill_def,
                    skill,
                    stage: 3,
                },
            );
            try_use_virtual_sha_on_without_distance(tx, player, target, false, 0)
        }
        3 => add_bound_skill(tx, player, helper_skill_def, target),
        _ => unreachable!(),
    }
}

pub(crate) fn increase_current_draw_count(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().turn.current != player || tx.state().turn.phase != Phase::Draw {
        return Err(EffectError::ContentFault(
            "draw-count increase requires the current draw-phase player".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::ContentFault(
            "draw-count increase requires a living player".into(),
        ));
    }
    let current = tx
        .state()
        .turn
        .flag(turn_flag::CURRENT_DRAW_COUNT_BONUS)
        .max(0) as u32;
    let next = current.saturating_add(count).min(i32::MAX as u32);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::CURRENT_DRAW_COUNT_BONUS, next as i32);
    Ok(Vec::new())
}

pub(crate) fn skip_next_discard_phase(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    if state.turn.current != player
        || !state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "skip-next-discard-phase owner is no longer current and alive".into(),
        ));
    }
    if !matches!(
        state.turn.phase,
        Phase::Prepare | Phase::Judge | Phase::Draw | Phase::Play
    ) {
        return Err(EffectError::ContentFault(
            "skip_next_discard_phase is only valid before the Discard phase".into(),
        ));
    }
    tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 1);
    Ok(Vec::new())
}

pub(crate) fn resolve_dcyingtu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
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
    match stage {
        0 => {
            let neighbors = crate::query::alive_seat_neighbors(tx.state(), player);
            let target = if neighbors.contains(&target) {
                target
            } else {
                neighbors
                    .into_iter()
                    .find(|candidate| {
                        let hand = tx.state().zones.find(ZoneKind::Hand, *candidate);
                        cards.iter().any(|card| {
                            tx.state().cards.get(*card).is_some_and(|state| {
                                state.owner == *candidate && Some(state.zone) == hand
                            })
                        })
                    })
                    .unwrap_or(PlayerId::NONE)
            };
            if target.is_none() {
                return Ok(Vec::new());
            }
            let options = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            );
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
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DCYINGTU_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                    1,
                ],
            )
        }
        2 => {
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
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DCYINGTU_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    target.raw(),
                    2,
                ],
            )
        }
        _ => Err(EffectError::ContentFault("bad Dcyingtu stage".into())),
    }
}

pub(crate) fn resolve_dcxiangshu(
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
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    // Guard: the observed player's hand count >= their HP.
    let observed_hp = tx.state().players.get(target).map(|p| p.hp).unwrap_or(0);
    if tx.state().hand_count(target) < observed_hp as u32 {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = (0..=5).collect();
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
            RESOLVE_DCXIANG_SHU_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            target.raw(),
        ],
    )
}

pub(crate) fn resolve_dcxiangshu_lottery(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
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
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let number = tx
        .state()
        .players
        .get(player)
        .map(|p| {
            p.marks
                .iter()
                .find(|(mark, _)| *mark == DCXIANG_SHU_NUMBER_MARK)
                .map(|(_, count)| *count)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    if number == 0 {
        return Ok(Vec::new());
    }
    let hand = tx.state().hand_count(target) as i32;
    let delta = (hand - number).abs();
    let mut effects = vec![EffectRequest::RemoveMark {
        player,
        mark: engine_api::MarkDefId::new(DCXIANG_SHU_NUMBER_MARK),
        amount: number.max(0) as u32,
    }];
    if delta <= 1
        && !crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .is_empty()
    {
        let options: Vec<_> = crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .into_iter()
        .map(CardId::raw)
        .collect();
        crate::effect::run_effects(tx, effects.clone())?;
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            options,
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_DCXIANG_SHU_CUSTOM_CHOICE,
                player.raw(),
                engine_api::SkillId::NONE.raw(),
                target.raw(),
                1,
                delta as u32,
            ],
        );
    }
    if delta == 0 {
        effects.push(EffectRequest::DealDamage {
            source: player,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        });
    }
    run_effects(tx, effects)
}

/// Cards one player lost during their own Discard phase that are still in the discard pile.
/// Requires the ABI 513 slice base; yields nothing when no Discard phase was entered.
pub(crate) fn discard_phase_discards_for(
    state: &crate::state::GameState,
    player: PlayerId,
) -> impl Iterator<Item = CardId> + '_ {
    let base = state.turn.flag(turn_flag::DISCARD_PHASE_LOST_BASE).max(0) as usize;
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    state
        .turn
        .lost_cards
        .iter()
        .skip(base.saturating_sub(1))
        .filter(move |(loser, _)| *loser == player)
        .filter_map(move |(_, card)| {
            state.cards.get(*card).and_then(|card_state| {
                discard
                    .is_some_and(|zone| card_state.zone == zone)
                    .then_some(*card)
            })
        })
}

/// Physical cards the player discarded during the current Discard phase, including cards that
/// nested triggers subsequently moved out of the discard pile.
pub(crate) fn discard_phase_discarded_cards_for(
    state: &crate::state::GameState,
    player: PlayerId,
) -> impl Iterator<Item = CardId> + '_ {
    let base = state.turn.flag(turn_flag::DISCARD_PHASE_LOST_BASE).max(0) as usize;
    state
        .turn
        .lost_cards
        .iter()
        .skip(base.saturating_sub(1))
        .filter(move |(loser, card)| *loser == player && state.turn.discarded_cards.contains(card))
        .map(|(_, card)| *card)
}

/// Every card discarded into the discard pile during the current Discard phase (any player),
/// still present in the discard pile. Requires the ABI 513 slice base.
pub(crate) fn discard_phase_discards_all(
    state: &crate::state::GameState,
) -> impl Iterator<Item = CardId> + '_ {
    let base = state.turn.flag(turn_flag::DISCARD_PHASE_LOST_BASE).max(0) as usize;
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    state
        .turn
        .lost_cards
        .iter()
        .skip(base.saturating_sub(1))
        .filter_map(move |(_, card)| {
            state.cards.get(*card).and_then(|card_state| {
                discard
                    .is_some_and(|zone| card_state.zone == zone)
                    .then_some(*card)
            })
        })
}

/// 双雄（shenhua 颜良文丑）：摸牌阶段可选改为判定，获得判定牌并记录判定颜色，
/// 本回合非该颜色手牌可当【决斗】使用。ABI 511。
pub(crate) fn resolve_shuangxiong(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Draw
        || tx.state().turn.draw_phase_is_fixed()
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    // 双雄 replaces the pending normal draw with a judgment.
    tx.state_mut().turn.set_draw_phase_fixed_count(0);
    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let color = match result.suit {
        Suit::Spade | Suit::Club => 2,
        _ => 1,
    };
    let mut events = vec![completed_judgment_event(player, result, CardId::NONE)];
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Shuangxiong owner hand is missing".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Shuangxiong discard pile is missing".into()))?;
    events.extend(move_cards(tx, vec![result.card], hand, Some(discard))?);
    events.extend(add_mark(tx, player, SHUANGXIONG_COLOR_MARK, color)?);
    events.extend(add_skill_until_turn_end(
        tx,
        player,
        engine_api::SkillDefId::new(SHUANGXIONG_VIEW_AS_DEF),
    )?);
    Ok(events)
}

/// 直谏（shenhua 张昭张纮）：出牌阶段限一次，将一张手牌区装备牌交给一名可装备该牌的
/// 其他角色，然后摸一张。ABI 512。stage 0 选择目标，stage 1 执行赠予与摸牌。
pub(crate) fn resolve_zhijian(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    stage: u8,
    target: PlayerId,
    skill: engine_api::SkillId,
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
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let slot = tx
        .state()
        .cards
        .get(card)
        .and_then(|card_state| card_state.kind.equip_slot());
    if stage == 0 {
        if !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card_state| hand.is_some_and(|zone| card_state.zone == zone))
        {
            return Ok(Vec::new());
        }
        let options: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|candidate| *candidate != player)
            .filter(|candidate| {
                tx.state()
                    .players
                    .get(*candidate)
                    .is_some_and(|candidate_state| {
                        candidate_state.alive
                            && slot.is_some_and(|s| candidate_state.equip_slot_enabled(s))
                    })
            })
            .map(|candidate| candidate.raw())
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
                RESOLVE_ZHIJIAN_CUSTOM_CHOICE,
                player.raw(),
                card.raw(),
                skill.raw(),
            ],
        );
    }
    if target.is_none()
        || target == player
        || !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card_state| hand.is_some_and(|zone| card_state.zone == zone))
        || !tx.state().players.get(target).is_some_and(|candidate| {
            candidate.alive && slot.is_some_and(|s| candidate.equip_slot_enabled(s))
        })
    {
        return Ok(Vec::new());
    }
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, target)
        .ok_or_else(|| EffectError::Illegal("Zhijian target equip zone is missing".into()))?;
    let mut events = give_cards(
        tx,
        vec![card],
        equip,
        hand.ok_or_else(|| EffectError::Illegal("Zhijian owner hand is missing".into()))?,
    )?;
    events.extend(run_effects(
        tx,
        vec![EffectRequest::DrawCards { player, count: 1 }],
    )?);
    Ok(events)
}

/// 固政（shenhua 张昭张纮）：其他角色弃牌阶段结束后，归还其一张该阶段弃置且仍处弃牌堆
/// 的牌，并可选获得其余弃牌。ABI 513。stage 0 选归还牌，stage 1 选是否取回剩余。
pub(crate) fn resolve_guzheng(
    tx: &mut Transaction,
    player: PlayerId,
    event_player: PlayerId,
    stage: u8,
    _cards: Vec<u32>,
    return_card: CardId,
    take_rest: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || event_player == player
        || !tx
            .state()
            .players
            .get(event_player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if stage == 0 {
        let options: Vec<_> = discard_phase_discards_for(tx.state(), event_player)
            .map(|card| card.raw())
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
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_GUZHENG_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                event_player.raw(),
                1, // stage: return-card chosen
            ],
        );
    }
    if stage == 1 {
        return open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![1, 0],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                RESOLVE_GUZHENG_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                event_player.raw(),
                2, // stage: take-rest chosen
                return_card.raw(),
            ],
        );
    }
    // stage 2: commit the return and optional take-rest.
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Guzheng discard pile is missing".into()))?;
    let still_in_discard = |tx: &Transaction, card: CardId| {
        tx.state()
            .cards
            .get(card)
            .is_some_and(|card_state| card_state.zone == discard)
    };
    if !still_in_discard(tx, return_card) {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, event_player)
        .ok_or_else(|| EffectError::Illegal("Guzheng target hand is missing".into()))?;
    let mut events = move_cards(tx, vec![return_card], hand, Some(discard))?;
    if take_rest {
        let owner_hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("Guzheng owner hand is missing".into()))?;
        let rest: Vec<_> = discard_phase_discards_all(tx.state())
            .filter(|card| *card != return_card)
            .collect();
        if !rest.is_empty() {
            events.extend(move_cards(tx, rest, owner_hand, Some(discard))?);
        }
    }
    Ok(events)
}

/// 制霸（孙策，主公技）：其他吴势力角色出牌阶段限一次可与拥有者拼点；拥有者未赢时
/// 可选择获得两张拼点牌，已觉醒时可先拒绝。ABI 523。
pub(crate) fn resolve_zhiba(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(actor)
            .is_some_and(|participant| participant.alive)
        || actor == player
        || tx.state().turn.current != actor
        || tx.state().turn.phase != Phase::Play
        || tx.state().player_group(actor) != Some("wu")
        || tx.state().hand_cards(player).is_empty()
        || tx.state().hand_cards(actor).is_empty()
    {
        return Ok(Vec::new());
    }
    // 已觉醒（魂姿/界魂姿均已移除）时，拥有者可先拒绝本次拼点。
    let awakened = !tx.state().skills.sorted_live().iter().any(|instance| {
        instance.owner == player
            && (instance.def.raw() == HUNZI_DEF || instance.def.raw() == REHUNZI_DEF)
    });
    if awakened {
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![1, 0],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                RESOLVE_ZHIBA_CUSTOM_CHOICE,
                player.raw(),
                actor.raw(),
                skill.raw(),
                0, // stage: refuse choice
            ],
        );
    }
    // 未觉醒：记录待决拼点并让发起方选择拼点牌（复用通用拼点选择续体）。
    tx.state_mut().skills.get_mut(skill).map(|instance| {
        instance.set_zhiba_compare(actor);
    });
    let options: Vec<_> = tx
        .state()
        .hand_cards(actor)
        .into_iter()
        .map(CardId::raw)
        .collect();
    open_choice(
        tx,
        actor,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseHandComparePlayerCard,
        vec![
            actor.raw(),
            player.raw(),
            engine_api::SkillDefId::NONE.raw(),
            engine_api::SkillDefId::NONE.raw(),
            skill.raw(),
        ],
    )
}

/// 制霸拼点完成结算：若拥有者未赢，可选择获得两张拼点牌。ABI 523。
pub(crate) fn resolve_zhiba_settle(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    winner: PlayerId,
    player_card: CardId,
    target_card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
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
    if stage == 0 {
        // 发起方（吴势力角色）获胜时没有取牌分支；平局或拥有者获胜时提供选择。
        if winner == actor {
            tx.state_mut().skills.get_mut(skill).map(|instance| {
                instance.clear_zhiba_compare();
            });
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![1, 0],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![
                RESOLVE_ZHIBA_CUSTOM_CHOICE,
                player.raw(),
                actor.raw(),
                skill.raw(),
                1, // stage: accept-gain choice
                winner.raw(),
                player_card.raw(),
                target_card.raw(),
            ],
        );
    }
    // stage 1: 拥有者选择获得两张拼点牌。
    tx.state_mut().skills.get_mut(skill).map(|instance| {
        instance.clear_zhiba_compare();
    });
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Zhiba discard pile is missing".into()))?;
    let still_in_discard = |tx: &Transaction, card: CardId| {
        tx.state()
            .cards
            .get(card)
            .is_some_and(|card_state| card_state.zone == discard)
    };
    if !still_in_discard(tx, player_card) && !still_in_discard(tx, target_card) {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Zhiba owner hand is missing".into()))?;
    let cards: Vec<_> = [player_card, target_card]
        .into_iter()
        .filter(|card| still_in_discard(tx, *card))
        .collect();
    move_cards(tx, cards, hand, Some(discard))
}

/// 从俭（张绣）：成为多目标锦囊牌的目标后，交给其中一名其他目标一张手牌/装备牌，
/// 然后摸一张（所给为装备牌则摸两张）。ABI 524。
pub(crate) fn resolve_drlt_congjian(
    tx: &mut Transaction,
    player: PlayerId,
    mut targets: Vec<PlayerId>,
    card: CardId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
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
    if stage == 0 {
        let options: Vec<_> = targets
            .into_iter()
            .filter(|candidate| {
                *candidate != player
                    && tx
                        .state()
                        .players
                        .get(*candidate)
                        .is_some_and(|participant| participant.alive)
            })
            .map(|candidate| candidate.raw())
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
                RESOLVE_DRLT_CONGJIAN_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                0, // stage: recipient choice
            ],
        );
    }
    if stage == 1 {
        let options: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .map(CardId::raw)
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
            ChoiceKind::SelectCards,
            ContinuationKind::Custom,
            vec![
                RESOLVE_DRLT_CONGJIAN_CUSTOM_CHOICE,
                player.raw(),
                skill.raw(),
                1, // stage: card choice
                target.raw(),
            ],
        );
    }
    // stage 2: 赠予并摸牌。
    if target.is_none() || target == player {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Congjian owner hand is missing".into()))?;
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("Congjian target hand is missing".into()))?;
    let is_equip = tx
        .state()
        .cards
        .get(card)
        .is_some_and(|card_state| card_state.kind.is_equip());
    let mut events = give_cards(tx, vec![card], target_hand, hand)?;
    events.extend(run_effects(
        tx,
        vec![EffectRequest::DrawCards {
            player,
            count: if is_equip { 2 } else { 1 },
        }],
    )?);
    Ok(events)
}

/// 飞军（王平）：出牌阶段限一次弃一张牌，选择“手牌数大于你者交给你一张牌”或
/// “装备区牌数大于你者弃置一张装备牌”分支（两项均可用时二选一）；目标首次被选时
/// 置兵略旗标（摸两张由 nzry_binglve 于出牌阶段结束结算）。ABI 526。
/// 庸肆（袁术）：摸牌阶段把摸牌数固定为存活势力数；出牌阶段结束时按本回合是否造成
/// 伤害：未造成伤害摸至体力值，否则授予既有庸肆弃牌阶段状态（回合末移除）。
/// ABI 527。
pub(crate) fn resolve_drlt_yongsi(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if tx.state().turn.phase != Phase::Draw || tx.state().turn.draw_phase_is_fixed() {
                return Ok(Vec::new());
            }
            let groups = crate::query::alive_group_count(tx.state()).max(1) as u32;
            tx.state_mut().turn.set_draw_phase_fixed_count(groups);
            Ok(Vec::new())
        }
        _ => {
            if tx.state().turn.damage_dealt_count(player) == 0 {
                let hp = tx
                    .state()
                    .players
                    .get(player)
                    .map(|p| p.hp.max(0) as u32)
                    .unwrap_or(0);
                let hand = tx.state().hand_count(player);
                if hand < hp {
                    run_effects(
                        tx,
                        vec![EffectRequest::DrawCards {
                            player,
                            count: hp - hand,
                        }],
                    )
                } else {
                    Ok(Vec::new())
                }
            } else {
                add_skill_until_turn_end(tx, player, engine_api::SkillDefId::new(DRLT_YONGSI1_DEF))
            }
        }
    }
}

/// 伪帝（袁术，主公技）：弃牌阶段开始把至多 min(需弃置数, 其他群势力角色数) 张手牌
/// 分别交给等量其他群势力角色；给出的牌离开手牌后弃牌需求自然下降。ABI 528。
pub(crate) fn resolve_drlt_weidi(
    tx: &mut Transaction,
    player: PlayerId,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    stage: u8,
    skill: engine_api::SkillId,
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
    match stage {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Discard {
                return Ok(Vec::new());
            }
            let limit = crate::query::query(
                tx.state(),
                &rule_api::QueryRequest::HandLimit { player },
                &mut crate::query::QueryContext::new(),
            );
            let limit = match limit {
                rule_api::QueryResponse::Int(value) => value.max(0) as u32,
                _ => return Ok(Vec::new()),
            };
            let hand = tx.state().hand_count(player);
            let need = hand.saturating_sub(limit);
            let qun_others: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player && tx.state().player_group(*candidate) == Some("qun")
                })
                .collect();
            let num = need.min(qun_others.len() as u32);
            if num == 0 {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .hand_cards(player)
                .into_iter()
                .filter(|card| {
                    !crate::query::card_discard_protected_from(tx.state(), player, player, *card)
                })
                .map(CardId::raw)
                .collect();
            if (options.len() as u32) < 1 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                num.min(u8::MAX as u32) as u8,
                true,
                options,
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE,
                    player.raw(),
                    skill.raw(),
                    0, // stage: card choice
                    qun_others.len() as u32,
                ],
            )
        }
        1 => {
            // targets choice; cards come from the payload.
            if cards.is_empty() || cards.len() > u8::MAX as usize {
                return Ok(Vec::new());
            }
            let qun_others: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| {
                    *candidate != player && tx.state().player_group(*candidate) == Some("qun")
                })
                .collect();
            if qun_others.len() < cards.len() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                cards.len() as u8,
                cards.len() as u8,
                true,
                qun_others
                    .into_iter()
                    .map(|candidate| candidate.raw())
                    .collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                {
                    let mut payload = vec![
                        RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE,
                        player.raw(),
                        skill.raw(),
                        1, // stage: target choice
                        cards.len() as u32,
                    ];
                    payload.extend(cards.iter().map(|card| card.raw()));
                    payload
                },
            )
        }
        _ => {
            // stage 2: give card[i] to target[i].
            if cards.len() != targets.len() || cards.is_empty() {
                return Ok(Vec::new());
            }
            let mut events = Vec::new();
            for (card, target) in cards.into_iter().zip(targets.into_iter()) {
                if target.is_none() || target == player {
                    continue;
                }
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::Illegal("Weidi owner hand is missing".into()))?;
                let target_hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, target)
                    .ok_or_else(|| EffectError::Illegal("Weidi target hand is missing".into()))?;
                let from = tx
                    .state()
                    .cards
                    .get(card)
                    .map(|card_state| card_state.zone)
                    .ok_or_else(|| EffectError::Illegal("Weidi card missing".into()))?;
                if from != hand {
                    continue;
                }
                events.extend(give_cards(tx, vec![card], target_hand, hand)?);
            }
            Ok(events)
        }
    }
}

pub(crate) fn resolve_lieren(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    win_skill_def: engine_api::SkillDefId,
    lose_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 拼点需要双方都有手牌。
    if tx.state().hand_cards(target).is_empty() {
        return Ok(Vec::new());
    }
    let own_hand = tx.state().hand_cards(player);
    GuardedChoice::new(
        skill,
        Participants::distinct_pair(player, target),
        ContinuationKind::ChooseHandComparePlayerCard,
        vec![
            player.raw(),
            target.raw(),
            win_skill_def.raw(),
            lose_skill_def.raw(),
            skill.raw(),
        ],
    )
    .open_cards(tx, own_hand)?;
    Ok(Vec::new())
}

pub(crate) fn resolve_tuntian(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    let skill_def = skill_pile_def(tx, player, skill)?;
    let result = crate::command::judge::flip_judgment(
        tx,
        player,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let judgment_event = completed_judgment_event(player, result, engine_api::CardId::NONE);
    if result.suit == engine_api::Suit::Heart {
        return Ok(vec![judgment_event]);
    }
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    move_cards(tx, vec![result.card], pile, Some(discard))
        .map(|events| prepend_event(judgment_event, events))
}

pub(crate) fn resolve_zaoxian(
    tx: &mut Transaction,
    player: PlayerId,
    jixi_def: engine_api::SkillDefId,
    tuntian_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    if tx.state().skill_pile_cards(player, tuntian_def.raw()).len() < 3 {
        return Ok(Vec::new());
    }
    // 已觉醒（持有急袭）时不再重复结算。
    if tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.owner == player && instance.def == jixi_def)
    {
        return Ok(Vec::new());
    }
    run_effects(
        tx,
        vec![
            EffectRequest::ChangeMaxHp { player, delta: -1 },
            EffectRequest::AddSkill {
                player,
                skill_def: jixi_def,
            },
        ],
    )
}

pub(crate) fn resolve_tiaoxin(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            let in_range_with_cards: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&candidate| candidate != player)
                .filter(|&candidate| {
                    crate::query::player_in_attack_range(tx.state(), candidate, player)
                })
                .filter(|&candidate| {
                    !crate::query::discardable_hand_equip_cards(tx.state(), candidate).is_empty()
                })
                .collect();
            GuardedChoice::new(
                skill,
                Participants::owner(player),
                ContinuationKind::Custom,
                vec![
                    RESOLVE_TIAOXIN_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    PlayerId::NONE.raw(),
                    skill.raw(),
                ],
            )
            .during(TurnContext::OwnersPhase(Phase::Play))
            .open_targets(tx, in_range_with_cards)
        }
        _ => Err(stage_must_resume_via_continuation("挑衅")),
    }
}

pub(crate) fn resolve_zhiji(
    tx: &mut Transaction,
    player: PlayerId,
    guanxing_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || !tx.state().hand_cards(player).is_empty()
    {
        return Ok(Vec::new());
    }
    // 已觉醒（持有观星）时不再重复结算。
    if tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.owner == player && instance.def == guanxing_def)
    {
        return Ok(Vec::new());
    }
    run_effects(
        tx,
        vec![
            EffectRequest::DrawCards { player, count: 2 },
            EffectRequest::RecoverHp {
                source: PlayerId::NONE,
                player,
                amount: 1,
            },
            EffectRequest::ChangeMaxHp { player, delta: -1 },
            EffectRequest::AddSkill {
                player,
                skill_def: guanxing_def,
            },
        ],
    )
}

/// ABI 681 (batch 176): watch the top `count` cards and re-order them one by one —
/// each round the player picks top-sequence or bottom-sequence for the current card.
/// Multi-round state rides the continuation payload:
/// `[player, count, round, flipped_len, flipped..., top_len, top..., bottom_len, bottom..., skill]`.
pub(crate) fn guanxing_top(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
        || count == 0
    {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let mut flipped = Vec::new();
    for _ in 0..count {
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
            .and_then(|zone| zone.cards.last().copied())
        else {
            break;
        };
        move_one_card(tx, card, draw, processing, PlayerId::NONE, true)?;
        flipped.push(card);
    }
    if flipped.is_empty() {
        return Ok(Vec::new());
    }
    let payload = vec![player.raw(), count, 0, flipped.len() as u32]
        .into_iter()
        .chain(flipped.iter().copied().map(CardId::raw))
        .chain([0u32, 0u32])
        .chain(std::iter::once(skill.raw()))
        .collect::<Vec<_>>();
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::SelectTargets,
        ContinuationKind::GuanxingPick,
        payload,
    )
}

/// ABI 681 guanxing_top commit: re-seat the picked sequences onto the draw pile —
/// bottom sequence goes first (lowest), then remaining pile, then top sequence on top.
pub(crate) fn commit_guanxing_top(
    tx: &mut Transaction,
    player: PlayerId,
    top: &[CardId],
    bottom: &[CardId],
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let _ = skill;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    // 所有翻开的牌都在 processing；按 bottom（原序）→ 其余 → top（栈序）落位：
    // top 序列最后放置且每张压到牌堆顶（后放者更靠顶），bottom 序列最先放且压到
    // 牌堆顶（后放的 bottom 更靠顶——与「置于牌堆底」语义一致：底序列逆序垫底）。
    for card in bottom.iter() {
        crate::effect::move_one_card(tx, *card, processing, draw, PlayerId::NONE, true)?;
    }
    for card in top.iter().rev() {
        crate::effect::move_one_card(tx, *card, processing, draw, PlayerId::NONE, true)?;
    }
    Ok(Vec::new())
}
