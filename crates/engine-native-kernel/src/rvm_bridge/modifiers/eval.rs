//! 锁定技／视作技与被动修正规则的运行时求值函数。

use super::*;

/// 迭代角色当前拥有的所有已启用技能（含标记临时技能）的 `VmModifierRule`。
pub fn for_each_player_vm_modifier<F>(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut f: F,
) where
    F: FnMut(SkillDefId, &VmModifierRule),
{
    for ((owner, def), inst) in &state.skills {
        if *owner == player && inst.enabled {
            let rules = catalog.skills[*def as usize].behavior.vm_modifiers();
            for rule in rules {
                f(*def, rule);
            }
        }
    }
}

/// 评估角色手牌上限修正（`HandLimit` / `GlobalHandLimit`）。
pub fn eval_hand_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    mut limit: i32,
) -> usize {
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::HandLimit
            && rule
                .selector
                .map_or(true, |s| s.matches_owner(state, player))
        {
            let val = rule.effective_value_for(state, catalog, player, Some(player), def);
            limit = rule.apply_to(limit, val);
        }
    });
    for owner in state.alive_players() {
        for_each_player_vm_modifier(state, catalog, owner, |def, rule| {
            if rule.target == VmModifierTarget::GlobalHandLimit
                && rule
                    .selector
                    .map_or(true, |s| s.matches_owner(state, player))
            {
                let val = rule.effective_value_for(state, catalog, owner, Some(player), def);
                limit = rule.apply_to(limit, val);
            }
        });
    }
    limit = super::d34_flags::apply_hand_limit(state, catalog, player, limit);
    limit.max(0) as usize
}

/// 评估角色手牌中不计入手牌上限的牌张数（`CardIgnoredHandLimit` / `ShaIgnoredHandLimit`）。
pub fn eval_ignored_hand_card_count(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> usize {
    state
        .player(player)
        .hand
        .iter()
        .filter(|&&card| {
            let kind = catalog.card_kind(card);
            let nature = match kind {
                CardKind::FireSha => DamageNature::Fire,
                CardKind::LeiSha => DamageNature::Thunder,
                _ => DamageNature::Normal,
            };
            let mut ignored = false;
            for_each_player_vm_modifier(state, catalog, player, |def, rule| match rule.target {
                VmModifierTarget::ShaIgnoredHandLimit if kind.is_sha_like() => {
                    if rule
                        .selector
                        .map_or(true, |s| s.matches_card(state, catalog, player, card, kind, nature))
                        && rule.effective_value(state, catalog, player, def) > 0
                    {
                        ignored = true;
                    }
                }
                VmModifierTarget::CardIgnoredHandLimit => {
                    if rule
                        .selector
                        .map_or(true, |s| s.matches_card(state, catalog, player, card, kind, nature))
                        && rule.effective_value(state, catalog, player, def) > 0
                    {
                        ignored = true;
                    }
                }
                _ => {}
            });
            ignored || super::d34_flags::is_card_ignored_hand_limit(state, catalog, player, card, kind)
        })
        .count()
}

/// 评估角色攻击范围修正（`AttackRange` / `AttackRangeFixed` / `AttackRangeAbsent` / `GlobalAttackRange`）。
pub fn eval_attack_range(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    base: u32,
) -> u32 {
    let mut cur = base as i32;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if !rule
            .selector
            .map_or(true, |s| s.matches_owner(state, player))
        {
            return;
        }
        match rule.target {
            VmModifierTarget::AttackRange | VmModifierTarget::AttackRangeFixed => {
                let val = rule.effective_value_for(state, catalog, player, Some(player), def);
                cur = rule.apply_to(cur, val);
            }
            VmModifierTarget::AttackRangeAbsent => {
                if rule.effective_value_for(state, catalog, player, Some(player), def) > 0 {
                    cur = 0;
                }
            }
            _ => {}
        }
    });
    for owner in state.alive_players() {
        for_each_player_vm_modifier(state, catalog, owner, |def, rule| {
            if rule.target == VmModifierTarget::GlobalAttackRange
                && rule
                    .selector
                    .map_or(true, |s| s.matches_owner(state, player))
            {
                let val = rule.effective_value_for(state, catalog, owner, Some(player), def);
                cur = rule.apply_to(cur, val);
            }
        });
    }
    cur = super::d34_flags::apply_attack_range(state, catalog, player, cur);
    cur.max(0) as u32
}

