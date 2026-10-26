//! Effect primitives for `resolve_jiaohua` and related helpers.

use super::*;

pub(crate) fn resolve_jiaohua(
    tx: &mut Transaction,
    player: PlayerId,
    category: u8,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && !participant.dying)
    {
        return Err(EffectError::Illegal("invalid Jiaohua skill context".into()));
    }
    let used = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.jiaohua_category_mask())
        .ok_or_else(|| EffectError::Illegal("Jiaohua skill disappeared".into()))?;
    let available: Vec<u8> = [0b001, 0b010, 0b100]
        .into_iter()
        .filter(|bit| used & bit == 0)
        .collect();

    if category == 0 {
        if !target.is_none() {
            return Err(EffectError::ContentFault(
                "initial Jiaohua effect cannot carry a target".into(),
            ));
        }
        let mut payload = vec![0, player.raw(), skill.raw(), available.len() as u32];
        payload.extend(available.iter().map(|bit| u32::from(*bit)));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            available.into_iter().map(u32::from).collect(),
            ChoiceKind::Custom,
            ContinuationKind::ResolveJiaohua,
            payload,
        );
    }
    if !available.contains(&category) {
        return Err(EffectError::Illegal(
            "Jiaohua category is unavailable".into(),
        ));
    }
    if target.is_none() {
        let targets = tx.state().players.alive_by_seat();
        let mut payload = vec![
            1,
            player.raw(),
            u32::from(category),
            skill.raw(),
            targets.len() as u32,
        ];
        payload.extend(targets.iter().map(|target| target.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets.into_iter().map(PlayerId::raw).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::ResolveJiaohua,
            payload,
        );
    }
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Jiaohua target is no longer alive".into(),
        ));
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("Jiaohua target hand missing".into()))?;
    let card = tx.state().zones.get(draw).and_then(|zone| {
        zone.cards.iter().rev().copied().find(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| match category {
                    0b001 => card.kind.is_basic(),
                    0b010 => card.kind.is_trick(),
                    0b100 => card.kind.is_equip(),
                    _ => false,
                })
        })
    });
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| instance.record_jiaohua_category(category))
    {
        return Err(EffectError::Illegal(
            "Jiaohua category state changed".into(),
        ));
    }
    match card {
        Some(card) => move_cards(tx, vec![card], hand, Some(draw)),
        None => Ok(Vec::new()),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_luanqun(
    tx: &mut Transaction,
    player: PlayerId,
    effect_skill_def: engine_api::SkillDefId,
    direct_hit_skill_def: engine_api::SkillDefId,
    stage: u8,
    participants: Vec<PlayerId>,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && !participant.dying)
        || effect_skill_def.is_none()
        || direct_hit_skill_def.is_none()
        || effect_skill_def == direct_hit_skill_def
    {
        return Err(EffectError::Illegal("invalid Luanqun skill context".into()));
    }
    match stage {
        0 => {
            if !participants.is_empty() || !cards.is_empty() || tx.state().hand_count(player) == 0 {
                return Err(EffectError::Illegal(
                    "initial Luanqun effect has invalid snapshots".into(),
                ));
            }
            let participants: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|participant| tx.state().hand_count(*participant) > 0)
                .collect();
            let Some(observer) = participants.first().copied() else {
                return Err(EffectError::Illegal(
                    "Luanqun has no hand-card participant".into(),
                ));
            };
            let mut payload = vec![
                0,
                player.raw(),
                effect_skill_def.raw(),
                direct_hit_skill_def.raw(),
                skill.raw(),
                0,
                participants.len() as u32,
            ];
            payload.extend(participants.iter().map(|participant| participant.raw()));
            open_choice(
                tx,
                observer,
                1,
                1,
                false,
                tx.state()
                    .hand_cards(observer)
                    .into_iter()
                    .map(CardId::raw)
                    .collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveLuanqun,
                payload,
            )
        }
        1 => {
            if participants.len() != cards.len() || participants.is_empty() {
                return Err(EffectError::ContentFault(
                    "Luanqun participant/card snapshots differ".into(),
                ));
            }
            for (participant, card) in participants.iter().zip(&cards) {
                let hand = tx.state().zones.find(ZoneKind::Hand, *participant);
                if !tx
                    .state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| Some(card.zone) == hand && card.owner == *participant)
                {
                    return Err(EffectError::Illegal(
                        "Luanqun selected card left its hand".into(),
                    ));
                }
            }
            let owner_index = participants
                .iter()
                .position(|participant| *participant == player)
                .ok_or_else(|| EffectError::ContentFault("Luanqun owner not selected".into()))?;
            let owner_color =
                crate::query::effective_card_color_mask(tx.state(), player, cards[owner_index]);
            let mut matching = Vec::new();
            let mut events = vec![DomainEvent::CardsRevealed {
                cards: cards.clone(),
                to: Vec::new(),
            }];
            for (participant, card) in participants.iter().copied().zip(cards.iter().copied()) {
                if participant == player {
                    continue;
                }
                if crate::query::effective_card_color_mask(tx.state(), participant, card)
                    == owner_color
                {
                    matching.push(card);
                } else {
                    events.extend(add_luanqun_helper(
                        tx,
                        participant,
                        effect_skill_def,
                        1,
                        player,
                    )?);
                    events.extend(add_luanqun_helper(
                        tx,
                        participant,
                        direct_hit_skill_def,
                        2,
                        player,
                    )?);
                }
            }
            if !matching.is_empty() {
                let cap = if tx.state().ruleset.name.starts_with("identity") {
                    4
                } else {
                    2
                };
                let mut payload = vec![1, player.raw(), skill.raw(), matching.len() as u32];
                payload.extend(matching.iter().map(|card| card.raw()));
                events.extend(open_choice(
                    tx,
                    player,
                    1,
                    cap.min(matching.len()).try_into().unwrap_or(u8::MAX),
                    true,
                    matching.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::ResolveLuanqun,
                    payload,
                )?);
            }
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "unknown mobile Luanqun stage".into(),
        )),
    }
}

