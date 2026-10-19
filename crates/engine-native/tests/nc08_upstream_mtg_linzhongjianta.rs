//! NC08 第⑦层（上游行为对照）样本 138：**来源 U09（mtg）** 地牌
//! `mtg_linzhongjianta`（林中尖塔）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`）：
//!
//! ```js
//! mtg_linzhongjianta: {
//!     type: "land", enable(card, player) { return !player.hasSkill("land_used"); },
//!     notarget: true,
//!     content() {
//!         "step 0"; game.changeLand("mtg_linzhongjianta", player);
//!                   player.discoverCard(card => get.subtype(card) === "equip1",
//!                                       "nogain", "选择一个武器装备之");   // **从武器中选一张（不获得）**
//!         "step 1"; if (result.choice) { player.equip(game.createCa…); }       // **装备之（生成）**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **使用限制**：**每回合限一次**；**无目标**；
//! 2. **发现**：从 **`equip1`（武器）** 中**选择一张**；`"nogain"` ⇒ **不入手**；
//! 3. **结果**：**直接装备之**（上游用 `game.createCard` **生成**）。
//!
//! 原生实现引用：`kernel/intrinsic.rs`（`CardKind::Linzhongjianta` → mtg 地牌机）。
//!
//! **本样本的定位**：验证「**选择武器 → 直接装备（不入入手牌）**」；并核对**生成 vs 实体**（牌堆计数）。
//!
//! 未覆盖（登记）：`land_used` 每回合重置；候选集合是否**限定 `equip1`**。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn mtg_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-mtg-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 mtg cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2+3（**行为面**）：选择武器 → **直接装备**（不入入手牌）。
#[test]
fn upstream_mtg_linzhongjianta_discover_and_equip() {
    let mut e = mtg_engine(20287101);
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
            .find(|(_, c)| c.name.contains("linzhongjianta"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("林中尖塔在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let deck_before = e.state_core().deck.len() as i64;
    let eq_before = e.debug_equips(0).len();
    let h_before = e.debug_hand(0).len() as i64;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 林中尖塔：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "林中尖塔应可发动（无目标）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut picked = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        if !picked && !cards.is_empty() {
            picked = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: vec![cards[0]] },
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
    let eq_after = e.debug_equips(0).len();
    let h_after = e.debug_hand(0).len() as i64;
    let deck_after = e.state_core().deck.len() as i64;
    eprintln!(
        "DBG 林中尖塔：询问={prompts:?} 已选={picked}；装备区 {eq_before} → {eq_after}；\
         手牌 {h_before} → {h_after}；牌堆 −{}",
        deck_before - deck_after
    );
    assert!(picked, "林中尖塔：应出现**选择武器**的询问");
    assert!(
        eq_after > eq_before,
        "林中尖塔：选择的武器应**直接装备**（装备区 {eq_before} → {eq_after}）"
    );
    assert_eq!(
        h_after,
        h_before - 1,
        "林中尖塔：`\"nogain\"` ⇒ **不入手牌**（仅打出本体 −1；{h_before} → {h_after}）"
    );
    eprintln!(
        "NOTE 林中尖塔（生成 vs 实体）：上游为 `game.createCard`（**生成**）；实测牌堆 −{} ⇒ \
         若为 0 则一致（生成），为 1 则属「生成 vs 实体取牌」族（须登记）",
        deck_before - deck_after
    );
}
