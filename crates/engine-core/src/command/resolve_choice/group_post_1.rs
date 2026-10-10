//! Choice-continuation handlers (post 1).
//!
//! Split out of the former single-file `resolve_choice.rs` (ADR 0594). Item text is
//! verbatim and in original order; only default-private visibility was widened to
//! `pub(crate)` so the dispatch in the parent module can reach it.

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
                || !crate::command::play::prompted_sha_source_cards(tx.state(), current).contains(&card)
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

fn between_participants_ok(
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

pub(crate) fn resolve_choose_add_current_card_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let card = CardId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    if observer != player || cancelled || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one legal additional card target".into(),
        ));
    }
    let target = PlayerId::new(selections[0]);
    let Some(extra_targets) =
        crate::command::play::current_card_use_enhancement_extra_targets(tx, player, card)?
    else {
        return Err(EffectError::Illegal(
            "additional card target context changed".into(),
        ));
    };
    if !extra_targets.contains(&target) {
        return Err(EffectError::Illegal(
            "selected card target is no longer legal".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::CommitAddSkillBoundCardTarget {
            player,
            card,
            target,
            skill,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_add_or_remove_current_card_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.first() == Some(&2) {
        return resolve_choose_current_trick_targets_to_nullify(
            tx, observer, payload, selections, cancelled,
        );
    }
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let card = CardId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "current-card target adjustment context changed".into(),
        ));
    }
    if cancelled {
        return run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).map(|_| ());
    }
    let (add_targets, remove_targets) =
        crate::command::play::current_card_target_adjustments(tx, player, card)?;
    if stage == 0 {
        if selections.len() != 1 || !matches!(selections[0], 0 | 1) {
            return Err(EffectError::Illegal(
                "choose add-target or remove-target mode".into(),
            ));
        }
        let mode = selections[0];
        let targets = if mode == 0 {
            add_targets
        } else {
            remove_targets
        };
        if targets.is_empty() {
            return Err(EffectError::Illegal(
                "selected target-adjustment mode is no longer legal".into(),
            ));
        }
        return crate::effect::open_choice(
            tx,
            player,
            1,
            1,
            false,
            targets.into_iter().map(PlayerId::raw).collect(),
            engine_api::ChoiceKind::SelectTargets,
            ContinuationKind::ChooseAddOrRemoveCurrentCardTarget,
            vec![1, player.raw(), card.raw(), skill.raw(), mode],
        )
        .map(|_| ());
    }
    if stage != 1 || payload.len() != 5 || selections.len() != 1 {
        return Err(EffectError::Illegal(
            "choose exactly one current-card target".into(),
        ));
    }
    let mode = payload[4];
    let target = PlayerId::new(selections[0]);
    let legal = if mode == 0 {
        &add_targets
    } else if mode == 1 {
        &remove_targets
    } else {
        return Err(EffectError::Illegal("bad target-adjustment mode".into()));
    };
    if !legal.contains(&target) {
        return Err(EffectError::Illegal(
            "selected current-card target is no longer legal".into(),
        ));
    }
    if mode == 0 {
        return run_effects(
            tx,
            vec![
                EffectRequest::RemoveSkill { skill },
                EffectRequest::CommitAddSkillBoundCardTarget {
                    player,
                    card,
                    target,
                    skill,
                },
            ],
        )
        .map(|_| ());
    }
    crate::command::play::adjust_saved_card_target(tx, player, card, target, false)?;
    run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).map(|_| ())
}

pub(crate) fn resolve_choose_current_trick_targets_to_nullify(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [2, player, source, card, skill] = payload else {
        return Err(EffectError::EngineFault(
            "bad current-trick target nullification payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let source = PlayerId::new(*source);
    let card = CardId::new(*card);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "current-trick target nullification context changed".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    let kind = tx
        .state()
        .cards
        .get(card)
        .map(|card| crate::query::effective_card_kind(tx.state(), source, card));
    let (_, legal) = crate::command::play::current_card_target_adjustments(tx, source, card)?;
    let selected: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
    if !kind.is_some_and(|kind| kind.is_trick() && !kind.is_delayed_trick())
        || selected.is_empty()
        || selected.len() > legal.len()
        || selected.iter().any(|target| !legal.contains(target))
        || (0..selected.len()).any(|index| selected[index + 1..].contains(&selected[index]))
    {
        return Err(EffectError::Illegal(
            "selected current-trick targets are no longer legal".into(),
        ));
    }
    crate::command::play::exclude_all_saved_card_targets(tx, source, card, &selected)?;
    run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).map(|_| ())
}

pub(crate) fn resolve_choose_discard_matching_equip_color_add_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let player = PlayerId::new(*payload.first().unwrap_or(&0));
    let target = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill_def = engine_api::SkillDefId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "matching-color discard chooser mismatch".into(),
        ));
    }
    if cancelled {
        return Ok(());
    }
    if selections.len() != 1 {
        return Err(EffectError::Illegal(
            "select exactly one matching-color non-basic discard cost".into(),
        ));
    }
    if player == target
        || tx.state().turn.current != target
        || tx.state().turn.phase != crate::state::Phase::Play
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "matching-color discard context changed".into(),
        ));
    }
    let cost = CardId::new(selections[0]);
    if !crate::query::discardable_non_basic_matching_equip_color_cards(tx.state(), player)
        .contains(&cost)
    {
        return Err(EffectError::Illegal(
            "matching-color discard cost is no longer legal".into(),
        ));
    }
    let color_mask = crate::query::effective_card_color_mask(tx.state(), player, cost);
    run_effects(
        tx,
        vec![
            EffectRequest::DiscardCards {
                player,
                cards: vec![cost],
            },
            EffectRequest::AddCardColorLockSkill {
                player: target,
                source: player,
                color_mask,
                skill_def,
            },
        ],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_draw_or_discard_add_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let draw_skill_def = engine_api::SkillDefId::new(*payload.get(2).unwrap_or(&0));
    let discard_skill_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    let three_way = payload.get(5).copied().unwrap_or(0) != 0;
    if observer != player {
        return Err(EffectError::Illegal(
            "draw-or-discard branch must be chosen by the skill owner".into(),
        ));
    }
    let state = tx.state();
    if state.turn.current != player
        || state.turn.phase != crate::state::Phase::Play
        || !state.skills.get(skill).is_some_and(|instance| {
            instance.owner == player && state.skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "draw-or-discard skill context changed".into(),
        ));
    }
    if cancelled {
        return if three_way && stage == 0 {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "this draw-or-discard choice cannot be cancelled".into(),
            ))
        };
    }

    match (stage, selections) {
        (0, [1]) if three_way => {
            run_effects(tx, vec![EffectRequest::DrawCards { player, count: 1 }]).map(|_| ())
        }
        (0, [2]) if three_way => run_effects(
            tx,
            vec![
                EffectRequest::DrawCards { player, count: 2 },
                EffectRequest::AddSkill {
                    player,
                    skill_def: draw_skill_def,
                },
            ],
        )
        .map(|_| ()),
        (0, [1]) => run_effects(
            tx,
            vec![
                EffectRequest::AddSkill {
                    player,
                    skill_def: draw_skill_def,
                },
                EffectRequest::DrawCards { player, count: 1 },
            ],
        )
        .map(|_| ()),
        (0, [branch]) if *branch == if three_way { 3 } else { 2 } => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "discard branch is no longer available".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseDrawOrDiscardAddSkill,
                vec![
                    1,
                    player.raw(),
                    draw_skill_def.raw(),
                    discard_skill_def.raw(),
                    skill.raw(),
                    u32::from(three_way),
                ],
            )
            .map(|_| ())
        }
        (1, [card]) => {
            let card = CardId::new(*card);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "discard cost is no longer a legal hand or equipment card".into(),
                ));
            }
            let effects = if three_way {
                vec![
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![card],
                    },
                    EffectRequest::AddSkill {
                        player,
                        skill_def: discard_skill_def,
                    },
                ]
            } else {
                vec![
                    EffectRequest::AddSkill {
                        player,
                        skill_def: discard_skill_def,
                    },
                    EffectRequest::DiscardCards {
                        player,
                        cards: vec![card],
                    },
                ]
            };
            run_effects(tx, effects).map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "choose one valid draw-or-discard branch or discard cost".into(),
        )),
    }
}

