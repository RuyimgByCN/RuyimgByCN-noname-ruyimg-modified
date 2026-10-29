//! Command operations for `source_free_wuzhong_has_legal_use` and related helpers.

use super::*;

pub(crate) fn saved_independent_trick_damage_card_and_bonus(args: &[u32]) -> (CardId, u32) {
    (
        queued_independent_trick(args)
            .map(|(payload, _)| CardId::new(payload[0]))
            .unwrap_or(CardId::NONE),
        saved_independent_trick_damage_bonus(args),
    )
}

pub(crate) fn use_independent_trick_targets(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    effect_tag: u32,
    field_source: bool,
) -> Result<(), EffectError> {
    let effect_args = independent_trick_effect_args(tx, actor, card, targets, effect_tag)?;
    let use_effect = if field_source {
        EffectRequest::UseFieldCard {
            player: actor,
            card,
            targets: targets.to_vec(),
        }
    } else {
        EffectRequest::UseCard {
            player: actor,
            card,
            targets: targets.to_vec(),
        }
    };
    run_effects_then_wuxie(tx, vec![use_effect], actor, effect_tag, actor, &effect_args)
}

pub(crate) fn independent_trick_effect_args(
    tx: &Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    effect_tag: u32,
) -> Result<Vec<u32>, EffectError> {
    let source_card = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("bad trick card".into()))?;
    let kind = crate::query::effective_card_kind(tx.state(), actor, source_card);
    let suit = crate::query::effective_card_suit(tx.state(), actor, source_card);
    let mut payload = vec![
        card.raw(),
        actor.raw(),
        kind.tag() as u32,
        super::super::judge::suit_tag(suit),
        effect_tag,
        0,
        targets.len() as u32,
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
    let mut effect_args = Vec::with_capacity(payload.len() + 1);
    effect_args.push(INDEPENDENT_TRICK_QUEUE_MAGIC);
    effect_args.extend_from_slice(&payload);
    Ok(effect_args)
}

/// Resolve an already-used independent trick without opening a Wuxie window.
pub(crate) fn resolve_independent_trick_without_wuxie(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    targets: &[PlayerId],
    effect_tag: u32,
) -> Result<(), EffectError> {
    let effect_args = independent_trick_effect_args(tx, actor, card, targets, effect_tag)?;
    apply_deferred_trick(tx, effect_tag, actor, &effect_args, card)
}

