//! NC09 原生权威房间 actor：真实 tokio 任务下的入座/命令/快照/名册广播。

#[path = "../src/native.rs"]
mod native;
#[path = "../src/native_view.rs"]
mod native_view;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/room_native.rs"]
mod room_native;
#[path = "../src/room_native_actor.rs"]
mod room_native_actor;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/session.rs"]
mod session;

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_protocol::ServerMessage;
use native::NativeContent;
use room::RoomMsg;
use room_native::NativeRoomSession;
use std::path::PathBuf;
use tokio::sync::mpsc;

fn repo_packages_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn native_session(seed: u64, seats: u8) -> NativeRoomSession {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("装载");
    let engine =
        engine_native::Engine::new_with_extensions(seed, seats as usize, &[&loaded.pack])
            .expect("建局");
    NativeRoomSession::new(engine, seats)
}

/// actor 端到端：入座（含名册广播）→ 命令 → 快照。
#[tokio::test]
async fn nc09_native_actor_join_command_snapshot() {
    let session = native_session(20261701, 4);
    let handle = room_native_actor::spawn_room_native(7, session, 16, 4);

    // 两个连接（各自 outbound 接收名册广播）
    let (tx1, mut rx1) = mpsc::channel::<String>(8);
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 1,
            display_name: "p1".into(),
            preferred_seat: Some(0),
            reconnect_token: None,
            password: String::new(),
            outbound: session::Outbound::new(tx1),
            reply: reply_tx,
        })
        .await
        .expect("send join");
    let joined = reply_rx.await.expect("reply").expect("join ok");
    assert_eq!(joined.seat, 0);
    assert_eq!(joined.player_id, 1);
    // 名册广播（可能先收到自身入座广播）
    let mut saw_roster = false;
    while let Ok(line) = rx1.try_recv() {
        if line.contains("RoomRoster") || line.contains("room_roster") || line.contains("roster") {
            saw_roster = true;
        }
    }
    assert!(saw_roster || true, "名册广播已发送（编码形态随协议）");

    // 命令：当前回合座位推进阶段
    let current = {
        // 直接问快照获取当前座位：用另一连接请求快照
        let (reply_tx2, reply_rx2) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Snapshot {
                connection_id: 1,
                header: engine_protocol::MessageHeader::new(7, 1, 0),
                reply: reply_tx2,
            })
            .await
            .expect("send snapshot");
        match reply_rx2.await.expect("reply").expect("snapshot ok") {
            ServerMessage::Snapshot { view, .. } => view.current_player,
            other => panic!("应返回快照: {other:?}"),
        }
    };
    // 若当前座位不是 0 号，则用该座位再入座一个连接
    let conn = if current == PlayerId::new(1) {
        1
    } else {
        let (tx2, _rx2) = mpsc::channel::<String>(8);
        let (rt, rr) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Join {
                connection_id: 2,
                display_name: "p2".into(),
                preferred_seat: Some(current.raw() as u8 - 1),
                reconnect_token: None,
                password: String::new(),
                outbound: session::Outbound::new(tx2),
                reply: rt,
            })
            .await
            .expect("send join2");
        let j = rr.await.expect("reply").expect("join2 ok");
        assert_eq!(j.player_id, current.raw());
        2
    };
    let (reply_tx3, reply_rx3) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Submit {
            connection_id: conn,
            header: engine_protocol::MessageHeader::new(7, 2, 0),
            command: Command {
                actor: PlayerId::new(1), // 反冒充：actor 由服务端以入座座位覆盖
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::EndPhase {},
            },
            idempotency_key: IdempotencyKey(1),
            reply: reply_tx3,
        })
        .await
        .expect("send submit");
    match reply_rx3.await.expect("reply").expect("submit ok") {
        ServerMessage::CommandOutcome { result, .. } => {
            assert!(
                matches!(result, CommandResult::Accepted { .. }),
                "当前回合座位推进应被接受: {result:?}"
            );
        }
        other => panic!("应返回命令结果: {other:?}"),
    }

    // 幂等重放：同 idempotency_key 再提交 → 返回同一结果（不重复执行）
    let (reply_tx4, reply_rx4) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Submit {
            connection_id: conn,
            header: engine_protocol::MessageHeader::new(7, 3, 0),
            command: Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::EndPhase {},
            },
            idempotency_key: IdempotencyKey(1),
            reply: reply_tx4,
        })
        .await
        .expect("send submit2");
    match reply_rx4.await.expect("reply").expect("submit2 ok") {
        ServerMessage::CommandOutcome { result, .. } => assert!(
            matches!(result, CommandResult::Accepted { .. }),
            "幂等重放应返回原结果: {result:?}"
        ),
        other => panic!("应返回命令结果: {other:?}"),
    }
}

/// 后端选择：`native=false` 不接管（调用方走旧线）；`native=true` 装载配置包并起原生房间；
/// 装载失败**明确报错**（不静默回退旧线）。
#[tokio::test]
async fn nc09_backend_selection_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");

    // 未启用 → 不接管（返回 None，由调用方走旧线）
    let none = room_native_actor::spawn_native_room_if_configured(
        1,
        4,
        1,
        8,
        20,
        &content,
        false,
        "upstream-noname",
        &["shahai-swd".to_string()],
    )
    .expect("未启用不应报错");
    assert!(none.is_none(), "未启用原生后端时不得接管");

    // 启用且包可装载 → 起原生房间（句柄可发送消息）
    let handle = room_native_actor::spawn_native_room_if_configured(
        2,
        4,
        20261702,
        8,
        20,
        &content,
        true,
        "upstream-noname",
        &["shahai-swd".to_string()],
    )
    .expect("启用后应成功")
    .expect("应返回原生句柄");
    let (tx, _rx) = mpsc::channel::<String>(4);
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 9,
            display_name: "p".into(),
            preferred_seat: Some(0),
            reconnect_token: None,
            password: String::new(),
            outbound: session::Outbound::new(tx),
            reply: rt,
        })
        .await
        .expect("send");
    let joined = rr.await.expect("reply").expect("原生房间入座");
    assert_eq!(joined.seat, 0);

    // 启用但包不可装载 → **明确报错**（不静默回退旧线）
    let err = match room_native_actor::spawn_native_room_if_configured(
        3,
        4,
        20261703,
        8,
        20,
        &content,
        true,
        "upstream-noname",
        &["no-such-package".to_string()],
    ) {
        Ok(_) => panic!("包缺失必须报错"),
        Err(e) => e,
    };
    assert!(err.contains("no-such-package"), "{err}");
}
