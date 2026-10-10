//! Effect primitive implementations — xianding 1 cluster.
//!
//! Split out of the former single-file `primitives.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the `apply_effect` dispatch in the parent module can reach it.

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

pub(crate) const SPMIAOLVE_CUSTOM_CHOICE: u32 = 0x5350_5391;
pub(crate) const SPYINGJIA_CUSTOM_CHOICE: u32 = 0x5359_5391;
pub(crate) const MANTIANGUOHAI_CARD_DEF: u32 = 3_442_665_819;

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
            super::run_effects(
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

pub(crate) fn zhengbi_payment_cards(tx: &Transaction, target: PlayerId, non_basic: bool) -> Vec<CardId> {
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

pub(crate) fn write_zhengbi_target(tx: &mut Transaction, skill: engine_api::SkillId, target: PlayerId) {
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

/// 崔琰毛玠「征辟」（ABI 585）。action 0 选择目标与分支；action 1 在本阶段结束时
/// 取得待结算目标的手牌/装备；action 2 交出基本牌；action 3 在赠牌触发完成后收取回礼。
pub(crate) fn resolve_zhengbi(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
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
                .filter(|candidate| *candidate != player)
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
                vec![RESOLVE_ZHENGBI_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let target = take_zhengbi_target(tx, skill);
            if !target.is_some()
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || !tx.state().turn.gained_card_in_play_phase(target)
            {
                return Ok(Vec::new());
            }
            let options = crate::query::gainable_target_cards(
                tx.state(),
                player,
                target,
                TargetCardScope::HandEquip,
            );
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            let equip = tx.state().zones.find(ZoneKind::Equip, target);
            let count = u32::from(options.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| Some(card.zone) == hand)
            })) + u32::from(options.iter().any(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| Some(card.zone) == equip)
            }));
            if count == 0 {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                count.min(u8::MAX as u32) as u8,
                count.min(u8::MAX as u32) as u8,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_ZHENGBI_CHOICE,
                    5,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            if player == target
                || !card.is_some()
                || !tx.state().cards.get(card).is_some_and(|value| {
                    value.owner == player
                        && Some(value.zone) == hand
                        && crate::query::effective_card_kind(tx.state(), player, value).is_basic()
                })
            {
                return Err(EffectError::Illegal("征辟赠牌已失效".into()));
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("征辟目标手牌区不存在".into()))?;
            let from = hand.ok_or_else(|| EffectError::Illegal("征辟手牌区不存在".into()))?;
            let events = give_cards_as(tx, player, vec![card], to, from)?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveZhengbi {
                    player,
                    target,
                    card: CardId::NONE,
                    action: 3,
                    skill,
                });
            Ok(events)
        }
        3 => {
            if ![player, target].into_iter().all(|participant| {
                tx.state()
                    .players
                    .get(participant)
                    .is_some_and(|state| state.alive)
            }) {
                return Ok(Vec::new());
            }
            let non_basic = zhengbi_payment_cards(tx, target, true);
            let basics = zhengbi_payment_cards(tx, target, false);
            let has_non_basic = !non_basic.is_empty();
            let has_two_basics = basics.len() >= 2;
            if has_non_basic && has_two_basics {
                return open_choice(
                    tx,
                    target,
                    1,
                    1,
                    false,
                    vec![0, 1],
                    ChoiceKind::Confirm,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_ZHENGBI_CHOICE,
                        3,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                    ],
                );
            }
            if has_non_basic || has_two_basics {
                let (mode, options, count) = if has_non_basic {
                    (0, non_basic, 1)
                } else {
                    (1, basics, 2)
                };
                return open_choice(
                    tx,
                    target,
                    count,
                    count,
                    false,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_ZHENGBI_CHOICE,
                        4,
                        player.raw(),
                        target.raw(),
                        skill.raw(),
                        mode,
                    ],
                );
            }
            let cards = tx.state().hand_cards(target);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("征辟拥有者手牌区不存在".into()))?;
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("征辟目标手牌区不存在".into()))?;
            give_cards_as(tx, target, cards, to, from)
        }
        4 => {
            if player != target
                && tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                write_zhengbi_target(tx, skill, target);
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad Zhengbi action".into())),
    }
}