/// Continue the next target of a non-global ordinary trick.
pub(crate) fn advance_independent_trick_queue(
    tx: &mut Transaction,
    payload: &[u32],
) -> Result<(), EffectError> {
    if payload.len() < 7 {
        return Err(EffectError::EngineFault(
            "malformed independent trick queue".into(),
        ));
    }
    let actor = PlayerId::new(payload[1]);
    let kind = CardKind::from_tag(payload[2] as u8)
        .ok_or_else(|| EffectError::EngineFault("bad queued trick kind".into()))?;
    let suit = super::super::judge::suit_from_tag(payload[3]);
    let effect_tag = payload[4];
    let index = payload[5] as usize;
    let len = payload[6] as usize;
    if payload.len() < 7 + len {
        return Err(EffectError::EngineFault(
            "truncated independent trick queue".into(),
        ));
    }
    if index >= len {
        let resolution = &mut tx.state_mut().resolution;
        resolution.continuation_kind = crate::state::ContinuationKind::None;
        resolution.continuation_payload.clear();
        if effect_tag == trick_effect::MANTIANGUOHAI {
            return begin_mantianguohai_gives(tx, payload);
        }
        if effect_tag == trick_effect::DIAOBINGQIANJIANG {
            let extension = 7usize.saturating_add(len);
            if payload.get(extension) != Some(&DIAOBING_POOL_MAGIC) {
                return Err(EffectError::EngineFault(
                    "调兵遣将 pool marker is missing".into(),
                ));
            }
            let count = *payload.get(extension + 1).unwrap_or(&u32::MAX) as usize;
            let revealed: Vec<_> = payload
                .get(extension + 2..extension + 2 + count)
                .ok_or_else(|| EffectError::EngineFault("调兵遣将 pool is truncated".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            return run_effects(
                tx,
                vec![EffectRequest::ContinueDiaobingqianjiang {
                    action: 1,
                    actor,
                    source_card: CardId::new(payload[0]),
                    targets: Vec::new(),
                    revealed,
                }],
            )
            .map(|_| ());
        }
        if effect_tag == trick_effect::WUGU {
            let extension = 7usize.saturating_add(len);
            if payload.get(extension) != Some(&WUGU_POOL_MAGIC) {
                return Err(EffectError::EngineFault(
                    "五谷丰登 pool marker is missing".into(),
                ));
            }
            let count = *payload.get(extension + 1).unwrap_or(&u32::MAX) as usize;
            let revealed: Vec<_> = payload
                .get(extension + 2..extension + 2 + count)
                .ok_or_else(|| EffectError::EngineFault("五谷丰登 pool is truncated".into()))?
                .iter()
                .copied()
                .map(CardId::new)
                .collect();
            return run_effects(
                tx,
                vec![EffectRequest::ContinueWugu {
                    action: 2,
                    actor,
                    source_card: CardId::new(payload[0]),
                    targets: Vec::new(),
                    excluded: Vec::new(),
                    revealed,
                }],
            )
            .map(|_| ());
        }
        return Ok(());
    }
    let mut effect_args = Vec::with_capacity(payload.len() + 1);
    effect_args.push(INDEPENDENT_TRICK_QUEUE_MAGIC);
    effect_args.extend_from_slice(payload);
    if effect_tag == trick_effect::HAOLINGTIANXIA {
        // 号令天下：非标准逐目标锦囊——链目标由 advance_haolingtianxia_player
        // 依次二选一驱动，不开启无懈窗口。
        return advance_haolingtianxia_player(tx, payload);
    }
    begin_wuxie_window(tx, actor, effect_tag, actor, kind, suit, &effect_args)
}

pub(crate) fn queued_independent_trick(args: &[u32]) -> Option<(&[u32], PlayerId)> {
    if args.first().copied() != Some(INDEPENDENT_TRICK_QUEUE_MAGIC) {
        return None;
    }
    let payload = args.get(1..)?;
    let index = *payload.get(5)? as usize;
    let len = *payload.get(6)? as usize;
    if index >= len || payload.len() < 7 + len {
        return None;
    }
    Some((payload, PlayerId::new(payload[7 + index])))
}

/// Finish one independently resolved trick target and advance to the next target.
pub(crate) fn finish_independent_trick_target(
    tx: &mut Transaction,
    args: &[u32],
) -> Result<(), EffectError> {
    let Some((payload, _)) = queued_independent_trick(args) else {
        return Ok(());
    };
    let mut next = payload.to_vec();
    next[5] = next[5].saturating_add(1);
    advance_independent_trick_queue(tx, &next)
}

pub(crate) fn begin_mantianguohai_gives(
    tx: &mut Transaction,
    queue: &[u32],
) -> Result<(), EffectError> {
    if queue.len() < 7 {
        return Err(EffectError::EngineFault(
            "malformed 瞒天过海 target queue".into(),
        ));
    }
    let actor = PlayerId::new(queue[1]);
    let len = queue[6] as usize;
    if queue.len() < 7 + len {
        return Err(EffectError::EngineFault(
            "truncated 瞒天过海 target queue".into(),
        ));
    }
    let successful: std::collections::BTreeSet<_> = queue[7..7 + len]
        .iter()
        .filter(|raw| **raw & MANTIANGUOHAI_SUCCESS_MASK != 0)
        .map(|raw| PlayerId::new(*raw & !MANTIANGUOHAI_SUCCESS_MASK))
        .collect();
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| successful.contains(target))
        .collect();
    resolve_mantianguohai_give(tx, actor, targets)
}

pub(crate) fn resolve_mantianguohai_give(
    tx: &mut Transaction,
    actor: PlayerId,
    mut targets: Vec<PlayerId>,
) -> Result<(), EffectError> {
    loop {
        let Some(target) = targets.first().copied() else {
            return Ok(());
        };
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|player| player.alive)
        {
            targets.remove(0);
            continue;
        }
        let mut options = tx.state().hand_cards(actor);
        if let Some(equip) = tx.state().zones.find(ZoneKind::Equip, actor) {
            options.extend(
                tx.state()
                    .zones
                    .get(equip)
                    .into_iter()
                    .flat_map(|zone| zone.cards.iter().copied()),
            );
        }
        if options.is_empty() {
            return Ok(());
        }
        let mut payload = vec![
            MANTIANGUOHAI_CUSTOM_CHOICE,
            1,
            actor.raw(),
            target.raw(),
            targets.len() as u32,
        ];
        payload.extend(targets.iter().map(|target| target.raw()));
        crate::effect::open_choice(
            tx,
            actor,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            engine_api::ChoiceKind::SelectCards,
            crate::state::ContinuationKind::Custom,
            payload,
        )?;
        return Err(EffectError::ChoiceRequested);
    }
}

