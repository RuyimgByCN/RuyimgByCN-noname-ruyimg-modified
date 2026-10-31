//! NC08 第⑦层（上游行为对照）样本 133：**来源 U09（mtg）** 地牌
//! `mtg_lindixiliu`（林地溪流）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`）：
//!
//! ```js
//! mtg_lindixiliu: {
//!     type: "land", fullborder: "wood",
//!     enable(card, player) { return !player.hasSkill("land_used"); },
//!     notarget: true,
//!     content() {
//!         "step 0"; game.changeLand("mtg_lindixiliu", player);
//!                   var card = get.cardPile2(card => get.suit(card) === "heart");   // **牌堆找红桃**
//!                   if (card) player.gain(card, "draw");                            // **获得之**
//!         "step 1"; player.chooseToDiscard("he", [1, Infinity], { suit: "heart" }, "林地溪流")…;
//!                   // **弃任意张红桃**（可取消；每弃一张将一张延时锦囊置入随机敌方判定区）
//!         "step 2"; if (result.bo…)
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **使用限制**：**每回合限一次**；**无目标**；
//! 2. **第一步**：**从牌堆取一张【红桃】牌并获得**；
//! 3. **第二步**：**可弃任意张红桃牌**（可取消）→ 每弃一张将一张**延时锦囊**置入**随机敌方**判定区。
//!
//! 原生实现引用：`kernel/intrinsic.rs`（`CardKind::Lindixiliu` → mtg 地牌机）。
//!
//! **本样本的定位**：验证**第一步**（获得红桃牌：**牌堆 −1**、**手中多一张红桃**）与
//! **第二步可取消**（取消后无后续结算）。
//!
//! 未覆盖（登记）：第二步**每弃一张置延时锦囊**的完整链路；**随机敌方**（登记差异面）。

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

/// 上游点 2（**行为面**）：**从牌堆获得一张红桃牌**；第二步**可取消**。
#[test]
fn upstream_mtg_lindixiliu_gains_heart() {
    let mut e = mtg_engine(20286601);
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
            .find(|(_, c)| c.name.contains("lindixiliu"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("林地溪流在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
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
    eprintln!("DBG 林地溪流：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "林地溪流应可发动（无目标）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // **第二步可取消**：一律取「Pass / 空选」，不弃红桃
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
    let hand = e.debug_hand(0);
    let hearts = hand
        .iter()
        .filter(|c| {
            matches!(
                e.catalog().cards[**c as usize].suit,
                engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
            ) && matches!(
                e.catalog().cards[**c as usize].suit,
                engine_native::catalog::Suit::Heart
            )
        })
        .count();
    eprintln!(
        "DBG 林地溪流：询问={prompts:?}；牌堆 {deck_before} → {deck_after}；\
         手牌 {h_before} → {}（其中红桃 {hearts} 张）",
        hand.len()
    );
    assert_eq!(
        deck_before as i64 - deck_after as i64,
        1,
        "林地溪流：应**从牌堆取一张**（牌堆 {deck_before} → {deck_after}）"
    );
    assert!(
        hearts >= 1,
        "林地溪流：获得的应是**红桃**牌（手中红桃 {hearts} 张）"
    );
    assert_eq!(
        hand.len() as i64,
        h_before as i64,
        "林地溪流：打出 −1、获得 1 ⇒ 手牌数不变（{h_before} → {}）",
        hand.len()
    );
}
