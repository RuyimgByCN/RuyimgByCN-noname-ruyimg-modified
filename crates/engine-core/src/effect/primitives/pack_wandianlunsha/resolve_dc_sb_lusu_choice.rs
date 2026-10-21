//! Effect primitives for `resolve_dc_sb_lusu_choice` and related helpers.

use super::*;

pub(crate) fn resolve_dc_sb_lusu_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DC_SB_LUSU_CHOICE) || payload.len() < 4 {
        return Err(EffectError::EngineFault(
            "truncated DC SB Lusu choice".into(),
        ));
    }
    match payload[1] {
        0 => {
            let player = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            if observer != player || cancelled || selections.len() != 3 {
                return Err(EffectError::Illegal(
                    "invalid Mingshi hand selection".into(),
                ));
            }
            let hand = tx.state().zones.find(ZoneKind::Hand, player);
            let mut cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            cards.sort_by_key(|card| card.raw());
            cards.dedup();
            if cards.len() != 3
                || cards.iter().any(|card| {
                    !tx.state().cards.get(*card).is_some_and(|card_state| {
                        Some(card_state.zone) == hand && card_state.owner == player
                    })
                })
            {
                return Err(EffectError::Illegal("stale Mingshi hand selection".into()));
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
                .collect();
            let mut next = vec![RESOLVE_DC_SB_LUSU_CHOICE, 1, player.raw(), skill.raw()];
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
            let player = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            if observer != player || cancelled || selections.len() != 1 || payload.len() != 7 {
                return Err(EffectError::Illegal("invalid Mingshi target".into()));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("stale Mingshi target".into()));
            }
            let cards: Vec<_> = payload[4..].iter().copied().map(CardId::new).collect();
            run_effects(
                tx,
                vec![
                    EffectRequest::RevealCards {
                        cards: cards.clone(),
                        to: Vec::new(),
                    },
                    EffectRequest::ResolveDcSbLusu {
                        player,
                        source: target,
                        card: CardId::NONE,
                        action: 2,
                        cards,
                        amount: 0,
                        skill,
                    },
                ],
            )
            .map(|_| ())
        }
        2 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault("truncated Mingshi gain".into()));
            }
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let count = payload[5] as usize;
            if observer != target
                || cancelled
                || selections.len() != 1
                || payload.len() != 6 + count
                || !payload[6..].contains(&selections[0])
            {
                return Err(EffectError::Illegal("invalid Mingshi gain choice".into()));
            }
            let card = CardId::new(selections[0]);
            let from = tx
                .state()
                .zones
                .find(ZoneKind::Hand, player)
                .ok_or_else(|| EffectError::Illegal("Mingshi source hand missing".into()))?;
            let to = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::Illegal("Mingshi target hand missing".into()))?;
            if !tx
                .state()
                .cards
                .get(card)
                .is_some_and(|state| state.zone == from && state.owner == player)
            {
                return Err(EffectError::Illegal("Mingshi card moved".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::GiveCards {
                    cards: vec![card],
                    to,
                    from,
                }],
            )
            .map(|_| ())
        }
        10 => {
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            if observer != target {
                return Err(EffectError::Illegal("Mengmou Sha chooser mismatch".into()));
            }
            if cancelled || selections.is_empty() {
                set_dcsbmengmou_sequence(tx, skill, None)?;
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("Mengmou needs one Sha".into()));
            }
            let card = CardId::new(selections[0]);
            let legal = crate::command::play::prompted_sha_targets_normal(tx, target, card);
            if legal.is_empty()
                || !crate::command::play::prompted_sha_source_cards(tx.state(), target)
                    .contains(&card)
            {
                return Err(EffectError::Illegal("Mengmou Sha became illegal".into()));
            }
            if crate::query::sha_targets_all_for(tx.state(), target) {
                let Some(mut sequence) =
                    tx.state().skills.get(skill).and_then(dcsbmengmou_sequence)
                else {
                    return Ok(());
                };
                sequence.card = card;
                sequence.remaining = sequence.remaining.saturating_sub(1);
                set_dcsbmengmou_sequence(tx, skill, Some(sequence))?;
                return crate::command::play::play_prompted_sha_source_normal(
                    tx, target, card, &legal,
                );
            }
            let max = crate::query::sha_target_limit_for(tx.state(), target)
                .min(legal.len() as u32)
                .min(u32::from(u8::MAX)) as u8;
            open_choice(
                tx,
                target,
                1,
                max,
                true,
                legal.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DC_SB_LUSU_CHOICE,
                    11,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    card.raw(),
                ],
            )
            .map(|_| ())
        }
        11 => {
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let card = CardId::new(payload[5]);
            if observer != target || cancelled || selections.is_empty() {
                set_dcsbmengmou_sequence(tx, skill, None)?;
                return Ok(());
            }
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            let Some(mut sequence) = tx.state().skills.get(skill).and_then(dcsbmengmou_sequence)
            else {
                return Ok(());
            };
            sequence.card = card;
            sequence.remaining = sequence.remaining.saturating_sub(1);
            set_dcsbmengmou_sequence(tx, skill, Some(sequence))?;
            crate::command::play::play_prompted_sha_source_normal(tx, target, card, &targets)
        }
        12 => {
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            if observer != target {
                return Err(EffectError::Illegal(
                    "Mengmou response chooser mismatch".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                return finish_dcsbmengmou_yin(tx, player, skill).map(|_| ());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal("Mengmou needs one response".into()));
            }
            let token = selections[0];
            if let Some(response_skill) = crate::command::skill::source_free_response_skill(
                tx.state(),
                target,
                CardKind::Sha,
                token,
            ) {
                return crate::command::skill::begin_source_free_response(
                    tx,
                    target,
                    response_skill,
                    CardKind::Sha,
                    7,
                    &[player.raw(), skill.raw()],
                );
            }
            if let Some(response_skill) = crate::command::skill::multi_card_response_skill(
                tx.state(),
                target,
                CardKind::Sha,
                token,
            ) {
                return crate::command::skill::begin_multi_card_response(
                    tx,
                    target,
                    response_skill,
                    CardKind::Sha,
                    7,
                    &[player.raw(), skill.raw()],
                );
            }
            if !crate::command::play::response_sha_tokens(tx.state(), target).contains(&token) {
                return Err(EffectError::Illegal(
                    "Mengmou response became illegal".into(),
                ));
            }
            let card = CardId::new(token);
            let from = crate::command::resolve_choice::response_card_zone(tx, target, card)?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("discard pile missing".into()))?;
            crate::command::resolve_choice::run_physical_response_effects(
                tx,
                vec![
                    EffectRequest::MoveCards {
                        cards: vec![card],
                        to: discard,
                        from: Some(from),
                    },
                    EffectRequest::CardResponded {
                        player: target,
                        respond_to: player,
                        card,
                        response_kind: CardKind::Sha.tag(),
                    },
                    EffectRequest::ResolveDcSbLusu {
                        player,
                        source: target,
                        card,
                        action: 22,
                        cards: Vec::new(),
                        amount: 0,
                        skill,
                    },
                ],
                ContinuationKind::None,
                Vec::new(),
                target,
                card,
                CardKind::Sha,
            )
        }
        _ => Err(EffectError::EngineFault("unknown DC SB Lusu choice".into())),
    }
}

