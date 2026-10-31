//! NC07 状态/编码/hash/引用校验**同源**审计（可复跑）。
//!
//! 方案 §NC07 要求状态、编码、摘要与恢复期校验出自同一处定义：任何进入状态
//! 的字段都应（a）参与状态摘要、（b）随归档往返、（c）在恢复期接受引用校验。
//!
//! 本文件对**代表性状态面**逐项验收三件事：
//! 1. **hashed**：改动该字段 → `state_hash()` 变化（未参与摘要的字段是漂移源）；
//! 2. **encoded**：改动该字段 → 归档 encode/decode + 恢复后仍为改动值；
//! 3. **validated**：篡改解码后的快照（越界引用）→ `Engine::restore` 明确拒绝。
//!
//! 覆盖面：体力/上限、座位环、回合与阶段、身份分布、技能实例（授予/启用）、
//! 装备区与手牌区、判定区、翻面、护驾/集驾计数、Boss 关卡档案。
//!
//! 未覆盖（登记待办）：`vm_aux` 内部各表（once/hits/marks）逐项、每张扩展字段的
//! 引用校验、跨版本迁移路径——见证据 README。

use engine_native::{Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

/// 归档往返（无扩展）后恢复。
fn roundtrip(e: &Engine) -> Engine {
    let bytes = engine_native::archive::encode_archive(&e.save());
    let snapshot = engine_native::archive::decode_archive(&bytes).expect("解码归档");
    Engine::restore(&snapshot).expect("恢复")
}

/// 断言：改动参与摘要、随归档往返、恢复后保持。
fn assert_field_is_same_sourced(
    label: &str,
    mutate: impl Fn(&mut Engine),
    observe: impl Fn(&Engine) -> String,
) {
    let mut e = Engine::new(20261091, 4);
    // 推进到出牌阶段，保证多数字段可写
    for _ in 0..10 {
        if e.turn().current == 0 && e.turn().phase == engine_native::Phase::Play {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
    let before_hash = e.state_hash();
    let before_obs = observe(&e);
    mutate(&mut e);
    let after_obs = observe(&e);
    assert_ne!(before_obs, after_obs, "[{label}] 探针未改变被观测值");
    assert_ne!(
        e.state_hash(),
        before_hash,
        "[{label}] 字段未参与状态摘要（hash 未变）"
    );
    let restored = roundtrip(&e);
    assert_eq!(
        observe(&restored),
        after_obs,
        "[{label}] 字段未随归档往返（encode/decode 丢字段）"
    );
    assert_eq!(
        restored.state_hash(),
        e.state_hash(),
        "[{label}] 恢复后摘要不一致"
    );
}

/// 体力：参与摘要、随归档往返。
#[test]
fn nc07_hp_is_same_sourced() {
    assert_field_is_same_sourced(
        "hp",
        |e| e.debug_set_hp(1, 1),
        |e| format!("{}", e.debug_hp(1)),
    );
}

/// 体力上限：参与摘要、随归档往返。
#[test]
fn nc07_max_hp_is_same_sourced() {
    assert_field_is_same_sourced(
        "max_hp",
        |e| e.debug_set_max_hp(1, 7),
        |e| format!("{}", e.state_core().player(1).max_hp),
    );
}

/// 座位环（换位）：参与摘要、随归档往返、顺座次序随之变化。
#[test]
fn nc07_seat_ring_is_same_sourced() {
    assert_field_is_same_sourced(
        "seat_ring",
        |e| e.state_core_mut().swap_seat(1, 3),
        |e| format!("{:?}", e.state_core().seat_ring),
    );
}

/// 回合与阶段：参与摘要、随归档往返。
#[test]
fn nc07_turn_state_is_same_sourced() {
    assert_field_is_same_sourced(
        "turn",
        |e| e.debug_set_turn(2, engine_native::Phase::Draw, 5),
        |e| {
            format!(
                "{}:{:?}:{}",
                e.turn().current,
                e.turn().phase,
                e.turn().round
            )
        },
    );
}

/// 身份分布：参与摘要、随归档往返。
#[test]
fn nc07_identity_roles_are_same_sourced() {
    assert_field_is_same_sourced(
        "identity",
        |e| {
            e.debug_set_identity_roles(vec![
                engine_native::state::Role::Lord,
                engine_native::state::Role::Rebel,
                engine_native::state::Role::Loyalist,
                engine_native::state::Role::Traitor,
            ])
        },
        |e| format!("{:?}", e.state_core().identity_roles()),
    );
}

/// 技能实例（授予 + 启用位）：参与摘要、随归档往返。
#[test]
fn nc07_skill_instances_are_same_sourced() {
    assert_field_is_same_sourced(
        "skills",
        |e| {
            // 先按目录名授予一个技能，再翻转其启用位
            let name = e.catalog().skills[1].name.to_string();
            e.debug_grant(2, &name);
            let key = *e
                .state_core()
                .skills
                .keys()
                .find(|(owner, _)| *owner == 2)
                .expect("刚授予的实例");
            e.state_core_mut()
                .skills
                .get_mut(&key)
                .expect("实例存在")
                .enabled = false;
        },
        |e| {
            let mut flags: Vec<bool> = e
                .state_core()
                .skills
                .values()
                .map(|inst| inst.enabled)
                .collect();
            flags.sort();
            format!("{flags:?}")
        },
    );
}

/// 装备区：参与摘要、随归档往返。
#[test]
fn nc07_equip_zone_is_same_sourced() {
    assert_field_is_same_sourced(
        "equips",
        |e| {
            // 装备区直接写入：牌取自实际牌堆（deck 中的首张），避免目录名依赖
            let card = *e.state_core().deck.first().expect("牌堆非空");
            e.state_core_mut().deck.remove(0);
            e.state_core_mut().player_mut(1).equips.push(card);
        },
        |e| format!("{:?}", e.debug_equips(1)),
    );
}

/// 判定区：参与摘要、随归档往返。
#[test]
fn nc07_judge_zone_is_same_sourced() {
    assert_field_is_same_sourced(
        "judge",
        |e| {
            let card = *e.state_core().deck.first().expect("牌堆非空");
            e.state_core_mut().deck.remove(0);
            e.state_core_mut().player_mut(1).judge.push(card);
        },
        |e| format!("{:?}", e.debug_judge(1)),
    );
}

/// 翻面（face_down）：参与摘要、随归档往返。
#[test]
fn nc07_face_down_is_same_sourced() {
    assert_field_is_same_sourced(
        "face_down",
        |e| e.debug_set_face_down(1, true),
        |e| format!("{}", e.debug_face_down(1)),
    );
}

/// 护驾/集驾计数：参与摘要、随归档往返。
#[test]
fn nc07_counters_are_same_sourced() {
    assert_field_is_same_sourced(
        "hujia",
        |e| e.debug_set_hujia(1, 2),
        |e| format!("{}", e.state_core().player(1).hujia),
    );
}

/// Boss 关卡档案：参与摘要、随归档往返。
#[test]
fn nc07_boss_stage_is_same_sourced() {
    let mut e = Engine::new_boss_stage(20261092, 4, 0, 0, 4, 4, 0);
    let before_hash = e.state_hash();
    e.debug_set_boss_progress(4, true, true);
    assert_ne!(e.state_hash(), before_hash, "Boss 关卡档案未参与摘要");
    let bytes = engine_native::archive::encode_archive(&e.save());
    let snapshot = engine_native::archive::decode_archive(&bytes).expect("解码");
    let restored = Engine::restore(&snapshot).expect("恢复");
    assert_eq!(
        restored.boss_stage_progress(),
        Some((4, true, true)),
        "Boss 关卡档案未随归档往返"
    );
}

/// 恢复期引用校验：越界引用必须被拒绝（同源校验而非静默接受）。
#[test]
fn nc07_restore_rejects_out_of_range_references() {
    let e = Engine::new(20261093, 4);
    let bytes = engine_native::archive::encode_archive(&e.save());

    // ① 座位环写入越界玩家
    let snapshot = engine_native::archive::decode_archive(&bytes).expect("解码");
    let mut tampered = snapshot.clone();
    tampered.core.seat_ring[0] = 99;
    assert!(
        Engine::restore(&tampered).is_err(),
        "座位环越界引用应被恢复校验拒绝"
    );

    // ② 武将槽越界（目录武将数之外）
    let mut tampered2 = snapshot.clone();
    tampered2.core.players[0].general_slots[0] = Some(u16::MAX);
    assert!(
        Engine::restore(&tampered2).is_err(),
        "武将槽越界应被恢复校验拒绝"
    );

    // ③ 技能定义越界
    let mut tampered3 = snapshot.clone();
    // 快照须含技能实例：先授予一个目录技能再取快照
    let mut seeded = Engine::new(20261093, 4);
    let skill_name = seeded.catalog().skills[1].name.to_string();
    seeded.debug_grant(0, &skill_name);
    let seeded_bytes = engine_native::archive::encode_archive(&seeded.save());
    let seeded_snapshot = engine_native::archive::decode_archive(&seeded_bytes).expect("解码");
    let existing = seeded_snapshot
        .core
        .skills
        .values()
        .next()
        .cloned()
        .expect("快照含技能实例（已授予）");
    tampered3.core.skills.insert((0, u16::MAX), existing);
    assert!(
        Engine::restore(&tampered3).is_err(),
        "技能定义越界应被恢复校验拒绝"
    );

    // ④ 正常快照仍可恢复（校验不是恒拒）
    assert!(Engine::restore(&snapshot).is_ok(), "原快照应可恢复");
}