pub(crate) fn effective_card_category(
    state: &crate::state::GameState,
    player: PlayerId,
    card: CardId,
) -> u8 {
    state
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(state, player, card))
        .map_or(0, |kind| {
            if kind.is_basic() {
                1
            } else if kind.is_trick() {
                2
            } else if kind.is_equip() {
                3
            } else {
                0
            }
        })
}

pub(crate) fn mbshanjia_discard_category(
    state: &crate::state::GameState,
    player: PlayerId,
    card: CardId,
) -> u8 {
    state.cards.get(card).map_or(0, |card| {
        let hand = state.zones.find(ZoneKind::Hand, player);
        let kind = if Some(card.zone) == hand {
            crate::query::effective_card_kind(state, player, card)
        } else {
            card.kind
        };
        if kind.is_basic() {
            1
        } else if kind.is_trick() {
            2
        } else if kind.is_equip() {
            3
        } else {
            0
        }
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_new_yajiao(
    tx: &mut Transaction,
    player: PlayerId,
    source_card: CardId,
    revealed: CardId,
    target: PlayerId,
    source_category: u8,
    stage: u8,
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
    match stage {
        0 => {
            if tx.state().turn.current == player || !originating_skill_is_enabled(tx, player, skill)
            {
                return Ok(Vec::new());
            }
            let source_category = effective_card_category(tx.state(), player, source_card);
            if source_category == 0 {
                return Ok(Vec::new());
            }
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("New Yajiao draw pile missing".into()))?;
            if tx
                .state()
                .zones
                .get(draw)
                .is_none_or(|zone| zone.cards.is_empty())
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let Some(revealed) = tx
                .state()
                .zones
                .get(draw)
                .and_then(|zone| zone.cards.last().copied())
            else {
                return Ok(Vec::new());
            };
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("New Yajiao processing zone missing".into()))?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveNewYajiao {
                    player,
                    source_card,
                    revealed,
                    target: PlayerId::NONE,
                    source_category,
                    stage: 1,
                    skill,
                });
            let mut events = move_cards(tx, vec![revealed], processing, Some(draw))?;
            events.push(DomainEvent::CardsRevealed {
                cards: vec![revealed],
                to: Vec::new(),
            });
            Ok(events)
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("New Yajiao processing zone missing".into()))?;
            if !tx
                .state()
                .cards
                .get(revealed)
                .is_some_and(|card| card.zone == processing)
            {
                return Err(EffectError::Illegal(
                    "New Yajiao revealed card left processing".into(),
                ));
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                tx.state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .map(PlayerId::raw)
                    .collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ResolveNewYajiao,
                vec![
                    0,
                    player.raw(),
                    source_card.raw(),
                    revealed.raw(),
                    u32::from(source_category),
                    skill.raw(),
                ],
            )
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("New Yajiao processing zone missing".into()))?;
            if !tx
                .state()
                .cards
                .get(revealed)
                .is_some_and(|card| card.zone == processing)
            {
                return Err(EffectError::Illegal(
                    "New Yajiao revealed card left processing".into(),
                ));
            }
            if effective_card_category(tx.state(), player, revealed) != source_category {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveNewYajiao {
                        player,
                        source_card,
                        revealed,
                        target,
                        source_category,
                        stage: 3,
                        skill,
                    },
                );
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("New Yajiao recipient hand missing".into()))?;
            move_cards(tx, vec![revealed], hand, Some(processing))
        }
        3 => {
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
                ContinuationKind::ResolveNewYajiao,
                vec![1, player.raw()],
            )
        }
        _ => Err(EffectError::ContentFault("unknown New Yajiao stage".into())),
    }
}