/// 崔琰毛玠「奉迎」（ABI 585）。辅助技能用 0/1 队列标记跳过此前已排队的同角色额外回合，
/// 只在本次奉迎插入的回合开始时补牌。
pub(crate) fn resolve_fengying(
    tx: &mut Transaction,
    player: PlayerId,
    helper_def: engine_api::SkillDefId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
            {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(player);
            let discardable: Vec<_> =
                crate::query::discardable_hand_equip_cards(tx.state(), player)
                    .into_iter()
                    .filter(|card| hand.contains(card))
                    .collect();
            if discardable.len() != hand.len() {
                return Ok(Vec::new());
            }
            if hand.is_empty() {
                return resolve_fengying(tx, player, helper_def, 1, skill);
            }
            let events = discard_cards(tx, player, hand)?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveFengying {
                    player,
                    helper_def,
                    action: 1,
                    skill,
                });
            Ok(events)
        }
        1 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
                || tx.state().turn.extra_turn_queue.len() >= 8
            {
                return Ok(Vec::new());
            }
            let queued = tx
                .state()
                .turn
                .extra_turn_queue
                .iter()
                .filter(|candidate| **candidate == player)
                .count();
            let existing = tx
                .state()
                .players
                .get(player)
                .into_iter()
                .flat_map(|participant| participant.skills.iter())
                .find(|candidate| {
                    tx.state()
                        .skills
                        .get(**candidate)
                        .is_some_and(|instance| instance.def == helper_def)
                })
                .copied();
            let mut markers = existing
                .and_then(|helper| tx.state().skills.get(helper))
                .filter(|instance| instance.state_blob.get(..4) == Some(b"FYH1"))
                .map(|instance| instance.state_blob[4..].to_vec())
                .unwrap_or_default();
            markers.resize(markers.len().max(queued), 0);
            markers.push(1);
            let blob = [b"FYH1".as_slice(), markers.as_slice()].concat();
            let events = add_skill_with_state(tx, player, helper_def, Some(blob))?;
            tx.state_mut().turn.extra_turn_queue.push(player);
            Ok(events)
        }
        2 => {
            let mut markers = tx
                .state()
                .skills
                .get(skill)
                .filter(|instance| {
                    instance.def == helper_def && instance.state_blob.get(..4) == Some(b"FYH1")
                })
                .map(|instance| instance.state_blob[4..].to_vec())
                .unwrap_or_default();
            if markers.is_empty() {
                return run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]);
            }
            let applies = markers.remove(0) != 0;
            let mut effects = Vec::new();
            if markers.is_empty() {
                effects.push(EffectRequest::RemoveSkill { skill });
            } else if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.state_blob = [b"FYH1".as_slice(), markers.as_slice()].concat();
            }
            if applies {
                let hp = tx
                    .state()
                    .players
                    .get(player)
                    .map(|state| state.hp)
                    .unwrap_or(0);
                let min_hp = tx
                    .state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .filter_map(|candidate| tx.state().players.get(candidate).map(|state| state.hp))
                    .min();
                let max_hp = tx
                    .state()
                    .players
                    .get(player)
                    .map(|state| state.max_hp)
                    .unwrap_or(0);
                let hand = tx.state().hand_cards(player).len().min(u32::MAX as usize) as u32;
                if min_hp == Some(hp) && max_hp > hand {
                    effects.push(EffectRequest::DrawCards {
                        player,
                        count: max_hp - hand,
                    });
                }
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault("bad Fengying action".into())),
    }
}

pub(crate) fn qiaojian_rank_sum(tx: &Transaction, cards: &[CardId]) -> u32 {
    cards
        .iter()
        .filter_map(|card| tx.state().cards.get(*card))
        .map(|card| u32::from(card.rank))
        .sum()
}

pub(crate) fn qiaojian_selection_valid(tx: &Transaction, player: PlayerId, cards: &[CardId]) -> bool {
    let allowed: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .collect();
    cards.iter().all(|card| allowed.contains(card))
        && cards
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == cards.len()
}

pub(crate) fn xicha_entries(blob: &[u8]) -> Option<Vec<(PlayerId, Vec<CardId>)>> {
    if blob.is_empty() {
        return Some(Vec::new());
    }
    if blob.get(..4) != Some(b"XCH1") {
        return None;
    }
    let mut cursor = 4;
    let mut entries = Vec::new();
    while cursor < blob.len() {
        if cursor + 5 > blob.len() {
            return None;
        }
        let source = PlayerId::new(u32::from_le_bytes(
            blob[cursor..cursor + 4].try_into().ok()?,
        ));
        let count = blob[cursor + 4] as usize;
        cursor += 5;
        if cursor + count.saturating_mul(4) > blob.len() {
            return None;
        }
        let cards = blob[cursor..cursor + count * 4]
            .chunks_exact(4)
            .map(|raw| CardId::new(u32::from_le_bytes(raw.try_into().unwrap())))
            .collect();
        cursor += count * 4;
        entries.push((source, cards));
    }
    Some(entries)
}

