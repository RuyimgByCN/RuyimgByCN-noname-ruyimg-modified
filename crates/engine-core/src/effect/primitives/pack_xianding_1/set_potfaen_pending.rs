//! Effect primitives for `set_potfaen_pending` and related helpers.

use super::*;

pub(crate) fn set_potfaen_pending(tx: &mut Transaction, skill: engine_api::SkillId, pending: u8) {
    let (last, _) = potfaen_state(tx, skill);
    set_potfaen_state(tx, skill, last, pending);
}

pub(crate) fn queue_extra_draw_phase(tx: &mut Transaction, player: PlayerId) {
    tx.state_mut().turn.extra_draw_phase_queue.push(player);
}

pub(crate) fn begin_potsifeng_distribution(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let max = u8::try_from(cards.len()).unwrap_or(u8::MAX);
    let mut payload = vec![POTSIFENG_CUSTOM_CHOICE, 0, player.raw(), skill.raw()];
    payload.extend(cards.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        1,
        max,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn commit_potsifeng_distribution(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    selected: Vec<CardId>,
    remaining: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let skill_def = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.def.raw())
        .ok_or_else(|| EffectError::Illegal("伺锋 skill disappeared".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
    let pile = ensure_skill_pile_zone(tx, target, skill_def)?;
    let mut events = move_cards(tx, selected, pile, Some(processing))?;
    events.extend(begin_potsifeng_distribution(tx, player, skill, remaining)?);
    Ok(events)
}

pub(crate) fn commit_potbihan(
    tx: &mut Transaction,
    target: PlayerId,
    damage_id: u32,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options =
        crate::query::discardable_target_cards(tx.state(), target, target, TargetCardScope::Hand);
    if cards.is_empty() || cards.iter().any(|card| !options.contains(card)) {
        return Err(EffectError::Illegal(
            "避憾 hand selection became invalid".into(),
        ));
    }
    let events = discard_cards(tx, target, cards)?;
    reduce_bound_damage_by_one(tx, damage_id)?;
    Ok(events)
}

pub(crate) fn resolve_potdimeng(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let [left, right] = targets.as_slice() else {
        return Err(EffectError::Illegal(
            "缔盟 requires exactly two targets".into(),
        ));
    };
    if !originating_skill_is_enabled(tx, player, skill)
        || left == right
        || !tx
            .state()
            .players
            .get(*left)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(*right)
            .is_some_and(|participant| participant.alive)
        || tx
            .state()
            .hand_count(*left)
            .abs_diff(tx.state().hand_count(*right))
            > 3
    {
        return Err(EffectError::Illegal("缔盟 targets became illegal".into()));
    }

    let left_cards = tx.state().hand_cards(*left);
    let right_cards = tx.state().hand_cards(*right);
    let left_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, *left)
        .ok_or_else(|| EffectError::EngineFault("缔盟 left hand missing".into()))?;
    let right_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, *right)
        .ok_or_else(|| EffectError::EngineFault("缔盟 right hand missing".into()))?;
    let mut events = move_cards(tx, left_cards, right_hand, Some(left_hand))?;
    events.extend(move_cards(tx, right_cards, left_hand, Some(right_hand))?);

    let lost_hp = tx
        .state()
        .players
        .get(player)
        .map(|owner| (owner.max_hp as i32 - owner.hp).max(0) as u32)
        .unwrap_or(0);
    if lost_hp == 0 {
        return Ok(events);
    }
    let discardable = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let discard_count = lost_hp.min(discardable.len() as u32);
    let left_count = tx.state().hand_count(*left);
    let right_count = tx.state().hand_count(*right);
    let lower = if left_count < right_count {
        *left
    } else if right_count < left_count {
        *right
    } else {
        PlayerId::NONE
    };
    if discard_count == 0 && lower.is_none() {
        return Ok(events);
    }
    let mode = if discard_count == 0 {
        2
    } else if lower.is_none() {
        0
    } else {
        1
    };
    let (min, max, options, kind) = if mode == 2 {
        (1, 1, vec![1], ChoiceKind::Custom)
    } else {
        (
            discard_count as u8,
            discard_count as u8,
            discardable.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
        )
    };
    events.extend(open_choice(
        tx,
        player,
        min,
        max,
        true,
        options,
        kind,
        ContinuationKind::Custom,
        vec![
            POTDIMENG_CUSTOM_CHOICE,
            player.raw(),
            lower.raw(),
            lost_hp,
            discard_count,
            mode,
            skill.raw(),
        ],
    )?);
    Ok(events)
}

pub(crate) fn resolve_spmiaolve(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let used: Vec<_> = tx
                .state()
                .cards
                .sorted_live()
                .into_iter()
                .filter(|card| card.kind == CardKind::TrickMantianguohai)
                .map(|card| card.suit)
                .collect();
            let mut available: Vec<_> = [
                engine_api::Suit::Spade,
                engine_api::Suit::Heart,
                engine_api::Suit::Club,
                engine_api::Suit::Diamond,
            ]
            .into_iter()
            .filter(|suit| !used.contains(suit))
            .collect();
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("妙略 owner has no hand zone".into()))?;
            let mut cards = Vec::new();
            for _ in 0..2 {
                if available.is_empty() {
                    break;
                }
                let mut rng = crate::rng::RuleRng::from_state(tx.state().rng);
                let index = rng.gen_range_u32(available.len() as u32) as usize;
                tx.state_mut().rng = rng.into_state();
                let suit = available.remove(index);
                let card = tx.state_mut().cards.alloc(Card {
                    id: CardId::NONE,
                    def: CardDefId::new(MANTIANGUOHAI_CARD_DEF),
                    kind: CardKind::TrickMantianguohai,
                    suit,
                    rank: 5,
                    weapon_range: 0,
                    zone: hand,
                    owner: player,
                    face_up: false,
                });
                tx.state_mut()
                    .zones
                    .get_mut(hand)
                    .expect("validated hand")
                    .cards
                    .push(card);
                cards.push(card);
            }
            if cards.is_empty() {
                Ok(Vec::new())
            } else {
                Ok(vec![DomainEvent::CardsMoved {
                    cards,
                    from: ZoneId::NONE,
                    to: hand,
                    reason: engine_api::CardMoveReason::Unspecified,
                }])
            }
        }
        1 => open_choice(
            tx,
            player,
            1,
            1,
            true,
            vec![
                0,
                CardKind::TrickWuzhong.tag().into(),
                CardKind::TrickGuohe.tag().into(),
                CardKind::TrickWuxie.tag().into(),
            ],
            ChoiceKind::Custom,
            ContinuationKind::Custom,
            vec![SPMIAOLVE_CUSTOM_CHOICE, player.raw(), skill.raw()],
        ),
        _ => Err(EffectError::ContentFault("妙略 action is invalid".into())),
    }
}

