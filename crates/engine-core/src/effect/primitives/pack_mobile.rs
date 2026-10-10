//! Effect primitive implementations — mobile cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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

pub(crate) fn give_cards(
    tx: &mut Transaction,
    cards: Vec<CardId>,
    to: ZoneId,
    from: ZoneId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let giver = tx
        .state()
        .zones
        .get(from)
        .map_or(PlayerId::NONE, |zone| zone.owner);
    give_cards_as(tx, giver, cards, to, from)
}

pub(crate) fn give_cards_as(
    tx: &mut Transaction,
    giver: PlayerId,
    cards: Vec<CardId>,
    to: ZoneId,
    from: ZoneId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let recipient = tx
        .state()
        .zones
        .get(to)
        .map_or(PlayerId::NONE, |zone| zone.owner);
    let given = cards.clone();
    let mut events = move_cards_with_reason(tx, cards, to, Some(from), CardMoveReason::Give)?;
    if giver.is_some() && recipient.is_some() && giver != recipient && !given.is_empty() {
        events.push(DomainEvent::CardsGiven {
            giver,
            recipient,
            cards: given,
        });
    }
    Ok(events)
}

pub(crate) fn move_cards_with_reason(
    tx: &mut Transaction,
    cards: Vec<CardId>,
    to: ZoneId,
    from: Option<ZoneId>,
    reason: CardMoveReason,
) -> Result<Vec<DomainEvent>, EffectError> {
    let to_zone = tx
        .state()
        .zones
        .get(to)
        .ok_or_else(|| EffectError::Illegal(format!("bad to zone {}", to.raw())))?
        .clone();
    let owner = to_zone.owner;
    let face_up = matches!(
        to_zone.kind,
        ZoneKind::DiscardPile
            | ZoneKind::Equip
            | ZoneKind::Processing
            | ZoneKind::Judge
            | ZoneKind::Expansion
    );
    let mut moved_from = from.unwrap_or(ZoneId::NONE);
    let mut equipment_transitions = Vec::new();
    for card in &cards {
        let c = tx
            .state()
            .cards
            .get(*card)
            .ok_or_else(|| EffectError::Illegal(format!("bad card {}", card.raw())))?
            .clone();
        let src = c.zone;
        let playable_pile = reason
            .is_use_or_respond()
            .then(|| {
                tx.state()
                    .skill_piles
                    .iter()
                    .find(|pile| pile.owner == c.owner && pile.zone == src && pile.usable_as_hand)
                    .copied()
            })
            .flatten();
        if let Some(pile) = playable_pile {
            tx.state_mut()
                .turn
                .record_playable_skill_pile_card(c.owner, *card);
            tx.state_mut()
                .turn
                .record_playable_skill_pile(c.owner, pile.skill_def);
        }
        let source_zone = tx
            .state()
            .zones
            .get(src)
            .ok_or_else(|| EffectError::Illegal(format!("bad from zone {}", src.raw())))?;
        equipment_transitions.push((source_zone.kind, c.owner, c.def));
        if from.is_some() && from != Some(src) {
            return Err(EffectError::Illegal("card not in from zone".into()));
        }
        moved_from = src;
        let previous_owner = c.owner;
        move_one_card(tx, *card, src, to, owner, face_up)?;
        // Opt-in per-player「本回合失去牌」计数（沙城 draw-X）：仅当对局中存在
        // 已声明 wants_lost_count 的技能堆（城族）时计数——其他包零额外状态、零哈希漂移。
        if previous_owner.is_some()
            && !reason.is_use_or_respond()
            && (owner != previous_owner || matches!(to_zone.kind, ZoneKind::DiscardPile | ZoneKind::DrawPile))
            && tx
                .state()
                .skill_piles
                .iter()
                .any(|pile| pile.wants_lost_count)
        {
            let key = crate::state::turn_flag::lost_card_count_by_player(previous_owner);
            let bumped = tx.state().turn.flag(key) + 1;
            tx.state_mut().turn.set_flag(key, bumped);
        }
    }
    let mut events = vec![DomainEvent::CardsMoved {
        cards,
        from: moved_from,
        to,
        reason,
    }];
    if to_zone.kind == ZoneKind::Equip {
        for (_, _, card_def) in &equipment_transitions {
            let Some(metadata) = tx.state().content.card_metadata(*card_def).cloned() else {
                continue;
            };
            for skill_def in metadata.equip_skills {
                events.extend(add_skill(
                    tx,
                    owner,
                    engine_api::SkillDefId::new(skill_def),
                )?);
            }
            if metadata.on_leave_equip_skill_def != 0 {
                events.extend(add_skill(
                    tx,
                    owner,
                    engine_api::SkillDefId::new(metadata.on_leave_equip_skill_def),
                )?);
            }
        }
    }
    for (source_kind, previous_owner, card_def) in equipment_transitions {
        if source_kind != ZoneKind::Equip
            || (to_zone.kind == ZoneKind::Equip && previous_owner == owner)
            || previous_owner.is_none()
        {
            continue;
        }
        let Some(metadata) = tx.state().content.card_metadata(card_def).cloned() else {
            continue;
        };
        for skill_def in metadata.equip_skills {
            let still_granted = tx
                .state()
                .zones
                .find(ZoneKind::Equip, previous_owner)
                .and_then(|zone| tx.state().zones.get(zone))
                .into_iter()
                .flat_map(|zone| zone.cards.iter())
                .filter_map(|card| tx.state().cards.get(*card))
                .filter_map(|card| tx.state().content.card_metadata(card.def))
                .any(|card| card.equip_skills.contains(&skill_def));
            if still_granted {
                continue;
            }
            let skill = tx
                .state()
                .players
                .get(previous_owner)
                .into_iter()
                .flat_map(|player| player.skills.iter())
                .find(|skill| {
                    tx.state()
                        .skills
                        .get(**skill)
                        .is_some_and(|instance| instance.def.raw() == skill_def)
                })
                .copied();
            if let Some(skill) = skill {
                events.extend(remove_skill(tx, skill)?);
            }
        }
    }
    Ok(events)
}

