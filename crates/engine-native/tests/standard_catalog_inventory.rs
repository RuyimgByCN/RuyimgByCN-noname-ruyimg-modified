//! A0 标准目录构成检查（演进提案 §3.A0 第 2 点，挂靠主稿 P0「回放与目录」）。
//!
//! `standard_catalog()` 的牌名 × 数量 ×（花色, 点数）必须与下方显式声明表
//! 逐一相等。声明表独立于实现手写，防止目录构造再次漂移。
//!
//! 历史缺陷（2026-09-24 修复）：拆牌 `for _ in 0..2` 循环体误含顺手牵羊、
//! 酒、雷杀的入目录语句，各重复生成一次（顺 8、酒 4、雷杀 4，共 88 张）；
//! 修复后为本表 80 张。声明表来源与取舍见
//! docs/architecture/native-rules-baseline.md「标准目录构成声明表」。

use std::collections::BTreeMap;

use engine_native::catalog::{CardKind, Suit};
use engine_native::Engine;

/// 一张牌的声明：(名称, 牌种, 花色, 点数)。无显式花色点数的牌按构造
/// 默认值（黑桃 A）声明。
type Entry = (&'static str, CardKind, Suit, u8);

/// 原生标准目录显式声明表（规则 v74 核定，共 124 张；批③-c 逐张花色点数对齐上游 standard list（武器/防具仁王盾/马/原生扩充段为原生建模或原生取值） = 上游 108 口径的非武器段（数量对齐，花色点数沿用原生循环）+ 原生扩充 13（酒 2/火杀 4/雷杀 2/铁索 4/兵 1）+ 测试武器 3；批③-b 武器逐把落地后再增 9）。
/// 杀/闪/桃按黑桃→红桃→梅花→方块循环分配花色，点数在基准上模 9 循环；
/// 其余牌为构造默认（黑桃 A）或显式花色点数（酒、雷杀、延时锦囊）。
const DECLARED: &[Entry] = &[
    ("sha", CardKind::Sha, Suit::Spade, 7),
    ("sha", CardKind::Sha, Suit::Spade, 8),
    ("sha", CardKind::Sha, Suit::Spade, 8),
    ("sha", CardKind::Sha, Suit::Spade, 9),
    ("sha", CardKind::Sha, Suit::Spade, 9),
    ("sha", CardKind::Sha, Suit::Spade, 10),
    ("sha", CardKind::Sha, Suit::Spade, 10),
    ("sha", CardKind::Sha, Suit::Club, 2),
    ("sha", CardKind::Sha, Suit::Club, 3),
    ("sha", CardKind::Sha, Suit::Club, 4),
    ("sha", CardKind::Sha, Suit::Club, 5),
    ("sha", CardKind::Sha, Suit::Club, 6),
    ("sha", CardKind::Sha, Suit::Club, 7),
    ("sha", CardKind::Sha, Suit::Club, 8),
    ("sha", CardKind::Sha, Suit::Club, 8),
    ("sha", CardKind::Sha, Suit::Club, 9),
    ("sha", CardKind::Sha, Suit::Club, 9),
    ("sha", CardKind::Sha, Suit::Club, 10),
    ("sha", CardKind::Sha, Suit::Club, 10),
    ("sha", CardKind::Sha, Suit::Club, 11),
    ("sha", CardKind::Sha, Suit::Club, 11),
    ("sha", CardKind::Sha, Suit::Heart, 10),
    ("sha", CardKind::Sha, Suit::Heart, 10),
    ("sha", CardKind::Sha, Suit::Heart, 11),
    ("sha", CardKind::Sha, Suit::Diamond, 6),
    ("sha", CardKind::Sha, Suit::Diamond, 7),
    ("sha", CardKind::Sha, Suit::Diamond, 8),
    ("sha", CardKind::Sha, Suit::Diamond, 9),
    ("sha", CardKind::Sha, Suit::Diamond, 10),
    ("sha", CardKind::Sha, Suit::Diamond, 13),
    ("shan", CardKind::Shan, Suit::Heart, 2),
    ("shan", CardKind::Shan, Suit::Heart, 2),
    ("shan", CardKind::Shan, Suit::Heart, 13),
    ("shan", CardKind::Shan, Suit::Diamond, 2),
    ("shan", CardKind::Shan, Suit::Diamond, 2),
    ("shan", CardKind::Shan, Suit::Diamond, 3),
    ("shan", CardKind::Shan, Suit::Diamond, 4),
    ("shan", CardKind::Shan, Suit::Diamond, 5),
    ("shan", CardKind::Shan, Suit::Diamond, 6),
    ("shan", CardKind::Shan, Suit::Diamond, 7),
    ("shan", CardKind::Shan, Suit::Diamond, 8),
    ("shan", CardKind::Shan, Suit::Diamond, 9),
    ("shan", CardKind::Shan, Suit::Diamond, 10),
    ("shan", CardKind::Shan, Suit::Diamond, 11),
    ("shan", CardKind::Shan, Suit::Diamond, 11),
    ("tao", CardKind::Tao, Suit::Heart, 3),
    ("tao", CardKind::Tao, Suit::Heart, 4),
    ("tao", CardKind::Tao, Suit::Heart, 6),
    ("tao", CardKind::Tao, Suit::Heart, 7),
    ("tao", CardKind::Tao, Suit::Heart, 8),
    ("tao", CardKind::Tao, Suit::Heart, 9),
    ("tao", CardKind::Tao, Suit::Heart, 12),
    ("tao", CardKind::Tao, Suit::Diamond, 12),
    ("savage", CardKind::SavageAttack, Suit::Spade, 7),
    ("savage", CardKind::SavageAttack, Suit::Spade, 13),
    ("savage", CardKind::SavageAttack, Suit::Club, 7),
    ("duel", CardKind::Duel, Suit::Spade, 1),
    ("duel", CardKind::Duel, Suit::Club, 1),
    ("duel", CardKind::Duel, Suit::Diamond, 1),
    ("snatch", CardKind::Snatch, Suit::Spade, 3),
    ("snatch", CardKind::Snatch, Suit::Spade, 4),
    ("snatch", CardKind::Snatch, Suit::Spade, 11),
    ("snatch", CardKind::Snatch, Suit::Diamond, 3),
    ("snatch", CardKind::Snatch, Suit::Diamond, 4),
    ("dismantle", CardKind::Dismantle, Suit::Spade, 3),
    ("dismantle", CardKind::Dismantle, Suit::Spade, 4),
    ("dismantle", CardKind::Dismantle, Suit::Spade, 12),
    ("dismantle", CardKind::Dismantle, Suit::Club, 3),
    ("dismantle", CardKind::Dismantle, Suit::Club, 4),
    ("dismantle", CardKind::Dismantle, Suit::Heart, 12),
    ("nullify", CardKind::Nullify, Suit::Spade, 11),
    ("nullify", CardKind::Nullify, Suit::Club, 12),
    ("nullify", CardKind::Nullify, Suit::Club, 13),
    ("le", CardKind::Le, Suit::Spade, 6),
    ("le", CardKind::Le, Suit::Club, 6),
    ("le", CardKind::Le, Suit::Heart, 6),
    ("shandian", CardKind::ShanDian, Suit::Spade, 1),
    ("shandian_safe", CardKind::ShanDian, Suit::Heart, 12),
    ("nullify", CardKind::Nullify, Suit::Diamond, 12),
    // —— 原生扩充（G1 前置，上游无对应；花色点数为原生取值）——
    ("jiu", CardKind::Jiu, Suit::Spade, 3),
    ("jiu", CardKind::Jiu, Suit::Club, 9),
    ("leisha", CardKind::LeiSha, Suit::Spade, 4),
    ("leisha", CardKind::LeiSha, Suit::Club, 5),
    ("fire_sha", CardKind::FireSha, Suit::Spade, 1),
    ("fire_sha", CardKind::FireSha, Suit::Spade, 1),
    ("fire_sha", CardKind::FireSha, Suit::Spade, 1),
    ("fire_sha", CardKind::FireSha, Suit::Spade, 1),
    ("chain", CardKind::Chain, Suit::Spade, 1),
    ("chain", CardKind::Chain, Suit::Spade, 1),
    ("chain", CardKind::Chain, Suit::Spade, 1),
    ("chain", CardKind::Chain, Suit::Spade, 1),
    ("archery", CardKind::ArcheryAttack, Suit::Heart, 1),
    ("bing", CardKind::Bing, Suit::Club, 3),
    // —— 测试武器（泛型 Weapon，射程建模）——
    ("weapon2", CardKind::Weapon, Suit::Spade, 1),
    ("weapon2b", CardKind::Weapon, Suit::Spade, 1),
    ("weapon3", CardKind::Weapon, Suit::Spade, 1),
    // —— 泛型防具/马（批②定案：维持泛型；花色为原生默认）——
    ("armor", CardKind::Armor, Suit::Spade, 1),
    ("armor", CardKind::Armor, Suit::Spade, 1),
    ("horse_plus", CardKind::HorsePlus, Suit::Spade, 1),
    ("horse_plus", CardKind::HorsePlus, Suit::Spade, 1),
    ("horse_plus", CardKind::HorsePlus, Suit::Spade, 1),
    ("horse_plus", CardKind::HorsePlus, Suit::Spade, 1),
    ("horse_minus", CardKind::HorseMinus, Suit::Spade, 1),
    ("horse_minus", CardKind::HorseMinus, Suit::Spade, 1),
    // —— 缺牌种批①/批③-b 新增牌种（上游逐张落位）——
    ("wuzhong", CardKind::Wuzhong, Suit::Heart, 7),
    ("wuzhong", CardKind::Wuzhong, Suit::Heart, 8),
    ("wuzhong", CardKind::Wuzhong, Suit::Heart, 9),
    ("wuzhong", CardKind::Wuzhong, Suit::Heart, 11),
    ("taoyuan", CardKind::Taoyuan, Suit::Heart, 1),
    ("wugu", CardKind::Wugu, Suit::Heart, 3),
    ("wugu", CardKind::Wugu, Suit::Heart, 4),
    ("jiedao", CardKind::Jiedao, Suit::Club, 12),
    ("jiedao", CardKind::Jiedao, Suit::Club, 13),
    ("renwang", CardKind::Renwang, Suit::Club, 2),
    ("qinggang", CardKind::Qinggang, Suit::Spade, 6),
    ("qilin", CardKind::Qilin, Suit::Heart, 5),
    ("hanbing", CardKind::Hanbing, Suit::Spade, 2),
    ("zhuge", CardKind::Zhuge, Suit::Club, 1),
    ("zhuge", CardKind::Zhuge, Suit::Diamond, 1),
    ("guanshi", CardKind::Guanshi, Suit::Diamond, 5),
    ("qinglong", CardKind::Qinglong, Suit::Spade, 5),
    ("zhangba", CardKind::Zhangba, Suit::Spade, 12),
    ("cixiong", CardKind::Cixiong, Suit::Spade, 2),
    ("fangtian", CardKind::Fangtian, Suit::Diamond, 5),
];

fn count(entries: &[Entry]) -> BTreeMap<(String, u8, String, u8), usize> {
    let mut map: BTreeMap<(String, u8, String, u8), usize> = BTreeMap::new();
    for (name, kind, suit, rank) in entries {
        // 牌种以 u8 编码入键（与实现编码一致，变体只追加不漂移）。
        let key = (name.to_string(), *kind as u8, format!("{suit:?}"), *rank);
        *map.entry(key).or_insert(0) += 1;
    }
    map
}

fn diff_map(
    actual: &BTreeMap<(String, u8, String, u8), usize>,
    expected: &BTreeMap<(String, u8, String, u8), usize>,
) -> Vec<String> {
    let mut lines = Vec::new();
    for (key, exp) in expected {
        let got = actual.get(key).copied().unwrap_or(0);
        if got != *exp {
            lines.push(format!("{key:?}: 声明 {exp} 张，实际 {got} 张"));
        }
    }
    for (key, got) in actual {
        if !expected.contains_key(key) {
            lines.push(format!("{key:?}: 声明外多出 {got} 张"));
        }
    }
    lines
}

#[test]
fn standard_catalog_matches_declared_inventory() {
    let engine = Engine::new(1, 2);
    let catalog = engine.catalog();
    let actual: Vec<Entry> = catalog
        .cards
        .iter()
        .map(|c| (c.name, c.kind, c.suit, c.rank))
        .collect();
    assert_eq!(
        actual.len(),
        DECLARED.len(),
        "标准目录牌数 {} != 声明 {}",
        actual.len(),
        DECLARED.len()
    );
    let (actual_map, declared_map) = (count(&actual), count(DECLARED));
    let diff = diff_map(&actual_map, &declared_map);
    assert!(
        diff.is_empty(),
        "标准目录构成与声明表不符：\n{}",
        diff.join("\n")
    );
}

/// 数量级速查：关键缺陷曾漂移的牌种各归位（拆 2、顺 4、酒 2、雷杀 2）。
#[test]
fn defective_kinds_hold_declared_counts() {
    let engine = Engine::new(1, 2);
    let catalog = engine.catalog();
    let by_name = |name: &str| {
        catalog
            .cards
            .iter()
            .filter(|c| c.name == name)
            .count()
    };
    // v64 数量对齐：拆 6、顺 5（上游同名牌数量）；酒 2/雷杀 2 保持扩充口径。
    assert_eq!(by_name("dismantle"), 6);
    assert_eq!(by_name("snatch"), 5);
    assert_eq!(by_name("jiu"), 2);
    assert_eq!(by_name("leisha"), 2);
}
