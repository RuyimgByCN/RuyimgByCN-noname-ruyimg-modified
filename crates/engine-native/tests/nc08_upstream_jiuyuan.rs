//! NC08 第⑦层（上游行为对照）样本 59：`jiuyuan`（救援，**标准面** / 孙权）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! jiuyuan: {
//!     trigger: { target: "taoBegin" },                 // 自己成为【桃】的目标时
//!     zhuSkill: true, forced: true,                    // **主公技 + 锁定**
//!     filter(event, player) {
//!         if (event.player === player) return false;   // 使用者非自己
//!         if (!player.hasZhuSkill("jiuyuan")) return false;   // **主公资格**
//!         if (event.player.group !== "wu") return false;      // 使用者为**吴**势力
//!         return true;
//!     },
//!     async content(event, trigger, player) { trigger.baseDamage++; },   // **回复量 +1**
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：自己（濒死时）**成为【桃】的目标**；**锁定**（`forced`）；
//! 2. **资格**：持技者须具**主公资格**（`hasZhuSkill`），且**用桃者为吴势力**、**非自己**；
//! 3. **效果**：该【桃】的**回复量 +1**（`baseDamage++` → 回复 2 点）。
//!
//! 原生实现引用：`kernel/dying.rs:160`（`has_skill(dying, JIU_YUAN)` + `responder != dying`
//! + `is_lord(dying)` + `effective_faction(responder) == Wu` → `amount = 2`；随后
//! `ctx.recover(dying, amount, Some(tao))`）。
//!
//! **登记差异（与 `hujia`/`jijiang` 同族，第 3 例）**：主公技资格用 `is_lord`，而其在**非身份局
//! 返回 true**（`state.rs` 注释「无身份体系的模式持技即可」）→ **原型模式下救援亦可生效**
//! （**已核对为有意的设计差异**，见修复清单第 17 项）。
//!
//! 未覆盖（登记）：**回复 2 点的行为面**（需构造「主公濒死 + 吴势力角色持桃救援」场景）；
//! `audio` 表现面。

use engine_native::Engine;

/// 上游点 2+3（**源码事实**）：原生资格四条件与回复量 +1 的实现。
#[test]
fn upstream_jiuyuan_qualification_and_bonus() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/dying.rs"
    ))
    .expect("读 dying");
    let idx = src.find("let jiuyuan").expect("救援判定");
    let seg: String = src[idx..].chars().take(900).collect();
    assert!(
        seg.contains("has_skill(self.dying, d)")
            && seg.contains("responder != self.dying")
            && seg.contains("is_lord(self.dying)")
            && seg.contains("Faction::Wu"),
        "原生依据：持技 + 非自己 + 主公资格 + 吴势力：{seg}"
    );
    assert!(
        seg.contains("let amount = if jiuyuan { 2 } else { 1 }"),
        "原生依据：救援使回复量变为 2：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tjiuyuan: {").expect("上游救援");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("zhuSkill: true")
            && seg.contains("event.player.group !== \"wu\"")
            && seg.contains("trigger.baseDamage++"),
        "上游依据：主公技 + 吴势力 + 回复量 +1：{seg}"
    );
}

/// **登记差异（同族第 3 例）**：主公技资格用 `is_lord`（非身份局为真 → 有意的设计差异）。
#[test]
fn upstream_jiuyuan_lord_gate_is_same_family() {
    let state = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/state.rs"
    ))
    .expect("读 state");
    let idx = state.find("pub fn is_lord").expect("is_lord");
    let seg: String = state[..idx].chars().rev().take(200).collect::<String>().chars().rev().collect();
    assert!(
        seg.contains("无身份体系的模式持技即可"),
        "登记：is_lord 注释说明『无身份体系的模式持技即可』（有意设计）：{seg}"
    );
    // 同族第 3 例：hujia / jijiang / jiuyuan 均以 is_lord 作主公技资格
    let sha = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/sha.rs"
    ))
    .expect("读 sha");
    let jijiang = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/jijiang.rs"
    ))
    .expect("读 jijiang");
    let dying = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/dying.rs"
    ))
    .expect("读 dying");
    assert!(
        sha.contains("is_lord(self.target)")
            && jijiang.contains("is_lord(owner)")
            && dying.contains("is_lord(self.dying)"),
        "同族对照：护驾/激将/救援三处均以 is_lord 作主公技资格（第 3 例）"
    );
    let _ = Engine::new(20269101, 4);
}
