//! Effect primitives for `current_wugu_revealed` and related helpers.

use super::*;

pub(crate) fn save_ddd_xiaheng_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    deleted: bool,
    targets: &[PlayerId],
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob.clear();
        instance
            .state_blob
            .extend_from_slice(&DDD_XIAHENG_STATE_MAGIC.to_le_bytes());
        instance.state_blob.push(u8::from(deleted));
        instance
            .state_blob
            .extend_from_slice(&(targets.len().min(u32::MAX as usize) as u32).to_le_bytes());
        for target in targets {
            instance
                .state_blob
                .extend_from_slice(&target.raw().to_le_bytes());
        }
    }
}

pub(crate) fn resolve_ddd_xiaheng(
    tx: &mut Transaction,
    player: PlayerId,
    discard_target: PlayerId,
    damage_target: PlayerId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match action {
        0 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
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
                vec![RESOLVE_DDD_XIAHENG_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
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
                    RESOLVE_DDD_XIAHENG_CHOICE,
                    2,
                    player.raw(),
                    discard_target.raw(),
                    skill.raw(),
                ],
            )
        }
        2 => {
            let (deleted, mut targets) = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| ddd_xiaheng_state(&instance.state_blob))
                .unwrap_or_default();
            if !targets.contains(&damage_target) {
                targets.push(damage_target);
            }
            let remove = !deleted && discard_target == damage_target;
            save_ddd_xiaheng_state(tx, skill, deleted || targets.len() >= 3, &targets);
            if deleted {
                return Ok(Vec::new());
            }
            let mut effects = Vec::new();
            if discard_target != player && damage_target != player {
                effects.push(EffectRequest::ChangeMaxHp { player, delta: -1 });
            }
            if remove {
                effects.push(EffectRequest::RemoveSkill { skill });
            }
            run_effects(tx, effects)
        }
        _ => Err(EffectError::ContentFault(
            "unknown 侠横 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_xiaheng_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_XIAHENG_CHOICE) || cancelled {
        return Err(EffectError::Illegal("stale 侠横 choice".into()));
    }
    match payload.get(1).copied() {
        Some(0) => {
            if payload.len() != 4 {
                return Err(EffectError::Illegal(
                    "bad 侠横 discard target choice".into(),
                ));
            }
            let player = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let target = PlayerId::new(selections[0]);
            if selections.len() != 1
                || observer != player
                || !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid 侠横 discard target".into()));
            }
            let cards = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let count = cards.len().min(2);
            if count == 0 {
                return resolve_ddd_xiaheng(tx, player, target, PlayerId::NONE, 1, skill)
                    .map(|_| ());
            }
            open_choice(
                tx,
                target,
                count as u8,
                count as u8,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    RESOLVE_DDD_XIAHENG_CHOICE,
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    count as u32,
                ],
            )
            .map(|_| ())
        }
        Some(1) => {
            if payload.len() != 6 {
                return Err(EffectError::Illegal("bad 侠横 discard choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let count = payload[5] as usize;
            let cards = crate::query::discardable_hand_equip_cards(tx.state(), target);
            let unique: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if observer != target
                || !originating_skill_is_enabled(tx, player, skill)
                || selections.len() != count
                || count != cards.len().min(2)
                || unique.len() != selections.len()
                || selections
                    .iter()
                    .any(|raw| !cards.contains(&CardId::new(*raw)))
            {
                return Err(EffectError::Illegal("invalid 侠横 discard".into()));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddXiaheng {
                    player,
                    discard_target: target,
                    damage_target: PlayerId::NONE,
                    action: 1,
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DiscardCards {
                    player: target,
                    cards: selections.iter().copied().map(CardId::new).collect(),
                }],
            )
            .map(|_| ())
        }
        Some(2) => {
            if payload.len() != 5 {
                return Err(EffectError::Illegal("bad 侠横 damage target choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let discard_target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let target = PlayerId::new(selections[0]);
            if selections.len() != 1
                || observer != player
                || !originating_skill_is_enabled(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid 侠横 damage target".into()));
            }
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddXiaheng {
                    player,
                    discard_target,
                    damage_target: target,
                    action: 2,
                    skill,
                },
            );
            run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: 0,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal("unknown 侠横 choice stage".into())),
    }
}

pub(crate) fn ddd_moyan_state(blob: &[u8]) -> (u32, PlayerId, u32) {
    if blob.len() != 16
        || u32::from_le_bytes(blob[0..4].try_into().unwrap()) != DDD_MOYAN_STATE_MAGIC
    {
        return (u32::MAX, PlayerId::NONE, 0);
    }
    (
        u32::from_le_bytes(blob[4..8].try_into().unwrap()),
        PlayerId::new(u32::from_le_bytes(blob[8..12].try_into().unwrap())),
        u32::from_le_bytes(blob[12..16].try_into().unwrap()),
    )
}

pub(crate) fn save_ddd_moyan_state(
    tx: &mut Transaction,
    skill: engine_api::SkillId,
    round: u32,
    target: PlayerId,
    count: u32,
) {
    if let Some(instance) = tx.state_mut().skills.get_mut(skill) {
        instance.state_blob.clear();
        instance
            .state_blob
            .extend_from_slice(&DDD_MOYAN_STATE_MAGIC.to_le_bytes());
        instance.state_blob.extend_from_slice(&round.to_le_bytes());
        instance
            .state_blob
            .extend_from_slice(&target.raw().to_le_bytes());
        instance.state_blob.extend_from_slice(&count.to_le_bytes());
    }
}

pub(crate) fn current_processing_cards(tx: &Transaction, cards: Vec<CardId>) -> Vec<CardId> {
    cards
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|state| {
                tx.state()
                    .zones
                    .get(state.zone)
                    .is_some_and(|zone| zone.kind == ZoneKind::Processing)
            })
        })
        .collect()
}

