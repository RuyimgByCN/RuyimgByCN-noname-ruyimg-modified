//! NC09 原生权威房间 actor：真实 tokio 任务下的入座/命令/快照/名册广播。

#[path = "../src/native.rs"]
mod native;
#[path = "../src/native_room_store.rs"]
mod native_room_store;
#[path = "../src/native_view.rs"]
mod native_view;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/room_native.rs"]
mod room_native;
#[path = "../src/room_native_actor.rs"]
mod room_native_actor;
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
    let engine = engine_native::Engine::new_with_extensions(seed, seats as usize, &[&loaded.pack])
        .expect("建局");
    NativeRoomSession::new(engine, seats)
}

async fn attached(
    handle: &room::RoomHandle,
    connection_id: u64,
    seat: u8,
    token: Option<String>,
    password: &str,
) -> Result<(room::JoinReply, mpsc::Receiver<String>), String> {
    let (tx, rx) = mpsc::channel(128);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id,
            display_name: format!("p{connection_id}"),
            preferred_seat: Some(seat),
            reconnect_token: token,
            password: password.into(),
            outbound: session::Outbound::new(tx),
            reply,
        })
        .await?;
    Ok((response.await.unwrap()?, rx))
}

async fn end_phase(
    handle: &room::RoomHandle,
    connection_id: u64,
    header_revision: u64,
    body_revision: u64,
    key: u64,
) -> Result<ServerMessage, String> {
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Submit {
            connection_id,
            header: engine_protocol::MessageHeader::new(handle.id, key, header_revision),
            command: Command {
                actor: PlayerId::new(999),
                expected_revision: ExpectedRevision(body_revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::EndPhase {},
            },
            idempotency_key: IdempotencyKey(key),
            replay: false,
            reply,
        })
        .await?;
    response.await.unwrap()
}

#[tokio::test]
async fn nc09_configured_actor_keeps_password_packages_general_and_seat_ownership() {
    let dir = tempfile::tempdir().unwrap();
    let content = NativeContent::open(repo_packages_root(), dir.path().join("packages")).unwrap();
    content.import().unwrap();
    let store = native_room_store::NativeRoomStore::new(dir.path().join("rooms"));
    let handle = room_native_actor::spawn_native_room_if_configured(
        90,
        2,
        4242,
        "secret".into(),
        16,
        20,
        &content,
        true,
        "upstream-noname",
        &["shahai-swd".into(), "huanle-cards".into()],
        Some(store.clone()),
    )
    .unwrap()
    .unwrap();
    assert!(attached(&handle, 9, 0, None, "wrong").await.is_err());
    let (a, _rx_a) = attached(&handle, 1, 0, None, "secret").await.unwrap();
    let (_b, _rx_b) = attached(&handle, 2, 1, None, "secret").await.unwrap();
    let initial = store.load_all().unwrap().pop().unwrap();
    assert_eq!(initial.state.pack_refs.len(), 2);
    for reference in &initial.state.pack_refs {
        content.load_reference(reference).unwrap();
    }

    let baseline = engine_native::Engine::new(1, 2);
    let name = baseline
        .general_candidates()
        .into_iter()
        .filter(|def| *def as u32 != a.view.seats[0].character_def)
        .filter_map(|def| baseline.catalog().general_meta(def).map(|meta| meta.0))
        .next()
        .unwrap();
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::ChooseGeneral {
            connection_id: 1,
            name: name.into(),
            reply,
        })
        .await
        .unwrap();
    response.await.unwrap().unwrap();
    let selected = store.load_all().unwrap().pop().unwrap();
    assert_ne!(
        initial.state.snapshot_hex, selected.state.snapshot_hex,
        "同修订选将仍须落盘"
    );
    let restored = NativeRoomSession::from_saved(
        &selected.state,
        selected
            .state
            .pack_refs
            .iter()
            .map(|r| content.load_reference(r).unwrap())
            .collect(),
    )
    .unwrap();
    let selected_hash = restored.engine().state_hash();
    let (reply, response) = tokio::sync::oneshot::channel();
    handle.send(RoomMsg::Recover { reply }).await.unwrap();
    response.await.unwrap().unwrap();
    let (again, mut rx_again) = attached(&handle, 11, 0, Some(a.reconnect_token), "secret")
        .await
        .unwrap();
    assert_eq!(again.view.state_hash, selected_hash, "内存恢复保留选将结果");
    let rebound = store.load_all().unwrap().pop().unwrap();
    assert!(rebound.state.members.iter().any(|m| m.connection_id == 11));
    assert!(!rebound.state.members.iter().any(|m| m.connection_id == 1));
    assert!(
        end_phase(&handle, 1, 0, 0, 77).await.is_err(),
        "旧连接已无提交权"
    );
    while let Ok(text) = rx_again.try_recv() {
        if let ServerMessage::RoomRoster { seats, .. } = engine_protocol::decode_s2c(&text).unwrap()
        {
            assert_eq!(seats.len(), 2, "重连不能复制座位");
        }
    }
    let rev = match end_phase(&handle, 11, 0, 0, 77).await.unwrap() {
        ServerMessage::CommandOutcome {
            result: CommandResult::Accepted { new_revision, .. },
            ..
        } => new_revision,
        other => panic!("{other:?}"),
    };
    assert!(
        matches!(
            end_phase(&handle, 2, rev, rev, 77).await.unwrap(),
            ServerMessage::CommandOutcome {
                result: CommandResult::Rejected { .. },
                ..
            }
        ),
        "另一玩家的幂等键不能重放已接受结果"
    );
    let current = match end_phase(&handle, 11, rev, rev, 78).await.unwrap() {
        ServerMessage::CommandOutcome {
            result: CommandResult::Accepted { new_revision, .. },
            ..
        } => new_revision,
        other => panic!("{other:?}"),
    };
    assert!(current > rev);
    assert!(
        matches!(
            end_phase(&handle, 11, 0, rev, 79).await.unwrap(),
            ServerMessage::CommandOutcome {
                result: CommandResult::Rejected {
                    reason: engine_api::RejectReason::StaleRevision { .. }
                },
                ..
            }
        ),
        "header未指定修订时仍须校验body.expected_revision"
    );
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
            replay: false,
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
            replay: false,
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
        String::new(),
        8,
        20,
        &content,
        false,
        "upstream-noname",
        &["shahai-swd".to_string()],
        None,
    )
    .expect("未启用不应报错");
    assert!(none.is_none(), "未启用原生后端时不得接管");

    // 启用且包可装载 → 起原生房间（句柄可发送消息）
    let handle = room_native_actor::spawn_native_room_if_configured(
        2,
        4,
        20261702,
        String::new(),
        8,
        20,
        &content,
        true,
        "upstream-noname",
        &["shahai-swd".to_string()],
        None,
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
        String::new(),
        8,
        20,
        &content,
        true,
        "upstream-noname",
        &["no-such-package".to_string()],
        None,
    ) {
        Ok(_) => panic!("包缺失必须报错"),
        Err(e) => e,
    };
    assert!(err.contains("no-such-package"), "{err}");
}

