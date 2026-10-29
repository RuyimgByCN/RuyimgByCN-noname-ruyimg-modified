//! Effect primitives for `resolve_jiaohua` and related helpers.

use super::*;

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

pub(crate) fn choose_give_or_recast_one_hand(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    require_skill_def: u32,
    grant_skill_def: u32,
    grant_mark_def: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    let allowed = originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive && !participant.dying);
    let cards: Vec<u32> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    if !allowed || cards.is_empty() {
        return Ok(Vec::new());
    }
    let require_skill_def = tx.state().skills.storage_get(skill, 2).max(require_skill_def);
    let grant_skill_def = tx.state().skills.storage_get(skill, 3).max(grant_skill_def);
    let grant_mark_def = tx.state().skills.storage_get(skill, 4).max(grant_mark_def);
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        cards,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseGiveOrRecastOneHand,
        vec![
            player.raw(),
            skill.raw(),
            0,
            require_skill_def,
            grant_skill_def,
            grant_mark_def,
        ],
    )
}