pub(crate) fn recover_selected_targets(
    tx: &mut Transaction,
    source: PlayerId,
    targets: &[PlayerId],
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if targets.is_empty() {
        return Err(EffectError::ContentFault(
            "selected-target recovery requires at least one target".into(),
        ));
    }
    let order = tx.state().players.alive_by_seat();
    let mut sorted = Vec::with_capacity(targets.len());
    for &target in targets {
        if sorted.contains(&target) {
            return Err(EffectError::ContentFault(
                "selected-target recovery received duplicate targets".into(),
            ));
        }
        if !order.contains(&target) {
            return Err(EffectError::Illegal(
                "selected-target recovery requires living targets".into(),
            ));
        }
        sorted.push(target);
    }
    sorted.sort_by_key(|target| {
        order
            .iter()
            .position(|candidate| candidate == target)
            .expect("validated living target")
    });
    let mut events = Vec::new();
    for target in sorted {
        events.extend(recover_hp(tx, source, target, amount)?);
    }
    Ok(events)
}

pub(crate) fn recover_if_min_current_hp(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participant = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad player".into()))?;
    if !participant.alive || amount == 0 {
        return Ok(Vec::new());
    }
    let hp = participant.hp;
    let minimum = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|candidate| tx.state().players.get(candidate).unwrap().hp)
        .min();
    if minimum != Some(hp) {
        return Ok(Vec::new());
    }
    recover_hp(tx, player, player, amount)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_then_give_one(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    drawn_only: bool,
    use_given_equipment: bool,
    skill: engine_api::SkillId,
    drawn: Vec<CardId>,
    ready_to_give: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|state| state.alive);
    if !target_alive {
        return Ok(Vec::new());
    }

    let target_hand = tx.state().zones.find(ZoneKind::Hand, target);
    if use_given_equipment
        && drawn.len() == 1
        && tx.state().cards.get(drawn[0]).is_some_and(|card| {
            let kind = crate::query::effective_card_kind(tx.state(), target, card);
            card.owner == target
                && Some(card.zone) == target_hand
                && kind.is_equip()
                && kind.equip_slot().is_some_and(|slot| {
                    tx.state()
                        .players
                        .get(target)
                        .is_some_and(|player| player.equip_slot_enabled(slot))
                })
                && !crate::query::card_use_cancelled_for_card(tx.state(), target, card.id)
        })
    {
        let card = drawn[0];
        return open_choice(
            tx,
            target,
            1,
            1,
            true,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::UseGivenEquipment,
            vec![target.raw(), card.raw()],
        );
    }

    let player_alive = tx
        .state()
        .players
        .get(player)
        .is_some_and(|state| state.alive);
    if !player_alive || player == target {
        return Ok(Vec::new());
    }

    if !ready_to_give && count > 0 {
        let events = draw_cards(tx, player, count)?;
        let drawn = events
            .iter()
            .find_map(|event| match event {
                DomainEvent::CardsDrawn {
                    player: drawer,
                    cards,
                } if *drawer == player => Some(cards.clone()),
                _ => None,
            })
            .unwrap_or_default();
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::DrawThenGiveOne {
                player,
                target,
                count,
                drawn_only,
                use_given_equipment,
                skill,
                drawn,
                ready_to_give: true,
            });
        return Ok(events);
    }

    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    let mut options: Vec<CardId> = if drawn_only {
        drawn
            .into_iter()
            .filter(|card| {
                tx.state().cards.get(*card).is_some_and(|state| {
                    state.owner == player && (Some(state.zone) == hand || Some(state.zone) == equip)
                })
            })
            .collect()
    } else {
        [hand, equip]
            .into_iter()
            .flatten()
            .flat_map(|zone| {
                tx.state()
                    .zones
                    .get(zone)
                    .map(|state| state.cards.clone())
                    .unwrap_or_default()
            })
            .collect()
    };
    options.sort_by_key(|card| card.raw());
    options.dedup();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        player.raw(),
        target.raw(),
        u32::from(drawn_only),
        u32::from(use_given_equipment),
        skill.raw(),
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::GiveOneHandOrEquip,
        payload,
    )
}

