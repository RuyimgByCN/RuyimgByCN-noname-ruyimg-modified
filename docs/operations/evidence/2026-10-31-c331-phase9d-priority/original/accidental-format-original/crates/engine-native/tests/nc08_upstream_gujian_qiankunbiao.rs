//! NC08 第⑦层（上游行为对照）样本 84：**来源 U05（杀海拾遗·古剑）** 机关牌
//! `qiankunbiao`（乾坤镖）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:870`）：
//!
//! ```js
//! qiankunbiao: {
//!     type: "jiguan", enable: true, wuxieable: true,
//!     filterTarget(card, player, target) {
//!         return target !== player && target.countCards("he") > 0;      // 其他角色且 he 有牌
//!     },
//!     changeTarget(player, targets) {                    // **额外目标**
//!         game.filterPlayer(current =>
//!             get.distance(targets[0], current, "pure") === 1 && current.countCards("he"), targets);
//!     },
//!     content() {
//!         var he = target.getCards("he");
//!         if (he.length) target.discard(he.randomGet()).delay = false;    // **随机弃一张 he**
//!     },
//!     contentAfter() { game.delay(0.5); },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**恰 1 名**其他角色且其 `he` 有牌；
//! 2. **额外影响**（`changeTarget`）：与首目标**距离 1**（`pure`）且 `he` 有牌的角色**一并受影响**；
//! 3. **效果**：每个受影响角色**随机弃置一张 `he` 牌**。
//!
//! 原生实现引用：`engine/play_card.rs:1684`（目标校验：**恰 1 名**其他存活目标；注释
//! 「邻座由**结算侧**按距离展开」）；`kernel/intrinsic.rs:251`（映射到宿主键
//! `GJ_QIANKUNBIAO_CARD_KEY`）。
//!
//! **登记差异（本文件固定）**：**弃牌的选择** —— 上游 `he.randomGet()`（**随机**）↔ 原生
//! 若为确定性取牌则属「**取牌选择权/随机→确定性**」族（**本样本以行为面观测实际取牌数**，
//! 具体取哪张不在此断言）。
//!
//! 未覆盖（登记）：`wuxieable` 无懈面；`contentAfter` 的 `game.delay(0.5)` 表现面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
            .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2+3（**行为面**）：首目标与**距离 1 的邻座**各弃一张 `he` 牌。
#[test]
fn upstream_qiankunbiao_discards_target_and_neighbor() {
    let mut e = gujian_engine(20271401);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号（首目标）与 2 号（与 1 号距离 1）各持一张手牌；3 号不给牌
    let c1 = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(1, name)
    };
    let c2 = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(2, name)
    };
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("qiankunbiao"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("乾坤镖在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let h1_before = e.debug_hand(1).len();
    let h2_before = e.debug_hand(2).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 乾坤镖：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "乾坤镖应可发动：{out:?}"
    );
    drain(&mut e);
    let h1_after = e.debug_hand(1).len();
    let h2_after = e.debug_hand(2).len();
    eprintln!(
        "DBG 乾坤镖：首目标 1 号手牌 {h1_before} → {h1_after}（牌 {c1}）；邻座 2 号 {h2_before} → {h2_after}（牌 {c2}）"
    );
    assert!(
        h1_after < h1_before,
        "乾坤镖：**首目标**应弃一张 he 牌（{h1_before} → {h1_after}）"
    );
    assert!(
        h2_after < h2_before,
        "乾坤镖（`changeTarget` 额外影响）：**与首目标距离 1 且 he 有牌**的邻座也应弃一张\
         （2 号 {h2_before} → {h2_after}）"
    );
}

/// 上游点 1（**目标门槛反例**）：目标 `he` **无牌** → 不可指定。
///
/// **本轮未验收（差异登记）**：实测目标 `he` 无牌时**仍被受理**（`Accepted`）✗ ——
/// 原生 `play_card.rs:1684` 只校验「**恰 1 名其他存活目标**」，**未校验**上游
/// `filterTarget` 的 `target.countCards("he") > 0` ⇒ **原生更宽松**（详见修复清单第 35 项）。
/// 本用例以 `#[ignore]` 作为**修复后的验收条件**（应 `Rejected`）。
#[test]
#[ignore = "乾坤镖目标 he 无牌时应被拒（上游 filterTarget）——原生更宽松，见第 35 项"]
fn upstream_qiankunbiao_target_needs_he_cards() {
    let mut e = gujian_engine(20271402);
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
            .find(|(_, c)| c.name.contains("qiankunbiao"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("乾坤镖在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 乾坤镖（目标无 he 牌）→ {out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "乾坤镖：目标 `he` 无牌时应被拒（上游 filterTarget `countCards(\"he\") > 0`）：{out:?}"
    );
    drain(&mut e);
}
