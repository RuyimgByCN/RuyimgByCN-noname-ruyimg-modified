//! NC08 第⑦层（上游行为对照）样本 88：**来源 U05（古剑）** 锦囊
//! `shujinsan`（舒筋散）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`shujinsan`）：
//!
//! ```js
//! shujinsan: {
//!     type: "trick", enable: true,
//!     filterTarget(card, player, target) { return target.countCards("he") > 0; },   // he 有牌（含自己）
//!     content() {
//!         "step 0";
//!         target.chooseToDiscard("he", [1, target.countCards("he")], "弃置任意张牌并摸等量的牌");   // **目标自选**
//!         "step 1";
//!         if (result.bool) target.draw(result.cards.length);      // **摸等量**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：`he` 有牌的角色（**含自己**）；
//! 2. **流程**：目标**自选弃任意张 `he` 牌**（1 张起）→ **摸等量**。
//!
//! 原生实现引用：宿主行为（`HostBehavior::Shujinsan*`；野升样本注释称「**舒筋散机器同构**」）
//! —— 本样本以**行为面**观测。
//!
//! **登记差异**：本样本**未发现**差异（弃 N 摸 N、装备可弃、门槛方向一致）；
//! 保留项：`ai`（弃牌评分）策略面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
            .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：目标**弃 2 张手牌 → 摸 2**（装备区不受影响）。
#[test]
fn upstream_shujinsan_discards_and_draws_equal() {
    let mut e = gujian_engine(20271801);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号：两张手牌 + 一件装备（装备用于验证「未被弃」）
    let hand_cards: Vec<engine_native::CardId> = (0..2)
        .map(|_| {
            let name = e
                .catalog()
                .cards
                .iter()
                .find(|c| c.name == "shan")
                .map(|c| c.name)
                .unwrap();
            e.debug_place(1, name)
        })
        .collect();
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
            .find(|(_, c)| c.name.contains("shujinsan"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("舒筋散在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let h1_before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 舒筋散：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "舒筋散应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut discarded = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 1 号的弃牌询问：弃那两张手牌（不用装备，验证装备不受影响）
        if !discarded && c.observer == 1 {
            let sel: Vec<ChoiceOption> = hand_cards
                .iter()
                .filter(|h| c.spec.options.contains(&ChoiceOption::Card(**h)))
                .map(|h| ChoiceOption::Card(*h))
                .collect();
            if sel.len() == hand_cards.len() {
                discarded = true;
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
    let hand = e.debug_hand(1);
    eprintln!(
        "DBG 舒筋散：询问={prompts:?} 弃牌={discarded} 1 号手牌 {h1_before} → {}（装备仍在={}）",
        hand.len(),
        e.debug_equips(1).contains(&equip)
    );
    assert!(discarded, "舒筋散：目标应出现弃牌询问（自选）");
    assert_eq!(
        hand.len(),
        h1_before,
        "舒筋散：弃 2 摸 2 → 手牌数应不变（{h1_before} → {}）",
        hand.len()
    );
    assert!(
        !hand_cards.iter().any(|c| hand.contains(c)),
        "舒筋散：被弃的两张手牌应已离手"
    );
    assert!(
        e.debug_equips(1).contains(&equip),
        "舒筋散：**装备区**牌未被弃（本样本只弃手牌）"
    );
}

/// 上游点 1（**门槛反例**）：目标 `he` 无牌 → 不可指定。
#[test]
fn upstream_shujinsan_target_needs_he_cards() {
    let mut e = gujian_engine(20271802);
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
            .find(|(_, c)| c.name.contains("shujinsan"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("舒筋散在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    eprintln!("DBG 舒筋散（目标无 he 牌）→ {out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "舒筋散：目标 he 无牌时应被拒（上游 `countCards(\"he\") > 0`）：{out:?}"
    );
    drain(&mut e);
}
