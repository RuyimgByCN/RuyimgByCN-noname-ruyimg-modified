//! NC08 第⑦层（上游行为对照）样本 86：**来源 U05（古剑）** 机关牌
//! `longxugou`（龙须钩）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`longxugou`）：
//!
//! ```js
//! longxugou: {
//!     type: "jiguan", enable: true, wuxieable: true,
//!     filterTarget(card, player, target) {
//!         return target !== player && target.countGainableCards(player, "e");   // 其他角色 + 可获得装备
//!     },
//!     content() {
//!         "step 0";
//!         var es = target.getGainableCards(player, "e");
//!         if (es.length) player.choosePlayerCard("e", target, true).set("es", es)…;   // **使用者选一张装备**
//!         "step 1";
//!         if (result.bool) { target.$give(result.links[0], player); target.lose(result.links[0], ui.special);
//!                            event.card = result.links[0]; game.delay(); }
//!         "step 2";
//!         if (event.card && get.position(event.card) === "s") player.equip(event.card);   // **自己装备之**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**其他角色**且其**装备区有可获得牌**；
//! 2. **流程**：**使用者选**目标装备区的一张 → **获得之** → **并自己装备上**（`player.equip`）。
//!
//! 原生实现引用：`engine/play_card.rs:1695`（目标校验：恰 1 名其他存活**且有装备**的目标；
//! 注释「龙须钩：恰 1 名其他存活且有装备的目标」）；效果面见宿主行为
//! `HostBehavior::Longxugou*`（本样本以**行为面**观测）。
//!
//! **登记差异（视实测而定）**：上游 `choosePlayerCard`（**使用者自选**）↔ 原生若为**确定性取牌**
//! 则属「**取牌选择权**」族。
//!
//! 未覆盖（登记）：`wuxieable` 无懈面；`game.delay()` 表现面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
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

/// 上游点 2（**行为面**）：获得目标装备区一张并**自己装备**（equips 迁移）。
///
/// **本轮未验收（差异登记）**：实测——询问 `gj_longxu_pick` 出现（**使用者自选** ✅ 与上游一致）、
/// 1 号装备区 **1 → 0**（牌被拿走 ✅），但 **0 号装备区 0 → 0**、该牌进入 **0 号手牌**
/// （`手牌含该装备=true`）✗ ⇒ 原生只「**获得**」未「**装备**」——上游 step 2 为
/// `player.equip(event.card)` ⇒ **规则损失**（详见修复清单第 36 项）。
/// 用例以 `#[ignore]` 作为修复后的验收条件（0 号装备区应 +1）。
#[test]
#[ignore = "龙须钩：获得后应自己装备（上游 player.equip）——原生仅入手，见第 36 项"]
fn upstream_longxugou_steals_and_equips() {
    let mut e = gujian_engine(20271601);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号装备一件（可获得）
    let equip = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("armor") || c.name.contains("weapon"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("装备牌");
        nc08_common::put_equips(&mut e, 1, id);
        id
    };
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("longxugou"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("龙须钩在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let e0_before = e.debug_equips(0).len();
    let e1_before = e.debug_equips(1).len();
    let h0_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 龙须钩：发动={out:?}（1 号装备={equip}）");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "龙须钩应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 若有「选装备」询问 → 选那张
        if c.spec.options.contains(&ChoiceOption::Card(equip)) {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(equip)],
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
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
    let e0_after = e.debug_equips(0).len();
    let e1_after = e.debug_equips(1).len();
    let h0_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 龙须钩：询问={prompts:?} 0 号装备 {e0_before} → {e0_after}（手牌 {h0_before} → {h0_after}，\
         手牌含该装备={}）；1 号装备 {e1_before} → {e1_after}",
        e.debug_hand(0).contains(&equip)
    );
    assert_eq!(e1_after, e1_before - 1, "龙须钩：目标装备区应减少一件");
    assert!(
        e0_after > e0_before,
        "龙须钩：使用者应**获得并装备**之（装备 {e0_before} → {e0_after}）"
    );
}

/// 上游点 1（**门槛反例**）：目标**装备区无牌** → 不可指定。
#[test]
fn upstream_longxugou_target_needs_equips() {
    let mut e = gujian_engine(20271602);
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
            .find(|(_, c)| c.name.contains("longxugou"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("龙须钩在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 龙须钩（目标无装备）→ {out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "龙须钩：目标装备区无牌时应被拒（上游 `countGainableCards(player, \"e\")`）：{out:?}"
    );
    drain(&mut e);
}
