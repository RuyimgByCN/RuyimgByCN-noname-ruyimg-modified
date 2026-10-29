//! Effect primitives for `resolve_beizhu` and related helpers.

use super::*;

/// 把主动技实例的 `active_turn_banned` 置位，禁用该主动技直到当前回合结束。
pub(crate) fn ban_active_until_turn_end(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .ok_or_else(|| EffectError::ContentFault("turn-end active ban skill missing".into()))?;
    if instance.owner != player {
        return Err(EffectError::ContentFault(
            "turn-end active ban owner mismatch".into(),
        ));
    }
    instance.active_turn_banned = true;
    Ok(vec![DomainEvent::RuleNote {
        code: 241,
        payload: vec![player.raw(), skill.raw(), 1],
    }])
}

pub(crate) fn force_others_use_sha_on_nearest_or_lose_hp(
    tx: &mut Transaction,
    owner: PlayerId,
    remaining: Vec<PlayerId>,
    started: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let mut remaining = if started {
        remaining
    } else {
        if !originating_skill_is_enabled(tx, owner, skill)
            || !tx
                .state()
                .players
                .get(owner)
                .is_some_and(|player| player.alive)
        {
            return Ok(Vec::new());
        }
        let order = tx.state().players.alive_by_seat();
        let Some(start) = order.iter().position(|&player| player == owner) else {
            return Ok(Vec::new());
        };
        (1..order.len())
            .map(|offset| order[(start + offset) % order.len()])
            .collect()
    };

    while remaining.first().is_some_and(|player| {
        !tx.state()
            .players
            .get(*player)
            .is_some_and(|participant| participant.alive)
    }) {
        remaining.remove(0);
    }
    if remaining.is_empty() {
        return Ok(Vec::new());
    }

    let current = remaining.remove(0);
    let target_limit = crate::query::sha_target_limit_for(tx.state(), current);
    let cards = if target_limit == 0 {
        Vec::new()
    } else {
        crate::command::play::prompted_sha_source_cards(tx.state(), current)
            .into_iter()
            .filter(|card| !crate::command::play::luanwu_sha_targets(tx, current, *card).is_empty())
            .collect()
    };
    if cards.is_empty() {
        crate::effect::continue_luanwu_after_hp_loss(tx, owner, current, skill, remaining)?;
        return Ok(Vec::new());
    }

    let mut payload = vec![
        0,
        owner.raw(),
        current.raw(),
        skill.raw(),
        remaining.len() as u32,
    ];
    payload.extend(remaining.into_iter().map(PlayerId::raw));
    open_choice(
        tx,
        current,
        0,
        1,
        true,
        cards.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::ForceOthersUseShaOnNearestOrLoseHp,
        payload,
    )
}

pub(crate) fn discard_selected_record_unique_category_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    let instance = state
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("missing originating active skill".into()))?;
    let contract = state
        .content
        .first_active_for_skill(instance.def)
        .ok_or_else(|| EffectError::ContentFault("missing originating active contract".into()))?;
    let targets = contract.targets.as_ref();
    let selection = contract.selection.as_ref();
    let turn_scoped = targets.is_some_and(|targets| {
        targets.min == 1
            && targets.max == 1
            && !targets.allow_self
            && targets.unused_by_skill_this_turn
    }) && selection.is_some_and(|selection| {
        selection.min == 1 && selection.max == 1 && selection.unused_category_by_skill_this_turn
    });
    let play_phase_scoped = targets.is_some_and(|targets| {
        targets.min == 1
            && targets.max == 1
            && targets.allow_self
            && targets.unused_by_skill_this_play_phase
    }) && selection.is_some_and(|selection| {
        selection.min == 1
            && selection.max == 1
            && selection.unused_category_by_skill_this_play_phase
    });
    if instance.owner != player
        || !state.skill_effectively_enabled(instance)
        || state.turn.current != player
        || state.turn.phase != Phase::Play
        || turn_scoped == play_phase_scoped
    {
        return Err(EffectError::ContentFault(
            "active uniqueness effect requires its declared contract".into(),
        ));
    }
    if (target == player && !targets.is_some_and(|targets| targets.allow_self))
        || !state
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !crate::query::discardable_hand_equip_cards(state, player).contains(&card)
    {
        return Err(EffectError::Illegal(
            "active uniqueness target or selected card is no longer legal".into(),
        ));
    }
    let category_bit = state
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(state, player, card))
        .map(|kind| {
            if kind.is_basic() {
                1
            } else if kind.is_trick() {
                2
            } else if kind.is_equip() {
                4
            } else {
                0
            }
        })
        .unwrap_or(0);
    let recorded = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| {
            if turn_scoped {
                instance.record_active_unique_use(category_bit, target)
            } else if category_bit != 0
                && instance.play_phase_category_mask & category_bit == 0
                && !instance
                    .active_play_phase_targets()
                    .unwrap_or_default()
                    .contains(&target)
                && instance.record_active_play_phase_target(target)
            {
                instance.play_phase_category_mask |= category_bit;
                true
            } else {
                false
            }
        });
    if !recorded {
        return Err(EffectError::Illegal(
            "active target or logical card category was already used in this uniqueness window"
                .into(),
        ));
    }
    discard_cards(tx, player, vec![card])
}

