//! NC08 第⑦层（上游行为对照）样本 115：**来源 U07（hearth）** 锦囊
//! `zhiliaobo`（治疗波）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! zhiliaobo: {
//!     enable: true, type: "trick",
//!     filterTarget(card, player, target) { return target.hp < target.maxHp; },   // **仅受伤角色**
//!     content() {
//!         "step 0"; target.judge(function (card) { return get.color(card) === "red" ? 1 : 0; });
//!         "step 1"; if (result.bool) target.recover();      // **红 → 回复 1**
//!                   else target.changeHujia();              // **非红 → +1 护甲**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**已受伤**（`hp < maxHp`）；
//! 2. **判定分支**：**红色 → 回复 1 点体力**；**非红 → +1 护甲**。
//!
//! 原生实现引用：`content/host/step/hearth.rs:44`（`HearthZhiliaoboCard`）；
//! `kernel/intrinsic.rs:441/455`（`CardKind::Zhiliaobo`）。
//!
//! **本样本的定位**：**两分支**（红/非红）+ **门槛反例**（满血不可指定）。
//!
//! 夹具要点：判定取**牌堆顶** ⇒ 用 `debug_set_deck_top(名)` **并按纪律回读 `deck.last()` 的牌色**，
//! 直到顶牌为所需颜色（同「置顶牌须回读核对」的教训）。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **hearth 包**建局。
fn hearth_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-hearth-upstream-noname/assets/cards.json"))
            .expect("读 hearth 包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 把牌堆顶置为**指定颜色**的牌（按纪律回读核对）。
fn top_of_color(e: &mut Engine, red: bool) -> bool {
    let names: Vec<String> = e
        .catalog()
        .cards
        .iter()
        .map(|c| c.name.to_string())
        .collect();
    for name in names {
        e.debug_set_deck_top(&name);
        if let Some(c) = e.state_core().deck.last().copied() {
            let suit = e.catalog().cards[c as usize].suit;
            let is_red = matches!(
                suit,
                engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
            );
            if is_red == red {
                return true;
            }
        }
    }
    false
}

/// 跑一次：判定顶牌为红/非红；返回 `(hp 变化, 护甲变化)`。
fn run(seed: u64, red: bool) -> (i32, i32) {
    let mut e = hearth_engine(seed);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号受伤（门槛要求 hp < maxHp）
    let max1 = e.state_core().player(1).max_hp as i32;
    e.debug_set_hp(1, max1 - 1);
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("zhiliaobo"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("治疗波在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    assert!(top_of_color(&mut e, red), "应能置出所需颜色的顶牌");
    let hp0 = e.debug_hp(1);
    let hj0 = e.state_core().player(1).hujia as i32;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "治疗波应可发动：{out:?}"
    );
    drain(&mut e);
    let d_hp = e.debug_hp(1) - hp0;
    let d_hj = e.state_core().player(1).hujia as i32 - hj0;
    eprintln!("DBG 治疗波：red={red} → 1 号 hp{d_hp:+}、护甲{d_hj:+}");
    (d_hp, d_hj)
}

/// 上游点 2（**行为面 · 两分支**）：**红 → 回复 1**；**非红 → +1 护甲**。
#[test]
fn upstream_zhiliaobo_two_branches() {
    let (hp_red, hj_red) = run(20274501, true);
    let (hp_black, hj_black) = run(20274502, false);
    eprintln!(
        "DBG 治疗波：红 → hp{hp_red:+}/护甲{hj_red:+}；非红 → hp{hp_black:+}/护甲{hj_black:+}"
    );
    assert_eq!(hp_red, 1, "治疗波（红判定）：应**回复 1 点体力**");
    assert_eq!(hj_red, 0, "治疗波（红判定）：**不获得护甲**");
    assert_eq!(hp_black, 0, "治疗波（非红判定）：**不回复体力**");
    assert_eq!(hj_black, 1, "治疗波（非红判定）：应 **+1 护甲**");
}

/// 上游点 1（**门槛反例**）：**满血**目标不可指定（`hp < maxHp`）。
#[test]
fn upstream_zhiliaobo_target_must_be_wounded() {
    let mut e = hearth_engine(20274503);
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
            .find(|(_, c)| c.name.contains("zhiliaobo"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("治疗波在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 1 号**满血**
    let max1 = e.state_core().player(1).max_hp as i32;
    e.debug_set_hp(1, max1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 治疗波（满血目标）→ {out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "治疗波：满血目标应被拒（上游 `target.hp < target.maxHp`）：{out:?}"
    );
    drain(&mut e);
}
