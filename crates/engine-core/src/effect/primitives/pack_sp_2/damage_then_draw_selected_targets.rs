//! Effect primitives for `resolve_shoulijian` and related helpers.

use super::*;

pub(crate) fn damage_then_draw_selected_targets(
    tx: &mut Transaction,
    source: PlayerId,
    targets: &[PlayerId],
    damage: u32,
    draw: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(source)
        .is_some_and(|player| player.alive)
        || targets.is_empty()
    {
        return Err(EffectError::ContentFault(
            "selected-target damage requires a living source and targets".into(),
        ));
    }
    let order = tx.state().players.alive_by_seat();
    let mut sorted = targets.to_vec();
    sorted.sort_by_key(|target| {
        order
            .iter()
            .position(|candidate| candidate == target)
            .unwrap_or(usize::MAX)
    });
    if sorted.iter().any(|target| !order.contains(target))
        || sorted.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err(EffectError::ContentFault(
            "selected-target damage requires distinct living targets".into(),
        ));
    }
    let effects = sorted
        .into_iter()
        .flat_map(|target| {
            [
                EffectRequest::DealDamage {
                    source,
                    target,
                    amount: damage,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                },
                EffectRequest::DrawCards {
                    player: target,
                    count: draw,
                },
            ]
        })
        .collect();
    super::super::run_effects(tx, effects).map(|_| Vec::new())
}

pub(crate) fn resolve_youdi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = |tx: &Transaction, player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    };
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !alive(tx, player)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::End
                || crate::query::discardable_hand_equip_cards(tx.state(), player).is_empty()
            {
                return Ok(Vec::new());
            }
            let options = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
                .map(PlayerId::raw)
                .collect::<Vec<_>>();
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
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveYoudi,
                vec![0, player.raw(), 0, skill.raw()],
            )
        }
        1 => {
            if player == target || !alive(tx, player) || !alive(tx, target) {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_target_cards(
                tx.state(),
                target,
                player,
                TargetCardScope::HandEquip,
            );
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveYoudi,
                vec![1, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            let legal = crate::query::discardable_target_cards(
                tx.state(),
                target,
                player,
                TargetCardScope::HandEquip,
            );
            if !legal.contains(&card) {
                return Err(EffectError::Illegal(
                    "Youdi owner card became illegal".into(),
                ));
            }
            let discarded_was_sha = tx.state().cards.get(card).is_some_and(|card| {
                crate::query::effective_card_kind(tx.state(), player, card).is_sha()
            });
            if !discarded_was_sha {
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(EffectRequest::ResolveYoudi {
                        player,
                        target,
                        card: CardId::NONE,
                        stage: 3,
                        skill,
                    });
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardTargetCards {
                    actor: target,
                    target: player,
                    cards: vec![card],
                    scope: TargetCardScope::HandEquip,
                }],
            )
            .map(|_| Vec::new())
        }
        3 => {
            if !alive(tx, player) || !alive(tx, target) {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
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
                ContinuationKind::ResolveYoudi,
                vec![2, player.raw(), target.raw(), skill.raw()],
            )
        }
        4 => {
            if !alive(tx, player) || !alive(tx, target) {
                return Ok(Vec::new());
            }
            let legal = crate::query::discardable_hand_equip_cards(tx.state(), target);
            if !legal.contains(&card) {
                return Err(EffectError::Illegal(
                    "Youdi target card became illegal".into(),
                ));
            }
            let from = tx
                .state()
                .cards
                .get(card)
                .ok_or_else(|| EffectError::Illegal("bad Youdi target card".into()))?
                .zone;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::EngineFault("Youdi owner hand missing".into()))?;
            move_cards(tx, vec![card], to, Some(from))
        }
        _ => Err(EffectError::ContentFault("unknown Youdi stage".into())),
    }
}

