//! NC08 第⑦层（上游行为对照）样本 137：**来源 U09（mtg）** 地牌
//! `mtg_longlushanfeng`（龙颅山峰）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`）：
//!
//! ```js
//! mtg_longlushanfeng: {
//!     type: "land", fullborder: "wood",
//!     enable(card, player) { return !player.hasSkill("land_used"); },
//!     notarget: true,
//!     content() {
//!         "step 0"; game.changeLand("mtg_longlushanfeng", player);
//!                   player.chooseControl("获得杀", "获得闪").set("prompt", "选择一项");   // **二选一**
//!         "step 1"; if (result.control === "获得杀") {
//!                       player.gain(game.createCard("sha"), "gain2");                  // **生成【杀】**并获得
//!                   } else { player.gain(game.createCard("shan"), "gain2"); }          // **生成【闪】**并获得
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **使用限制**：**每回合限一次**；**无目标**；
//! 2. **二选一**：**获得一张【杀】** 或 **获得一张【闪】**；
//! 3. **生成语义**：上游用 **`game.createCard(...)`（凭空造牌）** ⇒ 属「**生成 vs 实体取牌**」族
//!    （已有 `mujiaren`/`yangpijuan`/`liufengsan` 三例）⇒ 本样本以**牌堆计数**核对是否为第 4 例。
//!
//! 原生实现引用：`kernel/intrinsic.rs`（`CardKind::Longlushanfeng` → mtg 地牌机）。
//!
//! 未覆盖（登记）：`land_used` 每回合重置；`ai` 面。

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

/// 跑一次：`pick_sha` 决定选「获得杀」还是「获得闪」。
/// 返回 `(手牌变化, 牌堆变化, 获得的是否为杀/闪)`。
fn run(seed: u64, pick_sha: bool) -> (i64, i64, bool) {
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
            .find(|(_, c)| c.name.contains("longlushanfeng"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("龙颅山峰在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let deck_before = e.state_core().deck.len() as i64;
    let h_before = e.debug_hand(0).len() as i64;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "龙颅山峰应可发动（无目标）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut chose = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // **二选一**：优先按选项序号选择；否则取第一项
        if !chose && c.spec.options.len() >= 2 {
            chose = true;
            let idx = if pick_sha { 0 } else { 1 };
            let sel = vec![c.spec.options[idx.min(c.spec.options.len() - 1)]];
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
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
    let hand = e.debug_hand(0);
    let want = if pick_sha { engine_native::CardKind::Sha } else { engine_native::CardKind::Shan };
    let got = hand.iter().any(|c| e.catalog().card_kind(*c) == want);
    let d_h = hand.len() as i64 - h_before;
    let d_deck = deck_before - e.state_core().deck.len() as i64;
    eprintln!(
        "DBG 龙颅山峰：pick_sha={pick_sha} → 手牌{d_h:+}、牌堆−{d_deck}、获得目标牌={got}（询问{prompts:?}）"
    );
    (d_h, d_deck, got)
}

/// 上游点 2+3（**行为面 · 二选一**）：获得一张【杀】/【闪】；并核对**生成 vs 实体**（牌堆计数）。
#[test]
fn upstream_mtg_longlushanfeng_two_choices() {
    let (h_sha, deck_sha, got_sha) = run(20287001, true);
    let (h_shan, deck_shan, got_shan) = run(20287002, false);
    eprintln!(
        "DBG 龙颅山峰：选杀 → 手牌{h_sha:+}/牌堆−{deck_sha}/获得杀={got_sha}；\
         选闪 → 手牌{h_shan:+}/牌堆−{deck_shan}/获得闪={got_shan}"
    );
    assert_eq!(h_sha, 0, "龙颅山峰（选杀）：打出 −1、获得 1 ⇒ 手牌不变（实测 {h_sha:+}）");
    assert!(got_sha, "龙颅山峰（选杀）：获得的应是**【杀】**");
    assert_eq!(h_shan, 0, "龙颅山峰（选闪）：打出 −1、获得 1 ⇒ 手牌不变（实测 {h_shan:+}）");
    assert!(got_shan, "龙颅山峰（选闪）：获得的应是**【闪】**");
    eprintln!(
        "NOTE 龙颅山峰（生成 vs 实体）：上游为 `game.createCard(...)`（**生成**）；\
         实测牌堆变化 选杀 −{deck_sha}、选闪 −{deck_shan} ⇒ 若为 0 则与上游一致（生成），\
         若为 1 则属「生成 vs 实体取牌」族第 4 例（须登记）"
    );
}
