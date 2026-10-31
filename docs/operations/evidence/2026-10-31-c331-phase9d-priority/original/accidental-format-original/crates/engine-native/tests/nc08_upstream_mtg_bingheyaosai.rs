//! NC08 第⑦层（上游行为对照）样本 132：**来源 U09（mtg）** 地牌
//! `mtg_bingheyaosai`（冰河要塞）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`）：
//!
//! ```js
//! mtg_bingheyaosai: {
//!     type: "land", fullborder: "wood",
//!     enable(card, player) { return !player.hasSkill("land_used"); },   // 每回合限一次
//!     notarget: true,                                                   // 无目标
//!     content() {
//!         "step 0"; game.changeLand("mtg_bingheyaosai", player);
//!                   player.draw(2);                                      // **摸 2**
//!         "step 1"; player.chooseToDiscard("he", 2, true);               // **弃 2（强制）**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **使用限制**：**每回合限一次**（`land_used`）；**无目标**（`notarget`）；
//! 2. **效果**：**摸 2** → **弃 2**（**强制**）⇒ 打出本体 −1、摸 2、弃 2 ⇒ 手牌**净 −1**。
//!
//! 原生实现引用：`kernel/intrinsic.rs`（`CardKind::Bingheyaosai` → mtg 地牌机）。
//!
//! **本样本的定位**：验证「**摸 2 + 强制弃 2**」的净效果（以**牌堆 −2** 与**手牌净 −1** 为判据）。
//!
//! 未覆盖（登记）：`land_used` 的每回合重置；`game.changeLand` 的完整语义。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn mtg_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-mtg-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 mtg cards");
    let pack = content_native_loader::bridge_pack(&cards)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：**摸 2 + 弃 2（强制）** ⇒ 牌堆 −2、手牌净 −1。
#[test]
fn upstream_mtg_bingheyaosai_draw2_discard2() {
    let mut e = mtg_engine(20286501);
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
            .find(|(_, c)| c.name.contains("bingheyaosai"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("冰河要塞在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 另给 2 张手牌（供「弃 2」）
    for _ in 0..2 {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name);
    }
    let deck_before = e.state_core().deck.len();
    let h_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![],
        },
    });
    eprintln!("DBG 冰河要塞：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "冰河要塞应可发动（无目标）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut done = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        if !done && !cards.is_empty() {
            done = true;
            // **弃 2（强制）**：取所需张数
            let need = c.spec.min.max(1);
            let sel: Vec<ChoiceOption> = cards.into_iter().take(need).collect();
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
    drain(&mut e);
    let deck_after = e.state_core().deck.len();
    let h_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 冰河要塞：询问={prompts:?}；牌堆 {deck_before} → {deck_after}（应 −2）；\
         手牌 {h_before} → {h_after}（打出 −1、摸 2、弃 2 ⇒ 净 −1）"
    );
    assert_eq!(
        deck_before as i64 - deck_after as i64,
        2,
        "冰河要塞：应**摸 2**（牌堆 {deck_before} → {deck_after}）"
    );
    assert_eq!(
        h_after as i64,
        h_before as i64 - 1,
        "冰河要塞：打出 −1、摸 2、弃 2 ⇒ 手牌**净 −1**（{h_before} → {h_after}）"
    );
}