/// 评估角色 `a -> b` 的距离修正（`DistanceToOthers` / `DistanceFromOthers` / `DistanceFromIfHpGteTarget` / `GlobalDistanceTo`）。
pub fn eval_distance_total(
    state: &GameStateCore,
    catalog: &Catalog,
    a: PlayerId,
    b: PlayerId,
    base_total: i32,
) -> u32 {
    let mut cur = base_total;
    for_each_player_vm_modifier(state, catalog, a, |def, rule| {
        if !rule.selector.map_or(true, |s| s.matches_owner(state, a)) {
            return;
        }
        match rule.target {
            VmModifierTarget::DistanceToOthers => {
                let val = rule.effective_value_for(state, catalog, a, Some(b), def);
                cur = rule.apply_to(cur, val);
            }
            VmModifierTarget::DistanceFromIfHpGteTarget if state.player(a).hp >= state.player(b).hp => {
                let val = rule.effective_value_for(state, catalog, a, Some(b), def);
                cur = rule.apply_to(cur, val);
            }
            _ => {}
        }
    });
    for_each_player_vm_modifier(state, catalog, b, |def, rule| {
        if rule.target == VmModifierTarget::DistanceFromOthers
            && rule.selector.map_or(true, |s| s.matches_owner(state, b))
            && !(catalog.skills[def as usize].name == "feiying"
                && catalog.is_tieji_sealed(state, b))
        {
            let val = rule.effective_value_for(state, catalog, b, Some(a), def);
            cur = rule.apply_to(cur, val);
        }
    });
    for owner in state.alive_players() {
        for_each_player_vm_modifier(state, catalog, owner, |def, rule| {
            if rule.target == VmModifierTarget::GlobalDistanceTo
                && rule.selector.map_or(true, |s| s.matches_owner(state, b))
            {
                let val = rule.effective_value_for(state, catalog, owner, Some(b), def);
                cur = rule.apply_to(cur, val);
            }
        });
    }
    cur = super::d34_flags::apply_distance_total(state, catalog, a, b, cur);
    cur.max(1) as u32
}

/// 评估摸牌阶段摸牌数修正（`DrawCount` / `GlobalDrawCount`）。
pub fn eval_draw_count(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    base_count: i32,
) -> i32 {
    let mut cur = base_count;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::DrawCount
            && rule
                .selector
                .map_or(true, |s| s.matches_owner(state, player))
        {
            let val = rule.effective_value_for(state, catalog, player, Some(player), def);
            cur = rule.apply_to(cur, val);
        }
    });
    for owner in state.alive_players() {
        for_each_player_vm_modifier(state, catalog, owner, |def, rule| {
            if rule.target == VmModifierTarget::GlobalDrawCount
                && rule
                    .selector
                    .map_or(true, |s| s.matches_owner(state, player))
            {
                let val = rule.effective_value_for(state, catalog, owner, Some(player), def);
                cur = rule.apply_to(cur, val);
            }
        });
    }
    super::d34_flags::apply_draw_count(state, catalog, player, cur)
}

