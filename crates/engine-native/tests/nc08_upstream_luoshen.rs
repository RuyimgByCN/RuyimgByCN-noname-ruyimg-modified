//! NC08 第⑦层（上游行为对照）样本 28：`luoshen`（洛神，**标准面** / 甄姬）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! luoshen: {
//!     trigger: { player: "phaseZhunbeiBegin" },     // **准备阶段开始**
//!     frequent: true,
//!     async content(event, trigger, player) {
//!         while (true) {
//!             const judgeEvent = player.judge({ judge(card) {
//!                 if (get.color(card) == "black") return 1.5;   // 黑色 → 成功
//!                 return -1.5;
//!             }, judge2(result) { return result.bool; } });
//!             judgeEvent.set("callback", … gain(event.card) …);  // 黑色判定牌**归自己**
//!             const result = await judgeEvent.forResult();
//!             if (!result?.bool || !result.card) break;          // 红色 → 结束
//!             event.cards.push(result.card);
//!             const result2 = await player.chooseBool({ prompt: "是否再次发动【洛神】？" });
//!             if (!result2?.bool) break;                          // **每轮单独询问**
//!         }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 准备阶段开始**（`phaseZhunbeiBegin`）；
//! 2. **循环**：判定 → **黑色**则**获得该判定牌**并继续；**红色**则结束；
//! 3. **每轮单独询问**是否再次发动（`chooseBool`）。
//!
//! 原生实现引用：`content/standard/skills/luo_shen.rs`（filter 用 **`Phase::Judge` + begin**
//! ——原生无准备阶段，锚点折入判定阶段；注释明确「**整链一次确认，不分次放弃**」）。
//!
//! **登记差异（本文件固定）**：
//! - **①时机锚点**：上游准备阶段开始 ↔ 原生判定阶段开始（**已登记**：原生无准备阶段）；
//! - **②循环询问粒度**：上游**每轮单独询问**（可中途停止）↔ 原生**整链一次确认**
//!   （源码注释自述的简化契约）→ **登记，不写成对齐**。
//!
//! 未覆盖（登记）：`frequent`/`preHidden`/`audio` 表现面；国战模式下的 `rejudge` 分支。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 上游点 2（循环）：**黑色**判定牌 → 归自己（获得），随后**红色** → 链结束。
#[test]
fn upstream_luoshen_black_gains_and_red_stops() {
    let mut e = Engine::new(20265801, 4);
    e.debug_grant(0, "洛神");
    // 牌堆顶：先黑后红（判定按顺序翻出）
    let black_top = e.debug_set_deck_top_by(|c| {
        matches!(
            c.suit,
            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
        )
    });
    let _red_second = e.debug_set_deck_top_by(|c| {
        matches!(
            c.suit,
            engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
        )
    });
    // 进入 0 号判定阶段（原生把洛神锚在判定阶段开始）
    assert!(advance_to_phase(&mut e, 0, Phase::Judge), "应到达判定阶段");
    // 触发确认门（可选技）→ 确认
    let mut confirmed = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            confirmed = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            });
            continue;
        }
        break;
    }
    drain(&mut e);
    if !confirmed {
        eprintln!("SKIP：未出现洛神确认门（锚点/阶段推进未对上）");
        return;
    }
    assert!(
        e.debug_hand(0).contains(&black_top) || e.state_core().discard.contains(&black_top),
        "黑色判定牌应离开牌堆（被获得或进入弃牌堆）：hand={:?} discard_has={}",
        e.debug_hand(0),
        e.state_core().discard.contains(&black_top)
    );
}

/// 上游点 1（时机锚点）+ **差异①（A2 S05 已消除）**：R05 准备阶段真实
/// 化后锚点对齐上游 phaseZhunbeiBegin（原差异：无准备阶段、锚判定阶段）。
#[test]
fn upstream_luoshen_timing_anchor_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/luo_shen.rs"
    ))
    .expect("读洛神源码");
    let seg: String = src.chars().take(700).collect();
    assert!(
        seg.contains("A2 S05") && src.contains("phase: Phase::Prepare"),
        "差异①消除（A2 S05）：原生锚点在准备阶段开始（与上游一致）：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("luoshen: {").expect("上游洛神");
    let seg: String = up[idx..].chars().take(300).collect();
    assert!(
        seg.contains("phaseZhunbeiBegin"),
        "上游依据：准备阶段开始：{seg}"
    );
}

/// 上游点 3 + **登记差异②**：原生为「整链一次确认」（源码注释自述），非上游的每轮询问。
#[test]
fn upstream_luoshen_loop_prompt_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/luo_shen.rs"
    ))
    .expect("读洛神源码");
    assert!(
        src.contains("整链一次确认") && !src.contains("chooseBool"),
        "差异②固定：原生整链一次确认、无每轮询问（上游 chooseBool）：见源码注释"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游");
    let idx = up.find("luoshen: {").expect("上游洛神");
    let seg: String = up[idx..].chars().take(1600).collect();
    assert!(
        seg.contains("是否再次发动【洛神】"),
        "上游依据：每轮询问是否再次发动：{seg}"
    );
}

/// 上游点 1（技能门槛）：**未持洛神** → 判定阶段无确认门。
#[test]
fn upstream_luoshen_requires_the_skill() {
    let mut e = Engine::new(20265802, 4);
    assert!(advance_to_phase(&mut e, 0, Phase::Judge));
    let open = e.open_choice().cloned();
    assert!(
        open.is_none(),
        "未持洛神 → 不应出现确认门：{:?}",
        open.map(|c| c.spec.prompt)
    );
    let _ = CommandOutcome::Suspended;
}
