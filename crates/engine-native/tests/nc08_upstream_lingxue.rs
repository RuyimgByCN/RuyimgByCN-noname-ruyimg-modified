//! NC08 第⑦层（上游行为对照）样本 16：`lingxue`（灵雪，U05 xianjian 武将包 / 星璇）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js`）：
//!
//! ```js
//! lingxue: {
//!     trigger: { player: "recoverEnd" },   // 自己回复体力**结束后**
//!     forced: true,                        // 锁定技（无需确认）
//!     content() {
//!         player.changeHujia();            // +1 点护甲
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 自己回复体力结束后**（`trigger: { player: "recoverEnd" }`）；
//! 2. **锁定技**（`forced: true` → **无确认门**）；
//! 3. **效果 = +1 点护甲**（`changeHujia()`），且**不改变体力**（回复本身由来源事件完成）。
//!
//! 原生实现引用：`content/host/filtering/xianjian.rs:407`（过滤）、
//! `content/host/step/xianjian_status.rs:6`（`ctx.change_hujia(owner, 1)`）。
//!
//! 未覆盖（登记）：`changeHujia` 的护甲上限/衰减规则由护甲子系统既有测试覆盖。

use engine_native::{Engine, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn xianjian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shaai-xianjian-upstream-noname/assets/cards.json"))
            .expect("读 xianjian 包");
    let mut pack = content_native_loader::bridge_pack(&json).expect("桥接 pack 段").pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 驱动到静止：反复推进阶段并处理选择（回复等事件经帧结算，需推进才落地）。
fn drive_to_quiescence(e: &mut Engine) {
    for _ in 0..8 {
        drain(e);
        if e.resolution().stack.is_empty() && e.open_choice().is_none() {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: engine_native::GameAction::NextPhase,
        });
    }
    drain(e);
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == engine_native::Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: engine_native::GameAction::NextPhase,
        });
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
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

/// 上游点 1+2+3：**真实回复路径**（打出一张【桃】）→ 护甲 +1，且**不出现确认门**（锁定技）。
#[test]
fn upstream_lingxue_gains_hujia_on_real_recover() {
    let mut e = xianjian_engine(20264301);
    e.debug_grant(0, "lingxue");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 2); // 已受伤，桃可用
    let tao = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, tao);
    let hp_before = e.debug_hp(0);
    let hujia_before = e.state_core().player(0).hujia;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        // 桃为自目标牌：原生按「无目标」建模（与无中生有同口径）
        action: engine_native::GameAction::PlayCard { card: tao, targets: vec![] },
    });
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "打出【桃】应被接受：{out:?}"
    );
    // 锁定技：不应出现确认门
    assert!(
        e.open_choice().is_none(),
        "灵雪为锁定技（forced），不应出现确认门：{:?}",
        e.open_choice().map(|c| c.spec.prompt.clone())
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp_before + 1, "【桃】回复本身生效（体力 +1）");
    assert_eq!(
        e.state_core().player(0).hujia,
        hujia_before + 1,
        "上游 changeHujia()：回复后 +1 护甲"
    );
}

/// 上游点 3（**源码事实固定**）：原生效果为 `change_hujia(owner, 1)`，与上游 `changeHujia()` 同口径。
#[test]
fn upstream_lingxue_effect_is_fixed_by_source() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/step/xianjian_status.rs"
    ))
    .expect("读 xianjian 状态源码");
    let idx = src.find("HostBehavior::XianjianLingxue =>").expect("灵雪分支");
    let seg: String = src[idx..].chars().take(400).collect();
    assert!(
        seg.contains("change_hujia(self.owner, 1)"),
        "灵雪效果应为 +1 护甲（上游 changeHujia 同口径）：{seg}"
    );
    // 过滤臂：自己回复且 amount > 0
    let filt = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/filtering/xianjian.rs"
    ))
    .expect("读过滤源码");
    let fi = filt.find("HostBehavior::XianjianLingxue =>").expect("灵雪过滤");
    let fseg: String = filt[fi..].chars().take(260).collect();
    assert!(
        fseg.contains("Recover") && fseg.contains("amount"),
        "灵雪过滤应绑定 Recover 事件与 amount：{fseg}"
    );
}

/// 上游点 1（不满足）：**未持技**者回复体力不获得护甲（技能门槛）。
#[test]
fn upstream_lingxue_requires_the_skill() {
    let mut e = xianjian_engine(20264302);
    // 不授予 lingxue
    e.debug_set_hp(0, 2);
    let hujia_before = e.state_core().player(0).hujia;
    e.debug_recover(0, 1);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hujia,
        hujia_before,
        "未持灵雪：回复不产生护甲"
    );
}