/// 评估出牌阶段【杀】使用次数上限修正与是否无次数限制。
pub fn eval_sha_use_limit(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: Option<CardId>,
    kind: CardKind,
    nature: DamageNature,
    base_limit: u32,
) -> (u32, bool) {
    let mut cur = base_limit as i32;
    let mut unlimited = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        let sel_ok = match (rule.selector, card) {
            (None, _) => true,
            (Some(s), Some(c)) => s.matches_card(state, catalog, player, c, kind, nature),
            (Some(s), None) => matches!(
                s,
                VmModifierSelector::Any
                    | VmModifierSelector::Sha
                    | VmModifierSelector::Basic
                    | VmModifierSelector::DamageCard
            ),
        };
        if !sel_ok {
            return;
        }
        match rule.target {
            VmModifierTarget::ShaUseLimit | VmModifierTarget::CardUseLimit => {
                let val = rule.effective_value(state, catalog, player, def);
                if val >= 99 {
                    unlimited = true;
                } else {
                    cur = rule.apply_to(cur, val);
                }
            }
            VmModifierTarget::CardUseUnlimited => {
                if rule.effective_value(state, catalog, player, def) > 0 {
                    unlimited = true;
                }
            }
            _ => {}
        }
    });
    for owner in state.alive_players() {
        for_each_player_vm_modifier(state, catalog, owner, |def, rule| {
            if rule.target == VmModifierTarget::GlobalShaUseLimit {
                let val = rule.effective_value_for(state, catalog, owner, Some(player), def);
                if val >= 99 {
                    unlimited = true;
                } else {
                    cur = rule.apply_to(cur, val);
                }
            }
        });
    }
    if let Some(def) = catalog.skill_by_name("twrenfang") {
        if state.has_skill(player, def) {
            let marks = state
                .vm_aux
                .marks
                .get(&(player, 3_456_624_936))
                .copied()
                .unwrap_or(0)
                .max(0);
            cur += marks / 5;
        }
    }
    if let Some(def) = catalog.skill_by_name("olfangma") {
        if state.has_skill(player, def) {
            let left = state
                .vm_aux
                .flags
                .get(&(player, "olfangma_chitu_index".to_string()))
                .copied()
                .unwrap_or(0) as usize;
            let right = state.player(player).hand.len().saturating_sub(left);
            if left > right {
                unlimited = true;
            }
        }
    }
    (cur, unlimited) =
        super::d34_flags::apply_sha_use_limit(state, catalog, player, card, cur, unlimited);
    (cur.max(0) as u32, unlimited)
}

/// 评估【杀】／卡牌额外目标上限加成（`ShaExtraTargets` / `CardTargetLimit`）。
pub fn eval_sha_extra_targets(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> usize {
    let mut extra = 0i32;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if matches!(
            rule.target,
            VmModifierTarget::ShaExtraTargets | VmModifierTarget::CardTargetLimit
        ) && rule
            .selector
            .map_or(true, |s| s.matches_card(state, catalog, player, card, kind, nature))
        {
            let val = rule.effective_value(state, catalog, player, def);
            extra = rule.apply_to(extra, val);
        }
    });
    extra = super::d34_flags::apply_sha_extra_targets(state, catalog, player, kind, extra);
    extra.max(0) as usize
}

/// 评估使用卡牌是否忽略距离限制（`CardDistanceUnlimited`）。
pub fn eval_card_distance_unlimited(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    eval_card_distance_unlimited_to(state, catalog, player, None, card, kind, nature)
}

/// 评估使用卡牌对指定目标 `target` 是否忽略距离限制（如 `ssr_jinfan` 按目标手牌数判定）。
pub fn eval_card_distance_unlimited_to(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    target: Option<PlayerId>,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    let mut unlimited = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::CardDistanceUnlimited
            && rule
                .selector
                .map_or(true, |s| s.matches_card(state, catalog, player, card, kind, nature))
            && rule.effective_value(state, catalog, player, def) > 0
        {
            unlimited = true;
        }
    });
    super::d34_flags::apply_card_distance_unlimited_to(
        state, catalog, player, target, card, kind, unlimited,
    )
}