pub(crate) fn parse_spyingjia_state(blob: &[u8]) -> (bool, bool, Vec<(u8, u32)>) {
    if blob.len() < 12 || &blob[..4] != b"SYJ1" || (blob.len() - 12) % 8 != 0 {
        return (false, false, Vec::new());
    }
    let used = u32::from_le_bytes(blob[4..8].try_into().unwrap()) != 0;
    let repeated = u32::from_le_bytes(blob[8..12].try_into().unwrap()) != 0;
    let names = blob[12..]
        .chunks_exact(8)
        .map(|entry| {
            (
                entry[0],
                u32::from_le_bytes(entry[4..8].try_into().unwrap()),
            )
        })
        .collect();
    (used, repeated, names)
}

pub(crate) fn write_spyingjia_state(used: bool, repeated: bool, names: &[(u8, u32)]) -> Vec<u8> {
    let mut blob = b"SYJ1".to_vec();
    blob.extend_from_slice(&u32::from(used).to_le_bytes());
    blob.extend_from_slice(&u32::from(repeated).to_le_bytes());
    for (kind, def) in names {
        blob.extend_from_slice(&u32::from(*kind).to_le_bytes());
        blob.extend_from_slice(&def.to_le_bytes());
    }
    blob
}

pub(crate) fn resolve_spyingjia(
    tx: &mut Transaction,
    player: PlayerId,
    action: u8,
    helper_def: engine_api::SkillDefId,
    card: CardId,
    card_kind: u8,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action == 3 {
        if !originating_skill_is_enabled(tx, player, skill) {
            return Ok(Vec::new());
        }
        let mut markers = tx
            .state()
            .skills
            .get(skill)
            .filter(|instance| instance.state_blob.get(..4) == Some(b"SYD1"))
            .map(|instance| instance.state_blob[4..].to_vec())
            .unwrap_or_default();
        if markers.is_empty() {
            return run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]);
        }
        let draw = markers.remove(0) != 0;
        if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
            instance.state_blob = [b"SYD1".as_slice(), markers.as_slice()].concat();
        }
        let mut effects = Vec::new();
        if draw {
            effects.push(EffectRequest::DrawCards { player, count: 2 });
        }
        if markers.is_empty() {
            effects.push(EffectRequest::RemoveSkill { skill });
        }
        return run_effects(tx, effects);
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let (used, repeated, mut names) = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| parse_spyingjia_state(&instance.state_blob))
        .unwrap_or_default();
    match action {
        0 => {
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob =
                write_spyingjia_state(used, false, &[]);
            Ok(Vec::new())
        }
        1 => {
            let Some(kind) = CardKind::from_tag(card_kind) else {
                return Ok(Vec::new());
            };
            if !kind.is_trick() {
                return Ok(Vec::new());
            }
            let def = tx
                .state()
                .cards
                .get(card)
                .map(|card| card.def.raw())
                .unwrap_or(0);
            let key = (kind.tag(), if kind == CardKind::Other { def } else { 0 });
            let repeated = repeated || names.contains(&key);
            if !names.contains(&key) {
                names.push(key);
                names.sort_unstable();
            }
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob =
                write_spyingjia_state(used, repeated, &names);
            Ok(Vec::new())
        }
        2 => {
            if used || !repeated || tx.state().hand_cards(player).is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                tx.state()
                    .hand_cards(player)
                    .into_iter()
                    .map(CardId::raw)
                    .collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    SPYINGJIA_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    helper_def.raw(),
                    skill.raw(),
                ],
            )
        }
        4 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if used
                || !repeated
                || tx.state().turn.extra_turn_queue.len() >= 8
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx
                    .state()
                    .cards
                    .get(card)
                    .is_none_or(|card| card.owner != player || Some(card.zone) != hand)
            {
                return Err(EffectError::Illegal(
                    "迎驾 settlement became invalid".into(),
                ));
            }
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob =
                write_spyingjia_state(true, repeated, &names);
            let existing_target_turns = tx
                .state()
                .turn
                .extra_turn_queue
                .iter()
                .filter(|queued| **queued == target)
                .count();
            let existing_helper = tx
                .state()
                .players
                .get(target)
                .into_iter()
                .flat_map(|participant| participant.skills.iter())
                .find(|candidate| {
                    tx.state()
                        .skills
                        .get(**candidate)
                        .is_some_and(|instance| instance.def == helper_def)
                })
                .copied();
            let mut markers = existing_helper
                .and_then(|helper| tx.state().skills.get(helper))
                .filter(|instance| instance.state_blob.get(..4) == Some(b"SYD1"))
                .map(|instance| instance.state_blob[4..].to_vec())
                .unwrap_or_default();
            markers.resize(markers.len().max(existing_target_turns), 0);
            markers.push(1);
            let helper_blob = [b"SYD1".as_slice(), markers.as_slice()].concat();
            let mut events = add_skill_with_state(tx, target, helper_def, Some(helper_blob))?;
            tx.state_mut().turn.extra_turn_queue.push(target);
            events.extend(run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                }],
            )?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault("迎驾 action is invalid".into())),
    }
}

