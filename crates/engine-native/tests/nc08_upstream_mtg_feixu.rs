//! NC08 第⑦层（上游行为对照）样本 135：**来源 U09（mtg）** 地牌
//! `mtg_feixu`（废墟）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`）：
//!
//! ```js
//! mtg_feixu: {
//!     type: "land", fullborder: "wood",
//!     enable(card, player) { return !player.hasSkill("land_used"); },
//!     notarget: true,
//!     content() {
//!         game.changeLand("mtg_feixu", player);
//!         player.discoverCard(ui.discardPile.childNodes);    // **从弃牌堆「发现」一张并获得**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **使用限制**：**每回合限一次**；**无目标**；
//! 2. **效果**：**从弃牌堆发现（选择）一张牌并获得**。
//!
//! 原生实现引用：`kernel/intrinsic.rs`（`CardKind::Feixu` → mtg 地牌机）。
//!
//! **本样本的定位**：先让弃牌堆非空（打出一张【杀】结算后入弃堆），再用废墟**从弃牌堆获得一张**。
//!
//! 未覆盖（登记）：**弃牌堆为空**时（上游 `discoverCard` 无候选）的行为；`land_used` 每回合重置。

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

/// 上游点 2（**行为面**）：**从弃牌堆获得一张**。
#[test]
fn upstream_mtg_feixu_discovers_from_discard() {
    let mut e = mtg_engine(20286801);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // ① 先打出一张【杀】（结算后入弃牌堆，使弃堆非空）
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let out1 = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(
            out1,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "先出【杀】应被受理：{out1:?}"
    );
    // 应答【闪】询问（1 号放弃 ⇒ 受伤，无妨）
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
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
    let discard_before = e.state_core().discard.len();
    // ② 使用废墟
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("feixu"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("废墟在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let h_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 废墟：弃堆={discard_before}；发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "废墟应可发动（无目标）：{out:?}"
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
    let discard_after = e.state_core().discard.len();
    let h_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 废墟：询问={prompts:?} 已选={picked}；弃堆 {discard_before} → {discard_after}；\
         手牌 {h_before} → {h_after}"
    );
    assert!(picked, "废墟：应出现**从弃牌堆发现**的询问");
    assert!(
        discard_after < discard_before + 1,
        "废墟：应从弃牌堆**取走**一张（弃堆 {discard_before} → {discard_after}）"
    );
    assert_eq!(
        h_after as i64,
        h_before as i64,
        "废墟：打出 −1、从弃堆获得 1 ⇒ 手牌数不变（{h_before} → {h_after}）"
    );
}