/// Move a single card; records journal.
pub fn move_one_card(
    tx: &mut Transaction,
    card: CardId,
    from: ZoneId,
    to: ZoneId,
    new_owner: PlayerId,
    face_up: bool,
) -> Result<(), EffectError> {
    let loss_owner = (from != to && tx.state().content.tracks_turn_card_loss_history())
        .then(|| {
            let from_zone = tx.state().zones.get(from)?;
            let card_state = tx.state().cards.get(card)?;
            let previous_owner = card_state.owner;
            let stays_in_owned_region = tx.state().zones.get(to).is_some_and(|to_zone| {
                to_zone.owner == previous_owner
                    && matches!(to_zone.kind, ZoneKind::Hand | ZoneKind::Equip)
            });
            (previous_owner.is_some()
                && matches!(from_zone.kind, ZoneKind::Hand | ZoneKind::Equip)
                && !stays_in_owned_region)
                .then_some(previous_owner)
        })
        .flatten();
    let leaves_previous_owner_hand = from != to
        && tx
            .state()
            .zones
            .get(from)
            .is_some_and(|zone| zone.kind == ZoneKind::Hand)
        && tx.state().cards.get(card).is_some_and(|state| {
            state.owner.is_some()
                && !(state.owner == new_owner
                    && tx
                        .state()
                        .zones
                        .get(to)
                        .is_some_and(|zone| zone.kind == ZoneKind::Hand))
        });
    let record_gain = from != to
        && new_owner != PlayerId::NONE
        && tx.state().content.tracks_turn_card_gain_count()
        && tx
            .state()
            .zones
            .get(to)
            .is_some_and(|zone| zone.kind == ZoneKind::Hand);
    let (prev_owner, prev_face, prev_index) = {
        let c = tx
            .state()
            .cards
            .get(card)
            .ok_or_else(|| EffectError::EngineFault("card missing".into()))?;
        let idx = tx
            .state()
            .zones
            .get(from)
            .and_then(|z| z.cards.iter().position(|x| *x == card))
            .ok_or_else(|| {
                EffectError::Illegal(format!("card {} not in zone {}", card.raw(), from.raw()))
            })?;
        (c.owner, c.face_up, idx as u32)
    };
    tx.journal(JournalEntry::CardMove {
        card,
        prev_zone: from,
        prev_owner,
        prev_face_up: prev_face,
        prev_index,
    });
    if !tx.state_mut().zones.remove_card(from, card) {
        return Err(EffectError::Illegal(format!(
            "card {} not in zone {}",
            card.raw(),
            from.raw()
        )));
    }
    if !tx.state_mut().zones.push_card(to, card) {
        return Err(EffectError::EngineFault("to zone missing".into()));
    }
    let c = tx
        .state_mut()
        .cards
        .get_mut(card)
        .ok_or_else(|| EffectError::EngineFault("card missing".into()))?;
    c.zone = to;
    c.owner = new_owner;
    c.face_up = face_up;
    if leaves_previous_owner_hand {
        tx.state_mut().face_down_hand_cards.remove(&card);
        let skill_ids: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .filter(|skill| skill.owner == prev_owner)
            .map(|skill| skill.id)
            .collect();
        for skill in skill_ids {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .expect("collected live skill")
                .forget_remembered_hand_card(card);
        }
    }
    if record_gain {
        tx.state_mut().turn.record_card_gain(new_owner, card);
    }
    if from != to
        && new_owner.is_some()
        && tx.state().turn.phase == Phase::Play
        && tx.state().content.tracks_play_phase_card_gain_history()
        && tx
            .state()
            .zones
            .get(to)
            .is_some_and(|zone| zone.kind == ZoneKind::Hand)
    {
        tx.state_mut().turn.record_play_phase_card_gain(new_owner);
    }
    if let Some(previous_owner) = loss_owner {
        tx.state_mut().turn.record_card_loss(previous_owner, card);
        // gaintag_map foundation: record any content tags carried by the lost card so a later
        // trigger can detect "a tagged card was consumed this turn".
        let tags: Vec<u32> = tx
            .state()
            .content_card_tags
            .get(&card)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default();
        for tag in tags {
            tx.state_mut().turn.record_lost_card_tag(tag);
        }
    }
    Ok(())
}

/// Reshuffle discard pile into draw pile using RuleRng.
pub fn reshuffle_discard_into_draw(tx: &mut Transaction) -> Result<(), EffectError> {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let cards = tx
        .state()
        .zones
        .get(discard)
        .map(|z| z.cards.clone())
        .unwrap_or_default();
    if cards.is_empty() {
        return Ok(());
    }
    for cid in &cards {
        move_one_card(tx, *cid, discard, draw, PlayerId::NONE, false)?;
    }
    // Shuffle draw pile
    let mut pile = tx
        .state()
        .zones
        .get(draw)
        .map(|z| z.cards.clone())
        .unwrap_or_default();
    {
        let mut rng = RuleRng::from_state(tx.state().rng);
        tx.journal(JournalEntry::RngCheckpoint);
        rng.shuffle(&mut pile);
        tx.state_mut().rng = rng.into_state();
    }
    if let Some(z) = tx.state_mut().zones.get_mut(draw) {
        z.cards = pile;
    }
    tx.state_mut().draw_reshuffle_count = tx.state().draw_reshuffle_count.saturating_add(1);
    // Fix card zone/owner already set by move_one
    Ok(())
}

