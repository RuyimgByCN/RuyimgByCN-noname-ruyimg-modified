//! NC06 / NC07 / PA04 作用域化技能存储（Scoped Skill Storage）：
//!
//! 在不改变 `ARCHIVE_FORMAT_VERSION = 107` 与 14 个 golden 摘要的前提下，
//! 为武将/卡牌/模式技能提供结构化、带生命周期作用域（`Turn` / `Round` / `Match`）
//! 的统一存取与导入导出接口，底层同源承载于 `GameStateCore::vm_aux.flags`：
//! - **同源摘要**：任何字段写入立即反映到 `state_hash()`；
//! - **同源归档**：随 `encode_archive` / `decode_archive` 无损往返；
//! - **同源校验**：`owner` 座位引用随 `Engine::restore` 校验；
//! - **同源生命周期**：`Turn` 域随 `VmAux::clear_turn` 自动清理，`Round` 域随
//!   `VmAux::clear_round` 自动清理，`Match` 域跨整局持久保留；
//! - **AI 隐私边界**：非观察者座位的条目在 AI 遮蔽（`redact_opponent_hands`）时自动擦除。

use std::collections::{BTreeMap, BTreeSet};

use crate::ids::{CardId, PlayerId};
use crate::state::GameStateCore;

/// 技能存储条目的生命周期作用域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SkillStorageScope {
    /// 单回合有效：回合结束（`VmAux::clear_turn`）时自动清除。
    Turn,
    /// 单轮有效：轮次推进（`VmAux::clear_round`）时自动清除。
    Round,
    /// 整局有效：跨回合与轮次持久保留。
    Match,
}

impl SkillStorageScope {
    pub(crate) fn prefix(self) -> &'static str {
        match self {
            Self::Turn => "skill_turn:ss:",
            Self::Round => "skill_round:ss:",
            Self::Match => "skill_match:ss:",
        }
    }

    fn from_key(key: &str) -> Option<(Self, &str)> {
        for scope in [Self::Turn, Self::Round, Self::Match] {
            if let Some(rest) = key.strip_prefix(scope.prefix()) {
                return Some((scope, rest));
            }
        }
        None
    }
}

/// 结构化技能存储值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillStorageValue {
    /// 64 位有符号整数计数或标量。
    Int(i64),
    /// 布尔旗标。
    Flag(bool),
    /// 卡牌实体列表。
    CardList(Vec<CardId>),
    /// 座位列表。
    PlayerList(Vec<PlayerId>),
}

/// 作用域化技能存储的复合键。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SkillStorageKey {
    pub owner: PlayerId,
    pub skill: String,
    pub scope: SkillStorageScope,
    pub field: String,
}

fn base_key(scope: SkillStorageScope, skill: &str, field: &str) -> String {
    format!("{}{skill}:{field}", scope.prefix())
}

/// 判断 `vm_aux.flags` 的键是否为作用域化存储中的卡牌实体槽位（`:cards:{idx}`）。
pub fn is_scoped_card_entry(key: &str) -> bool {
    let Some((_, rest)) = SkillStorageScope::from_key(key) else {
        return false;
    };
    rest.rsplit_once(":cards:")
        .is_some_and(|(_, tail)| !tail.is_empty() && tail != "len" && tail.bytes().all(|b| b.is_ascii_digit()))
}

/// 判断 `vm_aux.flags` 的键是否为作用域化存储中的座位槽位（`:players:{idx}`）。
pub fn is_scoped_player_entry(key: &str) -> bool {
    let Some((_, rest)) = SkillStorageScope::from_key(key) else {
        return false;
    };
    rest.rsplit_once(":players:")
        .is_some_and(|(_, tail)| !tail.is_empty() && tail != "len" && tail.bytes().all(|b| b.is_ascii_digit()))
}