pub(crate) fn record_selected_play_phase_category(
    tx: &mut Transaction,
    card: CardId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let state = tx.state();
    let instance = state
        .skills
        .get(skill)
        .ok_or_else(|| EffectError::ContentFault("missing originating active skill".into()))?;
    let contract = state
        .content
        .first_active_for_skill(instance.def)
        .ok_or_else(|| EffectError::ContentFault("missing originating active contract".into()))?;
    if !state.skill_effectively_enabled(instance)
        || state.turn.current != instance.owner
        || state.turn.phase != Phase::Play
        || !contract.selection.as_ref().is_some_and(|selection| {
            selection.min == 1
                && selection.max == 1
                && selection.unused_category_by_skill_this_play_phase
        })
    {
        return Err(EffectError::ContentFault(
            "play-phase category effect requires its declared active contract".into(),
        ));
    }
    let owner = instance.owner;
    if !crate::query::discardable_hand_equip_cards(state, owner).contains(&card) {
        return Err(EffectError::Illegal(
            "selected card is no longer legal for play-phase category recording".into(),
        ));
    }
    let category_bit = state
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(state, owner, card))
        .map(|kind| {
            if kind.is_basic() {
                1
            } else if kind.is_trick() {
                2
            } else if kind.is_equip() {
                4
            } else {
                0
            }
        })
        .unwrap_or(0);
    let instance = tx
        .state_mut()
        .skills
        .get_mut(skill)
        .expect("originating active skill validated");
    if category_bit == 0 || instance.play_phase_category_mask & category_bit != 0 {
        return Err(EffectError::Illegal(
            "logical card category was already used in this play phase".into(),
        ));
    }
    instance.play_phase_category_mask |= category_bit;
    Ok(Vec::new())
}

pub(crate) fn choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    toggle_link: bool,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
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
    let count = tx
        .state()
        .skills
        .get(skill)
        .map(|instance| instance.use_count)
        .unwrap_or(0);
    if count == 0 {
        return Err(EffectError::ContentFault(
            "active draw-or-discard effect requires a recorded use".into(),
        ));
    }
    if crate::query::discardable_hand_equip_cards(tx.state(), target).is_empty() {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::FinishLinkIfTargetWounded {
                player,
                target,
                toggle_link,
                skill,
            },
        );
        return draw_cards(tx, target, count);
    }
    open_choice(
        tx,
        player,
        1,
        1,
        false,
        vec![1, 2],
        ChoiceKind::Custom,
        ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded,
        vec![
            0,
            player.raw(),
            target.raw(),
            skill.raw(),
            count,
            u32::from(toggle_link),
        ],
    )
}

pub(crate) fn finish_link_if_target_wounded(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    toggle_link: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let target_wounded = tx.state().players.get(target).is_some_and(|participant| {
        participant.alive && i64::from(participant.hp) < i64::from(participant.max_hp)
    });
    let player_unlinked = tx.state().players.get(player).is_some_and(|participant| {
        participant.alive
            && !participant
                .marks
                .iter()
                .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
    });
    if !target_wounded {
        Ok(Vec::new())
    } else if toggle_link && !player_unlinked {
        remove_mark(tx, player, crate::content::MARK_CHAIN, 0)
    } else if player_unlinked {
        add_mark(tx, player, crate::content::MARK_CHAIN, 1)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn choose_reveal_other_hand_card_remember_category(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|&target| target != player && tx.state().hand_count(target) > 0)
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
        ContinuationKind::ChooseRevealOtherHandCardRememberCategory,
        vec![0, player.raw(), skill.raw()],
    )
}

pub(crate) fn choose_reveal_hand_card_bind_category_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || tx.state().turn.current != target
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Ok(Vec::new());
    }
    let options: Vec<u32> = tx
        .state()
        .hand_cards(target)
        .into_iter()
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
        false,
        options,
        ChoiceKind::SelectCards,
        ContinuationKind::ChooseRevealOtherHandCardRememberCategory,
        vec![2, player.raw(), skill.raw(), target.raw()],
    )
}

