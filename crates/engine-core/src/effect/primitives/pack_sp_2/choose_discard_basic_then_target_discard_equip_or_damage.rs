//! Effect primitives for `discard_all_judge` and related helpers.

use super::*;

pub(crate) fn choose_discard_basic_then_target_discard_equip_or_damage(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || player == target
    {
        return Err(EffectError::Illegal("invalid Xiaoguo context".into()));
    }
    let options = crate::query::standard_xiaoguo_basic_hand_cards(tx.state(), player);
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
        ContinuationKind::XiaoguoDiscardBasicThenEquipOrDamage,
        vec![0, player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn target_discard_then_duel_if_non_sha_and_hp_gte_owner(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || player == target
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "invalid target-discard Duel context".into(),
        ));
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        target,
        1,
        1,
        false,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::TargetDiscardThenDuelIfNonShaAndHpGteOwner,
        vec![player.raw(), target.raw(), skill.raw()],
    )
}

pub(crate) fn resolve_target_discard_duel(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    discarded_was_sha: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    if discarded_was_sha
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
        || tx.state().players.get(target).unwrap().hp < tx.state().players.get(player).unwrap().hp
        || crate::command::play::validate_source_free_card_on_target(
            tx,
            player,
            CardKind::TrickJuedou,
            target,
        )
        .is_err()
    {
        return Ok(Vec::new());
    }
    use_virtual_card_on(tx, player, CardKind::TrickJuedou.tag(), target)
}

pub(crate) fn choose_target_lose_hp_discard_hand_sha_if_basic(
    tx: &mut Transaction,
    player: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
    stage: u8,
    discarded_basic: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    match stage {
        0 => {
            if !originating_skill_is_enabled(tx, player, skill)
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
                .filter(|&target| {
                    !crate::query::discardable_target_cards(
                        tx.state(),
                        target,
                        target,
                        rule_api::TargetCardScope::Hand,
                    )
                    .is_empty()
                })
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
                ContinuationKind::ChooseTargetLoseHpDiscardHandShaIfBasic,
                vec![0, player.raw(), skill.raw()],
            )
        }
        1 => {
            let options = crate::query::discardable_target_cards(
                tx.state(),
                target,
                target,
                rule_api::TargetCardScope::Hand,
            );
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
                || options.is_empty()
            {
                return Ok(Vec::new());
            }
            open_choice(
                tx,
                target,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                ChoiceKind::SelectCards,
                ContinuationKind::ChooseTargetLoseHpDiscardHandShaIfBasic,
                vec![1, player.raw(), target.raw(), skill.raw()],
            )
        }
        2 => {
            if discarded_basic
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
            {
                try_use_virtual_sha_on_without_distance(tx, player, target, false, 0)
            } else {
                Ok(Vec::new())
            }
        }
        _ => Err(EffectError::ContentFault(
            "bad target-discard conditional Sha stage".into(),
        )),
    }
}

pub(crate) fn choose_current_sha_target_discard_redirect_to_owner(
    tx: &mut Transaction,
    owner: PlayerId,
    source: PlayerId,
    card: CardId,
    target: PlayerId,
    skill: engine_api::SkillId,
    ready_to_redirect: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let participants_alive = [owner, source, target].into_iter().all(|player| {
        tx.state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    });
    if !participants_alive || source == owner || target == owner {
        return Ok(Vec::new());
    }
    if !originating_skill_is_enabled(tx, owner, skill)
        || crate::command::play::validate_saved_sha_target(tx, source, card, target).is_err()
        || crate::command::play::validate_saved_sha_target(tx, source, card, owner).is_ok()
        || !crate::query::player_in_attack_range(tx.state(), target, owner)
        || crate::command::play::validate_sha_target_with_options(tx, source, card, owner, true)
            .is_err()
    {
        return Ok(Vec::new());
    }
    if ready_to_redirect {
        let targets =
            crate::command::play::redirect_saved_sha_target(tx, source, card, target, owner)?;
        return Ok(vec![DomainEvent::CardTargetAdded {
            player: source,
            card,
            target: owner,
            targets,
        }]);
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
    if options.is_empty() {
        return Ok(Vec::new());
    }
    open_choice(
        tx,
        target,
        0,
        1,
        true,
        options.into_iter().map(CardId::raw).collect(),
        ChoiceKind::SelectCards,
        ContinuationKind::TongjiDiscardRedirectSha,
        vec![
            owner.raw(),
            source.raw(),
            card.raw(),
            target.raw(),
            skill.raw(),
        ],
    )
}

pub(crate) fn choose_discard_redirect_current_sha(
    tx: &mut Transaction,
    player: PlayerId,
    source: PlayerId,
    card: CardId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    if !originating_skill_is_enabled(tx, player, skill)
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|source| source.alive)
        || source == player
    {
        return Ok(Vec::new());
    }
    let candidates = crate::command::play::saved_sha_redirect_targets_in_attack_range(
        tx, source, card, player, player,
    )
    .unwrap_or_default();
    if target.is_some() {
        if !candidates.contains(&target) {
            return Ok(Vec::new());
        }
        let targets =
            crate::command::play::redirect_saved_sha_target(tx, source, card, player, target)?;
        return Ok(vec![DomainEvent::CardTargetAdded {
            player: source,
            card,
            target,
            targets,
        }]);
    }
    let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
    if options.is_empty() || candidates.is_empty() {
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
        ContinuationKind::LiuliDiscardRedirectSha,
        vec![0, player.raw(), source.raw(), card.raw(), skill.raw()],
    )
}

