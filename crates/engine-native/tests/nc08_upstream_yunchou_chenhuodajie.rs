//! NC08 第⑦层（上游行为对照）样本 145：**来源 U05（运筹）** 锦囊
//! `chenhuodajie`（趁火打劫）——**`global` 响应窗口卡**。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js`）：
//!
//! ```js
//! chenhuodajie: {
//!     type: "trick", filterTarget: true,
//!     global: "g_chenhuodajie",                                  // **也可在窗口打出**
//!     content() {
//!         if (target.countCards("he")) player.gainPlayerCard("he", target, true);   // **使用者选择**并获得
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：任意角色（`filterTarget: true`）；
//! 2. **效果**：**使用者获得目标一张 `he` 牌**（上游为**使用者选择**，`gainPlayerCard(..., true)`）；
//! 3. **`global`**：额外提供**响应窗口**用法。
//!
//! 原生实现引用：`catalog.rs:92`（注释：「趁火打劫（G1 运筹帷幄，锦囊）：获得目标一张手牌
//! （**登记差异**：…）」）。
//!
//! **本样本的定位**：验证「**获得目标一张 `he` 牌**」；并核对**选择权**是否与上游一致
//! （若为自动取首张 ⇒ 属既有「**确定性首张**」族）。
//!
//! 未覆盖（登记）：**`global` 响应窗口**的用法面；`ai` 面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn yunchou_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-yunchou-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读运筹 cards");
    let mut pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            pack.generals = b.generals;
        }
    }
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：**获得目标一张 `he` 牌**。
#[test]
fn upstream_chenhuodajie_gains_target_card() {
    let mut e = yunchou_engine(20287901);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号：1 张手牌 + 1 件装备（供 `he` 选择）
    let hand1 = {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(1, name)
    };
    let equip1 = {
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
            .find(|(_, c)| c.name.contains("chenhuodajie"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("趁火打劫在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let he_before = e.debug_hand(1).len() + e.debug_equips(1).len();
    let h0_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 趁火打劫：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "趁火打劫应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut picked = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        // 若出现**选择目标牌**的询问 ⇒ 选中该**装备**（便于身份判据）
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        if !picked && cards.contains(&ChoiceOption::Card(equip1)) {
            picked = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(equip1)],
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if !cards.is_empty() {
            vec![cards[0]]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
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
    let he_after = e.debug_hand(1).len() + e.debug_equips(1).len();
    let h0_after = e.debug_hand(0).len();
    let got_equip = e.debug_hand(0).contains(&equip1) || e.debug_equips(0).contains(&equip1);
    eprintln!(
        "DBG 趁火打劫：询问={prompts:?} 选中装备={picked}；1 号 he {he_before} → {he_after}；\
         0 号手牌 {h0_before} → {h0_after}；获得该装备={got_equip}（原手牌 {hand1} 仍在={}）",
        e.debug_hand(1).contains(&hand1)
    );
    assert_eq!(
        he_after,
        he_before - 1,
        "趁火打劫：目标应**失去一张 `he` 牌**（{he_before} → {he_after}）"
    );
    assert_eq!(
        h0_after,
        h0_before,
        "趁火打劫：打出 −1、获得 1 ⇒ 使用者手牌数不变（{h0_before} → {h0_after}）"
    );
    eprintln!(
        "NOTE 趁火打劫：询问={prompts:?}（若**无询问**则选择权为自动 ⇒ 属既有「确定性首张」族）"
    );
}