/// 评估目标是否免疫某张牌的指向（`CardTargetCancel` / `CardTargetEnabled`）。
pub fn eval_card_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    let mut canceled = false;
    for_each_player_vm_modifier(state, catalog, target, |def, rule| {
        if rule.target == VmModifierTarget::CardTargetCancel
            && rule
                .selector
                .map_or(true, |s| s.matches_card(state, catalog, actor, card, kind, nature))
            && rule.effective_value_for(state, catalog, target, Some(actor), def) > 0
        {
            canceled = true;
        }
        if rule.target == VmModifierTarget::CardTargetEnabled
            && rule
                .selector
                .map_or(true, |s| s.matches_card(state, catalog, actor, card, kind, nature))
            && rule.effective_value_for(state, catalog, target, Some(actor), def) <= 0
        {
            canceled = true;
        }
    });
    super::d34_flags::apply_card_target_canceled(
        state, catalog, actor, target, card, kind, canceled,
    )
}

/// 评估角色是否被禁止使用某张牌（`ShaUseCancel` / `CardUseCancel` / `HandCardUseResponseCancel` / `CardUsable`）。
pub fn eval_card_use_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    let is_sha = kind.is_sha_like();
    let in_hand = state.player(actor).hand.contains(&card);
    let mut canceled = false;
    if in_hand {
        let que_mask = state
            .vm_aux
            .flags
            .get(&(actor, "olsbguqu_restricted_suit".to_string()))
            .copied()
            .unwrap_or(0);
        if que_mask != 0 {
            let suit_bit = match catalog.cards.get(card as usize).map(|c| c.suit) {
                Some(crate::catalog::Suit::Spade) => 1u32,
                Some(crate::catalog::Suit::Heart) => 2u32,
                Some(crate::catalog::Suit::Club) => 4u32,
                Some(crate::catalog::Suit::Diamond) => 8u32,
                None => 0u32,
            };
            if (que_mask & suit_bit) != 0 {
                canceled = true;
            }
        }
    }
    for_each_player_vm_modifier(state, catalog, actor, |def, rule| {
        let sel_ok = rule
            .selector
            .map_or(true, |s| s.matches_card(state, catalog, actor, card, kind, nature));
        if !sel_ok {
            return;
        }
        let val = rule.effective_value(state, catalog, actor, def);
        match rule.target {
            VmModifierTarget::ShaUseCancel if is_sha && val > 0 => canceled = true,
            VmModifierTarget::CardUseCancel if val > 0 => canceled = true,
            VmModifierTarget::HandCardUseResponseCancel if in_hand && val > 0 => canceled = true,
            VmModifierTarget::CardUsable if val <= 0 => canceled = true,
            _ => {}
        }
    });
    if kind == CardKind::Tao && actor != state.turn.current && state.player(state.turn.current).alive
    {
        let cur = state.turn.current;
        for_each_player_vm_modifier(state, catalog, cur, |def, rule| {
            if rule.target == VmModifierTarget::DyingPeachTargetCancel
                && rule.value_source
                    == Some(VmModifierValueSource::TaoSaveCancelNonDyingDuringOwnerTurn)
                && rule.effective_value(state, catalog, cur, def) > 0
            {
                canceled = true;
            }
        });
    }
    super::d34_flags::apply_card_use_canceled(state, catalog, actor, card, kind, in_hand, canceled)
}