pub(crate) fn write_xicha_entries(
    instance: &mut crate::state::SkillInstance,
    entries: &[(PlayerId, Vec<CardId>)],
) {
    instance.state_blob = b"XCH1".to_vec();
    for (source, cards) in entries {
        instance
            .state_blob
            .extend_from_slice(&source.raw().to_le_bytes());
        instance
            .state_blob
            .push(cards.len().min(u8::MAX as usize) as u8);
        for card in cards.iter().take(u8::MAX as usize) {
            instance
                .state_blob
                .extend_from_slice(&card.raw().to_le_bytes());
        }
    }
}

pub(crate) fn gain_cards_by_zone(
    tx: &mut Transaction,
    player: PlayerId,
    cards: &[CardId],
) -> Result<Vec<DomainEvent>, EffectError> {
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("gain target hand missing".into()))?;
    let mut by_zone = std::collections::BTreeMap::<u32, Vec<CardId>>::new();
    for card in cards {
        if let Some(state) = tx.state().cards.get(*card) {
            if state.zone != hand {
                by_zone.entry(state.zone.raw()).or_default().push(*card);
            }
        }
    }
    let mut events = Vec::new();
    for (zone, cards) in by_zone {
        events.extend(move_cards_with_reason(
            tx,
            cards,
            hand,
            Some(ZoneId::new(zone)),
            CardMoveReason::Unspecified,
        )?);
    }
    Ok(events)
}

