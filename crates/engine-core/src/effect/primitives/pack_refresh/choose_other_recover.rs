//! Effect primitives for `refresh_skill` and related helpers.

use super::*;

pub(crate) fn choose_other_recover(
    tx: &mut Transaction,
    player: PlayerId,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    if amount == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .map(PlayerId::raw)
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
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherDrawOrRecover,
        vec![2, player.raw(), amount],
    )
}

pub(crate) fn choose_discard_hand_equip_then_remove_self_skill(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal(
            "dynamic discard requires the owner's enabled originating skill".into(),
        ));
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let required = usize::try_from(count)
        .unwrap_or(usize::MAX)
        .min(options.len());
    if required == 0 {
        return remove_skill(tx, skill);
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::RemoveSkill { skill });
    if required == options.len() {
        return discard_cards(tx, player, options);
    }
    let required = u8::try_from(required)
        .map_err(|_| EffectError::ContentFault("dynamic discard count exceeds 255".into()))?;
    let mut payload = vec![
        player.raw(),
        skill.raw(),
        u32::from(required),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        required,
        required,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill,
        payload,
    )
}

pub(crate) fn choose_discard_hand_equip_then_recover(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 || !originating_skill_is_enabled(tx, player, skill) {
        return Err(EffectError::Illegal(
            "discard-and-recover requires a positive count and the owner's enabled skill".into(),
        ));
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    let required = usize::try_from(count).unwrap_or(usize::MAX);
    if options.len() < required {
        return Ok(Vec::new());
    }
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::RecoverHp {
            source: player,
            player,
            amount,
        });
    if required == options.len() {
        return discard_cards(tx, player, options);
    }
    let required = u8::try_from(required)
        .map_err(|_| EffectError::ContentFault("discard-and-recover count exceeds 255".into()))?;
    let mut payload = vec![
        player.raw(),
        skill.raw(),
        u32::from(required),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    open_choice(
        tx,
        player,
        required,
        required,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill,
        payload,
    )
}

pub(crate) fn choose_discard_hand_equip_then_recover_target(
    tx: &mut Transaction,
    chooser: PlayerId,
    target: PlayerId,
    count: u32,
    amount: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 || amount == 0 || !originating_skill_is_enabled(tx, chooser, skill) {
        return Err(EffectError::Illegal(
            "optional discard-and-recover requires positive values and the chooser's enabled skill"
                .into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|player| player.alive && player.hp < player.max_hp as i32)
    {
        return Ok(Vec::new());
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), chooser);
    let required = usize::try_from(count).unwrap_or(usize::MAX);
    if options.len() < required {
        return Ok(Vec::new());
    }
    let required = u8::try_from(required).map_err(|_| {
        EffectError::ContentFault("optional discard-and-recover count exceeds 255".into())
    })?;
    let mut payload = vec![
        chooser.raw(),
        skill.raw(),
        u32::from(required),
        options.len().min(u32::MAX as usize) as u32,
    ];
    payload.extend(options.iter().map(|card| card.raw()));
    payload.extend([target.raw(), amount]);
    open_choice(
        tx,
        chooser,
        required,
        required,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill,
        payload,
    )
}

pub(crate) fn choose_target_draw(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    scope: DrawTargetScope,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    choose_targets_draw(tx, player, count, 1, scope, skill)
}

pub(crate) fn choose_target_draw_to_max_hp_cap(
    tx: &mut Transaction,
    player: PlayerId,
    cap: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cap == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(PlayerId::raw)
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
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetDraw,
        vec![player.raw(), cap, skill.raw()],
    )
}

pub(crate) fn choose_target_draw_then_owner_if_below_max_hp(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
    ready_to_check: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0 {
        return Err(EffectError::ContentFault(
            "target draw count must be positive".into(),
        ));
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
    if target.is_none() {
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(PlayerId::raw)
            .collect();
        if targets.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            ChoiceKind::SelectTargets,
            ContinuationKind::ChooseTargetDrawThenOwnerIfBelowMaxHp,
            vec![player.raw(), count, skill.raw()],
        );
    }
    let target_exists = tx.state().players.get(target).is_some();
    let target_alive = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive);
    if !target_exists || (!ready_to_check && !target_alive) {
        return if ready_to_check {
            Ok(Vec::new())
        } else {
            Err(EffectError::Illegal(
                "selected draw target is no longer alive".into(),
            ))
        };
    }
    if !ready_to_check {
        let events = draw_cards(tx, target, count)?;
        tx.state_mut().resolution.deferred_effects.insert(
            0,
            EffectRequest::ChooseTargetDrawThenOwnerIfBelowMaxHp {
                player,
                target,
                count,
                skill,
                ready_to_check: true,
            },
        );
        return Ok(events);
    }
    let below_max_hp = tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| tx.state().hand_count(target) < participant.max_hp);
    if below_max_hp {
        draw_cards(tx, player, 1)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn choose_target_draw_max_hp_cap_then_discard_excess(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cap: u32,
    skill: engine_api::SkillId,
    ready_to_discard: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cap == 0 {
        return Err(EffectError::ContentFault(
            "maximum-HP draw/discard cap must be positive".into(),
        ));
    }
    if !originating_skill_is_enabled(tx, player, skill) || tx.state().players.get(player).is_none()
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| {
                tx.state()
                    .players
                    .get(*target)
                    .is_some_and(|participant| participant.max_hp > 0)
            })
            .map(PlayerId::raw)
            .collect();
        if targets.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            ChoiceKind::SelectTargets,
            ContinuationKind::ChooseTargetDrawMaxHpCapThenDiscardExcess,
            vec![player.raw(), cap, skill.raw()],
        );
    }
    let participant = tx.state().players.get(target);
    let target_alive_with_positive_max_hp =
        participant.is_some_and(|participant| participant.alive && participant.max_hp > 0);
    if !ready_to_discard && !target_alive_with_positive_max_hp {
        return Err(EffectError::Illegal(
            "selected capped draw target is no longer legal".into(),
        ));
    }
    if ready_to_discard {
        let Some(participant) = participant.filter(|participant| participant.alive) else {
            return Ok(Vec::new());
        };
        let limit = participant.max_hp.min(cap);
        if tx.state().hand_count(target) > limit {
            return adjust_hand_to(tx, target, limit);
        }
        return Ok(Vec::new());
    }
    let count = participant
        .expect("validated capped draw target")
        .max_hp
        .min(cap);
    let events = draw_cards(tx, target, count)?;
    tx.state_mut().resolution.deferred_effects.insert(
        0,
        EffectRequest::ChooseTargetDrawMaxHpCapThenDiscardExcess {
            player,
            target,
            cap,
            skill,
            ready_to_discard: true,
        },
    );
    Ok(events)
}

