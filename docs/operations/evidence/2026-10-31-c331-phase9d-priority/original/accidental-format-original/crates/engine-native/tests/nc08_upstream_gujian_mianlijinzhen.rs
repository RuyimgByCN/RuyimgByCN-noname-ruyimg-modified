//! NC08 第⑦层（上游行为对照）样本 92：**来源 U05（古剑）** 机关牌
//! `mianlijinzhen`（棉里针）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`mianlijinzhen`）：
//!
//! ```js
//! mianlijinzhen: {
//!     type: "jiguan", enable: true,
//!     filterTarget(card, player, target) { return target.hp >= player.hp; },   // **体力不低于使用者**
//!     content() {
//!         "step 0"; target.draw();      // **目标摸 1**
//!         "step 1"; target.loseHp();    // **目标失去 1 点体力**（体力流失，非伤害）
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**体力值 ≥ 使用者**的角色；
//! 2. **效果**：目标**摸 1**，随后**失去 1 点体力**（净：手牌 +1、hp −1）。
//!
//! 原生实现引用：`content/host/step/gujian.rs:71`（`GujianMianliCard`）。
//!
//! **本样本的定位**：验证「摸 1 + 失去 1 点体力」的**双步净效果**与**门槛方向**。
//!
//! 未覆盖（登记）：`loseHp` 与 `damage` 的语义差异（需护甲载体）；`ai.result` 策略面。

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

fn mianli_card(e: &mut Engine) -> engine_native::CardId {
    let id = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("mianlijinzhen"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("棉里针在目录");
    nc08_common::put_hand(e, 0, id);
    id
}

/// 上游点 2（**行为面**）：目标**摸 1** 且**失去 1 点体力**。
#[test]
fn upstream_mianlijinzhen_draws_then_loses_hp() {
    let mut e = gujian_engine(20272201);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = mianli_card(&mut e);
    let hp_before = e.debug_hp(1);
    let h_before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 棉里针：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "棉里针应可发动（目标 hp ≥ 使用者）：{out:?}"
    );
    drain(&mut e);
    let hp_after = e.debug_hp(1);
    let h_after = e.debug_hand(1).len();
    eprintln!("DBG 棉里针：1 号手牌 {h_before} → {h_after}；hp {hp_before} → {hp_after}");
    assert_eq!(
        h_after,
        h_before + 1,
        "棉里针：目标应**摸 1**（手牌 {h_before} → {h_after}）"
    );
    assert_eq!(
        hp_after,
        hp_before - 1,
        "棉里针：目标应**失去 1 点体力**（hp {hp_before} → {hp_after}）"
    );
}

/// 上游点 1（**门槛反例**）：目标体力**低于**使用者 → 不可指定。
#[test]
fn upstream_mianlijinzhen_target_hp_gate() {
    let mut e = gujian_engine(20272202);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 让 1 号体力**低于** 0 号（0 号满血）
    let max1 = e.state_core().player(1).max_hp as i32;
    e.debug_set_hp(1, (max1 - 2).max(1));
    let card = mianli_card(&mut e);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 棉里针（目标体力更低）→ {out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "棉里针：目标体力低于使用者时应被拒（上游 `target.hp >= player.hp`）：{out:?}"
    );
    drain(&mut e);
}
