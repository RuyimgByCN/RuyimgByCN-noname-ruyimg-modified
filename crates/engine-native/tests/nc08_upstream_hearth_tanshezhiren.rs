//! NC08 第⑦层（上游行为对照）样本 117：**来源 U07（hearth）** 锦囊
//! `tanshezhiren`（坦蛇至人）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! tanshezhiren: {
//!     type: "trick", enable: true,
//!     filterTarget(card, player, target) { return target === player; },   // 仅自己
//!     selectTarget: -1, modTarget: true,
//!     content() {
//!         "step 0"; event.current = target;
//!                   event.num = game.countPlayer(); if (event.num % 2 === 0) event.num--;   // **奇数**
//!         "step 1"; if (event.num) {
//!             var enemies = event.current.getEnemies(); enemies.remove(player);           // **敌人**（排除自己）
//!             … if (enemies.length) {
//!                 var enemy = enemies.randomGet();                                        // **随机**
//!                 enemy.discard(enemy.getCards("h").randomGet());                         // **随机弃 1 张手牌**
//!                 event.current = enemy; event.num--; event.redo();                        // **链式传递**
//!             } }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**仅自己**；
//! 2. **效果**：**链式**——从自己开始，每次取「当前角色的敌人（排除自己、且手牌非空）」中**一个**，
//!    令其**弃一张手牌**，并以该角色为当前继续；**次数 = 角色数（偶数则减 1 取奇）**；
//! 3. **登记差异**（基准 golden 表第 55 行）：「**随机改确定性**」——上游 `randomGet()` 两处随机，
//!    原生为**座位序 + 首张**。
//!
//! **本样本的定位**：验证**链式弃牌的次数/覆盖面**（4 人局 ⇒ 3 次），并**行为面确认**该登记差异。
//!
//! 未覆盖（登记）：`modTarget` 影响面；`recastable` 注释项（上游已注释）。

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

/// 上游点 2（**行为面**）：4 人局 ⇒ **链式 3 次弃牌**（每名被链到的角色各弃 1）。
#[test]
fn upstream_tanshezhiren_chain_discards() {
    let mut e = hearth_engine(20274701);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 其余三人各 1 张手牌（供链式弃牌）
    for p in 1..4 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(p, name);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("tanshezhiren"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("坦蛇至人在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let before: Vec<usize> = (1..4).map(|p| e.debug_hand(p).len()).collect();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 坦蛇至人：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "坦蛇至人应可发动（仅自己）：{out:?}"
    );
    // 链式过程可能带弃牌询问
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..32 {
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
        let sel: Vec<ChoiceOption> = if !cards.is_empty() {
            vec![cards[0]]
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
    let after: Vec<usize> = (1..4).map(|p| e.debug_hand(p).len()).collect();
    let discarded: usize = before.iter().zip(after.iter()).map(|(b, a)| b - a).sum();
    eprintln!(
        "DBG 坦蛇至人：询问={prompts:?}；1..3 号手牌 {before:?} → {after:?}（合计弃 {discarded} 张）"
    );
    assert_eq!(
        discarded, 3,
        "坦蛇至人：4 人局 ⇒ `num` 取奇 = **3** ⇒ 链式弃牌 **3 张**（实测 {discarded}）——\
         登记差异：原生为**座位序 + 首张**（上游为两处随机）"
    );
}