pub(crate) fn resolve_choose_draw_or_recast_add_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let draw_skill_def = engine_api::SkillDefId::new(*payload.get(2).unwrap_or(&0));
    let recast_skill_def = engine_api::SkillDefId::new(*payload.get(3).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(4).unwrap_or(&0));
    if observer != player {
        return Err(EffectError::Illegal(
            "draw-or-recast branch must be resolved by the skill owner".into(),
        ));
    }
    if cancelled {
        return if stage == 0 && selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "only the initial draw-or-recast choice can be cancelled".into(),
            ))
        };
    }
    let state = tx.state();
    if state.turn.current != player
        || state.turn.phase != crate::state::Phase::Play
        || !state.skills.get(skill).is_some_and(|instance| {
            instance.owner == player && state.skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "draw-or-recast skill context changed".into(),
        ));
    }

    match (stage, selections) {
        (0, [1]) => run_effects(
            tx,
            vec![
                EffectRequest::AddSkill {
                    player,
                    skill_def: draw_skill_def,
                },
                EffectRequest::DrawCards { player, count: 1 },
            ],
        )
        .map(|_| ()),
        (0, [2]) => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), player);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "recast branch is no longer available".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseDrawOrRecastAddSkill,
                vec![
                    1,
                    player.raw(),
                    draw_skill_def.raw(),
                    recast_skill_def.raw(),
                    skill.raw(),
                ],
            )
            .map(|_| ())
        }
        (1, [card]) => {
            let card = CardId::new(*card);
            if !crate::query::discardable_hand_equip_cards(tx.state(), player).contains(&card) {
                return Err(EffectError::Illegal(
                    "recast card is no longer in hand or equipment".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::AddSkill {
                        player,
                        skill_def: recast_skill_def,
                    },
                    EffectRequest::RecastCards {
                        player,
                        cards: vec![card],
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "choose one valid draw-or-recast branch or recast card".into(),
        )),
    }
}

pub(crate) fn resolve_choose_draw_or_discard_target_card_record_sha(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad draw-or-discard Sha continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let card = CardId::new(payload[1]);
    let target = PlayerId::new(payload[2]);
    let skill = engine_api::SkillId::new(payload[3]);
    if observer != player {
        return Err(EffectError::Illegal(
            "draw-or-discard Sha branch must be resolved by its user".into(),
        ));
    }
    if cancelled {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled draw-or-discard Sha branch selected an option".into(),
            ))
        };
    }
    let state = tx.state();
    if ![player, target].into_iter().all(|participant| {
        state
            .players
            .get(participant)
            .is_some_and(|participant| participant.alive)
    }) || !state.skills.get(skill).is_some_and(|instance| {
        instance.owner == player && state.skill_effectively_enabled(instance)
    }) || !state.resolution.sha_target_resolving(player, card, target)
    {
        return Err(EffectError::Illegal(
            "draw-or-discard Sha context changed".into(),
        ));
    }

    let effect = match selections {
        [1] => EffectRequest::DrawCardsFromSkill {
            player,
            count: 1,
            skill,
        },
        [2] if !crate::query::discardable_target_cards(
            tx.state(),
            player,
            target,
            rule_api::TargetCardScope::HandEquip,
        )
        .is_empty() =>
        {
            EffectRequest::ChooseDiscardTargetCard {
                player,
                target,
                scope: rule_api::TargetCardScope::HandEquip,
            }
        }
        _ => {
            return Err(EffectError::Illegal(
                "choose one available draw-or-discard Sha branch".into(),
            ));
        }
    };
    if !tx
        .state_mut()
        .resolution
        .record_sha_skill_target(player, card, skill, target)
    {
        return Err(EffectError::Illegal(
            "resolving Sha target disappeared".into(),
        ));
    }
    run_effects(tx, vec![effect]).map(|_| ())
}

pub(crate) fn resolve_reveal_hand_choose_discard_color(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, offered_color_mask] = payload else {
        return Err(EffectError::EngineFault(
            "bad reveal-hand color continuation payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let [color] = selections else {
        return Err(EffectError::Illegal(
            "choose exactly one revealed hand color".into(),
        ));
    };
    if observer != player || cancelled {
        return Err(EffectError::Illegal(
            "revealed hand color requires a forced choice by that player".into(),
        ));
    }
    if !matches!(*color, 0b01 | 0b10) || offered_color_mask & color == 0 {
        return Err(EffectError::Illegal(
            "selected hand color was not offered".into(),
        ));
    }
    if !tx
        .state()
        .players
        .get(player)
        .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "revealed hand color player is no longer alive".into(),
        ));
    }
    let discardable: std::collections::BTreeSet<_> =
        crate::query::discardable_hand_equip_cards(tx.state(), player)
            .into_iter()
            .collect();
    let cards: Vec<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .filter(|card| {
            discardable.contains(card)
                && u32::from(crate::query::effective_card_color_mask(
                    tx.state(),
                    player,
                    *card,
                )) == *color
        })
        .collect();
    if cards.is_empty() {
        return Ok(());
    }
    run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).map(|_| ())
}

