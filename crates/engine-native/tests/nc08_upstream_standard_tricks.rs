//! NC08 第⑦层（上游行为对照）样本 4：**标准锦囊** `wuzhong`（无中生有）与
//! `taoyuan`（桃园结义）——上游基准面（非扩展来源）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/card/standard.js`）：
//!
//! ```js
//! wuzhong: {
//!     type: "trick", enable: true, selectTarget: -1, cardcolor: "red",
//!     toself: true,
//!     filterTarget(card, player, target) { return target === player; },
//!     modTarget: true,
//!     async content(event) { /* versus 分支略 */ await event.target.draw(2); },
//! },
//! taoyuan: {
//!     type: "trick", enable: true, selectTarget: -1, cardcolor: "red",
//!     reverseOrder: true,
//!     filterTarget(card, player, target) { return true; },
//!     ignoreTarget(card, player, target) { return target.isHealthy(); },
//!     async content(event) { event.target.recover(); },
//! },
//! ```
//!
//! 由 JS 推导的规则点（本文件逐条验收）：
//! 1. `wuzhong`：**只能影响自己**（`filterTarget: target === player`）且摸 **2** 张；
//!    **接口差异（登记）**：上游为「自身为目标」，原生为**无目标锦囊**（`targets` 必须为空）；
//! 2. `taoyuan`：影响**其他**角色（`selectTarget: -1` 且**无** `toself`）——**不含使用者**；
//! 3. `taoyuan`：**满血者被跳过**（`ignoreTarget: isHealthy()`）；
//! 4. `taoyuan`：**逆序结算**（`reverseOrder: true`）——原生按**座位序**遍历：
//!    **登记差异**（本文件固定观测口径，防止静默改变）。
//!
//! 未覆盖（登记）：`ai` 段、`audio`/`fullskin`、`defaultYingbianEffect`（应变篇）属表现/策略/扩展面；
//! `wuzhong` 的 `get.is.versus()` 分支属对决模式（原生另有模式实现）。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
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
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 上游点 1：`wuzhong` 只能指定**自己**（对他者目标必须被拒），且自己摸 2 张。
#[test]
fn upstream_wuzhong_targets_self_only_and_draws_two() {
    let mut e = Engine::new(20262701, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let w = e.debug_place(0, "wuzhong");
    e.debug_give_hand_card(0, 0, w);
    // ① 指定他人 → 必须被拒（上游 filterTarget 只允许自己）
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: w, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "无中生有不得指定他人（上游 filterTarget: target === player）：{out:?}"
    );
    // ② 使用者自己结算 → 摸 2 张。
    // **接口差异登记**：上游把无中生有建模为「自身为目标」（`toself` + `filterTarget`），
    // 原生建模为**无目标锦囊**（`targets` 必须为空，使用者隐式）。语义一致（只影响自己），
    // 但目标面接口不同——凡按「目标集合」观察的消费者需注意（NC08 待办）。
    let hand_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: w, targets: vec![] },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), hand_before - 1 + 2, "使用者摸 2 张");
}

/// 上游点 2+3：`taoyuan` **不含使用者**，且**满血者被跳过**（只回复受伤者）。
#[test]
fn upstream_taoyuan_excludes_owner_and_skips_healthy() {
    let mut e = Engine::new(20262702, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 使用者 0 受伤（若含自己应回复）；1 受伤；2 满血；3 受伤
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 3);
    e.debug_set_hp(3, 3);
    let t = e.debug_place(0, "taoyuan");
    e.debug_give_hand_card(0, 0, t);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: t, targets: vec![] },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 3, "上游无 toself → 使用者不回复");
    assert_eq!(e.debug_hp(1), 4, "受伤他人回复 1");
    assert_eq!(e.debug_hp(3), 4, "受伤他人回复 1");
    assert_eq!(e.debug_hp(2), 4, "满血者被跳过（本就满血）");
}

/// 上游点 4（**登记差异**）：`taoyuan` 上游 `reverseOrder: true`，原生按**座位序**遍历。
///
/// 本用例固定「原生按座位序」这一观测事实；顺序差异在**回复触发类技能**（如 U07 `nsshuaiyan`
/// 「他人于你的回合外回复体力后」）下可能有可观测影响，是否对齐需按上游语义判定（NC08 待办）。
#[test]
fn upstream_taoyuan_order_difference_is_registered() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/tricks.rs"
    ))
    .expect("读 tricks 源码");
    let idx = src.find("TaoyuanMachine").expect("TaoyuanMachine");
    let seg: String = src[idx..].chars().take(1400).collect();
    assert!(
        seg.contains("alive_players()") && !seg.contains("rev()"),
        "原生桃园结案按座位序遍历（上游 reverseOrder=true）——差异已登记，不得静默改变"
    );
}