/// 评估角色是否被禁止对濒死角色使用【桃】（`DyingPeachTargetCancel` / `DyingSavePeachUsable` / `CardSavable`）。
pub fn eval_dying_peach_target_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    actor: PlayerId,
    dying_target: PlayerId,
    card: CardId,
) -> bool {
    let kind = catalog.card_kind(card);
    let mut canceled = false;
    for_each_player_vm_modifier(state, catalog, actor, |def, rule| {
        if !rule.selector.map_or(true, |s| {
            s.matches_card(state, catalog, actor, card, kind, DamageNature::Normal)
        }) {
            return;
        }
        match rule.target {
            VmModifierTarget::DyingPeachTargetCancel => {
                let hit = match rule.value_source {
                    Some(VmModifierValueSource::TaoSaveCancelBeyondDistance2) => {
                        kind == CardKind::Tao
                            && actor != dying_target
                            && state.computed_distance(catalog, actor, dying_target) > 2
                    }
                    Some(VmModifierValueSource::TaoSaveCancelNonDyingDuringOwnerTurn) => false,
                    _ => rule.effective_value_for(state, catalog, actor, Some(dying_target), def) > 0,
                };
                if hit {
                    canceled = true;
                }
            }
            VmModifierTarget::DyingSavePeachUsable | VmModifierTarget::CardSavable => {
                if rule.effective_value_for(state, catalog, actor, Some(dying_target), def) <= 0 {
                    canceled = true;
                }
            }
            _ => {}
        }
    });
    if actor != dying_target && kind == CardKind::Tao {
        for_each_player_vm_modifier(state, catalog, dying_target, |def, rule| {
            if rule.target == VmModifierTarget::DyingPeachTargetCancel
                && rule.value_source
                    != Some(VmModifierValueSource::TaoSaveCancelNonDyingDuringOwnerTurn)
                && rule.value_source != Some(VmModifierValueSource::TaoSaveCancelBeyondDistance2)
                && rule.effective_value_for(state, catalog, dying_target, Some(actor), def) > 0
            {
                canceled = true;
            }
        });
        let cur = state.turn.current;
        if actor != cur && state.player(cur).alive {
            for_each_player_vm_modifier(state, catalog, cur, |def, rule| {
                if rule.target == VmModifierTarget::DyingPeachTargetCancel
                    && rule.value_source
                        == Some(VmModifierValueSource::TaoSaveCancelNonDyingDuringOwnerTurn)
                    && rule.effective_value(state, catalog, cur, def) > 0
                {
                    canceled = true;
                }
            });
        }
    }
    super::d34_flags::apply_dying_peach_target_canceled(
        state,
        catalog,
        actor,
        dying_target,
        card,
        kind,
        canceled,
    )
}

/// 评估伤害修正（`DamageAmount` / `DamageTakenAmount` / `DamageTakenCancel`）。
pub fn eval_damage_amount(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
    base: u32,
) -> u32 {
    let mut cur = base as i32;
    let matches_sel = |owner: PlayerId, sel: Option<VmModifierSelector>| match (sel, card) {
        (None, _) => true,
        (Some(s), Some(c)) => {
            let kind = catalog.card_kind(c);
            s.matches_card(state, catalog, owner, c, kind, nature)
        }
        (Some(s), None) => s.matches_nature(nature),
    };
    if let Some(src) = source {
        for_each_player_vm_modifier(state, catalog, src, |def, rule| {
            if rule.target == VmModifierTarget::DamageAmount && matches_sel(src, rule.selector) {
                let val = rule.effective_value_for(state, catalog, src, Some(target), def);
                cur = rule.apply_to(cur, val);
            }
        });
    }
    cur = super::d34_l28_mods::apply_damage_begin3(
        state, catalog, source, target, card, nature, cur,
    );
    for_each_player_vm_modifier(state, catalog, target, |def, rule| {
        if !matches_sel(target, rule.selector) {
            return;
        }
        match rule.target {
            VmModifierTarget::DamageTakenAmount => {
                let val = rule.effective_value_for(state, catalog, target, source, def);
                cur = rule.apply_to(cur, val);
            }
            VmModifierTarget::DamageTakenCancel => {
                if rule.effective_value_for(state, catalog, target, source, def) > 0 {
                    cur = 0;
                }
            }
            _ => {}
        }
    });
    cur = super::d34_flags::apply_damage_amount(state, catalog, source, target, card, nature, cur);
    cur.max(0) as u32
}

/// 评估承受伤害是否被防止（`DamageTakenCancel`）。
pub fn eval_damage_taken_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
    amount: u32,
) -> bool {
    amount > 0 && eval_damage_amount(state, catalog, source, target, card, nature, amount) == 0
}