pub(crate) fn resolve_mantianguohai(
    tx: &mut Transaction,
    player: PlayerId,
    stage: u8,
    payload: Vec<u32>,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            crate::command::play::advance_independent_trick_queue(tx, &payload).map(|_| Vec::new())
        }
        1 => crate::command::play::resolve_mantianguohai_give(
            tx,
            player,
            payload.into_iter().map(PlayerId::new).collect(),
        )
        .map(|_| Vec::new()),
        _ => Err(EffectError::EngineFault(
            "invalid 瞒天过海 continuation stage".into(),
        )),
    }
}

pub(crate) fn resolve_pothaoshi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
                .map(PlayerId::raw)
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                options,
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![POTHAOSHI_CUSTOM_CHOICE, player.raw(), skill.raw()],
            )
        }
        1 => {
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .filter(|instance| instance.owner == player)
                .ok_or_else(|| EffectError::Illegal("好施 skill disappeared".into()))?;
            instance.clear_pothaoshi_grants();
            Ok(Vec::new())
        }
        2 => {
            if target.is_none()
                || tx.state().hand_count(player) != 0
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|owner| owner.alive)
            {
                return Ok(Vec::new());
            }
            let consumed = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .filter(|instance| instance.owner == player)
                .is_some_and(|instance| instance.consume_pothaoshi_refill(target));
            if !consumed {
                return Ok(Vec::new());
            }
            draw_cards(tx, player, 3)
        }
        _ => Err(EffectError::ContentFault("invalid 好施 action".into())),
    }
}

