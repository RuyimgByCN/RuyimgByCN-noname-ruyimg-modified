//! 恢复后继续 `autoStep`：同一栈深多台挂起机器时，应答必须交给提问者。
//!
//! 反例（2026-11-01 `park-1` 七案例 / 07-oracle-diagnostic.log）：WASM 侧
//! 恢复件在同一栈深停着「遗计」确认门（内层 `JiJi` 等自己的询问）与「机捷」
//! `JiJie`（等 `jijie_give_target` 应答）。旧 `run` 按栈深顺序取第一台，把
//! 机捷的应答交给遗计内层；机捷随后读 `ctx.answer()` 命中
//! `expect("answer present on resume")` → `RuntimeError: unreachable`
//! （`engine_native_kernel::machine::Ctx::answer`）。次级
//! “attempted to take ownership of Rust value while it was borrowed” 只是
//! panic 后句柄借用未归还的清理噪声，不是根因。
//!
//! 本测试同时走 **typed**（`NativeSession`，WASM 窄壳同一实现）与 **byte**
//! （`archive::decode_archive` + `Engine::restore`）两条路径，并要求两条
//! 路径逐步给出逐字节相同的存档。

use engine_native::ai::BoundedPlanner;
use engine_native::resolution::{Slot, NO_FRAME};
use engine_native::snap::JiJieStageSnap;
use engine_native::{
    CommandOutcome, Engine, GameAction, MachineSnapshot, ParkSnapshot, Phase, StateSnapshot,
};
use engine_native_wasm::NativeSession;

fn hex_decode(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    assert!(bytes.len().is_multiple_of(2), "十六进制长度必须为偶数");
    let nibble = |c: u8| match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        other => panic!("非十六进制字节 {other}"),
    };
    bytes
        .chunks(2)
        .map(|pair| nibble(pair[0]) * 16 + nibble(pair[1]))
        .collect()
}

fn decode(hex: &str) -> StateSnapshot {
    engine_native::archive::decode_archive(&hex_decode(hex)).expect("可解码存档")
}

fn encode_hex(engine: &Engine) -> String {
    engine_native_wasm::hex_encode(&engine_native::archive::encode_archive(&engine.save()))
}

/// 恢复契约（既有例外）：只有 `auth_generation` 轮换，其余逐字段不变。
fn assert_only_generation_rotated(before: &StateSnapshot, after: &StateSnapshot) {
    let mut expected = before.clone();
    expected.auth_generation += 1;
    assert_eq!(*after, expected, "恢复只允许 auth_generation+1");
}

/// 「遗计确认门 + 同深度追加机捷挂起」的恢复件（修复前的 park-1 形态）。
fn same_depth_pair_hex() -> String {
    let mut engine = Engine::new(2306, 3);
    engine.debug_set_turn(0, Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_grant(1, "遗计");
    let dealt = engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(matches!(dealt, CommandOutcome::Pending { .. }), "{dealt:?}");
    let mut snapshot = engine.save();
    let depth = snapshot.resolution.stack.len();
    assert_eq!(
        snapshot
            .resolution
            .open_choice
            .as_ref()
            .unwrap()
            .spec
            .prompt,
        "trigger_confirm"
    );
    snapshot.resolution.parks.push(ParkSnapshot {
        resume_depth: depth,
        slot: Slot::Trigger { frame: NO_FRAME },
        machine: MachineSnapshot::JiJie {
            owner: 0,
            card: None,
            stage: JiJieStageSnap::Ask,
        },
    });
    engine_native_wasm::hex_encode(&engine_native::archive::encode_archive(&snapshot))
}

/// `NativeSession::auto_step` 的 byte 路径等价物：与 C331 的 `session.rs`
/// 同一动作选择（开放选择取前 N 个牌选项，否则前 N 个选项；无选择时用
/// `BoundedPlanner::new(64)` 规划）与同一提交入口／去重键。
fn byte_auto_step(engine: &mut Engine) -> GameAction {
    if let Some(c) = engine.open_choice() {
        let take = c.spec.min.max(1).min(c.spec.max).min(c.spec.options.len());
        let sel: Vec<_> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, engine_native::ChoiceOption::Card(_)))
            .take(take)
            .cloned()
            .collect();
        let sel = if sel.len() == take {
            sel
        } else {
            c.spec.options.iter().take(take).cloned().collect()
        };
        let (cid, obs) = (c.id, c.observer);
        let action = GameAction::Answer {
            choice_id: cid,
            selection: sel,
        };
        // 与 `NativeSession::auto_step` 同一去重键策略（逐字节对比的前提）。
        let outcome = engine.submit(engine_native::GameCommand {
            actor: obs,
            expected_revision: engine.revision(),
            dedup_id: engine.revision().wrapping_mul(2).max(1),
            action: action.clone(),
        });
        assert!(
            matches!(
                outcome,
                CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
            ),
            "{outcome:?}"
        );
        return action;
    }
    let seat = engine.turn().current;
    let plan = engine.plan_action(seat, &BoundedPlanner::new(64));
    let action = plan.action.clone();
    engine.apply_plan(&plan).expect("plan applies");
    action
}

