//! NC08 第⑦层（上游行为对照）样本 18：`ziyu`（自愈，U05 ow 武将包 / 禅雅塔）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/ow/skill.js`）：
//!
//! ```js
//! ziyu: {
//!     trigger: { global: "phaseEnd" },     // 任意角色的**阶段结束**
//!     direct: true,                        // 直接发动（进入选择，无需确认）
//!     filter(event, player) {
//!         return game.phaseNumber % 4 === 0;   // **每 4 个阶段**一次
//!     },
//!     content() {
//!         player.chooseDrawRecover(get.prompt("ziyu")).logSkill = "ziyu";
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 任意角色的阶段结束**（`global: phaseEnd`）；
//! 2. **门槛 = 阶段计数为 4 的倍数**（`phaseNumber % 4 === 0`）；
//! 3. **效果 = 二选一：摸 1 张 或 回复 1 点体力**（`chooseDrawRecover`）；
//! 4. **满血时无回复选项**（上游 `chooseDrawRecover` 同口径）。
//!
//! 原生实现引用：`content/host/filtering/common.rs:591`（`PhaseEnd` 且阶段计数条件）、
//! `content/host/step/wudian_recovery.rs:15`（`Tag(0)`=摸 / `Tag(1)`=回复，满血不提供 `Tag(1)`；
//! 可取消）。
//!
//! 未覆盖（登记）：`direct`/`logSkill` 属表现面；阶段计数的具体起点（原生用自身 turn 计数）。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn ow_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shaai-ow-upstream-noname/assets/cards.json"))
        .expect("读 ow 包");
    let mut pack = content_native_loader::bridge_pack(&json)
        .expect("桥接 pack 段")
        .pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 逐步推进直到出现 `ziyu` 询问；返回该询问（不回答）。
fn advance_until_ziyu(e: &mut Engine) -> Option<engine_native::OpenChoice> {
    let mut seen: Vec<String> = Vec::new();
    for i in 0..200 {
        if let Some(c) = e.open_choice().cloned() {
            if !seen.iter().any(|p| p == c.spec.prompt) {
                seen.push(c.spec.prompt.to_string());
            }
            if c.spec.prompt == "wudian_ziyu_choice" {
                return Some(c);
            }
            // 触发确认门一律**确认**（自愈为可选触发；取消它就不会发动——修罗样本的教训）
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
            continue;
        }
        if i == 30 {
            eprintln!(
                "DBG i=30 phase_number={} turn={} phase={:?} seen={:?}",
                e.state_core().turn.phase_number,
                e.turn().current,
                e.turn().phase,
                seen
            );
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
    eprintln!(
        "DBG 结束 phase_number={} seen={:?}",
        e.state_core().turn.phase_number,
        seen
    );
    None
}

/// 上游点 1+2：**阶段计数为 4 的倍数**时出现自愈询问（含「摸」选项）。
#[test]
fn upstream_ziyu_opens_every_fourth_phase() {
    let mut e = ow_engine(20264601);
    e.debug_grant(0, "ziyu");
    let c = advance_until_ziyu(&mut e).expect("应出现自愈询问（每 4 个阶段一次）");
    assert!(
        c.spec.options.contains(&ChoiceOption::Tag(0)),
        "应含「摸一张」选项：{:?}",
        c.spec.options
    );
    assert!(c.spec.allow_cancel, "上游 chooseDrawRecover 可取消");
}

/// 上游点 3+4：**已受伤** → 含「回复」选项；选回复 → 体力 +1。
#[test]
fn upstream_ziyu_damaged_offers_recover_and_applies() {
    let mut e = ow_engine(20264602);
    e.debug_grant(0, "ziyu");
    e.debug_set_hp(0, 2); // 受伤 → 应提供回复选项
    let c = advance_until_ziyu(&mut e).expect("应出现自愈询问");
    assert!(
        c.spec.options.contains(&ChoiceOption::Tag(1)),
        "已受伤应含「回复 1 点体力」选项：{:?}",
        c.spec.options
    );
    let hp_before = e.debug_hp(0);
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    });
    assert_eq!(e.debug_hp(0), hp_before + 1, "选回复 → 体力 +1");
}

/// 上游点 4：**满血**时**不提供**回复选项（上游 chooseDrawRecover 同口径）。
#[test]
fn upstream_ziyu_full_hp_has_no_recover_option() {
    let mut e = ow_engine(20264603);
    e.debug_grant(0, "ziyu");
    e.debug_set_hp(0, 4); // 满血（上限 4）
    let c = advance_until_ziyu(&mut e).expect("应出现自愈询问");
    assert!(
        !c.spec.options.contains(&ChoiceOption::Tag(1)),
        "满血不应提供回复选项：{:?}",
        c.spec.options
    );
    assert!(
        c.spec.options.contains(&ChoiceOption::Tag(0)),
        "仍应提供「摸一张」：{:?}",
        c.spec.options
    );
}

/// 上游点 3（摸支）：选「摸」→ 手牌 +1。
#[test]
fn upstream_ziyu_draw_option_draws_one() {
    let mut e = ow_engine(20264604);
    e.debug_grant(0, "ziyu");
    let c = advance_until_ziyu(&mut e).expect("应出现自愈询问");
    let hand_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hand(0).len(), hand_before + 1, "选「摸」→ 手牌 +1");
}