pub(crate) fn resolve_mbyuanmo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    _card: CardId,
    value: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match stage {
        0 => {
            let mut options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .flat_map(|source| crate::query::movable_field_cards_from(tx.state(), source))
                .map(CardId::raw)
                .collect();
            options.sort_unstable();
            options.dedup();
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
                ContinuationKind::Custom,
                vec![MBYUANMO_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let amount = value
                .saturating_sub(attack_range_population(tx, target))
                .min(5);
            if amount == 0
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|role| role.alive)
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                vec![1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    MBYUANMO_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    amount,
                    skill.raw(),
                ],
            )
        }
        2 => draw_cards(tx, target, value.min(5)),
        _ => Err(EffectError::ContentFault("bad 远谟 stage".into())),
    }
}

pub(crate) fn resolve_mobiledanshou(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || tx.state().turn.current != target
        || tx.state().turn.phase != Phase::End
    {
        return Ok(Vec::new());
    }
    let count = tx.state().turn.card_target_count(target, player);
    if count == 0 {
        return draw_cards(tx, player, 1);
    }
    let cards = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let Ok(required) = u8::try_from(count) else {
        return Ok(Vec::new());
    };
    if cards.len() < usize::from(required)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        required,
        required,
        true,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            MOBILEDANSHOU_CUSTOM_CHOICE,
            player.raw(),
            target.raw(),
            skill.raw(),
            count,
        ],
    )
}