/// 傅干「巧谏」（ABI 586）。双方选择均完成后才公开并同时提交弃牌；后续状态由单个辅助
/// 技能承载，等待目标下回合开始再激活，回合结束自动移除。
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_qiaojian(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    helper_def: engine_api::SkillDefId,
    action: u8,
    cards: Vec<CardId>,
    split: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Play
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|state| state.alive)
            {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            open_choice(
                tx,
                player,
                0,
                options.len().min(u8::MAX as usize) as u8,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_QIAOJIAN_CHOICE,
                    0,
                    player.raw(),
                    target.raw(),
                    helper_def.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            if ![player, target].into_iter().all(|candidate| {
                tx.state()
                    .players
                    .get(candidate)
                    .is_some_and(|state| state.alive)
            }) || !qiaojian_selection_valid(tx, player, &cards)
            {
                return Err(EffectError::Illegal("巧谏选择已失效".into()));
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let mut payload = vec![
                RESOLVE_QIAOJIAN_CHOICE,
                1,
                player.raw(),
                target.raw(),
                helper_def.raw(),
                skill.raw(),
                cards.len() as u32,
            ];
            payload.extend(cards.into_iter().map(CardId::raw));
            open_choice(
                tx,
                target,
                0,
                options.len().min(u8::MAX as usize) as u8,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            let split = split.min(cards.len() as u32) as usize;
            let (owner_cards, target_cards) = cards.split_at(split);
            if !qiaojian_selection_valid(tx, player, owner_cards)
                || !qiaojian_selection_valid(tx, target, target_cards)
            {
                return Err(EffectError::Illegal("巧谏弃牌已失效".into()));
            }
            let mut events = Vec::new();
            if !owner_cards.is_empty() {
                events.extend(discard_cards(tx, player, owner_cards.to_vec())?);
            }
            if !target_cards.is_empty() {
                events.extend(discard_cards(tx, target, target_cards.to_vec())?);
            }
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveQiaojian {
                    player,
                    target,
                    helper_def,
                    action: 3,
                    cards,
                    split: split as u32,
                    skill,
                });
            Ok(events)
        }
        3 => {
            let split = split.min(cards.len() as u32) as usize;
            if qiaojian_rank_sum(tx, &cards[..split]) > qiaojian_rank_sum(tx, &cards[split..]) {
                tx.state_mut().resolution.deferred_tail_effects.push(
                    EffectRequest::ResolveQiaojian {
                        player,
                        target,
                        helper_def,
                        action: 4,
                        cards: Vec::new(),
                        split: 0,
                        skill,
                    },
                );
                return recover_hp(tx, player, player, 1);
            }
            let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
            let available: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| Some(state.zone) == discard)
                })
                .collect();
            if available.is_empty() {
                return Ok(Vec::new());
            }
            let events = gain_cards_by_zone(tx, player, &available)?;
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveQiaojian {
                    player,
                    target,
                    helper_def,
                    action: 7,
                    cards: available,
                    split: 0,
                    skill,
                });
            Ok(events)
        }
        4 => open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![1, 2],
            ChoiceKind::Confirm,
            ContinuationKind::Custom,
            vec![
                RESOLVE_QIAOJIAN_CHOICE,
                2,
                player.raw(),
                target.raw(),
                helper_def.raw(),
                skill.raw(),
            ],
        ),
        5 => {
            let Some(helper) = tx.state_mut().skills.get_mut(skill) else {
                return Ok(Vec::new());
            };
            if helper.owner != player || helper.def != helper_def || helper.state_blob.len() != 6 {
                return Ok(Vec::new());
            }
            let mode = helper.state_blob[4];
            if !(1..=3).contains(&mode) || helper.state_blob[..4] != [0, 0, 0, 0] {
                return Ok(Vec::new());
            }
            helper.state_blob[0] = u8::from(mode & 2 != 0);
            helper.state_blob[5] = 0;
            helper.temporary_until_turn_end_of = player;
            Ok(Vec::new())
        }
        6 => {
            let kind = tx
                .state()
                .cards
                .get(cards.first().copied().unwrap_or(CardId::NONE))
                .map(|card| crate::query::effective_card_kind(tx.state(), player, card));
            let bit = kind.and_then(|kind| {
                if kind.is_sha() {
                    Some(1)
                } else if kind.is_trick() && !kind.is_delayed_trick() {
                    Some(2)
                } else {
                    None
                }
            });
            let Some(bit) = bit else {
                return Ok(Vec::new());
            };
            let Some(helper) = tx.state_mut().skills.get_mut(skill) else {
                return Ok(Vec::new());
            };
            if helper.state_blob.len() != 6
                || helper.temporary_until_turn_end_of != player
                || helper.state_blob[4] & 1 == 0
                || helper.state_blob[5] & bit != 0
            {
                return Ok(Vec::new());
            }
            helper.state_blob[5] |= bit;
            let card = cards.first().copied().unwrap_or(CardId::NONE);
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                vec![1],
                ChoiceKind::Confirm,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_QIAOJIAN_CHOICE,
                    3,
                    player.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        7 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let retained: Vec<_> = cards
                .into_iter()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| Some(state.zone) == hand)
                })
                .collect();
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                if !instance.add_tracked_cards(&retained) {
                    return Err(EffectError::ContentFault("巧谏无法记录获得牌".into()));
                }
            }
            Ok(Vec::new())
        }
        8 => {
            let card = cards.first().copied().unwrap_or(CardId::NONE);
            let Some(card_state) = tx.state().cards.get(card) else {
                return Ok(Vec::new());
            };
            let kind = crate::query::effective_card_kind(tx.state(), player, card_state);
            let targets = crate::command::play::saved_targeted_card_targets(tx, player, card)?;
            tx.state_mut()
                .resolution
                .register_card_replay(crate::state::PendingCardReplay {
                    source_card: card,
                    player,
                    kind: kind.tag(),
                    targets,
                    skill,
                });
            Ok(Vec::new())
        }
        9 => {
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                instance.take_tracked_cards(&cards);
            }
            Ok(Vec::new())
        }
        10 => {
            let branch = split.min(2) as u8;
            let existing = tx
                .state()
                .players
                .get(target)
                .into_iter()
                .flat_map(|player| player.skills.iter())
                .find(|id| {
                    tx.state()
                        .skills
                        .get(**id)
                        .is_some_and(|instance| instance.def == helper_def)
                })
                .copied();
            if let Some(existing) = existing {
                let helper = tx.state_mut().skills.get_mut(existing).unwrap();
                if helper.state_blob.len() == 6 && helper.state_blob[..4] == [0, 0, 0, 0] {
                    helper.state_blob[4] |= branch;
                    return Ok(Vec::new());
                }
            }
            add_skill_with_state(tx, target, helper_def, Some(vec![0, 0, 0, 0, branch, 0]))
        }
        _ => Err(EffectError::ContentFault("bad Qiaojian action".into())),
    }
}