pub(crate) fn draw_then_take_one(
    tx: &mut Transaction,
    player: PlayerId,
    taker: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
    ready_to_take: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !ready_to_take {
        let events = draw_cards(tx, player, count)?;
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::DrawThenTakeOne {
                player,
                taker,
                count,
                skill,
                ready_to_take: true,
            });
        return Ok(events);
    }

    if player == taker {
        return Ok(Vec::new());
    }
    take_one_card(tx, player, taker, 1)
}

pub(crate) fn take_one_card(
    tx: &mut Transaction,
    from: PlayerId,
    to: PlayerId,
    mode: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ![from, to].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    }) {
        return Ok(Vec::new());
    }
    let scope = match mode {
        1 => TargetCardScope::HandEquip,
        2 => TargetCardScope::Equip,
        3 => TargetCardScope::Hand,
        10..=14 => TargetCardScope::from_tag((mode - 10) as u8)
            .ok_or_else(|| EffectError::ContentFault("invalid take-card scope".into()))?,
        _ => return Err(EffectError::ContentFault("invalid take-card mode".into())),
    };
    let options = crate::query::gainable_target_cards(tx.state(), to, from, scope);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        to,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::PickTargetCard,
        vec![from.raw(), to.raw(), mode],
    )
}

pub(crate) fn choose_other_hand_store_skill_pile(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ![player, target]
        .into_iter()
        .all(|p| tx.state().players.get(p).is_some_and(|state| state.alive))
        || player == target
    {
        return Ok(Vec::new());
    }
    let options =
        crate::query::gainable_target_cards(tx.state(), player, target, TargetCardScope::Hand);
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
        ContinuationKind::PickTargetCard,
        vec![target.raw(), player.raw(), 20, skill.raw()],
    )
}