pub(crate) fn choose_other_excluding_source_random_gain_hand_equip_then_use_if_equip(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage == 1 {
        let Some(card_state) = tx.state().cards.get(card) else {
            return Ok(Vec::new());
        };
        let hand = tx.state().zones.find(ZoneKind::Hand, player);
        let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
        let can_use = hand == Some(card_state.zone)
            && card_state.owner == player
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
        return Ok(Vec::new());
    }
    if stage != 0 {
        return Err(EffectError::ContentFault("bad Jiebing stage".into()));
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            *target != player
                && *target != source
                && !crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    *target,
                    TargetCardScope::HandEquip,
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
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        vec![
            JIEBING_CUSTOM_CHOICE,
            player.raw(),
            source.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn resolve_gzjinfa(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cost_card: CardId,
    given_card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Err(EffectError::Illegal("invalid Jinfa context".into()));
    }
    match stage {
        0 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let equip = tx.state().zones.find(ZoneKind::Equip, player);
            if !tx.state().cards.get(cost_card).is_some_and(|state| {
                state.owner == player && (Some(state.zone) == hand || Some(state.zone) == equip)
            }) || crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty()
            {
                return Err(EffectError::Illegal("invalid Jinfa cost or target".into()));
            }
            super::super::run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![cost_card],
                    },
                    EffectRequest::ResolveGzjinfa {
                        player,
                        target,
                        cost_card,
                        given_card: CardId::NONE,
                        stage: 1,
                        skill,
                    },
                ],
            )
        }
        1 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let equip = tx.state().zones.find(ZoneKind::Equip, target);
            let mut options: Vec<_> = [hand, equip]
                .into_iter()
                .flatten()
                .flat_map(|zone| {
                    tx.state()
                        .zones
                        .get(zone)
                        .map(|state| state.cards.clone())
                        .unwrap_or_default()
                })
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| state.kind.is_equip())
                })
                .collect();
            options.sort_by_key(|card| card.raw());
            if options.is_empty() {
                let gainable = crate::query::gainable_target_cards(
                    tx.state(),
                    player,
                    target,
                    rule_api::TargetCardScope::HandEquip,
                );
                if gainable.is_empty() {
                    return Ok(Vec::new());
                }
                let mut payload = vec![
                    GZJINFA_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    gainable.len() as u32,
                ];
                payload.extend(gainable.iter().map(|card| card.raw()));
                return open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    gainable.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    payload,
                );
            }
            let mut payload = vec![
                GZJINFA_CUSTOM_CHOICE,
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            if !given_card.is_none()
                && tx.state().cards.get(given_card).is_some_and(|card| {
                    crate::query::effective_card_suit(tx.state(), target, card)
                        == engine_api::Suit::Spade
                })
            {
                try_use_virtual_sha_on_without_distance(tx, target, player, true, 0)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Err(EffectError::ContentFault("bad Jinfa stage".into())),
    }
}

pub(crate) fn resolve_wn_qifeng(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let lost = tx.state().turn.card_loss_count(player);
    if !originating_skill_is_enabled(tx, player, skill)
        || lost == 0
        || player == target
        || tx.state().turn.current != target
        || tx.state().turn.phase != Phase::End
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Ok(Vec::new());
    }
    try_use_virtual_sha_on_without_distance(tx, player, target, true, lost - 1)
}

pub(crate) fn dcjue_discard_count(tx: &Transaction) -> u32 {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    tx.state()
        .turn
        .discarded_cards
        .iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(**card)
                .is_some_and(|card| Some(card.zone) == discard)
        })
        .count()
        .min(u32::MAX as usize) as u32
}

pub(crate) fn dcjue_virtual_targets(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    kind: CardKind,
) -> Result<Vec<PlayerId>, EffectError> {
    let card = materialize_virtual_card(tx, player, kind)?;
    let spec = crate::command::play::skill_card_target_spec(tx, player, card);
    let options = match spec {
        Ok(crate::command::play::SkillCardTargetSpec::Immediate(options))
        | Ok(crate::command::play::SkillCardTargetSpec::Select { options, .. }) => options,
        Ok(crate::command::play::SkillCardTargetSpec::Pairs(_)) | Err(EffectError::Illegal(_)) => {
            Vec::new()
        }
        Err(error) => {
            remove_unused_virtual_card(tx, card)?;
            return Err(error);
        }
    };
    let targets = [player, target]
        .into_iter()
        .filter(|candidate| {
            options.contains(candidate)
                && (!matches!(kind, CardKind::Sha | CardKind::TrickGuohe) || *candidate != player)
        })
        .collect();
    remove_unused_virtual_card(tx, card)?;
    Ok(targets)
}