/// Apply one target's effects while preserving the remaining trick queue across choices/dying.
pub(crate) fn run_effects_then_finish_independent_trick_target(
    tx: &mut Transaction,
    effects: Vec<EffectRequest>,
    args: &[u32],
) -> Result<(), EffectError> {
    let Some((payload, _)) = queued_independent_trick(args) else {
        return run_effects(tx, effects).map(|_| ());
    };
    let mut next = payload.to_vec();
    next[5] = next[5].saturating_add(1);
    let (_, next) = run_effects_with_saved_resume(
        tx,
        effects,
        crate::state::ContinuationKind::IndependentTrickQueue,
        next,
    )?;
    advance_independent_trick_queue(tx, &next)
}

/// Run effects that may suspend on a nested choice (e.g. a damage trigger),
/// resuming into the independent-trick queue once the whole nested chain completes.
/// Used by 号令天下's per-player chain so each step can continue to the next actor.
pub(crate) fn run_effects_then_resume_trick_queue(
    tx: &mut Transaction,
    effects: Vec<EffectRequest>,
    queue: Vec<u32>,
) -> Result<(), EffectError> {
    let (_, next) = run_effects_with_saved_resume(
        tx,
        effects,
        crate::state::ContinuationKind::IndependentTrickQueue,
        queue,
    )?;
    advance_independent_trick_queue(tx, &next)
}

/// Advance / open the 号令天下 (guozhan) per-player option chain.
///
/// Queue layout (an independent-trick-queue superset):
/// `[0] card, [1] actor, [2] kind, [3] suit, [4] HAOLINGTIANXIA, [5] index, [6] len,
/// [7..7+len] chain players (seat order, excluding the trick target),
/// [7+len] trick target, [8+len..] original trick args tail (finish resume).`
///
/// Each live chain player with at least one legal option gets a stage-1 choice
/// (① 弃牌视为对目标用杀 / ② 弃置或获得目标一张牌); the trick user is a chain
/// member too when not the target. Dead / option-less players are skipped.
pub(crate) fn advance_haolingtianxia_player(
    tx: &mut Transaction,
    queue: &[u32],
) -> Result<(), EffectError> {
    if queue.len() < 8 {
        return Err(EffectError::EngineFault(
            "malformed 号令天下 chain queue".into(),
        ));
    }
    let index = queue[5] as usize;
    let len = queue[6] as usize;
    if index >= len {
        let tail = queue.get(8 + len..).unwrap_or_default();
        return finish_independent_trick_target(tx, tail);
    }
    let current = PlayerId::new(queue[7 + index]);
    let trick_target = PlayerId::new(queue[7 + len]);
    let advance = |tx: &mut Transaction| -> Result<(), EffectError> {
        let mut next = queue.to_vec();
        next[5] = next[5].saturating_add(1);
        advance_haolingtianxia_player(tx, &next)
    };
    if !tx
        .state()
        .players
        .get(current)
        .is_some_and(|player| player.alive)
    {
        return advance(tx);
    }
    let is_wei = tx
        .state()
        .player_group(current)
        .map(str::to_string)
        .as_deref()
        == Some("wei");
    let mut choices: Vec<u32> = Vec::new();
    // ① 视为对目标使用杀：需在攻击范围内，且（魏则免弃牌）或本人有可弃的「he」牌。
    if sha_in_range(tx, current, trick_target)
        && (is_wei || !target_zone_card_tokens(tx, current, true, true, false).is_empty())
    {
        choices.push(1);
    }
    // ② 弃置（或魏势力获得）目标一张牌：目标需有「he」牌。
    if !target_zone_card_tokens(tx, trick_target, true, true, false).is_empty() {
        choices.push(2);
    }
    if choices.is_empty() {
        return advance(tx);
    }
    let mut payload = vec![crate::effect::HAOLINGTIANXIA_CUSTOM_CHOICE, 1];
    payload.extend_from_slice(queue);
    crate::effect::open_choice(
        tx,
        current,
        1,
        1,
        true,
        choices,
        engine_api::ChoiceKind::Custom,
        crate::state::ContinuationKind::Custom,
        payload,
    )?;
    Err(EffectError::ChoiceRequested)
}