/// 写入或覆盖 `(owner, skill, scope, field)` 的结构化技能存储值。
pub fn set_scoped_skill_field(
    core: &mut GameStateCore,
    owner: PlayerId,
    skill: &str,
    scope: SkillStorageScope,
    field: &str,
    value: SkillStorageValue,
) {
    let base = base_key(scope, skill, field);
    let prefix = format!("{base}:");
    core.vm_aux
        .flags
        .retain(|(p, k), _| *p != owner || !k.starts_with(&prefix));

    match value {
        SkillStorageValue::Int(v) => {
            let bits = v as u64;
            let lo = (bits & 0xFFFF_FFFF) as u32;
            let hi = ((bits >> 32) & 0xFFFF_FFFF) as u32;
            core.vm_aux.flags.insert((owner, format!("{base}:int:lo")), lo);
            core.vm_aux.flags.insert((owner, format!("{base}:int:hi")), hi);
        }
        SkillStorageValue::Flag(b) => {
            core.vm_aux
                .flags
                .insert((owner, format!("{base}:flag")), u32::from(b));
        }
        SkillStorageValue::CardList(cards) => {
            core.vm_aux
                .flags
                .insert((owner, format!("{base}:cards:len")), cards.len() as u32);
            for (idx, &card) in cards.iter().enumerate() {
                core.vm_aux
                    .flags
                    .insert((owner, format!("{base}:cards:{idx}")), card);
            }
        }
        SkillStorageValue::PlayerList(players) => {
            core.vm_aux
                .flags
                .insert((owner, format!("{base}:players:len")), players.len() as u32);
            for (idx, &p) in players.iter().enumerate() {
                core.vm_aux
                    .flags
                    .insert((owner, format!("{base}:players:{idx}")), u32::from(p));
            }
        }
    }
}

/// 读取 `(owner, skill, scope, field)` 的结构化技能存储值。
pub fn get_scoped_skill_field(
    core: &GameStateCore,
    owner: PlayerId,
    skill: &str,
    scope: SkillStorageScope,
    field: &str,
) -> Option<SkillStorageValue> {
    let base = base_key(scope, skill, field);
    if let Some(&b) = core.vm_aux.flags.get(&(owner, format!("{base}:flag"))) {
        return Some(SkillStorageValue::Flag(b != 0));
    }
    if let (Some(&lo), Some(&hi)) = (
        core.vm_aux.flags.get(&(owner, format!("{base}:int:lo"))),
        core.vm_aux.flags.get(&(owner, format!("{base}:int:hi"))),
    ) {
        let bits = (u64::from(hi) << 32) | u64::from(lo);
        return Some(SkillStorageValue::Int(bits as i64));
    }
    if let Some(&len) = core.vm_aux.flags.get(&(owner, format!("{base}:cards:len"))) {
        let mut cards = Vec::with_capacity(len as usize);
        for idx in 0..len {
            let c = *core
                .vm_aux
                .flags
                .get(&(owner, format!("{base}:cards:{idx}")))?;
            cards.push(c);
        }
        return Some(SkillStorageValue::CardList(cards));
    }
    if let Some(&len) = core.vm_aux.flags.get(&(owner, format!("{base}:players:len"))) {
        let mut players = Vec::with_capacity(len as usize);
        for idx in 0..len {
            let p = *core
                .vm_aux
                .flags
                .get(&(owner, format!("{base}:players:{idx}")))?;
            players.push(p as PlayerId);
        }
        return Some(SkillStorageValue::PlayerList(players));
    }
    None
}

/// 导出当前 `GameStateCore` 中全部作用域化技能存储条目。
pub fn export_scoped_skill_storage(
    core: &GameStateCore,
) -> BTreeMap<SkillStorageKey, SkillStorageValue> {
    let mut keys = BTreeSet::new();
    for (owner, raw_key) in core.vm_aux.flags.keys() {
        let Some((scope, rest)) = SkillStorageScope::from_key(raw_key) else {
            continue;
        };
        let suffix_stripped = rest
            .strip_suffix(":flag")
            .or_else(|| rest.strip_suffix(":int:lo"))
            .or_else(|| rest.strip_suffix(":int:hi"))
            .or_else(|| rest.strip_suffix(":cards:len"))
            .or_else(|| rest.strip_suffix(":players:len"));
        let Some(skill_and_field) = suffix_stripped else {
            continue;
        };
        let Some((skill, field)) = skill_and_field.split_once(':') else {
            continue;
        };
        keys.insert(SkillStorageKey {
            owner: *owner,
            skill: skill.to_string(),
            scope,
            field: field.to_string(),
        });
    }
    let mut out = BTreeMap::new();
    for key in keys {
        if let Some(val) =
            get_scoped_skill_field(core, key.owner, &key.skill, key.scope, &key.field)
        {
            out.insert(key, val);
        }
    }
    out
}

/// 将作用域化技能存储表导入到 `GameStateCore`（先清空旧的作用域化条目再写入）。
pub fn import_scoped_skill_storage(
    core: &mut GameStateCore,
    entries: &BTreeMap<SkillStorageKey, SkillStorageValue>,
) {
    core.vm_aux
        .flags
        .retain(|(_, k), _| SkillStorageScope::from_key(k).is_none());
    for (key, value) in entries {
        set_scoped_skill_field(
            core,
            key.owner,
            &key.skill,
            key.scope,
            &key.field,
            value.clone(),
        );
    }
}
