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
