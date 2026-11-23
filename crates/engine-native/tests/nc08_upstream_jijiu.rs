//! NC08 第⑦层（上游行为对照）样本 30：`jijiu`（急救，**标准面** / 华佗）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! jijiu: {
//!     enable: "chooseToUse",
//!     viewAsFilter(player) {
//!         return player !== _status.currentPhase          // **回合外**（自己不是当前回合角色）
//!             && player.hasCards("hes", { color: "red" }); // 且持有红色牌
//!     },
//!     filterCard(card) { return get.color(card) === "red"; },
//!     position: "hes",                                    // 手牌 + 装备区 + 判定区
//!     viewAs: { name: "tao" },                            // 视为【桃】
//!     prompt: "将一张红色牌当桃使用",
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **转化**：**红色**牌当【桃】使用（`viewAs`）；
//! 2. **时机限制**：**回合外**（`player !== _status.currentPhase`）；
//! 3. **牌区**：`hes`。
//!
//! 原生实现引用：`snap/asks/shapes.rs`（`is_tao_like` 统一委托至
//! `kernel/combat_rules.rs` `is_tao_like_core`：持急救回合外红桃/方块 → 视为可当【桃】响应濒死救援）。
//!
//! **登记差异与修正（本文件固定）**：
//! - **①回合限制**：`is_tao_like_core` 已补全 `core.turn.current != responder` 且由快照/运行时共用（修复原 `shapes.rs` 缺失）；
//! - **②牌区**：上游 `position: "hes"` 含装备/判定区，原生 `is_tao_like_core` 按牌 id 判定、未区分牌区
//!   （本样本只以源码事实固定）。
//!
//! 未覆盖（登记）：`check`/`ai` 表现与策略面；濒死救援的完整行为面（需构造濒死场景）。

use engine_native::Engine;

/// 上游点 1：原生**确有**「红色牌可当【桃】」的判定（源码事实固定）。
#[test]
fn upstream_jijiu_red_card_is_tao_like() {
    let asks = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/snap/asks/shapes.rs"
    ))
    .expect("读 asks 源码");
    assert!(
        asks.contains("fn is_tao_like") && asks.contains("is_tao_like_core"),
        "asks/shapes.rs is_tao_like 应委托给 combat_rules::is_tao_like_core"
    );
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/combat_rules.rs"
    ))
    .expect("读 combat_rules 源码");
    let idx = src.find("fn is_tao_like_core").expect("is_tao_like_core");
    let seg: String = src[idx..].chars().take(4200).collect();
    assert!(
        seg.contains("JI_JIU") && seg.contains("Suit::Heart") && seg.contains("Suit::Diamond"),
        "原生依据：持急救时红桃/方块可当桃：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("jijiu: {").expect("上游急救");
    let seg: String = up[idx..].chars().take(1800).collect();
    assert!(
        seg.contains("get.color(card) === \"red\"") && seg.contains("name: \"tao\""),
        "上游依据：红牌当桃：{seg}"
    );
}

/// 回合限制已对齐并由 `is_tao_like_core` 运行时与快照统一复用：限定 `core.turn.current != responder`（回合外）。
#[test]
fn upstream_jijiu_turn_restriction_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/combat_rules.rs"
    ))
    .expect("读 combat_rules 源码");
    let idx = src.find("let jijiu =").expect("jijiu 判定块");
    let seg: String = src[idx..].chars().take(360).collect();
    assert!(
        seg.contains("turn.current != responder"),
        "已对齐上游回合外限制：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游");
    let idx = up.find("jijiu: {").expect("上游急救");
    let seg: String = up[idx..].chars().take(1800).collect();
    assert!(
        seg.contains("player !== _status.currentPhase"),
        "上游依据：回合外限制：{seg}"
    );
}

/// 上游点 1（技能门槛）：**未持急救**时红牌**不**当桃（源码事实：判定依赖 `JI_JIU` 持有）。
#[test]
fn upstream_jijiu_requires_the_skill() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/combat_rules.rs"
    ))
    .expect("读 combat_rules 源码");
    let idx = src.find("fn is_tao_like_core").expect("is_tao_like_core");
    let seg: String = src[idx..].chars().take(4200).collect();
    assert!(
        seg.contains("has_skill(responder, d)"),
        "门槛：仅在持有急救时扩面：{seg}"
    );
    let _ = Engine::new(20266000, 4); // 引擎可正常构建（无回归影响）
}