pub(crate) fn use_card(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let intent = crate::intent::resolve_card_use_intent(
        tx.state(),
        crate::intent::CardUseIntent {
            player,
            card,
            targets,
            cancelled: false,
        },
    );
    if intent.cancelled {
        return Ok(Vec::new());
    }
    let borrowed_refills = pothaoshi_refill_effects(tx.state(), player, &[card]);
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), player, card))
        .ok_or_else(|| EffectError::Illegal(format!("bad card {}", card.raw())))?;
    let source_was_hand = tx
        .state()
        .cards
        .get(card)
        .and_then(|card| tx.state().zones.get(card.zone))
        .is_some_and(|zone| zone.kind == ZoneKind::Hand);
    let move_after_use = {
        let c = tx
            .state()
            .cards
            .get(card)
            .ok_or_else(|| EffectError::Illegal(format!("bad card {}", card.raw())))?
            .clone();
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
        let view_as_material =
            tx.state().turn.flag(crate::state::turn_flag::VIEW_AS_CARD) == card.raw() as i32;
        let fusi_material = view_as_material
            && crate::command::skill::dddfusi_approved_source(tx.state(), player, card).is_some();
        let equip = tx.state().zones.find(ZoneKind::Equip, player);
        let planned_source = tx.state().resolution.has_planned_card_source(player, card);
        if kind.is_equip() && (c.zone == hand || planned_source) && !view_as_material {
            None
        } else if c.zone == hand
            || tx.state().playable_hand_cards(player).contains(&card)
            || fusi_material
            || (view_as_material && equip == Some(c.zone))
        {
            if c.zone != hand
                && tx
                    .state()
                    .skill_piles
                    .iter()
                    .any(|pile| pile.owner == player && pile.zone == c.zone && pile.usable_as_hand)
            {
                tx.state_mut()
                    .turn
                    .record_playable_skill_pile_card(player, card);
            }
            let destination_kind = if c.def.is_none() {
                ZoneKind::Removed
            } else {
                ZoneKind::DiscardPile
            };
            let destination = tx
                .state()
                .zones
                .find(destination_kind, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("no post-use destination".into()))?;
            Some((c.zone, destination))
        } else {
            let equip = tx
                .state()
                .zones
                .find(ZoneKind::Equip, player)
                .ok_or_else(|| EffectError::Illegal("no equip".into()))?;
            let kind = crate::query::effective_card_kind(tx.state(), player, &c);
            if c.zone != equip || !kind.is_equip() {
                return Err(EffectError::Illegal("card not in hand or equipped".into()));
            }
            None
        }
    };
    if let Some((from, destination)) = move_after_use {
        move_one_card(tx, card, from, destination, PlayerId::NONE, true)?;
    }
    let mut events = Vec::new();
    for refill in borrowed_refills {
        events.extend(apply_effect(tx, refill)?);
    }
    if player == tx.state().turn.current {
        let used = tx.state().turn.flag(turn_flag::CARD_USED);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::CARD_USED, used.saturating_add(1));
        if intent.targets.iter().any(|&target| target != player) {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::CARD_TARGETED_OTHER, 1);
        }
    }
    let view_as_skill_def = tx
        .state()
        .skills
        .get(engine_api::SkillId::new(
            tx.state().turn.flag(turn_flag::VIEW_AS_SKILL).max(0) as u32,
        ))
        .map_or(0, |skill| skill.def.raw());
    tx.state_mut().resolution.register_card_resolution(
        player,
        card,
        kind.tag(),
        intent.targets.clone(),
        source_was_hand,
        view_as_skill_def,
    );
    events.push(DomainEvent::CardUsed {
        player,
        card,
        targets: intent.targets,
    });
    Ok(events)
}

pub(crate) fn use_field_card(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let intent = crate::intent::resolve_card_use_intent(
        tx.state(),
        crate::intent::CardUseIntent {
            player,
            card,
            targets,
            cancelled: false,
        },
    );
    if intent.cancelled {
        return Ok(Vec::new());
    }
    let source = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal(format!("bad card {}", card.raw())))?
        .zone;
    let kind = crate::query::effective_card_kind(
        tx.state(),
        player,
        tx.state().cards.get(card).expect("validated field card"),
    );
    if !tx
        .state()
        .zones
        .get(source)
        .is_some_and(|zone| matches!(zone.kind, ZoneKind::Equip | ZoneKind::Judge))
    {
        return Err(EffectError::Illegal("card not in a field zone".into()));
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let mut events =
        move_cards_with_reason(tx, vec![card], discard, Some(source), CardMoveReason::Use)?;
    if player == tx.state().turn.current {
        let used = tx.state().turn.flag(turn_flag::CARD_USED);
        tx.state_mut()
            .turn
            .set_flag(turn_flag::CARD_USED, used.saturating_add(1));
        if intent.targets.iter().any(|&target| target != player) {
            tx.state_mut()
                .turn
                .set_flag(turn_flag::CARD_TARGETED_OTHER, 1);
        }
    }
    tx.state_mut().resolution.register_card_resolution(
        player,
        card,
        kind.tag(),
        intent.targets.clone(),
        false,
        0,
    );
    events.push(DomainEvent::CardUsed {
        player,
        card,
        targets: intent.targets,
    });
    Ok(events)
}