pub(crate) fn move_target_equipment_to_player_effects(
    state: &crate::state::GameState,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
) -> Result<Vec<EffectRequest>, EffectError> {
    if !crate::query::movable_target_equipment_cards(state, player, target).contains(&card) {
        return Err(EffectError::Illegal(
            "selected equipment card is no longer movable".into(),
        ));
    }
    let card_state = state
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("missing selected equipment card".into()))?;
    let slot = card_state
        .kind
        .equip_slot()
        .ok_or_else(|| EffectError::Illegal("selected card is not equipment".into()))?;
    let from = card_state.zone;
    let to = state
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::Illegal("missing destination equipment zone".into()))?;
    let replaced = state
        .zones
        .get(to)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|existing| {
            state
                .cards
                .get(*existing)
                .and_then(|card| card.kind.equip_slot())
                == Some(slot)
        })
        .collect::<Vec<_>>();
    let mut effects = Vec::with_capacity(2);
    if !replaced.is_empty() {
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("missing discard pile".into()))?;
        effects.push(EffectRequest::MoveCards {
            cards: replaced,
            to: discard,
            from: Some(to),
        });
    }
    effects.push(EffectRequest::MoveCards {
        cards: vec![card],
        to,
        from: Some(from),
    });
    Ok(effects)
}

pub(crate) fn resolve_kuangfu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid_players = |tx: &Transaction| {
        player != target
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
    };
    if !valid_players(tx) || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let discardable = crate::query::discardable_equipment_cards(tx.state(), player, target);
    match stage {
        0 => {
            if discardable.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                discardable.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveKuangfu,
                vec![0, player.raw(), target.raw(), skill.raw()],
            )
        }
        1 => {
            if !discardable.contains(&card) {
                return Err(EffectError::Illegal(
                    "Kuangfu equipment became illegal".into(),
                ));
            }
            if crate::query::movable_target_equipment_cards(tx.state(), player, target)
                .contains(&card)
            {
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Confirm,
                    ContinuationKind::ResolveKuangfu,
                    vec![1, player.raw(), target.raw(), card.raw(), skill.raw()],
                )
            } else {
                run_effects(
                    tx,
                    vec![EffectRequest::DiscardTargetCards {
                        actor: player,
                        target,
                        cards: vec![card],
                        scope: TargetCardScope::Equip,
                    }],
                )
                .map(|_| Vec::new())
            }
        }
        2 => {
            if !discardable.contains(&card) {
                return Err(EffectError::Illegal(
                    "Kuangfu equipment became illegal".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::DiscardTargetCards {
                    actor: player,
                    target,
                    cards: vec![card],
                    scope: TargetCardScope::Equip,
                }],
            )
            .map(|_| Vec::new())
        }
        3 => {
            let effects =
                move_target_equipment_to_player_effects(tx.state(), player, target, card)?;
            run_effects(tx, effects).map(|_| Vec::new())
        }
        _ => Err(EffectError::ContentFault("unknown Kuangfu stage".into())),
    }
}

pub(crate) fn resolve_shanjia(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    let count = count.min(7);
    match stage {
        0 => {
            if count == 0
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveShanjia {
                    player,
                    count,
                    stage: 1,
                    skill,
                });
            draw_cards(tx, player, count)
        }
        1 => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let required = usize::try_from(count)
                .unwrap_or(usize::MAX)
                .min(options.len());
            if required == 0 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                required.try_into().unwrap_or(u8::MAX),
                required.try_into().unwrap_or(u8::MAX),
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveShanjia,
                vec![player.raw(), required as u32, skill.raw()],
            )
        }
        _ => Err(EffectError::ContentFault("unknown Shanjia stage".into())),
    }
}

