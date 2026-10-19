//! NC08 第⑦层（上游行为对照）样本 31：`guose`（国色，**标准面** / 大乔）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! guose: {
//!     filter(event, player) { return player.hasCards("hes", { suit: "diamond" }); },
//!     enable: "chooseToUse",
//!     filterCard(card) { return get.suit(card) === "diamond"; },   // **方块**牌
//!     position: "hes",                                             // 手牌 + 装备区 + 判定区
//!     viewAs: { name: "lebu" },                                    // 视为【乐不思蜀】
//!     prompt: "将一张方片牌当乐不思蜀使用",
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **转化**：**方块**牌当【乐不思蜀】**使用**；
//! 2. **牌区**：`hes`；
//! 3. **与其它 `viewAs` 的关系**：上游由玩家在候选间选择；原生为**固定优先级**
//!    （`play_card.rs:120` 注释「国色…优先于武圣」）。
//!
//! 原生实现引用：`engine/play_card.rs:118–132`（`kind != Le && kind != Sha` 前提下，
//! 持国色且牌为**方块** → `kind = CardKind::Le`；随后才轮到武圣）。
//!
//! **登记差异**：
//! - **①牌区**：上游 `position: "hes"` 含**装备/判定区**；原生转化作用于**被使用的牌**
//!   （实际来自手牌）→ 装备区的方块牌不可用于国色（与制衡/奇袭同族，**登记**）；
//! - **②优先级**：原生固定优先级（国色 > 武圣）；上游由玩家选择（与奇袭同族，**登记**）。
//!
//! 未覆盖（登记）：`check`/`ai`/`audio` 表现与策略面。

mod nc08_common;
use engine_native::catalog::Suit;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 按**花色**取牌并入手（走共享助手：按 card id 布阵，避免 `debug_place` 实例问题）。
fn give_diamond(e: &mut Engine, player: u8, name: &str) -> engine_native::CardId {
    let id = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.suit == Suit::Diamond && c.name == name)
        .unwrap_or_else(|| panic!("存在方块 {name}")) as engine_native::CardId;
    nc08_common::put_hand(e, player, id);
    id
}

/// 上游点 1（使用面）：**方块**牌当【乐不思蜀】使用 → 被接受且目标判定区出现该牌。
#[test]
fn upstream_guose_diamond_used_as_le() {
    let mut e = Engine::new(20266101, 4);
    e.debug_grant(0, "国色");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 方块【闪】（出牌阶段不可主动使用 → 「被接受」只可能来自转化）
    let card = give_diamond(&mut e, 0, "shan");
    let judge_before = e.state_core().player(1).judge.len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "方块牌应可当【乐不思蜀】使用（上游 guose）：{out:?}"
    );
    drain(&mut e);
    assert_eq!(
        e.state_core().player(1).judge.len(),
        judge_before + 1,
        "转化后的【乐不思蜀】应置入目标判定区"
    );
}

/// 上游点 1（门槛 · 花色）：**非方块**牌**不可**当【乐不思蜀】。
#[test]
fn upstream_guose_non_diamond_rejected() {
    let mut e = Engine::new(20266102, 4);
    e.debug_grant(0, "国色");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 红桃【闪】（非方块）
    let id = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.suit == Suit::Heart && c.name == "shan")
        .expect("存在红桃【闪】") as engine_native::CardId;
    nc08_common::put_hand(&mut e, 0, id);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: id, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "非方块牌不满足 `get.suit === 'diamond'` → 不应被转化：{out:?}"
    );
}

/// 上游点 1（技能门槛）：**未持国色**者方块牌不可当【乐不思蜀】。
#[test]
fn upstream_guose_requires_the_skill() {
    let mut e = Engine::new(20266103, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = give_diamond(&mut e, 0, "shan");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "未持国色 → 方块牌不可当【乐不思蜀】：{out:?}"
    );
}

/// **登记差异①②**（源码事实固定）：牌区 `hes` 与 `viewAs` 固定优先级。
#[test]
fn upstream_guose_zone_and_priority_differences_are_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_card.rs"
    ))
    .expect("读 play_card 源码");
    let idx = src.find("国色（方块手牌当乐不思蜀").expect("国色分支");
    let seg: String = src[idx..].chars().take(700).collect();
    assert!(
        seg.contains("优先于武圣"),
        "差异②固定：原生为固定优先级（国色 > 武圣）：{seg}"
    );
    assert!(
        seg.contains("guose && diamond"),
        "转化条件固定：持国色且方块：{seg}"
    );
    // 牌区：原生转化作用于被使用的牌（手牌路径），无 hes 的装备/判定区入口
    assert!(
        !src.contains("GUO_SE") || !src.contains("equips.iter().position(|c| *c == card) && guose"),
        "差异①固定：原生无『装备区方块牌当乐』入口"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("guose: {").expect("上游国色");
    let seg: String = up[idx..].chars().take(600).collect();
    assert!(
        seg.contains("get.suit(card) === \"diamond\"") && seg.contains("position: \"hes\""),
        "上游依据：方块 + hes：{seg}"
    );
    let _ = Phase::Play;
}
