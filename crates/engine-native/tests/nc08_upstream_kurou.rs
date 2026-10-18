//! NC08 第⑦层（上游行为对照）：**以归档上游 JS 为准**核对原生实现。
//!
//! 上游源码（`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js`，
//! `config.zhijiechifa` 分支）：
//!
//! ```js
//! lib.skill.kurou = {
//!     audio: 2,
//!     enable: "phaseUse",       // 出牌阶段可用
//!     direct: true,             // 无目标直接生效
//!     content: function () {
//!         player.logSkill('kurou');
//!         player.loseHp(1);     // **失去** 1 点体力（非伤害）
//!         player.draw(2);       // 摸 2 张
//!     },
//!     ai: { ... },              // AI 估值，非规则
//! };
//! ```
//!
//! 由该 JS 可直接推导的规则点（本文件逐条验收）：
//! 1. **无 `usable`/`round`/`turn` 限制** → 同一出牌阶段**可多次发动**（上游未限次）；
//! 2. 效果是 `loseHp`（失去体力）而非 `damage`（伤害）→ **不触发受伤类/伤害类技能**；
//! 3. **无 `filter`** → 体力为 1 时仍可发动（`loseHp` 可致濒死）；
//! 4. 摸 2 张。
//!
//! 未覆盖（登记）：`audio`/`ai` 属表现与策略面，不属规则对照。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn ms_pack() -> engine_native::content::extension::ExtensionPack {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("mark-supplement-upstream-liuli/assets/cards.json"))
        .expect("读 L20 标记补充包");
    content_native_loader::bridge_pack(&json).expect("桥接").pack
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
    for _ in 0..30 {
        if e.turn().current == seat && e.turn().phase == engine_native::Phase::Play {
            return;
        }
        if let Some(c) = e.open_choice().cloned() {
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
            continue;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
    panic!("未到 {seat} 号出牌阶段");
}

fn engine(seed: u64) -> Engine {
    let pack = ms_pack();
    let mut e = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "kurou");
    advance_to_play(&mut e, 0);
    e
}

/// 上游点 1：同一出牌阶段可**多次**发动（上游无 `usable` 限制）。
#[test]
fn upstream_kurou_is_not_limited_per_phase() {
    let mut e = engine(20262201);
    let def = e.catalog().skill_by_name("kurou").expect("kurou 在目录");
    let hp0 = e.debug_hp(0);
    let hand0 = e.debug_hand(0).len();
    for i in 0..2u64 {
        let out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::ActivateSkill { skill: def },
        });
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "第 {} 次发动应被接受（上游未限次）：{out:?}",
            i + 1
        );
        drain(&mut e);
    }
    assert_eq!(e.debug_hp(0), hp0 - 2, "两次发动各失去 1 点体力");
    assert_eq!(e.debug_hand(0).len(), hand0 + 4, "两次发动各摸 2 张");
}

/// 上游点 2：`loseHp` 是**失去体力**而非伤害——不触发伤害/受伤类事件。
#[test]
fn upstream_kurou_loses_hp_not_damage() {
    let mut e = engine(20262202);
    let def = e.catalog().skill_by_name("kurou").expect("kurou");
    let before = e.log().len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e);
    let tail: Vec<String> = e
        .log()
        .iter()
        .skip(before)
        .map(|ev| format!("{ev:?}"))
        .collect();
    let joined = tail.join("\n");
    assert!(
        joined.contains("HpLost"),
        "应产生失去体力事件（`HpLost`）：\n{joined}"
    );
    assert!(
        !joined.contains("DamageDealt") && !joined.contains("Damage"),
        "loseHp 不得产生伤害事件（上游为失去体力）：\n{joined}"
    );
}

/// 上游点 3+4 + **顺序差异登记**：体力为 1 时仍可发动（上游无 `filter`）。
///
/// 顺序差异（实测，登记）：上游 `content` 的书写顺序是 `loseHp(1); draw(2);`，
/// 原生实现的日志顺序是 `CardsDrawn` → `HpLost`（先摸后失去）。该差异在「失去体力后
/// 进入濒死」的场景下有语义影响（摸到的【桃】是否已入手参与求援），故在此**显式记录**
/// 而非隐去；是否对齐需按采用的上游语义判定（NC08 待办）。
#[test]
fn upstream_kurou_has_no_hp_filter() {
    let mut e = engine(20262203);
    e.debug_set_hp(0, 1);
    let def = e.catalog().skill_by_name("kurou").expect("kurou");
    let hand0 = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "体力 1 时上游无 filter，应可发动：{out:?}"
    );
    drain(&mut e);
    // 失去 1 点后进入濒死；摸牌已在效果内结算（濒死结算可能随后介入）
    assert!(
        e.debug_hp(0) <= 0 || !e.debug_alive(0),
        "体力 1 发动后应进入濒死或已阵亡（实为 hp={}）",
        e.debug_hp(0)
    );
    let _ = hand0;
}

/// 顺序差异登记（可复跑）：原生日志顺序为「先摸牌、后失去体力」。
///
/// 上游书写顺序为 `loseHp(1); draw(2);`（先失去）。本用例把**观测到的顺序**固定下来，
/// 使差异不会被静默改变；对齐与否由 NC08 的上游判定决定。
#[test]
fn upstream_kurou_effect_order_is_recorded() {
    let mut e = engine(20262204);
    let def = e.catalog().skill_by_name("kurou").expect("kurou");
    let before = e.log().len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e);
    let names: Vec<&'static str> = e
        .log()
        .iter()
        .skip(before)
        .filter_map(|ev| match ev {
            engine_native::LogEvent::CardsDrawn { .. } => Some("draw"),
            engine_native::LogEvent::HpLost { .. } => Some("lose_hp"),
            _ => None,
        })
        .collect();
    assert_eq!(
        names,
        vec!["draw", "lose_hp"],
        "原生顺序固定为「先摸后失去」；上游书写顺序为「先失去后摸」——差异已登记（NC08 待办）"
    );
}
