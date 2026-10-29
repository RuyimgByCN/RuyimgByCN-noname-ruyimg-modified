//! Query operations for `bootstrap_modifier_defs` and related helpers.

use super::*;

/// Bootstrap modifier defs.
pub fn bootstrap_modifier_defs() -> Vec<ModifierDefinition> {
    use crate::trigger::{BOOTSTRAP_MASHU, BOOTSTRAP_QICAO};
    vec![
        ModifierDefinition {
            id: 1,
            skill_def: SkillDefId::new(BOOTSTRAP_MASHU),
            target: ModifierTarget::DistanceFrom,
            mode: CombineMode::Add,
            value_source: None,
            value: -1, // 马术: 计算距离时 -1
            order: 0,
            label_key: "skill.mashu".into(),
        },
        ModifierDefinition {
            id: 2,
            skill_def: SkillDefId::new(BOOTSTRAP_QICAO),
            target: ModifierTarget::AttackRange,
            mode: CombineMode::Add,
            value_source: None,
            value: 1,
            order: 0,
            label_key: "skill.qicao_range".into(),
        },
    ]
}

pub(crate) fn mode_rank(m: CombineMode) -> u8 {
    match m {
        CombineMode::Override => 0,
        CombineMode::Add => 1,
        CombineMode::Multiply => 2,
        CombineMode::Min => 3,
        CombineMode::Max => 4,
        CombineMode::Veto => 5,
    }
}

/// Apply modifiers to a base integer; return final value + sources.
pub fn apply_int_modifiers(
    base: i32,
    mods: &[&ActiveModifier],
    owner_filter: Option<PlayerId>,
) -> (i32, Vec<ModifierSource>, bool) {
    apply_int_modifiers_by(base, mods, owner_filter, |modifier| modifier.value)
}

/// Apply modifiers after resolving query-time values from the current state.
pub fn apply_int_modifiers_for_state(
    state: &GameState,
    base: i32,
    mods: &[&ActiveModifier],
    owner_filter: Option<PlayerId>,
) -> (i32, Vec<ModifierSource>, bool) {
    apply_int_modifiers_for_state_with_other(state, base, mods, owner_filter, None)
}

/// Apply modifiers with the other participant of a contextual query.
pub fn apply_int_modifiers_for_state_with_other(
    state: &GameState,
    base: i32,
    mods: &[&ActiveModifier],
    owner_filter: Option<PlayerId>,
    other: Option<PlayerId>,
) -> (i32, Vec<ModifierSource>, bool) {
    apply_int_modifiers_for_state_with_context(state, base, mods, owner_filter, None, other)
}

/// Apply modifiers with an explicit queried subject and optional other participant.
pub fn apply_int_modifiers_for_state_with_context(
    state: &GameState,
    base: i32,
    mods: &[&ActiveModifier],
    owner_filter: Option<PlayerId>,
    queried: Option<PlayerId>,
    other: Option<PlayerId>,
) -> (i32, Vec<ModifierSource>, bool) {
    apply_int_modifiers_by(base, mods, owner_filter, |modifier| {
        resolved_value_with_context(state, modifier, queried, other)
    })
}

pub(crate) fn apply_int_modifiers_by(
    base: i32,
    mods: &[&ActiveModifier],
    owner_filter: Option<PlayerId>,
    value_of: impl Fn(&ActiveModifier) -> i32,
) -> (i32, Vec<ModifierSource>, bool) {
    let mut value = base;
    let mut sources = Vec::new();
    let mut vetoed = false;

    // Pass 1: overrides (last override wins among filtered)
    for m in mods.iter().filter(|m| m.mode == CombineMode::Override) {
        if let Some(o) = owner_filter {
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        value = contribution;
        sources.push(src(m, contribution));
    }

    // Pass 2: add
    for m in mods.iter().filter(|m| m.mode == CombineMode::Add) {
        if let Some(o) = owner_filter {
            // DistanceFrom: only owner's outgoing distance mods
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        value = value.saturating_add(contribution);
        sources.push(src(m, contribution));
    }

    // Pass 3: multiply
    for m in mods.iter().filter(|m| m.mode == CombineMode::Multiply) {
        if let Some(o) = owner_filter {
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        value = value.saturating_mul(contribution);
        sources.push(src(m, contribution));
    }

    // Pass 4: min / max
    for m in mods.iter().filter(|m| m.mode == CombineMode::Min) {
        if let Some(o) = owner_filter {
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        value = value.min(contribution);
        sources.push(src(m, contribution));
    }
    for m in mods.iter().filter(|m| m.mode == CombineMode::Max) {
        if let Some(o) = owner_filter {
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        value = value.max(contribution);
        sources.push(src(m, contribution));
    }

    // Pass 5: veto
    for m in mods.iter().filter(|m| m.mode == CombineMode::Veto) {
        if let Some(o) = owner_filter {
            if m.owner != o {
                continue;
            }
        }
        let contribution = value_of(m);
        if contribution != 0 {
            vetoed = true;
            sources.push(src(m, contribution));
        }
    }

    (value, sources, vetoed)
}

/// Apply distance modifiers and return an optional final distance override.
pub fn apply_distance_modifiers(
    state: &GameState,
    base: i32,
    from: PlayerId,
    to: PlayerId,
) -> (i32, Vec<ModifierSource>, Option<i32>) {
    let mut sources = Vec::new();
    let mut value = base;

    let from_mods = state.modifiers.for_target(ModifierTarget::DistanceFrom);
    for m in from_mods {
        if m.owner != from {
            continue;
        }
        if m.mode == CombineMode::Add {
            let contribution = resolved_value(state, m);
            value += contribution;
            sources.push(src(m, contribution));
        }
    }
    let to_mods = state.modifiers.for_target(ModifierTarget::DistanceTo);
    for m in to_mods {
        if m.owner != to {
            continue;
        }
        if m.mode == CombineMode::Add {
            let contribution = resolved_value(state, m);
            value += contribution;
            sources.push(src(m, contribution));
        }
    }
    let global_to_mods = state.modifiers.for_target(ModifierTarget::GlobalDistanceTo);
    for m in global_to_mods {
        if !state.players.get(m.owner).is_some_and(|owner| owner.alive)
            || m.mode != CombineMode::Add
        {
            continue;
        }
        let contribution = resolved_value_with_context(state, m, Some(to), Some(from));
        value += contribution;
        sources.push(src(m, contribution));
    }
    let mut final_override = None;
    if matches!(
        (state.players.get(from), state.players.get(to)),
        (Some(from_player), Some(to_player)) if from_player.hp >= to_player.hp
    ) {
        for m in state
            .modifiers
            .for_target(ModifierTarget::DistanceFromHpGteTarget)
        {
            if m.owner != from || m.mode != CombineMode::Override {
                continue;
            }
            let value = resolved_value(state, m);
            final_override = Some(value);
            sources.push(src(m, value));
        }
    }
    for m in state
        .modifiers
        .for_target(ModifierTarget::DistanceFromSkillBoundTarget)
    {
        if m.mode != CombineMode::Override
            || !super::super::skill_bound_target_modifier_applies(state, &m, from, to)
        {
            continue;
        }
        let value = resolved_value(state, m);
        final_override = Some(value);
        sources.push(src(m, value));
    }
    // Distance never below 1 when different players and both alive (caller clamps)
    (value, sources, final_override)
}
