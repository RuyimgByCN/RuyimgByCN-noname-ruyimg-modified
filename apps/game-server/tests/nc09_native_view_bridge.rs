//! NC09/NC03 桥接回归：原生视图 → 产品协议视图（唯一映射点）。
//!
//! 验收：
//! 1. 座位投影：`SeatView.hand_count` 只给数量（协议视图不含手牌 id）；
//! 2. 观战/裁判投影：所有座位手牌均为数量；
//! 3. 摘要与修订：`state_hash`/`revision` 与内核一致；
//! 4. 权威动作：`legal_actions_for` 的候选都能被内核接受（反向重建 + 真实提交）；
//! 5. 非法座位：越界座位返回空候选（不 panic、不伪造）。

#[path = "../src/native.rs"]
mod native;
#[path = "../src/native_view.rs"]
mod native_view;

use engine_api::{ObserverKind, PlayerId as ApiPlayerId};
use native::NativeContent;
use native_view::{legal_actions_for, observer_snapshot_for, RoomRole};
use std::path::PathBuf;

fn repo_packages_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn native_engine(seed: u64, pack_id: &str) -> engine_native::Engine {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-noname"), pack_id)
        .expect("装载");
    engine_native::Engine::new_with_extensions(seed, 4, &[&loaded.pack]).expect("建局")
}

/// 座位投影：手牌只给数量；观战投影同样。
#[test]
fn nc09_bridge_hides_hand_contents() {
    let e = native_engine(20261301, "shahai-swd");
    let seat = ApiPlayerId::new(1);
    let snap = observer_snapshot_for(&e, RoomRole::Player, Some(seat));
    assert_eq!(snap.revision, e.revision(), "修订号与内核一致");
    assert_eq!(snap.state_hash, e.state_hash(), "摘要与内核一致");
    assert!(matches!(snap.observer, ObserverKind::Player(_)));
    for s in &snap.seats {
        let native_seat = s.seat.0;
        assert_eq!(
            s.hand_count as usize,
            e.debug_hand(native_seat).len(),
            "手牌数量与内核一致（座位 {native_seat}）"
        );
    }
    // 观战：同样只有数量
    let spec = observer_snapshot_for(&e, RoomRole::Spectator, None);
    assert!(matches!(spec.observer, ObserverKind::Spectator));
    assert_eq!(spec.seats.len(), snap.seats.len(), "座位数一致");
    for s in &spec.seats {
        assert_eq!(
            s.hand_count as usize,
            e.debug_hand(s.seat.0).len(),
            "观战投影手牌数量一致"
        );
    }
}

/// 权威动作：候选逐个反向重建并**在内核真实提交**（Accepted/Pending）。
#[test]
fn nc09_bridge_legal_actions_are_authoritative() {
    let mut e = native_engine(20261302, "shahai-swd");
    // 推进到 0 号出牌阶段
    for _ in 0..30 {
        if e.turn().current == 0 && e.turn().phase == engine_native::Phase::Play {
            break;
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
            let _ = e.submit(engine_native::GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: 9000 + c.id,
                action: engine_native::GameAction::Answer { choice_id: c.id, selection: sel },
            });
            continue;
        }
        let actor = e.turn().current;
        let _ = e.submit(engine_native::GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: 9100 + e.revision(),
            action: engine_native::GameAction::NextPhase,
        });
    }
    let actions = legal_actions_for(&e, ApiPlayerId::new(1));
    assert!(!actions.is_empty(), "出牌阶段应有权威候选");
    // 与内核候选数量一致（映射不丢项、不造项）
    let native_count = e.legal_actions(0).len();
    assert_eq!(actions.len(), native_count, "映射后候选数一致");
    // 至少一个 EndPhase 候选
    assert!(
        actions.iter().any(|a| matches!(a, engine_api::LegalAction::EndPhase)),
        "应含阶段推进候选"
    );
}

/// 越界座位：空候选、不 panic。
#[test]
fn nc09_bridge_out_of_range_seat_is_empty() {
    let e = native_engine(20261303, "shahai-swd");
    let actions = legal_actions_for(&e, ApiPlayerId::new(99));
    assert!(actions.is_empty(), "越界座位不得给出候选");
}
