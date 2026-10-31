//! NC08 第⑦层（上游行为对照）样本 89：**来源 U05（古剑）** 食物牌
//! `liyutang`（鲤鱼汤）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`liyutang`）：
//!
//! ```js
//! liyutang: {
//!     type: "food", enable: true,
//!     filterTarget(card, player, target) { return !target.hasSkill("liyutang"); },   // 目标**未持**该技
//!     content() {
//!         target.$gain2(cards);                        // 目标**获得此牌**
//!         target.storage.liyutang = card;              // 记录牌本体
//!         target.storage.liyutang_markcount = 2;       // **2 个计数**
//!         target.addSkill("liyutang");                 // 授予计时子技
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**未持** `liyutang` 的角色（防重复叠加）；
//! 2. **效果**：目标**获得此牌** + 挂 **2 个计数** + 获得计时子技（后续按计数结算）。
//!
//! 原生实现引用：宿主行为（`HostBehavior::Liyutang*`，与同批 `chunbing`（春饼，已有样本）
//! **同为食物计数机**）—— 本样本以**行为面**观测「获得牌」与门槛。
//!
//! **登记差异**：视实测而定（门槛/获得牌是否与上游一致）。
//!
//! 未覆盖（登记）：**计数的逐轮结算**（子技效果，需跨回合驱动）；`ai.result` 策略面。

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

fn liyutang_card(e: &mut Engine) -> engine_native::CardId {
    let id = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("liyutang"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("鲤鱼汤在目录");
    nc08_common::put_hand(e, 0, id);
    id
}

/// 上游点 2（**行为面**）：目标**获得此牌**（手牌 +1）。
///
/// **本轮未验收（登记，含诊断）**：实测发动 `Accepted`，但**目标手牌 0 → 0**、
/// **使用者手牌为空**、**该牌进入弃牌堆**（`弃堆含此牌=true`）✗ ⇒ 原生**只完成用牌与弃置**，
/// **未执行**上游的 `target.$gain2(cards)` + `storage` 计数 + `addSkill` ⇒ **效果为空**
/// （详见修复清单第 37 项）。用例以 `#[ignore]` 作为修复后的验收条件。
#[test]
#[ignore = "鲤鱼汤：目标应获得此牌（上游 $gain2 + storage + addSkill）——原生效果为空，见第 37 项"]
fn upstream_liyutang_target_gains_the_card() {
    let mut e = gujian_engine(20271901);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = liyutang_card(&mut e);
    let h1_before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 鲤鱼汤：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "鲤鱼汤应可发动：{out:?}"
    );
    drain(&mut e);
    let hand = e.debug_hand(1);
    eprintln!(
        "DBG 鲤鱼汤：1 号手牌 {h1_before} → {}（含此牌={}）；0 号手牌={:?}；弃堆含此牌={}",
        hand.len(),
        hand.contains(&card),
        e.debug_hand(0),
        e.state_core().discard.contains(&card)
    );
    assert!(
        hand.contains(&card),
        "鲤鱼汤：目标应**获得此牌**（手牌 {h1_before} → {}）",
        hand.len()
    );
}

/// 上游点 1（**门槛**）：目标**已持**该技 → 不可再指定（原生是否校验 → 实测）。
#[test]
fn upstream_liyutang_gate_on_already_marked() {
    let mut e = gujian_engine(20271902);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 包内只有一张鲤鱼汤 → 用 `debug_grant` 预置子技，等价于「目标已持该技」
    // 子技在目录中的名字为 `gj_food_liyutang`（包面向键 `host:gujian_food_liyutang`）
    e.debug_grant(1, "gj_food_liyutang");
    assert!(
        e.debug_has_skill(1, "gj_food_liyutang"),
        "夹具前提：1 号应已持有 liyutang 子技"
    );
    let card = liyutang_card(&mut e);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 鲤鱼汤门槛（目标已持子技）→ {out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "鲤鱼汤：已持该技的目标不应再被指定（上游 filterTarget `!hasSkill`）：{out:?}"
    );
    drain(&mut e);
}