/// 傅干「析察」（ABI 586）。每个来源只有一份“下一次用牌”监听；同来源重复发动会合并实体牌。
pub(crate) fn resolve_xicha(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) || source.is_none() {
                return Ok(Vec::new());
            }
            let hand = tx.state().hand_cards(source);
            if hand.is_empty() {
                return Ok(Vec::new());
            }
            let count = hand.len().min(2) as u8;
            open_choice(
                tx,
                player,
                count,
                count,
                false,
                hand.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_XICHA_CHOICE,
                    player.raw(),
                    source.raw(),
                    skill.raw(),
                ],
            )
        }
        1 => {
            let mut entries = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| xicha_entries(&instance.state_blob))
                .unwrap_or_default();
            let Some(index) = entries.iter().position(|(saved, _)| *saved == source) else {
                return Ok(Vec::new());
            };
            let watched = entries.remove(index).1;
            if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
                write_xicha_entries(instance, &entries);
            }
            if !cards.iter().any(|used| watched.contains(used)) {
                return Ok(Vec::new());
            }
            crate::command::play::cancel_saved_card_use(tx, source, card)?;
            let movable: Vec<_> = cards
                .into_iter()
                .filter(|used| {
                    tx.state()
                        .cards
                        .get(*used)
                        .is_some_and(|state| state.owner != player)
                })
                .collect();
            gain_cards_by_zone(tx, player, &movable)
        }
        2 => {
            let source_hand = tx.state().zones.find(ZoneKind::Hand, source);
            let expected = tx.state().hand_cards(source).len().min(2);
            if expected == 0
                || cards.len() != expected
                || cards.iter().any(|card| {
                    !tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|state| Some(state.zone) == source_hand)
                })
            {
                return Err(EffectError::Illegal("析察选择已失效".into()));
            }
            let instance = tx
                .state_mut()
                .skills
                .get_mut(skill)
                .ok_or_else(|| EffectError::Illegal("析察技能已消失".into()))?;
            let mut entries = xicha_entries(&instance.state_blob).unwrap_or_default();
            if let Some((_, tracked)) = entries.iter_mut().find(|(saved, _)| *saved == source) {
                tracked.extend(cards);
                tracked.sort_unstable_by_key(|card| card.raw());
                tracked.dedup();
            } else {
                entries.push((source, cards));
                entries.sort_by_key(|(source, _)| source.raw());
            }
            write_xicha_entries(instance, &entries);
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad Xicha action".into())),
    }
}

/// 胡遵「斩涛 / 安境」（ABI 587）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_huzun(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(player)
                    .is_some_and(|owner| owner.alive)
            {
                return Ok(Vec::new());
            }
            if crate::query::physical_card_number_sum(tx.state(), card, &cards).is_none() {
                return Ok(Vec::new());
            }
            let result = crate::command::judge::flip_judgment(
                tx,
                player,
                crate::command::judge::JudgmentFavor::Neutral,
            )?;
            let mut continuation_cards = Vec::with_capacity(cards.len() + 1);
            continuation_cards.push(result.card);
            continuation_cards.extend(cards);
            run_effects(
                tx,
                vec![
                    EffectRequest::EmitJudgmentResolved {
                        player,
                        result_card: result.card,
                        suit: result.suit,
                        rank: result.rank,
                        cause_card: CardId::NONE,
                    },
                    EffectRequest::ResolveHuzun {
                        player,
                        source,
                        card,
                        action: 3,
                        cards: continuation_cards,
                        targets: Vec::new(),
                        skill,
                    },
                ],
            )?;
            Ok(Vec::new())
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let mut options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|target| target.hp < target.max_hp as i32)
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let count = tx
                .state()
                .skills
                .get(skill)
                .and_then(|instance| {
                    (instance.state_blob.len() == 8 && &instance.state_blob[..4] == b"ANJ1")
                        .then(|| u32::from_le_bytes(instance.state_blob[4..8].try_into().unwrap()))
                })
                .unwrap_or(0)
                .saturating_add(1);
            let mut blob = b"ANJ1".to_vec();
            blob.extend_from_slice(&count.to_le_bytes());
            tx.state_mut().skills.get_mut(skill).unwrap().state_blob = blob;
            let max = options.len().min(count as usize).min(usize::from(u8::MAX)) as u8;
            let mut payload = vec![
                RESOLVE_HUZUN_CHOICE,
                player.raw(),
                skill.raw(),
                u32::from(max),
                options.len() as u32,
            ];
            payload.extend(options.drain(..).map(PlayerId::raw));
            open_choice(
                tx,
                player,
                1,
                max,
                false,
                payload[5..].to_vec(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            let alive = tx.state().players.alive_by_seat();
            let current = tx.state().turn.current;
            let mut ordered = targets;
            ordered.sort_by_key(|target| {
                crate::trigger::seat_rel_to_current(&alive, current, *target)
            });
            let mut effects: Vec<_> = ordered
                .iter()
                .map(|target| EffectRequest::DrawCards {
                    player: *target,
                    count: 1,
                })
                .collect();
            effects.push(EffectRequest::ResolveHuzun {
                player,
                source: PlayerId::NONE,
                card: CardId::NONE,
                action: 4,
                cards: Vec::new(),
                targets: ordered,
                skill,
            });
            run_effects(tx, effects)
        }
        3 => {
            let Some((&judgment, materials)) = cards.split_first() else {
                return Err(EffectError::EngineFault("斩涛缺少判定牌".into()));
            };
            let judgment_rank = tx
                .state()
                .cards
                .get(judgment)
                .map(|card| card.rank)
                .unwrap_or(0);
            let threshold = crate::query::physical_card_number_sum(tx.state(), card, materials);
            if threshold.is_none_or(|threshold| u32::from(judgment_rank) <= threshold)
                || !tx
                    .state()
                    .players
                    .get(source)
                    .is_some_and(|target| target.alive)
            {
                return Ok(Vec::new());
            }
            let virtual_sha = allocate_virtual_sha(tx)?;
            if crate::command::play::validate_sha_target_with_options(
                tx,
                player,
                virtual_sha,
                source,
                false,
            )
            .is_err()
            {
                remove_unused_virtual_card(tx, virtual_sha)?;
                return Ok(Vec::new());
            }
            crate::command::play::handle_virtual_sha_with_policies_and_bonus(
                tx,
                player,
                virtual_sha,
                &[source],
                false,
                true,
                0,
            )?;
            Ok(Vec::new())
        }
        4 => {
            let min_hp = targets
                .iter()
                .filter_map(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .filter(|target| target.alive)
                        .map(|target| target.hp)
                })
                .min();
            let Some(min_hp) = min_hp else {
                return Ok(Vec::new());
            };
            let candidates: Vec<_> = targets
                .into_iter()
                .filter(|target| {
                    tx.state()
                        .players
                        .get(*target)
                        .is_some_and(|target| target.alive && target.hp == min_hp)
                })
                .collect();
            let mut rng = RuleRng::from_state(tx.state().rng);
            let target = candidates[rng.gen_range_u32(candidates.len() as u32) as usize];
            tx.journal(JournalEntry::RngCheckpoint);
            tx.state_mut().rng = rng.into_state();
            run_effects(
                tx,
                vec![EffectRequest::RecoverHp {
                    source: player,
                    player: target,
                    amount: 1,
                }],
            )
        }
        _ => Err(EffectError::ContentFault("bad Huzun action".into())),
    }
}

