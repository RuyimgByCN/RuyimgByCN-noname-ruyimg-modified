//! PA12 批⑭：目标选择、增减目标与分支交互原语（tags 129, 130, 132, 133, 136, 137, 139, 140, 142, 143, 144, 150, 153）。
use super::batch13_targets::{
    draw_or_recover_options, draw_target_options, is_wounded, other_alive_options, seats_from,
};
use super::*;

fn within_distance_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: u8,
    max_distance: u32,
) -> Vec<ChoiceOption> {
    seats_from(core, owner)
        .into_iter()
        .filter(|&p| {
            p != owner
                && core.player(p).alive
                && core.computed_distance(catalog, owner, p) <= max_distance
        })
        .map(ChoiceOption::Player)
        .collect()
}

fn missing_skill_options(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: u8,
    skill_defs: &[rule_api::SkillDefId; 3],
) -> Vec<ChoiceOption> {
    let mut out = Vec::new();
    for (idx, def_id) in skill_defs.iter().enumerate() {
        if !def_id.is_some() {
            continue;
        }
        let raw = def_id.raw();
        let already_has = if let Some(def) = crate::rvm_ops::resolve_skill_def(catalog, raw) {
            core.has_skill(owner, def)
        } else {
            core.vm_aux
                .flags
                .get(&(owner, format!("skill:{raw}")))
                .copied()
                .unwrap_or(0)
                > 0
        };
        if !already_has {
            out.push(ChoiceOption::Tag(idx as u8));
        }
    }
    out
}

fn max_mark_other_options(core: &GameStateCore, owner: u8, mark: u32) -> Vec<ChoiceOption> {
    let others: Vec<u8> = seats_from(core, owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive)
        .collect();
    let Some(max_val) = others
        .iter()
        .map(|&p| core.vm_aux.marks.get(&(p, mark)).copied().unwrap_or(0))
        .max()
    else {
        return Vec::new();
    };
    if max_val <= 0 {
        return Vec::new();
    }
    others
        .into_iter()
        .filter(|&p| core.vm_aux.marks.get(&(p, mark)).copied().unwrap_or(0) == max_val)
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn active_ask(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match effect {
        EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            ..
        } => {
            let owner = seat(*player, core)?;
            if state == [1] {
                let options = other_alive_options(core, owner);
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    "rvm_choice",
                    ChoiceKind::SelectPlayers,
                    options,
                    1,
                    1,
                    false,
                ))
            } else if state.len() == 2 && state[0] == 2 && (state[1] as usize) < core.players.len()
            {
                let target = state[1] as u8;
                let mult = (*times).max(1);
                let options = draw_or_recover_options(
                    core,
                    target,
                    draw_count.saturating_mul(mult),
                    recover_count.saturating_mul(mult),
                );
                if options.is_empty() {
                    return None;
                }
                Some(ask(
                    owner,
                    "rvm_choice",
                    ChoiceKind::PickTag,
                    options,
                    1,
                    1,
                    false,
                ))
            } else {
                None
            }
        }
        EffectRequest::ChooseAddOrRemoveCurrentCardTarget { player, .. }
        | EffectRequest::ChooseAddCurrentShaTarget { player, .. }
            if state == [1] =>
        {
            let owner = seat(*player, core)?;
            let options = other_alive_options(core, owner);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                1,
                1,
                true,
            ))
        }
        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            ..
        } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = draw_target_options(core, owner, *scope);
            if options.is_empty() {
                return None;
            }
            let min = (*min_count as usize).max(1).min(options.len());
            let max = (*max_count as usize).max(min).min(options.len());
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                min,
                max,
                *min_count == 0,
            ))
        }
        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            min_count,
            max_count,
            scope,
            ..
        } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = draw_target_options(core, owner, *scope);
            if options.is_empty() {
                return None;
            }
            let min = (*min_count as usize).max(1).min(options.len());
            let max = (*max_count as usize).max(min).min(options.len());
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                min,
                max,
                true,
            ))
        }
        EffectRequest::ChooseTargetsDraw {
            player,
            target_count,
            scope,
            ..
        } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = draw_target_options(core, owner, *scope);
            if options.is_empty() {
                return None;
            }
            let max = (*target_count as usize).max(1).min(options.len());
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                1,
                max,
                false,
            ))
        }
        EffectRequest::ChooseLinkOther { player, .. }
        | EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, .. }
            if state == [1] =>
        {
            let owner = seat(*player, core)?;
            let options = other_alive_options(core, owner);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                1,
                1,
                false,
            ))
        }
        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            max_distance,
            ..
        } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = within_distance_options(core, catalog, owner, *max_distance);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                1,
                1,
                true,
            ))
        }
        EffectRequest::ChooseRecoverTarget {
            chooser, target, ..
        } if state == [1] => {
            let c = seat(*chooser, core)?;
            let t = seat(*target, core)?;
            if !core.player(c).alive || !core.player(t).alive || !is_wounded(core, t) {
                return None;
            }
            Some(ask(
                c,
                "rvm_choice",
                ChoiceKind::Confirm,
                vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                1,
                1,
                true,
            ))
        }
        EffectRequest::ChooseTargetDrawToMaxHpCap { player, .. } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = alive_options(core);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                1,
                1,
                false,
            ))
        }
        EffectRequest::ChooseAddMissingSkill {
            player, skill_defs, ..
        } if state == [1] => {
            let owner = seat(*player, core)?;
            let options = missing_skill_options(core, catalog, owner, skill_defs);
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::PickTag,
                options,
                1,
                1,
                false,
            ))
        }
        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { player, mark, .. }
            if state == [1] =>
        {
            let owner = seat(*player, core)?;
            let options = max_mark_other_options(core, owner, mark.raw());
            if options.is_empty() {
                return None;
            }
            Some(ask(
                owner,
                "rvm_choice",
                ChoiceKind::SelectPlayers,
                options,
                1,
                1,
                false,
            ))
        }
        _ => None,
    }
}

