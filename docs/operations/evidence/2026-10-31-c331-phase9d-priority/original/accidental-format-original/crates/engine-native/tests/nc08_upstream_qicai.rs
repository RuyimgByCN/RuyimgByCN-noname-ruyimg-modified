//! NC08 第⑦层（上游行为对照）样本 33：`qicai`（奇才，**标准面** / 黄月英）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! qicai: {
//!     mod: {
//!         targetInRange(card, player, target, now) {
//!             if (["trick", "delay"].includes(get.type(card))) {   // **任意锦囊/延时锦囊**
//!                 return true;                                      // 无距离限制
//!             }
//!         },
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **范围豁免**：使用**锦囊/延时锦囊**时**无视距离**（`targetInRange` 恒真）；
//! 2. 对**非锦囊**（基本牌/装备）**无**该豁免。
//!
//! 原生实现引用：`engine/play_card.rs:698`（`qicai` 判定）与 `:737`（`!qicai && … &&
//! computed_distance > 1 → IllegalTargets`）—— **位于 `CardKind::Snatch`（顺手牵羊）分支内**。
//!
//! **登记差异（本文件固定）**：
//! - **①覆盖面**：上游 `mod.targetInRange` 覆盖**所有**锦囊/延时锦囊；原生 `qicai` 仅在
//!   **顺手牵羊**分支被读取（`play_card.rs` 中 `qicai` 仅 2 处引用，均在 Snatch 分支）。
//!   **附带新发现**：另一张有距离限制的锦囊【兵粮寸断】（原生 `CardKind::Bing`）的目标校验
//!   **完全没有距离检查**（`play_card.rs:764–769` 只查目标数/非自己/存活）→ **对所有人**都比上游
//!   宽松（上游要求距离 1）→ 该差异比「奇才覆盖面」更基础，**登记为规则更宽松**；
//! - **②实现方式**：上游为 `mod`（范围查询层），原生为**目标校验处的分支豁免**（等价但位置不同）。
//!
//! 未覆盖（登记）：`now` 参数（上游 mod 签名含时机参数，原生无对应概念）。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点 1（**顺手牵羊**·范围内）：距离 1 的目标可用（基线行为）。
#[test]
fn upstream_qicai_snatch_within_range_ok() {
    let mut e = Engine::new(20266301, 4);
    e.debug_grant(0, "奇才");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 目标（1 号，距离 1）持一张牌
    let v = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(1, name)
    };
    e.debug_give_hand_card(1, 1, v);
    let snatch = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "snatch")
        .expect("顺手牵羊在目录") as engine_native::CardId;
    nc08_common::put_hand(&mut e, 0, snatch);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: snatch,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "距离 1 的顺手牵羊应可用：{out:?}"
    );
}

/// 上游点 1（**奇才豁免**）：持奇才时，距离 >1 的顺手牵羊**仍可用**。
#[test]
fn upstream_qicai_ignores_distance_for_snatch() {
    let mut e = Engine::new(20266302, 4);
    e.debug_grant(0, "奇才");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 目标选 2 号（距离 2）
    let v = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(2, name)
    };
    e.debug_give_hand_card(2, 2, v);
    let snatch = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "snatch")
        .expect("顺手牵羊在目录") as engine_native::CardId;
    nc08_common::put_hand(&mut e, 0, snatch);
    let dist = e.state_core().computed_distance(e.catalog(), 0, 2);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: snatch,
            targets: vec![2],
        },
    });
    eprintln!("DBG 距离={dist} 持奇才对距离>1 目标出顺手牵羊：{out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "奇才：锦囊无距离限制（距离 {dist}）→ 应可用：{out:?}"
    );
    drain(&mut e);
}

/// 上游点 1（反例 · 技能门槛）：**未持奇才**时距离 >1 的顺手牵羊**被拒**。
#[test]
fn upstream_qicai_requires_the_skill() {
    let mut e = Engine::new(20266303, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let v = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(2, name)
    };
    e.debug_give_hand_card(2, 2, v);
    let snatch = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "snatch")
        .expect("顺手牵羊在目录") as engine_native::CardId;
    nc08_common::put_hand(&mut e, 0, snatch);
    let dist = e.state_core().computed_distance(e.catalog(), 0, 2);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: snatch,
            targets: vec![2],
        },
    });
    eprintln!("DBG 距离={dist} 未持奇才对距离>1 目标出顺手牵羊：{out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "未持奇才 → 距离 {dist} 的顺手牵羊应被拒：{out:?}"
    );
    let _ = Phase::Play;
}

/// **登记差异①②**（源码事实固定）：原生仅在顺手牵羊分支读取奇才；且目录无【兵粮寸断】。
#[test]
fn upstream_qicai_coverage_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_card.rs"
    ))
    .expect("读 play_card 源码");
    let hits = src.matches("qicai").count();
    assert!(
        hits == 2,
        "差异①固定：play_card.rs 中 `qicai` 仅 {hits} 处（定义 + Snatch 分支），未覆盖全部锦囊"
    );
    let idx = src.find("qicai").expect("qicai");
    let seg: String = src[idx..].chars().take(60).collect();
    assert!(seg.contains("qicai"), "{seg}");
    // 【兵粮寸断】在原生目录中为 `CardKind::Bing`（`catalog.rs:939`）→ 牌种**已实现**；
    // 但其目标校验分支（`play_card.rs:764–769`）**只查目标数/非自己/存活**，**无距离检查**
    // → 即**对所有人**都没有「距离 1」限制（比上游更宽松）→ 这是比「奇才覆盖面」更基础的差异。
    let bing_seg = {
        let idx = src.find("CardKind::Bing =>").expect("兵粮分支");
        src[idx..].chars().take(700).collect::<String>()
    };
    assert!(
        !bing_seg.contains("computed_distance") && !bing_seg.contains("qicai"),
        "新发现：兵粮寸断分支无距离检查（上游要求距离 1）→ 登记：{bing_seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("qicai: {").expect("上游奇才");
    let seg: String = up[idx..].chars().take(300).collect();
    assert!(
        seg.contains("[\"trick\", \"delay\"]"),
        "上游依据：任意锦囊/延时锦囊无视距离：{seg}"
    );
}
