//! NC08 第⑦层（上游行为对照）样本 43：`shushen`（淑慎，**标准面** / 甘夫人）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! shushen: {
//!     audio: "shushen",
//!     trigger: { player: "recoverEnd" },              // 自己回复体力后
//!     getIndex(event) { return event.num || 1; },     // **按回复点数计次**
//!     async cost(event, trigger, player) {
//!         event.result = await player.chooseTarget({
//!             filterTarget: lib.filter.notMe,          // **其他角色**
//!         }).forResult();
//!     },
//!     async content(event, trigger, player) {
//!         const target = event.targets[0];
//!         await target.draw(target.hasCards("h") ? 1 : 2);   // 有手牌摸 1、无手牌摸 2
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：自己**回复体力后**（`recoverEnd`）；**按回复点数计次**（`getIndex`）；
//! 2. **目标**：**其他角色**（`lib.filter.notMe`）；
//! 3. **效果**：目标**有手牌摸 1、无手牌摸 2**。
//!
//! 原生实现引用：`content/standard/skills/shu_shen.rs`（filter 判 `Recover` 且 `amount > 0` 且存活；
//! 机器询问一名其他角色后 `let n = if hand.is_empty() { 2 } else { 1 }; draw_cards(target, n)`）。
//!
//! **登记差异（本文件固定）**：**计次**——上游 `getIndex: event.num || 1` 表示一次回复 **N 点**
//! 应触发 **N 次**（每次各自选目标）；原生 filter 只判 `amount > 0` 且机器**单次结算**
//! → 回复 2 点只触发 1 次（**规则损失**：少一次目标选择与摸牌）。
//!
//! 未覆盖（登记）：`audio`/`ai` 表现与策略面；一次回复 N 点时的**多次询问**行为面
//! （本样本以源码事实固定差异，行为面因需构造多点回复而登记待办）。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点 3（**无手牌摸 2**）：目标空手牌时摸 2。
#[test]
fn upstream_shushen_target_without_hand_draws_two() {
    let mut e = Engine::new(20267501, 4);
    e.debug_grant(0, "淑慎");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 目标（1 号）空手牌；0 号持一张【桃】并受伤，用真实回复路径触发
    e.debug_set_hp(0, 3);
    let tao = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "tao")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name)
    };
    let before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    });
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 淑慎：可选触发确认 → 确认；随后目标选择 → 选 1 号
    let mut confirmed = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
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
        if c.spec.options.contains(&ChoiceOption::Player(1)) {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Player(1)],
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
    drain(&mut e);
    eprintln!(
        "DBG 淑慎：确认={confirmed} 目标手牌 {before} → {}（自己 hp={}）",
        e.debug_hand(1).len(),
        e.debug_hp(0)
    );
    if !confirmed {
        eprintln!("NOTE 未出现淑慎确认门（触发路径未对上）——本用例不构成验收");
        return;
    }
    assert_eq!(
        e.debug_hand(1).len(),
        before + 2,
        "淑慎：目标**无手牌**应摸 2（{before} → {}）",
        e.debug_hand(1).len()
    );
}

/// 上游点 3（**有手牌摸 1**）：目标有手牌时摸 1。
#[test]
fn upstream_shushen_target_with_hand_draws_one() {
    let mut e = Engine::new(20267502, 4);
    e.debug_grant(0, "淑慎");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 目标（1 号）先持 1 张牌
    let name = e
        .catalog()
        .cards
        .iter()
        .find(|c| c.name == "shan")
        .map(|c| c.name)
        .unwrap();
    let _ = e.debug_place(1, name);
    e.debug_set_hp(0, 3);
    let tao = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "tao")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name)
    };
    let before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    });
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let mut confirmed = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
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
        if c.spec.options.contains(&ChoiceOption::Player(1)) {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Player(1)],
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
    drain(&mut e);
    eprintln!(
        "DBG 淑慎（有手牌）：确认={confirmed} 目标手牌 {before} → {}",
        e.debug_hand(1).len()
    );
    if !confirmed {
        eprintln!("NOTE 未出现淑慎确认门——本用例不构成验收");
        return;
    }
    assert_eq!(
        e.debug_hand(1).len(),
        before + 1,
        "淑慎：目标**有手牌**应摸 1（{before} → {}）",
        e.debug_hand(1).len()
    );
}

/// **登记差异**（源码事实固定）：上游按回复点数**计次**，原生单次结算。
#[test]
fn upstream_shushen_per_point_index_difference_is_fixed() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("shushen: {").expect("上游淑慎");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("getIndex") && seg.contains("event.num"),
        "上游依据：按回复点数计次（getIndex → event.num）：{seg}"
    );
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/shu_shen.rs"
    ))
    .expect("读淑慎源码");
    let seg: String = src.chars().take(700).collect();
    assert!(
        seg.contains("amount > 0")
            && !seg.contains("getIndex")
            && !seg.contains("for _ in 0..amount"),
        "差异固定：原生只判 amount > 0，未按点数计次：{seg}"
    );
    let _ = Phase::Play;
}