pub(crate) fn resolve_mbshanjia(
    tx: &mut Transaction,
    player: PlayerId,
    lost_equipment_count: u32,
    distance_skill_def: engine_api::SkillDefId,
    stage: u8,
    reward_mask: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive);
    if !alive {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveMbshanjia {
                    player,
                    lost_equipment_count,
                    distance_skill_def,
                    stage: 1,
                    reward_mask: 0,
                    skill,
                });
            draw_cards(tx, player, 3)
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            let required = usize::try_from(3u32.saturating_sub(lost_equipment_count))
                .unwrap_or(usize::MAX)
                .min(options.len());
            if required == 0 {
                return resolve_mbshanjia(
                    tx,
                    player,
                    lost_equipment_count,
                    distance_skill_def,
                    2,
                    0b11,
                    skill,
                );
            }
            open_choice(
                tx,
                player,
                required.try_into().unwrap_or(u8::MAX),
                required.try_into().unwrap_or(u8::MAX),
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveMbshanjia,
                vec![
                    player.raw(),
                    required as u32,
                    distance_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            if reward_mask & 0b11 == 0b11 {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveMbshanjia {
                        player,
                        lost_equipment_count,
                        distance_skill_def,
                        stage: 3,
                        reward_mask,
                        skill,
                    },
                );
            }
            if reward_mask & 0b10 != 0 {
                add_skill(tx, player, distance_skill_def)
            } else if reward_mask & 0b01 != 0 {
                resolve_mbshanjia(
                    tx,
                    player,
                    lost_equipment_count,
                    distance_skill_def,
                    3,
                    reward_mask,
                    skill,
                )
            } else {
                Ok(Vec::new())
            }
        }
        3 => choose_use_virtual_sha(tx, player, false, true, false, skill, 0, 0),
        _ => Err(EffectError::ContentFault(
            "unknown mobile Shanjia stage".into(),
        )),
    }
}

pub(crate) fn resolve_daoji(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    tw: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || player == target
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let options = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                if tw {
                    rule_api::TargetCardScope::HandEquip
                } else {
                    rule_api::TargetCardScope::Equip
                },
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
                ContinuationKind::ResolveDaoji,
                if tw {
                    vec![player.raw(), target.raw(), skill.raw(), 1]
                } else {
                    vec![player.raw(), target.raw(), skill.raw()]
                },
            )
        }
        1 => {
            let Some(card_state) = tx.state().cards.get(card).cloned() else {
                return Ok(Vec::new());
            };
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let is_weapon = card_state.kind == CardKind::Weapon;
            let kind = crate::query::effective_card_kind(tx.state(), player, &card_state);
            let remains_in_hand = hand == Some(card_state.zone) && card_state.owner == player;
            if tw && remains_in_hand && kind.is_basic() {
                return run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player,
                        count: 1,
                        skill,
                    }],
                );
            }
            if (!tw && is_weapon) || (tw && remains_in_hand && kind.equip_slot().is_some()) {
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(EffectRequest::ResolveDaoji {
                        player,
                        target,
                        card,
                        stage: 2,
                        tw,
                        skill,
                    });
            }
            let can_use = remains_in_hand
                && tx.state().players.get(player).is_some_and(|participant| {
                    participant.alive
                        && !participant.dying
                        && kind
                            .equip_slot()
                            .is_some_and(|slot| participant.equip_slot_enabled(slot))
                })
                && !crate::query::card_use_cancelled_for_card(tx.state(), player, card);
            if can_use {
                crate::command::play::use_equipment_from_hand(tx, player, card)?;
            }
            Ok(Vec::new())
        }
        2 => super::super::run_effects(
            tx,
            vec![EffectRequest::DealDamage {
                source: player,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        _ => Err(EffectError::ContentFault(
            "unknown mobile Daoji stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn give_all_hand_then_choose_use_one_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    stage: u8,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || player == target
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let cards = tx.state().hand_cards(player);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::GiveAllHandThenChooseUseOneOrDamage {
                    player,
                    target,
                    cards: cards.clone(),
                    targets: Vec::new(),
                    stage: 1,
                    card: CardId::NONE,
                    skill,
                },
            );
            give_all_hand(tx, player, target)
        }
        1 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let options: Vec<_> = cards
                .iter()
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| hand == Some(state.zone) && state.owner == target)
                        && crate::command::play::skill_card_target_spec(tx, target, *card).is_ok()
                })
                .collect();
            if options.is_empty() {
                return run_effects(
                    tx,
                    vec![EffectRequest::DealDamage {
                        source: player,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: rule_api::damage_nature::NORMAL,
                    }],
                );
            }
            let mut payload = vec![
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                0,
                cards.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::GiveAllHandThenChooseUseOneOrDamage,
                payload,
            )
        }
        2 => {
            crate::command::play::handle_skill_card(tx, target, card, &targets)?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown give-all-hand use-or-damage stage".into(),
        )),
    }
}