pub(crate) fn give_selected_then_target_may_use_draw_owner(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || player == target
                || ![player, target].into_iter().all(|participant| {
                    tx.state()
                        .players
                        .get(participant)
                        .is_some_and(|state| state.alive)
                })
            {
                return Err(EffectError::Illegal("invalid Yuanlve context".into()));
            }
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Yuanlve owner hand missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("Yuanlve target hand missing".into()))?;
            if !tx.state().cards.get(card).is_some_and(|state| {
                state.owner == player
                    && state.zone == from
                    && !crate::query::effective_card_kind(tx.state(), player, state).is_equip()
            }) {
                return Err(EffectError::Illegal(
                    "Yuanlve requires one non-equipment hand card".into(),
                ));
            }
            let events = give_cards_as(tx, player, vec![card], to, from)?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::GiveSelectedThenTargetMayUseDrawOwner {
                    player,
                    target,
                    card,
                    targets: Vec::new(),
                    stage: 1,
                    skill,
                },
            );
            Ok(events)
        }
        1 => {
            let hand = tx.state().zones.find(ZoneKind::Hand, target);
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|state| state.alive)
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.owner == target && Some(state.zone) == hand)
                || crate::command::play::skill_card_target_spec(tx, target, card).is_err()
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                true,
                vec![card.raw()],
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    YUANLVE_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            if tx
                .state()
                .players
                .get(player)
                .is_some_and(|state| state.alive)
            {
                tx.state_mut()
                    .resolution
                    .deferred_tail_effects
                    .push(EffectRequest::DrawCards { player, count: 1 });
            }
            crate::command::play::handle_skill_card(tx, target, card, &targets)?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault("bad Yuanlve stage".into())),
    }
}

