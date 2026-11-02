//! NC08 第⑦层（上游行为对照）样本 36：`guicai`（鬼才，**标准面** / 司马懿）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! guicai: {
//!     trigger: { global: "judge" },                 // **任何角色**的判定
//!     preHidden: true,
//!     filter(event, player) {
//!         return player.hasCards(get.mode() === "guozhan" ? "hes" : "hs");   // 非国战：手牌+特殊区
//!     },
//!     async cost(event, trigger, player) {
//!         event.result = await player.chooseCard({ … }).forResult();         // 打出一张牌替换判定牌
//!     },
//!     // position: get.mode() === "guozhan" ? "hes" : "hs"
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 任何角色的判定**（`global: "judge"`，不只自己）；
//! 2. **可打出的牌**：非国战为 **`hs`（手牌 + 特殊区）**，**不限花色**（与鬼道的「黑色牌」不同）；
//! 3. **效果**：以打出的牌**替换**当前判定牌。
//!
//! 原生使用共享 Judge 改判候选函数；鬼才非国战仅手牌，国战再含装备。
//! Native 特殊区/修饰器与其来源完整流程仍未覆盖。
//!
//! **同类登记**：`guidao`（鬼道）**不在 `character/standard/skill.js`**（仅见于 `character/sb/`）
//! → 与「据守」同属**非标准面技能**，本样本**不**为它建样本（登记）。
//!
//! 未覆盖（登记）：`filterCard` 的 `checkMod(..., "unchanged", ...)` 门控（牌可用性修饰）；
//! `preHidden`/`audio` 表现面。

mod nc08_common;
use engine_native::catalog::Suit;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 在 0 号 Draw 阶段放入【乐不思蜀】→ **下一轮**判定阶段消费（判定区在 Judge begin 处被 take）。
fn seed_delay_trick(e: &mut Engine, owner: u8) {
    let le = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "le")
        .expect("乐不思蜀在目录") as engine_native::CardId;
    let core = e.state_core_mut();
    core.player_mut(owner).judge.push(le);
}

/// 上游点 1+3：**他人**的判定也可改判（`global: judge`）→ 打出一张**红色手牌**替换。
#[test]
fn upstream_guicai_rewrites_others_judgement_with_any_hand_card() {
    let mut e = Engine::new(20266601, 4);
    e.debug_grant(1, "鬼才"); // 改判者 = 1 号（非判定者）
    assert!(advance_to_phase(&mut e, 0, Phase::Draw), "到 0 号摸牌阶段");
    seed_delay_trick(&mut e, 0);
    // 1 号持一张**红色**手牌（上游 hs 不限花色）
    let red = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.suit == Suit::Heart && c.name == "shan")
            .expect("红桃闪") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 1, id);
        id
    };
    // 推进一整轮，寻找改判询问（鬼才的选项里应含该红牌）
    let mut saw_guicai_option = false;
    let mut specs: Vec<String> = Vec::new();
    for _ in 0..200 {
        let Some(c) = e.open_choice().cloned() else {
            let actor = e.turn().current;
            let _ = e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::NextPhase,
            });
            continue;
        };
        specs.push(format!("{}:{:?}", c.spec.prompt, c.spec.options.len()));
        if c.spec.options.contains(&ChoiceOption::Card(red)) {
            saw_guicai_option = true;
            // 打出该红牌改判
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(red)],
                },
            });
            break;
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
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
    eprintln!("DBG 鬼才：红牌入候选={saw_guicai_option} 询问序列={specs:?}");
    assert!(
        saw_guicai_option,
        "鬼才：**任意手牌**（含红色）应可作为改判牌（上游 hs 不限花色）"
    );
}

/// Source uses hand/special outside Kingdoms, and additionally equipment in Kingdoms.
#[test]
fn upstream_guicai_positions_are_mode_specific() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("guicai: {").expect("上游鬼才");
    let seg: String = up[idx..].chars().take(900).collect();
    assert!(
        seg.contains("global: \"judge\"") && seg.contains("\"hes\" : \"hs\""),
        "上游依据：任何角色判定 + 非国战 hs：{seg}"
    );
}

/// 同类登记（源码事实）：`guidao` **不在**标准面 → 不作为第⑦层样本。
#[test]
fn guidao_is_not_in_the_standard_face() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    assert!(
        !up.contains("guidao: {"),
        "登记：`guidao` 不在 character/standard/skill.js（仅见于 character/sb/）→ 非标准面技能"
    );
    let sb = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/sb/skill.js"),
    )
    .expect("读上游 sb/skill.js");
    assert!(
        sb.contains("guidao: {"),
        "对照：`guidao` 确在 sb 面（非基准面）"
    );
}
