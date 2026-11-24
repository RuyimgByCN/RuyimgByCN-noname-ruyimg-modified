//! Step 5 VM 修饰求值扩展（拆分自 `eval.rs` 以维持单文件有效行数上限）。

use super::*;

pub(super) fn card_effective_rank(
    state: &GameStateCore,
    catalog: &Catalog,
    card: CardId,
) -> u8 {
    catalog.resolved_card_def(state, card).map_or(0, |d| d.rank)
}

pub(super) fn skill_mark_value(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    skill_def: SkillDefId,
) -> i32 {
    if let Some(pkg_id) = catalog
        .skills
        .get(skill_def as usize)
        .and_then(|d| d.behavior.package_skill_id())
    {
        if let Some(&v) = state.vm_aux.marks.get(&(player, pkg_id)) {
            return v;
        }
    }
    state
        .vm_aux
        .marks
        .get(&(player, u32::from(skill_def)))
        .copied()
        .unwrap_or(0)
}

pub(super) fn bound_target_seat(state: &GameStateCore, owner: PlayerId) -> Option<PlayerId> {
    let encoded = state
        .vm_aux
        .flags
        .get(&(owner, "bound_target".to_string()))
        .copied()
        .unwrap_or(0);
    let idx = encoded.checked_sub(1)? as usize;
    (idx < state.players.len()).then_some(idx as PlayerId)
}

/// 判定角色是否持有依赖 `sha_used_count` 精确计数的技能（避免无用写入污染标准 golden 哈希）。
pub fn player_tracks_sha_used_count(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    let mut tracks = false;
    for_each_player_vm_modifier(state, catalog, player, |_, rule| {
        if matches!(
            rule.value_source,
            Some(
                VmModifierValueSource::OwnerShaUsedZero
                    | VmModifierValueSource::OwnerShaUsedNonzero
                    | VmModifierValueSource::OwnerShaUsedExactlyOne
                    | VmModifierValueSource::OwnerShaUsedExactlyTwo
            )
        ) {
            tracks = true;
        }
    });
    tracks
        || ["jintao", "xinjintao"]
            .into_iter()
            .any(|sk| catalog.skill_by_name(sk).is_some_and(|d| state.has_skill(player, d)))
}

/// 评估使用牌是否被限制只能指定自己为目标（`CardTargetsSelfOnly`，如 `zishou2`）。
pub fn eval_card_targets_self_only(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
) -> bool {
    let mut self_only = false;
    for_each_player_vm_modifier(state, catalog, actor, |def, rule| {
        if rule.target == VmModifierTarget::CardTargetsSelfOnly
            && rule.effective_value(state, catalog, actor, def) > 0
        {
            self_only = true;
        }
    });
    self_only
}

/// 评估对所有目标是否均命中目标绑定无限【杀】（`ShaUseUnlimitedSkillBoundTarget`，如 `xianzhen2`/`rexianzhen2`）。
pub fn eval_sha_unlimited_for_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    targets: &[PlayerId],
) -> bool {
    if targets.is_empty() {
        return false;
    }
    let Some(bound) = bound_target_seat(state, player) else {
        return false;
    };
    if !targets.iter().all(|&t| t == bound) {
        return false;
    }
    let mut unlimited = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::ShaUseUnlimitedSkillBoundTarget
            && rule.effective_value_for(state, catalog, player, Some(bound), def) > 0
        {
            unlimited = true;
        }
    });
    unlimited
        || super::d34_flags::d34_l11_mods::d34_l11_mods_vm_b::eval_sha_unlimited_for_targets(
            state, catalog, player, targets,
        )
}

/// 评估指定实体/虚拟【杀】是否豁免出牌阶段次数计数（`ShaNoUseCountRankGtMark`，如 `qiangwu`）。
pub fn eval_sha_no_use_count(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
) -> bool {
    if !kind.is_sha_like() {
        return false;
    }
    if super::d34_l11_mods_vm_c::sha_no_use_count(state, player, card, kind) {
        return true;
    }
    let Some(c) = card else {
        return false;
    };
    let rank = card_effective_rank(state, catalog, c);
    if rank == 0 {
        return false;
    }
    let mut no_count = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::ShaNoUseCountRankGtMark {
            let mark = skill_mark_value(state, catalog, player, def);
            if mark > 0
                && i32::from(rank) > mark
                && rule.effective_value(state, catalog, player, def) > 0
            {
                no_count = true;
            }
        }
    });
    no_count
}