pub(crate) fn set_current_damage_amount(
    tx: &mut Transaction,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id == 0 {
        return Err(EffectError::ContentFault(
            "Zhangjian damage rewrite used outside damage_before".into(),
        ));
    }
    let request = tx
        .state_mut()
        .resolution
        .deferred_tail_effects
        .iter_mut()
        .find(|request| {
            matches!(
                request,
                EffectRequest::CommitDamage { damage_id: saved, .. }
                    | EffectRequest::CommitDamageBanActiveIfDying {
                        damage_id: saved,
                        ..
                    } if *saved == damage_id
            )
        })
        .ok_or_else(|| EffectError::EngineFault("bound damage commit is missing".into()))?;
    match request {
        EffectRequest::CommitDamage {
            amount: current, ..
        }
        | EffectRequest::CommitDamageBanActiveIfDying {
            amount: current, ..
        } => *current = amount,
        _ => unreachable!(),
    }
    Ok(Vec::new())
}

pub(crate) fn resolve_zhangjian(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    grant_skill_def: engine_api::SkillDefId,
    action: u8,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if matches!(action, 0 | 1) && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let rank = tx
                .state()
                .cards
                .get(card)
                .map_or(0, |card_state| u32::from(card_state.rank));
            if rank == 0 {
                run_effects(
                    tx,
                    vec![EffectRequest::PreventDamage {
                        damage_id: tx.state().resolution.current_damage_id,
                    }],
                )
            } else {
                set_current_damage_amount(tx, rank)
            }
        }
        1 => {
            if grant_skill_def.is_none()
                || crate::query::discardable_hand_equip_cards(tx.state(), player).is_empty()
            {
                return Ok(Vec::new());
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|candidate| *candidate != player)
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
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_ZHANGJIAN_CHOICE,
                    player.raw(),
                    grant_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            if grant_skill_def.is_none()
                || target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            add_source_turn_bound_skill(tx, target, grant_skill_def, player)
        }
        _ => Err(EffectError::ContentFault(
            "unknown Zhangjian resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_zhangjian_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || payload.first() != Some(&RESOLVE_ZHANGJIAN_CHOICE) {
        return Err(EffectError::EngineFault(
            "truncated Zhangjian target choice".into(),
        ));
    }
    let player = PlayerId::new(payload[1]);
    let grant_skill_def = engine_api::SkillDefId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "invalid Zhangjian target choice".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    if target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal("stale Zhangjian target choice".into()));
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::ResolveZhangjian {
            player,
            card: CardId::NONE,
            grant_skill_def,
            action: 2,
            target,
            skill,
        });
    discard_all_hand_and_equip(tx, player).map(|_| ())
}