/// 举讹（杨彪，ABI 583）：一次选择后按仍在弃牌堆的本回合弃牌数循环；每轮以权威
/// RNG 打乱三种虚拟牌，使用第一个对本人/目标至少有一个合法目标的牌。
pub(crate) fn resolve_dcjue(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    remaining: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.phase != Phase::End
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
            if target != tx.state().turn.current
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || dcjue_discard_count(tx) == 0
                || tx
                    .state()
                    .skills
                    .get(skill)
                    .is_some_and(|instance| instance.round_fire_round == tx.state().turn.round)
            {
                return Ok(Vec::new());
            }
            if target == player {
                let options = tx
                    .state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter(|candidate| *candidate != player)
                    .map(PlayerId::raw)
                    .collect::<Vec<_>>();
                open_choice(
                    tx,
                    player,
                    0,
                    1,
                    true,
                    options,
                    ChoiceKind::SelectTargets,
                    ContinuationKind::Custom,
                    vec![RESOLVE_DCJUE_CHOICE, player.raw(), skill.raw()],
                )
            } else {
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Confirm,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_DCJUE_CHOICE,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ],
                )
            }
        }
        1 => {
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let round = tx.state().turn.round;
            if tx
                .state()
                .skills
                .get(skill)
                .is_some_and(|instance| instance.round_fire_round == round)
            {
                return Ok(Vec::new());
            }
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.round_fire_round = round;
            }
            let cap = tx
                .state()
                .players
                .get(target)
                .map(|participant| participant.max_hp)
                .unwrap_or(0);
            let count = dcjue_discard_count(tx).min(cap);
            run_effects(
                tx,
                vec![EffectRequest::ResolveDcjue {
                    player,
                    target,
                    stage: 2,
                    remaining: count,
                    skill,
                }],
            )
        }
        2 => {
            if remaining == 0
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let mut kinds = [CardKind::Sha, CardKind::TrickGuohe, CardKind::TrickWugu];
            let mut rng = RuleRng::from_state(tx.state().rng);
            tx.journal(JournalEntry::RngCheckpoint);
            rng.shuffle(&mut kinds);
            tx.state_mut().rng = rng.into_state();
            for kind in kinds {
                let targets = dcjue_virtual_targets(tx, player, target, kind)?;
                if targets.is_empty() {
                    continue;
                }
                return run_effects(
                    tx,
                    vec![
                        EffectRequest::UseVirtualCardOnSelectedTargets {
                            player,
                            kind: kind.tag(),
                            targets,
                        },
                        EffectRequest::ResolveDcjue {
                            player,
                            target,
                            stage: 2,
                            remaining: remaining - 1,
                            skill,
                        },
                    ],
                );
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad dcjue stage".into())),
    }
}

pub(crate) fn ruanji_has_jiu(tx: &Transaction, player: PlayerId) -> bool {
    tx.state().players.get(player).is_some_and(|player| {
        player
            .marks
            .iter()
            .any(|(mark, count)| *mark == crate::content::MARK_JIU && *count > 0)
    })
}

pub(crate) fn ruanji_discardable_hand(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                tx.state()
                    .zones
                    .get(card.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Hand)
            })
        })
        .collect()
}

