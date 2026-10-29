//! Command operations for `resolve_force_others_use_sha_on_nearest_or_lose_hp` and related helpers.

use super::*;

pub(crate) fn resolve_force_others_use_sha_on_nearest_or_lose_hp(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload
        .first()
        .ok_or_else(|| EffectError::EngineFault("missing Luanwu choice stage".into()))?;
    let owner = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let current = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != current {
        return Err(EffectError::Illegal(
            "Luanwu choice must be made by the prompted player".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(current)
        .is_some_and(|player| player.alive)
    {
        return Err(EffectError::Illegal(
            "Luanwu prompted player is no longer alive".into(),
        ));
    }

    match stage {
        0 => {
            let len = *payload.get(4).unwrap_or(&u32::MAX) as usize;
            let remaining = payload
                .get(5..)
                .filter(|rest| rest.len() == len)
                .ok_or_else(|| EffectError::EngineFault("truncated Luanwu queue".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            if cancelled || selections.is_empty() {
                return crate::effect::continue_luanwu_after_hp_loss(
                    tx, owner, current, skill, remaining,
                );
            }
            let [selected] = selections else {
                return Err(EffectError::Illegal(
                    "choose at most one Luanwu Sha source".into(),
                ));
            };
            let card = CardId::new(*selected);
            let legal_targets = crate::command::play::luanwu_sha_targets(tx, current, card);
            let target_limit = crate::query::sha_target_limit_for(tx.state(), current) as usize;
            if legal_targets.is_empty()
                || target_limit == 0
                || !crate::command::play::prompted_sha_source_cards(tx.state(), current)
                    .contains(&card)
            {
                return Err(EffectError::Illegal(
                    "Luanwu Sha source is no longer legal".into(),
                ));
            }
            let mut next = vec![
                1,
                owner.raw(),
                current.raw(),
                skill.raw(),
                card.raw(),
                remaining.len() as u32,
            ];
            next.extend(remaining.into_iter().map(PlayerId::raw));
            crate::effect::open_choice(
                tx,
                current,
                1,
                legal_targets.len().min(target_limit).min(u8::MAX as usize) as u8,
                true,
                legal_targets.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::ForceOthersUseShaOnNearestOrLoseHp,
                next,
            )
            .map(|_| ())
        }
        1 => {
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            let len = *payload.get(5).unwrap_or(&u32::MAX) as usize;
            let remaining = payload
                .get(6..)
                .filter(|rest| rest.len() == len)
                .ok_or_else(|| EffectError::EngineFault("truncated Luanwu queue".into()))?
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            if cancelled || selections.is_empty() {
                return crate::effect::continue_luanwu_after_hp_loss(
                    tx, owner, current, skill, remaining,
                );
            }
            let targets = selections
                .iter()
                .copied()
                .map(PlayerId::new)
                .collect::<Vec<_>>();
            if !remaining.is_empty() {
                tx.state_mut().resolution.deferred_effects.insert(
                    0,
                    EffectRequest::ForceOthersUseShaOnNearestOrLoseHp {
                        player: owner,
                        remaining,
                        started: true,
                        skill,
                    },
                );
            }
            match crate::command::play::play_luanwu_sha_source(tx, current, card, &targets) {
                Ok(()) => crate::effect::resume_deferred_resolution(tx),
                Err(error) => Err(error),
            }
        }
        _ => Err(EffectError::Illegal("unknown Luanwu choice stage".into())),
    }
}

pub(crate) fn resolve_choose_add_current_sha_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let card = CardId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "additional Sha target must be chosen by its user".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one additional Sha target".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::CommitAddCurrentShaTarget {
            player,
            card,
            target: PlayerId::new(selections[0]),
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn between_participants_ok(
    state: &crate::state::GameState,
    skill: engine_api::SkillId,
    player: PlayerId,
) -> bool {
    state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    })
}

/// ABI 680 stage 1 resume: `first` picked the opponent.
pub(crate) fn resolve_between_opponent(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let first = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if cancelled || observer != first || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "between comparison: first must choose exactly one opponent".into(),
        ));
    }
    let second = PlayerId::new(selections[0]);
    let state = tx.state();
    let legal = second != first
        && second != player
        && state
            .players
            .get(second)
            .is_some_and(|participant| participant.alive)
        && !state.hand_cards(second).is_empty()
        && !state.hand_cards(first).is_empty();
    if !between_participants_ok(state, skill, player) || !legal {
        return Err(EffectError::Illegal(
            "between comparison context changed".into(),
        ));
    }
    let options: Vec<u32> = state
        .hand_cards(first)
        .into_iter()
        .map(|card| card.raw())
        .collect();
    crate::effect::open_choice(
        tx,
        first,
        1,
        1,
        false,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::BetweenFirstCard,
        vec![player.raw(), first.raw(), second.raw(), skill.raw()],
    )
    .map(|_| ())
}

