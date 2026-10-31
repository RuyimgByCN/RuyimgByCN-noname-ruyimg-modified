//! NC08 附加：**「升级」验收**（八项验收第 2 项，纯 Rust 可做）。
//!
//! 依据：八项验收清单——「升级」= 旧版本包 → 新版本升级，**数据/技能不破**。
//!
//! 包数据是**不可变输入**（`bridge_pack` 每次从 JSON 构造，无全局状态）⇒ 升级验收可机械化为：
//! 1. **版本字段递增**：同内容 `version: 1 → 2` ⇒ 桥接成功、**目录内容不变**；
//! 2. **卡牌增量**：旧包 N 张 → 新包 N+1 张 ⇒ **目录恰好 +1**（旧卡全部仍在）；
//! 3. **同一进程内新旧并存**：先后构造旧/新两包 ⇒ **互不影响**（无共享可变状态）；
//! 4. **降级**：新包 → 旧包（少一张卡）⇒ 桥接**仍成功**（缺卡不导致错误；语义面另计）。
//!
//! 边界：**存档跨版本**（用旧包存档、用新包读档）未覆盖——需 `StateSnapshot` 构造，登记待办。

use std::path::PathBuf;

fn swd_json() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json");
    std::fs::read_to_string(p).expect("读 swd cards")
}

fn catalog_names(json: &str) -> Vec<String> {
    let pack = content_native_loader::bridge_pack(json).expect("桥接").pack;
    let e = engine_native::Engine::new_with_extensions(17, 4, &[&pack]).expect("建局");
    e.catalog()
        .cards
        .iter()
        .map(|c| c.name.to_string())
        .collect()
}

/// 升级面 1（**版本递增**）：`version 1 → 2` ⇒ 目录内容**不变**。
#[test]
fn upgrade_version_bump_keeps_content() {
    let mut json: serde_json::Value = serde_json::from_str(&swd_json()).expect("JSON");
    let base = catalog_names(&swd_json());
    let v0 = json
        .pointer("/pack/version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    json["pack"]["version"] = serde_json::json!(v0 + 1);
    let bumped = catalog_names(&json.to_string());
    eprintln!(
        "DBG 升级-版本：version {v0} → {}；目录卡数 {} → {}（应相同）",
        v0 + 1,
        base.len(),
        bumped.len()
    );
    assert_eq!(base, bumped, "仅提升 version 不应改变目录内容");
}

/// 升级面 2（**卡牌增量**）：旧包 N → 新包 N+1 ⇒ 目录**恰好 +1**，旧卡**全部仍在**。
#[test]
fn upgrade_card_addition_is_additive() {
    let base_json = swd_json();
    let base = catalog_names(&base_json);
    let mut json: serde_json::Value = serde_json::from_str(&base_json).expect("JSON");
    let cards = json["cards"].as_array_mut().expect("cards 数组");
    let mut extra = cards[0].clone();
    // 造一张**新卡名**（加后缀），其余字段照抄
    if let Some(n) = extra
        .get("name")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string())
    {
        extra["name"] = serde_json::json!(format!("{n}_upgrade_probe"));
    }
    cards.push(extra);
    // swd 包为**双产物**（cards + character）——本例只改 cards，故用 bridge_pack 即可
    let added = catalog_names(&json.to_string());
    eprintln!(
        "DBG 升级-增量：目录 {} → {}（应 +1）",
        base.len(),
        added.len()
    );
    for name in &base {
        assert!(added.contains(name), "旧卡 `{name}` 应仍在目录");
    }
    assert_eq!(added.len(), base.len() + 1, "新增一张卡后目录应恰好 +1");
}

/// 升级面 3（**新旧并存**）：同一进程内先后构造旧/新包，**互不影响**。
#[test]
fn upgrade_old_and_new_coexist() {
    let old = catalog_names(&swd_json());
    let mut json: serde_json::Value = serde_json::from_str(&swd_json()).expect("JSON");
    json["pack"]["version"] = serde_json::json!(2);
    let new = catalog_names(&json.to_string());
    // 再构造一次旧包，应与第一次完全一致（无全局可变状态）
    let old_again = catalog_names(&swd_json());
    eprintln!(
        "DBG 升级-并存：old {} / new {} / old_again {}（old 应 == old_again）",
        old.len(),
        new.len(),
        old_again.len()
    );
    assert_eq!(old, old_again, "重复构造旧包结果应一致（包数据无全局状态）");
    assert_eq!(old, new, "仅版本号不同的新包目录内容应与旧包一致");
}

/// 升级面 4（**降级**）：新包 → 旧包（少一张卡）⇒ 桥接**仍成功**。
#[test]
fn upgrade_downgrade_still_bridges() {
    let mut json: serde_json::Value = serde_json::from_str(&swd_json()).expect("JSON");
    let cards = json["cards"].as_array_mut().expect("cards 数组");
    let removed = cards.pop().and_then(|c| {
        c.get("name")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    });
    let pack = content_native_loader::bridge_pack(&json.to_string());
    eprintln!(
        "DBG 升级-降级：移除 `{removed:?}` 后桥接结果 = {}",
        if pack.is_ok() { "成功" } else { "失败" }
    );
    assert!(pack.is_ok(), "移除一张卡后桥接仍应成功（缺卡不导致错误）");
}