pub(super) fn step(
    ctx: &mut Ctx<'_>,
    skill: &str,
    effect: &EffectRequest,
    state: &mut Vec<u32>,
) -> Option<Flow> {
    match effect {
        EffectRequest::ChooseOtherDrawOrRecover {
            player,
            draw_count,
            recover_count,
            times,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_other_draw_or_recover(
                ctx,
                owner,
                *draw_count,
                *recover_count,
                *times,
                state,
            ))
        }
        EffectRequest::ChooseAddOrRemoveCurrentCardTarget { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_toggle_card_target(ctx, owner, true, state))
        }
        EffectRequest::ChooseAddCurrentShaTarget { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_toggle_card_target(ctx, owner, false, state))
        }
        EffectRequest::ChooseTargetsDamage {
            player,
            min_count,
            max_count,
            scope,
            amount,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_targets_damage(
                ctx, owner, *min_count, *max_count, *scope, *amount, state,
            ))
        }
        EffectRequest::ChooseOptionalTargetsDraw {
            player,
            count,
            min_count,
            max_count,
            scope,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_targets_draw(
                ctx, owner, *count, *min_count, *max_count, *scope, true, state,
            ))
        }
        EffectRequest::ChooseTargetsDraw {
            player,
            count,
            target_count,
            scope,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_targets_draw(
                ctx,
                owner,
                *count,
                1,
                *target_count,
                *scope,
                false,
                state,
            ))
        }
        EffectRequest::ChooseLinkOther { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_link_other(ctx, owner, state))
        }
        EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_other_draw_lost_hp_turnover(ctx, owner, state))
        }
        EffectRequest::ChooseAddCurrentShaTargetWithinDistance {
            player,
            max_distance,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_add_sha_target_within_distance(
                ctx,
                owner,
                *max_distance,
                state,
            ))
        }
        EffectRequest::ChooseRecoverTarget {
            chooser,
            target,
            amount,
        } => {
            let c = seat(*chooser, ctx.core)?;
            let t = seat(*target, ctx.core)?;
            Some(step_recover_target(ctx, c, t, *amount, state))
        }
        EffectRequest::ChooseTargetDrawToMaxHpCap { player, cap, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_target_draw_to_cap(ctx, owner, *cap, state))
        }
        EffectRequest::ChooseAddMissingSkill {
            player, skill_defs, ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_add_missing_skill(ctx, owner, skill_defs, state))
        }
        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { player, mark, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_max_mark_other_judge(
                ctx,
                owner,
                mark.raw(),
                skill,
                state,
            ))
        }
        _ => None,
    }
}

