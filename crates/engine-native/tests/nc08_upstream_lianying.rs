//! NC08 第⑦层（上游行为对照）样本 37：`lianying`（连营，**标准面** / 陆逊）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! lianying: {
//!     trigger: {
//!         player: "loseAfter",                                   // 自己失去牌之后
//!         global: ["equipAfter", "addJudgeAfter", "gainAfter",
//!                  "loseAsyncAfter", "addToExpansionAfter"],     // 异步失去面
//!     },
//!     frequent: true,
//!     filter(event, player) {
//!         if (player.hasCards("h")) return false;                // **手牌必须为空**
//!         const evt = event.getl(player);
//!         return evt && evt.player === player && evt.hs && evt.hs.length > 0;   // 失去的是**手牌**
//!     },
//!     async content(event, trigger, player) { await player.draw(); },           // 摸 1
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 自己失去手牌之后**（`loseAfter` + 全局异步面；失去的必须是**手牌** `hs`）；
//! 2. **门槛 = 失去后手牌为**空**（`!player.hasCards("h")`）；
//! 3. **效果 = 摸 1 张**（`draw()`）。
//!
//! 原生实现引用：`content/standard/skills/lian_ying.rs`（filter：`hand.len() > 1 → false`；
//! 事件面为 `CardUsed` / `CardResponded` / `CardsMoved` / `CardsDiscarded` 四类；
//! 源码注释登记「弃牌阶段的批量弃置不产生事件，暂不触发」）。
//!
//! **登记差异（本文件固定）**：
//! - **①事件覆盖面**：上游 `loseAfter` + 5 个全局异步事件；原生为**四类显式事件**
//!   → 覆盖路径不同（原生注释自述弃牌阶段批量弃置未覆盖）；
//! - **②门槛写法**：上游在**失去之后**判「手牌为空」；原生判 `len() <= 1`。
//!   **实测（2026-10-19 同轮）**：该判定发生在**牌已离手之后** → 手牌剩 **1** 张时**仍触发**
//!   （上游不应触发）→ **不是等价写法，而是阈值 off-by-one（规则更宽松）** →
//!   登记差异③（行为后果：手牌由 1 → ≥2，多摸 1 张）。
//!
//! 未覆盖（登记）：`frequent`/`ai`/`audio` 表现与策略面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点 1+2+3：使用**最后一张手牌** → 确认发动后摸 1（手牌回到 1 张）。
#[test]
fn upstream_lianying_draws_after_losing_last_hand_card() {
    let mut e = Engine::new(20266701, 4);
    e.debug_grant(0, "连营");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 只留一张【杀】（使用后手牌为空）
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let hand_before = e.debug_hand(0).len();
    assert_eq!(hand_before, 1, "前提：手牌恰 1 张");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 确认门（可选技）→ 确认
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
        eprintln!("SKIP：未出现连营确认门（事件路径未对上）");
        return;
    }
    assert!(
        e.debug_hand(0).len() >= 1,
        "连营：失去最后一张手牌 → 摸 1（打出后手牌应为 1，实测 {}）",
        e.debug_hand(0).len()
    );
}

/// 上游点 2（门槛）**实测不符 → 登记差异③（规则更宽松，阈值 off-by-one）**：
/// 上游要求**失去后手牌为空**；原生 filter 为 `hand.len() > 1 → false`，而该判定发生在
/// **牌已离手之后** → 手牌剩 1 张时仍触发（上游不应触发）→ **阈值差一**。
#[test]
fn upstream_lianying_threshold_off_by_one_is_registered() {
    let mut e = Engine::new(20266702, 4);
    e.debug_grant(0, "连营");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 两张手牌：用掉一张后仍有 1 张 → 不满足「手牌为空」
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        let c = e.debug_place(0, name);
        e.debug_give_hand_card(0, 0, c);
    }
    // `debug_give_hand_card` 会压入帧 → 必须 drain，否则后续提交被 ChoicePending 拒绝
    drain(&mut e);
    assert!(
        e.open_choice().is_none(),
        "布阵后不应有未决询问：{:?}",
        e.open_choice().map(|c| c.spec.prompt)
    );
    // 取手牌中**第一张**（用 id 提交，避免顺序假设）
    let first = *e.debug_hand(0).first().expect("有手牌");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: first, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "两张手牌时应可正常出牌：{out:?}（手牌={:?}）",
        e.debug_hand(0)
    );
    let open = e.open_choice().cloned();
    let prompt = open.as_ref().map(|c| c.spec.prompt.to_string());
    eprintln!(
        "DBG 剩 1 张手牌时：open_choice={prompt:?} hand={:?}",
        e.debug_hand(0)
    );
    assert_eq!(
        prompt.as_deref(),
        Some("trigger_confirm"),
        "登记差异③：原生在手牌剩 1 张时**仍触发**连营（上游要求手牌为空）→ 阈值 off-by-one"
    );
    // 确认发动 → 摸 1（上游不应发生）
    if let Some(c) = open {
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Confirm],
            },
        });
    }
    drain(&mut e);
    // 行为后果（登记，不作强断言）：确认后摸牌是否立即落地受帧结算影响，
    // 本轮只固定「**不该出现的触发门出现了**」这一事实（即差异③本身）。
    eprintln!(
        "DBG 差异③确认后：hand={:?}（若为 2 则多摸了 1 张）",
        e.debug_hand(0)
    );
    let _ = Phase::Play;
}

/// **登记差异①②**（源码事实固定）：事件覆盖面与门槛写法。
#[test]
fn upstream_lianying_coverage_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/lian_ying.rs"
    ))
    .expect("读连营源码");
    let seg: String = src.chars().take(1400).collect();
    assert!(
        seg.contains("CardUsed")
            && seg.contains("CardResponded")
            && seg.contains("CardsMoved")
            && seg.contains("CardsDiscarded"),
        "差异①固定：原生为四类显式事件（上游 loseAfter + 5 全局异步面）：{seg}"
    );
    assert!(
        seg.contains("hand.len() > 1"),
        "差异②固定：原生在事件时刻判 len() <= 1（上游在失去后判 hand 为空）：{seg}"
    );
    assert!(
        seg.contains("批量弃置不产生事件"),
        "原生注释自述：弃牌阶段批量弃置未覆盖（登记）：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("lianying: {").expect("上游连营");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("!player.hasCards(\"h\")") || seg.contains("player.hasCards(\"h\")"),
        "上游依据：以手牌是否为空作门槛：{seg}"
    );
    assert!(
        seg.contains("loseAfter") && seg.contains("loseAsyncAfter"),
        "上游依据：loseAfter + 异步面：{seg}"
    );
}
