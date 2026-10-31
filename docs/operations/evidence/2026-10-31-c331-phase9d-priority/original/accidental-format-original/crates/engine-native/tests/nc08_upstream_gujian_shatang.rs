//! NC08 第⑦层（上游行为对照）样本 **100**：**来源 U05（古剑）** 锦囊
//! `shatang`（沙棠）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`shatang`）：
//!
//! ```js
//! shatang: {
//!     type: "trick", enable: true, filterTarget: true,     // 任意角色
//!     cardcolor: "red", cardnature: "fire",
//!     content() {
//!         "step 0"; target.damage("fire");      // **1 点火焰伤害**
//!         "step 1"; target.changeHujia();       // **+1 护甲**（伤害之后）
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：任意角色（`filterTarget: true`）；
//! 2. **效果（两步）**：目标先受 **1 点火焰伤害**，随后 **+1 护甲** —— 两步**顺序固定**
//!    （本样本同时验证「两步都发生」这一顺序不变量）。
//!
//! 原生实现引用：`content/host/step/jiguan.rs`（`HostBehavior::JiguanShatangCard`；
//! `kernel/intrinsic.rs:409` 的 `CardKind::Shatang | Heilonglinpian | Liutouge` 分支）。
//!
//! **本样本的定位**：验证**双效果**（伤害 + 护甲）是否都执行——这是「顺序不变量」类检查。
//!
//! 未覆盖（登记）：`cardnature: fire` 的**属性面**（火伤修饰/藤甲等）；`ai.result` 策略面。

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

/// 上游点 2（**行为面 · 双效果**）：目标**受 1 点火伤**且**获得 1 点护甲**。
#[test]
fn upstream_shatang_damages_then_grants_hujia() {
    let mut e = gujian_engine(20273001);
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
            .find(|(_, c)| c.name.contains("shatang"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("沙棠在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp_before = e.debug_hp(1);
    let hujia_before = e.state_core().player(1).hujia;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 沙棠：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "沙棠应可发动：{out:?}"
    );
    drain(&mut e);
    let p1 = e.state_core().player(1);
    eprintln!(
        "DBG 沙棠：1 号 hp {hp_before} → {}；护甲 {hujia_before} → {}",
        p1.hp, p1.hujia
    );
    assert_eq!(
        p1.hp,
        hp_before - 1,
        "沙棠：目标应先受 **1 点火焰伤害**（hp {hp_before} → {}）",
        p1.hp
    );
    assert_eq!(
        p1.hujia,
        hujia_before + 1,
        "沙棠：随后应 **+1 护甲**（护甲 {hujia_before} → {}）——两步顺序不变量",
        p1.hujia
    );
}