pub(crate) fn resolve_choose_discard_target_cards_recover_same_category(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if cancelled || payload.len() < 6 {
        return Err(EffectError::Illegal(
            "target discard-and-recover choice cannot be cancelled".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let count = payload[2];
    let amount = payload[3];
    let skill = engine_api::SkillId::new(payload[4]);
    let option_count = payload[5] as usize;
    let required = usize::try_from(count)
        .unwrap_or(usize::MAX)
        .min(option_count);
    if observer != player
        || count == 0
        || required == 0
        || payload.len() != 6 + option_count
        || selections.len() != required
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "invalid target discard-and-recover selection".into(),
        ));
    }
    let original: std::collections::BTreeSet<_> = payload[6..].iter().copied().collect();
    let current: std::collections::BTreeSet<_> = crate::query::discardable_target_cards(
        tx.state(),
        player,
        target,
        rule_api::TargetCardScope::HandEquip,
    )
    .into_iter()
    .map(CardId::raw)
    .collect();
    let mut selected = std::collections::BTreeSet::new();
    if !selections
        .iter()
        .all(|card| selected.insert(*card) && original.contains(card) && current.contains(card))
    {
        return Err(EffectError::Illegal(
            "target discard card is duplicated or no longer legal".into(),
        ));
    }
    let cards: Vec<_> = selections.iter().copied().map(CardId::new).collect();
    let same_category =
        crate::effect::target_discard_cards_share_category(tx.state(), target, &cards);
    if required == usize::try_from(count).unwrap_or(usize::MAX) && amount > 0 && same_category {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::RecoverHp {
                source: player,
                player,
                amount,
            });
    }
    run_effects(
        tx,
        vec![EffectRequest::DiscardTargetCards {
            actor: player,
            target,
            cards,
            scope: rule_api::TargetCardScope::HandEquip,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_gain_sha_for_opening_extra_play(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, card, from, skill, damage_baseline] = payload else {
        return Err(EffectError::Illegal(
            "bad opening extra Play Sha-gain payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let card = CardId::new(*card);
    let from = engine_api::ZoneId::new(*from);
    let skill = engine_api::SkillId::new(*skill);
    if observer != player {
        return Err(EffectError::Illegal(
            "opening extra Play Sha gain belongs to another player".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled opening extra Play Sha gain has selections".into(),
            ))
        };
    }
    if selections != [1] {
        return Err(EffectError::Illegal(
            "opening extra Play Sha gain requires the confirm option".into(),
        ));
    }
    let state = tx.state();
    if state.turn.current != player
        || state.turn.phase != Phase::Play
        || state.turn.opening_extra_play_active_skill != skill
        || !state.skills.get(skill).is_some_and(|instance| {
            instance.owner == player && state.skill_effectively_enabled(instance)
        })
        || !state.cards.get(card).is_some_and(|candidate| {
            candidate.zone == from
                && candidate.kind == CardKind::Sha
                && state.zones.get(from).is_some_and(|zone| {
                    matches!(zone.kind, ZoneKind::DrawPile | ZoneKind::DiscardPile)
                })
        })
    {
        return Err(EffectError::Illegal(
            "opening extra Play Sha-gain context changed".into(),
        ));
    }
    let hand = state
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::Illegal("no hand zone".into()))?;
    let policy = crate::state::SkillInstance::phase_card_use_policy_state(
        card,
        crate::state::SkillInstance::CARD_USE_POLICY_DISTANCE_UNLIMITED,
        *damage_baseline,
    );
    tx.state_mut()
        .skills
        .get_mut(skill)
        .expect("validated originating skill")
        .state_blob = policy;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: hand,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_opening_extra_play_hp_loss_gain(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let [player, kind, mandatory_skill_def, skill] = payload else {
        return Err(EffectError::Illegal(
            "bad opening extra Play HP-loss gain payload".into(),
        ));
    };
    let player = PlayerId::new(*player);
    let skill = engine_api::SkillId::new(*skill);
    let mandatory_skill_def = engine_api::SkillDefId::new(*mandatory_skill_def);
    if observer != player {
        return Err(EffectError::Illegal(
            "opening extra Play HP-loss gain belongs to another player".into(),
        ));
    }
    if cancelled || selections.is_empty() {
        return if selections.is_empty() {
            Ok(())
        } else {
            Err(EffectError::Illegal(
                "cancelled opening extra Play HP-loss gain has selections".into(),
            ))
        };
    }
    if selections != [1] {
        return Err(EffectError::Illegal(
            "opening extra Play HP-loss gain requires the confirm option".into(),
        ));
    }
    let state = tx.state();
    if state.turn.current != player
        || state.turn.phase != Phase::Play
        || state.turn.opening_extra_play_active_skill != skill
        || !state.skills.get(skill).is_some_and(|instance| {
            instance.owner == player && state.skill_effectively_enabled(instance)
        })
        || state
            .skills
            .sorted_live()
            .into_iter()
            .any(|instance| instance.owner == player && instance.def == mandatory_skill_def)
    {
        return Err(EffectError::Illegal(
            "opening extra Play HP-loss gain context changed".into(),
        ));
    }
    crate::effect::commit_opening_extra_play_hp_loss_gain(
        tx,
        player,
        u8::try_from(*kind)
            .map_err(|_| EffectError::Illegal("invalid opening gain card kind".into()))?,
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_target_draw_or_discard_by_skill_use_count_link_if_wounded(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let target = PlayerId::new(*payload.get(2).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
    let count = *payload.get(4).unwrap_or(&0);
    let toggle_link = payload.get(5).copied().unwrap_or(0) != 0;
    if cancelled || count == 0 {
        return Err(EffectError::Illegal(
            "draw-or-discard branch cannot be cancelled or empty".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || !tx
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
            "draw-or-discard branch is no longer available".into(),
        ));
    }
    let finish = EffectRequest::FinishLinkIfTargetWounded {
        player,
        target,
        toggle_link,
        skill,
    };
    match (stage, selections) {
        (0, [1]) if observer == player => run_effects(
            tx,
            vec![
                EffectRequest::DrawCardsFromSkill {
                    player: target,
                    count,
                    skill,
                },
                finish,
            ],
        )
        .map(|_| ()),
        (0, [2]) if observer == player => {
            let options = crate::query::discardable_hand_equip_cards(tx.state(), target);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "target discard branch is no longer available".into(),
                ));
            }
            let required = count.min(options.len() as u32) as u8;
            crate::effect::open_choice(
                tx,
                target,
                required,
                required,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded,
                vec![
                    1,
                    player.raw(),
                    target.raw(),
                    skill.raw(),
                    count,
                    u32::from(toggle_link),
                ],
            )
            .map(|_| ())
        }
        (1, _) if observer == target => {
            let options: std::collections::BTreeSet<u32> =
                crate::query::discardable_hand_equip_cards(tx.state(), target)
                    .into_iter()
                    .map(CardId::raw)
                    .collect();
            let selected: std::collections::BTreeSet<u32> = selections.iter().copied().collect();
            let required = count.min(options.len() as u32) as usize;
            if required == 0
                || selections.len() != required
                || selected.len() != required
                || !selected.is_subset(&options)
            {
                return Err(EffectError::Illegal(
                    "target must discard the current required card count".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::DiscardCards {
                        player: target,
                        cards: selections.iter().copied().map(CardId::new).collect(),
                    },
                    finish,
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "choose one valid draw-or-discard branch".into(),
        )),
    }
}

pub(crate) fn resolve_choose_reveal_other_hand_card_remember_category(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    let player = PlayerId::new(*payload.get(1).unwrap_or(&0));
    let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
    let bound_target = PlayerId::new(*payload.get(3).unwrap_or(&0));
    let phase_owner = if stage == 2 { bound_target } else { player };
    if cancelled
        || observer != player
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || tx.state().turn.current != phase_owner
        || tx.state().turn.phase != Phase::Play
    {
        return Err(EffectError::Illegal(
            "remembered-category choice context changed".into(),
        ));
    }
    match (stage, selections) {
        (0, [target]) => {
            let target = PlayerId::new(*target);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "choose another living player with hand cards".into(),
                ));
            }
            let options: Vec<_> = tx.state().hand_cards(target);
            if options.is_empty() {
                return Err(EffectError::Illegal(
                    "chosen player's hand is now empty".into(),
                ));
            }
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                options.into_iter().map(CardId::raw).collect(),
                engine_api::ChoiceKind::SelectCards,
                ContinuationKind::ChooseRevealOtherHandCardRememberCategory,
                vec![1, player.raw(), skill.raw(), target.raw()],
            )
            .map(|_| ())
        }
        (1, [card]) if payload.len() == 4 => {
            let target = PlayerId::new(payload[3]);
            let card = CardId::new(*card);
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal(
                    "revealed card is no longer in the chosen hand".into(),
                ));
            }
            let category = tx
                .state()
                .cards
                .get(card)
                .map(|card| crate::query::effective_card_kind(tx.state(), target, card))
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
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.set_remembered_category(category))
            {
                return Err(EffectError::Illegal(
                    "revealed card has no supported logical category".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::RevealCards {
                    cards: vec![card],
                    to: Vec::new(),
                }],
            )
            .map(|_| ())
        }
        (2, [card]) if payload.len() == 4 => {
            let target = bound_target;
            if !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "bound reveal target is no longer alive".into(),
                ));
            }
            let card = CardId::new(*card);
            if !tx.state().hand_cards(target).contains(&card) {
                return Err(EffectError::Illegal(
                    "revealed card is no longer in the bound hand".into(),
                ));
            }
            let category = tx
                .state()
                .cards
                .get(card)
                .map(|card| crate::query::effective_card_kind(tx.state(), target, card))
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
            if !tx
                .state_mut()
                .skills
                .get_mut(skill)
                .is_some_and(|instance| instance.set_bound_category_target(target, category))
            {
                return Err(EffectError::Illegal(
                    "revealed card has no supported bound category".into(),
                ));
            }
            run_effects(
                tx,
                vec![EffectRequest::RevealCards {
                    cards: vec![card],
                    to: Vec::new(),
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::Illegal(
            "choose exactly one reveal target or hand card".into(),
        )),
    }
}