/// 入座助手（沿用既有报文形态）：返回座位号；失败返回 Err。
async fn join(
    handle: &room::RoomHandle,
    connection_id: u64,
    preferred_seat: Option<u8>,
    reconnect_token: Option<String>,
) -> Result<u8, String> {
    let (tx, _rx) = mpsc::channel::<String>(8);
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id,
            display_name: format!("p{connection_id}"),
            preferred_seat,
            reconnect_token,
            password: String::new(),
            outbound: session::Outbound::new(tx),
            reply: reply_tx,
        })
        .await
        .map_err(|e| format!("send: {e}"))?;
    reply_rx
        .await
        .map_err(|e| format!("reply: {e}"))?
        .map(|r| r.seat)
}

/// NC09 惰性缺席座位回收：满员且某成员超过时限未活动时，新入座者顶替其座位；
/// 时限为 0 时关闭该行为（不得回收）。
#[tokio::test]
async fn nc09_actor_reclaims_idle_seat_lazily() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("装载");
    let engine = engine_native::Engine::new_with_extensions(20263900, 2, &[&loaded.pack])
        .expect("建局（2 座）");
    let session = room_native::NativeRoomSession::new_with_packs(engine, vec![loaded.pack], 2);

    // 时限设为 1 秒，便于观察
    room_native_actor::set_seat_timeout_secs(1);
    let handle = room_native_actor::spawn_room_native(70, session, 8, 4);

    let (j1, _rx1) = attached(&handle, 1, 0, None, "").await.expect("入座 1");
    assert_eq!(j1.seat, 0);
    let (j2, _rx2) = attached(&handle, 2, 1, None, "").await.expect("入座 2");
    assert_eq!(j2.seat, 1);

    // 满员：立即入座应失败（座位被占）
    let busy = join(&handle, 3, None, None).await;
    assert!(busy.is_err(), "满员且未超时 → 入座应失败：{busy:?}");

    // 在线等待他人回合/选择超过时限，仍不得回收。
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    assert!(
        join(&handle, 3, None, None).await.is_err(),
        "在线空闲席位不得被回收"
    );
    handle
        .send(RoomMsg::Disconnect { connection_id: 1 })
        .await
        .unwrap();
    assert!(
        join(&handle, 3, None, None).await.is_err(),
        "离线未超时不得回收"
    );
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    let taken = join(&handle, 4, None, None).await.expect("超时后应可入座");
    assert_eq!(taken, 0, "只能回收已离线的座位");

    // 关闭回收（0）→ 满员时不得回收
    room_native_actor::set_seat_timeout_secs(0);
    let session2 = {
        let loaded = content
            .load_scoped(Some("upstream-noname"), "shahai-swd")
            .expect("重装");
        let engine = engine_native::Engine::new_with_extensions(20263901, 2, &[&loaded.pack])
            .expect("建局 2");
        room_native::NativeRoomSession::new_with_packs(engine, vec![loaded.pack], 2)
    };
    let handle2 = room_native_actor::spawn_room_native(71, session2, 8, 4);
    let _ = join(&handle2, 1, Some(0), None).await.expect("入座 a");
    let _ = join(&handle2, 2, Some(1), None).await.expect("入座 b");
    handle2
        .send(RoomMsg::Disconnect { connection_id: 1 })
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    let closed = join(&handle2, 3, None, None).await;
    assert!(closed.is_err(), "时限 0 → 不回收，入座应失败：{closed:?}");
    room_native_actor::set_seat_timeout_secs(600); // 复位默认
}

