//! NC08 第⑦层（上游行为对照）样本 116：**来源 U07（hearth）** 锦囊
//! `yuansuhuimie`（元素毁灭）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! yuansuhuimie: {
//!     type: "trick", cardnature: "thunder", enable: true,
//!     selectTarget: -1, filterTarget: true, reverseOrder: true,     // **全体、逆序结算**
//!     content() {
//!         "step 0"; target.chooseToDiscard([1, 2], "he")…;          // **弃 1–2 张**
//!         "step 1"; if (!result.bool || result.cards.length < 2) { … }   // **少弃 → 雷伤**
//!     },
//! },
//! ```
//!
//! 依据（规则基准 golden 表第 55 行）：「元素毁灭（**逐目标弃 1–2 每少弃一张受 1 雷伤**）」。
//!
//! 由 JS/基准推导的规则点：
//! 1. **目标**：**全体**（`selectTarget: -1`），**逆序结算**（`reverseOrder`）；
//! 2. **效果**：每名目标**弃 1–2 张**；**每少弃一张**（相对 2）**受 1 点雷属性伤害**
//!    ⇒ 弃 2 无事、弃 1 受 1、弃 0（或无牌）受 2。
//!
//! 原生实现引用：`content/host/step/hearth.rs`（`HearthYuansuhuimieCard`）。
//!
//! **本样本的定位**：**三档并列验证**（弃 2 / 弃 1 / 无牌 ⇒ 0 / 1 / 2 点雷伤）。
//!
//! 未覆盖（登记）：**逆序结算**的顺序面；`cardnature: thunder` 的属性修饰面（藤甲等）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **hearth 包**建局。
fn hearth_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-hearth-upstream-noname/assets/cards.json"))
        .expect("读 hearth 包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面 · 三档**）：弃 2 → 0 伤；弃 1 → 1 伤；无牌 → 2 伤。
#[test]
fn upstream_yuansuhuimie_discard_shortfall_damage() {
    let mut e = hearth_engine(20274601);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号：2 张（全弃 ⇒ 0 伤）；2 号：1 张（弃 1 ⇒ 1 伤）；3 号：0 张（⇒ 2 伤）
    let mut hand1 = Vec::new();
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        hand1.push(e.debug_place(1, name));
    }
    let hand2 = {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(2, name)
    };
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("yuansuhuimie"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("元素毁灭在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp_before = [e.debug_hp(1), e.debug_hp(2), e.debug_hp(3)];
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        // `selectTarget: -1`（全体）：先试「无显式目标」形态（对照 AOE 与 toself 的两种受理口径）
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 元素毁灭：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "元素毁灭应可发动（全体目标）：{out:?}"
    );
    // 逐目标响应：1 号弃 2 张；2 号弃 1 张；3 号无牌（弃 0）
    let mut prompts: Vec<String> = Vec::new();
    let mut done = [false; 4usize];
    for _ in 0..48 {
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
        let obs = c.observer as usize;
        if !done[obs] && !cards.is_empty() {
            done[obs] = true;
            // 1 号（obs=1）弃满 2 张；其余弃 1 张（2 号）/ 尽可能少
            let take = if c.observer == 1 { 2 } else { 1 };
            let sel: Vec<ChoiceOption> = cards.into_iter().take(take).collect();
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
    let d1 = hp_before[0] - e.debug_hp(1);
    let d2 = hp_before[1] - e.debug_hp(2);
    let d3 = hp_before[2] - e.debug_hp(3);
    eprintln!("DBG 元素毁灭：询问={prompts:?}；1 号（弃 2）掉血 {d1}；2 号（弃 1）掉血 {d2}；3 号（无牌）掉血 {d3}");
    let _ = (hand1, hand2);
    assert_eq!(d1, 0, "元素毁灭：**弃满 2 张** ⇒ 不受伤害（实测掉血 {d1}）");
    assert_eq!(d2, 1, "元素毁灭：**少弃 1 张** ⇒ 受 **1** 点雷伤（实测掉血 {d2}）");
    assert_eq!(d3, 2, "元素毁灭：**无牌可弃**（少 2 张）⇒ 受 **2** 点雷伤（实测掉血 {d3}）");
}
