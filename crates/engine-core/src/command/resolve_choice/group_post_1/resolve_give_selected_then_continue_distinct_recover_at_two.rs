//! Command operations for `resolve_give_selected_then_continue_distinct_recover_at_two` and related helpers.

use super::*;

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