pub(crate) fn discard_processing_cards(
    tx: &mut Transaction,
    cards: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let cards = current_processing_cards(tx, cards);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
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
    run_effects(
        tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards,
            to: discard,
            from: Some(processing),
            reason: CardMoveReason::Discard,
        }],
    )
}

pub(crate) fn resolve_ddd_liuchong(
    tx: &mut Transaction,
    player: PlayerId,
    actor: PlayerId,
    card: CardId,
    action: u8,
    cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match action {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill) || actor != tx.state().turn.current
            {
                return Ok(Vec::new());
            }
            let alive = tx.state().players.alive_by_seat();
            let first_ready = alive
                .iter()
                .copied()
                .find(|candidate| {
                    tx.state()
                        .players
                        .get(*candidate)
                        .is_some_and(|participant| !participant.turned_over)
                })
                .or_else(|| alive.first().copied());
            let round = tx.state().turn.round;
            let prompted = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| ddd_moyan_state(&instance.state_blob).0)
                .unwrap_or(u32::MAX);
            if first_ready != Some(actor) || prompted == round {
                return Ok(Vec::new());
            }
            save_ddd_moyan_state(tx, skill, round, PlayerId::NONE, 0);
            let draw = tx
                .state()
                .zones
                .find(ZoneKind::DrawPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("draw pile missing".into()))?;
            if tx
                .state()
                .zones
                .get(draw)
                .map_or(0, |zone| zone.cards.len())
                < 3
            {
                reshuffle_discard_into_draw(tx)?;
            }
            let cards: Vec<_> = tx
                .state()
                .zones
                .get(draw)
                .into_iter()
                .flat_map(|zone| zone.cards.iter().rev())
                .take(3)
                .copied()
                .collect();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::ResolveDddLiuchong {
                    player,
                    actor,
                    card: CardId::NONE,
                    action: 3,
                    cards: cards.clone(),
                    skill,
                },
            );
            run_effects(
                tx,
                vec![
                    EffectRequest::MoveCardsWithReason {
                        cards: cards.clone(),
                        to: processing,
                        from: Some(draw),
                        reason: CardMoveReason::Unspecified,
                    },
                    EffectRequest::RevealCards {
                        cards,
                        to: Vec::new(),
                    },
                ],
            )
        }
        1 => {
            if !originating_skill_is_enabled(tx, player, skill) {
                return Ok(Vec::new());
            }
            let (round, target, count) = tx
                .state()
                .skills
                .get(skill)
                .map(|instance| ddd_moyan_state(&instance.state_blob))
                .unwrap_or((u32::MAX, PlayerId::NONE, 0));
            if actor != player
                || round != tx.state().turn.round
                || count == 0
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
                || tx.state().hand_count(target) > count
            {
                return Ok(Vec::new());
            }
            run_effects(
                tx,
                vec![EffectRequest::ModifyCurrentDamage {
                    player,
                    delta: 1,
                    skill,
                }],
            )
        }
        2 => {
            if !originating_skill_is_enabled(tx, player, skill)
                || card.is_none()
                || !tx
                    .state()
                    .cards
                    .get(card)
                    .is_some_and(|state| state.kind.equip_slot() == Some(EquipSlot::Weapon))
            {
                return Ok(Vec::new());
            }
            let range = current_attack_range(tx, player);
            if tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .any(|candidate| current_attack_range(tx, candidate) > range)
            {
                return Ok(Vec::new());
            }
            run_effects(
                tx,
                vec![EffectRequest::ChooseOtherDamage {
                    player,
                    amount: 1,
                    skill,
                }],
            )
        }
        3 => {
            let cards = current_processing_cards(tx, cards);
            if !originating_skill_is_enabled(tx, player, skill) {
                return discard_processing_cards(tx, cards);
            }
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let round = tx.state().turn.round;
            let mut payload = vec![
                RESOLVE_DDD_LIUCHONG_CHOICE,
                0,
                player.raw(),
                skill.raw(),
                round,
                cards.len().min(u32::MAX as usize) as u32,
            ];
            payload.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                0,
                1,
                true,
                tx.state()
                    .players
                    .alive_by_seat()
                    .into_iter()
                    .map(PlayerId::raw)
                    .collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::Custom,
                payload,
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown 刘宠 resolver action".into(),
        )),
    }
}

