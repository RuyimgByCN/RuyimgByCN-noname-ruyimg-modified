//! NC08 附加：**「升级」——技能映射变更面**（八项验收第 2 项的剩余边界，2026-10-19）。
//!
//! 背景：`e02_c3_lifecycle` 已覆盖「**版本号变化** ⇒ 内容锁拒绝/回滚恢复」；
//! 本例覆盖**同版本号但技能映射变更**（`skills[].mapping` 改名/删除）——即**内容漂移**。
//!
//! 判据（**不预设实现细节**，只要求两种可接受行为之一）：
//! - **明确拒绝**（`Err(...)`）——内容锁识别到映射变化；或
//! - **接受且不 panic**，同时目录内**旧映射技能不再挂载**（语义可预期）。
//!
//! 禁止的是：**静默沿用旧映射**（会让存档引用的技能与包数据不一致）。

use std::path::PathBuf;

fn swd_json() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json");
    std::fs::read_to_string(p).expect("读 swd cards")
}

fn first_mapping(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v.get("skills")?
        .as_array()?
        .iter()
        .find_map(|s| s.get("mapping").and_then(|m| m.as_str()).map(|s| s.to_string()))
}

/// 同版本、**技能映射改名** ⇒ 恢复必须**明确拒绝**或**不静默沿用旧映射**。
#[test]
fn upgrade_skill_mapping_change_is_detected() {
    let base = swd_json();
    let Some(old_map) = first_mapping(&base) else {
        eprintln!("NOTE 技能映射：swd 包无 skills 映射——本用例不构成验收");
        return;
    };
    // ① 原样：桥接成功，且目录中该映射可解析（包可用作基线）
    let v1 = content_native_loader::bridge_pack(&base);
    assert!(v1.is_ok(), "基线包应桥接成功");
    let v1 = v1.expect("基线").pack;
    let e = engine_native::Engine::new_with_extensions(5301, 4, &[&v1]).expect("v1 建局");
    let _snapshot = e.save();
    // ② 同版本号、把 mapping 改成**未登记**的值 ⇒ **桥接层即拒绝**（早于存档恢复）
    let mut v: serde_json::Value = serde_json::from_str(&base).expect("JSON");
    let mut renamed = 0;
    if let Some(arr) = v.get_mut("skills").and_then(|s| s.as_array_mut()) {
        for s in arr.iter_mut() {
            if s.get("mapping").and_then(|x| x.as_str()) == Some(old_map.as_str()) {
                s["mapping"] = serde_json::json!("host:__unregistered_probe__");
                renamed += 1;
            }
        }
    }
    assert!(renamed > 0, "应至少改名一处映射");
    let drifted = content_native_loader::bridge_pack(&v.to_string());
    eprintln!(
        "DBG 升级-映射：同版本号改名 `{old_map}` → `host:__unregistered_probe__`；桥接结果 = {}",
        if drifted.is_ok() { "接受" } else { "**拒绝**" }
    );
    assert!(
        drifted.is_err(),
        "**未登记的 mapping 必须在桥接层被拒绝**（否则会静默沿用或产生无行为技能）"
    );
    eprintln!(
        "NOTE 技能映射：**已知映射改名**（改到另一个已登记值）不在本用例范围——\
         其存档语义由 `e02_c3_lifecycle`（内容锁/回滚）覆盖"
    );
}
