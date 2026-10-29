//! NC08 第⑦层（上游行为对照）样本 65：**来源 U05（杀海拾遗·swd）** 机关牌
//! `gouhunluo`（勾魂锣）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:592`）：
//!
//! ```js
//! gouhunluo: {
//!     enable: true, type: "jiguan", wuxieable: true,
//!     filterTarget(card, player, target) { return !target.hasSkill("gouhunluo"); },  // 目标**未持**该技
//!     content() {
//!         target.storage.gouhunluo = 3;        // **3 轮倒计时**
//!         target.storage.gouhunluo2 = player;  // **来源记录**
//!         target.addSkill("gouhunluo");        // 授予倒计时子技
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**未持** `gouhunluo` 的角色（防重复叠加）；
//! 2. **效果**：目标挂 **3 轮**倒计时 + 记录**来源** + 授予倒计时子技。
//!
//! 原生实现引用：`content/host/step/swd_card_effects.rs:613`
//! （`pl.gouhunluo_rounds = 3`；`pl.gouhunluo_source = Some(self.owner)`；
//! 授予 `SWD_GOUHUNLUO_SUB_KEY` 实例）。
//!
//! **门槛（已核对，2026-10-19 更正）**：上游 `filterTarget` 要求 `!target.hasSkill("gouhunluo")`
//! → 原生**确在校验**，位置在 **`engine/play_card.rs:1534`**（出牌命令面）：
//! `if core.player(targets[0]).gouhunluo_rounds > 0 { return Err(RejectReason::IllegalTargets) }`
//! —— **不是**差异（初次登记为差异系**只读效果分支、未查完整校验链**所致，已更正）。
//!
//! 未覆盖（登记）：3 轮倒计时的**逐轮结算行为**（需跨多回合驱动）；`wuxieable` 的无懈面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：目标获得 **3 轮**倒计时 + **来源记录** + 倒计时子技。
#[test]
fn upstream_gouhunluo_marks_target() {
    let mut e = swd_engine(20269701);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.name.contains("gouhunluo"))
            .expect("勾魂锣在目录") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let rounds_before = e.state_core().player(1).gouhunluo_rounds;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 勾魂锣：发动={out:?}（目标倒计时前值={rounds_before}）");
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "勾魂锣应可发动：{out:?}"
    );
    drain(&mut e);
    let p1 = e.state_core().player(1);
    eprintln!(
        "DBG 勾魂锣：目标 rounds={} source={:?}",
        p1.gouhunluo_rounds, p1.gouhunluo_source
    );
    assert_eq!(p1.gouhunluo_rounds, 3, "勾魂锣：目标应挂 **3 轮**倒计时");
    assert_eq!(
        p1.gouhunluo_source,
        Some(0),
        "勾魂锣：应记录来源为发动者（0 号）"
    );
}

/// 上游点 1（**门槛行为面**）：已挂倒计时的目标**不可再次成为目标**（第二张勾魂锣被拒）。
#[test]
fn upstream_gouhunluo_rejects_already_marked_target() {
    let mut e = swd_engine(20269702);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 包内有两张勾魂锣（swd_gouhunluo1 / 2）
    let cards: Vec<engine_native::CardId> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.name.contains("gouhunluo"))
        .map(|(i, _)| i as engine_native::CardId)
        .collect();
    assert!(cards.len() >= 2, "包内应有两张勾魂锣，实测 {}", cards.len());
    for c in cards.iter().take(2) {
        nc08_common::put_hand(&mut e, 0, *c);
    }
    // 第一张：命中 1 号 → 挂 3 轮
    let out1 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: cards[0], targets: vec![1] },
    });
    assert!(
        matches!(
            out1,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "第一张勾魂锣应可发动：{out1:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(1).gouhunluo_rounds, 3, "目标应已挂倒计时");
    // 第二张：同一目标 → 应被拒（上游 filterTarget !hasSkill("gouhunluo")）
    let out2 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: cards[1], targets: vec![1] },
    });
    eprintln!("DBG 勾魂锣门槛：第二张对同一目标 = {out2:?}");
    assert!(
        matches!(out2, CommandOutcome::Rejected(_)),
        "已挂倒计时的目标不应再次成为目标（上游 filterTarget）：{out2:?}"
    );
}

/// 上游点 1（**门槛源码事实，已核对**）：原生在**出牌命令面**校验「目标未挂倒计时」。
#[test]
fn upstream_gouhunluo_target_gate_is_enforced() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/swd_card_effects.rs"
    ))
    .expect("读 swd_card_effects");
    let idx = src.find("HostBehavior::SwdGouhunluoCard =>").expect("勾魂锣分支");
    let seg: String = src[idx..].chars().take(900).collect();
    assert!(
        seg.contains("gouhunluo_rounds = 3") && seg.contains("gouhunluo_source = Some(self.owner)"),
        "原生依据：3 轮倒计时 + 来源记录：{seg}"
    );
    assert!(
        seg.contains("gouhunluo_rounds = 3"),
        "效果分支：挂 3 轮倒计时：{seg}"
    );
    // 门槛在**命令面**（不是效果分支）——须读完整校验链
    let play_card = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_card_targets.rs"
    ))
    .expect("读 play_card");
    let idx = play_card.find("gouhunluo_rounds > 0").expect("勾魂锣门槛");
    let gate: String = play_card[idx.saturating_sub(160)..idx].chars().take(400).collect();
    assert!(
        gate.contains("IllegalTargets"),
        "门槛依据：命令面对已挂倒计时目标返回 IllegalTargets：{gate}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js",
        ),
    )
    .expect("读上游 swd/card.js");
    let idx = up.find("\tgouhunluo: {").expect("上游勾魂锣");
    let seg: String = up[idx..].chars().take(520).collect();
    assert!(
        seg.contains("!target.hasSkill(\"gouhunluo\")")
            && seg.contains("target.storage.gouhunluo = 3")
            && seg.contains("target.storage.gouhunluo2 = player"),
        "上游依据：未持该技 + storage 3 + 记录来源：{seg}"
    );
}