pub(crate) fn choose_use_virtual_sha(
    tx: &mut Transaction,
    player: PlayerId,
    ignore_distance: bool,
    ignore_use_count: bool,
    draw_on_no_use: bool,
    skill: engine_api::SkillId,
    lose_hp_before_use: u32,
    damage_bonus: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let actor = tx
        .state()
        .players
        .get(player)
        .ok_or_else(|| EffectError::Illegal("bad virtual Sha user".into()))?;
    if !actor.alive || actor.dying {
        return Ok(Vec::new());
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no removed zone".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);

    let used = crate::query::sha_play_phase_use_count(tx.state());
    if crate::query::card_use_cancelled_for_card(tx.state(), player, card)
        || (!ignore_use_count
            && used >= crate::query::sha_use_limit_for_card(tx.state(), player, card))
    {
        move_one_card(tx, card, processing, removed, PlayerId::NONE, true)?;
        return if draw_on_no_use {
            draw_cards(tx, player, 1)
        } else {
            Ok(Vec::new())
        };
    }
    let legal: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            crate::command::play::validate_sha_target_with_options(
                tx,
                player,
                card,
                target,
                ignore_distance,
            )
            .is_ok()
        })
        .collect();
    if legal.is_empty() {
        move_one_card(tx, card, processing, removed, PlayerId::NONE, true)?;
        return if draw_on_no_use {
            draw_cards(tx, player, 1)
        } else {
            Ok(Vec::new())
        };
    }
    let (min_count, max_count) = if crate::query::sha_targets_all_for(tx.state(), player) {
        let count = u8::try_from(legal.len()).unwrap_or(u8::MAX);
        (count, count)
    } else {
        (
            1,
            u8::try_from(
                legal
                    .len()
                    .min(crate::query::sha_target_limit_for(tx.state(), player) as usize),
            )
            .unwrap_or(u8::MAX),
        )
    };
    open_choice(
        tx,
        player,
        min_count,
        max_count,
        true,
        legal.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::VirtualShaTargets,
        vec![
            player.raw(),
            card.raw(),
            u32::from(ignore_distance),
            u32::from(ignore_use_count),
            u32::from(draw_on_no_use),
            skill.raw(),
            lose_hp_before_use,
            damage_bonus,
        ],
    )
}

pub(crate) fn choose_use_virtual_card(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_use_virtual_card_impl(tx, player, kind, skill, VirtualCardTracking::None, true)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum VirtualCardTracking {
    None,
    SourceFree,
    Dcctjiuxian,
}

pub(crate) fn choose_use_virtual_card_impl(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    skill: engine_api::SkillId,
    tracking: VirtualCardTracking,
    suppress_phase_history: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .filter(|kind| {
            matches!(
                kind,
                CardKind::TrickWanjian | CardKind::TrickJuedou | CardKind::TrickNanman
            )
        })
        .ok_or_else(|| {
            EffectError::ContentFault(
                "choose-use virtual card supports only Wanjian, Juedou or Nanman".into(),
            )
        })?;
    if !originating_skill_is_enabled(tx, player, skill)
        || !crate::command::play::source_free_card_has_legal_use(tx.state(), player, kind)
    {
        return Ok(Vec::new());
    }
    if suppress_phase_history {
        tx.state_mut()
            .turn
            .set_flag(turn_flag::PHASE_HISTORY_SUPPRESSED, 1);
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::FinishPhaseEndVirtualCard);
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .expect("validated processing zone")
        .cards
        .push(card);
    match tracking {
        VirtualCardTracking::None => {}
        VirtualCardTracking::SourceFree => tx
            .state_mut()
            .skills
            .get_mut(skill)
            .expect("validated originating skill")
            .set_source_free_physical_card(card),
        VirtualCardTracking::Dcctjiuxian => {
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.add_tracked_cards(&[card]))
            {
                return Err(EffectError::ContentFault(
                    "Dcctjiuxian could not track its virtual Duel".into(),
                ));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDcctjiuxian {
                    player,
                    event_player: PlayerId::NONE,
                    target: PlayerId::NONE,
                    card,
                    action: 6,
                    cards: Vec::new(),
                    skill,
                },
            );
        }
    }
    match crate::command::play::skill_card_target_spec(tx, player, card)? {
        crate::command::play::SkillCardTargetSpec::Immediate(targets) => {
            if tracking == VirtualCardTracking::Dcctjiuxian {
                let target = targets.first().copied().ok_or_else(|| {
                    EffectError::ContentFault("Dcctjiuxian Duel requires one target".into())
                })?;
                if !tx
                    .state_mut()
                    .skills
                    .get_mut(skill)
                    .is_some_and(|instance| instance.record_active_play_phase_target(target))
                {
                    return Err(EffectError::ContentFault(
                        "Dcctjiuxian could not record its Duel target".into(),
                    ));
                }
            }
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_CARD, card.raw() as i32);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_SKILL, skill.raw() as i32);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, i32::from(kind.tag()));
            let result = crate::command::play::handle_skill_card(tx, player, card, &targets);
            tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_CARD, 0);
            tx.state_mut().turn.set_flag(turn_flag::VIEW_AS_SKILL, 0);
            tx.state_mut()
                .turn
                .set_flag(turn_flag::VIEW_AS_LOGICAL_KIND, 0);
            result?;
            Ok(Vec::new())
        }
        crate::command::play::SkillCardTargetSpec::Select { min, max, options } => {
            let mut payload = vec![
                player.raw(),
                card.raw(),
                kind.tag().into(),
                skill.raw(),
                options.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(options.iter().map(|target| target.raw()));
            if tracking == VirtualCardTracking::Dcctjiuxian {
                payload.push(1);
            }
            open_choice(
                tx,
                player,
                min,
                max,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseUseVirtualCard,
                payload,
            )
        }
        crate::command::play::SkillCardTargetSpec::Pairs(_) => Err(EffectError::ContentFault(
            "choose-use virtual card does not support paired targets".into(),
        )),
    }
}

