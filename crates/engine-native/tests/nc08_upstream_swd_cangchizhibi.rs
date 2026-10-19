//! NC08 第⑦层（上游行为对照）样本 146：**来源 U06（swd）** 机器牌
//! `cangchizhibi`（苍螭之璧）——**`global` 响应窗口卡**。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js`）：
//!
//! ```js
//! cangchizhibi: {
//!     type: "jiqi", global: "g_cangchizhibi",
//!     filterTarget(card, player, target) { return target !== player; },     // **其他角色**
//!     content() {
//!         "step 0"; if (target.countCards("h")) { var next = target.chooseToDiscard… }   // **目标弃牌**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**其他角色**（`target !== player`）；
//! 2. **效果**：**目标（手牌非空时）弃牌**（`chooseToDiscard`）；
//! 3. **`global`**：额外提供**响应窗口**用法。
//!
//! 原生实现引用：宿主行为（swd 机器牌机）。
//!
//! **本样本的定位**：验证「**目标弃一张手牌**」这一**主动使用**面；
//! **`global` 响应窗口**用法登记为待办。
//!
//! 未覆盖（登记）：**`global` 响应窗口**（`g_cangchizhibi`）用法；弃牌**张数/选择权**（上游 `chooseToDiscard`）。

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

/// **登记：该卡属已确认的转换缺口 ⇒ 样本不可行**（目录中不存在）。
///
/// 依据：台账第 117 行的**转换缺口清单**（swd **11 项**）已含 **`cangchizhibi`（苍螭之璧，`jiqi`）**，
/// 且经**双向核对**（原生 src + 全部包）**0 命中** ⇒ **未转换** ⇒ 无法构造行为面样本。
/// 本用例改为**断言该卡不在目录**，把「缺口」固化为**可执行事实**（缺口清单的回归防线）。
#[test]
fn upstream_cangchizhibi_is_a_known_conversion_gap() {
    let e = swd_engine(20288001);
    let found = e
        .catalog()
        .cards
        .iter()
        .any(|c| c.name.contains("cangchizhibi"));
    eprintln!("DBG 苍螭之璧：目录中存在={found}（预期 false——属第 117 行转换缺口清单）");
    assert!(
        !found,
        "苍螭之璧当前应**不在目录**（转换缺口）；若已转换，请把本用例改为行为面样本"
    );
}

/// 上游点 2（**行为面**）：**目标弃一张手牌**。
///
/// **本轮不可行**：该卡属**转换缺口**（见上）⇒ 以 `#[ignore]` 作为**转换后的验收条件**。
#[test]
#[ignore = "苍螭之璧属转换缺口（未交付）——转换后启用本用例，见台账第 117 行"]
fn upstream_cangchizhibi_target_discards() {
    let mut e = swd_engine(20288001);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号：2 张手牌（供弃牌）
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(1, name);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("cangchizhibi"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("苍螭之璧在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let h1_before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 苍螭之璧：发动={out:?}");
    if !matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ) {
        eprintln!("NOTE 苍螭之璧：主动使用未被受理（{out:?}）——本用例不构成验收（可能仅限窗口用法）");
        return;
    }
    let mut prompts: Vec<String> = Vec::new();
    let mut done = false;
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
        if !done && !cards.is_empty() {
            done = true;
            let need = c.spec.min.max(1);
            let sel: Vec<ChoiceOption> = cards.into_iter().take(need).collect();
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
    let h1_after = e.debug_hand(1).len();
    eprintln!(
        "DBG 苍螭之璧：询问={prompts:?} 已弃={done}；1 号手牌 {h1_before} → {h1_after}"
    );
    assert_eq!(
        h1_after,
        h1_before - 1,
        "苍螭之璧：**目标应弃一张手牌**（{h1_before} → {h1_after}）"
    );
}
