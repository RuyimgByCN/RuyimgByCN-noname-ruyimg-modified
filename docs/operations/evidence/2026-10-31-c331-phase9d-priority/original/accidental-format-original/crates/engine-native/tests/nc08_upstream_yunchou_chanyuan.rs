//! NC08 第⑦层（上游行为对照）样本 72：**来源 U05（杀海拾遗·运筹）** 武将技
//! `diychanyuan`（蝉缘·变体）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:402`）：
//!
//! ```js
//! diychanyuan: {
//!     trigger: { player: "dieBegin" },        // **自己（濒死→）死亡时**
//!     forced: true,                            // 锁定
//!     filter(event) { return event.source !== undefined; },   // **有来源**
//!     content() { trigger.source.loseMaxHp(true); },          // **来源失去 1 点体力上限**
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：持技者**死亡**时（锁定）；
//! 2. **门槛**：死亡**有来源**（`event.source !== undefined`）；
//! 3. **效果**：**来源失去 1 点体力上限**（`loseMaxHp(true)` → 体力同时钳到新上限）。
//!
//! 原生实现引用：`content/host/step/yunchou.rs:26`（`ctx.lose_max_hp(src, 1)`）。
//!
//! **登记差异**：本样本**未发现**差异（来源上限 −1、体力钳上限均与上游一致）；
//! 保留项：`ai.threaten`/`ai.result` 策略面；**无来源死亡**分支（`filter` 排除）未构造。
//!
//! 未覆盖（登记）：无来源死亡（如体力流失致死）分支；`dieBegin` 与死亡结算的**顺序**细节。

mod nc08_common;
use engine_native::Engine;
use nc08_common::{advance_to_play, drain};

/// 运筹帷幄：**双产物桥接**（武将面在 `character.json`）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹 character.json");
    let mut pack = content_native_loader::bridge_pack(&json)
        .expect("桥接 pack 段")
        .pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1+3（**行为面**）：持技者被击杀 → **来源失去 1 点体力上限**。
#[test]
fn upstream_chanyuan_killer_loses_max_hp() {
    let mut e = yunchou_engine(20270401);
    e.debug_grant(1, "ycndiychanyuan"); // 1 号持蝉缘（将被击杀）
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let max0_before = e.state_core().player(0).max_hp;
    let max1 = e.state_core().player(1).max_hp as i32;
    // 0 号造成**致命伤害**（真实伤害路径）→ 1 号死亡 → dieBegin 触发蝉缘
    e.debug_deal_damage(
        Some(0),
        1,
        max1 as u32,
        engine_native::event::DamageNature::Normal,
    );
    drain(&mut e);
    let max0_after = e.state_core().player(0).max_hp;
    let p0 = e.state_core().player(0);
    eprintln!(
        "DBG 蝉缘：1 号 alive={}；来源（0 号）上限 {max0_before} → {max0_after}（hp={} 上限={}）",
        e.state_core().player(1).alive,
        p0.hp,
        p0.max_hp
    );
    assert!(!e.state_core().player(1).alive, "夹具前提：1 号应已被击杀");
    assert_eq!(
        max0_after,
        max0_before - 1,
        "蝉缘：击杀者（来源）应**失去 1 点体力上限**（{max0_before} → {max0_after}）"
    );
    assert!(
        p0.hp <= max0_after as i32,
        "蝉缘：体力应被钳到新上限（hp={} ≤ 上限={}）",
        p0.hp,
        p0.max_hp
    );
}

/// 上游点 1+2（**源码事实**）：时机、门槛与效果式。
#[test]
fn upstream_chanyuan_source_matches() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    let idx = src.find("YunchouDiychanyuan =>").expect("蝉缘分支");
    let seg: String = src[idx..].chars().take(220).collect();
    assert!(
        seg.contains("lose_max_hp(src, 1)"),
        "原生依据：来源失去 1 点体力上限：{seg}"
    );
    let up = std::fs::read_to_string(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
    ))
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\tdiychanyuan: {").expect("上游蝉缘");
    let seg: String = up[idx..].chars().take(400).collect();
    assert!(
        seg.contains("dieBegin")
            && seg.contains("event.source !== undefined")
            && seg.contains("trigger.source.loseMaxHp(true)"),
        "上游依据：dieBegin + 有来源 + 来源 loseMaxHp：{seg}"
    );
}