pub(crate) fn malingli_targets_after_owner(tx: &Transaction, owner: PlayerId) -> Vec<PlayerId> {
    let mut players = tx.state().players.alive_by_seat();
    let Some(index) = players.iter().position(|player| *player == owner) else {
        return Vec::new();
    };
    players.rotate_left(index + 1);
    players.retain(|player| *player != owner);
    players
}

pub(crate) fn malingli_targets_are_contiguous(
    tx: &Transaction,
    owner: PlayerId,
    targets: &[PlayerId],
) -> bool {
    let ordered = malingli_targets_after_owner(tx, owner);
    let selected: std::collections::BTreeSet<_> = targets.iter().copied().collect();
    if selected.is_empty()
        || selected.len() != targets.len()
        || !selected.iter().all(|target| ordered.contains(target))
    {
        return false;
    }
    let positions: Vec<_> = ordered
        .iter()
        .enumerate()
        .filter_map(|(index, target)| selected.contains(target).then_some(index))
        .collect();
    positions
        .windows(2)
        .all(|pair| pair[1] == pair[0].saturating_add(1))
}

pub(crate) fn ensure_malingli_helper(
    tx: &mut Transaction,
    target: PlayerId,
    helper_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if tx.state().players.get(target).is_none() || helper_def.is_none() {
        return Err(EffectError::Illegal("invalid Xiaoyin helper target".into()));
    }
    if tx.state().players.get(target).is_some_and(|player| {
        player.skills.iter().any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def == helper_def)
        })
    }) {
        return Ok(Vec::new());
    }
    grant_skill_instance(tx, target, helper_def, Vec::new()).map(|(_, events)| events)
}

