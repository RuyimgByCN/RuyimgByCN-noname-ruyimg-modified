//! PA12 批⑬：目标选择与分支交互原语（tags 109..=114）。
use super::*;

pub(super) fn seats_from(core: &GameStateCore, start: u8) -> Vec<u8> {
    let n = core.players.len();
    (0..n).map(|i| ((start as usize + i) % n) as u8).collect()
}

pub(super) fn is_wounded(core: &GameStateCore, p: u8) -> bool {
    let ps = core.player(p);
    ps.hp < ps.max_hp as i32
}

pub(super) fn other_alive_options(core: &GameStateCore, owner: u8) -> Vec<ChoiceOption> {
    seats_from(core, owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive)
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn draw_target_options(
    core: &GameStateCore,
    owner: u8,
    scope: rule_api::DrawTargetScope,
) -> Vec<ChoiceOption> {
    match scope {
        rule_api::DrawTargetScope::Other => other_alive_options(core, owner),
        rule_api::DrawTargetScope::MaleOther => seats_from(core, owner)
            .into_iter()
            .filter(|&p| {
                p != owner
                    && core.player(p).alive
                    && core.effective_gender(p)
                        == engine_native_kernel::vocabulary::generals::Gender::Male
            })
            .map(ChoiceOption::Player)
            .collect(),
        rule_api::DrawTargetScope::FemaleOther => seats_from(core, owner)
            .into_iter()
            .filter(|&p| {
                p != owner
                    && core.player(p).alive
                    && core.effective_gender(p)
                        == engine_native_kernel::vocabulary::generals::Gender::Female
            })
            .map(ChoiceOption::Player)
            .collect(),
        rule_api::DrawTargetScope::HandLteChooser => {
            let my_hand = core.player(owner).hand.len();
            seats_from(core, owner)
                .into_iter()
                .filter(|&p| {
                    p != owner && core.player(p).alive && core.player(p).hand.len() <= my_hand
                })
                .map(ChoiceOption::Player)
                .collect()
        }
        rule_api::DrawTargetScope::Any => seats_from(core, owner)
            .into_iter()
            .filter(|&p| core.player(p).alive)
            .map(ChoiceOption::Player)
            .collect(),
        rule_api::DrawTargetScope::MinHand => {
            let Some(min_h) = core
                .players
                .iter()
                .filter(|ps| ps.alive)
                .map(|ps| ps.hand.len())
                .min()
            else {
                return Vec::new();
            };
            seats_from(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && core.player(p).hand.len() == min_h)
                .map(ChoiceOption::Player)
                .collect()
        }
        rule_api::DrawTargetScope::SameHp => {
            let my_hp = core.player(owner).hp;
            seats_from(core, owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && core.player(p).hp == my_hp)
                .map(ChoiceOption::Player)
                .collect()
        }
        rule_api::DrawTargetScope::HandEmpty => seats_from(core, owner)
            .into_iter()
            .filter(|&p| core.player(p).alive && core.player(p).hand.is_empty())
            .map(ChoiceOption::Player)
            .collect(),
        rule_api::DrawTargetScope::Wounded => seats_from(core, owner)
            .into_iter()
            .filter(|&p| core.player(p).alive && is_wounded(core, p))
            .map(ChoiceOption::Player)
            .collect(),
    }
}

pub(super) fn other_wounded_options(core: &GameStateCore, owner: u8) -> Vec<ChoiceOption> {
    seats_from(core, owner)
        .into_iter()
        .filter(|&p| p != owner && core.player(p).alive && is_wounded(core, p))
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn draw_or_recover_options(
    core: &GameStateCore,
    owner: u8,
    draw_count: u32,
    recover_count: u32,
) -> Vec<ChoiceOption> {
    let mut opts = Vec::new();
    if draw_count > 0 {
        opts.push(ChoiceOption::Tag(0));
    }
    if recover_count > 0 && is_wounded(core, owner) {
        opts.push(ChoiceOption::Tag(1));
    }
    opts
}

pub(super) fn active_ask(
    effect: &EffectRequest,
    state: &[u32],
    core: &GameStateCore,
) -> Option<ExpectedAsk> {
    if state != [1] {
        return None;
    }
    match effect {
        EffectRequest::ChooseOtherDamage { player, .. }
        | EffectRequest::ChooseOtherAddMark { player, .. }
        | EffectRequest::ResolveHfjieying { player, .. } => {
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
        EffectRequest::ChooseTargetDraw { player, scope, .. } => {
            let owner = seat(*player, core)?;
            let options = draw_target_options(core, owner, *scope);
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
        EffectRequest::ChooseOtherRecover { player, .. } => {
            let owner = seat(*player, core)?;
            let options = other_wounded_options(core, owner);
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
        EffectRequest::ChooseDrawOrRecover {
            player,
            draw_count,
            recover_count,
            allow_cancel,
        } => {
            let owner = seat(*player, core)?;
            let options = draw_or_recover_options(core, owner, *draw_count, *recover_count);
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
                *allow_cancel,
            ))
        }
        _ => None,
    }
}

pub(super) fn step(
    ctx: &mut Ctx<'_>,
    effect: &EffectRequest,
    state: &mut Vec<u32>,
) -> Option<Flow> {
    match effect {
        EffectRequest::ChooseOtherDamage { player, amount, .. } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_other_damage(ctx, owner, *amount, state))
        }
        EffectRequest::ChooseTargetDraw {
            player,
            count,
            scope,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_target_draw(ctx, owner, *count, *scope, state))
        }
        EffectRequest::ChooseOtherRecover { player, amount } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_other_recover(ctx, owner, *amount, state))
        }
        EffectRequest::ChooseDrawOrRecover {
            player,
            draw_count,
            recover_count,
            allow_cancel,
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_draw_or_recover(
                ctx,
                owner,
                *draw_count,
                *recover_count,
                *allow_cancel,
                state,
            ))
        }
        EffectRequest::ChooseOtherAddMark {
            player,
            mark,
            delta,
            ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_other_add_mark(ctx, owner, *mark, *delta, state))
        }
        EffectRequest::ResolveHfjieying {
            player, grant_def, ..
        } => {
            let owner = seat(*player, ctx.core)?;
            Some(step_hfjieying(ctx, owner, grant_def.raw(), state))
        }
        _ => None,
    }
}