/// 评估攻击方是否无视目标防具（`IgnoreArmor`）。
pub fn eval_ignore_armor(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
) -> bool {
    let mut ignore = false;
    for_each_player_vm_modifier(state, catalog, source, |def, rule| {
        if rule.target == VmModifierTarget::IgnoreArmor {
            let sel_ok = match (rule.selector, card) {
                (None, _) => true,
                (Some(s), Some(c)) => {
                    let kind = catalog.card_kind(c);
                    s.matches_card(state, catalog, source, c, kind, nature)
                }
                (Some(s), None) => s.matches_nature(nature),
            };
            if sel_ok && rule.effective_value_for(state, catalog, source, Some(target), def) > 0 {
                ignore = true;
            }
        }
    });
    super::d34_flags::apply_ignore_armor_with_card(state, catalog, source, target, card, ignore)
}

/// 评估使用卡牌是否不可被响应（`CardUseUnrespondable` / `ShaUnrespondable` / `CardUnrespondable`）。
pub fn eval_card_unrespondable(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
    nature: DamageNature,
) -> bool {
    let is_sha = kind.is_sha_like();
    let mut unrespondable = false;
    for_each_player_vm_modifier(state, catalog, source, |def, rule| {
        let sel_ok = rule
            .selector
            .map_or(true, |s| s.matches_card(state, catalog, source, card, kind, nature));
        if !sel_ok || rule.effective_value_for(state, catalog, source, Some(target), def) <= 0 {
            return;
        }
        match rule.target {
            VmModifierTarget::CardUseUnrespondable | VmModifierTarget::CardUnrespondable => {
                unrespondable = true;
            }
            VmModifierTarget::ShaUnrespondable if is_sha => {
                unrespondable = true;
            }
            _ => {}
        }
    });
    super::d34_flags::apply_card_unrespondable_with_nature(
        state,
        catalog,
        source,
        target,
        card,
        kind,
        nature,
        unrespondable,
    )
}

/// 评估伤害来源是否触发「造成伤害视为体力流失」（`DamageAsHpLoss`）。
pub fn eval_damage_as_hp_loss(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
) -> bool {
    let mut replace = false;
    for_each_player_vm_modifier(state, catalog, source, |def, rule| {
        if rule.target != VmModifierTarget::DamageAsHpLoss {
            return;
        }
        let sel_ok = match (rule.selector, card) {
            (None, _) => true,
            (Some(s), Some(c)) => s.matches_card(state, catalog, source, c, catalog.card_kind(c), nature),
            (Some(s), None) => s.matches_nature(nature),
        };
        if sel_ok && rule.effective_value_for(state, catalog, source, Some(target), def) > 0 {
            replace = true;
        }
    });
    replace
}

/// 评估防具栏空置且未失效时是否视为装备【八卦阵】（`VirtualBaguaIfArmorSlotEmpty`）。
pub fn eval_virtual_bagua_if_armor_slot_empty(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    if !state.player(player).alive
        || crate::kernel::mbcmfangzhu_baiban(state, player)
        || super::d34_flags::is_target_unequip2(state, player)
    {
        return false;
    }
    let armor_disabled = !crate::rvm_ops::has_enabled_equip_slot(state, player, EquipSlot::Armor);
    let has_armor_equip = state
        .player(player)
        .equips
        .iter()
        .any(|&c| catalog.card_slot_of(state, c) == Some(EquipSlot::Armor));
    if armor_disabled || has_armor_equip {
        return false;
    }
    let mut active = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::VirtualBaguaIfArmorSlotEmpty
            && rule.effective_value(state, catalog, player, def) > 0
        {
            active = true;
        }
    });
    active
}

/// 评估【杀】目标或来源因「若对方为女性则多需闪」（`ShaResponseCountIfOtherFemale`）增加的额外【闪】需求。
pub fn eval_sha_response_extra_count(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
) -> u8 {
    if source == target {
        return 0;
    }
    let mut extra = 0i32;
    if state.matches_gender(catalog, target, Gender::Female) {
        for_each_player_vm_modifier(state, catalog, source, |def, rule| {
            if rule.target == VmModifierTarget::ShaResponseCountIfOtherFemale {
                let val = rule.effective_value_for(state, catalog, source, Some(target), def);
                extra = rule.apply_to(extra, val);
            }
        });
    }
    if state.matches_gender(catalog, source, Gender::Female) {
        for_each_player_vm_modifier(state, catalog, target, |def, rule| {
            if rule.target == VmModifierTarget::ShaResponseCountIfOtherFemale {
                let val = rule.effective_value_for(state, catalog, target, Some(source), def);
                extra = rule.apply_to(extra, val);
            }
        });
    }
    extra.clamp(0, 8) as u8
}

