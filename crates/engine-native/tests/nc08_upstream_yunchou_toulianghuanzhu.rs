//! NC08 第⑦层：**来源 U05（运筹）** 锦囊 `toulianghuanzhu`（偷梁换柱）——
//! 由**受理面普查**（台账第 127 行）发现的 panic 候选，本文件**专门核实**。
//!
//! 上游源码（`.../yunchou/card.js`，`toulianghuanzhu`）：
//!
//! ```js
//! toulianghuanzhu: {
//!     type: "trick", enable: true,
//!     filterTarget(card, player, target) { return target.countCards("h") > 0 && player != target; },
//!     async content(event, trigger, player) {
//!         const { target } = event;
//!         if (!target.countCards("h")) return;
//!         const hs = player.getCards("h");
//!         if (hs.length) {
//!             const result = await player.chooseButton(2, [
//!                 `###偷梁换柱###…你可以选择你与${get.translation(target)}其的各一张手牌以交换`,
//!                 `${get.translation(target)}的手牌`, target.ge…   // **双方各选一张手牌交换**
//!             ]);
//!         }
//!     },
//! },
//! ```
//!
//! **本文件的定位**：普查中该卡触发 `machine.rs:314` 的
//! `expect("non-empty selection")` panic —— 但普查**未应答**中途询问（可能因此空选）⇒
//! **本用例以「正确应答所有询问」的方式复测**，用于判定 panic 是**实现缺陷**还是**普查夹具问题**。
//! **不预设结论**：panic 则记录为**缺陷**（第 48 项）；否则记录为**普查夹具问题**（更正普查口径）。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
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

/// **复测**：正确应答所有询问（含空选时的 `Pass`），观察是否仍 panic。
#[test]
fn upstream_toulianghuanzhu_with_proper_answers() {
    let mut e = yunchou_engine(20275701);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 双方各有手牌（上游要求 `player != target` 且目标手牌非空）
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(0, name);
        e.debug_place(1, name);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("toulianghuanzhu"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("偷梁换柱在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 偷梁换柱：发动={out:?}");
    // **正确应答**：逐个询问给出非空选择（有牌选牌，否则 Pass/取消）
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..32 {
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
        let need = c.spec.min.max(1);
        let sel: Vec<ChoiceOption> = if cards.len() >= need {
            cards.into_iter().take(need).collect()
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
        } else if !cards.is_empty() {
            cards
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(need).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
    drain(&mut e);
    eprintln!(
        "DBG 偷梁换柱：询问={prompts:?}；0 号手牌={}；1 号手牌={}（正常应答后未 panic）",
        e.debug_hand(0).len(),
        e.debug_hand(1).len()
    );
}
