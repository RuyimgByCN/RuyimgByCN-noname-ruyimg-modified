//! Effect primitives for `resolve_twtianyu` and related helpers.

use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twguohuai(
    tx: &mut Transaction,
    player: PlayerId,
    _target: PlayerId,
    source: PlayerId,
    card: CardId,
    mark_def: u32,
    helper_skill_def: u32,
    action: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            let earned = tx.state().turn.drew_in_play_phase(player)
                || tx.state().turn.damage_dealt_count(player) > 0;
            if !earned && card.is_some() {
                let baseline = tx.state().turn.damage_dealt_count(player);
                tx.state_mut().skills.get_mut(skill).unwrap().state_blob =
                    crate::state::SkillInstance::phase_card_use_policy_state(
                        card,
                        crate::state::SkillInstance::CARD_USE_POLICY_DRAW_ON_DAMAGE,
                        baseline,
                    );
            }
            let mut effects = vec![EffectRequest::DrawCards { player, count: 2 }];
            if earned {
                effects.push(EffectRequest::AddMark {
                    player,
                    mark: engine_api::MarkDefId::new(mark_def),
                    delta: 1,
                });
            }
            run_effects(tx, effects)
        }
        (1, 0) => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_phase_card_use_policy_state();
            add_mark(tx, player, mark_def, 1)
        }
        (2, 0) => {
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .clear_phase_card_use_policy_state();
            Ok(Vec::new())
        }
        (3, 0) => {
            let mut options = vec![1];
            if !crate::query::discardable_hand_equip_cards(tx.state(), source).is_empty() {
                options.push(2);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options,
                ChoiceKind::Custom,
                ContinuationKind::ResolveTwguohuai,
                vec![
                    0,
                    player.raw(),
                    source.raw(),
                    mark_def,
                    helper_skill_def,
                    skill.raw(),
                ],
            )
        }
        (4, 0) => {
            let mut events = remove_mark(tx, player, mark_def, 1)?;
            events.extend(skip_current_phase(tx, player)?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Guohuai resolver stage".into(),
        )),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twchengpu(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    card: CardId,
    chunlao_skill_def: u32,
    action: u8,
    stage: u8,
    _selected_card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            crate::command::play::convert_saved_sha_to_fire(tx, player, card)?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .set_twlihuo_dying_cost(card);
            Ok(Vec::new())
        }
        (1, 0) => {
            let targets: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| !twchengpu_field_cards(tx.state(), *target).is_empty())
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
                    TWCHENGPU_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    chunlao_skill_def,
                    skill.raw(),
                ],
            )
        }
        (2, 0) if source == player => {
            crate::command::play::increase_saved_sha_base_damage(tx, source, card, 1)?;
            Ok(Vec::new())
        }
        (2, 0) => {
            let options = twchengpu_hand_equip_cards(tx.state(), source);
            if options.is_empty() {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                source,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::Custom,
                vec![
                    TWCHENGPU_CUSTOM_CHOICE,
                    2,
                    player.raw(),
                    source.raw(),
                    card.raw(),
                    chunlao_skill_def,
                    skill.raw(),
                ],
            )
        }
        (2, 1) => {
            crate::command::play::increase_saved_sha_base_damage(tx, source, card, 1)?;
            Ok(Vec::new())
        }
        (3, 0) => {
            let cards = tx.state().skill_pile_cards(target, chunlao_skill_def);
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let from = tx
                .state()
                .skill_piles
                .iter()
                .find(|pile| pile.owner == target && pile.skill_def == chunlao_skill_def)
                .map(|pile| pile.zone)
                .ok_or_else(|| EffectError::EngineFault("TW Chunlao pile missing".into()))?;
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            let mut events =
                move_cards_with_reason(tx, cards, discard, Some(from), CardMoveReason::Discard)?;
            events.extend(run_effects(
                tx,
                vec![
                    EffectRequest::DrawCards { player, count: 1 },
                    EffectRequest::RecoverHp {
                        source: player,
                        player: target,
                        amount: 1,
                    },
                ],
            )?);
            Ok(events)
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Chengpu resolver stage".into(),
        )),
    }
}