fn step_other_draw_or_recover(
    ctx: &mut Ctx<'_>,
    owner: u8,
    draw_count: u32,
    recover_count: u32,
    times: u32,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    let mult = times.max(1);
    let draw_total = draw_count.saturating_mul(mult);
    let recover_total = recover_count.saturating_mul(mult);
    if state.is_empty() {
        let options = other_alive_options(ctx.core, owner);
        if options.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            options,
            1,
            1,
            false,
        ));
    }
    if state == &[1] {
        let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
            return Flow::Done;
        };
        if target == owner || !ctx.core.player(target).alive {
            return Flow::Done;
        }
        let can_recover = recover_total > 0 && is_wounded(ctx.core, target);
        if can_recover && draw_total > 0 {
            *state = vec![2, target as u32];
            let options = draw_or_recover_options(ctx.core, target, draw_total, recover_total);
            return wait(ask(
                owner,
                "rvm_choice",
                ChoiceKind::PickTag,
                options,
                1,
                1,
                false,
            ));
        }
        if can_recover {
            ctx.recover(target, recover_total, None);
        } else if draw_total > 0 {
            ctx.start_draw(target, draw_total);
        }
        return Flow::Done;
    }
    if state.len() == 2 && state[0] == 2 {
        let target = state[1] as u8;
        if !ctx.core.player(target).alive {
            return Flow::Done;
        }
        match ctx.answer().first().copied() {
            Some(ChoiceOption::Tag(0)) if draw_total > 0 => {
                ctx.start_draw(target, draw_total);
            }
            Some(ChoiceOption::Tag(1)) if recover_total > 0 && is_wounded(ctx.core, target) => {
                ctx.recover(target, recover_total, None);
            }
            _ => {}
        }
    }
    Flow::Done
}

fn step_toggle_card_target(
    ctx: &mut Ctx<'_>,
    owner: u8,
    allow_remove: bool,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = other_alive_options(ctx.core, owner);
        if options.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            options,
            1,
            1,
            true,
        ));
    }
    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if target != owner && ctx.core.player(target).alive {
        for frame in ctx.res.stack.iter_mut().rev() {
            if let EventData::CardUsed(u) = &mut frame.data {
                if u.targets.contains(&target) {
                    if allow_remove && u.targets.len() > 1 {
                        u.targets.retain(|&p| p != target);
                    }
                } else {
                    u.targets.push(target);
                }
                break;
            }
        }
    }
    Flow::Done
}

fn step_targets_damage(
    ctx: &mut Ctx<'_>,
    owner: u8,
    min_count: u8,
    max_count: u8,
    scope: rule_api::DrawTargetScope,
    amount: u32,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive || amount == 0 {
        return Flow::Done;
    }
    let valid = draw_target_options(ctx.core, owner, scope);
    if state.is_empty() {
        if valid.is_empty() {
            return Flow::Done;
        }
        let min = (min_count as usize).max(1).min(valid.len());
        let max = (max_count as usize).max(min).min(valid.len());
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            valid,
            min,
            max,
            min_count == 0,
        ));
    }
    let mut chosen = Vec::new();
    for opt in ctx.answer() {
        if let ChoiceOption::Player(p) = *opt {
            if valid.contains(&ChoiceOption::Player(p)) && !chosen.contains(&p) {
                chosen.push(p);
            }
        }
    }
    for target in chosen.into_iter().rev() {
        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
            Some(owner),
            target,
            amount as i32,
            crate::event::DamageNature::Normal,
            None,
        )));
    }
    Flow::Done
}