pub(crate) fn use_virtual_card_on(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::ContentFault("unknown virtual card kind".into()))?;
    if !tx.state().players.get(player).is_some_and(|state| {
        state.alive && (!state.dying || (kind == CardKind::Jiu && player == target))
    }) || !tx
        .state()
        .players
        .get(target)
        .is_some_and(|state| state.alive)
    {
        return Ok(Vec::new());
    }
    let card = materialize_virtual_card(tx, player, kind)?;
    crate::command::play::play_as_kind(
        tx,
        player,
        card,
        kind,
        &[target],
        false,
        false,
        false,
        false,
    )?;
    Ok(Vec::new())
}

pub(crate) fn use_virtual_card_on_selected_targets(
    tx: &mut Transaction,
    player: PlayerId,
    kind: u8,
    targets: Vec<PlayerId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let kind = CardKind::from_tag(kind)
        .ok_or_else(|| EffectError::ContentFault("unknown virtual card kind".into()))?;
    let card = materialize_virtual_card(tx, player, kind)?;
    match kind {
        CardKind::TrickJuedou => {
            crate::command::play::play_source_free_multi_target_juedou(tx, player, card, &targets)?;
        }
        CardKind::TrickShunshou => {
            crate::command::play::play_source_free_multi_target_shunshou(
                tx, player, card, &targets,
            )?;
        }
        CardKind::TrickTiesuo => {
            crate::command::play::play_as_kind(
                tx, player, card, kind, &targets, false, false, false, false,
            )?;
        }
        CardKind::Sha
        | CardKind::TrickGuohe
        | CardKind::TrickWugu
        | CardKind::TrickDiaobingqianjiang => {
            crate::command::play::play_as_kind(
                tx, player, card, kind, &targets, false, false, false, false,
            )?;
        }
        _ => {
            return Err(EffectError::ContentFault(
                "selected-target virtual card kind unsupported".into(),
            ));
        }
    }
    Ok(Vec::new())
}

pub(crate) fn materialize_virtual_card(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
) -> Result<CardId, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no virtual card user hand".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);
    move_one_card(tx, card, processing, hand, player, false)?;
    Ok(card)
}

pub(crate) fn try_use_virtual_sha_on_without_distance(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    ignore_use_count: bool,
    damage_bonus: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no processing zone".into()))?;
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no removed zone".into()))?;
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind: CardKind::Sha,
        suit: engine_api::Suit::None,
        rank: 0,
        weapon_range: 0,
        zone: processing,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(processing)
        .unwrap()
        .cards
        .push(card);
    let use_count_allowed = ignore_use_count
        || crate::query::sha_play_phase_use_count(tx.state())
            < crate::query::sha_use_limit_for_card(tx.state(), source, card)
        || crate::query::sha_use_unlimited_for_target(tx.state(), source, target);
    if crate::query::card_use_cancelled_for_card(tx.state(), source, card)
        || !use_count_allowed
        || crate::command::play::validate_sha_target_with_options(tx, source, card, target, true)
            .is_err()
    {
        move_one_card(tx, card, processing, removed, PlayerId::NONE, true)?;
        return Ok(Vec::new());
    }
    crate::command::play::handle_virtual_sha_with_policies_and_bonus(
        tx,
        source,
        card,
        &[target],
        true,
        ignore_use_count,
        damage_bonus,
    )?;
    Ok(Vec::new())
}

/// 雷劫（releijie）：对目标判定——黑桃 2-9 则其摸两张牌，否则对其使用两张雷【杀】。
/// 上游 (sp/skill.js:19804)：target.judge(spade 2-9 → 有利)；判定成功 target.draw(2)，
/// 失败则 player 对 target 使用至多两张雷【杀】（每次可用才用）。
pub(crate) fn judge_spade_two_to_nine_draw_two_or_thunder_sha_twice(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("bad releijie judging player".into()));
    }
    let result = crate::command::judge::flip_judgment(
        tx,
        target,
        crate::command::judge::JudgmentFavor::Neutral,
    )?;
    let suit = result.suit;
    let rank = result.rank;
    tx.push_event(DomainEvent::RuleNote {
        code: 236, // standalone skill judgment (releijie)
        payload: vec![
            target.raw(),
            crate::command::judge::suit_tag(suit),
            u32::from(rank),
        ],
    });
    let emit = EffectRequest::EmitJudgmentResolved {
        player: target,
        result_card: result.card,
        suit,
        rank,
        cause_card: CardId::NONE,
    };
    crate::effect::run_effects(tx, vec![emit])?;
    let spade_two_to_nine = suit == engine_api::Suit::Spade && (2..=9).contains(&rank);
    if spade_two_to_nine {
        return crate::effect::run_effects(
            tx,
            vec![EffectRequest::DrawCards {
                player: target,
                count: 2,
            }],
        );
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    for _ in 0..2 {
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::NONE,
            kind: CardKind::ShaThunder,
            suit: engine_api::Suit::None,
            rank: 0,
            weapon_range: 0,
            zone: processing,
            owner: PlayerId::NONE,
            face_up: true,
        });
        tx.state_mut()
            .zones
            .get_mut(processing)
            .unwrap()
            .cards
            .push(card);
        if crate::command::play::validate_sha_target_with_options(tx, source, card, target, true)
            .is_err()
        {
            break;
        }
        crate::command::play::handle_virtual_sha_with_policies_and_bonus(
            tx,
            source,
            card,
            &[target],
            true,
            false,
            0,
        )?;
    }
    Ok(Vec::new())
}

