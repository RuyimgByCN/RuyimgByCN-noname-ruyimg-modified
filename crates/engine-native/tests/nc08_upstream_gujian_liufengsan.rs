//! NC08 第⑦层（上游行为对照）样本 98：**来源 U05（古剑）** 锦囊
//! `liufengsan`（六风散）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`liufengsan`）：
//!
//! ```js
//! liufengsan: {
//!     type: "trick", enable: true, fullskin: true, filterTarget: true,   // 任意角色
//!     content() {
//!         var list = [];
//!         for (var i = 0; i < 2; i++) list.push(game.createCard("shan"));   // **凭空造 2 张【闪】**
//!         target.gain(list, "gain2");                                      // **目标获得之**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：任意角色（`filterTarget: true`）；
//! 2. **效果**：目标**获得 2 张【闪】**（上游为 `game.createCard("shan")` **生成**，
//!    而非从牌堆/弃堆取实体牌）。
//!
//! 原生实现引用：宿主行为（古剑锦囊机）—— 本样本以**行为面**观测「手牌 +2 且均为【闪】」。
//!
//! **登记差异（视实测）**：**生成 vs 实体取牌** —— 若原生从牌堆/弃堆取实体【闪】，
//! 则属「**生成 vs 实体取牌**」族（已有 `mujiaren`/`yangpijuan` 两例，本项为**第 3 例**）。
//!
//! 未覆盖（登记）：`ai.result` 策略面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
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

/// 上游点 2（**行为面**）：目标**获得 2 张【闪】**。
#[test]
fn upstream_liufengsan_grants_two_shan() {
    let mut e = gujian_engine(20272801);
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
            .find(|(_, c)| c.name.contains("liufengsan"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("六风散在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let deck_before = e.state_core().deck.len();
    let h1_before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 六风散：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "六风散应可发动：{out:?}"
    );
    drain(&mut e);
    let hand = e.debug_hand(1);
    let shan_count = hand
        .iter()
        .filter(|c| e.catalog().cards[**c as usize].name.contains("shan"))
        .count();
    let deck_after = e.state_core().deck.len();
    eprintln!(
        "DBG 六风散：1 号手牌 {h1_before} → {}（其中【闪】{shan_count} 张）；牌堆 {deck_before} → {deck_after}",
        hand.len()
    );
    assert_eq!(
        hand.len(),
        h1_before + 2,
        "六风散：目标应**获得 2 张牌**（{h1_before} → {}）",
        hand.len()
    );
    assert_eq!(
        shan_count, 2,
        "六风散：获得的 2 张应均为**【闪】**（上游 `game.createCard(\"shan\")`）"
    );
    // **登记差异线索**：上游为**生成**（牌堆不变）；若原生从牌堆取牌，则牌堆减少
    eprintln!(
        "DBG 六风散（生成 vs 实体）：牌堆 {deck_before} → {deck_after}（减少 {}）",
        deck_before as i64 - deck_after as i64
    );
}