pub(crate) fn resolve_give_exact_record_phase_target(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 {
        return Err(EffectError::EngineFault(
            "bad exact phase-gift continuation".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let count = payload[3] as usize;
    if cancelled || observer != player || count == 0 || selections.len() != count {
        return Err(EffectError::Illegal(
            "phase gift requires the exact card count".into(),
        ));
    }
    if !tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    }) || tx.state().turn.current != target
        || tx.state().turn.phase != Phase::Play
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "phase gift context changed before payment".into(),
        ));
    }
    let selected: std::collections::BTreeSet<_> = selections.iter().copied().collect();
    if selected.len() != count {
        return Err(EffectError::Illegal(
            "phase gift cards must be distinct".into(),
        ));
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player);
    let equip = tx.state().zones.find(ZoneKind::Equip, player);
    let current: std::collections::BTreeMap<_, _> = selections
        .iter()
        .copied()
        .map(CardId::new)
        .filter_map(|card| {
            tx.state()
                .cards
                .get(card)
                .filter(|state| {
                    state.owner == player && (Some(state.zone) == hand || Some(state.zone) == equip)
                })
                .map(|state| (card, state.zone))
        })
        .collect();
    if current.len() != count {
        return Err(EffectError::Illegal(
            "one or more phase gift cards left hand/equipment".into(),
        ));
    }
    if !tx
        .state_mut()
        .skills
        .get_mut(skill)
        .is_some_and(|instance| instance.set_phase_gift_target(target))
    {
        return Err(EffectError::ContentFault(
            "could not record phase gift target".into(),
        ));
    }
    let target_hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, target)
        .ok_or_else(|| EffectError::EngineFault("phase gift target hand missing".into()))?;
    let mut by_zone = std::collections::BTreeMap::<u32, Vec<CardId>>::new();
    for card in selections.iter().copied().map(CardId::new) {
        let zone = current[&card];
        by_zone.entry(zone.raw()).or_default().push(card);
    }
    let effects = by_zone
        .into_iter()
        .map(|(zone, cards)| EffectRequest::GiveCards {
            cards,
            to: target_hand,
            from: engine_api::ZoneId::new(zone),
        })
        .collect();
    run_effects(tx, effects).map(|_| ())
}

pub(crate) fn resolved_sha_gift_groups(
    payload: &[u32],
    offset: usize,
) -> Result<(Vec<CardId>, Vec<CardId>, Vec<PlayerId>), EffectError> {
    let sha_len = *payload
        .get(offset)
        .ok_or_else(|| EffectError::EngineFault("missing resolved Sha group length".into()))?
        as usize;
    let sha_end = offset
        .checked_add(1)
        .and_then(|start| start.checked_add(sha_len))
        .ok_or_else(|| EffectError::EngineFault("resolved Sha group overflow".into()))?;
    let sha: Vec<_> = payload
        .get(offset + 1..sha_end)
        .ok_or_else(|| EffectError::EngineFault("truncated resolved Sha group".into()))?
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let shan_len = *payload
        .get(sha_end)
        .ok_or_else(|| EffectError::EngineFault("missing resolved Shan group length".into()))?
        as usize;
    let shan_end = sha_end
        .checked_add(1)
        .and_then(|start| start.checked_add(shan_len))
        .ok_or_else(|| EffectError::EngineFault("resolved Shan group overflow".into()))?;
    let shan: Vec<_> = payload
        .get(sha_end + 1..shan_end)
        .ok_or_else(|| EffectError::EngineFault("truncated resolved Shan group".into()))?
        .iter()
        .copied()
        .map(CardId::new)
        .collect();
    let excluded_len = *payload
        .get(shan_end)
        .ok_or_else(|| EffectError::EngineFault("missing resolved Sha exclusions".into()))?
        as usize;
    let excluded_end = shan_end
        .checked_add(1)
        .and_then(|start| start.checked_add(excluded_len))
        .ok_or_else(|| EffectError::EngineFault("resolved Sha exclusions overflow".into()))?;
    if payload.len() != excluded_end {
        return Err(EffectError::EngineFault(
            "truncated resolved Sha exclusions".into(),
        ));
    }
    let excluded = payload[shan_end + 1..excluded_end]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    Ok((sha, shan, excluded))
}