pub(crate) fn deal_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    is_lose_hp: bool,
    nature: u8,
    logical_sha: bool,
    card: CardId,
    card_kind: u8,
    card_flags: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(mut prepared) = prepare_damage(
        tx.state(),
        source,
        target,
        amount,
        is_lose_hp,
        nature,
        logical_sha,
        card,
    )?
    else {
        return Ok(Vec::new());
    };
    let EffectRequest::CommitDamage {
        card: prepared_card,
        card_kind: prepared_kind,
        card_flags: prepared_flags,
        ..
    } = &mut prepared
    else {
        unreachable!();
    };
    *prepared_card = card;
    *prepared_kind = card_kind;
    *prepared_flags = card_flags;
    let EffectRequest::CommitDamage {
        damage_id,
        source,
        target,
        amount,
        base_amount,
        is_lose_hp,
        nature,
        card,
        card_kind,
        card_flags,
    } = prepared
    else {
        unreachable!();
    };
    let (mut events, followups) = commit_damage(
        tx,
        damage_id,
        source,
        target,
        amount,
        base_amount,
        is_lose_hp,
        nature,
        card,
        card_kind,
        card_flags,
    )?;
    for followup in followups {
        events.extend(apply_effect(tx, followup)?);
    }
    Ok(events)
}

/// Resolve static damage modifiers without mutating state.
/// 主动原语 (批 270, ADR 0641 §8.58)：断索——先重置所选全部连环角色
/// （移除 MARK_CHAIN），再对其各造成 1 点火焰伤害。两遍循环与上游 content
/// 一致：重置完成后伤害不再经铁索传导。
pub(crate) fn reset_linked_selected_targets(
    tx: &mut Transaction,
    source: PlayerId,
    targets: &[PlayerId],
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut events = Vec::new();
    for target in targets {
        if tx
            .state()
            .players
            .get(*target)
            .is_some_and(|participant| participant.alive)
        {
            events.extend(super::remove_mark(
                tx,
                *target,
                crate::content::MARK_CHAIN,
                0,
            )?);
        }
    }
    for target in targets {
        if tx
            .state()
            .players
            .get(*target)
            .is_some_and(|participant| participant.alive)
        {
            events.extend(deal_damage(
                tx,
                source,
                *target,
                1,
                false,
                1,
                false,
                CardId::NONE,
                0,
                0,
            )?);
        }
    }
    Ok(events)
}

pub(crate) fn prepare_damage(
    state: &crate::state::GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    is_lose_hp: bool,
    nature: u8,
    logical_sha: bool,
    card: CardId,
) -> Result<Option<EffectRequest>, EffectError> {
    let p = state
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if amount == 0 || !p.alive {
        return Ok(None);
    }
    let source = crate::query::nanman_damage_source_for(state, source, card).unwrap_or(source);
    let amount = if logical_sha {
        crate::query::sha_damage_source_range_bonus_for(state, source, amount)
    } else {
        amount
    };
    let intent = crate::intent::resolve_damage_intent_with_kind(
        state,
        crate::intent::DamageIntent {
            source,
            target,
            amount,
            is_lose_hp,
            nature,
            cancelled: false,
        },
        logical_sha,
    );
    if intent.cancelled || intent.amount == 0 {
        return Ok((!intent.is_lose_hp).then_some(EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount: 0,
            base_amount: 0,
            is_lose_hp: false,
            nature: intent.nature,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }));
    }
    let amount = intent.amount;
    let is_lose_hp = intent.is_lose_hp;
    let nature = intent.nature;
    // 藤甲：火焰伤害 +1（失去体力通道不触发）；传导时各目标独立结算。
    let base_amount = amount;
    let mut amount = if is_lose_hp {
        amount
    } else {
        crate::query::damage_taken_amount_for(state, source, target, nature, amount)
    };
    if amount == 0 {
        return Ok(Some(EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount: 0,
            base_amount,
            is_lose_hp: false,
            nature,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }));
    }
    if !is_lose_hp
        && nature == rule_api::damage_nature::FIRE
        && crate::content::player_has_tengjia(state, target)
        && !crate::query::armor_effect_ignored_for_card(state, source, target, card)
    {
        amount = amount.saturating_add(1);
    }
    if !is_lose_hp {
        amount = crate::query::damage_taken_final_amount_for(state, target, nature, amount);
        if logical_sha {
            amount = crate::query::sha_damage_source_range_cap_for(state, source, amount);
        }
        if amount == 0 {
            return Ok(Some(EffectRequest::CommitDamage {
                damage_id: 0,
                source,
                target,
                amount: 0,
                base_amount,
                is_lose_hp: false,
                nature,
                card: CardId::NONE,
                card_kind: 0,
                card_flags: 0,
            }));
        }
    }
    Ok(Some(EffectRequest::CommitDamage {
        damage_id: 0,
        source,
        target,
        amount,
        base_amount,
        is_lose_hp,
        nature,
        card: CardId::NONE,
        card_kind: 0,
        card_flags: 0,
    }))
}

