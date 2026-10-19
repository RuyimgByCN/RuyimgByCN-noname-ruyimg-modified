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
//! **登记差异（本文件固定）**：**目标门槛未在原生侧校验**——上游 `filterTarget` 要求
//! `!target.hasSkill("gouhunluo")`；原生该分支只做效果（`if alive`），**未见**对
//! 「目标已持该技」的拒绝（与同类 host 卡的门槛承载方式一致，**未核对** `filter_swd` 是否覆盖）。
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

/// 上游点 1（**门槛**）：上游要求目标**未持** `gouhunluo`；原生是否校验 → 源码事实登记。
#[test]
fn upstream_gouhunluo_target_gate_note() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/step/swd_card_effects.rs"
    ))
    .expect("读 swd_card_effects");
    let idx = src.find("HostBehavior::SwdGouhunluoCard =>").expect("勾魂锣分支");
    let seg: String = src[idx..].chars().take(900).collect();
    assert!(
        seg.contains("gouhunluo_rounds = 3") && seg.contains("gouhunluo_source = Some(self.owner)"),
        "原生依据：3 轮倒计时 + 来源记录：{seg}"
    );
    assert!(
        !seg.contains("has_skill") || !seg.contains("gouhunluo_rounds > 0"),
        "登记：原生该分支**未**校验「目标已持该技」（上游 filterTarget 要求）：{seg}"
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