#[allow(clippy::too_many_arguments)]
fn step_targets_draw(
    ctx: &mut Ctx<'_>,
    owner: u8,
    count: u32,
    min_count: u8,
    max_count: u8,
    scope: rule_api::DrawTargetScope,
    allow_cancel: bool,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive || count == 0 {
        return Flow::Done;
    }
    let valid = draw_target_options(ctx.core, owner, scope);
    if state.is_empty() {
        if valid.is_empty() {
            return Flow::Done;
        }
        let min = (min_count as usize).max(1).min(valid.len());
        let max = (max_count as usize).max(min).min(valid.len());
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            valid,
            min,
            max,
            allow_cancel,
        ));
    }
    let mut chosen = Vec::new();
    for opt in ctx.answer() {
        if let ChoiceOption::Player(p) = *opt {
            if valid.contains(&ChoiceOption::Player(p)) && !chosen.contains(&p) {
                chosen.push(p);
            }
        }
    }
    for target in chosen.into_iter().rev() {
        ctx.start_draw(target, count);
    }
    Flow::Done
}

fn step_link_other(ctx: &mut Ctx<'_>, owner: u8, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = other_alive_options(ctx.core, owner);
        if options.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            options,
            1,
            1,
            false,
        ));
    }
    if let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() {
        if target != owner && ctx.core.player(target).alive {
            let next_linked = !ctx.core.player(target).linked;
            ctx.core.player_mut(target).linked = next_linked;
            ctx.emit(EventData::Linked {
                player: target,
                linked: next_linked,
            });
        }
    }
    Flow::Done
}

fn step_other_draw_lost_hp_turnover(ctx: &mut Ctx<'_>, owner: u8, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = other_alive_options(ctx.core, owner);
        if options.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            options,
            1,
            1,
            false,
        ));
    }
    if let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() {
        if target != owner && ctx.core.player(target).alive {
            let lost =
                (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as u32;
            ctx.emit(EventData::TurnOverBefore {
                player: owner,
                cancelled: false,
            });
            if lost > 0 {
                ctx.start_draw(target, lost);
            }
        }
    }
    Flow::Done
}

fn step_add_sha_target_within_distance(
    ctx: &mut Ctx<'_>,
    owner: u8,
    max_distance: u32,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    let valid = within_distance_options(ctx.core, ctx.catalog, owner, max_distance);
    if state.is_empty() {
        if valid.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            valid,
            1,
            1,
            true,
        ));
    }
    if let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() {
        if valid.contains(&ChoiceOption::Player(target)) {
            for frame in ctx.res.stack.iter_mut().rev() {
                if let EventData::CardUsed(u) = &mut frame.data {
                    if !u.targets.contains(&target) {
                        u.targets.push(target);
                    }
                    break;
                }
            }
        }
    }
    Flow::Done
}

fn step_recover_target(
    ctx: &mut Ctx<'_>,
    chooser: u8,
    target: u8,
    amount: u32,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(chooser).alive || !ctx.core.player(target).alive || amount == 0 {
        return Flow::Done;
    }
    if state.is_empty() {
        if !is_wounded(ctx.core, target) {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            chooser,
            "rvm_choice",
            ChoiceKind::Confirm,
            vec![ChoiceOption::Confirm, ChoiceOption::Pass],
            1,
            1,
            true,
        ));
    }
    if matches!(ctx.answer().first().copied(), Some(ChoiceOption::Confirm))
        && is_wounded(ctx.core, target)
    {
        ctx.recover(target, amount, None);
    }
    Flow::Done
}