pub(crate) fn remove_empty_malingli_helper(
    tx: &mut Transaction,
    target: PlayerId,
    helper_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .skill_pile_cards(target, helper_def.raw())
        .is_empty()
    {
        return Ok(Vec::new());
    }
    let helper = tx.state().players.get(target).and_then(|player| {
        player.skills.iter().copied().find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def == helper_def)
        })
    });
    helper.map_or(Ok(Vec::new()), |skill| remove_skill(tx, skill))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_malingli(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    action: u8,
    source: PlayerId,
    target: PlayerId,
    nature: u8,
    damage_id: u32,
    event_cards: Vec<CardId>,
    event_targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || tx.state().turn.current != player
                || tx.state().turn.phase != Phase::Prepare
            {
                return Ok(Vec::new());
            }
            let count = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    crate::query::player_within_distance(tx.state(), player, *target, 1)
                })
                .count();
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            let mut revealed = Vec::new();
            let mut events = Vec::new();
            for _ in 0..count {
                if tx
                    .state()
                    .zones
                    .get(draw)
                    .is_none_or(|zone| zone.cards.is_empty())
                {
                    reshuffle_discard_into_draw(tx)?;
                }
                let Some(next) = tx
                    .state()
                    .zones
                    .get(draw)
                    .and_then(|zone| zone.cards.last().copied())
                else {
                    break;
                };
                events.extend(move_cards(tx, vec![next], processing, Some(draw))?);
                revealed.push(next);
            }
            if revealed.is_empty() {
                return Ok(events);
            }
            events.push(DomainEvent::CardsRevealed {
                cards: revealed.clone(),
                to: Vec::new(),
            });
            let (red, black): (Vec<_>, Vec<_>) = revealed.into_iter().partition(|card| {
                tx.state()
                    .cards
                    .get(*card)
                    .is_some_and(|card| crate::query::card_color_mask_for_suit(card.suit) == 1)
            });
            if !red.is_empty() {
                let hand = tx
                    .state()
                    .zones
                    .find(ZoneKind::Hand, player)
                    .ok_or_else(|| EffectError::EngineFault("owner hand missing".into()))?;
                events.extend(move_cards(tx, red, hand, Some(processing))?);
            }
            if black.is_empty() {
                return Ok(events);
            }
            let targets = malingli_targets_after_owner(tx, player);
            if targets.is_empty() {
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                events.extend(move_cards(tx, black, discard, Some(processing))?);
                return Ok(events);
            }
            let mut payload = vec![
                RESOLVE_MALINGLI_CHOICE,
                0,
                player.raw(),
                helper_skill_def.raw(),
                skill.raw(),
                black.len() as u32,
            ];
            payload.extend(black.iter().map(|card| card.raw()));
            events.extend(open_choice(
                tx,
                player,
                1,
                black.len().min(targets.len()).min(u8::MAX as usize) as u8,
                false,
                targets.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )?);
            Ok(events)
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !malingli_targets_are_contiguous(tx, player, &event_targets)
                || event_targets.len() > event_cards.len()
            {
                return Err(EffectError::Illegal(
                    "stale Xiaoyin target selection".into(),
                ));
            }
            if event_targets.len() == event_cards.len() {
                return resolve_malingli(
                    tx,
                    player,
                    CardId::NONE,
                    helper_skill_def,
                    2,
                    PlayerId::NONE,
                    PlayerId::NONE,
                    0,
                    0,
                    event_cards,
                    event_targets,
                    skill,
                );
            }
            let mut payload = vec![
                RESOLVE_MALINGLI_CHOICE,
                1,
                player.raw(),
                helper_skill_def.raw(),
                skill.raw(),
                event_targets.len() as u32,
            ];
            payload.extend(event_targets.iter().map(|target| target.raw()));
            payload.push(event_cards.len() as u32);
            payload.extend(event_cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                event_targets.len() as u8,
                event_targets.len() as u8,
                false,
                event_cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                payload,
            )
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || !malingli_targets_are_contiguous(tx, player, &event_targets)
                || event_cards.len() != event_targets.len()
            {
                return Err(EffectError::Illegal("stale Xiaoyin placement".into()));
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            if !event_cards.iter().all(|card| {
                tx.state().cards.get(*card).is_some_and(|state| {
                    state.zone == processing
                        && crate::query::card_color_mask_for_suit(state.suit) == 2
                })
            }) {
                return Err(EffectError::Illegal("Xiaoyin black cards moved".into()));
            }
            let ordered = malingli_targets_after_owner(tx, player);
            let selected: std::collections::BTreeSet<_> = event_targets.into_iter().collect();
            let targets: Vec<_> = ordered
                .into_iter()
                .filter(|target| selected.contains(target))
                .collect();
            let mut events = Vec::new();
            for (target, card) in targets.into_iter().zip(event_cards.iter().copied()) {
                events.extend(ensure_malingli_helper(tx, target, helper_skill_def)?);
                let pile = ensure_skill_pile_zone(tx, target, helper_skill_def.raw())?;
                events.extend(move_cards(tx, vec![card], pile, Some(processing))?);
            }
            Ok(events)
        }
        10 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || player != target
                || source.is_none()
            {
                return Ok(Vec::new());
            }
            let damage_id = if damage_id == 0 {
                tx.state().resolution.current_damage_id
            } else {
                damage_id
            };
            let pile = tx.state().skill_pile_cards(player, helper_skill_def.raw());
            if pile.is_empty() || damage_id == 0 {
                return Ok(Vec::new());
            }
            if nature == rule_api::damage_nature::FIRE {
                let categories: std::collections::BTreeSet<_> = pile
                    .iter()
                    .filter_map(|card| card_category_of(tx.state(), *card))
                    .collect();
                let options: Vec<_> =
                    crate::query::discardable_hand_equip_cards(tx.state(), source)
                        .into_iter()
                        .filter(|card| {
                            card_category_of(tx.state(), *card)
                                .is_some_and(|category| categories.contains(&category))
                        })
                        .collect();
                if options.is_empty() {
                    return Ok(Vec::new());
                }
                open_choice(
                    tx,
                    source,
                    0,
                    1,
                    true,
                    options.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_MALINGLI_CHOICE,
                        10,
                        player.raw(),
                        source.raw(),
                        helper_skill_def.raw(),
                        skill.raw(),
                        damage_id,
                    ],
                )
            } else {
                open_choice(
                    tx,
                    source,
                    0,
                    1,
                    true,
                    pile.into_iter().map(CardId::raw).collect(),
                    ChoiceKind::SelectCards,
                    ContinuationKind::Custom,
                    vec![
                        RESOLVE_MALINGLI_CHOICE,
                        12,
                        player.raw(),
                        source.raw(),
                        helper_skill_def.raw(),
                        skill.raw(),
                        damage_id,
                    ],
                )
            }
        }
        11 => {
            let category = card_category_of(tx.state(), card)
                .ok_or_else(|| EffectError::Illegal("Xiaoyin cost card missing".into()))?;
            let matching: Vec<_> = tx
                .state()
                .skill_pile_cards(player, helper_skill_def.raw())
                .into_iter()
                .filter(|candidate| card_category_of(tx.state(), *candidate) == Some(category))
                .collect();
            if matching.is_empty() {
                return Err(EffectError::Illegal(
                    "matching Xiaoyin card disappeared".into(),
                ));
            }
            if matching.len() == 1 {
                return resolve_malingli(
                    tx,
                    player,
                    matching[0],
                    helper_skill_def,
                    12,
                    source,
                    player,
                    rule_api::damage_nature::FIRE,
                    damage_id,
                    vec![card],
                    Vec::new(),
                    skill,
                );
            }
            open_choice(
                tx,
                source,
                1,
                1,
                false,
                matching.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_MALINGLI_CHOICE,
                    11,
                    player.raw(),
                    source.raw(),
                    helper_skill_def.raw(),
                    skill.raw(),
                    damage_id,
                    card.raw(),
                ],
            )
        }
        12 => {
            let cost = event_cards.first().copied().unwrap_or(CardId::NONE);
            if card_category_of(tx.state(), card) != card_category_of(tx.state(), cost) {
                return Err(EffectError::Illegal("Xiaoyin card category changed".into()));
            }
            let pile = ensure_skill_pile_zone(tx, player, helper_skill_def.raw())?;
            if !tx
                .state()
                .zones
                .get(pile)
                .is_some_and(|zone| zone.cards.contains(&card))
            {
                return Err(EffectError::Illegal("Xiaoyin card moved".into()));
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let mut events = move_cards_with_reason(
                tx,
                vec![card],
                discard,
                Some(pile),
                CardMoveReason::Discard,
            )?;
            increase_bound_damage(tx, damage_id, 1)?;
            events.extend(remove_empty_malingli_helper(tx, player, helper_skill_def)?);
            Ok(events)
        }
        13 => {
            let pile = ensure_skill_pile_zone(tx, player, helper_skill_def.raw())?;
            if !tx
                .state()
                .zones
                .get(pile)
                .is_some_and(|zone| zone.cards.contains(&card))
            {
                return Err(EffectError::Illegal("Xiaoyin card moved".into()));
            }
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, source)
                .ok_or_else(|| EffectError::EngineFault("damage source hand missing".into()))?;
            let mut events =
                move_cards_with_reason(tx, vec![card], hand, Some(pile), CardMoveReason::Give)?;
            set_bound_damage_nature(tx, damage_id, rule_api::damage_nature::FIRE)?;
            events.extend(remove_empty_malingli_helper(tx, player, helper_skill_def)?);
            Ok(events)
        }
        14 => resolve_malingli(
            tx,
            player,
            card,
            helper_skill_def,
            11,
            source,
            player,
            rule_api::damage_nature::FIRE,
            damage_id,
            Vec::new(),
            Vec::new(),
            skill,
        ),
        20 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| {
                    !tx.state()
                        .skill_pile_cards(*target, helper_skill_def.raw())
                        .is_empty()
                })
                .filter(|target| {
                    crate::command::play::validate_sha_target_with_options(
                        tx, player, card, *target, true,
                    )
                    .is_ok()
                })
                .collect();
            if targets.is_empty() {
                return Ok(Vec::new());
            }
            crate::command::play::replace_saved_sha_targets_ignoring_distance(
                tx, player, card, &targets,
            )?;
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown Malingli resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_malingli_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 2 || payload.first() != Some(&RESOLVE_MALINGLI_CHOICE) {
        return Err(EffectError::EngineFault("truncated Malingli choice".into()));
    }
    match payload[1] {
        0 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault("truncated Xiaoyin targets".into()));
            }
            let player = PlayerId::new(payload[2]);
            let helper = engine_api::SkillDefId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let count = payload[5] as usize;
            if observer != player
                || cancelled
                || selections.is_empty()
                || payload.len() != 6 + count
            {
                return Err(EffectError::Illegal("invalid Xiaoyin targets".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveMalingli {
                    player,
                    card: CardId::NONE,
                    helper_skill_def: helper,
                    action: 1,
                    source: PlayerId::NONE,
                    target: PlayerId::NONE,
                    nature: 0,
                    damage_id: 0,
                    event_cards: payload[6..].iter().copied().map(CardId::new).collect(),
                    event_targets: selections.iter().copied().map(PlayerId::new).collect(),
                    skill,
                }],
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 6 {
                return Err(EffectError::EngineFault("truncated Xiaoyin cards".into()));
            }
            let player = PlayerId::new(payload[2]);
            let helper = engine_api::SkillDefId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let target_count = payload[5] as usize;
            let black_count_index = 6 + target_count;
            let black_count = payload.get(black_count_index).copied().unwrap_or(0) as usize;
            if observer != player
                || cancelled
                || selections.len() != target_count
                || payload.len() != black_count_index + 1 + black_count
            {
                return Err(EffectError::Illegal("invalid Xiaoyin cards".into()));
            }
            let allowed: std::collections::BTreeSet<_> = payload[black_count_index + 1..]
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            let selected: std::collections::BTreeSet<_> =
                selections.iter().copied().map(CardId::new).collect();
            if selected.len() != selections.len() || !selected.is_subset(&allowed) {
                return Err(EffectError::Illegal(
                    "invalid Xiaoyin black-card subset".into(),
                ));
            }
            let leftovers: Vec<_> = allowed.difference(&selected).copied().collect();
            let mut effects = Vec::new();
            if !leftovers.is_empty() {
                let processing = tx
                    .state()
                    .zones
                    .find(ZoneKind::Processing, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
                let discard = tx
                    .state()
                    .zones
                    .find(ZoneKind::DiscardPile, PlayerId::NONE)
                    .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
                effects.push(EffectRequest::MoveCards {
                    cards: leftovers,
                    to: discard,
                    from: Some(processing),
                });
            }
            effects.push(EffectRequest::ResolveMalingli {
                player,
                card: CardId::NONE,
                helper_skill_def: helper,
                action: 2,
                source: PlayerId::NONE,
                target: PlayerId::NONE,
                nature: 0,
                damage_id: 0,
                event_cards: selections.iter().copied().map(CardId::new).collect(),
                event_targets: payload[6..6 + target_count]
                    .iter()
                    .copied()
                    .map(PlayerId::new)
                    .collect(),
                skill,
            });
            run_effects(tx, effects).map(|_| ())
        }
        10 => {
            if payload.len() != 7 {
                return Err(EffectError::EngineFault(
                    "truncated Xiaoyin fire cost".into(),
                ));
            }
            let target = PlayerId::new(payload[2]);
            let source = PlayerId::new(payload[3]);
            if observer != source || selections.len() > 1 || (cancelled && !selections.is_empty()) {
                return Err(EffectError::Illegal("invalid Xiaoyin cost".into()));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let cost = CardId::new(selections[0]);
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: source,
                        cards: vec![cost],
                    },
                    EffectRequest::ResolveMalingli {
                        player: target,
                        card: cost,
                        helper_skill_def: engine_api::SkillDefId::new(payload[4]),
                        action: 14,
                        source,
                        target,
                        nature: rule_api::damage_nature::FIRE,
                        damage_id: payload[6],
                        event_cards: Vec::new(),
                        event_targets: Vec::new(),
                        skill: engine_api::SkillId::new(payload[5]),
                    },
                ],
            )
            .map(|_| ())
        }
        11 => {
            if payload.len() != 8 || cancelled || selections.len() != 1 {
                return Err(EffectError::Illegal("invalid Xiaoyin pile choice".into()));
            }
            let target = PlayerId::new(payload[2]);
            let source = PlayerId::new(payload[3]);
            if observer != source {
                return Err(EffectError::Illegal("Xiaoyin chooser changed".into()));
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveMalingli {
                    player: target,
                    card: CardId::new(selections[0]),
                    helper_skill_def: engine_api::SkillDefId::new(payload[4]),
                    action: 12,
                    source,
                    target,
                    nature: rule_api::damage_nature::FIRE,
                    damage_id: payload[6],
                    event_cards: vec![CardId::new(payload[7])],
                    event_targets: Vec::new(),
                    skill: engine_api::SkillId::new(payload[5]),
                }],
            )
            .map(|_| ())
        }
        12 => {
            if payload.len() != 7 || selections.len() > 1 {
                return Err(EffectError::Illegal("invalid Xiaoyin gain choice".into()));
            }
            let target = PlayerId::new(payload[2]);
            let source = PlayerId::new(payload[3]);
            if observer != source || selections.is_empty() {
                return Ok(());
            }
            run_effects(
                tx,
                vec![EffectRequest::ResolveMalingli {
                    player: target,
                    card: CardId::new(selections[0]),
                    helper_skill_def: engine_api::SkillDefId::new(payload[4]),
                    action: 13,
                    source,
                    target,
                    nature: 0,
                    damage_id: payload[6],
                    event_cards: Vec::new(),
                    event_targets: Vec::new(),
                    skill: engine_api::SkillId::new(payload[5]),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown Malingli choice stage".into(),
        )),
    }
}