pub(crate) fn recover_to_max_then_optional_exchange_skill(
    tx: &mut Transaction,
    player: PlayerId,
    skill: engine_api::SkillId,
    removable_skill_def: engine_api::SkillDefId,
    restore_skill_def: engine_api::SkillDefId,
    ready_to_choose: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if ready_to_choose {
        if restore_skill_def.raw() == 0
            || !tx
                .state()
                .players
                .get(player)
                .is_some_and(|participant| participant.alive)
        {
            return Ok(Vec::new());
        }
        let has_removable = tx.state().players.get(player).is_some_and(|participant| {
            participant.skills.iter().any(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .is_some_and(|instance| instance.def == removable_skill_def)
            })
        });
        if !has_removable {
            return Ok(Vec::new());
        }
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::RecoverToMaxThenOptionalExchangeSkill,
            vec![removable_skill_def.raw(), restore_skill_def.raw()],
        );
    }

    let Some(instance) = tx.state().skills.get(skill).cloned() else {
        return Ok(Vec::new());
    };
    if instance.owner != player || !originating_skill_is_enabled(tx, player, skill) {
        return Ok(Vec::new());
    }
    let Some(participant) = tx
        .state()
        .players
        .get(player)
        .filter(|participant| participant.alive)
    else {
        return Ok(Vec::new());
    };
    let amount = (i64::from(participant.max_hp) - i64::from(participant.hp))
        .max(0)
        .min(i64::from(u32::MAX)) as u32;
    let restore_skill_def = instance.def;
    let mut events = remove_skill(tx, skill)?;
    tx.state_mut().resolution.deferred_tail_effects.push(
        EffectRequest::RecoverToMaxThenOptionalExchangeSkill {
            player,
            skill: engine_api::SkillId::NONE,
            removable_skill_def,
            restore_skill_def,
            ready_to_choose: true,
        },
    );
    events.extend(recover_hp(tx, player, player, amount)?);
    Ok(events)
}

pub(crate) fn draw_then_give_exact_record_phase_target(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    count: u32,
    skill: engine_api::SkillId,
    ready_to_give: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid = originating_skill_is_enabled(tx, player, skill)
        && count > 0
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
        && tx.state().turn.phase == Phase::Play;
    if !valid {
        return Ok(Vec::new());
    }
    if !ready_to_give {
        let events = draw_cards(tx, player, count)?;
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::DrawThenGiveExactRecordPhaseTarget {
                player,
                target,
                count,
                skill,
                ready_to_give: true,
            },
        );
        return Ok(events);
    }
    let mut options: Vec<CardId> = [ZoneKind::Hand, ZoneKind::Equip]
        .into_iter()
        .filter_map(|kind| tx.state().zones.find(kind, player))
        .flat_map(|zone| {
            tx.state()
                .zones
                .get(zone)
                .map(|state| state.cards.clone())
                .unwrap_or_default()
        })
        .collect();
    options.sort_by_key(|card| card.raw());
    options.dedup();
    if options.len() < count as usize || count > u8::MAX as u32 {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        player,
        count as u8,
        count as u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::GiveExactRecordPhaseTarget,
        vec![player.raw(), target.raw(), skill.raw(), count],
    )
}

pub(crate) fn hand_cards_sorted(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let mut cards = tx.state().hand_cards(player);
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn discardable_hand_cards_sorted(tx: &Transaction, player: PlayerId) -> Vec<CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let mut cards: Vec<_> = crate::query::discardable_hand_equip_cards(tx.state(), player)
        .into_iter()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| Some(state.zone) == hand)
        })
        .collect();
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn standard_ganglie_discardable_hand_cards(
    tx: &Transaction,
    player: PlayerId,
) -> Vec<CardId> {
    discardable_hand_cards_sorted(tx, player)
}

pub(crate) fn ganglie_discardable_cards(
    tx: &Transaction,
    player: PlayerId,
    hand_only: bool,
) -> Vec<CardId> {
    if hand_only {
        return standard_ganglie_discardable_hand_cards(tx, player);
    }
    let mut cards = crate::query::discardable_hand_equip_cards(tx.state(), player);
    cards.sort_by_key(|card| card.raw());
    cards.dedup();
    cards
}

pub(crate) fn open_draw_give_or_discard_hand_discard(
    tx: &mut Transaction,
    player: PlayerId,
    payment_count: u32,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let options = discardable_hand_cards_sorted(tx, player);
    let required = options.len().min(payment_count as usize);
    if required == 0 {
        return Ok(Vec::new());
    }
    if required == options.len() {
        return discard_cards(tx, player, options);
    }
    open_choice(
        tx,
        player,
        required as u8,
        required as u8,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::DrawThenChooseGiveOrDiscardHand,
        vec![3, player.raw(), required as u32, skill.raw()],
    )
}