fn step_target_draw_to_cap(ctx: &mut Ctx<'_>, owner: u8, cap: u32, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = alive_options(ctx.core);
        if options.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            options,
            1,
            1,
            false,
        ));
    }
    if let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() {
        if ctx.core.player(target).alive {
            let goal = ctx.core.player(target).max_hp.min(cap) as usize;
            let cur = ctx.core.player(target).hand.len();
            if cur < goal {
                ctx.start_draw(target, (goal - cur) as u32);
            }
        }
    }
    Flow::Done
}

fn step_add_missing_skill(
    ctx: &mut Ctx<'_>,
    owner: u8,
    skill_defs: &[rule_api::SkillDefId; 3],
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    let valid = missing_skill_options(ctx.core, ctx.catalog, owner, skill_defs);
    if state.is_empty() {
        if valid.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::PickTag,
            valid,
            1,
            1,
            false,
        ));
    }
    if let Some(ChoiceOption::Tag(idx)) = ctx.answer().first().copied() {
        let i = idx as usize;
        if i < 3 && skill_defs[i].is_some() {
            let raw = skill_defs[i].raw();
            if let Some(def) = crate::rvm_ops::resolve_skill_def(ctx.catalog, raw) {
                ctx.ensure_skill(
                    (owner, def),
                    crate::state::SkillInstance {
                        owner,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            } else {
                ctx.core
                    .vm_aux
                    .flags
                    .insert((owner, format!("skill:{raw}")), 1);
            }
        }
    }
    Flow::Done
}

fn step_max_mark_other_judge(
    ctx: &mut Ctx<'_>,
    owner: u8,
    mark: u32,
    _skill: &str,
    state: &mut Vec<u32>,
) -> Flow {
    let valid = max_mark_other_options(ctx.core, owner, mark);
    if state.is_empty() {
        if valid.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::SelectPlayers,
            valid,
            1,
            1,
            false,
        ));
    }
    if let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() {
        if valid.contains(&ChoiceOption::Player(target)) && ctx.core.player(target).alive {
            let mut saved = false;
            if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
                let jc = ctx.draw_judge_card();
                saved = matches!(
                    ctx.catalog.card_kind(jc),
                    crate::catalog::CardKind::Tao | crate::catalog::CardKind::Taoyuan
                );
                ctx.to_discard(jc);
            }
            if !saved {
                ctx.emit(EventData::Death {
                    player: target,
                    killer: None,
                });
            }
        }
    }
    Flow::Done
}

pub(super) fn validate(effect: &EffectRequest, state: &[u32], core: &GameStateCore) -> bool {
    let seats = core.players.len();
    match effect {
        EffectRequest::ChooseOtherDrawOrRecover { player, .. } => {
            player.raw() > 0
                && (player.raw() as usize) <= seats
                && (state == [1]
                    || (state.len() == 2 && state[0] == 2 && (state[1] as usize) < seats))
        }
        EffectRequest::ChooseRecoverTarget {
            chooser, target, ..
        } => {
            chooser.raw() > 0
                && (chooser.raw() as usize) <= seats
                && target.raw() > 0
                && (target.raw() as usize) <= seats
                && state == [1]
        }
        EffectRequest::ChooseAddOrRemoveCurrentCardTarget { player, .. }
        | EffectRequest::ChooseTargetsDamage { player, .. }
        | EffectRequest::ChooseAddCurrentShaTarget { player, .. }
        | EffectRequest::ChooseOptionalTargetsDraw { player, .. }
        | EffectRequest::ChooseTargetsDraw { player, .. }
        | EffectRequest::ChooseLinkOther { player, .. }
        | EffectRequest::ChooseOtherDrawOwnerLostHpTurnOver { player, .. }
        | EffectRequest::ChooseAddCurrentShaTargetWithinDistance { player, .. }
        | EffectRequest::ChooseTargetDrawToMaxHpCap { player, .. }
        | EffectRequest::ChooseAddMissingSkill { player, .. }
        | EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan { player, .. } => {
            player.raw() > 0 && (player.raw() as usize) <= seats && state == [1]
        }
        _ => false,
    }
}