pub(crate) fn resolve_nsshuaiyan(
    tx: &mut Transaction,
    player: PlayerId,
    recovered_player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    // 率言只在他人回合结算。
    if tx.state().turn.current == player {
        return Ok(Vec::new());
    }
    GuardedChoice::new(
        skill,
        Participants::distinct_pair(player, recovered_player),
        ContinuationKind::Custom,
        vec![
            RESOLVE_NSSHUAIYAN_CHOICE,
            player.raw(),
            recovered_player.raw(),
        ],
    )
    .chosen_by(recovered_player)
    .open_tokens(tx, vec![0, 1], ChoiceKind::Custom)
}

pub(crate) fn resolve_nsshuaiyan_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3
        || payload.first() != Some(&RESOLVE_NSSHUAIYAN_CHOICE)
        || observer != PlayerId::new(payload[2])
        || cancelled
        || selections.len() != 1
        || selections[0] > 1
    {
        return Err(EffectError::Illegal("invalid Shuaiyan choice".into()));
    }
    let player = PlayerId::new(payload[1]);
    let recovered_player = PlayerId::new(payload[2]);
    let effect = if selections[0] == 0 {
        EffectRequest::DrawCards { player, count: 1 }
    } else {
        EffectRequest::ChooseDiscardTargetCard {
            player,
            target: recovered_player,
            scope: rule_api::TargetCardScope::HandEquip,
        }
    };
    run_effects(tx, vec![effect]).map(|_| ())
}

