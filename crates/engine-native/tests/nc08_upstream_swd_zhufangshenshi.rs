//! NC08 第⑦层（上游行为对照）样本 158：**来源 U06（swd）** 法器
//! `zhufangshenshi`（朱fang神石）——**`global` 卡窗口的第 7 张验证**（`useCardAfter` 类）。
//!
//! 上游源码（`skill.js` 的 **`g_zhufangshenshi`**，**逐字**）：
//!
//! ```js
//! zhufangshenshi: { type: "jiqi", global: "g_zhufangshenshi" },   // 卡本体无 content
//! g_zhufangshenshi: {
//!     trigger: { player: "useCardAfter" },                         // **本人使用牌后**
//!     forced: true, popup: false,
//!     filter(event, player) { return event.card.name === "zhufangshenshi"; },  // 仅**本卡**
//!     content() { player.draw(); },                                // **摸 1**
//! },
//! ```
//!
//! **本样本的定位**：**使用朱fang神石后摸 1**（`forced`）。
//!
//! 未覆盖（登记）：`popup: false` 的表现面；「非本卡」的负例。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游 `g_zhufangshenshi`（**forced**）：使用本卡后**摸 1**。
#[test]
fn upstream_zhufangshenshi_draws_after_use() {
    let mut e = swd_engine(20289001);
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
            .find(|(_, c)| c.name.contains("zhufangshenshi"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("朱fang神石在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let deck_before = e.state_core().deck.len();
    // 形态：`[]`→Accepted（全局普查已证）；`[1]` 亦可，先试 `[1]`
    let mut out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    if !matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }) {
        out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card, targets: vec![] },
        });
    }
    eprintln!("DBG 朱fang神石：使用={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "朱fang神石应可使用：{out:?}"
    );
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        let sel: Vec<ChoiceOption> = if !cards.is_empty() {
            vec![cards[0]]
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
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
    let deck_after = e.state_core().deck.len();
    eprintln!(
        "DBG 朱fang神石：牌堆 {deck_before} → {deck_after}（使用后摸 1 ⇒ 应 −1）"
    );
    assert_eq!(
        deck_before as i64 - deck_after as i64,
        1,
        "朱fang神石（**forced**）：使用本卡后应**摸 1**（牌堆 {deck_before} → {deck_after}）"
    );
}
