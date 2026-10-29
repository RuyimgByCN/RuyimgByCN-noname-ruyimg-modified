//! 名称符号表（演进提案 S12/C0「名称所有权」；总量上限策略 ADR 0779）。
//!
//! 目录与扩展的名称字符串（技能键、牌定义名、武将名）全局只分配一次：
//! 同名名称经 `intern` 复用同一 `&'static str`；武将技能名单等名称切片经
//! `intern_list` 按内容复用。重复「装载 / 开局 / 启用 / 禁用」循环不再线性
//! 增长泄漏——总量以「唯一名称数 + 唯一名单数」为上界（`interned_count`
//! 供内存界验收读取）。类型仍是 `&'static str`：名称进入目录、快照与
//! 字节存档的路径字节不变，不需要递增任何版本。
//!
//! 上限策略（ADR 0779）：表项不回收（`&'static` 贯穿目录与存档语义），
//! 以 [`SYMBOL_TABLE_LIMIT`] 为总量上限；生产入口（装载器）在超限时
//! 拒装而非无界增长。

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, OnceLock};

/// 符号表总量上限：唯一名称数 + 唯一名单数合计。标准目录与全部已知
/// 来源的名称总量在 1e5 以下，此处留约两个数量级余量；超限由装载器
/// 拒装（`BridgeError::SymbolTableLimit`），不在引擎内 panic。
pub const SYMBOL_TABLE_LIMIT: usize = 1_000_000;

fn table() -> &'static Mutex<BTreeSet<&'static str>> {
    static TABLE: OnceLock<Mutex<BTreeSet<&'static str>>> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(BTreeSet::new()))
}

fn list_table() -> &'static Mutex<BTreeMap<Vec<&'static str>, &'static [&'static str]>> {
    static TABLE: OnceLock<Mutex<BTreeMap<Vec<&'static str>, &'static [&'static str]>>> =
        OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// intern：同名名称返回同一 `&'static str`；首次出现才分配一次。
pub fn intern(name: String) -> &'static str {
    let mut table = table().lock().expect("名称符号表锁不可中毒");
    if let Some(&existing) = table.get(name.as_str()) {
        return existing;
    }
    let leaked: &'static str = Box::leak(name.into_boxed_str());
    table.insert(leaked);
    leaked
}

/// 名单 intern：同内容名单返回同一 `&'static [&'static str]`；首次出现才
/// 分配一次（元素应先经 `intern`）。
pub fn intern_list(list: Vec<&'static str>) -> &'static [&'static str] {
    let mut table = list_table().lock().expect("名单符号表锁不可中毒");
    if let Some(&existing) = table.get(&list) {
        return existing;
    }
    let leaked: &'static [&'static str] = Box::leak(list.clone().into_boxed_slice());
    table.insert(list, leaked);
    leaked
}

/// 已 intern 的唯一名称数 + 唯一名单数（重复循环的内存界验收读数）。
pub fn interned_count() -> usize {
    table().lock().expect("名称符号表锁不可中毒").len()
        + list_table().lock().expect("名单符号表锁不可中毒").len()
}

/// 距 [`SYMBOL_TABLE_LIMIT`] 的剩余名额（0 = 已达上限，装载器拒装）。
pub fn remaining_capacity() -> usize {
    SYMBOL_TABLE_LIMIT.saturating_sub(interned_count())
}