pub(crate) fn choose_gain_target_hand_ceil_half_difference(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
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
        return Err(EffectError::Illegal("invalid Yangwu context".into()));
    }
    if !ready_to_choose {
        let hand = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("Yangwu owner hand missing".into()))?;
        if !tx.state().cards.get(card).is_some_and(|state| {
            state.owner == player
                && state.zone == hand
                && crate::query::effective_card_suit(tx.state(), player, state)
                    == engine_api::Suit::Heart
        }) {
            return Err(EffectError::Illegal(
                "Yangwu cost must be one heart hand card".into(),
            ));
        }
        return super::run_effects(
            tx,
            vec![
                EffectRequest::DiscardCards {
                    player,
                    cards: vec![card],
                },
                EffectRequest::ChooseGainTargetHandCeilHalfDifference {
                    player,
                    target,
                    card,
                    skill,
                    ready_to_choose: true,
                },
            ],
        );
    }
    let difference = tx
        .state()
        .hand_count(target)
        .saturating_sub(tx.state().hand_count(player));
    let count = difference.div_ceil(2);
    if count == 0 {
        return Ok(Vec::new());
    }
    let options = tx.state().hand_cards(target);
    if options.len() < count as usize {
        return Err(EffectError::Illegal("Yangwu target hand changed".into()));
    }
    open_choice(
        tx,
        player,
        u8::try_from(count).unwrap_or(u8::MAX),
        u8::try_from(count).unwrap_or(u8::MAX),
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            YANGWU_GAIN_HAND_CUSTOM_CHOICE,
            player.raw(),
            target.raw(),
            skill.raw(),
            count,
        ],
    )
}

pub(crate) fn resolve_refuhai(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let mut participants = tx.state().players.alive_by_seat();
    let Some(owner_index) = participants.iter().position(|target| *target == player) else {
        return Ok(Vec::new());
    };
    participants.rotate_left(owner_index + 1);
    participants.retain(|target| *target != player);
    let Some(observer) = participants.first().copied() else {
        return Ok(Vec::new());
    };
    let mut payload = vec![
        REFUHAI_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        0,
        participants.len() as u32,
    ];
    payload.extend(participants.iter().map(|target| target.raw()));
    open_choice(
        tx,
        observer,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn choose_other_random_gain_region_then_draw_compare_color_lose_hp(
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
    let options: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            *target != player
                && [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
                    .into_iter()
                    .any(|kind| {
                        tx.state()
                            .zones
                            .find(kind, *target)
                            .and_then(|zone| tx.state().zones.get(zone))
                            .is_some_and(|zone| !zone.cards.is_empty())
                    })
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
            RANDOM_GAIN_REGION_COMPARE_COLOR_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn reveal_choose_category_distribute_self_threshold_hp_loss(
    tx: &mut Transaction,
    player: PlayerId,
    reveal_count: u8,
    recipient_cap: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if reveal_count == 0 || recipient_cap == 0 || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("category distribution draw pile missing".into()))?;
    while tx
        .state()
        .zones
        .get(draw)
        .map_or(0, |zone| zone.cards.len())
        < usize::from(reveal_count)
    {
        let before = tx
            .state()
            .zones
            .get(draw)
            .map_or(0, |zone| zone.cards.len());
        reshuffle_discard_into_draw(tx)?;
        if tx
            .state()
            .zones
            .get(draw)
            .map_or(0, |zone| zone.cards.len())
            == before
        {
            break;
        }
    }
    let cards: Vec<_> = tx
        .state()
        .zones
        .get(draw)
        .map(|zone| {
            zone.cards
                .iter()
                .rev()
                .take(usize::from(reveal_count))
                .copied()
                .collect()
        })
        .unwrap_or_default();
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| {
            EffectError::Illegal("category distribution processing zone missing".into())
        })?;
    let mut categories: Vec<_> = cards
        .iter()
        .map(|card| effective_card_category(tx.state(), player, *card))
        .filter(|category| *category != 0)
        .collect();
    categories.sort_unstable();
    categories.dedup();
    run_effects(
        tx,
        vec![
            EffectRequest::MoveCards {
                cards: cards.clone(),
                to: processing,
                from: Some(draw),
            },
            EffectRequest::RevealCards {
                cards: cards.clone(),
                to: Vec::new(),
            },
        ],
    )?;
    if categories.is_empty() {
        let discard = tx
            .state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .ok_or_else(|| EffectError::Illegal("category distribution discard missing".into()))?;
        return run_effects(
            tx,
            vec![EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(processing),
            }],
        );
    }
    let mut payload = vec![
        REVEAL_CATEGORY_DISTRIBUTION_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        u32::from(recipient_cap),
        0,
        cards.len() as u32,
    ];
    payload.extend(cards.iter().map(|card| card.raw()));
    payload.extend([0, 0]); // no recipient counts, no pending cards
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        categories.into_iter().map(u32::from).collect(),
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn resolve_sibian(
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
        || tx.state().turn.phase != Phase::Draw
        || tx.state().turn.draw_phase_fixed_count().is_some()
    {
        return Ok(Vec::new());
    }
    let draw_zone = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no draw pile".into()))?;
    let pile: Vec<CardId> = tx
        .state()
        .zones
        .get(draw_zone)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    if pile.is_empty() {
        return Ok(Vec::new());
    }
    let take = pile.len().min(4);
    let cards: Vec<CardId> = pile[pile.len() - take..].to_vec();
    // Reveal the chosen pile cards.
    let mut effects = vec![EffectRequest::RevealCards {
        cards: cards.clone(),
        to: Vec::new(),
    }];
    let mut by_number: Vec<(u8, CardId)> = cards
        .iter()
        .map(|card| {
            let rank = tx
                .state()
                .cards
                .get(*card)
                .map(|card| card.rank)
                .unwrap_or(0);
            (rank, *card)
        })
        .collect();
    by_number.sort_by(|a, b| b.0.cmp(&a.0)); // descending, like the legacy sort
    let (max_rank, min_rank) = (
        by_number.first().map(|pair| pair.0).unwrap_or(0),
        by_number.last().map(|pair| pair.0).unwrap_or(0),
    );
    let mut gain = Vec::new();
    let mut middle = Vec::new();
    for (rank, card) in by_number {
        if rank == max_rank || rank == min_rank {
            gain.push(card);
        } else {
            middle.push(card);
        }
    }
    // Owner gains the extreme-numbered cards now.
    for card in &gain {
        effects.push(EffectRequest::MoveCards {
            cards: vec![*card],
            to: tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("owner hand missing".into()))?,
            from: Some(draw_zone),
        });
    }
    // Skip the ordinary draw.
    tx.state_mut().turn.set_draw_phase_fixed_count(0);
    if middle.is_empty() {
        return run_effects(tx, effects);
    }
    // A minimum-hand player receives the middle cards.
    let min_hand = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|p| tx.state().hand_count(p))
        .min()
        .unwrap_or(0);
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| tx.state().hand_count(candidate) == min_hand)
        .map(PlayerId::raw)
        .collect();
    let mut payload = vec![
        RESOLVE_SIBIAN_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        middle.len() as u32,
    ];
    payload.extend(middle.iter().map(|card| card.raw()));
    crate::effect::run_effects(tx, effects)?;
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn resolve_spjiedao(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(participant) = tx.state().players.get(player) else {
        return Ok(Vec::new());
    };
    if !participant.alive
        || participant.hp >= participant.max_hp as i32
        || tx.state().resolution.current_damage_id == 0
        || tx.state().turn.player_dealt_damage(player)
    {
        return Ok(Vec::new());
    }
    let num = (participant.max_hp as i32 - participant.hp).max(1) as u32;
    let options: Vec<_> = (1..=num.min(8)).collect();
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![RESOLVE_SPJIEDAO_CUSTOM_CHOICE, player.raw(), skill.raw(), 0],
    )
}


