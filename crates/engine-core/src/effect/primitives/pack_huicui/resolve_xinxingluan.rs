//! Effect primitives for `resolve_dcchiying` and related helpers.

use super::*;

pub(crate) fn resolve_xinxingluan(
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
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let mut options = Vec::new();
    options.push(1);
    let any_other_he = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .any(|candidate| {
            candidate != player
                && !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    candidate,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        });
    if any_other_he {
        options.push(2);
        options.push(3);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            RESOLVE_XINXINGLUAN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            0,
            0,
            0,
        ],
    )
}

pub(crate) fn resolve_dcmanzhi(
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
    {
        return Ok(Vec::new());
    }
    // The once-per-turn branch marks reset at each Prepare (turn start).
    if tx.state().turn.phase == Phase::Prepare {
        for mark in [DCMANZHI_GIVE_MARK, DCMANZHI_GAIN_MARK] {
            let count = tx
                .state()
                .players
                .get(player)
                .map(|p| {
                    p.marks
                        .iter()
                        .find(|(m, _)| *m == mark)
                        .map(|(_, c)| *c)
                        .unwrap_or(0)
                })
                .unwrap_or(0);
            if count > 0 {
                run_effects(
                    tx,
                    vec![EffectRequest::RemoveMark {
                        player,
                        mark: engine_api::MarkDefId::new(mark),
                        amount: count.max(0) as u32,
                    }],
                )?;
            }
        }
    }
    let mark = |id: u32| {
        tx.state()
            .players
            .get(player)
            .map(|p| {
                p.marks
                    .iter()
                    .find(|(m, _)| *m == id)
                    .map(|(_, c)| *c)
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    };
    let give_available = mark(DCMANZHI_GIVE_MARK) == 0
        && tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .any(|candidate| {
                candidate != player
                    && !crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        candidate,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
            });
    let gain_available = mark(DCMANZHI_GAIN_MARK) == 0
        && tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .any(|candidate| {
                candidate != player
                    && !crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        candidate,
                        rule_api::TargetCardScope::HandEquipJudge,
                    )
                    .is_empty()
            });
    let mut options = Vec::new();
    if give_available {
        options.push(1);
    }
    if gain_available {
        options.push(2);
    }
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
        ChoiceKind::Custom,
        ContinuationKind::Custom,
        vec![
            RESOLVE_DCMANZHI_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            0,
            0,
            0,
        ],
    )
}

pub(crate) fn resolve_xinfu_qinguo(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
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
    let used_equip = card.is_some()
        && tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card| card.kind.is_equip());
    if !used_equip {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&candidate| candidate != player)
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
        ContinuationKind::Custom,
        vec![
            RESOLVE_XINFU_QINGUO_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn resolve_gongjian(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.current != player
        || targets.is_empty()
    {
        return Ok(Vec::new());
    }
    let previous = tx.state().turn.flag(turn_flag::LAST_SHA_TARGETS).max(0) as u32;
    if previous == 0 {
        return Ok(Vec::new());
    }
    let shared: Vec<_> = targets
        .into_iter()
        .filter(|target| {
            *target != player
                && target
                    .raw()
                    .checked_sub(1)
                    .and_then(|shift| 1_u32.checked_shl(shift))
                    .is_some_and(|bit| previous & bit != 0)
                && !crate::query::discardable_target_cards(
                    tx.state(),
                    player,
                    *target,
                    rule_api::TargetCardScope::HandEquip,
                )
                .is_empty()
        })
        .collect();
    if shared.is_empty() {
        return Ok(Vec::new());
    }
    open_gongjian_discard_choice(tx, player, skill, &shared, 0, Vec::new())
}

pub(crate) fn open_gongjian_discard_choice(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    shared: &[PlayerId],
    index: usize,
    accumulated: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target = shared[index];
    let options: Vec<_> = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    )
    .into_iter()
    .map(CardId::raw)
    .collect();
    let mut payload = vec![
        RESOLVE_GONGJIAN_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        shared.len() as u32,
        index as u32,
    ];
    payload.extend(shared.iter().map(|target| target.raw()));
    payload.push(accumulated.len() as u32);
    payload.extend(accumulated.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        1,
        options.len().min(2).min(u8::MAX as usize) as u8,
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        payload,
    )
}

pub(crate) fn resolve_xinfu_lianpian(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || targets.is_empty()
    {
        return Ok(Vec::new());
    }
    let previous = tx.state().turn.flag(turn_flag::LAST_CARD_TARGETS).max(0) as u32;
    if previous == 0 {
        return Ok(Vec::new());
    }
    let shared: Vec<_> = targets
        .into_iter()
        .filter(|target| {
            *target != player
                && target
                    .raw()
                    .checked_sub(1)
                    .and_then(|shift| 1_u32.checked_shl(shift))
                    .is_some_and(|bit| previous & bit != 0)
        })
        .collect();
    if shared.is_empty() {
        return Ok(Vec::new());
    }
    // Draw one card and remember it.
    let before: Vec<_> = tx.state().hand_cards(player);
    run_effects(
        tx,
        vec![EffectRequest::DrawCardsFromSkill {
            player,
            count: 1,
            skill,
        }],
    )?;
    let after: Vec<_> = tx.state().hand_cards(player);
    let drawn = after
        .into_iter()
        .find(|card| !before.contains(card))
        .ok_or_else(|| EffectError::Illegal("Lianpian draw produced no card".into()))?;
    open_choice(
        tx,
        player,
        1,
        1,
        true,
        shared.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        vec![
            RESOLVE_XINFU_LIANPIAN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            drawn.raw(),
        ],
    )
}

