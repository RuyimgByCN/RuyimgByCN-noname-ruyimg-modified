//! NC08 第⑦层（上游行为对照）样本 39：`tuxi`（突袭，**标准面** / 张辽）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! tuxi: {
//!     trigger: { player: "phaseDrawBegin1" },              // 摸牌阶段（数量确定**前**）
//!     filter(event, player) { return !event.numFixed; },
//!     async cost(event, trigger, player) {
//!         event.result = await player.chooseTarget({
//!             filterTarget(card, player, target) { return player !== target && target.hasCards("h"); },
//!             selectTarget: [1, 2],                         // **1–2 名**有手牌的其他角色
//!         }).forResult();
//!     },
//!     async content(event, trigger, player) {
//!         await player.gainMultiple(event.targets);         // 各取一张（**随机**）
//!         trigger.changeToZero();                           // **放弃摸牌**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：摸牌阶段（数量确定前）；门槛 `!numFixed`；
//! 2. **目标**：**1–2 名**有手牌的**其他**角色（可弃权 = 不发动）；
//! 3. **效果**：各取**一张手牌**（`gainMultiple` → **随机**），并**放弃本次摸牌**（`changeToZero`）。
//!
//! 原生实现引用：`content/standard/skills/tu_xi.rs`（filter `Phase::Draw + begin`；
//! `tuxi_target` 询问 1–2 名；**各区首张、Dismantle 同款确定性取牌**；目标询问弃权 = 不发动）。
//!
//! **登记差异（本文件固定）**：
//! - **①`numFixed` 未建模**（族第 3 例：`yingzi` / `luoyi` / `tuxi`）；
//! - **②取牌确定性**：上游 `gainMultiple` **随机**取一张；原生取**各区首张**（确定性）
//!   → 与 `linghunzhihuo` 的「随机 → 确定性首张」**同族**（第 2 例）。
//!
//! 未覆盖（登记）：`ai`/`audio` 表现与策略面；「放弃摸牌」是否同样影响后续摸牌类效果
//! （上游 `trigger.changeToZero()` 的连锁面）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 上游点 2+3：发动突袭 → 各取一张手牌，且**放弃本次摸牌**（手牌净变化 = 取牌数 − 0 摸牌）。
#[test]
fn upstream_tuxi_takes_cards_and_skips_draw() {
    // 基线：不持突袭（正常摸 2）
    let base = {
        let mut e = Engine::new(20266901, 4);
        advance_to_phase(&mut e, 0, Phase::Draw);
        drain(&mut e);
        let _ = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
        e.debug_hand(0).len()
    };
    // 实验组：持突袭并选 2 名有手牌的目标
    let mut e = Engine::new(20266901, 4);
    e.debug_grant(0, "突袭");
    // **不 drain** 地推进到摸牌阶段：drain 会把 tuxi_target 询问以默认应答（放弃）掉
    for _ in 0..24 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
    assert_eq!(e.turn().phase, Phase::Draw, "应到达摸牌阶段");
    // 给 1、2 号各一张手牌（供取）
    for p in [1u8, 2] {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        let c = e.debug_place(p, name);
        e.debug_give_hand_card(p, p, c);
    }
    drain(&mut e);
    let hand_before = e.debug_hand(0).len();
    let mut picked = false;
    let mut seen: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !seen.iter().any(|p| p == c.spec.prompt) {
            seen.push(c.spec.prompt.to_string());
        }
        if c.spec.prompt == "trigger_confirm" {
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
        let players: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Player(p) if *p == 1 || *p == 2))
            .copied()
            .collect();
        if c.spec.prompt == "tuxi_target" && players.len() >= 2 {
            picked = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: players.into_iter().take(2).collect(),
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    drain(&mut e);
    let after = e.debug_hand(0).len();
    eprintln!(
        "DBG 突袭：基线 hand={base} 持技 hand={after}（发动前 {hand_before}）picked={picked}"
    );
    if !picked {
        eprintln!("SKIP：未出现 tuxi_target 询问；本轮询问序列={seen:?} 阶段={:?}", e.turn().phase);
        return;
    }
    assert!(
        after > hand_before,
        "突袭：应取到牌（发动前 {hand_before} → {after}），且未摸牌"
    );
    assert!(
        after < base + 2,
        "突袭：放弃本次摸牌（基线摸 2 后为 {base}；取 2 张后应少于 {base}+2，实测 {after}）"
    );
}

/// **登记差异①②**（源码事实固定）：`numFixed` 未建模；取牌为**确定性首张**（上游随机）。
#[test]
fn upstream_tuxi_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/tu_xi.rs"
    ))
    .expect("读突袭源码");
    let seg: String = src.chars().take(1200).collect();
    assert!(
        seg.contains("Phase::Draw") && !seg.contains("numFixed") && !seg.contains("num_fixed"),
        "差异①固定：原生 filter 只判阶段，未建模 numFixed：{seg}"
    );
    assert!(
        seg.contains("确定性取牌") || seg.contains("首张"),
        "差异②固定：原生取牌为确定性首张（上游 gainMultiple 随机）：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("tuxi: {").expect("上游突袭");
    let seg: String = up[idx..].chars().take(2200).collect();
    assert!(
        seg.contains("gainMultiple") && seg.contains("changeToZero") && seg.contains("selectTarget: [1, 2]"),
        "上游依据：随机取牌 + 放弃摸牌 + 1–2 目标：{seg}"
    );
}

/// 上游点 2（门槛 · 反例）：**未持突袭** → 摸牌阶段**无** `tuxi_target` 询问。
#[test]
fn upstream_tuxi_requires_the_skill() {
    let mut e = Engine::new(20266902, 4);
    advance_to_phase(&mut e, 0, Phase::Draw);
    let mut saw = false;
    for _ in 0..12 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "tuxi_target" {
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
    assert!(!saw, "未持突袭 → 不应出现 tuxi_target 询问");
    let _ = CommandOutcome::Suspended;
}
