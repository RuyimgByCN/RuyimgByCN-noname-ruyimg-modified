//! Effect primitives for `reveal_uniform_hand_draw_self_choose_others` and related helpers.

use super::*;

pub(crate) fn reveal_uniform_hand_draw_self_choose_others(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.phase == Phase::Idle
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: cards.clone(),
        to: Vec::new(),
    }];
    let mut color = None;
    for card in &cards {
        let current = crate::query::effective_card_color_mask(tx.state(), player, *card);
        if !matches!(current, 0b01 | 0b10) || color.is_some_and(|color| color != current) {
            return Ok(events);
        }
        color = Some(current);
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .collect();
    let max = cards.len().min(options.len()).min(usize::from(u8::MAX)) as u8;
    if max == 0 {
        events.extend(draw_cards(tx, player, 1)?);
        return Ok(events);
    }
    events.extend(open_choice(
        tx,
        player,
        1,
        max,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOthersAfterUniformHandReveal,
        vec![
            player.raw(),
            u32::from(max),
            skill.raw(),
            tx.state().turn.current.raw(),
            u32::from(tx.state().turn.phase.to_u8()),
        ],
    )?);
    Ok(events)
}

pub(crate) fn reveal_uniform_hand_choose_damage(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
        || !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.phase == Phase::Idle
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let mut events = vec![DomainEvent::CardsRevealed {
        cards: cards.clone(),
        to: Vec::new(),
    }];
    let mut color = None;
    for card in cards {
        let current = crate::query::effective_card_color_mask(tx.state(), player, card);
        if !matches!(current, 0b01 | 0b10) || color.is_some_and(|color| color != current) {
            return Ok(events);
        }
        color = Some(current);
    }
    let options = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
        .collect();
    events.extend(open_choice(
        tx,
        player,
        1,
        1,
        false,
        options,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherDamage,
        vec![player.raw(), amount, skill.raw()],
    )?);
    Ok(events)
}

pub(crate) fn choose_card_user_or_add_target(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.play_phase_fire_count > 0)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            target != player
                && crate::command::play::saved_card_extra_target_is_legal(tx, player, card, target)
        })
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
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseCardUserOrAddTarget,
        vec![player.raw(), card.raw(), skill.raw()],
    )
}

pub(crate) fn discardable_equipment_cards(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state().cards.get(*card).is_some_and(|card_state| {
                crate::query::effective_card_kind(tx.state(), player, card_state).is_equip()
            })
        })
        .collect()
}

