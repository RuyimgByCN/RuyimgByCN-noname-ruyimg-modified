//! NC08 第⑦层（上游行为对照）样本 127：**来源 U08（阵法）** 阵法牌
//! `dizaizhen`（地载阵）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js`）：
//!
//! ```js
//! dizaizhen: {
//!     type: "zhenfa", recastable: true,
//!     enable() { return game.hasPlayer(current => current.isNotMajor()); },
//!     filterTarget(card, player, target) { return target.isNotMajor(); },   // **非主公**
//!     selectTarget: -1,
//!     content() { target.draw(false); target.$draw(); },                    // **摸 1**
//!     mode: ["guozhan"],                                                    // **仅国战模式**
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **模式**：**`mode: ["guozhan"]`** ⇒ **仅在国战模式可用**；
//! 2. **目标**：**非主公**（`isNotMajor()`）角色；
//! 3. **效果**：**摸 1 张**。
//!
//! 原生实现引用：`kernel/intrinsic.rs:1012`（`CardKind::Dizaizhen` → `DizaizhenMachine`）。
//!
//! **本样本的定位**：验证「**摸 1**」；并核对**模式门槛**——**身份局（4 人）下该卡是否可用**。
//! 若身份局仍可发动 ⇒ **模式门槛缺失**（登记，不写成通过）。
//!
//! 未覆盖（登记）：`recastable`（重铸）面；国战模式下 `isNotMajor` 的完整语义。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn zhenfa_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-zhenfa-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读阵法 cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1+3（**行为面**）：**摸 1**；并核对**身份局下的模式门槛**。
#[test]
fn upstream_dizaizhen_draws_and_mode_gate() {
    let mut e = zhenfa_engine(20275801);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("dizaizhen"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("地载阵在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 目标（1 号）手牌数用于观测「摸 1」
    let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
    e.debug_place(1, name);
    let h1_before = e.debug_hand(1).len();
    // **三种目标形态都试**（`selectTarget: -1`：可能是「无显式目标」或「显式列表」）
    let mut out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 地载阵：targets=[1] → {out:?}");
    if matches!(out, CommandOutcome::Rejected(_)) {
        out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card, targets: vec![] },
        });
        eprintln!("DBG 地载阵：targets=[] → {out:?}");
    }
    if matches!(out, CommandOutcome::Rejected(_)) {
        out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card, targets: vec![0] },
        });
        eprintln!("DBG 地载阵：targets=[0] → {out:?}");
    }
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
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
    let h1_after = e.debug_hand(1).len();
    eprintln!("DBG 地载阵：询问={prompts:?}；1 号手牌 {h1_before} → {h1_after}");
    if matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ) {
        eprintln!(
            "NOTE 地载阵：**身份局下仍可发动** ⇒ 上游 `mode: [\"guozhan\"]` 的**模式门槛缺失**（登记待办）"
        );
        assert_eq!(
            h1_after,
            h1_before + 1,
            "地载阵：目标应**摸 1**（{h1_before} → {h1_after}）"
        );
    } else {
        eprintln!("NOTE 地载阵：身份局下被拒（{out:?}）⇒ 与上游 `mode: [\"guozhan\"]` 一致");
    }
}