pub(crate) fn twchengpu_field_cards(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<CardId> {
    [ZoneKind::Hand, ZoneKind::Equip, ZoneKind::Judge]
        .into_iter()
        .filter_map(|kind| state.zones.find(kind, player))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect()
}

pub(crate) fn twchengpu_hand_equip_cards(
    state: &crate::state::GameState,
    player: PlayerId,
) -> Vec<CardId> {
    [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| state.zones.find(kind, player))
        .filter_map(|zone| state.zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_twcaoang(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    source: PlayerId,
    card: CardId,
    helper_skill_def: u32,
    action: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            crate::command::play::redirect_saved_sha_target(tx, source, card, target, player)?;
            tx.state_mut()
                .skills
                .get_mut(skill)
                .unwrap()
                .set_twxiaolian_redirect(card, target);
            Ok(Vec::new())
        }
        (1, 0) => {
            let Some((remembered, original_target)) = tx
                .state()
                .skills
                .get(skill)
                .and_then(|skill| skill.twxiaolian_redirect())
            else {
                return Ok(Vec::new());
            };
            if remembered != card {
                return Ok(Vec::new());
            }
            let options = twchengpu_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .unwrap()
                    .clear_twxiaolian_redirect();
                return Ok(Vec::new());
            }
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
                    TWCAOANG_CUSTOM_CHOICE,
                    0,
                    player.raw(),
                    original_target.raw(),
                    helper_skill_def,
                    skill.raw(),
                ],
            )
        }
        (2, 0) => {
            if tx
                .state()
                .skills
                .get(skill)
                .and_then(|skill| skill.twxiaolian_redirect())
                .is_some_and(|(remembered, _)| remembered == card)
            {
                tx.state_mut()
                    .skills
                    .get_mut(skill)
                    .unwrap()
                    .clear_twxiaolian_redirect();
            }
            Ok(Vec::new())
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Caoang resolver stage".into(),
        )),
    }
}

pub(crate) fn resolve_twniujin(
    tx: &mut Transaction,
    player: PlayerId,
    limited_skill_def: engine_api::SkillDefId,
    action: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    match (action, stage) {
        (0, 0) => {
            let hand = tx.state().hand_count(player);
            let highest = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|target| tx.state().hand_count(target))
                .max()
                .unwrap_or(hand);
            let count = highest.saturating_sub(hand).min(5);
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(EffectRequest::ResolveTwniujin {
                    player,
                    limited_skill_def,
                    action,
                    stage: 1,
                    skill,
                });
            run_effects(tx, vec![EffectRequest::DrawCards { player, count }])
        }
        (0, 1) if !tx.state().disabled_judge_zones.contains(&player) => {
            tx.state_mut().disabled_judge_zones.insert(player);
            let judge = tx
                .state()
                .zones
                .find(ZoneKind::Judge, player)
                .ok_or_else(|| EffectError::EngineFault("TW Niujin judge zone missing".into()))?;
            let cards = tx
                .state()
                .zones
                .get(judge)
                .map(|zone| zone.cards.clone())
                .unwrap_or_default();
            if cards.is_empty() {
                return Ok(Vec::new());
            }
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .ok_or_else(|| EffectError::EngineFault("discard pile missing".into()))?;
            move_cards_with_reason(
                tx,
                cards,
                discard,
                Some(judge),
                engine_api::CardMoveReason::Discard,
            )
        }
        (0, 1) => {
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|target| *target != player)
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
                vec![TWNIUJIN_CUSTOM_CHOICE, 0, player.raw(), skill.raw()],
            )
        }
        (1, 0) => {
            let can_reset = tx
                .state()
                .skills
                .sorted_live()
                .into_iter()
                .find(|instance| instance.owner == player && instance.def == limited_skill_def)
                .is_some_and(|instance| instance.lifetime_fire_count > 0);
            if !can_reset {
                return draw_cards(tx, player, 2);
            }
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                vec![0, 1],
                ChoiceKind::Custom,
                ContinuationKind::Custom,
                vec![
                    TWNIUJIN_CUSTOM_CHOICE,
                    1,
                    player.raw(),
                    limited_skill_def.raw(),
                    skill.raw(),
                ],
            )
        }
        _ => Err(EffectError::ContentFault(
            "unknown TW Niujin resolver stage".into(),
        )),
    }
}

pub(crate) fn resolve_twmoukui(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 {
        return Err(EffectError::ContentFault(
            "unknown TW Moukui resolver stage".into(),
        ));
    }
    // 谋溃只在自己的【杀】指定该目标的结算窗口内发动。
    if !tx
        .state()
        .resolution
        .sha_target_resolving(player, card, target)
    {
        return Ok(Vec::new());
    }
    let mut options = vec![1];
    if !crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    )
    .is_empty()
    {
        options.push(2);
    }
    options.push(3);
    GuardedChoice::new(
        skill,
        Participants::pair(player, target),
        ContinuationKind::Custom,
        vec![
            TWMOUKUI_CUSTOM_CHOICE,
            player.raw(),
            target.raw(),
            card.raw(),
            skill.raw(),
        ],
    )
    .shape(ChoiceShape::range(0, 1, true))
    .open_tokens(tx, options, ChoiceKind::Custom)
}