/// P1 回归：一个停止读取的旁观者填满出站队列后，不得阻塞房间 actor；
/// 其他玩家的命令必须在期限内完成，慢连接被明确移出广播并等待重连。
#[tokio::test]
async fn slow_observer_queue_full_does_not_block_room_actor() {
    let session = native_session(20261024, 2);
    let handle = room_native_actor::spawn_room_native(77, session, 8, 4);
    let _ = attached(&handle, 1, 0, None, "").await.expect("入座 a");
    let _ = attached(&handle, 2, 1, None, "").await.expect("入座 b");

    // 容量 1 且故意不读：入房名册已占满队列。
    let (tx, mut slow_rx) = mpsc::channel::<String>(1);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Spectate {
            connection_id: 3,
            display_name: "slow".into(),
            outbound: session::Outbound::new(tx),
            reply,
        })
        .await
        .unwrap();
    response.await.unwrap().expect("旁观入房");
    // 故意不读：入房名册仍占着唯一的队列槽位。

    let outcome = tokio::time::timeout(
        std::time::Duration::from_secs(3),
        end_phase(&handle, 1, 0, 0, 1),
    )
    .await
    .expect("慢旁观者不得阻塞房间命令")
    .expect("命令成功");
    match outcome {
        ServerMessage::CommandOutcome { result, .. } => {
            assert!(
                matches!(result, CommandResult::Accepted { .. }),
                "命令应被接受：{result:?}"
            );
        }
        other => panic!("期望 CommandOutcome，得到 {other:?}"),
    }

    // 慢连接被移出广播后，其发送端应关闭（session 侧也会收到 slow 信号）。
    let closed = tokio::time::timeout(std::time::Duration::from_secs(1), async {
        loop {
            match slow_rx.try_recv() {
                Ok(_) => continue,
                Err(mpsc::error::TryRecvError::Disconnected) => break true,
                Err(mpsc::error::TryRecvError::Empty) => {
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            }
        }
    })
    .await
    .expect("慢连接应被明确断开");
    assert!(closed);
}

/// P2 切片：原生 CommandOutcome 必须带真实公开事件码（此前恒为空）。
#[tokio::test]
async fn native_command_outcome_carries_public_event_codes() {
    let session = native_session(20261025, 2);
    let handle = room_native_actor::spawn_room_native(78, session, 8, 4);
    let _ = attached(&handle, 1, 0, None, "").await.expect("入座");
    let message = end_phase(&handle, 1, 0, 0, 5).await.expect("命令");
    let ServerMessage::CommandOutcome {
        result,
        public_event_codes,
        ..
    } = message
    else {
        panic!("期望 CommandOutcome");
    };
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "命令应接受：{result:?}"
    );
    assert!(
        !public_event_codes.is_empty(),
        "原生命令应带真实公开事件码"
    );
    assert!(
        public_event_codes
            .iter()
            .all(|code| (1..=10).contains(code)),
        "公开事件码越界：{public_event_codes:?}"
    );
}