#[test]
fn restored_same_depth_continuation_is_accepted_on_both_paths() {
    let fixture = same_depth_pair_hex();
    let original = decode(&fixture);

    // ---- typed 路径（与 WASM 窄壳同一实现）----
    let mut typed = NativeSession::new(3, 2306).unwrap();
    typed.restore_hex(&fixture).expect("typed restore");
    assert_only_generation_rotated(&original, &decode(&typed.save_hex()));
    let first = typed.auto_step().expect("确认门应答");
    assert!(first.contains("\"kind\":\"answer\""), "{first}");
    assert!(first.contains("pending"), "{first}");
    let after_first = typed.save_hex();
    assert!(
        typed
            .open_choice_json(0)
            .unwrap()
            .contains("jijie_give_target"),
        "机捷开窗"
    );

    let mut typed = NativeSession::new(3, 2306).unwrap();
    typed
        .restore_hex(&after_first)
        .expect("typed restore after ask");
    let continued = typed.auto_step().expect("恢复后继续不得 panic");
    assert!(continued.contains("\"kind\":\"answer\""), "{continued}");
    assert!(continued.contains("pending"), "{continued}");
    // 恢复后继续的完整信封面（浏览器每步都查这些）：不得触发内核 panic。
    let view = typed.view_json(1).expect("view");
    let legal = typed.legal_actions_json(1).expect("legal actions");
    let choice = typed.open_choice_json(1).expect("choice view");
    let hash = typed.state_hash_hex();
    let codes = typed.public_event_codes_json(0);
    assert!(!view.is_empty());
    assert!(
        legal.contains("\"actions\":[{"),
        "恢复后继续必须仍有合法动作：{legal}"
    );
    assert!(
        choice.contains("\"prompt\":\"jiji_"),
        "遗计内层应提出自己的询问：{choice}"
    );
    assert_eq!(typed.revision(), 3);
    assert!(!hash.is_empty() && !codes.is_empty());

    // 恢复后继续的存档仍可再次恢复（承接关系自洽）。
    let final_hex = typed.save_hex();
    let mut again = NativeSession::new(3, 2306).unwrap();
    again
        .restore_hex(&final_hex)
        .expect("continuation archive restores");
    assert_eq!(again.state_hash_hex(), hash);

    // ---- byte 路径：与 typed 逐步逐字节一致 ----
    let mut byte = Engine::restore(&original).expect("byte restore");
    let first_action = byte_auto_step(&mut byte);
    assert!(
        matches!(first_action, GameAction::Answer { .. }),
        "{first_action:?}"
    );
    assert_eq!(
        encode_hex(&byte),
        after_first,
        "typed 与 byte 的首次应答存档必须逐字节一致"
    );
    let mut byte = Engine::restore(&decode(&after_first)).expect("byte restore after ask");
    let continued_action = byte_auto_step(&mut byte);
    assert!(
        matches!(continued_action, GameAction::Answer { .. }),
        "{continued_action:?}"
    );
    assert_eq!(
        engine_native_wasm::hex_encode(&byte.state_hash()),
        hash,
        "typed 与 byte 的继续状态摘要必须一致"
    );
    assert_eq!(
        encode_hex(&byte),
        final_hex,
        "typed 与 byte 的继续存档必须逐字节一致"
    );
    assert!(
        !byte.legal_actions(1).is_empty(),
        "byte 路径合法动作同样非空"
    );
}

#[test]
fn malformed_restore_keeps_the_restored_session_atomic() {
    let fixture = same_depth_pair_hex();
    let mut session = NativeSession::new(3, 2306).unwrap();
    session.restore_hex(&fixture).expect("typed restore");
    session.auto_step().expect("确认门应答");
    let saved = session.save_hex();
    let before = decode(&saved);

    // 控制组：坏档（"00"/奇数位/非十六进制/截断）受控拒绝，句柄与存档原子不变。
    for bad in ["00", "0", "zz", &saved[..saved.len() - 2]] {
        assert!(
            session.restore_hex(bad).is_err(),
            "坏档必须受控拒绝：{bad:?}"
        );
        assert_eq!(session.save_hex(), saved, "坏恢复不得改变原局");
    }
    // 拒绝之后仍能继续对局（不是只能读）。
    let stepped = session.auto_step().expect("坏恢复后仍可继续");
    assert!(stepped.contains("\"kind\":\"answer\""), "{stepped}");
    assert_eq!(session.revision(), before.core.revision + 1);
}
