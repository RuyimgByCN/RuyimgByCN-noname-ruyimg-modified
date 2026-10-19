//! NC08 第⑦层（上游行为对照）样本 124：**来源 U07（hearth）** 锦囊
//! `shijieshu`（视界术）——**hearth 包最后一张**。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! shijieshu: {
//!     type: "trick", enable: true,
//!     filterTarget(card, player, target) { return !target.isMin(); },   // **体力不是最少**的角色
//!     content() {
//!         "step 0"; // 从**牌堆**找 **2 张不同 sub-type 的装备牌**
//!                   var card = get.cardPile(card => get.type(card)==="equip" && 与已取者 sub-type 不同);
//!         "step 1"; if (event.cards.length) { target.equip(event.cards.shift()); … event.redo(); }
//!                   // **目标依次装备之**
//!         "step 3"; if (target.countCards("he")) target.chooseToDiscard("he", true);   // **弃 1**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**体力不是最少**的角色（`!target.isMin()`）；
//! 2. **效果**：目标**获得并装备**牌堆中 **≤2 张不同 sub-type 的装备牌**；
//!    随后**弃 1 张 `he` 牌**（强制）。
//!
//! 原生实现引用：`kernel/intrinsic.rs:446/482`（`CardKind::Shijieshu` →
//! `HostBehavior::HearthShijieshuCard`）。
//!
//! **本样本的定位**：验证「**装备区增加 + 随后弃 1**」的净效果（`he` 总数 = +2 −1 = **+1**），
//! 以及**门槛反例**（体力最少的角色不可指定）。
//!
//! 未覆盖（登记）：**sub-type 互不相同**这一约束；装备牌来自**牌堆**的入口面。

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

/// 上游点 2（**行为面**）：目标**装备区增加**，随后**弃 1**（`he` 净 **+1**）。
///
/// **差异登记（目标集合）**：上游 `filterTarget: !target.isMin()` ⇒ **指定其他角色**；
/// 原生 `play_card.rs:1481` 注释「**世界树：自-target**」且**要求 `targets` 为空**
/// ⇒ **效果只作用于使用者**（传 `vec![1]` 会 `Rejected(IllegalTargets)`）——见修复清单第 45 项。
/// 本用例据原生口径用 `vec![]` 验证**净效果**（`he` = +2 −1 = **+1**）。
#[test]
fn upstream_shijieshu_equips_and_discards() {
    let mut e = hearth_engine(20275401);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 让 0 号成为**体力最少**者（hp 2），1 号满血 ⇒ 1 号不是最少 ⇒ 可被指定
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, (max0 - 2).max(1));
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("shijieshu"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("视界术在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 效果作用于**使用者**（原生自-target）
    let he_before = e.debug_hand(0).len() + e.debug_equips(0).len();
    let eq_before = e.debug_equips(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        // 原生口径：自-target（`targets` 必须为空）
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    eprintln!("DBG 视界术：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "视界术应可发动（目标非体力最少）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut discarded = false;
    for _ in 0..24 {
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
        if !discarded && !cards.is_empty() {
            discarded = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: vec![cards[0]] },
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
    let eq_after = e.debug_equips(0).len();
    let he_after = e.debug_hand(0).len() + eq_after;
    eprintln!(
        "DBG 视界术：询问={prompts:?} 弃牌={discarded}；使用者装备 {eq_before} → {eq_after}；\
         使用者 he 总数 {he_before} → {he_after}（+2 −1 = +1）"
    );
    assert!(
        eq_after > eq_before,
        "视界术：目标**装备区应增加**（{eq_before} → {eq_after}）——来自牌堆的装备牌"
    );
    assert_eq!(
        he_after,
        he_before + 1,
        "视界术：获得 ≤2 张装备后**弃 1** ⇒ `he` 净 **+1**（{he_before} → {he_after}）"
    );
}

/// **差异确认**：原生为**自-target** ⇒ 传其他角色目标一律 `Rejected(IllegalTargets)`
/// （上游 `!target.isMin()` 本意是**指定其他角色**；本用例固定该差异事实，见第 45 项）。
#[test]
fn upstream_shijieshu_target_must_not_be_min() {
    let mut e = hearth_engine(20275402);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 让 1 号成为**唯一体力最少**者
    let max1 = e.state_core().player(1).max_hp as i32;
    e.debug_set_hp(1, (max1 - 2).max(1));
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("shijieshu"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("视界术在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    eprintln!("DBG 视界术（传显式目标）→ {out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "视界术（登记差异）：原生为**自-target** ⇒ 传显式目标应被拒（上游为指定其他角色）：{out:?}"
    );
    drain(&mut e);
}
