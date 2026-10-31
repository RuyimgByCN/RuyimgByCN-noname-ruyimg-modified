//! NC06 内容私有状态：**通用内容状态面**（`vm_aux`）的同源验收 + 核心专用字段普查。
//!
//! 背景（本轮普查，见证据 README）：`GameStateCore` 的核心状态里累计了约
//! **140 个内容专用类型字段**（`TurnState` 60 项中约 55 项、`PlayerState` 104 项
//! 中约 85 项按技能命名，如 `mufeng_used`/`shenmiguo_used`/`baoxue_used`），
//! 与 NC06「普通内容不得继续要求扩张核心专用字段」直接冲突。
//!
//! 通用替代面是 `vm_aux`（marks/flags/once/piles/hits/lost/drawn）。本文件验收
//! 这些通用形状满足**与类型字段同等的持久化契约**：参与摘要、随归档往返、
//! 恢复期校验通过——即「把内容状态放进通用面」不会失去任何既有保证。

use engine_native::{Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn roundtrip(e: &Engine) -> Engine {
    let bytes = engine_native::archive::encode_archive(&e.save());
    let snapshot = engine_native::archive::decode_archive(&bytes).expect("解码归档");
    Engine::restore(&snapshot).expect("恢复")
}

/// 通用形状：改动 → 摘要变化（hashed）+ 归档往返（encoded）+ 恢复后一致。
fn assert_shape_is_same_sourced(
    label: &str,
    mutate: impl Fn(&mut Engine),
    observe: impl Fn(&Engine) -> String,
) {
    let mut e = Engine::new(20261101, 4);
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
    let before = e.state_hash();
    let before_obs = observe(&e);
    mutate(&mut e);
    let after_obs = observe(&e);
    assert_ne!(before_obs, after_obs, "[{label}] 探针未改变被观测值");
    assert_ne!(e.state_hash(), before, "[{label}] 未参与状态摘要");
    let restored = roundtrip(&e);
    assert_eq!(observe(&restored), after_obs, "[{label}] 未随归档往返");
    assert_eq!(
        restored.state_hash(),
        e.state_hash(),
        "[{label}] 恢复摘要不一致"
    );
}

/// marks：层数（整数）——标量内容状态。
#[test]
fn nc06_shape_marks_is_same_sourced() {
    assert_shape_is_same_sourced(
        "marks",
        |e| {
            e.state_core_mut().vm_aux.marks.insert((1, 42), 3);
            e.state_core_mut().vm_aux.marks.insert((2, 42), -1);
        },
        |e| format!("{:?}", e.state_core().vm_aux.marks),
    );
}

/// flags：具名 u32（pending/assigned 等）——计数/旗标内容状态。
#[test]
fn nc06_shape_flags_is_same_sourced() {
    assert_shape_is_same_sourced(
        "flags",
        |e| {
            e.state_core_mut()
                .vm_aux
                .flags
                .insert((1, "nc06_pending".to_string()), 7);
        },
        |e| format!("{:?}", e.state_core().vm_aux.flags),
    );
}

/// once：范围化「已消费」集合（1 回合 / 2 轮 / 3 整局）——限次内容状态。
#[test]
fn nc06_shape_once_is_same_sourced() {
    assert_shape_is_same_sourced(
        "once",
        |e| {
            e.state_core_mut()
                .vm_aux
                .once
                .insert((1, "nc06_limit".to_string(), 3));
        },
        |e| format!("{:?}", e.state_core().vm_aux.once),
    );
}

/// piles：内容自有牌堆（牌 id 列表）——牌引用类内容状态。
#[test]
fn nc06_shape_piles_is_same_sourced() {
    assert_shape_is_same_sourced(
        "piles",
        |e| {
            // 从牌堆取两张真实牌 id（保持牌区唯一性：移出牌堆）
            let mut taken = Vec::new();
            for _ in 0..2 {
                let card = *e.state_core().deck.first().expect("牌堆非空");
                e.state_core_mut().deck.remove(0);
                taken.push(card);
            }
            e.state_core_mut()
                .vm_aux
                .piles
                .insert((1, "nc06_store".to_string()), taken);
        },
        |e| format!("{:?}", e.state_core().vm_aux.piles),
    );
}

/// hits：本回合伤害触发计数——回合内计数内容状态。
#[test]
fn nc06_shape_hits_is_same_sourced() {
    assert_shape_is_same_sourced(
        "hits",
        |e| {
            e.state_core_mut()
                .vm_aux
                .hits
                .insert((1, "nc06_first_damage".to_string()), 2);
        },
        |e| format!("{:?}", e.state_core().vm_aux.hits),
    );
}

/// 换回合清空语义：通用面的回合范围数据随换回合清理（与类型字段的
/// `clear_turn` 语义一致，避免「通用面不清、类型字段清」的双口径）。
#[test]
fn nc06_generic_turn_scoped_state_clears_on_turn_change() {
    let mut e = Engine::new(20261102, 4);
    e.state_core_mut().vm_aux.marks.insert((1, 7), 2); // 整局范围（不清）
    e.state_core_mut().vm_aux.flags.insert((1, "f".into()), 1); // 回合范围（清）
    e.state_core_mut().vm_aux.hits.insert((1, "h".into()), 3); // 回合范围（清）
    e.state_core_mut().vm_aux.once.insert((1, "o".into(), 1)); // 回合范围（清）
    e.state_core_mut().vm_aux.once.insert((1, "o".into(), 3)); // 整局范围（不清）

    // 推进到下一回合（换回合触发 clear_turn）；途中应答开放选择
    for _ in 0..80 {
        if e.turn().round > 1 {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel: Vec<engine_native::ChoiceOption> =
                if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                    vec![engine_native::ChoiceOption::Pass]
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
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
    }
    assert!(e.turn().round > 1, "应已进入第 2 回合");
    let aux = &e.state_core().vm_aux;
    assert!(
        aux.marks.contains_key(&(1, 7)),
        "整局范围标记不随换回合清理"
    );
    assert!(
        aux.once.contains(&(1, "o".into(), 3)),
        "整局范围 once 不随换回合清理"
    );
    assert!(
        !aux.once.contains(&(1, "o".into(), 1)),
        "回合范围 once 随换回合清理"
    );
}
