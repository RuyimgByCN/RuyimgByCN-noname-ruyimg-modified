//! NC08 第⑦层（上游行为对照）样本 63：`xinfu_jiyuan`（**急援**，标准面 / 伊籍）。
//!
//! **命名提示**：上游键 `xinfu_jiyuan` 在 `translate.js` 中映射为「**急援**」；
//! 原生常量 `JI_YUAN`（`names.rs:51`）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! xinfu_jiyuan: {
//!     trigger: { global: ["dying", "gainAfter", "loseAsyncAfter"] },   // **三个事件臂**
//!     getIndex(event, player) {
//!         if (event.name !== "loseAsync") return [event.player];
//!         return game.filterPlayer(c => c !== player && event.getg(c).length > 0).sortBySeat();
//!     },
//!     filter(event, player, triggername, target) {
//!         if (!target?.isIn()) return false;
//!         if (event.name === "dying") return true;                      // ① 濒死臂
//!         if (event.giver !== player) return false;                     // ② 给予臂（须由自己给出）
//!         if (event.name === "gain") return event.player != player && event.getg(target).length > 0;
//!         return game.hasPlayer(c => c !== player && event.getg(c).length > 0);   // ③ 失去臂
//!     },
//!     async content(event, trigger, player) { await event.targets[0].draw(); },   // 目标**摸一张**
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：**三个臂**——① 任意角色**进入濒死**；② **自己给出的牌**被获得（`gainAfter`）；
//!    ③ **异步失去**（`loseAsyncAfter`，如被弃置/被拿走）；
//! 2. **效果**：令**目标角色摸一张牌**（三臂共用同一 `content`）。
//!
//! 原生实现引用：`content/standard/skills/ji_yuan.rs`（`filter` 仅 `EventData::Dying { .. }`；
//! 文件头注释自述「**登记差异：仅濒死臂**」）。
//!
//! **登记差异（规则损失，原生已自述）**：原生**只实现濒死臂** ✗，**缺 `gainAfter`（给予臂）
//! 与 `loseAsyncAfter`（失去臂）** → 上游三臂中 **2 臂缺失**（**规则损失**，宜并入修复清单）。
//!
//! 未覆盖（登记）：**濒死臂的行为面**（需构造「角色进入濒死 + 伊籍存活」场景并驱动结算）；
//! `audio` 表现面。

use engine_native::Engine;

/// 上游点 2（**共用效果**）：原生濒死臂与上游 `content` 同为「目标摸一张」。
#[test]
fn upstream_jiyuan_effect_is_draw_one() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/ji_yuan.rs"
    ))
    .expect("读急援源码");
    let head: String = src.chars().take(700).collect();
    assert!(
        head.contains("濒死") && head.contains("摸一张牌"),
        "原生注释应写明『濒死时令其摸一张牌』：{head}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\txinfu_jiyuan: {").expect("上游急援");
    let seg: String = up[idx..].chars().take(2400).collect();
    assert!(
        seg.contains("await event.targets[0].draw()"),
        "上游依据：效果为目标摸一张：{seg}"
    );
    assert!(
        seg.contains("\"dying\", \"gainAfter\", \"loseAsyncAfter\""),
        "上游依据：三个事件臂：{seg}"
    );
}

/// **登记差异（规则损失）**：原生只实现濒死臂，缺给予臂与失去臂。
#[test]
fn upstream_jiyuan_missing_arms_are_registered() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/ji_yuan.rs"
    ))
    .expect("读急援源码");
    // filter 仅匹配 Dying
    let idx = src.find("fn filter").expect("filter");
    let seg: String = src[idx..].chars().take(200).collect();
    assert!(
        seg.contains("EventData::Dying"),
        "原生 filter 仅匹配 Dying：{seg}"
    );
    assert!(
        !src.contains("CardsMoved") && !src.contains("gainAfter") && !src.contains("loseAsync"),
        "差异固定：原生未见给予臂/失去臂的事件面"
    );
    let head: String = src.chars().take(300).collect();
    assert!(
        head.contains("登记差异") && head.contains("仅濒死臂"),
        "原生已自述该差异（仅濒死臂）：{head}"
    );
    let _ = Engine::new(20269501, 4);
}
