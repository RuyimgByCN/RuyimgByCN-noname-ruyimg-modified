//! Batch U05-1 内核修饰器：索链甲属性免伤（受 `unequip`/`unequip2` 与无视防具制约）、
//! 凤扬阵临时飞影（+1 被计算距离）、云垂阵临时无双、马钧精械等。

use super::d34_flags::flag_val;
use super::d34_l28_mods;
use super::is_target_unequip2;
use crate::catalog::{CardKind, Catalog};
use crate::event::DamageNature;
use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

/// `card/yunchou/skill.js:7-28`: 索链甲防止即将受到的属性伤害（Fire/Thunder），
/// 但仍受 `unequip` / `unequip2` 及来源无视防具效果影响。
pub(super) fn suolianjia_prevents_damage(
    state: &GameStateCore,
    catalog: &Catalog,
    source: Option<PlayerId>,
    target: PlayerId,
    card: Option<CardId>,
    nature: DamageNature,
) -> bool {
    if nature == DamageNature::Normal {
        return false;
    }
    let has_suolianjia = state
        .player(target)
        .equips
        .iter()
        .any(|c| catalog.card_kind(*c) == CardKind::Suolianjia);
    if !has_suolianjia {
        return false;
    }
    if is_target_unequip2(state, target)
        || source.is_some_and(|s| {
            d34_l28_mods::source_ignores_armor(state, catalog, s, target, card, None, nature)
        })
    {
        return false;
    }
    true
}

/// 凤扬阵（`card/zhenfa/card.js:123-143`）：被围攻角色获临时【飞影】（其他角色计算至其距离 +1）。
pub(super) fn apply_fengyang_distance(state: &GameStateCore, target: PlayerId, cur: i32) -> i32 {
    if flag_val(state, target, "u05_fengyang_feiying") > 0 {
        cur + 1
    } else {
        cur
    }
}

/// 伤害提交后的 Batch U05-1 临时状态清扫与计数（凤扬阵飞影到期、云垂阵无双到期、
/// 丝连额外回合伤敌记录）。
pub(crate) fn on_damage_settled_u05(
    core: &mut GameStateCore,
    source: Option<PlayerId>,
    target: PlayerId,
) {
    core.vm_aux
        .flags
        .remove(&(target, "u05_fengyang_feiying".to_string()));
    if let Some(src) = source {
        core.vm_aux
            .flags
            .remove(&(src, "u05_yunchuizhen_wushuang".to_string()));
        if flag_val(core, src, "u05_msilian_extra_active") > 0 {
            let entry = core
                .vm_aux
                .flags
                .entry((src, "u05_msilian_damaged".to_string()))
                .or_insert(0);
            *entry = entry.saturating_add(1);
        }
    }
}
