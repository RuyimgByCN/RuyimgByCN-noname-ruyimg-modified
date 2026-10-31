//! NC08 第⑦层（上游行为对照）样本 96：**来源 U05（古剑）** 基本牌
//! `dinvxuanshuang`（帝女玄霜）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`dinvxuanshuang`）：
//!
//! ```js
//! dinvxuanshuang: {
//!     type: "basic", savable: true, selectTarget: -1,     // **可作濒死救援牌**
//!     content() {
//!         "step 0"; target.recover();                     // **回复 1 点体力**
//!         "step 1";
//!         if (target.isIn()) target.chooseToDiscard([1, Infinity], "he", "弃置任意张牌并摸等量的牌");
//!         else event.finish();
//!         "step 2";
//!         if (result.bool) target.draw(result.cards.length);   // **弃 N 摸 N**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **性质**：`basic` + **`savable`**（可作**濒死救援**牌）；
//! 2. **效果**：目标**回复 1 点体力**；随后**可弃任意张 `he` 牌并摸等量**（可取消）。
//!
//! 原生实现引用：宿主行为（古剑基本牌机）—— 本样本以**行为面**观测「回复」这一即时效果。
//!
//! **本样本的定位**：验证**回复效果**是否执行（对照：同批食物牌「本体归属」缺失，
//! 但那是 `$gain2` 面；本卡的回复属另一机制）。
//!
//! 未覆盖（登记）：**`savable`（濒死救援）**的使用路径；弃 N 摸 N 的数值面；`ai` 策略面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
            .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：**全部角色**（`selectTarget: -1`）各回复 1 点体力。
///
/// **本轮未验收（差异登记）**：实测——发动 `Accepted`、**使用者 hp 3 → 4**（回复 ✅），
/// 但**其他角色 1 号 hp 3 → 3（未回复）** ✗ ⇒ 原生**只回复使用者**，而上游 `selectTarget: -1`
/// 为**全部角色** ⇒ 与「**目标集合范围**」族**同族**（第 2 例；第 1 例为隙焰——方向相反：
/// 隙焰是原生**排除**使用者，本卡是原生**只含**使用者）。详见修复清单第 39 项。
/// 用例以 `#[ignore]` 作为修复后的验收条件。
#[test]
#[ignore = "帝女玄霜：selectTarget -1 应令全部角色各回复 1——原生只回复使用者，见第 39 项"]
fn upstream_dinvxuanshuang_recovers() {
    let mut e = gujian_engine(20272601);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 使用者与 1 号**都受伤**（便于区分「仅自己回复」与「全员回复」）
    let max0 = e.state_core().player(0).max_hp as i32;
    e.debug_set_hp(0, max0 - 1);
    let max1 = e.state_core().player(1).max_hp as i32;
    e.debug_set_hp(1, max1 - 1);
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("dinvxuanshuang"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("帝女玄霜在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp0_before = e.debug_hp(0);
    let hp_before = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        // `selectTarget: -1` + `savable` → 引擎按「无显式目标」形态受理（对照 toself 卡）
        action: GameAction::PlayCard {
            card,
            targets: vec![],
        },
    });
    eprintln!("DBG 帝女玄霜：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "帝女玄霜应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 后续「弃任意张」询问：本样本**不弃**（验证回复效果即可）
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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
    let hp_after = e.debug_hp(1);
    let hp0_after = e.debug_hp(0);
    eprintln!(
        "DBG 帝女玄霜：询问={prompts:?} 使用者 hp {hp0_before} → {hp0_after}；1 号 hp {hp_before} → {hp_after}"
    );
    // 上游 `selectTarget: -1` → **全部角色**各回复 1（本样本以「使用者与 1 号是否都回复」判定）
    assert_eq!(
        hp0_after,
        hp0_before + 1,
        "帝女玄霜：使用者应回复 1（{hp0_before} → {hp0_after}）"
    );
    assert_eq!(
        hp_after,
        hp_before + 1,
        "帝女玄霜（`selectTarget: -1`）：**1 号也应回复 1**（{hp_before} → {hp_after}）"
    );
}