pub(crate) fn choose_target_draw_max_hp_cap_then_add_skill(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    cap: u32,
    skill: engine_api::SkillId,
    add_def: engine_api::SkillDefId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if cap == 0
        || add_def.raw() == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    if target.is_none() {
        let targets: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|target| {
                tx.state()
                    .players
                    .get(*target)
                    .is_some_and(|target| target.max_hp > 0)
            })
            .map(PlayerId::raw)
            .collect();
        if targets.is_empty() {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets,
            ChoiceKind::SelectTargets,
            ContinuationKind::Custom,
            vec![
                CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL_CUSTOM_CHOICE,
                player.raw(),
                cap,
                add_def.raw(),
            ],
        );
    }
    let Some(participant) = tx
        .state()
        .players
        .get(target)
        .filter(|participant| participant.alive)
    else {
        return Ok(Vec::new());
    };
    let count = participant
        .max_hp
        .min(cap)
        .saturating_sub(tx.state().hand_count(target));
    let mut events = super::super::run_effects(
        tx,
        vec![EffectRequest::DrawCardsFromSkill {
            player: target,
            count,
            skill,
        }],
    )?;
    events.extend(super::super::run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: add_def,
        }],
    )?);
    Ok(events)
}

pub(crate) fn give_selected_then_target_choose_discard_same_suit_or_lose_hp(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    suit: engine_api::Suit,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ready_to_choose {
        if !tx
            .state()
            .players
            .get(target)
            .is_some_and(|state| state.alive)
        {
            return Ok(Vec::new());
        }
        if tx.state().hand_count(target) == 0 {
            return super::super::run_effects(
                tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target,
                    amount: 1,
                    is_lose_hp: true,
                    nature: 0,
                }],
            )
            .map(|_| Vec::new());
        }
        return open_choice(
            tx,
            target,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Custom,
            ContinuationKind::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp,
            vec![target.raw(), crate::command::judge::suit_tag(suit)],
        );
    }
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
    {
        return Err(EffectError::Illegal(
            "selected-card gift requires distinct living participants".into(),
        ));
    }
    let from = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("missing source hand zone".into()))?;
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("missing recipient hand zone".into()))?;
    let captured_suit = tx
        .state()
        .cards
        .get(card)
        .filter(|state| state.owner == player && state.zone == from)
        .map(|state| state.suit)
        .ok_or_else(|| {
            EffectError::Illegal("selected gift card is no longer in the owner's hand".into())
        })?;
    super::super::run_effects(
        tx,
        vec![
            EffectRequest::RevealCards {
                cards: vec![card],
                to: Vec::new(),
            },
            EffectRequest::GiveCards {
                cards: vec![card],
                to,
                from,
            },
            EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player,
                target,
                card,
                suit: captured_suit,
                skill,
                ready_to_choose: true,
            },
        ],
    )
    .map(|_| Vec::new())
}

pub(crate) fn target_choose_suit_gain_owner_hand_damage_if_mismatch(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
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
        || tx.state().hand_count(player) == 0
    {
        return Err(EffectError::Illegal(
            "suit guess requires distinct living participants and an owner hand card".into(),
        ));
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        vec![2, 4, 3, 1],
        ChoiceKind::Custom,
        ContinuationKind::FanjianChooseSuitOrHand,
        vec![0, player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn choose_other_draw_owner_lost_hp_turn_over(
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
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player)
        .map(PlayerId::raw)
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
        targets,
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseOtherDrawOwnerLostHpTurnOver,
        vec![player.raw(), skill.raw()],
    )
}

pub(crate) fn choose_targets_draw(
    tx: &mut Transaction,
    player: PlayerId,
    count: u32,
    target_count: u8,
    scope: DrawTargetScope,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if count == 0
        || target_count == 0
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| player_target_matches(tx.state(), player, target, scope))
        .collect();
    if targets.len() < usize::from(target_count) {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        target_count,
        target_count,
        false,
        targets.into_iter().map(PlayerId::raw).collect(),
        ChoiceKind::SelectTargets,
        ContinuationKind::ChooseTargetDraw,
        if target_count == 1 {
            vec![player.raw(), count, u32::from(scope.tag()), skill.raw()]
        } else {
            vec![
                player.raw(),
                count,
                u32::from(target_count),
                u32::from(scope.tag()),
                skill.raw(),
            ]
        },
    )
}