pub(crate) fn choose_discard_equipment_current_damage(
    tx: &mut Transaction,
    player: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
    prevent: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if damage_id == 0 || tx.state().resolution.current_damage_id != damage_id {
        return Err(EffectError::ContentFault(
            "equipment damage reduction used outside its damage_before event".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let cards = discardable_equipment_cards(tx, player);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardEquipmentReduceCurrentDamage,
        vec![player.raw(), damage_id, skill.raw(), u32::from(prevent)],
    )
}

pub(crate) fn damage_endpoint_discard_cards(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    let mut cards: Vec<_> = [player, target]
        .into_iter()
        .flat_map(|endpoint| {
            crate::query::discardable_target_cards(
                tx.state(),
                player,
                endpoint,
                TargetCardScope::HandEquip,
            )
        })
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn choose_discard_damage_endpoint_card_increase_if_suit(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    suit: u8,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !(1..=4).contains(&suit) || amount == 0 {
        return Err(EffectError::ContentFault(
            "damage-endpoint discard requires a real suit and positive amount".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || !bound_sha_damage_matches_any_segment(tx, damage_id, player, target)
    {
        return Err(EffectError::Illegal(
            "damage-endpoint discard context changed".into(),
        ));
    }
    let options = damage_endpoint_discard_cards(tx, player, target);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        player.raw(),
        target.raw(),
        damage_id,
        u32::from(suit),
        amount,
        skill.raw(),
        options.len() as u32,
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
        ContinuationKind::ChooseDiscardDamageEndpointCardIncreaseIfSuit,
        payload,
    )
}

pub(crate) fn choose_move_target_equipment_or_draw(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_move_target_equipment_draw_options(tx, player, target, skill, 0)
}

pub(crate) fn choose_move_target_equipment_and_or_draw(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_move_target_equipment_draw_options(tx, player, target, skill, 3)
}

pub(crate) fn choose_draw_view_two_gain_or_move_equipment(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_move_target_equipment_draw_options(tx, player, target, skill, 4)
}

pub(crate) fn choose_move_target_equipment_draw_options(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    stage: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let mut options = vec![1]; // draw one
    if stage == 4 && !tx.state().hand_cards(target).is_empty() {
        options.push(2); // privately view up to two hand cards, then gain one
    }
    if !crate::query::movable_target_equipment_cards(tx.state(), player, target).is_empty() {
        options.insert(0, 0); // move one equipment card
    }
    open_choice(
        tx,
        player,
        1,
        if stage == 3 { options.len() as u8 } else { 1 },
        true,
        options,
        ChoiceKind::Custom,
        ContinuationKind::ChooseMoveTargetEquipmentOrDraw,
        vec![stage, player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn give_all_equipment_choose_recover_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "equipment gift requires distinct living participants".into(),
        ));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Equip, player)
        .ok_or_else(|| EffectError::Illegal("missing source equipment zone".into()))?;
    let cards = tx
        .state()
        .zones
        .get(from)
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "equipment gift requires at least one equipped card".into(),
        ));
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("missing recipient hand zone".into()))?;
    let count = cards.len().min(u32::MAX as usize) as u32;
    super::super::run_effects(
        tx,
        vec![
            EffectRequest::GiveCards { cards, to, from },
            EffectRequest::ChooseRecoverOwnerOrDamageByGivenEquipment {
                owner: player,
                recipient: target,
                count,
                skill,
            },
        ],
    )
    .map(|_| Vec::new())
}

pub(crate) fn choose_recover_owner_or_damage_by_given_equipment(
    tx: &mut Transaction,
    owner: PlayerId,
    recipient: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == owner)
        || !tx
            .state()
            .players
            .get(recipient)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| crate::query::player_in_attack_range(tx.state(), recipient, target))
        .collect();
    if options.is_empty() {
        if tx
            .state()
            .players
            .get(owner)
            .is_some_and(|participant| participant.alive)
        {
            return recover_hp(tx, recipient, owner, count);
        }
        return Ok(Vec::new());
    }
    let max = count.min(options.len() as u32).min(u32::from(u8::MAX)) as u8;
    open_choice(
        tx,
        recipient,
        1,
        max,
        true,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseRecoverOwnerOrDamageByGivenEquipment,
        vec![owner.raw(), recipient.raw(), count, skill.raw()],
    )
}

pub(crate) fn physical_equipped_weapons(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    tx.state()
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| tx.state().zones.get(zone))
        .map(|zone| {
            zone.cards
                .iter()
                .copied()
                .filter(|card| {
                    tx.state()
                        .cards
                        .get(*card)
                        .is_some_and(|card| card.kind == CardKind::Weapon)
                })
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn physical_weapon_range(tx: &Transaction, player: PlayerId) -> Option<u8> {
    physical_equipped_weapons(tx, player)
        .into_iter()
        .find_map(|card| {
            tx.state()
                .cards
                .get(card)
                .map(|card| card.weapon_range.max(1))
        })
}

pub(crate) fn bound_sha_damage_matches(
    tx: &Transaction,
    damage_id: u32,
    player: PlayerId,
    target: PlayerId,
) -> bool {
    bound_sha_damage_matches_inner(tx, damage_id, player, target, true)
}

pub(crate) fn bound_sha_damage_matches_any_segment(
    tx: &Transaction,
    damage_id: u32,
    player: PlayerId,
    target: PlayerId,
) -> bool {
    bound_sha_damage_matches_inner(tx, damage_id, player, target, false)
}

pub(crate) fn bound_sha_damage_matches_inner(
    tx: &Transaction,
    damage_id: u32,
    player: PlayerId,
    target: PlayerId,
    direct_only: bool,
) -> bool {
    damage_id != 0
        && tx.state().resolution.current_damage_id == damage_id
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
                        card_kind,
                        card_flags,
                        ..
                    } if *saved == damage_id
                        && *source == player
                        && *saved_target == target
                        && CardKind::from_tag(*card_kind).is_some_and(CardKind::is_sha)
                        && (!direct_only
                            || *card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED == 0)
                )
            })
}