/// 评估 `!num_fixed` 摸牌阶段的可选追加摸牌数（`OptionalDrawCount`，如 `dujin`）。
pub fn eval_optional_draw_count(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    base_count: i32,
) -> i32 {
    let mut cur = base_count;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::OptionalDrawCount
            && rule
                .selector
                .map_or(true, |s| s.matches_owner(state, player))
        {
            let val = rule.effective_value_for(state, catalog, player, Some(player), def);
            cur = rule.apply_to(cur, val);
        }
    });
    cur
}

/// 评估卡牌使用距离上限（`CardDistanceLimit`，默认由调用方传入如 `1`）。
pub fn eval_card_distance_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    nature: DamageNature,
    base_limit: i32,
) -> i32 {
    let sentinel = card.unwrap_or(CardId::MAX);
    let mut limit = base_limit;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::CardDistanceLimit
            && rule
                .selector
                .map_or(true, |s| s.matches_card(state, catalog, player, sentinel, kind, nature))
        {
            let val = rule.effective_value(state, catalog, player, def);
            limit = rule.apply_to(limit, val);
        }
    });
    limit.max(0)
}

/// 评估是否命中 `CardUseUnlimited`（如 `jiaojie` 黑牌无次数限制、含黑【酒】）。
pub fn eval_card_use_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    let sentinel = card.unwrap_or(CardId::MAX);
    let mut unlimited = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::CardUseUnlimited
            && rule
                .selector
                .map_or(true, |s| s.matches_card(state, catalog, player, sentinel, kind, nature))
            && rule.effective_value(state, catalog, player, def) > 0
        {
            unlimited = true;
        }
    });
    super::d34_l11_mods_vm_c::apply_card_use_unlimited(state, catalog, player, card, kind, unlimited)
}

/// 评估 `CardIgnoredHandLimit` 是否同时禁止在弃牌阶段弃置该牌（`jiaojie` 红牌）。
pub fn eval_card_ignored_discard_phase(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) -> bool {
    let kind = catalog.card_kind(card);
    let nature = match kind {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    let mut non_discardable = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if catalog.skill_name(def) == "jiaojie"
            && rule.target == VmModifierTarget::CardIgnoredHandLimit
            && rule
                .selector
                .map_or(true, |s| s.matches_card(state, catalog, player, card, kind, nature))
            && rule.effective_value(state, catalog, player, def) > 0
        {
            non_discardable = true;
        }
    });
    non_discardable
}

/// 评估静态牌名重写 `CardName`（如 `jinjiu` 酒视为杀、`wushen` 红桃视为杀），基于物理牌定义防递归。
pub fn eval_card_name_as_sha(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) -> bool {
    let Some(phys) = catalog.cards.get(card as usize) else {
        return false;
    };
    let phys_kind = phys.kind;
    let phys_nature = match phys_kind {
        CardKind::FireSha => DamageNature::Fire,
        CardKind::LeiSha => DamageNature::Thunder,
        _ => DamageNature::Normal,
    };
    let mut is_sha = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::CardName
            && rule.selector.is_some_and(|s| {
                s.matches_card(state, catalog, player, card, phys_kind, phys_nature)
            })
            && rule.effective_value(state, catalog, player, def) > 0
        {
            is_sha = true;
        }
    });
    is_sha
}

/// 判定某次伤害评估是否属于【杀】伤害渠道（`ShaDamage`，支持实体【杀】、牌名覆盖及虚拟【杀】窗标）。
pub(super) fn is_sha_damage_eval(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    card: Option<CardId>,
) -> bool {
    if let Some(c) = card {
        if crate::kernel::effective_card_kind_in_core(state, catalog, c).is_sha_like()
            || eval_card_name_as_sha(state, catalog, source, c)
        {
            return true;
        }
    }
    state
        .vm_aux
        .flags
        .get(&(source, "active_sha_damage_eval".to_string()))
        .copied()
        .unwrap_or(0)
        > 0
}
