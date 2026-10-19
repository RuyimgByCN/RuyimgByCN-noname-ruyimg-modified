//! NC08 第⑦层（上游行为对照）样本 126：**来源 U05（运筹）** 延时锦囊
//! `huoshan`（火山）——**三维核对中唯一「无实现且无登记」的卡**（第 47 项）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js`）：
//!
//! ```js
//! huoshan: {
//!     fullskin: true, type: "delay", cardcolor: "red", cardnature: "fire", toself: true,
//!     modTarget(card, player, target) { return lib.filter.judge(card, player, target); },
//!     enable(card, player) { return player.canAddJudge(card); },
//!     filterTarget(card, player, target) {
//!         return lib.filter.judge(card, player, target) && player === target; },
//!     selectTarget: [-1, -1],
//!     judge(card) {                                   // **判定**：红桃 2–9 → 通过（−6）
//!         if (get.suit(card) === "heart" && get.number(card) > 1 && get.number(card) < 10) return -6;
//!         return 1; },
//!     judge2(result) { if (result.bool === false) return true; return false; },   // **反转**
//!     effect() {
//!         if (result.bool === false) {
//!             player.damage(2, "fire", "nosource");   // **自己受 2 点火伤（无来源）**
//!             var players = game.filterPlayer(current =>
//!                 get.distance(player, current) <= 1 && player !== current);
//!             players.sort(lib.sort.seat);            // **距离 ≤1 的其他角色**（座位序）
//!             …
//!         }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **类型**：**延时锦囊**（`type: "delay"`，进**判定区**；`toself` 且 `selectTarget: [-1,-1]`）；
//! 2. **判定**：**红桃 2–9** ⇒ 通过；否则**生效**（`judge2` 反转）；
//! 3. **效果**：**自己受 2 点火伤（无来源）** + **距离 ≤1 的其他角色**（按座位序）后续结算。
//!
//! **本轮未验收（登记，第 47 项）**：`huoshan` 在 `crates/engine-native/src/` **全库 0 命中**
//! 且 **`docs/**` 0 命中** ⇒ **包内交付、无实现、无登记**。本用例**记录受理基线**
//! （发动结果 + 判定区状态），作为修复后的对照。
//!
//! 未覆盖（登记）：`effect` 尾段（相邻角色的后续结算）逐字确认；`judge2` 反转的语义面。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **运筹包**建局（本卡在 yunchou 包）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = root.join("shahai-yunchou-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读运筹 cards");
    let mut pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    // 运筹为**双产物**包：武将/技能在 character.json
    if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            pack.generals = b.generals;
        }
    }
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// **实测受理基线**（登记用）：记录发动结果与判定区状态。
#[test]
fn upstream_huoshan_record_acceptance() {
    let mut e = yunchou_engine(20275601);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = match e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("huoshan"))
        .map(|(i, _)| i as engine_native::CardId)
    {
        Some(id) => {
            nc08_common::put_hand(&mut e, 0, id);
            id
        }
        None => {
            eprintln!("NOTE 火山不在运筹包目录——本用例不构成验收");
            return;
        }
    };
    let judge_before = e.state_core().player(0).judge.len();
    // 目标形态两种都试（`selectTarget: [-1,-1]` + `toself`：可能按「无显式目标」或「显式自己」受理）
    let mut out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 火山：targets=[] → {out:?}");
    if matches!(out, CommandOutcome::Rejected(_)) {
        out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card, targets: vec![0] },
        });
        eprintln!("DBG 火山：targets=[0] → {out:?}");
    }
    // **第三种形态**：其他角色（受理面普查显示此形态可能被受理——须核实，避免以偏概全）
    if matches!(out, CommandOutcome::Rejected(_)) {
        out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card, targets: vec![1] },
        });
        eprintln!("DBG 火山：targets=[1] → {out:?}");
    }
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
    drain(&mut e);
    let judge_after = e.state_core().player(0).judge.len();
    eprintln!(
        "DBG 火山（登记基线）：发动={out:?}；询问={prompts:?}；0 号判定区 {judge_before} → {judge_after}"
    );
    eprintln!(
        "NOTE 火山：源码核对——crates/engine-native/src 全库 0 命中、docs/** 0 命中 ⇒ 包内交付但无实现\
         且无登记（第 47 项）；本用例只记录受理基线，不写成通过"
    );
}