pub(crate) fn bound_damage_matches(
    tx: &Transaction,
    damage_id: u32,
    player: PlayerId,
    target: PlayerId,
) -> bool {
    damage_id != 0
        && tx.state().resolution.current_damage_id == damage_id
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
                        ..
                    } if *saved == damage_id && *source == player && *saved_target == target
                ) || matches!(
                    request,
                    EffectRequest::CommitDamageBanActiveIfDying {
                        damage_id: saved,
                        source,
                        target: saved_target,
                        ..
                    } if *saved == damage_id && *source == player && *saved_target == target
                )
            })
}

pub(crate) fn distance_hand_discard_count(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Option<usize> {
    match crate::query::query(
        tx.state(),
        &rule_api::QueryRequest::Distance {
            from: player,
            to: target,
        },
        &mut crate::query::QueryContext::new(),
    ) {
        rule_api::QueryResponse::Int(distance) => {
            Some(4_i32.saturating_sub(distance).max(1) as usize)
        }
        _ => None,
    }
}

pub(crate) fn open_target_discard_hand_or_increase_damage(
    tx: &mut Transaction,
    mode: u32,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    required: usize,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let required = u8::try_from(required)
        .map_err(|_| EffectError::Illegal("damage discard count exceeds choice limit".into()))?;
    let options = tx.state().hand_cards(target);
    if options.len() < usize::from(required) {
        increase_bound_damage_by_one(tx, damage_id)?;
        return Ok(Vec::new());
    }
    let mut payload = vec![
        mode,
        player.raw(),
        target.raw(),
        damage_id,
        skill.raw(),
        u32::from(required),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        target,
        required,
        required,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTargetDiscardWeaponRangeHandOrIncreaseDamage,
        payload,
    )
}

pub(crate) fn choose_target_discard_weapon_range_hand_or_increase_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !bound_sha_damage_matches(tx, damage_id, player, target)
    {
        return Err(EffectError::Illegal(
            "weapon-range damage choice context changed".into(),
        ));
    }
    let required = physical_weapon_range(tx, player).ok_or_else(|| {
        EffectError::Illegal("weapon-range damage choice requires a physical weapon".into())
    })?;
    open_target_discard_hand_or_increase_damage(
        tx,
        DAMAGE_DISCARD_WEAPON_RANGE,
        player,
        target,
        damage_id,
        usize::from(required),
        skill,
    )
}

pub(crate) fn choose_target_discard_distance_hand_or_increase_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !bound_damage_matches(tx, damage_id, player, target)
    {
        return Err(EffectError::Illegal(
            "distance damage choice context changed".into(),
        ));
    }
    let required = distance_hand_discard_count(tx, player, target)
        .ok_or_else(|| EffectError::Illegal("damage distance is unavailable".into()))?;
    open_target_discard_hand_or_increase_damage(
        tx,
        DAMAGE_DISCARD_DISTANCE,
        player,
        target,
        damage_id,
        required,
        skill,
    )
}

pub(crate) fn current_attack_range(tx: &Transaction, player: PlayerId) -> usize {
    match crate::query::query(
        tx.state(),
        &rule_api::QueryRequest::AttackRange { player },
        &mut crate::query::QueryContext::new(),
    ) {
        rule_api::QueryResponse::Int(range) => range.max(0) as usize,
        _ => 0,
    }
}

pub(crate) fn open_attack_range_discard_choice(
    tx: &mut Transaction,
    stage: u32,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
    attack_range: usize,
    options: Vec<CardId>,
) -> Result<Vec<DomainEvent>, EffectError> {
    // ponytail: ChoiceRequest counts are u8; split into batches if decks can exceed 255 cards.
    let required = attack_range.min(options.len()).min(usize::from(u8::MAX)) as u8;
    if required == 0 {
        return Ok(Vec::new());
    }
    let mut payload = vec![
        stage,
        player.raw(),
        target.raw(),
        damage_id,
        skill.raw(),
        attack_range.min(u32::MAX as usize) as u32,
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        target,
        required,
        required,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth,
        payload,
    )
}