/// ABI 680 stage 2 resume: `first` stashed their card; now `second` picks.
pub(crate) fn resolve_between_first_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let first = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let second = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if cancelled || observer != first || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "between comparison: first must stash exactly one card".into(),
        ));
    }
    let options: Vec<u32> = tx
        .state()
        .hand_cards(second)
        .into_iter()
        .map(|card| card.raw())
        .collect();
    if options.is_empty() {
        return Err(EffectError::Illegal(
            "between comparison: opponent has no hand card".into(),
        ));
    }
    crate::effect::open_choice(
        tx,
        second,
        1,
        1,
        false,
        options,
        engine_api::ChoiceKind::SelectCards,
        crate::state::ContinuationKind::BetweenSecondCard,
        vec![
            player.raw(),
            first.raw(),
            second.raw(),
            CardId::new(selections[0]).raw(),
            skill.raw(),
        ],
    )
    .map(|_| ())
}

/// ABI 680 stage 3 resume: `second` stashed theirs; commit and run the
/// winner-vs-loser virtual Sha on non-tie.
pub(crate) fn resolve_between_second_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let first = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let second = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let first_card = CardId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if cancelled || observer != second || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "between comparison: second must stash exactly one card".into(),
        ));
    }
    crate::effect::commit_hand_compare_between(
        tx,
        player,
        first,
        second,
        first_card,
        CardId::new(selections[0]),
        skill,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_compare_target_hand_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let player_card = CardId::new(*payload.get(2).unwrap_or(&0));
    let win_skill_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let lose_skill_def = engine_api::SkillDefId::new(*payload.get(4).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(5).unwrap_or(&0));
    let grant_flag = *payload.get(6).unwrap_or(&0);
    if observer != target || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "comparison target must choose exactly one hand card".into(),
        ));
    }
    crate::effect::commit_hand_compare_ext(
        tx,
        player,
        target,
        player_card,
        CardId::new(selections[0]),
        win_skill_def,
        lose_skill_def,
        skill,
        grant_flag == 1,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_compare_target_hand_card_second_failure_seat_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 8 {
        return Err(EffectError::EngineFault(
            "bad comparison failure-path continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    if observer != target || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "comparison target must choose exactly one hand card".into(),
        ));
    }
    crate::effect::commit_hand_compare_second_failure_seat_damage(
        tx,
        player,
        target,
        CardId::new(payload[2]),
        CardId::new(selections[0]),
        payload[3],
        payload[4],
        payload[5],
        payload[6],
        engine_api::SkillId::new(payload[7]),
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_draw_or_discard_target_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad comparison win continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let draw_count = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player
        || cancelled
        || selections.len() != 1
        || draw_count == 0
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == player)
    {
        return Err(EffectError::Illegal(
            "choose the comparison win branch".into(),
        ));
    }
    match selections[0] {
        1 => run_effects(
            tx,
            vec![EffectRequest::DrawCardsFromSkill {
                player,
                count: draw_count,
                skill,
            }],
        )
        .map(|_| ()),
        2 if tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
            && !crate::query::discardable_target_cards(
                tx.state(),
                player,
                target,
                rule_api::TargetCardScope::HandEquip,
            )
            .is_empty() =>
        {
            run_effects(
                tx,
                vec![EffectRequest::ChooseDiscardTargetCard {
                    player,
                    target,
                    scope: rule_api::TargetCardScope::HandEquip,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "comparison win branch is no longer legal".into(),
        )),
    }
}

pub(crate) fn resolve_choose_seat_path_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 6 {
        return Err(EffectError::EngineFault(
            "bad seat-path continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let amount = payload[1];
    let skill = engine_api::SkillId::new(payload[2]);
    let path_count = usize::try_from(payload[3])
        .map_err(|_| EffectError::EngineFault("seat path is too large".into()))?;
    if path_count == 0 || payload.len() != 4 + path_count * 2 {
        return Err(EffectError::EngineFault(
            "invalid seat-path snapshot".into(),
        ));
    }
    if observer != player
        || cancelled
        || selections.len() != 1
        || !matches!(selections[0], 1 | 2)
        || amount == 0
        || !tx
            .state()
            .skills
            .get(skill)
            .is_some_and(|instance| instance.owner == player)
    {
        return Err(EffectError::Illegal("choose one seat path".into()));
    }
    let offset = if selections[0] == 1 {
        4
    } else {
        4 + path_count
    };
    let targets = payload[offset..offset + path_count]
        .iter()
        .copied()
        .map(PlayerId::new)
        .filter(|target| {
            *target != player
                && tx
                    .state()
                    .players
                    .get(*target)
                    .is_some_and(|participant| participant.alive)
        })
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return Ok(());
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamageEachInOrder {
            source: player,
            targets,
            amount,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_hand_compare_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let win_skill_def = engine_api::SkillDefId::new(*payload.get(1).unwrap_or(&0));
    let lose_skill_def = engine_api::SkillDefId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let grant_flag = *payload.get(4).unwrap_or(&0);
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "comparison owner must choose exactly one target".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    let state = tx.state();
    if !state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    }) || !state
        .players
        .get(target)
        .is_some_and(|participant| participant.alive)
        || target == player
        || state.hand_cards(player).is_empty()
        || state.hand_cards(target).is_empty()
    {
        return Err(EffectError::Illegal(
            "comparison target is no longer legal".into(),
        ));
    }
    crate::effect::open_choice(
        tx,
        player,
        1,
        1,
        false,
        state
            .hand_cards(player)
            .into_iter()
            .map(CardId::raw)
            .collect(),
        engine_api::ChoiceKind::SelectCards,
        ContinuationKind::ChooseHandComparePlayerCard,
        vec![
            player.raw(),
            target.raw(),
            win_skill_def.raw(),
            lose_skill_def.raw(),
            skill.raw(),
            grant_flag,
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_hand_compare_player_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let win_skill_def = engine_api::SkillDefId::new(*payload.get(2).unwrap_or(&0));
    let lose_skill_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let grant_flag = *payload.get(5).unwrap_or(&0);
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "comparison owner must choose exactly one hand card".into(),
        ));
    }
    if grant_flag == 1 {
        // ABI 678 grant flow: open the target's card choice directly, bypassing the
        // classic `ChooseCompareSelectedHandCard` request whose commit always grants
        // the chooser side.
        let state = tx.state();
        let options: Vec<u32> = state
            .hand_cards(target)
            .into_iter()
            .map(|card| card.raw())
            .collect();
        if options.is_empty() {
            return Err(EffectError::Illegal(
                "comparison target has no hand card".into(),
            ));
        }
        crate::effect::open_choice(
            tx,
            target,
            1,
            1,
            false,
            options,
            engine_api::ChoiceKind::SelectCards,
            ContinuationKind::ChooseCompareTargetHandCard,
            vec![
                player.raw(),
                target.raw(),
                CardId::new(selections[0]).raw(),
                win_skill_def.raw(),
                lose_skill_def.raw(),
                skill.raw(),
                1,
            ],
        )
        .map(|_| ())
    } else {
        run_effects(
            tx,
            vec![EffectRequest::ChooseCompareSelectedHandCard {
                player,
                target,
                card: CardId::new(selections[0]),
                win_skill_def,
                lose_skill_def,
                skill,
            }],
        )
        .map(|_| ())
    }
}

pub(crate) fn resolve_choose_add_skill_bound_card_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let card = CardId::new(*payload.get(1).unwrap_or(&0));
    let bound_target = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "bound extra target must be chosen by the card user".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections != [bound_target.raw()] {
        return Err(EffectError::Illegal(
            "choose exactly the current skill-bound target".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::CommitAddSkillBoundCardTarget {
            player,
            card,
            target: bound_target,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_current_card_use_enhancements(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let card = CardId::new(*payload.get(1).unwrap_or(&0));
    let helper_skill_def = engine_api::SkillDefId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "card-use enhancements require a forced choice by the card user".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) {
        return Err(EffectError::Illegal(
            "card-use enhancement skill context changed".into(),
        ));
    }
    let Some(extra_targets) =
        crate::command::play::current_card_use_enhancement_extra_targets(tx, player, card)?
    else {
        return Err(EffectError::Illegal(
            "card-use enhancement context changed".into(),
        ));
    };
    if !(1..=2).contains(&selections.len()) {
        return Err(EffectError::Illegal(
            "choose one or two card-use enhancements".into(),
        ));
    }
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if selected.len() != selections.len()
        || selected.iter().any(|option| *option > 3)
        || (selected.contains(&0) && extra_targets.is_empty())
    {
        return Err(EffectError::Illegal(
            "invalid or duplicate card-use enhancement".into(),
        ));
    }

    let mut policy_mask = 0u8;
    if selected.contains(&1) {
        policy_mask |= crate::state::SkillInstance::CARD_USE_POLICY_IGNORE_ARMOR;
    }
    if selected.contains(&2) {
        policy_mask |= crate::state::SkillInstance::CARD_USE_POLICY_UNRESPONDABLE;
    }
    if selected.contains(&3) {
        policy_mask |= crate::state::SkillInstance::CARD_USE_POLICY_DRAW_ON_DAMAGE;
    }
    if policy_mask != 0 {
        run_effects(
            tx,
            vec![EffectRequest::AddCardUsePolicySkill {
                player,
                card,
                policy_mask,
                skill_def: helper_skill_def,
            }],
        )?;
    }
    if selected.contains(&0) {
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            extra_targets.into_iter().map(PlayerId::raw).collect(),
            engine_api::ChoiceKind::SelectTargets,
            ContinuationKind::ChooseAddCurrentCardTarget,
            vec![player.raw(), card.raw(), skill.raw()],
        )
        .map(|_| ());
    }
    Ok(())
}