fn step_other_damage(ctx: &mut Ctx<'_>, owner: u8, amount: u32, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive || amount == 0 {
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
    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if target != owner && ctx.core.player(target).alive {
        *ctx.core
            .vm_aux
            .flags
            .entry((owner, "lifetime".to_string()))
            .or_insert(0) += amount;
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

fn step_target_draw(
    ctx: &mut Ctx<'_>,
    owner: u8,
    count: u32,
    scope: rule_api::DrawTargetScope,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive || count == 0 {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = draw_target_options(ctx.core, owner, scope);
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
    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if draw_target_options(ctx.core, owner, scope).contains(&ChoiceOption::Player(target)) {
        ctx.start_draw(target, count);
    }
    Flow::Done
}

fn step_other_recover(ctx: &mut Ctx<'_>, owner: u8, amount: u32, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive || amount == 0 {
        return Flow::Done;
    }
    if state.is_empty() {
        let options = other_wounded_options(ctx.core, owner);
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
    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if target != owner && ctx.core.player(target).alive && is_wounded(ctx.core, target) {
        ctx.recover(target, amount, None);
    }
    Flow::Done
}

fn step_draw_or_recover(
    ctx: &mut Ctx<'_>,
    owner: u8,
    draw_count: u32,
    recover_count: u32,
    allow_cancel: bool,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive {
        return Flow::Done;
    }
    if state.is_empty() {
        let can_recover = recover_count > 0 && is_wounded(ctx.core, owner);
        if !can_recover && !allow_cancel {
            if draw_count > 0 {
                ctx.start_draw(owner, draw_count);
            }
            return Flow::Done;
        }
        let options = draw_or_recover_options(ctx.core, owner, draw_count, recover_count);
        if options.is_empty() {
            return Flow::Done;
        }
        *state = vec![1];
        return wait(ask(
            owner,
            "rvm_choice",
            ChoiceKind::PickTag,
            options,
            1,
            1,
            allow_cancel,
        ));
    }
    match ctx.answer().first().copied() {
        Some(ChoiceOption::Tag(0)) if draw_count > 0 => {
            ctx.start_draw(owner, draw_count);
        }
        Some(ChoiceOption::Tag(1)) if recover_count > 0 && is_wounded(ctx.core, owner) => {
            ctx.recover(owner, recover_count, None);
        }
        _ => {}
    }
    Flow::Done
}

fn step_other_add_mark(
    ctx: &mut Ctx<'_>,
    owner: u8,
    mark: u32,
    delta: i32,
    state: &mut Vec<u32>,
) -> Flow {
    if !ctx.core.player(owner).alive || mark == 0 || delta == 0 {
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
    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if target != owner && ctx.core.player(target).alive {
        let prev = ctx
            .core
            .vm_aux
            .marks
            .get(&(target, mark))
            .copied()
            .unwrap_or(0);
        let entry = ctx.core.vm_aux.marks.entry((target, mark)).or_insert(0);
        *entry += delta;
        if *entry <= 0 {
            ctx.core.vm_aux.marks.remove(&(target, mark));
        }
        if delta < 0 && prev > 0 && mark == 3_304_378_096 {
            ctx.core
                .vm_aux
                .flags
                .insert((target, "dcdichou_mark_removed_this_turn".to_string()), 1);
        }
    }
    Flow::Done
}

fn step_hfjieying(ctx: &mut Ctx<'_>, owner: u8, grant_def: u32, state: &mut Vec<u32>) -> Flow {
    if !ctx.core.player(owner).alive || grant_def == 0 {
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
    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
        return Flow::Done;
    };
    if target != owner && ctx.core.player(target).alive {
        if let Some(def) = crate::rvm_ops::resolve_skill_def(ctx.catalog, grant_def) {
            ctx.ensure_skill(
                (target, def),
                crate::state::SkillInstance {
                    owner: target,
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
                .insert((target, format!("skill:{grant_def}")), 1);
        }
    }
    Flow::Done
}

pub(super) fn validate(effect: &EffectRequest, state: &[u32], core: &GameStateCore) -> bool {
    let seats = core.players.len();
    let player = match effect {
        EffectRequest::ChooseOtherDamage { player, .. }
        | EffectRequest::ChooseTargetDraw { player, .. }
        | EffectRequest::ChooseOtherRecover { player, .. }
        | EffectRequest::ChooseDrawOrRecover { player, .. }
        | EffectRequest::ChooseOtherAddMark { player, .. }
        | EffectRequest::ResolveHfjieying { player, .. } => *player,
        _ => return false,
    };
    player.raw() > 0 && (player.raw() as usize) <= seats && state == [1]
}