/// Commit already prepared damage and return raw chained-damage follow-ups.
pub(crate) fn commit_damage(
    tx: &mut Transaction,
    damage_id: u32,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    base_amount: u32,
    is_lose_hp: bool,
    nature: u8,
    card: CardId,
    card_kind: u8,
    card_flags: u8,
) -> Result<(Vec<DomainEvent>, Vec<EffectRequest>), EffectError> {
    if damage_id != 0 {
        if tx.state().resolution.current_damage_id == damage_id {
            tx.state_mut().resolution.current_damage_id = 0;
        }
        if let Some(index) = tx
            .state()
            .resolution
            .prevented_damage_ids
            .iter()
            .position(|id| *id == damage_id)
        {
            tx.state_mut().resolution.prevented_damage_ids.remove(index);
            return Ok((
                vec![DomainEvent::DamagePrevented {
                    source,
                    target,
                    current: tx.state().turn.current,
                }],
                Vec::new(),
            ));
        }
    }
    if amount == 0 {
        return Ok((
            (!is_lose_hp)
                .then_some(DomainEvent::DamagePrevented {
                    source,
                    target,
                    current: tx.state().turn.current,
                })
                .into_iter()
                .collect(),
            Vec::new(),
        ));
    }
    let p = tx
        .state()
        .players
        .get(target)
        .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
    if !p.alive {
        return Ok((Vec::new(), Vec::new()));
    }
    let mut amount = amount;
    let mut events = Vec::new();
    if !is_lose_hp {
        let hujia = tx
            .state()
            .players
            .get(target)
            .map(|p| {
                p.marks
                    .iter()
                    .find(|(mark, _)| *mark == crate::content::MARK_HUJIA)
                    .map_or(0, |(_, count)| (*count).max(0) as u32)
            })
            .unwrap_or(0);
        let absorbed = hujia.min(amount);
        if absorbed > 0 {
            events.extend(remove_mark(
                tx,
                target,
                crate::content::MARK_HUJIA,
                absorbed,
            )?);
            amount -= absorbed;
            if amount == 0 {
                events.push(DomainEvent::DamagePrevented {
                    source,
                    target,
                    current: tx.state().turn.current,
                });
                return Ok((events, Vec::new()));
            }
        }
    }
    let (prev_hp, prev_dying) = tx
        .state()
        .players
        .get(target)
        .map(|p| (p.hp, p.dying))
        .unwrap_or((0, false));

    tx.journal(JournalEntry::PlayerHp {
        id: target,
        prev: prev_hp,
    });
    {
        let p = tx
            .state_mut()
            .players
            .get_mut(target)
            .ok_or_else(|| EffectError::Illegal("bad target".into()))?;
        // HP may go negative while dying (noname-style); do not saturating-floor at 0.
        p.hp -= amount as i32;
    }
    if tx.state().content.tracks_turn_hp_change_history() {
        tx.state_mut().turn.record_hp_change(target);
    }
    if tx.state().content.tracks_damage_taken_count() && !is_lose_hp {
        tx.state_mut().turn.record_damage_taken(target);
    }
    if tx.state().content.tracks_phase_target_hp_history()
        && tx.state().turn.flag(turn_flag::PHASE_HISTORY_SUPPRESSED) == 0
    {
        tx.state_mut().turn.mark_phase_hp_changed(target);
    }
    if !is_lose_hp && card.is_some() && CardKind::from_tag(card_kind).is_some_and(CardKind::is_sha)
    {
        let logical_source = if source.is_some() {
            source
        } else {
            tx.state().resolution.logical_sha_source(card)
        };
        if logical_source.is_some() {
            crate::command::play::mark_current_sha_dealt_damage(tx, logical_source, card);
        }
    }
    if !is_lose_hp && card.is_some() && CardKind::from_tag(card_kind) == Some(CardKind::TrickJuedou)
    {
        tx.state_mut().resolution.record_duel_damage(card);
    }
    if !is_lose_hp {
        if card.is_some() {
            if tx.state().content.tracks_turn_haoyi_cards() {
                let materials = tx.state().resolution.card_source_cards_for_card(card);
                tx.state_mut().turn.record_damage_card_materials(materials);
            }
            tx.state_mut()
                .resolution
                .record_card_resolution_damage(card, target);
        }
        if source.is_some() {
            tx.state_mut().damage_history.insert((source, target));
            if tx.state().content.tracks_turn_damage_pair_counts() {
                tx.state_mut().turn.record_damage_pair(source, target);
            }
            let total = tx
                .state_mut()
                .damage_dealt_totals
                .entry(source)
                .or_insert(0);
            *total = total.saturating_add(amount);
            if card_kind == CardKind::Sha.tag() {
                let key = turn_flag::sha_damage_dealt_by_player(source);
                let count = tx.state().turn.flag(key).saturating_add(1);
                tx.state_mut().turn.set_flag(key, count);
            }
        }
        let source_is_current = source.is_some() && source == tx.state().turn.current;
        let damaged_other = source_is_current && source != target;
        let damaged_other_qun =
            source_is_current && source != target && tx.state().player_group(target) == Some("qun");
        tx.journal(JournalEntry::TurnCheckpoint);
        tx.state_mut().turn.mark_player_damaged(target);
        if source.is_some() {
            tx.state_mut().turn.mark_player_dealt_damage(source);
            // ADR 0644 P3：逐源受伤位图（仅当加载内容使用 align_debate_opinions
            // 时记录——保持无关包的 canonical state hash 不变）。
            if tx.state().content.tracks_damaged_by_player_bitset() && source != target {
                if let Some(seat) = tx.state().players.get(target).map(|p| p.seat.raw()) {
                    let key = turn_flag::damaged_by_player(source);
                    let bits = tx.state().turn.flag(key) as u32;
                    tx.state_mut()
                        .turn
                        .set_flag(key, (bits | (1 << seat)) as i32);
                }
            }
        }
        if source_is_current {
            let dealt = tx
                .state()
                .turn
                .flag(turn_flag::DAMAGE_DEALT)
                .saturating_add(i32::try_from(amount).unwrap_or(i32::MAX));
            tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, dealt);
            if damaged_other {
                tx.state_mut()
                    .turn
                    .set_flag(turn_flag::DAMAGE_DEALT_OTHER, 1);
            }
            if damaged_other_qun {
                tx.state_mut()
                    .turn
                    .set_flag(turn_flag::DAMAGE_DEALT_OTHER_QUN, 1);
            }
        }
    }
    let had_chain = !is_lose_hp
        && matches!(
            nature,
            rule_api::damage_nature::FIRE | rule_api::damage_nature::THUNDER
        )
        && tx
            .state()
            .players
            .get(target)
            .map(|p| {
                p.marks
                    .iter()
                    .any(|(m, c)| *m == crate::content::MARK_CHAIN && *c > 0)
            })
            .unwrap_or(false);

    events.push(DomainEvent::DamageDealt {
        source,
        target,
        amount,
        is_lose_hp,
        nature,
    });
    let p = tx.state().players.get(target).unwrap();
    if p.hp <= 0 && !prev_dying && !tx.state().resolution.gzbuqu_nodying {
        tx.journal(JournalEntry::PlayerLife {
            id: target,
            prev_alive: true,
            prev_dying: false,
        });
        tx.state_mut().players.get_mut(target).unwrap().dying = true;
        events.push(DomainEvent::EnteredDying {
            player: target,
            source,
        });
    }

    // 铁索：传导伤害一次，并解除所有横置（属性一并传导）
    let mut followups = Vec::new();
    if !is_lose_hp
        && card_flags & rule_api::damage_card_flag::BEIZHU != 0
        && card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0
    {
        followups.push(EffectRequest::DrawCards {
            player: target,
            count: amount,
        });
    }
    if had_chain {
        let chained: Vec<PlayerId> = tx
            .state()
            .players
            .iter_live()
            .filter(|p| {
                p.alive
                    && p.id != target
                    && p.marks
                        .iter()
                        .any(|(m, c)| *m == crate::content::MARK_CHAIN && *c > 0)
            })
            .map(|p| p.id)
            .collect();
        let all_chained: Vec<PlayerId> = tx
            .state()
            .players
            .iter_live()
            .filter(|p| {
                p.marks
                    .iter()
                    .any(|(m, c)| *m == crate::content::MARK_CHAIN && *c > 0)
            })
            .map(|p| p.id)
            .collect();
        for pid in all_chained {
            let _ = remove_mark(tx, pid, crate::content::MARK_CHAIN, 0);
        }
        for o in chained {
            followups.push(EffectRequest::DealCardDamage {
                source,
                target: o,
                amount: base_amount,
                is_lose_hp: false,
                nature,
                card,
                card_kind,
                card_flags: card_flags | rule_api::damage_card_flag::CHAIN_PROPAGATED,
            });
        }
    }
    Ok((events, followups))
}

