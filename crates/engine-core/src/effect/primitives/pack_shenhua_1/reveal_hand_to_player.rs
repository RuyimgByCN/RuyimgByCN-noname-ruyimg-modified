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