pub(crate) fn current_discard_group(tx: &Transaction, cards: &[CardId]) -> Vec<CardId> {
    let discard = tx.state().zones.find(ZoneKind::DiscardPile, PlayerId::NONE);
    cards
        .iter()
        .copied()
        .filter(|card| {
            tx.state()
                .cards
                .get(*card)
                .is_some_and(|state| Some(state.zone) == discard)
        })
        .collect()
}

pub(crate) fn give_resolved_sha_group(
    tx: &mut Transaction,
    owner: PlayerId,
    recipient: PlayerId,
    skill: engine_api::SkillId,
    cards: &[CardId],
    draw_if_black: bool,
) -> Result<(), EffectError> {
    let cards = current_discard_group(tx, cards);
    if cards.is_empty() {
        return Err(EffectError::Illegal(
            "chosen resolved card group is no longer in discard".into(),
        ));
    }
    let red = cards.iter().any(|card| {
        tx.state().cards.get(*card).is_some_and(|card| {
            matches!(
                card.suit,
                engine_api::Suit::Heart | engine_api::Suit::Diamond
            )
        })
    });
    let black = draw_if_black
        && cards.iter().any(|card| {
            tx.state().cards.get(*card).is_some_and(|card| {
                matches!(card.suit, engine_api::Suit::Spade | engine_api::Suit::Club)
            })
        });
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::Illegal("no discard pile".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, recipient)
        .ok_or_else(|| EffectError::Illegal("no recipient hand".into()))?;
    if red {
        tx.state_mut().resolution.deferred_tail_effects.push(
            EffectRequest::ChooseUseShaAfterResolvedGift {
                owner,
                player: recipient,
                skill,
            },
        );
    }
    if black {
        tx.state_mut()
            .resolution
            .deferred_tail_effects
            .push(EffectRequest::DrawCards {
                player: recipient,
                count: 1,
            });
    }
    run_effects(
        tx,
        vec![EffectRequest::GiveCards {
            cards,
            to: hand,
            from: discard,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_gain_resolved_sha_or_shan_then_optional_use_sha(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let stage = *payload.first().unwrap_or(&u32::MAX);
    match stage {
        0 | 4 => {
            let owner = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let skill = engine_api::SkillId::new(*payload.get(2).unwrap_or(&0));
            if cancelled || observer != owner || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose exactly one resolved Sha gift recipient".into(),
                ));
            }
            let (sha, shan, excluded) = resolved_sha_gift_groups(payload, 3)?;
            let recipient = PlayerId::new(selections[0]);
            if excluded.contains(&recipient)
                || !tx
                    .state()
                    .players
                    .get(recipient)
                    .is_some_and(|participant| participant.alive)
                || !tx.state().skills.get(skill).is_some_and(|instance| {
                    instance.owner == owner && tx.state().skill_effectively_enabled(instance)
                })
            {
                return Err(EffectError::Illegal(
                    "resolved Sha gift recipient is no longer legal".into(),
                ));
            }
            let sha = current_discard_group(tx, &sha);
            let shan = current_discard_group(tx, &shan);
            if stage == 4 {
                let cards: Vec<_> = sha.into_iter().chain(shan).collect();
                return if cards.is_empty() {
                    Ok(())
                } else {
                    give_resolved_sha_group(tx, owner, recipient, skill, &cards, true)
                };
            }
            match (sha.is_empty(), shan.is_empty()) {
                (true, true) => Ok(()),
                (false, true) => give_resolved_sha_group(tx, owner, recipient, skill, &sha, false),
                (true, false) => give_resolved_sha_group(tx, owner, recipient, skill, &shan, false),
                (false, false) => {
                    let mut next = vec![
                        1,
                        owner.raw(),
                        recipient.raw(),
                        skill.raw(),
                        sha.len() as u32,
                    ];
                    next.extend(sha.iter().map(|card| card.raw()));
                    next.push(shan.len() as u32);
                    next.extend(shan.iter().map(|card| card.raw()));
                    next.push(excluded.len() as u32);
                    next.extend(excluded.iter().map(|target| target.raw()));
                    crate::effect::open_choice(
                        tx,
                        owner,
                        1,
                        1,
                        false,
                        vec![1, 2],
                        engine_api::ChoiceKind::Custom,
                        ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha,
                        next,
                    )
                    .map(|_| ())
                }
            }
        }
        1 => {
            let owner = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let recipient = PlayerId::new(*payload.get(2).unwrap_or(&0));
            let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
            if cancelled || observer != owner || selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose the resolved Sha or Shan group".into(),
                ));
            }
            let (sha, shan, _) = resolved_sha_gift_groups(payload, 4)?;
            match selections[0] {
                1 => give_resolved_sha_group(tx, owner, recipient, skill, &sha, false),
                2 => give_resolved_sha_group(tx, owner, recipient, skill, &shan, false),
                _ => Err(EffectError::Illegal("unknown resolved card group".into())),
            }
        }
        2 => {
            let owner = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
            let skill = engine_api::SkillId::new(*payload.get(3).unwrap_or(&0));
            if observer != player {
                return Err(EffectError::Illegal(
                    "follow-up Sha choice belongs to another player".into(),
                ));
            }
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled follow-up Sha choice has selections".into(),
                    ));
                }
                return Ok(());
            }
            if selections.is_empty() {
                return Ok(());
            }
            if selections.len() != 1 {
                return Err(EffectError::Illegal(
                    "choose at most one follow-up Sha source".into(),
                ));
            }
            let card = CardId::new(selections[0]);
            let targets = crate::command::play::prompted_sha_targets(tx, owner, player, card);
            if targets.is_empty()
                || !crate::command::play::prompted_sha_source_cards(tx.state(), player)
                    .contains(&card)
            {
                return Err(EffectError::Illegal(
                    "follow-up Sha source is no longer legal".into(),
                ));
            }
            let max = targets
                .len()
                .min(crate::query::sha_target_limit_for(tx.state(), player) as usize)
                .min(u8::MAX as usize) as u8;
            crate::effect::open_choice(
                tx,
                player,
                1,
                max,
                true,
                targets.into_iter().map(PlayerId::raw).collect(),
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::ChooseGainResolvedShaOrShanThenOptionalUseSha,
                vec![3, owner.raw(), player.raw(), skill.raw(), card.raw()],
            )
            .map(|_| ())
        }
        3 => {
            let owner = PlayerId::new(*payload.get(1).unwrap_or(&0));
            let player = PlayerId::new(*payload.get(2).unwrap_or(&0));
            let card = CardId::new(*payload.get(4).unwrap_or(&0));
            if observer != player {
                return Err(EffectError::Illegal(
                    "follow-up Sha target choice belongs to another player".into(),
                ));
            }
            if cancelled || selections.is_empty() {
                return Ok(());
            }
            let targets: Vec<_> = selections.iter().copied().map(PlayerId::new).collect();
            crate::command::play::play_prompted_sha_source(tx, owner, player, card, &targets)
        }
        _ => Err(EffectError::EngineFault(
            "unknown resolved Sha gift continuation stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_other_targets_by_skipped_phase_count_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 {
        return Err(EffectError::EngineFault(
            "bad skipped-phase damage continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let amount = payload[1];
    let skill = engine_api::SkillId::new(payload[2]);
    let max = tx.state().turn.skipped_phase_count() as usize;
    if observer != player
        || cancelled
        || amount == 0
        || selections.is_empty()
        || selections.len() > max
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::End
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "skipped-phase damage choice context changed".into(),
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut targets = Vec::with_capacity(selections.len());
    for raw in selections {
        let target = PlayerId::new(*raw);
        if target == player
            || !seen.insert(*raw)
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "skipped-phase damage target is no longer legal".into(),
            ));
        }
        targets.push(target);
    }
    run_effects(
        tx,
        targets
            .into_iter()
            .map(|target| EffectRequest::DealDamage {
                source: player,
                target,
                amount,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            })
            .collect(),
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_other_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-other-damage continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let amount = payload[1];
    let skill = engine_api::SkillId::new(payload[2]);
    let target = PlayerId::new(selections[0]);
    if observer != player
        || cancelled
        || amount == 0
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
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "choose-other-damage context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: player,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_bound_source_attack_range_damage(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 4 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad bound-source attack-range damage continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let source = PlayerId::new(payload[1]);
    let amount = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    let target = PlayerId::new(selections[0]);
    let skill_matches = tx
        .state()
        .skills
        .get(skill)
        .filter(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        .and_then(|instance| instance.bound_target())
        == Some(source);
    if observer != player
        || cancelled
        || amount == 0
        || !skill_matches
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx
            .state()
            .players
            .get(source)
            .is_some_and(|participant| participant.alive)
        || target == source
        || !tx
            .state()
            .players
            .get(target)
            .is_some_and(|participant| participant.alive)
        || !crate::query::player_in_attack_range(tx.state(), source, target)
    {
        return Err(EffectError::Illegal(
            "bound-source attack-range damage context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_other_discard_target_card(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-other-discard-target-card continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let scope = u8::try_from(payload[1])
        .ok()
        .and_then(rule_api::TargetCardScope::from_tag)
        .ok_or_else(|| EffectError::EngineFault("bad target-card scope payload".into()))?;
    let skill = engine_api::SkillId::new(payload[2]);
    let target = PlayerId::new(selections[0]);
    if observer != player
        || cancelled
        || target == player
        || ![player, target].into_iter().all(|participant| {
            tx.state()
                .players
                .get(participant)
                .is_some_and(|state| state.alive)
        })
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || crate::query::discardable_target_cards(tx.state(), player, target, scope).is_empty()
    {
        return Err(EffectError::Illegal(
            "choose-other-discard-target-card context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::ChooseDiscardTargetCard {
            player,
            target,
            scope,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_gain_discard_card_by_kind(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-gain-discard-card continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let kind = u8::try_from(payload[1])
        .ok()
        .and_then(CardKind::from_tag)
        .ok_or_else(|| EffectError::EngineFault("bad card-kind payload".into()))?;
    let skill = engine_api::SkillId::new(payload[2]);
    let card = CardId::new(selections[0]);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
    if observer != player
        || cancelled
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card| card.zone == discard && card.kind == kind)
    {
        return Err(EffectError::Illegal(
            "choose-gain-discard-card context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: hand,
            from: Some(discard),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_gain_own_lost_discard_card_this_turn(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 2 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-gain-own-lost-card continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let card = CardId::new(selections[0]);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .ok_or_else(|| EffectError::EngineFault("shared discard pile missing".into()))?;
    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, player)
        .ok_or_else(|| EffectError::EngineFault("player hand missing".into()))?;
    let loss_count = tx.state().turn.card_loss_count(player);
    if observer != player
        || cancelled
        || tx.state().turn.current != player
        || !tx.state().players.get(player).is_some_and(|owner| {
            owner.alive && loss_count > 0 && loss_count <= owner.hp.max(0) as u32
        })
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .turn
            .lost_cards_for(player)
            .any(|lost| lost == card)
        || !tx
            .state()
            .cards
            .get(card)
            .is_some_and(|card| card.zone == discard)
    {
        return Err(EffectError::Illegal(
            "choose-gain-own-lost-card context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: hand,
            from: Some(discard),
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_move_field_card_to(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 3 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-move-field-card continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let target = PlayerId::new(payload[1]);
    let skill = engine_api::SkillId::new(payload[2]);
    let card = CardId::new(selections[0]);
    if observer != player
        || cancelled
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !crate::query::movable_field_cards_to(tx.state(), target).contains(&card)
    {
        return Err(EffectError::Illegal(
            "choose-move-field-card context changed".into(),
        ));
    }
    let card_state = tx
        .state()
        .cards
        .get(card)
        .ok_or_else(|| EffectError::Illegal("selected field card is missing".into()))?;
    let from = card_state.zone;
    let from_kind = tx
        .state()
        .zones
        .get(from)
        .map(|zone| zone.kind)
        .ok_or_else(|| EffectError::Illegal("selected field card zone is missing".into()))?;
    let to_kind = match from_kind {
        ZoneKind::Equip => ZoneKind::Equip,
        ZoneKind::Judge => ZoneKind::Judge,
        _ => {
            return Err(EffectError::Illegal(
                "selected card is no longer in a field zone".into(),
            ));
        }
    };
    let to = tx
        .state()
        .zones
        .find(to_kind, target)
        .ok_or_else(|| EffectError::Illegal("field-card destination zone is missing".into()))?;
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to,
            from: Some(from),
        }],
    )
    .map(|_| ())
}

pub(crate) fn decode_optional_hand_distribution_plan(
    payload: &[u32],
) -> Result<
    (
        u32,
        PlayerId,
        u32,
        engine_api::SkillId,
        Vec<CardId>,
        Vec<PlayerId>,
        &[u32],
    ),
    EffectError,
> {
    if payload.len() < 5 {
        return Err(EffectError::EngineFault(
            "truncated optional hand-distribution payload".into(),
        ));
    }
    let stage = payload[0];
    let player = PlayerId::new(payload[1]);
    let give_cap = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    let plan_len = payload[4] as usize;
    let plan_end = 5usize
        .checked_add(plan_len.saturating_mul(2))
        .ok_or_else(|| {
            EffectError::EngineFault("oversized optional hand-distribution payload".into())
        })?;
    if plan_end > payload.len() || plan_len > give_cap as usize {
        return Err(EffectError::EngineFault(
            "invalid optional hand-distribution plan length".into(),
        ));
    }
    let mut cards = Vec::with_capacity(plan_len);
    let mut targets = Vec::with_capacity(plan_len);
    for pair in payload[5..plan_end].chunks_exact(2) {
        cards.push(CardId::new(pair[0]));
        targets.push(PlayerId::new(pair[1]));
    }
    Ok((
        stage,
        player,
        give_cap,
        skill,
        cards,
        targets,
        &payload[plan_end..],
    ))
}

pub(crate) fn resolve_draw_then_optionally_distribute_hand_cards(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    let (stage, player, give_cap, skill, mut cards, mut targets, tail) =
        decode_optional_hand_distribution_plan(payload)?;
    let skill_enabled = tx.state().skills.get(skill).is_some_and(|instance| {
        instance.owner == player && tx.state().skill_effectively_enabled(instance)
    });
    if observer != player
        || give_cap == 0
        || give_cap > u8::MAX as u32
        || !skill_enabled
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "optional hand-distribution choice context changed".into(),
        ));
    }

    let current: std::collections::BTreeSet<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let mut planned = std::collections::BTreeSet::new();
    if !cards
        .iter()
        .all(|card| planned.insert(card.raw()) && current.contains(&card.raw()))
        || targets.iter().any(|target| {
            *target == player
                || !tx
                    .state()
                    .players
                    .get(*target)
                    .is_some_and(|participant| participant.alive)
        })
    {
        return Err(EffectError::Illegal(
            "optional hand-distribution plan became stale".into(),
        ));
    }

    match stage {
        0 if tail.is_empty() => {
            if cancelled {
                if !selections.is_empty() {
                    return Err(EffectError::Illegal(
                        "cancelled hand-distribution choice contains cards".into(),
                    ));
                }
                return run_effects(
                    tx,
                    vec![EffectRequest::DrawThenOptionallyDistributeHandCards {
                        player,
                        draw_count: 0,
                        give_cap,
                        cards,
                        targets,
                        skill,
                        ready_to_choose: true,
                        finish: true,
                    }],
                )
                .map(|_| ());
            }
            let remaining = give_cap as usize - cards.len();
            let mut selected = std::collections::BTreeSet::new();
            if selections.is_empty()
                || selections.len() > remaining
                || !selections.iter().all(|card| {
                    selected.insert(*card) && current.contains(card) && !planned.contains(card)
                })
            {
                return Err(EffectError::Illegal(
                    "optional hand-distribution card selection is invalid".into(),
                ));
            }
            let mut recipients: Vec<_> = tx
                .state()
                .players
                .iter_live()
                .filter(|target| target.alive && target.id != player)
                .map(|target| target.id.raw())
                .collect();
            recipients.sort_unstable();
            recipients.dedup();
            if recipients.is_empty() {
                return Err(EffectError::Illegal(
                    "optional hand distribution has no recipient".into(),
                ));
            }
            let mut next = vec![1, player.raw(), give_cap, skill.raw(), cards.len() as u32];
            for (card, target) in cards.iter().zip(&targets) {
                next.push(card.raw());
                next.push(target.raw());
            }
            next.push(selections.len() as u32);
            next.extend_from_slice(selections);
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                recipients,
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::DrawThenOptionallyDistributeHandCards,
                next,
            )
            .map(|_| ())
        }
        1 => {
            if cancelled || selections.len() != 1 || tail.is_empty() {
                return Err(EffectError::Illegal(
                    "optional hand-distribution recipient is required".into(),
                ));
            }
            let selected_len = tail[0] as usize;
            if selected_len == 0
                || tail.len() != selected_len + 1
                || selected_len > give_cap as usize - cards.len()
            {
                return Err(EffectError::EngineFault(
                    "invalid optional hand-distribution selected-card payload".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "optional hand-distribution recipient became stale".into(),
                ));
            }
            let mut selected = std::collections::BTreeSet::new();
            if !tail[1..].iter().all(|card| {
                selected.insert(*card) && current.contains(card) && !planned.contains(card)
            }) {
                return Err(EffectError::Illegal(
                    "optional hand-distribution cards became stale".into(),
                ));
            }
            for card in &tail[1..] {
                cards.push(CardId::new(*card));
                targets.push(target);
            }
            run_effects(
                tx,
                vec![EffectRequest::DrawThenOptionallyDistributeHandCards {
                    player,
                    draw_count: 0,
                    give_cap,
                    cards,
                    targets,
                    skill,
                    ready_to_choose: true,
                    finish: false,
                }],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown optional hand-distribution stage".into(),
        )),
    }
}

pub(crate) fn resolve_give_selected_then_continue_distinct_recover_at_two(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() < 5 {
        return Err(EffectError::EngineFault(
            "truncated distinct phase-gift payload".into(),
        ));
    }
    let stage = payload[0];
    let player = PlayerId::new(payload[1]);
    let total_given = payload[2];
    let skill = engine_api::SkillId::new(payload[3]);
    let used_len = payload[4] as usize;
    let used_end = 5usize
        .checked_add(used_len)
        .ok_or_else(|| EffectError::EngineFault("oversized distinct phase-gift payload".into()))?;
    if used_end > payload.len() {
        return Err(EffectError::EngineFault(
            "truncated distinct phase-gift target history".into(),
        ));
    }
    let used_targets: Vec<_> = payload[5..used_end]
        .iter()
        .copied()
        .map(PlayerId::new)
        .collect();
    let tail = &payload[used_end..];
    let mut distinct_targets = std::collections::BTreeSet::new();
    if observer != player
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || tx.state().turn.current != player
        || tx.state().turn.phase != Phase::Play
        || used_targets
            .iter()
            .any(|target| *target == player || !distinct_targets.insert(target.raw()))
    {
        return Err(EffectError::Illegal(
            "distinct phase-gift choice context changed".into(),
        ));
    }

    let current: std::collections::BTreeSet<_> = tx
        .state()
        .hand_cards(player)
        .into_iter()
        .map(CardId::raw)
        .collect();
    match stage {
        0 if tail.is_empty() => {
            if cancelled {
                if selections.is_empty() {
                    return Ok(());
                }
                return Err(EffectError::Illegal(
                    "cancelled distinct phase-gift choice contains cards".into(),
                ));
            }
            let mut selected = std::collections::BTreeSet::new();
            if selections.is_empty()
                || !selections
                    .iter()
                    .all(|card| selected.insert(*card) && current.contains(card))
            {
                return Err(EffectError::Illegal(
                    "distinct phase-gift card selection is invalid".into(),
                ));
            }
            let mut targets: Vec<_> = tx
                .state()
                .players
                .iter_live()
                .filter(|target| {
                    target.alive
                        && target.id != player
                        && !distinct_targets.contains(&target.id.raw())
                })
                .map(|target| target.id.raw())
                .collect();
            targets.sort_unstable();
            targets.dedup();
            if targets.is_empty() {
                return Err(EffectError::Illegal(
                    "distinct phase gift has no unused recipient".into(),
                ));
            }
            let mut next = vec![
                1,
                player.raw(),
                total_given,
                skill.raw(),
                used_targets.len() as u32,
            ];
            next.extend(used_targets.iter().map(|target| target.raw()));
            next.push(selections.len() as u32);
            next.extend_from_slice(selections);
            crate::effect::open_choice(
                tx,
                player,
                1,
                1,
                false,
                targets,
                engine_api::ChoiceKind::SelectTargets,
                ContinuationKind::GiveSelectedThenContinueDistinctRecoverAtTwo,
                next,
            )
            .map(|_| ())
        }
        1 => {
            if cancelled || selections.len() != 1 || tail.is_empty() {
                return Err(EffectError::Illegal(
                    "distinct phase-gift recipient is required".into(),
                ));
            }
            let selected_len = tail[0] as usize;
            if selected_len == 0 || tail.len() != selected_len + 1 {
                return Err(EffectError::EngineFault(
                    "invalid distinct phase-gift selected-card payload".into(),
                ));
            }
            let target = PlayerId::new(selections[0]);
            if target == player
                || distinct_targets.contains(&target.raw())
                || !tx
                    .state()
                    .players
                    .get(target)
                    .is_some_and(|participant| participant.alive)
            {
                return Err(EffectError::Illegal(
                    "distinct phase-gift recipient became stale".into(),
                ));
            }
            let mut selected = std::collections::BTreeSet::new();
            if !tail[1..]
                .iter()
                .all(|card| selected.insert(*card) && current.contains(card))
            {
                return Err(EffectError::Illegal(
                    "distinct phase-gift cards became stale".into(),
                ));
            }
            run_effects(
                tx,
                vec![
                    EffectRequest::GiveSelectedThenContinueDistinctRecoverAtTwo {
                        player,
                        target,
                        cards: tail[1..].iter().copied().map(CardId::new).collect(),
                        used_targets,
                        total_given,
                        skill,
                        continue_only: false,
                    },
                ],
            )
            .map(|_| ())
        }
        _ => Err(EffectError::EngineFault(
            "unknown distinct phase-gift stage".into(),
        )),
    }
}

pub(crate) fn resolve_choose_add_missing_skill(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 || selections.len() != 1 {
        return Err(EffectError::EngineFault(
            "bad choose-add-missing-skill continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let skill = engine_api::SkillId::new(payload[1]);
    let skill_defs = [
        engine_api::SkillDefId::new(payload[2]),
        engine_api::SkillDefId::new(payload[3]),
        engine_api::SkillDefId::new(payload[4]),
    ];
    if skill_defs.iter().any(|skill_def| skill_def.raw() == 0)
        || skill_defs[0] == skill_defs[1]
        || skill_defs[0] == skill_defs[2]
        || skill_defs[1] == skill_defs[2]
    {
        return Err(EffectError::EngineFault(
            "invalid choose-add-missing-skill definitions".into(),
        ));
    }
    let selected = engine_api::SkillDefId::new(selections[0]);
    let already_owned = tx
        .state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|participant| participant.skills.iter())
        .any(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def == selected)
        });
    if observer != player
        || cancelled
        || !skill_defs.contains(&selected)
        || already_owned
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
    {
        return Err(EffectError::Illegal(
            "choose-add-missing-skill context changed".into(),
        ));
    }
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: selected,
        }],
    )
    .map(|_| ())
}

pub(crate) fn resolve_choose_others_after_uniform_hand_reveal(
    tx: &mut Transaction,
    observer: PlayerId,
    payload: &[u32],
    selections: &[u32],
    cancelled: bool,
) -> Result<(), EffectError> {
    if payload.len() != 5 {
        return Err(EffectError::EngineFault(
            "bad uniform-hand draw continuation payload".into(),
        ));
    }
    let player = PlayerId::new(payload[0]);
    let max = payload[1] as usize;
    let skill = engine_api::SkillId::new(payload[2]);
    let current = PlayerId::new(payload[3]);
    let phase = Phase::from_u8(payload[4] as u8);
    if observer != player
        || max == 0
        || (cancelled && !selections.is_empty())
        || (!cancelled && selections.is_empty())
        || selections.len() > max
        || tx.state().turn.current != current
        || tx.state().turn.phase != phase
        || phase == Phase::Idle
        || !tx.state().skills.get(skill).is_some_and(|instance| {
            instance.owner == player && tx.state().skill_effectively_enabled(instance)
        })
        || !tx
            .state()
            .players
            .get(player)
            .is_some_and(|participant| participant.alive)
    {
        return Err(EffectError::Illegal(
            "uniform-hand draw choice context changed".into(),
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut selected = std::collections::BTreeSet::new();
    for raw in selections {
        let target = PlayerId::new(*raw);
        if target == player
            || !seen.insert(*raw)
            || !tx
                .state()
                .players
                .get(target)
                .is_some_and(|participant| participant.alive)
        {
            return Err(EffectError::Illegal(
                "uniform-hand draw target is no longer legal".into(),
            ));
        }
        selected.insert(target);
    }
    selected.insert(player);
    let draw_order: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|participant| selected.contains(participant))
        .collect();
    if draw_order.len() != selected.len() {
        return Err(EffectError::Illegal(
            "uniform-hand draw participant is no longer alive".into(),
        ));
    }
    run_effects(
        tx,
        draw_order
            .into_iter()
            .map(|participant| EffectRequest::DrawCardsFromSkill {
                player: participant,
                count: 1,
                skill,
            })
            .collect(),
    )
    .map(|_| ())
}