pub(crate) fn gain_all_skill_pile_cards(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = skill_pile_def(tx, player, skill)?;
    let cards = tx.state().skill_pile_cards(player, skill_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile = ensure_skill_pile_zone(tx, player, skill_def)?;
    move_cards(tx, cards, hand, Some(pile))
}

/// Move every card in `target`'s skill pile keyed by `skill_def` into `player`'s hand.
/// Implements 英逝 (sp/yingshi_die): a dying player's 应逝 skill pile is gained.
pub(crate) fn gain_other_skill_pile_cards(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let cards = tx.state().skill_pile_cards(target, skill_def);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let pile = ensure_skill_pile_zone(tx, target, skill_def)?;
    move_cards(tx, cards, hand, Some(pile))
}

pub(crate) fn take_target_hand_then_repeat_until_max(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    open_take_larger_hand_card_choice(tx, player, target, skill)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn choose_draw_or_increase_current_card_damage_or_both_mark(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    draw_count: u32,
    damage_amount: u32,
    mark_def: engine_api::MarkDefId,
    mark_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let owner = tx.state().players.get(player);
    if !owner.is_some_and(|owner| owner.alive && owner.hp < owner.max_hp as i32)
        || !originating_skill_is_enabled(tx, player, skill)
    {
        return Ok(Vec::new());
    }
    if tx.state().cards.get(card).is_none()
        || draw_count == 0
        || damage_amount == 0
        || mark_def.is_none()
        || mark_count == 0
        || mark_count > i32::MAX as u32
    {
        return Err(EffectError::ContentFault(
            "invalid current-card branch parameters".into(),
        ));
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2, 3],
        ChoiceKind::Custom,
        ContinuationKind::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark,
        vec![
            player.raw(),
            card.raw(),
            draw_count,
            damage_amount,
            mark_def.raw(),
            mark_count,
            skill.raw(),
        ],
    )
}

pub(crate) fn continue_take_larger_hand_until_max(
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
    let player_hand = tx.state().hand_count(player);
    let max_hand = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|candidate| tx.state().hand_count(candidate))
        .max()
        .unwrap_or(player_hand);
    if player_hand >= max_hand {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && tx.state().hand_count(target) > player_hand)
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
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTakeLargerHandUntilMax,
        vec![0, player.raw(), skill.raw()],
    )
}

pub(crate) fn open_take_larger_hand_card_choice(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
        || tx.state().hand_count(target) <= tx.state().hand_count(player)
    {
        return Err(EffectError::Illegal(
            "take target must be another living player with more hand cards".into(),
        ));
    }
    let options = tx.state().hand_cards(target);
    if options.is_empty() {
        return Err(EffectError::Illegal(
            "take target must have a hand card".into(),
        ));
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTakeLargerHandUntilMax,
        vec![1, player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn move_equipment_then_draw(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = crate::query::movable_equipment_cards(tx.state(), player);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        0,
        player.raw(),
        skill.raw(),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::MoveEquipmentThenDraw,
        payload,
    )
}

pub(crate) fn discard_equipment_pair_with_armor(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !crate::query::equipment_pair_with_armor_legal(tx.state(), player, &targets) {
        return Err(EffectError::Illegal(
            "equipment pair with armor is no longer legal".into(),
        ));
    }
    let target0 = targets[0];
    let target1 = targets[1];
    let options = if crate::query::discardable_armor_equipment_cards(tx.state(), player, target1)
        .is_empty()
    {
        crate::query::discardable_armor_equipment_cards(tx.state(), player, target0)
    } else {
        crate::query::discardable_equipment_cards(tx.state(), player, target0)
    };
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DiscardEquipmentPairWithArmor,
        vec![0, player.raw(), skill.raw(), target0.raw(), target1.raw()],
    )
}

pub(crate) fn discard_target_equipment_gain_mount(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    open_discard_target_card_gain_mount(
        tx,
        player,
        target,
        rule_api::TargetCardScope::Equip,
        ContinuationKind::DiscardTargetEquipmentGainMount,
    )
}

pub(crate) fn discard_target_equipment_draw_weapon_range(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Err(EffectError::Illegal(
            "weapon-range discard context changed".into(),
        ));
    }
    let options = crate::query::discardable_equipment_cards(tx.state(), player, target);
    if options.is_empty() {
        return Err(EffectError::Illegal(
            "target has no discardable equipment".into(),
        ));
    }
    let mut payload = vec![
        player.raw(),
        target.raw(),
        skill.raw(),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DiscardTargetEquipmentDrawWeaponRange,
        payload,
    )
}