/// 某技能定义是否声明「手牌及装备牌改判」（`JudgmentRewriteHandEquip`）。
pub fn skill_has_judgment_rewrite_hand_equip(catalog: &Catalog, def: SkillDefId) -> bool {
    catalog.skills.get(def as usize).is_some_and(|s| {
        s.behavior
            .vm_modifiers()
            .iter()
            .any(|r| r.target == VmModifierTarget::JudgmentRewriteHandEquip && r.value > 0)
    })
}

/// 收集角色当前启用且未被铁骑封印的 `JudgmentRewriteHandEquip` 技能定义。
pub fn player_judgment_rewrite_hand_equip_defs(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> Vec<SkillDefId> {
    let sealed = catalog.is_tieji_sealed(state, player);
    let mut defs = Vec::new();
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::JudgmentRewriteHandEquip
            && !(sealed && !Catalog::is_locked_skill_name(catalog.skill_name(def), None))
            && rule.effective_value(state, catalog, player, def) > 0
            && !defs.contains(&def)
        {
            defs.push(def);
        }
    });
    defs
}

/// 评估未横置角色是否阻止横置（`ArmorLinkGainCancel`）。
pub fn eval_armor_link_gain_canceled(
    state: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
) -> bool {
    if state.player(player).linked {
        return false;
    }
    let mut blocked = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::ArmorLinkGainCancel
            && rule.effective_value(state, catalog, player, def) > 0
        {
            blocked = true;
        }
    });
    blocked
}

/// 评估已横置角色是否阻止重置（`LinkLossCancel`）。
pub fn eval_link_loss_canceled(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    if !state.player(player).linked {
        return false;
    }
    let mut blocked = false;
    for_each_player_vm_modifier(state, catalog, player, |def, rule| {
        if rule.target == VmModifierTarget::LinkLossCancel
            && rule.effective_value(state, catalog, player, def) > 0
        {
            blocked = true;
        }
    });
    blocked
}

/// 角色由未横置进入横置前统一门控。
pub fn can_gain_link(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    !super::d34_l28_mods::rewrite_tengjia_blocks_link(state, catalog, player)
        && !super::txminguangkai_blocks_link(state, catalog, player)
        && !eval_armor_link_gain_canceled(state, catalog, player)
}

/// 角色由横置解除前统一门控。
pub fn can_lose_link(state: &GameStateCore, catalog: &Catalog, player: PlayerId) -> bool {
    !super::txlianbi_blocks_unlink(state, catalog, player)
        && !eval_link_loss_canceled(state, catalog, player)
}

/// 评估他人是否禁止弃置目标装备区中的某张牌（`CardDiscardProtected`）。
pub fn eval_card_discard_protected(
    state: &GameStateCore,
    catalog: &Catalog,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
) -> bool {
    if source == target || !state.player(target).equips.contains(&card) {
        return false;
    }
    let kind = catalog.card_kind(card);
    let slot = catalog.card_slot_of(state, card);
    let mut protected = false;
    for_each_player_vm_modifier(state, catalog, target, |def, rule| {
        if rule.target != VmModifierTarget::CardDiscardProtected {
            return;
        }
        let sel_ok = match rule.selector {
            None => true,
            Some(VmModifierSelector::ArmorOrTreasure) => {
                matches!(slot, Some(EquipSlot::Armor | EquipSlot::Treasure))
            }
            Some(s) => s.matches_card(state, catalog, target, card, kind, DamageNature::Normal),
        };
        if sel_ok && rule.effective_value_for(state, catalog, target, Some(source), def) > 0 {
            protected = true;
        }
    });
    protected
}
