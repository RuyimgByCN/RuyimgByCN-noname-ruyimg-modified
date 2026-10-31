//! NC08 第⑦层（上游行为对照）样本 136：**来源 U09（mtg）** 地牌
//! `mtg_shuimomuxue`（水没墓穴）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`）：
//!
//! ```js
//! mtg_shuimomuxue: {
//!     type: "land", enable(card, player) { return !player.hasSkill("land_used"); },
//!     notarget: true,
//!     content() {
//!         "step 0"; game.changeLand("mtg_shuimomuxue", player);
//!                   if (player.countDiscardableCards("he")) {
//!                       player.chooseToDiscard("是否弃置一张牌并摸两张牌？", "he")…;   // **可取消**
//!                   } else { event.fin… }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **使用限制**：**每回合限一次**；**无目标**；
//! 2. **选择支**：有可弃牌时**可弃 1 张 → 摸 2**（**可取消**）；无可弃牌则无事。
//!
//! 原生实现引用：`kernel/intrinsic.rs`（`CardKind::Shuimomuxue` → mtg 地牌机）。
//!
//! **本样本的定位**：验证**两支**——①**弃 1 → 摸 2**（净：打出 −1、弃 1、摸 2 ⇒ 手牌不变）；
//! ②**取消 → 不摸**（净 −1）。
//!
//! 未覆盖（登记）：`land_used` 每回合重置；无可弃牌分支。

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

/// 跑一次：`discard` 决定是否弃牌。返回 `(手牌变化, 牌堆变化)`。
fn run(seed: u64, discard: bool) -> (i64, i64) {
    let mut e = mtg_engine(seed);
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
            .find(|(_, c)| c.name.contains("shuimomuxue"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("水没墓穴在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
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
    let deck_before = e.state_core().deck.len() as i64;
    let h_before = e.debug_hand(0).len() as i64;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "水没墓穴应可发动（无目标）：{out:?}"
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
        if !done && discard && !cards.is_empty() {
            done = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![cards[0]],
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
    let d_h = e.debug_hand(0).len() as i64 - h_before;
    let d_deck = deck_before - e.state_core().deck.len() as i64;
    eprintln!("DBG 水没墓穴：discard={discard} → 手牌{d_h:+}、牌堆−{d_deck}（询问{prompts:?}）");
    (d_h, d_deck)
}

/// 上游点 2（**行为面 · 两支**）：弃 1 → 摸 2；取消 → 不摸。
#[test]
fn upstream_mtg_shuimomuxue_two_branches() {
    let (h_d, deck_d) = run(20286901, true);
    let (h_c, deck_c) = run(20286902, false);
    eprintln!("DBG 水没墓穴：弃牌支 → 手牌{h_d:+}/摸{deck_d}；取消支 → 手牌{h_c:+}/摸{deck_c}");
    assert_eq!(
        deck_d, 2,
        "水没墓穴（**弃 1**）：应**摸 2**（实测摸 {deck_d}）"
    );
    assert_eq!(
        h_d, 0,
        "水没墓穴（弃 1）：打出 −1、弃 1、摸 2 ⇒ 手牌**不变**（实测 {h_d:+}）"
    );
    assert_eq!(
        deck_c, 0,
        "水没墓穴（**取消**）：**不摸牌**（实测摸 {deck_c}）"
    );
    assert_eq!(
        h_c, -1,
        "水没墓穴（取消）：仅打出本体 ⇒ 手牌 **−1**（实测 {h_c:+}）"
    );
}
