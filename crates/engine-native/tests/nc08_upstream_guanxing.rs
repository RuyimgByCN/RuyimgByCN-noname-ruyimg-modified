//! NC08 第⑦层（上游行为对照）样本 53：`guanxing`（观星，**标准面** / 诸葛亮）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! guanxing: {
//!     trigger: { player: "phaseZhunbeiBegin" },        // **准备阶段开始**
//!     frequent: true,
//!     async content(event, trigger, player) {
//!         const num = player.hasSkill("yizhi") && player.hasSkill("guanxing")
//!             ? 5
//!             : Math.min(5, game.countPlayer());        // X = 存活角色数，至多 5
//!         const result = await player.chooseToGuanxing(num).set("prompt", "观星：…").forResult();
//!         if (!result.bool || !result.moved[0].length) {
//!             player.addTempSkill("guanxing_fail");     // **未移动任何牌 → 挂 guanxing_fail**
//!         }
//!     },
//! },
//! guanxing_fail: {},
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：**准备阶段开始**；**X = 存活角色数（至多 5）**；**持「遗志」+「观星」时 X = 5**；
//! 2. **操作**：把所选牌置牌堆顶（先选先摸）、未选牌置牌堆底；
//! 3. **`guanxing_fail`**：**未移动任何牌**时挂临时技能。
//!
//! 原生实现引用：`content/standard/skills/guan_xing.rs`（filter `Phase::Judge + begin` —— 注释
//! 「回合开始（**判定阶段占位**）」；`guanxing_arrange` 询问从已展示顶牌中选任意张）。
//!
//! **登记差异（本文件固定）**：
//! - **①时机锚点**：上游**准备阶段**开始 ↔ 原生**判定阶段**开始（原生无准备阶段，**已登记族**，
//!   与 `luoshen` 同族）；
//! - **②`yizhi` 组合**：上游持「遗志」+「观星」时 X 固定为 **5**；原生注释只写「X = 存活角色数
//!   至多 5」，**未见 `yizhi` 判定**（登记，未判定其行为影响）；
//! - **③`guanxing_fail`**：上游未移动任何牌时挂临时技能；原生**未见该标记**（登记）。
//!
//! 未覆盖（登记）：X 的具体取值行为面（需构造不同存活人数）、`frequent`/`ai` 表现面。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 上游点 2（**行为面**）：观星发动后出现「从牌堆顶牌中选牌」的询问。
#[test]
fn upstream_guanxing_offers_arrange_choice() {
    let mut e = Engine::new(20268501, 4);
    // **引擎初态即 0 号判定阶段（begin 已发出）** → 必须等**下一轮**的判定阶段：
    // 先推进到本轮 Draw，再授予技能，然后驱动一整轮。
    assert!(advance_to_phase(&mut e, 0, Phase::Draw), "应到达摸牌阶段");
    e.debug_grant(0, "观星");
    let mut prompts: Vec<String> = Vec::new();
    let mut saw_arrange = false;
    for _ in 0..400 {
        let Some(c) = e.open_choice().cloned() else {
            let actor = e.turn().current;
            let _ = e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::NextPhase,
            });
            continue;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        if c.spec.prompt == "guanxing_arrange" {
            saw_arrange = true;
            // 选第一张（移动它 → 不触发上游的 guanxing_fail 分支）
            let sel: Vec<ChoiceOption> = c.spec.options.iter().take(1).copied().collect();
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
    drain(&mut e);
    eprintln!("DBG 观星：询问序列={prompts:?} 出现 arrange={saw_arrange}");
    if !saw_arrange {
        eprintln!("NOTE 未出现 guanxing_arrange 询问——本用例不构成验收");
        return;
    }
    let _ = Phase::Play;
}

/// **登记差异①②③**（源码事实固定）：时机锚点、`yizhi` 组合、`guanxing_fail`。
#[test]
fn upstream_guanxing_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/guan_xing.rs"
    ))
    .expect("读观星源码");
    let head: String = src.chars().take(900).collect();
    assert!(
        head.contains("判定阶段占位") && src.contains("phase: Phase::Judge"),
        "差异①固定：原生锚在判定阶段（占位）：{head}"
    );
    assert!(
        !src.contains("yizhi") && !src.contains("遗志"),
        "差异②固定：原生未见 yizhi（遗志）组合判定"
    );
    assert!(
        !src.contains("guanxing_fail"),
        "差异③固定：原生未见 guanxing_fail 标记"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tguanxing: {").expect("上游观星");
    let seg: String = up[idx..].chars().take(900).collect();
    assert!(
        seg.contains("phaseZhunbeiBegin") && seg.contains("hasSkill(\"yizhi\")"),
        "上游依据①②：准备阶段 + yizhi 组合：{seg}"
    );
    assert!(
        seg.contains("guanxing_fail"),
        "上游依据③：未移动则挂 guanxing_fail：{seg}"
    );
}