/// 阮籍（ABI 584）：复用技能实体牌集合与 source-free 转化保存「昭文」展示牌；
/// 「酒遁」只改写当前【酒】的尾效果，或在黑牌目标窗口打开唯一一次上下文选择。
pub(crate) fn resolve_ruanji(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::Play {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let token = tx
                .state()
                .turn
                .round
                .saturating_mul(16)
                .saturating_add(tx.state().turn.current.raw());
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("昭文 skill disappeared".into()))?;
            let old = instance.tracked_cards();
            instance.take_tracked_cards(&old);
            if !instance.add_tracked_cards(&hand) {
                return Err(EffectError::ContentFault(
                    "昭文 cannot store tracked hand cards".into(),
                ));
            }
            instance.view_as_use_round = token;
            instance.view_as_used_branches = 0;
            Ok(Vec::new())
        }
        1 => {
            let tracked = tx
                .state()
                .skills
                .get(skill)
                .map_or_else(Vec::new, |skill| skill.tracked_cards());
            let count = cards
                .into_iter()
                .filter(|used| tracked.contains(used))
                .filter(|used| {
                    tx.state().cards.get(*used).is_some_and(|card| {
                        matches!(
                            crate::query::effective_card_suit(tx.state(), player, card),
                            engine_api::Suit::Heart | engine_api::Suit::Diamond
                        )
                    })
                })
                .count()
                .min(u32::MAX as usize) as u32;
            if count == 0 {
                return Ok(Vec::new());
            }
            run_effects(tx, vec![EffectRequest::DrawCards { player, count }])
        }
        2 => {
            if !card.is_some()
                || !tx.state().cards.get(card).is_some_and(|card| {
                    crate::query::effective_card_kind(tx.state(), source, card) == CardKind::Jiu
                })
            {
                return Ok(Vec::new());
            }
            let before = tx.state().resolution.deferred_tail_effects.len();
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .retain(|effect| match effect {
                    EffectRequest::RecoverHp {
                        source: effect_source,
                        player: target,
                        ..
                    } => !(*effect_source == source && *target == player),
                    EffectRequest::AddMark {
                        player: target,
                        mark,
                        ..
                    } => !(*target == player && mark.raw() == crate::content::MARK_JIU),
                    _ => true,
                });
            if before == tx.state().resolution.deferred_tail_effects.len() {
                return Err(EffectError::Illegal(
                    "酒遁 found no current Jiu effect to rewrite".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(crate::content::MARK_JIU),
                    delta: 1,
                }],
            )
        }
        3 => {
            if source == player || !card.is_some() {
                return Ok(Vec::new());
            }
            if ruanji_has_jiu(tx, player) {
                let options = ruanji_discardable_hand(tx, player);
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
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_RUANJI_CHOICE,
                        player.raw(),
                        source.raw(),
                        card.raw(),
                        skill.raw(),
                        1,
                    ],
                )
            } else {
                open_choice(
                    tx,
                    player,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Confirm,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_RUANJI_CHOICE,
                        player.raw(),
                        source.raw(),
                        card.raw(),
                        skill.raw(),
                        0,
                    ],
                )
            }
        }
        4 => {
            crate::command::play::exclude_saved_card_target(tx, card, player)?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad Ruanji action".into())),
    }
}

pub(crate) fn zhengbi_payment_cards(
    tx: &Transaction,
    target: PlayerId,
    non_basic: bool,
) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, target);
    crate::query::discardable_hand_equip_cards(tx.state(), target)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                let basic = crate::query::effective_card_kind(tx.state(), target, card).is_basic();
                if non_basic {
                    !basic
                } else {
                    basic && Some(card.zone) == hand
                }
            })
        })
        .collect()
}

pub(crate) fn write_zhengbi_target(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    target: PlayerId,
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob = [b"ZBI1".as_slice(), &target.raw().to_le_bytes()].concat();
    }
}

pub(crate) fn take_zhengbi_target(tx: &mut Transaction, skill: engine_api::SkillId) -> PlayerId {
    let target = tx
        .state()
        .skills
        .get(skill)
        .filter(|skill| skill.state_blob.len() == 8 && &skill.state_blob[..4] == b"ZBI1")
        .map(|skill| {
            PlayerId::new(u32::from_le_bytes(
                skill.state_blob[4..8].try_into().unwrap(),
            ))
        })
        .unwrap_or(PlayerId::NONE);
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob.clear();
    }
    target
}