pub(crate) fn choose_target_discard_attack_range_cards_or_increase_damage_weapon_forces_both(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    damage_id: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !bound_sha_damage_matches(tx, damage_id, player, target)
    {
        return Err(EffectError::Illegal(
            "attack-range damage choice context changed".into(),
        ));
    }
    let attack_range = current_attack_range(tx, player);
    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
    let weapons = physical_equipped_weapons(tx, player);
    if !weapons.is_empty() {
        if options.is_empty() || attack_range == 0 {
            return super::super::run_effects(
                tx,
                vec![
                    EffectRequest::ContinueIncreaseBoundDamage {
                        player,
                        damage_id,
                        amount: 1,
                        skill,
                    },
                    EffectRequest::DiscardCards {
                        player,
                        cards: weapons,
                    },
                ],
            );
        }
        return open_attack_range_discard_choice(
            tx,
            2,
            player,
            target,
            damage_id,
            skill,
            attack_range,
            options,
        );
    }
    if options.is_empty() || attack_range == 0 {
        increase_bound_damage_by_one(tx, damage_id)?;
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseTargetDiscardAttackRangeCardsOrIncreaseDamageWeaponForcesBoth,
        vec![
            0,
            player.raw(),
            target.raw(),
            damage_id,
            skill.raw(),
            attack_range.min(u32::MAX as usize) as u32,
        ],
    )
}

pub(crate) fn choose_hand_count_different_target_draw_or_random_discard(
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
        return Err(EffectError::Illegal(
            "hand-count target choice context changed".into(),
        ));
    }
    let hand_count = tx.state().hand_count(player);
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| tx.state().hand_count(target) != hand_count)
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![player.raw(), skill.raw(), options.len() as u32];
    payload.extend(options.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseHandCountDifferentTargetDrawOrRandomDiscard,
        payload,
    )
}

pub(crate) fn rezhenjun_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Prepare
}

pub(crate) fn choose_target_discard_excess_then_self_discard_or_target_draw(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !rezhenjun_context_is_current(tx, player, skill) {
        return Err(EffectError::Illegal(
            "prepare-phase target discard context changed".into(),
        ));
    }
    let options: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| {
            !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty()
        })
        .collect();
    if options.is_empty() {
        return Ok(Vec::new());
    }
    let mut payload = vec![0, player.raw(), skill.raw(), options.len() as u32];
    payload.extend(options.iter().map(|target| target.raw()));
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        options.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
        payload,
    )
}

pub(crate) fn choose_self_discard_or_target_draw_after_target_discard(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if !rezhenjun_context_is_current(tx, player, skill)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "prepare-phase self-discard branch context changed".into(),
        ));
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let required = usize::try_from(count).unwrap_or(usize::MAX);
    if options.len() < required || count > u32::from(u8::MAX) {
        return super::super::run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player: target,
                count,
                skill,
            }],
        );
    }
    let mut payload = vec![
        2,
        player.raw(),
        target.raw(),
        skill.raw(),
        count,
        options.len() as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        count as u8,
        count as u8,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
        payload,
    )
}

pub(crate) fn decadezhenjun_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == player
        && matches!(tx.state().turn.phase, Phase::Prepare | Phase::End)
}

pub(crate) fn resolve_decadezhenjun(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !decadezhenjun_context_is_current(tx, player, skill) {
                return Err(EffectError::Illegal(
                    "Decade Zhenjun phase context changed".into(),
                ));
            }
            let options: Vec<_> = tx
                .state()
                .players
                .alive_by_seat()
                .into_iter()
                .filter(|&target| {
                    !crate::query::discardable_target_cards(
                        tx.state(),
                        player,
                        target,
                        rule_api::TargetCardScope::HandEquip,
                    )
                    .is_empty()
                })
                .collect();
            if options.is_empty() {
                return Ok(Vec::new());
            }
            let mut payload = vec![3, player.raw(), skill.raw(), options.len() as u32];
            payload.extend(options.iter().map(|target| target.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(PlayerId::raw).collect(),
                ChoiceKind::SelectTargets,
                ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
                payload,
            )
        }
        1 => {
            if count == 0
                || !decadezhenjun_context_is_current(tx, player, skill)
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return super::super::run_effects(
                    tx,
                    vec![EffectRequest::DrawCardsFromSkill {
                        player: target,
                        count,
                        skill,
                    }],
                );
            }
            let mut payload = vec![
                5,
                player.raw(),
                target.raw(),
                skill.raw(),
                count,
                options.len() as u32,
            ];
            payload.extend(options.iter().map(|card| card.raw()));
            open_choice(
                tx,
                player,
                1,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw,
                payload,
            )
        }
        _ => Err(EffectError::Illegal(
            "invalid Decade Zhenjun internal stage".into(),
        )),
    }
}

