//! Effect primitives for `resolve_sibian` and related helpers.

use super::*;

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
        return Err(EffectError::Illegal(
            "insert-extra-turn chooser mismatch".into(),
        ));
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
    if !tx.state().players.get(target).is_some_and(|p| p.alive) {
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