pub(crate) fn resolve_twfuman(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cards: Vec<CardId>,
    action: u8,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if stage != 0 || action > 2 {
        return Err(EffectError::ContentFault(
            "unknown TW Fuman resolver stage".into(),
        ));
    }
    if action != 0 {
        if !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == player)
        {
            return Ok(Vec::new());
        }
        tx.state_mut()
            .skills
            .get_mut(skill)
            .unwrap()
            .take_tracked_cards(&cards);
        return Ok(Vec::new());
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|participant| participant.alive)
        })
        || cards.len() != 1
    {
        return Err(EffectError::Illegal("invalid TW Fuman gift".into()));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("TW Fuman owner has no hand".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("TW Fuman target has no hand".into()))?;
    let card = cards[0];
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.owner == player && state.zone == from)
    {
        return Err(EffectError::Illegal(
            "TW Fuman card left the owner's hand".into(),
        ));
    }
    record_active_target(tx, player, target, skill)?;
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| instance.add_tracked_cards(&cards))
    {
        return Err(EffectError::ContentFault(
            "TW Fuman tracked-card state is occupied".into(),
        ));
    }
    give_cards_as(tx, player, cards, to, from)
}

pub(crate) fn resolve_cuijin(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state().skills.get(skill) else {
        return Ok(Vec::new());
    };
    if instance.owner != player || action > 1 {
        return Err(EffectError::ContentFault(
            "invalid Cuijin resolver context".into(),
        ));
    }
    if action == 0 {
        if !originating_skill_is_enabled(tx, player, skill)
            || !tx
                .state()
                .resolution
                .sha_resolution_records
                .iter()
                .any(|record| record.source == source && record.card == card)
        {
            return Ok(Vec::new());
        }
        tx.state_mut()
            .skills
            .get_mut(skill)
            .unwrap()
            .set_source_free_physical_card(card);
    } else if instance.source_free_physical_card() == Some(card) {
        tx.state_mut()
            .skills
            .get_mut(skill)
            .unwrap()
            .take_source_free_physical_card();
    }
    Ok(Vec::new())
}

pub(crate) fn resolve_twqingkou(
    tx: &mut Transaction,
    player: PlayerId,
    winner: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let Some(instance) = tx.state().skills.get(skill) else {
        return Ok(Vec::new());
    };
    if instance.owner != player || action > 1 {
        return Err(EffectError::ContentFault(
            "invalid TW Qingkou resolver context".into(),
        ));
    }
    if action == 0 {
        return choose_use_virtual_card_impl(
            tx,
            player,
            CardKind::TrickJuedou.tag(),
            skill,
            VirtualCardTracking::SourceFree,
            true,
        );
    }
    if instance.source_free_physical_card() != Some(card)
        || !tx
            .state()
            .players
            .get(winner)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .take_source_free_physical_card();
    if winner == player {
        if !tx.state().pending_judge_skips.contains(&player) {
            tx.state_mut().pending_judge_skips.push(player);
            tx.state_mut()
                .pending_judge_skips
                .sort_by_key(|participant| participant.raw());
        }
        tx.state_mut().turn.set_flag(turn_flag::SKIP_DISCARD, 1);
    }
    draw_cards(tx, winner, 1)
}

pub(crate) fn twchenxun_targets(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Vec<PlayerId> {
    let used = tx
        .state()
        .skills
        .get(skill)
        .map_or_else(Vec::new, |instance| instance.twchenxun_targets());
    tx.state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|target| {
            !used.contains(target)
                && crate::command::play::validate_source_free_card_on_target(
                    tx,
                    player,
                    CardKind::TrickJuedou,
                    *target,
                )
                .is_ok()
        })
        .collect()
}

pub(crate) fn open_twchenxun_target(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let targets = twchenxun_targets(tx, player, skill);
    if targets.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        TWCHENXUN_CUSTOM_CHOICE,
        player.raw(),
        skill.raw(),
        targets.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(targets.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        0,
        1,
        true,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::Custom,
        payload,
    )
}