pub(crate) fn equip_card(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
    kind: CardKind,
) -> Result<(), EffectError> {
    let slot = kind
        .equip_slot()
        .ok_or_else(|| EffectError::Illegal("card has no equipment slot".into()))?;
    if !tx
        .state()
        .players
        .get(actor)
        .is_some_and(|player| player.equip_slot_enabled(slot))
    {
        return Err(EffectError::Illegal("equipment slot disabled".into()));
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, actor)
        .ok_or_else(|| EffectError::Illegal("no hand".into()))?;
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, actor)
        .ok_or_else(|| EffectError::Illegal("no equip".into()))?;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard".into()))?;
    let source = tx
        .state()
        .cards
        .get(card)
        .map(|card| card.zone)
        .ok_or_else(|| EffectError::Illegal("equipment card disappeared".into()))?;
    if source != hand
        && crate::query::borrowed_hand_source(tx.state(), actor, card).is_none()
        && !tx.state().resolution.has_planned_card_source(actor, card)
    {
        return Err(EffectError::Illegal(
            "equipment card is not usable from hand".into(),
        ));
    }
    let borrowed_refills = crate::effect::pothaoshi_refill_effects(tx.state(), actor, &[card]);

    // Unequip same slot first
    let existing: Vec<CardId> = tx
        .state()
        .zones
        .get(equip)
        .map(|z| z.cards.clone())
        .unwrap_or_default()
        .into_iter()
        .filter(|cid| {
            tx.state()
                .cards
                .get(*cid)
                .map(|c| same_equip_slot(c.kind, kind))
                .unwrap_or(false)
        })
        .collect();

    let mut queue = vec![EffectRequest::UseCard {
        player: actor,
        card,
        targets: vec![],
    }];
    if !existing.is_empty() {
        queue.push(EffectRequest::MoveCards {
            cards: existing,
            to: discard,
            from: Some(equip),
        });
    }
    queue.push(EffectRequest::MoveCardsWithReason {
        cards: vec![card],
        to: equip,
        from: Some(source),
        reason: CardMoveReason::Use,
    });
    queue.extend(borrowed_refills);
    queue.push(EffectRequest::EmitRuleEvent {
        code: 50, // equipped
        payload: vec![actor.raw(), card.raw()],
    });
    run_effects(tx, queue)?;
    crate::effect::finish_pending_card_replays_if_idle(tx)
}

/// Cancel the exact card use currently suspended behind its CardUsed triggers.
pub(crate) fn cancel_saved_card_use(
    tx: &mut Transaction,
    actor: PlayerId,
    card: CardId,
) -> Result<(), EffectError> {
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|state| crate::query::effective_card_kind(tx.state(), actor, state))
        .ok_or_else(|| EffectError::Illegal("current card disappeared".into()))?;
    if kind.is_sha() {
        let targets = saved_targeted_card_targets(tx, actor, card)?;
        return exclude_all_saved_card_targets(tx, actor, card, &targets);
    }
    if kind.is_trick() {
        let resolution = &mut tx.state_mut().resolution;
        if resolution.resume_kind != ContinuationKind::WuxieWindow
            || !resolution
                .resume_payload
                .windows(2)
                .any(|pair| pair == [SAVED_WUXIE_SOURCE_CARD_MAGIC, card.raw()])
        {
            return Err(EffectError::Illegal(
                "no matching trick use to cancel".into(),
            ));
        }
        resolution.resume_payload[1] = 1;
        return Ok(());
    }
    if kind.is_equip() {
        let tail = &mut tx.state_mut().resolution.deferred_tail_effects;
        let Some(end) = tail.iter().position(|effect| {
            matches!(effect, EffectRequest::EmitRuleEvent { code: 50, payload } if payload.get(1) == Some(&card.raw()))
        }) else {
            return Err(EffectError::Illegal("no matching equipment use to cancel".into()));
        };
        tail.drain(..=end);
        return Ok(());
    }
    let before = tx.state().resolution.deferred_tail_effects.len();
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .retain(|effect| match effect {
            EffectRequest::RecoverHp { source, player, .. }
                if *source == actor && *player == actor =>
            {
                false
            }
            EffectRequest::AddMark { player, mark, .. }
                if *player == actor && mark.raw() == crate::content::MARK_JIU =>
            {
                false
            }
            _ => true,
        });
    if before == tx.state().resolution.deferred_tail_effects.len() {
        return Err(EffectError::Illegal(
            "current card use cannot be cancelled".into(),
        ));
    }
    Ok(())
}