pub(crate) fn sp_xiaoguo_initial_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && player != target
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        && tx.state().turn.current == target
        && tx.state().turn.phase == Phase::End
}

pub(crate) fn sp_xiaoguo_participants_alive(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> bool {
    tx.state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
        && tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
}

pub(crate) fn resolve_sp_xiaoguo(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    stage: u8,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !sp_xiaoguo_initial_context_is_current(tx, player, target, skill) {
                return Err(EffectError::Illegal(
                    "SP Xiaoguo phase context changed".into(),
                ));
            }
            let options = crate::query::sp_xiaoguo_hand_cards(tx.state(), player);
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
                ContinuationKind::ResolveSpXiaoguo,
                vec![0, player.raw(), target.raw(), skill.raw()],
            )
        }
        1 => {
            if !sp_xiaoguo_participants_alive(tx, player, target) {
                return Ok(Vec::new());
            }
            let options = crate::query::sp_xiaoguo_equipment_cards(tx.state(), target);
            if options.is_empty() {
                return super::super::run_effects(
                    tx,
                    vec![EffectRequest::ResolveSpXiaoguo {
                        player,
                        target,
                        stage: 2,
                        skill,
                    }],
                );
            }
            open_choice(
                tx,
                target,
                0,
                1,
                true,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ResolveSpXiaoguo,
                vec![2, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            if !sp_xiaoguo_participants_alive(tx, player, target) {
                return Ok(Vec::new());
            }
            super::super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: player,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
        }
        3 => {
            if !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
            {
                return Ok(Vec::new());
            }
            super::super::run_effects(
                tx,
                vec![EffectRequest::DrawCardsFromSkill {
                    player,
                    count: 1,
                    skill,
                }],
            )
        }
        _ => Err(EffectError::Illegal(
            "invalid SP Xiaoguo internal stage".into(),
        )),
    }
}

pub(crate) fn eu_tuxi_context_is_current(
    tx: &Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> bool {
    originating_skill_is_enabled(tx, player, skill)
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
}

/// Reproduce legacy `trigger.player`, `getPrevious()`, `getNext()` after the draw barrier.
/// Previous and next intentionally remain duplicated in a two-player game.
pub(crate) fn damage_target_and_legacy_neighbors(
    tx: &Transaction,
    target: PlayerId,
) -> Option<Vec<PlayerId>> {
    let target_seat = tx.state().players.get(target)?.seat.raw();
    let alive = tx.state().players.alive_by_seat();
    let mut targets = vec![target];
    if alive.len() <= 1 {
        return Some(targets);
    }

    if let Some(position) = alive.iter().position(|&player| player == target) {
        targets.push(alive[(position + alive.len() - 1) % alive.len()]);
        targets.push(alive[(position + 1) % alive.len()]);
        return Some(targets);
    }

    let previous = alive
        .iter()
        .rev()
        .copied()
        .find(|player| {
            tx.state()
                .players
                .get(*player)
                .is_some_and(|participant| participant.seat.raw() < target_seat)
        })
        .or_else(|| alive.last().copied());
    let next = alive
        .iter()
        .copied()
        .find(|player| {
            tx.state()
                .players
                .get(*player)
                .is_some_and(|participant| participant.seat.raw() > target_seat)
        })
        .or_else(|| alive.first().copied());
    targets.extend(previous);
    targets.extend(next);
    Some(targets)
}

pub(crate) fn eu_tuxi_gain_options(
    tx: &Transaction,
    player: PlayerId,
    target: PlayerId,
) -> Vec<CardId> {
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Vec::new();
    }
    let hand = (target != player)
        .then(|| tx.state().zones.find(ZoneKind::Hand, target))
        .flatten();
    let equip = tx.state().zones.find(ZoneKind::Equip, target);
    let mut options: Vec<_> = [hand, equip]
        .into_iter()
        .flatten()
        .filter_map(|zone| tx.state().zones.get(zone))
        .flat_map(|zone| zone.cards.iter().copied())
        .collect();
    options.sort_by_key(|card| card.raw());
    options.dedup();
    options
}

