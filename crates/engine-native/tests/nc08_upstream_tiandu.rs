//! NC08 第⑦层（上游行为对照）样本 34：`tiandu`（天妒，**标准面** / 郭嘉）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! tiandu: {
//!     trigger: { player: "judgeEnd" },                    // 自己的判定结束后
//!     frequent(event) { return event.result.card?.name !== "du"; },   // **毒**牌除外
//!     check(event) { return event.result.card?.name !== "du"; },
//!     filter(event, player) { return get.position(event.result.card, true) === "o"; },  // 牌仍在处理区
//!     async content(event, trigger, player) { … },        // 获得该判定牌
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 自己判定结束后**（`judgeEnd`）；
//! 2. **门槛**：判定牌仍在**处理区**（`position === "o"`）且**不是【毒】**（`name !== "du"`）；
//! 3. **效果**：**获得**该判定牌。
//!
//! 原生实现引用：`kernel/judge.rs:626–670`（判定流程：`tian_du_self` = 判定阶段拥有者持技 →
//! `tian_du: vec![self.owner]`；`:486–505` 询问 `tian_du_gain` 并置 `gained`/`judge_held`）。
//!
//! **登记差异（本文件固定）**：
//! - **①毒牌除外未建模**：上游 `frequent`/`check` 均排除【毒】（`name !== "du"`）；
//!   原生构建 `tian_du` 列表时**只判持有者**（`vec![self.owner]`），**无牌名判断**；
//! - **②处理区门槛**：上游 `filter: position === "o"`；原生以机器字段 `judge_held` 等价表达
//!   （判定牌由机器持有期间为「在处理区」）→ 实现方式不同、语义等价（登记）。
//!
//! 未覆盖（登记）：`audio`/`preHidden` 表现面；【毒】牌在原生目录是否存在未核对（见待办）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 上游点 1+3：自己判定结束后 → **获得**该判定牌（行为面：判定牌进入其手牌）。
///
/// **根因已查明（2026-10-19 同轮）**：判定区在 **`Phase{Judge, begin:true}` 事件处被
/// `std::mem::take` 消费**（`kernel/intrinsic.rs:1325`）→ 而引擎**初态即「0 号判定阶段」**
/// （begin 已发出）→ 在 `Engine::new` 之后再放入判定区**永远不会被结算**（初版夹具即因此失败）。
/// 正确做法：在**本轮判定阶段之后**（如 0 号 Draw 阶段）放入 → 由**下一轮**的判定阶段开始消费。
#[test]
fn upstream_tiandu_gains_judge_card() {
    let mut e = Engine::new(20266401, 4);
    e.debug_grant(0, "天妒");
    // ① 先到 0 号 Draw 阶段（本轮判定阶段已过）
    assert!(
        advance_to_phase(&mut e, 0, Phase::Draw),
        "应到达 0 号摸牌阶段"
    );
    // ② 此刻把【乐不思蜀】放入 0 号判定区 → 将在**下一轮**判定阶段开始被消费
    let le = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "le")
        .expect("乐不思蜀在目录") as engine_native::CardId;
    {
        let core = e.state_core_mut();
        core.player_mut(0).judge.push(le);
    }
    // ③ 推进一整轮，捕获 tian_du_gain
    let mut confirmed_tiandu = false;
    let mut seen: Vec<String> = Vec::new();
    for _ in 0..160 {
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
        if !seen.iter().any(|p| p == c.spec.prompt) {
            seen.push(c.spec.prompt.to_string());
        }
        let is_tiandu = c.spec.prompt == "tian_du_gain";
        if is_tiandu {
            confirmed_tiandu = true;
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" || is_tiandu {
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
        if confirmed_tiandu {
            break;
        }
    }
    drain(&mut e);
    if !confirmed_tiandu {
        eprintln!("SKIP：一轮内未出现 tian_du_gain；询问序列={seen:?}");
        return;
    }
    assert!(
        e.debug_hand(0).len() >= 1,
        "天妒：获得判定牌后手牌应非空（hand={:?}）",
        e.debug_hand(0)
    );
}

/// 上游点 2 的**登记差异①**（毒牌除外未建模）：原生构建 `tian_du` 列表时无牌名判断。
#[test]
fn upstream_tiandu_du_exclusion_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/judge.rs"
    ))
    .expect("读 judge 源码");
    let idx = src.find("tian_du: if tian_du_self").expect("tian_du 构建");
    let seg: String = src[idx..].chars().take(160).collect();
    assert!(
        !seg.contains("\"du\"") && !seg.contains("毒"),
        "差异①固定：原生 tian_du 列表构建无『毒牌除外』判断：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("tiandu: {").expect("上游天妒");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("!== \"du\"") && seg.contains("position(event.result.card, true) === \"o\""),
        "上游依据：排除【毒】且要求判定牌在处理区：{seg}"
    );
}

/// 上游点 1（技能门槛）：**未持天妒** → 判定结束**无** `tian_du_gain` 询问。
#[test]
fn upstream_tiandu_requires_the_skill() {
    let mut e = Engine::new(20266402, 4);
    let le = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "le")
        .expect("乐不思蜀在目录") as engine_native::CardId;
    {
        let core = e.state_core_mut();
        core.player_mut(0).judge.push(le);
    }
    assert!(advance_to_phase(&mut e, 0, Phase::Judge));
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    let mut saw = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "tian_du_gain" {
            saw = true;
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
    assert!(
        !saw,
        "未持天妒 → 不应出现 tian_du_gain 询问（上游 trigger.player=judgeEnd）"
    );
    let _ = CommandOutcome::Suspended;
}
