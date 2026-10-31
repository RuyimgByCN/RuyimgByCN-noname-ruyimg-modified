//! NC08 第⑦层（上游行为对照）样本 15：`zhiyuan`（支援，U05 ow 武将包 / 安娜）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/ow/skill.js`）：
//!
//! ```js
//! zhiyuan: {
//!     trigger: { source: "damageBefore" },     // 自己**即将造成**伤害
//!     check(event, player) { … },              // AI 是否发动（策略面）
//!     logTarget: "player",
//!     filter(event, player) { return event.num > 0; },   // 伤害量 > 0
//!     content() {
//!         trigger.cancel();                    // **防止**此伤害
//!         trigger.player.recover(trigger.num); // 目标回复**等量**体力
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 自己即将造成伤害**（`trigger: { source: "damageBefore" }`）；
//! 2. **门槛 = 伤害量 > 0**（`filter: event.num > 0`）；
//! 3. **可选**（需经触发确认门；`check` 为 AI 策略面）；
//! 4. **效果 = 防止此伤害 + 目标回复等量体力**（`trigger.cancel(); recover(trigger.num)`）。
//!
//! 原生实现引用：`content/host/filtering/common.rs:762`（`Damage` 且 `source == owner &&
//! !prevented && amount > 0`）、`content/host/step/wudian_recovery.rs:271`（`d.prevented = true`
//! + `recover(hurt, num)`）。
//!
//! 未覆盖（登记）：`check`/`ai` 属策略面；`logTarget` 属表现面。

use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn ow_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shaai-ow-upstream-noname/assets/cards.json"))
        .expect("读 ow 包 character.json");
    let mut pack = content_native_loader::bridge_pack(&json)
        .expect("桥接 pack 段")
        .pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
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
}

/// 上游点 3+4：**确认发动** → 伤害被防止、目标回复等量体力。
#[test]
fn upstream_zhiyuan_prevents_damage_and_recovers_equal() {
    let mut e = ow_engine(20264201);
    e.debug_grant(0, "zhiyuan");
    e.debug_set_hp(1, 2); // 受伤目标（可回复）
    let hp1 = e.debug_hp(1);
    // 自己（0 号）即将造成 2 点伤害
    let _ = e.debug_deal_damage(Some(0), 1, 2, engine_native::event::DamageNature::Normal);
    // 触发确认门：确认发动
    let c = e.open_choice().cloned().expect("应出现支援触发确认");
    assert_eq!(c.spec.prompt, "trigger_confirm", "支援为可选技，先经确认门");
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    drain(&mut e);
    assert_eq!(
        e.debug_hp(1),
        hp1 + 2,
        "上游：cancel() 防止伤害 + recover(num=2) → 目标体力 +2（净效果）"
    );
}

/// 上游点 3（不发动）：**取消**确认 → 伤害照常结算（不得回复）。
#[test]
fn upstream_zhiyuan_declined_damage_resolves_normally() {
    let mut e = ow_engine(20264202);
    e.debug_grant(0, "zhiyuan");
    e.debug_set_hp(1, 3);
    let hp1 = e.debug_hp(1);
    let _ = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().cloned().expect("应出现支援触发确认");
    // 取消（空选择）
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: Vec::new(),
        },
    });
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "不发动 → 伤害照常（体力 -1）");
}

/// 上游点 1+2：**非自己造成**的伤害不触发（来源门槛）；伤害量为 0 不触发。
#[test]
fn upstream_zhiyuan_requires_self_as_source() {
    let mut e = ow_engine(20264203);
    e.debug_grant(0, "zhiyuan");
    e.debug_set_hp(1, 2);
    // 由 2 号造成伤害（0 号不是来源）→ 不应出现支援确认
    let _ = e.debug_deal_damage(Some(2), 1, 1, engine_native::event::DamageNature::Normal);
    let open = e.open_choice().cloned();
    assert!(
        open.is_none(),
        "非自己造成的伤害不得触发支援（上游 trigger.source=damageBefore）：{:?}",
        open.map(|c| c.spec.prompt)
    );
}