pub(crate) fn resolve_ddd_liuchong_choice(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() != Some(&RESOLVE_DDD_LIUCHONG_CHOICE) || payload.len() < 6 {
        return Err(EffectError::Illegal("stale 末焱 choice".into()));
    }
    match payload[1] {
        0 => {
            let player = PlayerId::new(payload[2]);
            let skill = engine_api::SkillId::new(payload[3]);
            let round = payload[4];
            let count = payload[5] as usize;
            if observer != player || payload.len() != 6 + count {
                return Err(EffectError::Illegal("invalid 末焱 target context".into()));
            }
            let cards: Vec<_> = payload[6..].iter().copied().map(CardId::new).collect();
            if cancelled || selections.is_empty() {
                return discard_processing_cards(tx, cards).map(|_| ());
            }
            if selections.len() != 1
                || !originating_skill_is_enabled(tx, player, skill)
                || round != tx.state().turn.round
            {
                return Err(EffectError::Illegal("invalid 末焱 target choice".into()));
            }
            let target = PlayerId::new(selections[0]);
            let cards = current_processing_cards(tx, cards);
            if cards.len() != count
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("illegal 末焱 recipient".into()));
            }
            let mut next = vec![
                RESOLVE_DDD_LIUCHONG_CHOICE,
                1,
                player.raw(),
                target.raw(),
                skill.raw(),
                round,
                count as u32,
            ];
            next.extend(cards.iter().map(|card| card.raw()));
            open_choice(
                tx,
                target,
                1,
                count.min(u8::MAX as usize) as u8,
                false,
                cards.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                next,
            )
            .map(|_| ())
        }
        1 => {
            if payload.len() < 7 || cancelled || selections.is_empty() {
                return Err(EffectError::Illegal("invalid 末焱 card choice".into()));
            }
            let player = PlayerId::new(payload[2]);
            let target = PlayerId::new(payload[3]);
            let skill = engine_api::SkillId::new(payload[4]);
            let round = payload[5];
            let count = payload[6] as usize;
            if observer != target
                || payload.len() != 7 + count
                || selections.len() > count
                || !originating_skill_is_enabled(tx, player, skill)
                || round != tx.state().turn.round
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal("invalid 末焱 card context".into()));
            }
            let cards: Vec<_> = payload[7..].iter().copied().map(CardId::new).collect();
            let current = current_processing_cards(tx, cards.clone());
            let unique: std::collections::BTreeSet<_> = selections.iter().copied().collect();
            if current.len() != count
                || unique.len() != selections.len()
                || selections
                    .iter()
                    .any(|raw| !current.contains(&CardId::new(*raw)))
            {
                return Err(EffectError::Illegal("illegal 末焱 cards".into()));
            }
            let selected: Vec<_> = selections.iter().copied().map(CardId::new).collect();
            let rest: Vec<_> = cards
                .into_iter()
                .filter(|card| !selected.contains(card))
                .collect();
            save_ddd_moyan_state(tx, skill, round, target, selected.len() as u32);
            let processing = tx
                .state()
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("processing zone missing".into()))?;
            let hand = tx
                .state()
                .zones
                .find(ZoneKind::Hand, target)
                .ok_or_else(|| EffectError::EngineFault("recipient hand missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let mut effects = vec![EffectRequest::MoveCardsWithReason {
                cards: selected,
                to: hand,
                from: Some(processing),
                reason: CardMoveReason::Unspecified,
            }];
            if !rest.is_empty() {
                effects.push(EffectRequest::MoveCardsWithReason {
                    cards: rest,
                    to: discard,
                    from: Some(processing),
                    reason: CardMoveReason::Discard,
                });
            }
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::Illegal("unknown 末焱 choice stage".into())),
    }
}