pub(crate) fn resolve_spfengyin(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || target == player
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || tx.state().turn.current != target
    {
        return Ok(Vec::new());
    }
    // The target's HP must be at least the owner's.
    let (owner_hp, target_hp) = (
        tx.state().players.get(player).map(|p| p.hp).unwrap_or(0),
        tx.state().players.get(target).map(|p| p.hp).unwrap_or(0),
    );
    if target_hp < owner_hp {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|card| card.kind.is_sha())
        })
        .map(CardId::raw)
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
        ChoiceKind::SelectCards,
        ContinuationKind::Custom,
        vec![
            RESOLVE_SPFENGYIN_CUSTOM_CHOICE,
            player.raw(),
            skill.raw(),
            target.raw(),
        ],
    )
}

pub(crate) fn skip_next_play_phase(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    if state.turn.current != player
        || !state
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "skip-next-play-phase owner is no longer current and alive".into(),
        ));
    }
    if !matches!(
        state.turn.phase,
        Phase::Prepare | Phase::Judge | Phase::Draw
    ) {
        return Err(EffectError::ContentFault(
            "skip_next_play_phase is only valid before the current Play phase".into(),
        ));
    }
    tx.state_mut().turn.set_flag(turn_flag::SKIP_PLAY, 1);
    Ok(Vec::new())
}

pub(crate) fn skip_next_judge_phase(
    tx: &mut Transaction,
    player: PlayerId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if !tx.state().pending_judge_skips.contains(&player) {
        tx.state_mut().pending_judge_skips.push(player);
        tx.state_mut()
            .pending_judge_skips
            .sort_by_key(|participant| participant.raw());
    }
    Ok(Vec::new())
}

pub(crate) fn skip_judge_and_draw_phases(
    tx: &mut Transaction,
    player: PlayerId,
    _skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "skip-judge-and-draw requires a living player".into(),
        ));
    }
    if tx.state().turn.current != player {
        return Err(EffectError::ContentFault(
            "skip_judge_and_draw_phases requires the current phase player".into(),
        ));
    }
    if !matches!(tx.state().turn.phase, Phase::Prepare | Phase::Judge) {
        return Err(EffectError::ContentFault(
            "skip_judge_and_draw_phases is only valid during Prepare or Judge".into(),
        ));
    }
    // 复用既有 pending_judge_skip 机制：判定阶段入口已检查该队列，
    // 触发体内设置后由 turn.rs 在进入判定时消费。
    if !tx.state().pending_judge_skips.contains(&player) {
        tx.state_mut().pending_judge_skips.push(player);
        tx.state_mut()
            .pending_judge_skips
            .sort_by_key(|participant| participant.raw());
    }
    tx.state_mut().turn.set_flag(turn_flag::SKIP_DRAW, 1);
    Ok(Vec::new())
}

pub(crate) fn choose_add_current_sha_target(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_add_current_sha_target_within_distance(tx, player, card, u32::MAX, skill)
}

pub(crate) fn choose_add_current_sha_target_within_distance(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    max_distance: u32,
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
    let targets = crate::command::play::saved_sha_additional_targets(tx, player, card)?
        .into_iter()
        .filter(|target| {
            matches!(
                crate::query::query(
                    tx.state(),
                    &QueryRequest::Distance {
                        from: player,
                        to: *target,
                    },
                    &mut crate::query::QueryContext::new(),
                ),
                QueryResponse::Int(distance)
                    if distance <= max_distance.min(i32::MAX as u32) as i32
            )
        })
        .collect::<Vec<_>>();
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
        ContinuationKind::ChooseAddCurrentShaTarget,
        vec![player.raw(), card.raw(), skill.raw()],
    )
}

pub(crate) fn choose_compare_selected_hand_card(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    win_skill_def: engine_api::SkillDefId,
    lose_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    let valid_skill = state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    });
    if !valid_skill {
        return Err(EffectError::ContentFault(
            "hand comparison requires its active skill instance".into(),
        ));
    }
    if !state
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || player == target
    {
        return Err(EffectError::Illegal(
            "hand comparison requires two different living players".into(),
        ));
    }
    let hand = state
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("comparison owner has no hand zone".into()))?;
    if !state
        .cards
        .get(card)
        .is_some_and(|value| value.owner == player && value.zone == hand)
    {
        return Err(EffectError::Illegal(
            "selected comparison card is no longer in hand".into(),
        ));
    }
    let options = state.hand_cards(target);
    if options.is_empty() {
        return Err(EffectError::Illegal(
            "comparison target has no hand card".into(),
        ));
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseCompareTargetHandCard,
        vec![
            player.raw(),
            target.raw(),
            card.raw(),
            win_skill_def.raw(),
            lose_skill_def.raw(),
            skill.raw(),
        ],
    )
}

/// ABI 678 (batch 166): hand comparison whose strict-win branch grants `win_grant_def`
/// to the **target** instead of the chooser; tie/loss grants nothing. Payload reuses
/// continuations 75/76/49 with a trailing `flag=1` to mark the grant flow.
pub(crate) fn choose_hand_compare_grant_target_on_win(
    tx: &mut Transaction,
    player: PlayerId,
    win_grant_def: engine_api::SkillDefId,
    lose_skill_def: engine_api::SkillDefId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    if !state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::ContentFault(
            "hand comparison requires its skill instance".into(),
        ));
    }
    if !state
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        || state.hand_cards(player).is_empty()
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && !state.hand_cards(target).is_empty())
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
        ContinuationKind::ChooseHandCompareTarget,
        vec![
            player.raw(),
            win_grant_def.raw(),
            lose_skill_def.raw(),
            skill.raw(),
            1,
        ],
    )
}
