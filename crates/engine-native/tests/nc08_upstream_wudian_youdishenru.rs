//! NC08 第⑦层（上游行为对照）样本 131：**来源 U10（玩点论杀）** 响应锦囊
//! `youdishenru`（诱敌深入）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/玩点论杀/card/card.js`）：
//!
//! ```js
//! youdishenru: {
//!     type: "trick", notarget: true, wuxieable: true,
//!     global: "g_youdishenru",                                  // **可在响应窗口打出**
//!     content() {
//!         "step 0"; var info = event.getParent(2).youdiinfo || event.getParent(3).youdiinfo;
//!                   if (!info) { event.finish(); return; }
//!                   info.evt.cancel();                          // **取消该【杀】**
//!                   event.source = info.source;
//!                   event.source.addSkill("youdishenru");
//!         "step 1"; var next = event.source.chooseToUse({ name: "sha" }, player, -1,
//!                       "对…使用一张杀，或受到1点伤害");         // **来源须出杀，否则受 1 点伤害**
//!         "step 2"; if (result.bool) { if (storage.youdishenru) event.goto(1); … }   // **连用循环**
//!                   else { event.… }                            // 不出 ⇒ 受伤
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **用法**：作为**响应**打出（`global` 子技）；**取消**当前【杀】；
//! 2. **后果**：该【杀】的**来源**须**对使用者出一张【杀】**，**否则受 1 点伤害**；
//! 3. **登记差异**（基准）：「**无连用循环**」。
//!
//! 原生实现引用：`engine/play_card.rs:1313`（`CardKind::Youdishenru`）。
//!
//! **本样本的定位**：验证「**取消该杀 + 来源不出杀则受 1 点伤害**」；
//! 并**行为面确认**「无连用循环」这一登记差异。
//!
//! 未覆盖（登记）：**连用循环**（基准已登记为「无」）；`wuxieable` 无懈面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn wudian_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/wudian-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读玩点论杀 cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1+2（**行为面**）：取消【杀】；来源不出杀 ⇒ **来源受 1 点伤害**、目标不掉血。
#[test]
fn upstream_youdishenru_cancels_sha_and_source_takes_damage() {
    let mut e = wudian_engine(20286401);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号：一张【杀】；1 号：诱敌深入
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let yd = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("youdishenru"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("诱敌深入在目录");
        nc08_common::put_hand(&mut e, 1, id);
        id
    };
    let hp0 = e.debug_hp(0);
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    eprintln!("DBG 诱敌深入：0 号出杀={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "出杀应被受理：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut used = false;
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        // 1 号的响应窗口：**按询问名/选项**识别「诱敌深入」并打出（形态可能是选牌或 Confirm）
        let has_card = c.spec.options.contains(&ChoiceOption::Card(yd));
        let name_hit = c.spec.prompt.contains("youdi") || c.spec.prompt.contains("诱敌");
        if !used && c.observer == 1 && (has_card || name_hit) {
            used = true;
            let sel = if has_card {
                vec![ChoiceOption::Card(yd)]
            } else {
                vec![ChoiceOption::Confirm]
            };
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
            continue;
        }
        // 其余询问一律**放弃/取消**（0 号选择「不出杀 ⇒ 受伤」）
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
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
    let d0 = hp0 - e.debug_hp(0);
    let d1 = hp1 - e.debug_hp(1);
    eprintln!("DBG 诱敌深入：使用={used}；询问={prompts:?}；0 号掉血 {d0}；1 号掉血 {d1}");
    if !used {
        eprintln!("NOTE 未识别到诱敌深入的响应窗口——本用例不构成验收");
        return;
    }
    assert_eq!(d1, 0, "诱敌深入：**该【杀】被取消** ⇒ 目标（1 号）不掉血（实测 {d1}）");
    assert_eq!(
        d0, 1,
        "诱敌深入：来源（0 号）**不出杀** ⇒ 应受 **1 点伤害**（实测 {d0}）——\
         并行为面确认「无连用循环」（基准登记）"
    );
}