pub(crate) fn same_equip_slot(a: CardKind, b: CardKind) -> bool {
    matches!(
        (a, b),
        (CardKind::Weapon, CardKind::Weapon)
            | (CardKind::Armor, CardKind::Armor)
            | (CardKind::HorseOff, CardKind::HorseOff)
            | (CardKind::HorseDef, CardKind::HorseDef)
            | (CardKind::Treasure, CardKind::Treasure)
    )
}

pub(crate) fn has_delayed_of_kind(tx: &Transaction, player: PlayerId, kind: CardKind) -> bool {
    let Some(jz) = tx.state().zones.find(ZoneKind::Judge, player) else {
        return false;
    };
    let Some(z) = tx.state().zones.get(jz) else {
        return false;
    };
    z.cards.iter().any(|cid| {
        tx.state()
            .cards
            .get(*cid)
            .map(|c| c.kind == kind)
            .unwrap_or(false)
    })
}

/// After 杀 is used: ask target to play 闪 (or view_as 闪). Cancel = damage.
/// 无双: need 2 闪. `nature` = damage_nature tag for the eventual damage.
pub(crate) fn begin_sha_response(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
    nature: u8,
    card_flags: u8,
    respondable: bool,
    card: Option<CardId>,
) -> Result<(), EffectError> {
    let need = crate::query::sha_response_count_for(tx.state(), source, target, 1);
    let respondable = respondable
        && !crate::query::sha_use_unrespondable_within_distance(tx.state(), source, &[target])
        && !crate::query::card_use_unrespondable_within_distance(tx.state(), source, target)
        && card.is_none_or(|card| {
            !crate::query::sha_unrespondable_for_card_target(tx.state(), source, target, card)
                && !crate::query::potshishi_blocks_response(tx.state(), source, card, target)
        });
    // Source damage modifiers (裸衣 +1 etc.) apply to 杀 damage.
    let amount = card.map_or_else(
        || crate::query::sha_damage_for(tx.state(), source, amount),
        |card| crate::query::sha_damage_for_card(tx.state(), source, card, amount),
    );
    open_sha_response_step(
        tx,
        source,
        target,
        amount,
        need,
        0,
        nature,
        card_flags,
        respondable,
        card.unwrap_or(CardId::NONE),
    )
}

/// Resolve a multi-target 杀 through the existing ordered response queue.
/// Queue payload: `[3, source, amount, idx, len, nature, need, have, t0, ..., r0, ..., card]`.
pub(crate) fn begin_sha_queue(
    tx: &mut Transaction,
    source: PlayerId,
    targets: &[PlayerId],
    amount: u32,
    nature: u8,
    card_flags: u8,
    respondable: bool,
    card: Option<CardId>,
    forced_unrespondable: &[PlayerId],
    target_damage_bonuses: &[u32],
) -> Result<(), EffectError> {
    let need = 1;
    let use_locked =
        crate::query::sha_use_unrespondable_within_distance(tx.state(), source, targets);
    let amount = card.map_or_else(
        || crate::query::sha_damage_for(tx.state(), source, amount),
        |card| crate::query::sha_damage_for_card(tx.state(), source, card, amount),
    );
    let mut payload = vec![
        3,
        source.raw(),
        amount,
        0,
        targets.len() as u32,
        pack_damage_card_context(nature, card_flags),
        need,
        0,
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
    payload.extend(targets.iter().enumerate().map(|(index, target)| {
        let target_locked = card.is_some_and(|card| {
            crate::query::sha_unrespondable_for_card_target(tx.state(), source, *target, card)
                || crate::query::potshishi_blocks_response(tx.state(), source, card, *target)
        }) || forced_unrespondable.contains(target);
        let distance_locked =
            crate::query::card_use_unrespondable_within_distance(tx.state(), source, *target);
        let response_flag =
            u32::from(respondable && !use_locked && !target_locked && !distance_locked);
        let bonus = target_damage_bonuses
            .get(index)
            .copied()
            .unwrap_or(0)
            .min(u32::MAX >> SHA_TARGET_DAMAGE_BONUS_SHIFT);
        response_flag | (bonus << SHA_TARGET_DAMAGE_BONUS_SHIFT)
    }));
    payload.push(card.unwrap_or(CardId::NONE).raw());
    let resolution = &mut tx.state_mut().resolution;
    resolution.continuation_kind = crate::state::ContinuationKind::RespondAoeQueue;
    resolution.continuation_payload = payload;
    advance_aoe_queue(tx)
}
