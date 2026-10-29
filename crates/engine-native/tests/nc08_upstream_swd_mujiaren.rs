//! NC08 第⑦层（上游行为对照）样本 66：**来源 U05（杀海拾遗·swd）** 机关牌
//! `mujiaren`（木甲人）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:490`）：
//!
//! ```js
//! mujiaren: {
//!     enable: true, type: "jiguan", usable: 1, updateUsable: "phaseUse", forceUsable: true,
//!     wuxieable: true, selectTarget: -1,
//!     filterTarget(card, player, target) { return target === player; },   // **仅自己**
//!     content() {
//!         "step 0";
//!         var cards = target.getCards("h", c => get.type(c) !== "basic");  // **非基本手牌**
//!         if (cards.length) target.lose(cards)._triggered = null;          // **静默失去**（不触发技能）
//!         event.num = 1 + cards.length;                                    // **1 + 失去数**
//!         "step 1";
//!         var list = get.typeCard("jiguan");
//!         for (var i = 0; i < list.length; i++) if (lib.card[list[i]].derivation) list.splice(i--, 1);
//!         if (list.length) { for (var i = 0; i < event.num; i++) cards.push(game.createCard(list.randomGet()));
//!                            target.directgain(cards); }                   // **随机生成机关牌**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**仅自己**（`target === player`）；出牌阶段**限一次**；
//! 2. **代价**：**静默失去全部非基本手牌**（`_triggered = null` → 不触发失去类技能）；
//! 3. **收益**：**随机生成**「**1 + 失去数**」张机关牌（`createCard`，**排除 `derivation`**）。
//!
//! 原生实现引用：`content/host/step/swd_card_effects.rs:552`
//! （弃全部非基本手牌 → `num = 1 + dropped.len()` → 从**牌堆+弃堆**的 Jiguan 池随机取 num 张）。
//!
//! **登记差异（本文件固定）**：
//! - **①牌的来源**：上游 `game.createCard(...)`（按**牌定义池**生成**新牌**）↔ 原生从**实体牌堆/弃堆**
//!   取**已有牌**（原生注释自述「实体发现」）→ **同数量、不同来源**（牌堆构成不同时结果不同）；
//! - **②`derivation` 排除**：上游从池中剔除 `derivation` 卡；原生过滤条件为
//!   `card_kind == Jiguan && != self.card`，**未见 derivation 排除**（登记，未判定影响）。
//!
//! 未覆盖（登记）：`wuxieable` 无懈面；`forceUsable`/`updateUsable` UI 面。

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

/// 上游点 2+3（**行为面**）：失去非基本手牌 N 张 → 获得 **1+N** 张机关牌。
#[test]
fn upstream_mujiaren_trades_nonbasic_for_jiguan() {
    let mut e = swd_engine(20269801);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号：两张**非基本**牌（装备/锦囊）+ 木甲人本体
    let nonbasic1 = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.name.contains("armor"))
            .expect("装备牌") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let nonbasic2 = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.name.contains("dismantle"))
            .expect("锦囊牌") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.name.contains("mujiaren"))
            .expect("木甲人在目录") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hand_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        // 上游 `selectTarget: -1` + `filterTarget: target === player` → **目标为自己**
        action: GameAction::PlayCard { card, targets: vec![0] },
    });
    eprintln!("DBG 木甲人：发动={out:?}（发动前手牌 {hand_before} 张）");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "木甲人应可发动（仅自己）：{out:?}"
    );
    drain(&mut e);
    let hand = e.debug_hand(0);
    let jiguan = hand
        .iter()
        .filter(|c| e.catalog().card_kind(**c) == engine_native::catalog::CardKind::Jiguan)
        .count();
    eprintln!(
        "DBG 木甲人：手牌 {hand_before} → {}（其中机关牌 {jiguan} 张；非基本牌是否仍在={}/{}）",
        hand.len(),
        hand.contains(&nonbasic1),
        hand.contains(&nonbasic2)
    );
    assert!(
        !hand.contains(&nonbasic1) && !hand.contains(&nonbasic2),
        "木甲人：全部**非基本**手牌应被失去"
    );
    assert_eq!(
        jiguan, 3,
        "木甲人：应获得 **1 + 失去数(2) = 3** 张机关牌（实测 {jiguan}）"
    );
}

/// **登记差异①②**（源码事实固定）：牌的来源为实体池（非生成）；未见 `derivation` 排除。
#[test]
fn upstream_mujiaren_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/swd_card_effects.rs"
    ))
    .expect("读 swd_card_effects");
    let idx = src.find("HostBehavior::SwdMujiarenCard =>").expect("木甲人分支");
    let seg: String = src[idx..].chars().take(1400).collect();
    assert!(
        seg.contains("num = 1 + dropped.len()"),
        "原生依据：1 + 失去数：{seg}"
    );
    assert!(
        seg.contains("实体发现"),
        "差异①固定：原生注释自述『实体发现』（从牌堆/弃堆取实体牌）：{seg}"
    );
    assert!(
        !seg.contains("derivation"),
        "差异②固定：原生过滤未见 derivation 排除：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js",
        ),
    )
    .expect("读上游 swd/card.js");
    let idx = up.find("\tmujiaren: {").expect("上游木甲人");
    let seg: String = up[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("game.createCard(list.randomGet())"),
        "上游依据：按牌定义池**生成**新牌：{seg}"
    );
    assert!(
        seg.contains("derivation") && seg.contains("target === player"),
        "上游依据：排除 derivation + 仅自己：{seg}"
    );
}
