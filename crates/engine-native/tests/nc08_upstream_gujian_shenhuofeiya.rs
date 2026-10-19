//! NC08 第⑦层（上游行为对照）样本 85：**来源 U05（古剑）** 机关牌
//! `shenhuofeiya`（神火飞鸦）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:906`）：
//!
//! ```js
//! shenhuofeiya: {
//!     type: "jiguan", enable: true, wuxieable: true,
//!     filterTarget(card, player, target) { return target !== player; },   // **只要求其他角色**
//!     changeTarget(player, targets) {                    // **额外影响**：与首目标距离 1（**无 he 条件**）
//!         game.filterPlayer(current => get.distance(targets[0], current, "pure") === 1, targets);
//!     },
//!     cardcolor: "red", cardnature: "fire",
//!     content() {
//!         "step 0"; var next = target.chooseToRespond({ name: "shan" });   // **须出【闪】**
//!         "step 1"; if (…) target.damage("fire");                          // **否则受 1 点火伤**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**其他角色**（**不要求** `he` 有牌——与同批 `qiankunbiao` **不同**）；
//! 2. **额外影响**（`changeTarget`）：与首目标**距离 1** 的角色（**亦无 `he` 条件**）；
//! 3. **效果**：受影响者各**须出【闪】**，否则受 **1 点火焰伤害**。
//!
//! 原生实现引用：`engine/play_card.rs:1684`（`CardKind::Qiankunbiao | CardKind::Shenhuofeiya`：
//! 恰 1 名其他存活目标）；`kernel/intrinsic.rs:253`（映射 `GJ_SHENHUOFEIYA_CARD_KEY`）。
//!
//! **本样本的作用（对修复清单第 35 项的核对）**：第 35 项指出乾坤镖**缺** `he` 门槛；
//! 本样本**反向确认**神火飞鸦**不应**有 `he` 门槛——目标 `he` 无牌时**必须仍被受理**（`Accepted`）。
//!
//! **登记差异**：本样本**未发现**差异（门槛与额外目标方向一致）；
//! 保留项：`wuxieable` 无懈面；`cardcolor/cardnature` 的属性面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
        .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1（**门槛反向确认**）：目标 `he` **无牌**仍应受理（与乾坤镖不同）。
#[test]
fn upstream_shenhuofeiya_target_without_he_is_legal() {
    let mut e = gujian_engine(20271501);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("shenhuofeiya"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("神火飞鸦在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 神火飞鸦（目标无 he 牌）→ {out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "神火飞鸦：上游 `filterTarget` **只要求其他角色**（无 `he` 门槛）→ 应受理：{out:?}"
    );
    drain(&mut e);
}

/// 上游点 2+3（**行为面**）：受影响者**无【闪】** → 各受 **1 点火伤**。
#[test]
fn upstream_shenhuofeiya_burns_who_cannot_dodge() {
    let mut e = gujian_engine(20271502);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("shenhuofeiya"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("神火飞鸦在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp_before: Vec<i32> = (0..4).map(|p| e.debug_hp(p)).collect();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "神火飞鸦应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
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
    drain(&mut e);
    let hp_after: Vec<i32> = (0..4).map(|p| e.debug_hp(p)).collect();
    eprintln!(
        "DBG 神火飞鸦：询问={prompts:?} hp {hp_before:?} → {hp_after:?}"
    );
    assert!(
        hp_after[1] < hp_before[1],
        "神火飞鸦：首目标（无【闪】）应受 1 点火伤（{hp_before:?} → {hp_after:?}）"
    );
    // 额外影响：与 1 号距离 1 的 2 号（无【闪】）亦应受 1 点火伤
    assert!(
        hp_after[2] < hp_before[2],
        "神火飞鸦（`changeTarget` 额外影响）：与首目标距离 1 的 2 号亦应受 1 点火伤\
         （{hp_before:?} → {hp_after:?}）"
    );
    assert_eq!(hp_after[0], hp_before[0], "神火飞鸦：使用者不是目标（0 号 hp 不变）");
}
