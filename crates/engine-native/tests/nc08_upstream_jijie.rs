//! NC08 第⑦层（上游行为对照）样本 62：`xinfu_jijie`（**机捷**，标准面 / 伊籍）。
//!
//! **命名提示**：上游键 `xinfu_jijie` 在 `translate.js` 中映射为「**机捷**」（**不是**「新服结姻」），
//! 原生常量为 `JI_JIE`（`names.rs:46` 注释「伊籍「机捷」：出牌阶段限一次，取牌堆底一张牌交给一名角色」）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! xinfu_jijie: {
//!     enable: "phaseUse", usable: 1,                 // 出牌阶段**限一次**
//!     async content(event, trigger, player) {
//!         const card = get.bottomCards()[0];          // **牌堆底**一张
//!         game.cardsGotoOrdering(card);
//!         event.card = card;
//!         const { bool, targets } = await player.chooseTarget({
//!             forced: true,                           // **强制选目标**（无 filterTarget → 任意存活角色，含自己）
//!             ai(target) { … },
//!         });
//!         …
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/限次**：出牌阶段、**每回合限一次**；
//! 2. **取牌**：**牌堆底**一张（`get.bottomCards()[0]`）；
//! 3. **目标**：`chooseTarget({forced: true})` 且**无 `filterTarget`** → **任意存活角色（含自己）**。
//!
//! 原生实现引用：`content/standard/skills/ji_jie.rs`（注释「出牌阶段**限一次**主动技——取**牌堆底**
//! 一张牌交给一名**任意存活角色**（上游 `chooseTarget` **无过滤器**，AI 含自引加成，原生**同允许自选**）」）。
//!
//! **登记差异**：本样本**未发现**差异（取牌位置、目标范围、限次均与上游一致）；保留项：
//! `ai(target)` 的自引加成评分与 `audio` 表现面。
//!
//! 未覆盖（登记）：**行为面**（牌堆底牌移动的逐帧验证；本样本为源码事实级）。

use engine_native::Engine;

/// 上游点 2（**牌堆底**）：原生实现取牌堆底一张。
#[test]
fn upstream_jijie_takes_bottom_card() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/ji_jie.rs"
    ))
    .expect("读机捷源码");
    let head: String = src.chars().take(1400).collect();
    assert!(
        head.contains("牌堆底"),
        "原生注释应写明取『牌堆底』一张：{head}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\txinfu_jijie: {").expect("上游机捷");
    let seg: String = up[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("get.bottomCards()[0]"),
        "上游依据：取牌堆底一张：{seg}"
    );
    assert!(
        seg.contains("forced: true"),
        "上游依据：chooseTarget 为 forced：{seg}"
    );
}

/// 上游点 3（**目标无过滤器 → 含自己**）：原生允许自选。
#[test]
fn upstream_jijie_allows_self_target() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/ji_jie.rs"
    ))
    .expect("读机捷源码");
    // 注释按行宽折行（「chooseTarget 无\n过滤器」）→ 用短片段断言
    let head: String = src.chars().take(400).collect();
    assert!(
        head.contains("chooseTarget 无") && head.contains("过滤器") && head.contains("同允许自选"),
        "原生注释应说明上游无 filterTarget、原生同允许自选：{head}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游");
    let idx = up.find("\txinfu_jijie: {").expect("上游机捷");
    let seg: String = up[idx..].chars().take(1600).collect();
    assert!(
        !seg.contains("filterTarget"),
        "上游依据：chooseTarget **无** filterTarget（任意存活角色，含自己）"
    );
    // 命名提示：上游键在 translate.js 映射为「机捷」
    let tr = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/translate.js"),
    )
    .expect("读 translate.js");
    assert!(
        tr.contains("xinfu_jijie: \"机捷\""),
        "命名依据：translate.js 中 xinfu_jijie → 「机捷」"
    );
    let _ = Engine::new(20269401, 4);
}