pub(crate) fn choose_target_reveal_hand_damage_if_sha(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
        || !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| tx.state().hand_count(target) > 0)
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
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetRevealHandDamageIfSha,
        vec![player.raw(), amount, skill.raw()],
    )
}

/// ABI 682 (batch 180): choose one other player with hand cards — they give one hand
/// card to the chooser, then **they** recover 1 HP (psdaohe 踏鹤).
pub(crate) fn choose_other_give_then_recover(
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
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .filter(|&target| {
            tx.state()
                .zones
                .find(ZoneKind::Hand, target)
                .and_then(|zone| tx.state().zones.get(zone))
                .is_some_and(|zone| !zone.cards.is_empty())
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
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherGiveThenRecover,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn choose_other_give_one_hand_equip(
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
        return Ok(Vec::new());
    }
    let has_card = [ZoneKind::Hand, ZoneKind::Equip].into_iter().any(|kind| {
        tx.state()
            .zones
            .find(kind, player)
            .and_then(|zone| tx.state().zones.get(zone))
            .is_some_and(|zone| !zone.cards.is_empty())
    });
    if !has_card {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
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
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherGiveOneHandEquip,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn discard_target_card_gain_mount(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    scope: rule_api::TargetCardScope,
) -> Result<Vec<DomainEvent>, EffectError> {
    open_discard_target_card_gain_mount(
        tx,
        player,
        target,
        scope,
        ContinuationKind::DiscardTargetCardGainMount,
    )
}

pub(crate) fn choose_discard_target_card(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    scope: rule_api::TargetCardScope,
) -> Result<Vec<DomainEvent>, EffectError> {
    open_discard_target_card_choice(
        tx,
        player,
        target,
        scope,
        ContinuationKind::ChooseDiscardTargetCard,
        false,
        None,
    )
}

pub(crate) fn choose_optional_discard_target_card(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    scope: rule_api::TargetCardScope,
) -> Result<Vec<DomainEvent>, EffectError> {
    open_discard_target_card_choice(
        tx,
        player,
        target,
        scope,
        ContinuationKind::ChooseDiscardTargetCard,
        true,
        None,
    )
}

pub(crate) fn equipped_cards(tx: &Transaction, target: PlayerId) -> Vec<CardId> {
    let mut cards = tx
        .state()
        .zones
        .find(ZoneKind::Equip, target)
        .and_then(|zone| tx.state().zones.get(zone))
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    cards.sort_by_key(|card| card.raw());
    cards
}

pub(crate) fn attack_range_population(tx: &Transaction, player: PlayerId) -> u32 {
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| crate::query::player_in_attack_range(tx.state(), player, target))
        .count()
        .min(u32::MAX as usize) as u32
}

pub(crate) fn choose_target_reclaim_equipment_draw_if_range_reduced(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target
        || !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "target-equipment reclamation context changed".into(),
        ));
    }
    let options = equipped_cards(tx, target);
    if options.is_empty() {
        return Err(EffectError::Illegal(
            "target no longer has equipment to reclaim".into(),
        ));
    }
    let mut payload = vec![
        player.raw(),
        target.raw(),
        skill.raw(),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTargetReclaimEquipmentDrawIfRangeReduced,
        payload,
    )
}

