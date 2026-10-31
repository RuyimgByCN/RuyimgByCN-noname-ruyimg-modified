//! NC08 第⑦层（上游行为对照）样本 83：**来源 U05（杀海拾遗·运筹）** 武将技
//! `guihan`（归汉）——**已注册但效果为空**（登记型样本）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`guihan`）：
//!
//! ```js
//! guihan: {
//!     enable: "chooseToUse", skillAnimation: "epic", limited: true,      // **限定技**
//!     filter(event, player) {
//!         if (event.type !== "dying") return false;     // **濒死时**
//!         if (player !== event.dying) return false;     // **自己濒死**
//!         return true;
//!     },
//!     filterTarget(card, player, target) {
//!         return target.hasSex("male") && player !== target;    // **男性其他角色**
//!     },
//!     content() {
//!         "step 0"; player.awakenSkill(event.name); player.recover();   // 自己回复 1
//!         "step 1"; player.draw(2);                                     // 自己摸 2
//!         "step 2"; target.recover();                                   // 目标回复 1
//!         "step 3"; target.draw(2);                                     // 目标摸 2
//!     },
//! },
//! ```
//!
//! 原生现状（**逐点核对**）：
//!
//! | 环节 | 现状 |
//! |---|---|
//! | 键与行为 | ✅ `keys.rs:579` `HostBehavior::WudianGuihan => "host:guihan"`（运筹 `character.json` 中 `ycnguihan` 的 `mapping` 正是 `host:guihan`） |
//! | 过滤器 | `filtering/common.rs:28` → **`false`**（作为**主动技**不随事件触发，**方向正确**） |
//! | 工厂 | `factory.rs:231` → `(None, None)`（无目标/牌注入） |
//! | **效果分支** | **`step/wudian_status.rs:82` → `Flow::Done`（空实现）** ✗ |
//!
//! ⇒ **归汉的激活路径没有效果实现**（既未回复/摸牌，也未 `awakenSkill`）——
//! 与「**未实现**」在效果面上等价，但成因不同（**已注册、效果为空**）。
//!
//! **登记**：并入修复清单第 32 项（运筹未实现技能清单，**由三项增至四项**）。
//!
//! 未覆盖（登记）：行为面（无实现 → 无从验收）；`limited`（限定技）在原生侧的表示方式。

use std::path::PathBuf;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 上游侧：限定技 + 自己濒死 + 男性其他目标 + 双方回复/摸牌（避免登记不存在的东西）。
#[test]
fn upstream_defines_guihan() {
    let p = manifest().join(
        "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
    );
    let s = std::fs::read_to_string(&p).expect("读上游 yunchou/skill.js");
    let idx = s.find("\tguihan: {").expect("上游归汉");
    let seg: String = s[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("limited: true") && seg.contains("event.type !== \"dying\""),
        "上游依据：限定技 + 濒死触发条件：{seg}"
    );
    assert!(
        seg.contains("target.hasSex(\"male\")") && seg.contains("player !== target"),
        "上游依据：目标为男性其他角色：{seg}"
    );
    assert!(
        seg.contains("player.recover()")
            && seg.contains("player.draw(2)")
            && seg.contains("target.recover()")
            && seg.contains("target.draw(2)"),
        "上游依据：自己回复 1/摸 2 + 目标回复 1/摸 2：{seg}"
    );
}

/// 原生侧：**已注册但效果为空**（固定现状；实现后请翻转本断言）。
#[test]
fn native_guihan_effect_is_empty() {
    let src = manifest().join("../engine-native-content/src/content/host");
    let step =
        std::fs::read_to_string(src.join("step/wudian_status.rs")).expect("读 wudian_status");
    let idx = step.find("WudianGuihan =>").expect("归汉分支");
    let seg: String = step[idx..].chars().take(120).collect();
    assert!(
        seg.contains("Flow::Done"),
        "登记：原生归汉的效果分支为**空实现**（Flow::Done）——实现后请翻转本断言：{seg}"
    );
    let filter = std::fs::read_to_string(src.join("filtering/common.rs")).expect("读 filtering");
    assert!(
        filter.contains("HostBehavior::WudianGuihan => false"),
        "登记：原生归汉过滤器恒 false（作为主动技方向正确）"
    );
    let keys = std::fs::read_to_string(
        manifest().join("../engine-native-kernel/src/vocabulary/host/keys.rs"),
    )
    .expect("读 keys");
    assert!(
        keys.contains("HostBehavior::WudianGuihan => \"host:guihan\""),
        "键映射存在：host:guihan（运筹 ycnguihan 的 mapping）"
    );
    // 运筹包侧：蔡文姬确实引用该键
    let chars = std::fs::read_to_string(
        manifest()
            .join("../../packages/extensions/shahai-yunchou-upstream-noname/assets/character.json"),
    )
    .expect("读运筹 character.json");
    assert!(
        chars.contains("\"ycnguihan\"") && chars.contains("\"mapping\": \"host:guihan\""),
        "包数据依据：蔡文姬的 ycnguihan 映射到 host:guihan"
    );
}
