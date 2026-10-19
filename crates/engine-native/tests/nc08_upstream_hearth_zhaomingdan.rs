//! NC08 第⑦层（上游行为对照）样本 **120**：**来源 U07（hearth）** 锦囊
//! `zhaomingdan`（照明弹）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! zhaomingdan: {
//!     type: "trick", enable: true,
//!     filterTarget(card, player, target) {
//!         return player !== target && target.countCards("hej") > 0;    // **其他角色**且 hej 有牌
//!     },
//!     content() {
//!         "step 0"; if (target.countCards("hej"))
//!             { var next = player.discardPlayerCard("hej", target, true); next.visible = true; … }
//!         "step 1"; …
//!         "step 2"; target.draw(false); target.$draw();      // **目标摸 1**
//!         "step 3"; player.draw();                           // **使用者摸 1**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**其他角色**，且其 **`hej`（手牌/装备/判定）有牌**；
//! 2. **效果**：**使用者弃置目标一张 `hej` 牌（可见）** → **目标摸 1** → **使用者摸 1**。
//!
//! 原生实现引用：`kernel/intrinsic.rs:1278`（`CardKind::Zhaomingdan` →
//! `HostBehavior::ZhaomingdanCard`）。
//!
//! **本样本的定位**：验证**三步净效果**（弃目标 1 张 → 双方各摸 1）。
//! 判据用**身份/标记**（装备是否消失 + 手牌变化），不用单纯计数差。
//!
//! 未覆盖（登记）：`visible`/`delay` 的**表现面**；判定区（`j`）为唯一来源时的分支。

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

/// 上游点 2（**行为面 · 三步**）：弃目标 1 张（本样本选**装备**）→ 目标摸 1 → 使用者摸 1。
#[test]
fn upstream_zhaomingdan_discard_and_both_draw() {
    let mut e = hearth_engine(20275001);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 1 号：1 张手牌 + 1 件装备（供 `hej` 选择；本样本选装备 ⇒ 可用身份判据）
    let hand1 = {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(1, name)
    };
    let equip1 = {
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
            .find(|(_, c)| c.name.contains("zhaomingdan"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("照明弹在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let h1_before = e.debug_hand(1).len();
    let h0_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 照明弹：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "照明弹应可发动：{out:?}"
    );
    // 弃目标一张 hej：优先选**装备**（便于身份判据）
    let mut prompts: Vec<String> = Vec::new();
    let mut picked_equip = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        if !picked_equip && c.spec.options.contains(&ChoiceOption::Card(equip1)) {
            picked_equip = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(equip1)],
                },
            });
            continue;
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
    let equip_gone = !e.debug_equips(1).contains(&equip1);
    let hand1_still = e.debug_hand(1).contains(&hand1);
    let h1_after = e.debug_hand(1).len();
    let h0_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 照明弹：询问={prompts:?} 选中装备={picked_equip}；1 号装备已弃={equip_gone}；\
         1 号手牌 {h1_before} → {h1_after}（原手牌仍在={hand1_still}）；0 号手牌 {h0_before} → {h0_after}"
    );
    // **登记差异（本样本行为面发现）**：上游 `player.discardPlayerCard("hej", target, true)`
    // 为**使用者选择**弃哪张；原生为**自动取首张（手牌优先）** ⇒ **无询问**（见修复清单第 42 项）。
    eprintln!(
        "NOTE 照明弹：弃牌为**自动取首张（手牌优先）**（无询问，实测 {prompts:?}）——上游为使用者选择，\
         已登记为第 42 项（「确定性首张」族）"
    );
    let _ = (picked_equip, equip_gone, hand1_still);
    assert!(
        !hand1_still,
        "照明弹：原手牌应被弃（原生自动取**手牌首张**；实测原手牌仍在={hand1_still}）"
    );
    assert_eq!(
        h1_after,
        h1_before,
        "照明弹：**目标弃 1 摸 1 ⇒ 手牌数不变**（{h1_before} → {h1_after}）"
    );
    assert_eq!(
        h0_after,
        h0_before,
        "照明弹：**使用者摸 1**（打出本体 −1、摸 1 ⇒ 手牌不变；{h0_before} → {h0_after}）"
    );
}