pub(crate) fn draw_then_choose_gain_one_from_damage_target_and_neighbors(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !eu_tuxi_context_is_current(tx, player, skill) {
        return if ready_to_choose {
            Ok(Vec::new())
        } else {
            Err(EffectError::Illegal(
                "damage-source sequential gain context changed".into(),
            ))
        };
    }
    if tx.state().players.get(target).is_none() {
        return if ready_to_choose {
            Ok(Vec::new())
        } else {
            Err(EffectError::Illegal("missing damaged player".into()))
        };
    }
    if !ready_to_choose {
        let events = draw_cards(tx, player, 1)?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DrawThenChooseGainOneFromDamageTargetAndNeighbors {
                player,
                target,
                skill,
                ready_to_choose: true,
            },
        );
        return Ok(events);
    }

    let Some(targets) = damage_target_and_legacy_neighbors(tx, target) else {
        return Ok(Vec::new());
    };
    continue_choose_gain_one_from_damage_target_and_neighbors(
        tx,
        player,
        targets,
        0,
        CardId::NONE,
        skill,
    )
}

pub(crate) fn continue_choose_gain_one_from_damage_target_and_neighbors(
    tx: &mut Transaction,
    player: PlayerId,
    targets: Vec<PlayerId>,
    mut index: u32,
    selected: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !eu_tuxi_context_is_current(tx, player, skill) {
        return Ok(Vec::new());
    }

    if selected.is_some() {
        let current = targets
            .get(index as usize)
            .copied()
            .ok_or_else(|| EffectError::Illegal("sequential gain index changed".into()))?;
        if !eu_tuxi_gain_options(tx, player, current).contains(&selected) {
            return Err(EffectError::Illegal(
                "sequential gain selection is invalid or stale".into(),
            ));
        }
        let from = tx
            .state()
            .cards
            .get(selected)
            .map(|card| card.zone)
            .ok_or_else(|| EffectError::Illegal("missing selected gain card".into()))?;
        let to = tx
            .state()
            .zones
            .find(ZoneKind::Hand, player)
            .ok_or_else(|| EffectError::Illegal("missing skill owner hand".into()))?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ContinueChooseGainOneFromDamageTargetAndNeighbors {
                player,
                targets,
                index: index.saturating_add(1),
                selected: CardId::NONE,
                skill,
            },
        );
        return move_cards(tx, vec![selected], to, Some(from));
    }

    while let Some(current) = targets.get(index as usize).copied() {
        let options = eu_tuxi_gain_options(tx, player, current);
        if options.is_empty() {
            index = index.saturating_add(1);
            continue;
        }
        let mut payload = vec![player.raw(), skill.raw(), index, targets.len() as u32];
        payload.extend(targets.iter().map(|target| target.raw()));
        payload.push(options.len() as u32);
        payload.extend(options.iter().map(|card| card.raw()));
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.into_iter().map(CardId::raw).collect(),
            ChoiceKind::SelectCards,
            ContinuationKind::ChooseGainOneFromDamageTargetAndNeighbors,
            payload,
        );
    }
    Ok(Vec::new())
}

pub(crate) fn modify_current_damage(
    tx: &mut Transaction,
    player: PlayerId,
    delta: i32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if delta == 0 {
        return Ok(Vec::new());
    }
    let damage_id = tx.state().resolution.current_damage_id;
    if damage_id == 0 {
        return Err(EffectError::ContentFault(
            "current damage modification used outside its damage_before event".into(),
        ));
    }
    if delta > 0 {
        increase_bound_damage(tx, damage_id, delta as u32)?;
    } else {
        for _ in 0..(-delta) {
            reduce_bound_damage_by_one(tx, damage_id)?;
        }
    }
    let _ = player;
    Ok(Vec::new())
}
