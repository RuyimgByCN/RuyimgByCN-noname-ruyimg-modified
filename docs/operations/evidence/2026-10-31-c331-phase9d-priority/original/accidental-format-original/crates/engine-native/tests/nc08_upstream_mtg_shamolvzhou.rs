//! NC08 第⑦层（上游行为对照）样本 134：**来源 U09（mtg）** 地牌
//! `mtg_shamolvzhou`（沙漠绿洲）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`）：
//!
//! ```js
//! mtg_shamolvzhou: {
//!     type: "land", fullborder: "wood",
//!     enable(card, player) { return !player.hasSkill("land_used"); },
//!     notarget: true,
//!     content() {
//!         game.changeLand("mtg_shamolvzhou", player);
//!         player.discoverCard(get.inpile("basic"));    // **从基本牌池「发现」一张并获得**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **使用限制**：**每回合限一次**；**无目标**；
//! 2. **效果**：**从基本牌池发现一张牌并获得**（`discoverCard` = 从候选集合中选择）。
//!
//! 原生实现引用：`kernel/intrinsic.rs`（`CardKind::Shamolvzhou` → mtg 地牌机）。
//!
//! **本样本的定位**：验证「**发现并获得一张基本牌**」（手牌 +1 且该牌为基本牌）。
//!
//! 未覆盖（登记）：`discoverCard` 的**候选集合构成**（上游为 `get.inpile("basic")`）；
//! `land_used` 每回合重置。

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

/// 上游点 2（**行为面**）：**发现并获得一张基本牌**。
#[test]
fn upstream_mtg_shamolvzhou_discovers_basic() {
    let mut e = mtg_engine(20286701);
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
            .find(|(_, c)| c.name.contains("shamolvzhou"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("沙漠绿洲在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
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
    eprintln!("DBG 沙漠绿洲：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "沙漠绿洲应可发动（无目标）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut picked = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
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
    let hand = e.debug_hand(0);
    let basics = hand
        .iter()
        .filter(|c| {
            matches!(
                e.catalog().card_kind(**c),
                engine_native::CardKind::Sha
                    | engine_native::CardKind::Shan
                    | engine_native::CardKind::Tao
                    | engine_native::CardKind::Jiu
            )
        })
        .count();
    eprintln!(
        "DBG 沙漠绿洲：询问={prompts:?} 已选={picked}；手牌 {h_before} → {}（其中基本牌 {basics}）",
        hand.len()
    );
    assert!(picked, "沙漠绿洲：应出现**发现（选择）**的询问");
    assert_eq!(
        hand.len() as i64,
        h_before as i64,
        "沙漠绿洲：打出 −1、发现获得 1 ⇒ 手牌数不变（{h_before} → {}）",
        hand.len()
    );
    assert!(
        basics >= 1,
        "沙漠绿洲：发现的应是**基本牌**（手中基本牌 {basics} 张）"
    );
}
