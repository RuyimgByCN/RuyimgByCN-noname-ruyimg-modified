//! NC08 第⑦层（上游行为对照）样本 129：**来源 U09（mtg）** 地牌
//! `mtg_haidao`（海岛）——**mtg 来源的首个行为面样本**。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`）：
//!
//! ```js
//! mtg_haidao: {
//!     type: "land", fullborder: "wood",
//!     enable(card, player) { return !player.hasSkill("land_used"); },   // **每回合限一次**
//!     notarget: true,                                                   // **无目标**
//!     content() {
//!         "step 0"; game.changeLand("mtg_haidao", player);              // **变更地牌**
//!                   if (player.isHealthy()) { player.changeHujia(); event.finish(); }   // **满血 → +1 护甲**
//!                   else { player.chooseToDiscard("he", "海岛")… }       // 否则**弃 1 张（可取消）**
//!         "step 1"; if (result.bool) player.recover();                   // **弃了 → 回复 1**
//!                   else player.changeHujia();                           // **取消 → +1 护甲**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **使用限制**：**每回合限一次**（`land_used`）；**无目标**（`notarget`）；
//! 2. **三分支**：**满血** → **+1 护甲**；**受伤且弃 1 张** → **回复 1**；**受伤且取消** → **+1 护甲**。
//!
//! 原生实现引用：`kernel/intrinsic.rs:875`（`CardKind::Haidao` → `HostBehavior::MtgHaidaoLand`）。
//!
//! **本样本的定位**：验证**三分支**（满血护甲 / 受伤弃牌回复 / 受伤取消护甲）。
//!
//! 未覆盖（登记）：`game.changeLand`（**地牌变更**）的完整语义；`land_used` 的**每回合重置**。

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

/// 跑一次：`wounded` 决定 0 号是否受伤；`discard` 决定受伤时是否弃牌。返回 `(hp 变化, 护甲变化)`。
fn run(seed: u64, wounded: bool, discard: bool) -> (i32, i32) {
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
            .find(|(_, c)| c.name.contains("haidao"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("海岛在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(0, name);
    }
    if wounded {
        let max0 = e.state_core().player(0).max_hp as i32;
        e.debug_set_hp(0, max0 - 1);
    }
    let hp0 = e.debug_hp(0);
    let hj0 = e.state_core().player(0).hujia as i32;
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
        "海岛应可发动（无目标）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut done = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        if !done && discard && !cards.is_empty() {
            done = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: vec![cards[0]] },
            });
            continue;
        }
        // 不弃牌：优先 Pass / 空选（上游为**可取消**）
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
    let d_hp = e.debug_hp(0) - hp0;
    let d_hj = e.state_core().player(0).hujia as i32 - hj0;
    eprintln!(
        "DBG 海岛：wounded={wounded} discard={discard} → hp{d_hp:+}、护甲{d_hj:+}（询问{prompts:?}）"
    );
    (d_hp, d_hj)
}

/// 上游点 2（**行为面 · 三分支**）：满血 → +1 护甲；受伤弃牌 → 回复 1；受伤取消 → +1 护甲。
#[test]
fn upstream_mtg_haidao_three_branches() {
    let (hp_full, hj_full) = run(20286201, false, false);
    let (hp_disc, hj_disc) = run(20286202, true, true);
    let (hp_cancel, hj_cancel) = run(20286203, true, false);
    eprintln!(
        "DBG 海岛：满血 → hp{hp_full:+}/护甲{hj_full:+}；受伤弃牌 → hp{hp_disc:+}/护甲{hj_disc:+}；\
         受伤取消 → hp{hp_cancel:+}/护甲{hj_cancel:+}"
    );
    assert_eq!(hj_full, 1, "海岛（**满血**）：应 **+1 护甲**（且不回复）");
    assert_eq!(hp_full, 0, "海岛（满血）：不应回复体力");
    assert_eq!(hp_disc, 1, "海岛（**受伤且弃 1 张**）：应 **回复 1**（且不加护甲）");
    assert_eq!(hj_disc, 0, "海岛（受伤且弃牌）：不应加护甲");
    assert_eq!(hj_cancel, 1, "海岛（**受伤且取消**）：应 **+1 护甲**（且不回复）");
    assert_eq!(hp_cancel, 0, "海岛（受伤且取消）：不应回复体力");
}