pub(crate) fn choose_target_gain_bottom_card(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid_owner = originating_skill_is_enabled(tx, player, skill)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Play
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive);
    if !valid_owner {
        return Err(EffectError::Illegal("invalid Jijie context".into()));
    }

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Jijie draw pile is missing".into()))?;
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Jijie processing zone is missing".into()))?;

    if card.is_none() {
        if target.is_some() {
            return Err(EffectError::EngineFault(
                "Jijie target exists before card staging".into(),
            ));
        }
        if tx
            .state()
            .zones
            .get(draw)
            .is_none_or(|zone| zone.cards.is_empty())
        {
            reshuffle_discard_into_draw(tx)?;
        }
        let Some(card) = tx
            .state()
            .zones
            .get(draw)
            .and_then(|zone| zone.cards.first())
            .copied()
        else {
            return Ok(Vec::new());
        };
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ChooseTargetGainBottomCard {
                player,
                card,
                target: PlayerId::NONE,
                skill,
            },
        );
        let mut events = move_cards(tx, vec![card], processing, Some(draw))?;
        events.push(DomainEvent::CardsRevealed {
            cards: vec![card],
            to: Vec::new(),
        });
        return Ok(events);
    }

    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.zone == processing && state.owner.is_none())
    {
        return Err(EffectError::Illegal(
            "Jijie bottom card left processing".into(),
        ));
    }
    if target.is_none() {
        let options = tx.state().players.alive_by_seat();
        return open_choice(
            tx,
            player,
            1,
            1,
            false,
            options.iter().map(|target| target.raw()).collect(),
            ChoiceKind::SelectTargets,
            ContinuationKind::JijieChooseBottomCardRecipient,
            vec![player.raw(), card.raw(), skill.raw()],
        );
    }
    if !tx
        .state()
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "Jijie recipient is invalid or stale".into(),
        ));
    }
    let to = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::Illegal("Jijie recipient hand is missing".into()))?;
    give_cards_as(tx, player, vec![card], to, processing)
}

pub(crate) fn black_gain_repeat_request(
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
    protect_hand_limit: bool,
) -> EffectRequest {
    if protect_hand_limit {
        EffectRequest::JudgeBlackGainRepeatProtectHandLimit {
            player,
            card,
            skill,
            ready_to_choose,
        }
    } else {
        EffectRequest::JudgeBlackGainRepeat {
            player,
            card,
            skill,
            ready_to_choose,
        }
    }
}

pub(crate) fn judge_black_gain_repeat(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
    protect_hand_limit: bool,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid = originating_skill_is_enabled(tx, player, skill)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Prepare
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive);
    if !valid {
        return if card.is_none() {
            Err(EffectError::Illegal("invalid Luoshen context".into()))
        } else {
            Ok(Vec::new())
        };
    }
    if ready_to_choose {
        return open_choice(
            tx,
            player,
            0,
            1,
            true,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::LuoshenRepeatBlackJudgment,
            if protect_hand_limit {
                vec![player.raw(), skill.raw(), 1]
            } else {
                vec![player.raw(), skill.raw()]
            },
        );
    }
    if card.is_none() {
        let result = crate::command::judge::flip_judgment(
            tx,
            player,
            crate::command::judge::JudgmentFavor::Neutral,
        )?;
        let result_card = result.card;
        let is_black = matches!(
            result.suit,
            engine_api::Suit::Spade | engine_api::Suit::Club
        );
        let judgment = completed_judgment_event(player, result, CardId::NONE);
        if is_black {
            tx.state_mut()
                .resolution
                .deferred_tail_effects
                .push(black_gain_repeat_request(
                    player,
                    result_card,
                    skill,
                    false,
                    protect_hand_limit,
                ));
        }
        return Ok(vec![judgment]);
    }

    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(black_gain_repeat_request(
            player,
            card,
            skill,
            true,
            protect_hand_limit,
        ));
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Luoshen discard pile is missing".into()))?;
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.zone == discard)
    {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Luoshen owner hand is missing".into()))?;
    let events = move_cards(tx, vec![card], hand, Some(discard))?;
    if protect_hand_limit {
        tx.state_mut()
            .turn
            .hand_limit_exempt_cards
            .insert((player, card));
    }
    Ok(events)
}