pub(crate) fn resolve_wdb_hanlong(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    helper_skill_def: engine_api::SkillDefId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            if tx.state().turn.current != player || tx.state().turn.phase != Phase::End {
                return Ok(Vec::new());
            }
            let count = discard_phase_discarded_cards_for(tx.state(), player)
                .filter(|card| {
                    tx.state().cards.get(*card).is_some_and(|card_state| {
                        crate::query::effective_card_kind(tx.state(), player, card_state).is_sha()
                    })
                })
                .count() as u32;
            if count == 0 {
                return Ok(Vec::new());
            }
            run_effects(
                tx,
                vec![EffectRequest::DrawCards {
                    player,
                    count: count.saturating_mul(2),
                }],
            )
        }
        1 => {
            let damage_id = tx.state().resolution.current_damage_id;
            let valid_damage = damage_id != 0
                && tx
                    .state()
                    .resolution
                    .deferred_tail_effects
                    .iter()
                    .any(|request| {
                        matches!(
                            request,
                            EffectRequest::CommitDamage {
                                damage_id: saved,
                                source,
                                target: saved_target,
                                is_lose_hp: false,
                                card: saved_card,
                                card_kind,
                                ..
                            } if *saved == damage_id
                                && *source == player
                                && *saved_target == target
                                && *saved_card == card
                                && CardKind::from_tag(*card_kind).is_some_and(|kind| kind.is_sha())
                        )
                    });
            let healthy = tx.state().players.get(target).is_some_and(|target| {
                target.alive && target.hp > 0 && target.hp == target.max_hp as i32
            });
            if !valid_damage || !healthy {
                return Ok(Vec::new());
            }
            let (_, increased) = increase_bound_damage_by_one(tx, damage_id)?;
            let hp = tx.state().players.get(target).unwrap().hp.max(0) as u32;
            if increased < hp {
                return Ok(Vec::new());
            }
            let mut events = add_skill_until_turn_end(tx, target, helper_skill_def)?;
            events.extend(remove_skill(tx, skill)?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "unknown Wdb Hanlong resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_wdb_liufu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    zhucheng_skill_def: engine_api::SkillDefId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if action != 3 && !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    if zhucheng_skill_def.is_none() {
        return Err(EffectError::ContentFault(
            "Wdb Liufu requires a 筑城 definition".into(),
        ));
    }
    match action {
        0 => {
            if tx.state().turn.current != player
                || tx.state().turn.phase != Phase::End
                || skill_pile_def(tx, player, skill)? != zhucheng_skill_def.raw()
                || !tx
                    .state()
                    .skill_pile_cards(player, zhucheng_skill_def.raw())
                    .is_empty()
            {
                return Ok(Vec::new());
            }
            let owner = tx
                .state()
                .players
                .get(player)
                .ok_or_else(|| EffectError::Illegal("筑城 owner is missing".into()))?;
            let count = (owner.max_hp as i32 - owner.hp).max(1) as usize;
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::Illegal("draw pile is missing".into()))?;
            let pile = ensure_skill_pile_zone(tx, player, zhucheng_skill_def.raw())?;
            let mut cards = Vec::new();
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
                move_one_card(tx, card, draw, pile, player, true)?;
                cards.push(card);
            }
            if cards.is_empty() {
                Ok(Vec::new())
            } else {
                Ok(vec![DomainEvent::CardsMoved {
                    cards,
                    from: draw,
                    to: pile,
                    reason: CardMoveReason::Unspecified,
                }])
            }
        }
        1 => {
            if crate::command::play::validate_saved_sha_target(tx, target, card, player).is_err()
                || tx
                    .state()
                    .skill_pile_cards(player, zhucheng_skill_def.raw())
                    .is_empty()
            {
                return Ok(Vec::new());
            }
            let required = tx
                .state()
                .skill_pile_cards(player, zhucheng_skill_def.raw())
                .len();
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            if options.len() < required {
                return exclude_saved_sha_target(tx, card, player);
            }
            open_choice(
                tx,
                target,
                required.min(u8::MAX as usize) as u8,
                required.min(u8::MAX as usize) as u8,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_WDB_LIUFU_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    card.raw(),
                    zhucheng_skill_def.raw(),
                    skill.raw(),
                    required as u32,
                ],
            )
        }
        2 => {
            let current = tx.state().turn.current;
            let pile = tx
                .state()
                .skill_pile_cards(player, zhucheng_skill_def.raw());
            if current == player || tx.state().turn.phase != Phase::Play || pile.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                pile.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_WDB_LIUFU_CHOICE,
                    2,
                    player.raw(),
                    current.raw(),
                    0,
                    zhucheng_skill_def.raw(),
                    skill.raw(),
                    1,
                ],
            )
        }
        3 => {
            if tx.state().turn.current == target && tx.state().turn.phase == Phase::Play {
                crate::command::turn::finish_current_phase_abruptly(tx)?;
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown Wdb Liufu resolver action".into(),
        )),
    }
}
