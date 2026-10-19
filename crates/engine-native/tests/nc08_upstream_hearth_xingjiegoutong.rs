//! NC08 第⑦层（上游行为对照）样本 118：**来源 U07（hearth）** 锦囊
//! `xingjiegoutong`（星界沟通）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! xingjiegoutong: {
//!     type: "trick", enable: true,
//!     selectTarget: -1, modTarget: true, toself: true,
//!     filterTarget(card, player, target) { return player === target; },
//!     content() {
//!         target.gainMaxHp();                        // **体力上限 +1**
//!         target.recover();                          // **回复 1 点体力**
//!         target.discard(target.getCards("h"));      // **弃置全部手牌**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**仅自己**（`toself`）；
//! 2. **效果（三步）**：**体力上限 +1** → **回复 1** → **弃置全部手牌**。
//!
//! 原生实现引用：`kernel/intrinsic.rs:443/470`（`CardKind::Xingjiegoutong`）。
//! 规则基准 golden 表第 55 行记：「星界沟通（**上限+1 回 1 弃全部**）」。
//!
//! **本样本的定位**：验证**三步净效果**（上限 +1、体力 +1、手牌清零）。
//!
//! 未覆盖（登记）：`modTarget` 影响面；体力上限变化对**后续结算**（如濒死阈值）的影响。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **hearth 包**建局。
fn hearth_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-hearth-upstream-noname/assets/cards.json"))
        .expect("读 hearth 包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面 · 三步**）：**上限 +1**、**回复 1**、**弃全部手牌**。
#[test]
fn upstream_xingjiegoutong_three_steps() {
    let mut e = hearth_engine(20274801);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号受伤（便于观测回复），并持两张额外手牌（供「弃全部」）
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, max0 - 1);
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("xingjiegoutong"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("星界沟通在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(0, name);
    }
    let max_before = e.state_core().player(0).max_hp as i32;
    let hp_before = e.debug_hp(0);
    let h_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 星界沟通：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "星界沟通应可发动（仅自己）：{out:?}"
    );
    drain(&mut e);
    let p0 = e.state_core().player(0);
    let max_after = p0.max_hp as i32;
    let hp_after = e.debug_hp(0);
    let h_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 星界沟通：上限 {max_before} → {max_after}；体力 {hp_before} → {hp_after}；\
         手牌 {h_before} → {h_after}"
    );
    assert_eq!(
        max_after,
        max_before + 1,
        "星界沟通：应 **体力上限 +1**（{max_before} → {max_after}）"
    );
    assert_eq!(
        hp_after,
        hp_before + 1,
        "星界沟通：应 **回复 1 点体力**（{hp_before} → {hp_after}）"
    );
    assert_eq!(
        h_after, 0,
        "星界沟通：应 **弃置全部手牌**（{h_before} → {h_after}）"
    );
}
