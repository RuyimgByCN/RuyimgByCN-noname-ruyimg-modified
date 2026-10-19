//! NC08 第⑦层（上游行为对照）样本 61：`qingguo`（倾国，**标准面** / 甄姬）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! qingguo: {
//!     viewAs: { name: "shan" },                        // 当【闪】使用/打出
//!     viewAsFilter(player) {
//!         if (!player.hasCards("hs", { color: "black" })) return false;   // 黑色牌（hs）
//!     },
//!     position: "hs",                                  // **手牌 + 装备区**
//!     prompt: "将一张黑色手牌当闪使用或打出",
//!     ai: { order: 3, respondShan: true, … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **转化**：**黑色**牌当【闪】**使用或打出**（`viewAs`）；
//! 2. **牌区**：**`hs`（手牌 + 装备区）**；
//! 3. 与龙胆（闪当杀/杀当闪）**并列**出现在【闪】响应候选中。
//!
//! 原生实现引用：`kernel/sha/step.rs:374`/`:789`/`:1195`（倾国并入【闪】响应选项；
//! `:1192` 注释「倾国（甄姬，**v202 对齐上游 `position: "hs"`**）：黑色手牌可当闪打出」）。
//!
//! **登记差异（牌区族第 6 例）**：原生候选**只遍历 `.hand`**（`ctx.core.player(target).hand.iter()`）
//! → **装备区的黑色牌不能当【闪】**；上游 `position: "hs"` **含装备区** → **规则损失**
//! （与 `zhiheng`/`qixi`/`guose`/`wusheng`/`lijian` **同族**）。
//!
//! 未覆盖（登记）：**行为面**（黑色手牌当闪的完整响应链；本样本为源码事实级）；
//! `ai.order`/`respondShan` 策略面。

use engine_native::Engine;

/// 上游点 1+3（**源码事实**）：倾国并入【闪】响应候选，且与龙胆并列。
#[test]
fn upstream_qingguo_in_shan_options() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/sha/step.rs"
    ))
    .expect("读 sha/step");
    // 全文件检查（同文件有多个候选构造点，固定窗口会命中其它处）
    let seg: &str = &src;
    assert!(
        seg.contains("qingguo && black") && seg.contains("longdan && kind.is_sha_like()"),
        "原生依据：倾国（黑色）与龙胆并列进入【闪】候选：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tqingguo: {").expect("上游倾国");
    let seg: String = up[idx..].chars().take(1500).collect();
    assert!(
        seg.contains("viewAs: { name: \"shan\" }") && seg.contains("get.color(card)"),
        "上游依据：黑色牌当闪：{seg}"
    );
    assert!(
        seg.contains("position: \"hs\""),
        "上游依据：position 为 hs（含装备区）：{seg}"
    );
}

/// **登记差异（牌区族第 6 例）**：原生只遍历手牌，未含装备区。
#[test]
fn upstream_qingguo_zone_difference_is_registered() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/sha/step.rs"
    ))
    .expect("读 sha/step");
    // 定位到含 `qingguo && black` 的候选构造点（其上方即为 `.hand` 遍历）
    let anchor = src.find("qingguo && black").expect("倾国候选");
    let start = src[..anchor].rfind(".hand").expect("手牌遍历");
    let seg: String = src[start.saturating_sub(40)..anchor].chars().take(400).collect();
    assert!(
        seg.contains(".hand") && !seg.contains(".equips"),
        "差异固定：原生候选只取手牌（.hand），未含装备区（上游 position 'hs'）：{seg}"
    );
    // 与既有牌区族成员对照：guose/qixi 等在 play_card.rs 的命令面同样限定手牌
    let play_card = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_card.rs"
    ))
    .expect("读 play_card");
    assert!(
        play_card.contains("hand.contains(&card)"),
        "同族对照：出牌命令面亦限定手牌（play_card.rs）"
    );
    let _ = Engine::new(20269301, 4);
}
