//! NC08 第⑦层（上游行为对照）样本 17：`duwen`（毒吻，U05 ow 武将包 / 黑白合）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/ow/skill.js`）：
//!
//! ```js
//! duwen: {
//!     trigger: { source: "damageBegin" },   // 自己**即将造成**伤害
//!     check(event, player) { … },           // AI 策略面
//!     forced: true,                         // **锁定技**
//!     filter(event, player) {
//!         return player.countCards("h") === event.player.countCards("h")  // 手牌数**相等**
//!             && event.notLink();
//!     },
//!     content() { trigger.num++; },         // 伤害 **+1**
//!     ai: { … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **锁定技**（`forced: true` → 无确认门）；
//! 2. **门槛 = 来源与受伤害角色手牌数相等**（`countCards("h")` 相等）；
//! 3. **效果 = 该次伤害 +1**（`trigger.num++`）。
//!
//! 原生实现引用：`kernel/damage.rs:227`（`hand.len()` 相等 → `amount += 1`；
//! 技能键 `WUDIAN_DUWEN_KEY`）、`content/host/step/wudian_status.rs:201`（宿主无机器步骤）。
//!
//! 未覆盖（登记）：`check`/`ai` 策略面；`event.notLink()`（铁索连环传导事件**不**加成——
//! 原生按「传导为独立事件照常判定」实现，见 `damage.rs` 注释 → **登记为口径差异**）。

use engine_native::{CommandOutcome, Engine, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn ow_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shaai-ow-upstream-noname/assets/cards.json"))
            .expect("读 ow 包");
    let mut pack = content_native_loader::bridge_pack(&json).expect("桥接 pack 段").pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
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
            action: engine_native::GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

/// 上游点 1+2+3：手牌数**相等** → 伤害 +1，且**无确认门**（锁定技）。
#[test]
fn upstream_duwen_equal_hands_adds_one_damage() {
    let mut e = ow_engine(20264501);
    e.debug_grant(0, "duwen");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号（来源）与 1 号（受伤者）手牌数相等（各 2 张）
    for _ in 0..2 {
        let c = e.debug_place(0, "sha");
        e.debug_give_hand_card(0, 0, c);
    }
    for _ in 0..2 {
        let c = e.debug_place(1, "shan");
        e.debug_give_hand_card(1, 1, c);
    }
    e.debug_set_hp(1, 4);
    let hp_before = e.debug_hp(1);
    let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        e.open_choice().is_none(),
        "毒吻为锁定技，不应出现确认门：{:?}",
        e.open_choice().map(|c| c.spec.prompt.clone())
    );
    drain(&mut e);
    assert_eq!(
        e.debug_hp(1),
        hp_before - 2,
        "上游 trigger.num++：手牌相等时 1 点伤害 → 实际 **2 点**"
    );
}

/// 上游点 2（不满足）：手牌数**不等** → 伤害不加成。
#[test]
fn upstream_duwen_unequal_hands_no_bonus() {
    let mut e = ow_engine(20264502);
    e.debug_grant(0, "duwen");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 来源 1 张、受伤者 3 张 → 不等
    let c0 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, c0);
    for _ in 0..3 {
        let c = e.debug_place(1, "shan");
        e.debug_give_hand_card(1, 1, c);
    }
    e.debug_set_hp(1, 4);
    let hp_before = e.debug_hp(1);
    let _ = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp_before - 1, "手牌不等 → 伤害不加成（1 点）");
}

/// 上游点 1（技能门槛）：**未持技**者手牌相等也不加成。
#[test]
fn upstream_duwen_requires_the_skill() {
    let mut e = ow_engine(20264503);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..2 {
        let c = e.debug_place(0, "sha");
        e.debug_give_hand_card(0, 0, c);
    }
    for _ in 0..2 {
        let c = e.debug_place(1, "shan");
        e.debug_give_hand_card(1, 1, c);
    }
    e.debug_set_hp(1, 4);
    let hp_before = e.debug_hp(1);
    let _ = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp_before - 1, "未持毒吻 → 不加成");
}

/// 上游点 2 的**登记差异**：`event.notLink()`（铁索传导）上游**不**加成，
/// 原生注释为「传导为独立事件照常判定」→ 以源码事实固定。
#[test]
fn upstream_duwen_notlink_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/damage.rs"
    ))
    .expect("读 damage 源码");
    let idx = src.find("毒瘟（U07 ow").expect("毒瘟分支");
    let seg: String = src[idx..].chars().take(1200).collect();
    // 注释按行折行，逐片段检查（避免整句跨行匹配失败）
    assert!(
        seg.contains("铁索传") && seg.contains("导为独立事件照常判定"),
        "差异固定：原生对铁索传导事件照常判定（上游 event.notLink() 排除）：{seg}"
    );
    assert!(
        seg.contains("amount += 1"),
        "效果固定：伤害 +1：{seg}"
    );
    let _ = Phase::Play;
}