pub(crate) fn finish_target_reclaim_equipment_draw_if_range_reduced(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    before: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let after = attack_range_population(tx, target);
    if after < before
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        draw_cards(tx, player, 1)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn commit_target_reclaim_equipment_draw_if_range_reduced(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    before: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, target)
        .ok_or_else(|| EffectError::Illegal("missing target equipment zone".into()))?;
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .zones
            .get(equip)
            .is_some_and(|zone| zone.cards.contains(&card))
    {
        return Err(EffectError::Illegal(
            "selected equipment is no longer reclaimable".into(),
        ));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("missing target hand zone".into()))?;
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::FinishTargetReclaimEquipmentDrawIfRangeReduced {
            player,
            target,
            before,
            skill,
        },
    );
    move_cards(tx, vec![card], hand, Some(equip))
}

pub(crate) fn open_discard_target_card_gain_mount(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    scope: rule_api::TargetCardScope,
    continuation: ContinuationKind,
) -> Result<Vec<DomainEvent>, EffectError> {
    open_discard_target_card_choice(tx, player, target, scope, continuation, false, None)
}

pub(crate) fn open_discard_target_card_choice(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    scope: rule_api::TargetCardScope,
    continuation: ContinuationKind,
    allow_cancel: bool,
    current_sha_nonbasic_bonus_skill: Option<engine_api::SkillDefId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participants_alive = [player, target].into_iter().all(|participant| {
        tx.state()
            .players
            .get(participant)
            .is_some_and(|state| state.alive)
    });
    if !participants_alive {
        return Ok(Vec::new());
    }
    let options = crate::query::discardable_target_cards(tx.state(), player, target, scope);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        allow_cancel,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        continuation,
        if matches!(
            continuation,
            ContinuationKind::DiscardTargetCardGainMount
                | ContinuationKind::ChooseDiscardTargetCard
                | ContinuationKind::ChooseDiscardTargetCardResolveCurrentSha
        ) {
            if continuation == ContinuationKind::ChooseDiscardTargetCardResolveCurrentSha {
                let card = match tx.state().resolution.resume_kind {
                    ContinuationKind::ResumePlayedSha => {
                        *tx.state().resolution.resume_payload.get(5).unwrap_or(&0)
                    }
                    ContinuationKind::ResumePlayedShaQueue => {
                        *tx.state().resolution.resume_payload.get(4).unwrap_or(&0)
                    }
                    _ => 0,
                };
                let mut payload = vec![player.raw(), target.raw(), card];
                if let Some(skill_def) = current_sha_nonbasic_bonus_skill {
                    payload.push(skill_def.raw());
                }
                payload
            } else {
                let mut payload = vec![player.raw(), target.raw(), u32::from(scope.tag())];
                if continuation == ContinuationKind::ChooseDiscardTargetCard && allow_cancel {
                    payload.push(1);
                }
                payload
            }
        } else {
            vec![player.raw(), target.raw()]
        },
    )
}

pub(crate) fn move_cards(
    tx: &mut Transaction,
    cards: Vec<CardId>,
    to: ZoneId,
    from: Option<ZoneId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    move_cards_with_reason(tx, cards, to, from, CardMoveReason::Unspecified)
}

pub(crate) fn choose_give_or_recast_one_hand(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    require_skill_def: u32,
    grant_skill_def: u32,
    grant_mark_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let allowed = originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && !participant.dying);
    let cards: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    if !allowed || cards.is_empty() {
        return Ok(Vec::new());
    }
    let require_skill_def = tx.state().skills.storage_get(skill, 2).max(require_skill_def);
    let grant_skill_def = tx.state().skills.storage_get(skill, 3).max(grant_skill_def);
    let grant_mark_def = tx.state().skills.storage_get(skill, 4).max(grant_mark_def);
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        cards,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGiveOrRecastOneHand,
        vec![
            player.raw(),
            skill.raw(),
            0,
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
        ],
    )
}