pub(crate) fn judge_same_color_gain_repeat(
    tx: &mut Transaction,
    player: PlayerId,
    card: CardId,
    skill: engine_api::SkillId,
    ready_to_choose: bool,
    color: u8,
) -> Result<Vec<DomainEvent>, EffectError> {
    let valid = originating_skill_is_enabled(tx, player, skill)
        && tx.state().turn.current == player
        && tx.state().turn.phase == Phase::Prepare
        && tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive);
    if !valid {
        return if card.is_none() && color == 0 {
            Err(EffectError::Illegal("invalid Yjluoshen context".into()))
        } else {
            Ok(Vec::new())
        };
    }
    if ready_to_choose {
        return open_choice(
            tx,
            player,
            0,
            1,
            true,
            vec![0, 1],
            ChoiceKind::Confirm,
            ContinuationKind::JudgeSameColorGainRepeat,
            vec![player.raw(), skill.raw(), u32::from(color)],
        );
    }
    if card.is_none() {
        let result = crate::command::judge::flip_judgment(
            tx,
            player,
            crate::command::judge::JudgmentFavor::Neutral,
        )?;
        let result_card = result.card;
        let result_color = crate::query::card_color_mask_for_suit(result.suit);
        let first_color = if color == 0 { result_color } else { color };
        let judgment = completed_judgment_event(player, result, CardId::NONE);
        if result_color != 0 && result_color == first_color {
            tx.state_mut().resolution.deferred_tail_effects.push(
                EffectRequest::JudgeSameColorGainRepeat {
                    player,
                    card: result_card,
                    skill,
                    ready_to_choose: false,
                    color: first_color,
                },
            );
        }
        return Ok(vec![judgment]);
    }

    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::JudgeSameColorGainRepeat {
            player,
            card,
            skill,
            ready_to_choose: true,
            color,
        });
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("Yjluoshen discard pile is missing".into()))?;
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.zone == discard)
    {
        return Ok(Vec::new());
    }
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("Yjluoshen owner hand is missing".into()))?;
    move_cards(tx, vec![card], hand, Some(discard))
}

pub(crate) fn use_virtual_juedou_second_on_first(
    tx: &mut Transaction,
    targets: Vec<PlayerId>,
    card: CardId,
) -> Result<Vec<DomainEvent>, EffectError> {
    let [target, user] = targets.as_slice() else {
        return Err(EffectError::ContentFault(
            "Lijian requires exactly two selected targets".into(),
        ));
    };
    let target = *target;
    let user = *user;
    if target == user {
        return Err(EffectError::ContentFault(
            "Lijian targets must be distinct".into(),
        ));
    }
    if card.is_none() {
        crate::command::play::validate_source_free_card_on_target(
            tx,
            user,
            CardKind::TrickJuedou,
            target,
        )?;
        let card = materialize_virtual_card(tx, user, CardKind::TrickJuedou)?;
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::UseVirtualJuedouSecondOnFirst { targets, card });
        return super::super::run_effects(
            tx,
            vec![EffectRequest::UseCard {
                player: user,
                card,
                targets: vec![target],
            }],
        );
    }
    if !tx
        .state()
        .cards
        .get(card)
        .is_some_and(|state| state.def.is_none() && state.kind == CardKind::TrickJuedou)
    {
        return Err(EffectError::Illegal(
            "Lijian virtual Duel context changed".into(),
        ));
    }
    crate::command::play::resolve_independent_trick_without_wuxie(
        tx,
        user,
        card,
        &[target],
        crate::command::play::trick_effect::JUEDOU,
    )?;
    Ok(Vec::new())
}