/// Commit ordinary source-free damage and ban the originating active skill only
/// when this exact commit creates the target's dying state.
pub(crate) fn commit_damage_ban_active_if_dying(
    tx: &mut Transaction,
    damage_id: u32,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    base_amount: u32,
    nature: u8,
    skill: engine_api::SkillId,
) -> Result<(Vec<DomainEvent>, Vec<EffectRequest>), EffectError> {
    let instance =
        tx.state().skills.get(skill).ok_or_else(|| {
            EffectError::ContentFault("damage-linked active skill missing".into())
        })?;
    if instance.owner != source {
        return Err(EffectError::ContentFault(
            "damage-linked active skill owner mismatch".into(),
        ));
    }

    let committed = commit_damage(
        tx,
        damage_id,
        source,
        target,
        amount,
        base_amount,
        false,
        nature,
        CardId::NONE,
        0,
        0,
    )?;
    if committed
        .0
        .iter()
        .any(|event| matches!(event, DomainEvent::EnteredDying { player, .. } if *player == target))
    {
        tx.state_mut()
            .skills
            .get_mut(skill)
            .expect("validated skill instance")
            .active_turn_banned = true;
    }
    Ok(committed)
}

/// Commit a skill owner's HP loss, charge one maximum HP only when that exact loss first enters
/// dying, and defer equal ordinary damage to another player until the dying/save window finishes.
pub(crate) fn commit_lose_hp_then_damage_other_lose_max_hp_if_dying(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<(Vec<DomainEvent>, Vec<EffectRequest>), EffectError> {
    let instance =
        tx.state().skills.get(skill).ok_or_else(|| {
            EffectError::ContentFault("HP-loss-linked active skill missing".into())
        })?;
    if instance.owner != player {
        return Err(EffectError::ContentFault(
            "HP-loss-linked active skill owner mismatch".into(),
        ));
    }
    if player == target {
        return Err(EffectError::ContentFault(
            "HP-loss-linked damage requires another target".into(),
        ));
    }

    let (mut events, followups) = commit_damage(
        tx,
        0,
        PlayerId::NONE,
        player,
        amount,
        amount,
        true,
        rule_api::damage_nature::NORMAL,
        CardId::NONE,
        0,
        0,
    )?;
    debug_assert!(followups.is_empty(), "HP loss cannot propagate a chain");

    let entered_dying = events.iter().any(
        |event| matches!(event, DomainEvent::EnteredDying { player: dying, .. } if *dying == player),
    );
    if entered_dying {
        events.extend(change_max_hp(tx, player, -1)?);
    }

    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::DealDamage {
            source: player,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        });
    Ok((events, Vec::new()))
}