/// 放权·目标插相（ABI 671，shenhua fangquan2）：选择一名其他存活角色，将其
/// 排入 extra_turn_queue；check 非零时同时登记尾随失去体力 check 标记。
pub(crate) fn choose_others_insert_extra_turn(
    tx: &mut Transaction,
    player: PlayerId,
    check: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx.state().players.get(player).is_some_and(|p| p.alive)
    {
        return Ok(Vec::new());
    }
    if tx.state().turn.extra_turn_queue.len() >= 8 {
        return Err(EffectError::Illegal(
            "extra turn queue overflow (max 8)".into(),
        ));
    }
    let options: Vec<u32> = tx
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
        true,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOthersInsertExtraTurn,
        vec![player.raw(), check, skill.raw()],
    )
}

/// Cont 281 resolver（放权·目标插相）：把选中目标 push 入 extra_turn_queue。
pub(crate) fn resolve_choose_others_insert_extra_turn(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, check, _skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad insert-extra-turn payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    if observer != player {
        return Err(EffectError::Illegal("insert-extra-turn chooser mismatch".into()));
    }
    if cancelled {
        return Ok(());
    }
    let [target_raw] = selections else {
        return Err(EffectError::Illegal(
            "insert-extra-turn must pick one target".into(),
        ));
    };
    let target = PlayerId::new(*target_raw);
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|p| p.alive)
    {
        return Err(EffectError::Illegal("bad extra-turn target".into()));
    }
    if tx.state().turn.extra_turn_queue.len() >= 8 {
        return Ok(());
    }
    tx.state_mut().turn.extra_turn_queue.push(target);
    if *check != 0 {
        tx.state_mut().turn.set_flag(
            crate::state::turn_flag::extra_turn_check_pending_by_player(target),
            1,
        );
    }
    Ok(())
}
