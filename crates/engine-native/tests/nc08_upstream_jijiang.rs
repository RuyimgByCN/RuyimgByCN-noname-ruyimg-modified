//! NC08 第⑦层（上游行为对照）样本 58：`jijiang`（激将，**标准面** / 刘备）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! jijiang: {
//!     group: ["jijiang1"], zhuSkill: true,                    // **主公技**
//!     filter(event, player) {
//!         if (!player.hasZhuSkill("jijiang")                      // **须具主公资格**
//!             || !game.hasPlayer(c => c !== player && c.group === "shu")) return false;  // 有蜀势力他人
//!         return !event.jijiang && (event.type !== "phase" || !player.hasSkill("jijiang3"));
//!     },
//!     enable: ["chooseToUse", "chooseToRespond"],
//!     viewAs: { name: "sha" },
//!     filterCard() { return false; },                         // **不用自己的牌**
//!     selectCard: -1,
//!     ai: { respondSha: true, … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **资格**：持技者须具**主公资格**（`hasZhuSkill`），且场上存在**其他蜀势力**角色；
//! 2. **用法**：**不使用自己的牌**（`filterCard → false`、`selectCard: -1`），由**蜀势力角色代打**
//!    【杀】；可用于**主动使用**与**响应**（`chooseToUse` / `chooseToRespond`）。
//!
//! 原生实现引用：`content/standard/skills/jijiang.rs`（`activate` 要求 `is_lord(owner)`
//! —— 注释「主公资格（**身份局须主公身份**）+ 存在蜀势力角色；**不以暗手牌预筛**——有无杀由
//! 被询问者逐人应答时才知道」；机器 `HelperAsk` → …）。
//!
//! **登记差异（与 `hujia` 同族）**：**主公技资格判定**——上游 `hasZhuSkill` **仅身份局**成立；
//! 原生用 `is_lord`，而其在**非身份局返回 true**（`state.rs` 的 `None => true`，注释「无身份体系的
//! 模式持技即可」）→ **原型模式下激将亦可发动**（**已核对为有意的设计差异**，见修复清单第 17 项）。
//!
//! 未覆盖（登记）：`jijiang1` 子技（代打询问队列）的**行为面**（需构造「需要【杀】」的场景）；
//! `jijiang3` 相关分支；`audio` 表现面。

use engine_native::Engine;

/// 上游点 1（**资格**）：原生 `activate` 的主公资格 + 蜀势力判定与上游 filter 对应（源码事实）。
#[test]
fn upstream_jijiang_qualification_matches() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/jijiang.rs"
    ))
    .expect("读激将源码");
    let head: String = src.chars().take(1600).collect();
    assert!(
        head.contains("is_lord(owner)") && head.contains("Faction::Shu"),
        "原生依据：主公资格 + 蜀势力判定：{head}"
    );
    assert!(
        head.contains("不以暗手牌预筛"),
        "原生注释：不以暗手牌预筛（与上游一致——有无杀由应答时才知道）"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tjijiang: {").expect("上游激将");
    let seg: String = up[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("hasZhuSkill(\"jijiang\")") && seg.contains("current.group === \"shu\""),
        "上游依据：主公技资格 + 其他蜀势力：{seg}"
    );
    assert!(
        seg.contains("filterCard() {") && seg.contains("selectCard: -1"),
        "上游依据：不用自己的牌（filterCard false / selectCard -1）：{seg}"
    );
}

/// 上游点 2（**不用自己的牌**）：原生以「代打」形态实现（无手牌也可请求）。
#[test]
fn upstream_jijiang_uses_helpers_not_own_cards() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/jijiang.rs"
    ))
    .expect("读激将源码");
    assert!(
        src.contains("HelperAsk"),
        "原生依据：以『代打（HelperAsk）』形态实现，不使用自己的牌"
    );
    // 与「需要自己的牌」的主动技对比：激将的 activate 不检查手牌
    let idx = src.find("fn activate").expect("activate");
    let seg: String = src[idx..].chars().take(700).collect();
    assert!(
        !seg.contains("hand.is_empty"),
        "差异对照：激将 activate **不**要求手牌非空（上游 filterCard 恒 false）：{seg}"
    );
}

/// **登记差异（与 `hujia` 同族）**：主公技资格判定在非身份局为真（有意的设计差异）。
#[test]
fn upstream_jijiang_lord_gate_is_design_difference() {
    let state = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/state.rs"
    ))
    .expect("读 state");
    let idx = state.find("pub fn is_lord").expect("is_lord");
    let seg: String = state[..idx].chars().rev().take(200).collect::<String>().chars().rev().collect();
    assert!(
        seg.contains("无身份体系的模式持技即可"),
        "登记：is_lord 的文档注释说明『无身份体系的模式持技即可』（有意设计）：{seg}"
    );
    // 与护驾同族：两处均用 is_lord 作主公技资格
    let hujia = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/sha.rs"
    ))
    .expect("读 sha");
    assert!(
        hujia.contains("is_lord(self.target)"),
        "同族对照：护驾亦用 is_lord 作主公技资格"
    );
    let _ = Engine::new(20269001, 4);
}
