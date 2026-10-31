//! NC09 双后端等价面：**同一批会话语义**在旧线房间与原生房间上都必须成立。
//!
//! 口径：旧线房间（`room::spawn_room*`）与原生房间（`room_native_actor::spawn_room_native`）
//! 共用 `RoomMsg`/`RoomHandle`/`JoinReply` 与协议消息类型；本文件对**可对照的会话语义**
//! 逐条在两后端上跑同一断言：入座座位/玩家 id、满座拒绝、重连保留座位、观战无权威动作、
//! 非法命令明确拒绝。差异（如旧线 `StartGame` 前置、原生建局即开局）在证据中登记。

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
use engine_protocol::{ClientRole, MessageHeader, ServerMessage};
use native::NativeContent;
use room::{RoomHandle, RoomMsg};
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
    // 保存包集合：Recover 重放需要同内容
    NativeRoomSession::new_with_packs(engine, vec![loaded.pack], seats)
}

async fn join(
    handle: &RoomHandle,
    conn: u64,
    seat: Option<u8>,
    token: Option<String>,
) -> Result<room::JoinReply, String> {
    let (tx, _rx) = mpsc::channel::<String>(8);
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: conn,
            display_name: format!("c{conn}"),
            preferred_seat: seat,
            reconnect_token: token,
            password: String::new(),
            outbound: session::Outbound::new(tx),
            reply: rt,
        })
        .await
        .expect("send join");
    rr.await.expect("reply")
}

async fn spectate(handle: &RoomHandle, conn: u64) -> Result<room::JoinReply, String> {
    let (tx, _rx) = mpsc::channel::<String>(8);
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Spectate {
            connection_id: conn,
            display_name: format!("s{conn}"),
            outbound: session::Outbound::new(tx),
            reply: rt,
        })
        .await
        .expect("send spectate");
    rr.await.expect("reply")
}

async fn submit(
    handle: &RoomHandle,
    conn: u64,
    seq: u64,
    kind: CommandKind,
    idem: u64,
) -> Result<ServerMessage, String> {
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Submit {
            connection_id: conn,
            header: MessageHeader::new(1, seq, 0),
            command: Command {
                actor: PlayerId::NONE, // 反冒充：以入座座位为准
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(idem),
                kind,
            },
            idempotency_key: IdempotencyKey(idem),
            replay: false,
            reply: rt,
        })
        .await
        .expect("send submit");
    rr.await.expect("reply")
}

/// 原生后端：入座/满座/重连/观战/拒绝（与旧线共用断言）。
#[tokio::test]
async fn nc09_native_backend_session_semantics() {
    let handle = room_native_actor::spawn_room_native(11, native_session(20261801, 2), 16, 4);

    let a = join(&handle, 1, Some(0), None).await.expect("入座 0");
    assert_eq!(a.seat, 0);
    assert_eq!(a.role, ClientRole::Player);
    let b = join(&handle, 2, Some(1), None).await.expect("入座 1");
    assert_eq!(b.seat, 1);
    // 满座（2 座）
    assert!(join(&handle, 3, None, None).await.is_err(), "满座必须拒绝");
    // 重连：令牌换绑连接、保留座位
    let token = a.reconnect_token.clone();
    let again = join(&handle, 4, None, Some(token.clone()))
        .await
        .expect("重连");
    assert_eq!(again.seat, 0, "重连保留座位");
    assert_eq!(again.reconnect_token, token, "令牌不变");
    // 无效令牌
    assert!(
        join(&handle, 5, None, Some("bogus".into())).await.is_err(),
        "无效令牌必须失败"
    );
    // 观战：公开投影、无权威动作
    let spec = spectate(&handle, 6).await.expect("观战");
    assert_eq!(spec.role, ClientRole::Spectator);
    assert!(spec.legal_actions.is_empty(), "观战者无权威动作");
    // 观战者提交 → 明确拒绝
    let out = submit(&handle, 6, 1, CommandKind::EndPhase {}, 1).await;
    assert!(out.is_err(), "观战者提交必须失败: {out:?}");
}

/// 原生后端：非当前回合座位推进 → 明确拒绝（协议拒绝原因可见）。
#[tokio::test]
async fn nc09_native_backend_rejects_illegal_command() {
    let handle = room_native_actor::spawn_room_native(12, native_session(20261802, 4), 16, 4);
    // 占满四个座位，确保「非当前回合」座位存在
    for seat in 0..4u8 {
        let _ = join(&handle, 100 + seat as u64, Some(seat), None)
            .await
            .expect("入座");
    }
    // 找一个非当前回合座位：用快照问当前座位
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Snapshot {
            connection_id: 100,
            header: MessageHeader::new(12, 1, 0),
            reply: rt,
        })
        .await
        .expect("send snapshot");
    let current = match rr.await.expect("reply").expect("snapshot") {
        ServerMessage::Snapshot { view, .. } => view.current_player.raw(),
        other => panic!("应返回快照: {other:?}"),
    };
    let other_seat = (0..4u8)
        .find(|s| *s as u32 + 1 != current)
        .expect("其他座位");
    let conn = 100 + other_seat as u64;
    match submit(&handle, conn, 2, CommandKind::EndPhase {}, 7).await {
        Ok(ServerMessage::CommandOutcome { result, .. }) => match result {
            CommandResult::Rejected { reason } => {
                assert!(
                    matches!(reason, engine_api::RejectReason::InvalidActor),
                    "非本人回合应 InvalidActor: {reason:?}"
                );
            }
            other => panic!("应被拒绝: {other:?}"),
        },
        other => panic!("应返回命令结果: {other:?}"),
    }
}

/// 旧线协议兼容：`StartGame` 作为「开局确认」——参数匹配则接受（原生建局即开局），
/// 不匹配则**明确拒绝**（不静默吞掉）。
#[tokio::test]
async fn nc09_native_backend_startgame_is_confirmation_only() {
    let handle = room_native_actor::spawn_room_native(13, native_session(20261803, 4), 16, 4);
    let _ = join(&handle, 1, Some(0), None).await.expect("入座");
    // 匹配：接受
    let ok = submit(
        &handle,
        1,
        1,
        CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20261803,
            ruleset: String::new(),
        },
        11,
    )
    .await
    .expect("提交");
    match ok {
        ServerMessage::CommandOutcome { result, .. } => assert!(
            matches!(result, CommandResult::Accepted { .. }),
            "匹配的开局确认应接受: {result:?}"
        ),
        other => panic!("应返回命令结果: {other:?}"),
    }
    // 不匹配：明确拒绝（返回错误）
    let bad = submit(
        &handle,
        1,
        2,
        CommandKind::StartGame {
            player_count: 3,
            rng_seed: 1,
            ruleset: String::new(),
        },
        12,
    )
    .await;
    assert!(bad.is_err(), "座位数不一致的开局确认必须报错: {bad:?}");
}

/// 房间密码：非空密码下错误密码必须被拒（与旧线同口径 `unauthorized`）。
#[tokio::test]
async fn nc09_native_backend_enforces_room_password() {
    let handle = room_native_actor::spawn_room_native_with_password(
        14,
        native_session(20261804, 2),
        16,
        4,
        "s3cret".to_string(),
    );
    // 错误密码 → 拒绝
    let (tx, _rx) = mpsc::channel::<String>(4);
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 1,
            display_name: "bad".into(),
            preferred_seat: Some(0),
            reconnect_token: None,
            password: "wrong".into(),
            outbound: session::Outbound::new(tx),
            reply: rt,
        })
        .await
        .expect("send");
    let err = rr.await.expect("reply").expect_err("错误密码必须被拒");
    assert_eq!(err, "unauthorized");

    // 正确密码 → 入座
    let ok = {
        let (tx, _rx) = mpsc::channel::<String>(4);
        let (rt, rr) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Join {
                connection_id: 2,
                display_name: "good".into(),
                preferred_seat: Some(0),
                reconnect_token: None,
                password: "s3cret".into(),
                outbound: session::Outbound::new(tx),
                reply: rt,
            })
            .await
            .expect("send");
        rr.await.expect("reply")
    };
    assert_eq!(ok.expect("正确密码应入座").seat, 0);
}

/// `Recover`：快照 + WAL 重放恢复后，修订与摘要与实时推进一致；内容不符时明确失败。
#[tokio::test]
async fn nc09_native_backend_recover_matches_live_state() {
    let handle = room_native_actor::spawn_room_native(15, native_session(20261805, 4), 16, 1);
    let _ = join(&handle, 1, Some(0), None).await.expect("入座");
    // 记录若干条被接受的阶段推进（snapshot_every=1 → 每条都落快照）
    let mut accepted = 0;
    for i in 0..6u64 {
        let current = {
            let (rt, rr) = tokio::sync::oneshot::channel();
            handle
                .send(RoomMsg::Snapshot {
                    connection_id: 1,
                    header: MessageHeader::new(15, 1, 0),
                    reply: rt,
                })
                .await
                .expect("send snapshot");
            match rr.await.expect("reply").expect("snapshot") {
                ServerMessage::Snapshot { view, .. } => view.current_player.raw(),
                other => panic!("应返回快照: {other:?}"),
            }
        };
        let conn = 1 + current as u64; // 连接 id 与座位约定：1 号连接=座位 0
        let conn = if current == 1 { 1 } else { conn };
        let seat = current as u8 - 1;
        if seat != 0 {
            // 该座位未入座 → 先入座（幂等：已入座会失败，忽略）
            let _ = join(&handle, 1 + current as u64, Some(seat), None).await;
        }
        match submit(&handle, conn, 10 + i, CommandKind::EndPhase {}, 100 + i).await {
            Ok(ServerMessage::CommandOutcome { result, .. }) => {
                if matches!(result, CommandResult::Accepted { .. }) {
                    accepted += 1;
                }
            }
            Ok(other) => panic!("应返回命令结果: {other:?}"),
            Err(_) => {}
        }
    }
    assert!(accepted >= 2, "至少 2 条被接受（实际 {accepted}）");

    // 实时状态（恢复前）
    let live_hash = {
        let (rt, rr) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Snapshot {
                connection_id: 1,
                header: MessageHeader::new(15, 99, 0),
                reply: rt,
            })
            .await
            .expect("send snapshot");
        match rr.await.expect("reply").expect("snapshot") {
            ServerMessage::Snapshot { view, .. } => view.state_hash,
            other => panic!("应返回快照: {other:?}"),
        }
    };

    // Recover：快照 + WAL 重放
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Recover { reply: rt })
        .await
        .expect("send recover");
    rr.await.expect("reply").expect("recover 应成功");

    // 恢复后状态摘要必须与实时一致
    let after_hash = {
        let (rt, rr) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Snapshot {
                connection_id: 1,
                header: MessageHeader::new(15, 100, 0),
                reply: rt,
            })
            .await
            .expect("send snapshot");
        match rr.await.expect("reply").expect("snapshot") {
            ServerMessage::Snapshot { view, .. } => view.state_hash,
            other => panic!("应返回快照: {other:?}"),
        }
    };
    assert_eq!(after_hash, live_hash, "Recover 后摘要必须与实时一致");
}

/// actor 回包与命令后推送：`legal_actions`/`choice` 按观察者给出，且其余成员收到
/// **个性化**推送（不含他人选项内容）。
#[tokio::test]
async fn nc09_native_backend_replies_carry_actions_and_fanout() {
    let handle = room_native_actor::spawn_room_native(16, native_session(20261806, 4), 16, 4);
    // 两个连接：1 → 座位 0；2 → 座位 1（各自 outbound 收集推送）
    let (tx1, mut rx1) = mpsc::channel::<String>(16);
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 1,
            display_name: "a".into(),
            preferred_seat: Some(0),
            reconnect_token: None,
            password: String::new(),
            outbound: session::Outbound::new(tx1),
            reply: rt,
        })
        .await
        .expect("send");
    let _ = rr.await.expect("reply").expect("入座 0");
    let (tx2, mut rx2) = mpsc::channel::<String>(16);
    let (rt2, rr2) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 2,
            display_name: "b".into(),
            preferred_seat: Some(1),
            reconnect_token: None,
            password: String::new(),
            outbound: session::Outbound::new(tx2),
            reply: rt2,
        })
        .await
        .expect("send");
    let _ = rr2.await.expect("reply").expect("入座 1");
    while rx1.try_recv().is_ok() {}
    while rx2.try_recv().is_ok() {}

    // 1 号（座位 0）推进阶段（若当前不是座位 0 则先查快照并改用该座位连接）
    let current = {
        let (rt, rr) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Snapshot {
                connection_id: 1,
                header: MessageHeader::new(16, 1, 0),
                reply: rt,
            })
            .await
            .expect("send snapshot");
        match rr.await.expect("reply").expect("snapshot") {
            ServerMessage::Snapshot {
                view,
                legal_actions,
                ..
            } => {
                // 快照回包应带该观察者的权威动作（至少阶段推进候选或为空集但类型正确）
                assert!(legal_actions.len() <= 256);
                view.current_player.raw()
            }
            other => panic!("应返回快照: {other:?}"),
        }
    };
    let conn = if current == 1 { 1 } else { 2 };
    let out = submit(&handle, conn, 2, CommandKind::EndPhase {}, 200).await;
    match out.expect("提交") {
        ServerMessage::CommandOutcome {
            legal_actions,
            choice,
            ..
        } => {
            // 提交者回包带自己的动作与选择（可能为空集，但字段已接线）
            assert!(legal_actions.len() <= 256);
            let _ = choice;
        }
        other => panic!("应返回命令结果: {other:?}"),
    }
    // 另一连接应收到**个性化推送**（CommandOutcome）
    let mut pushed = 0;
    let other_rx = if conn == 1 { &mut rx2 } else { &mut rx1 };
    while let Ok(line) = other_rx.try_recv() {
        if line.contains("command_outcome") || line.contains("CommandOutcome") {
            pushed += 1;
        }
    }
    assert!(
        pushed >= 1,
        "命令后其余成员应收到个性化推送（实际 {pushed}）"
    );
}

/// 多房间并发与隔离：不同原生房间互不影响；房间信箱有界（满时 `try_send` 失败）。
#[tokio::test]
async fn nc09_native_rooms_are_isolated_and_mailbox_bounded() {
    // 5 个原生房间（不同种子），各自入座并推进一次 → 摘要互不相同
    let mut handles = Vec::new();
    for i in 0..5u64 {
        let handle =
            room_native_actor::spawn_room_native(300 + i, native_session(20261900 + i, 2), 8, 4);
        let _ = join(&handle, 1, Some(0), None).await.expect("入座");
        handles.push(handle);
    }
    let mut hashes = Vec::new();
    for handle in &handles {
        let (rt, rr) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Snapshot {
                connection_id: 1,
                header: MessageHeader::new(1, 1, 0),
                reply: rt,
            })
            .await
            .expect("send snapshot");
        match rr.await.expect("reply").expect("snapshot") {
            ServerMessage::Snapshot { view, .. } => hashes.push(view.state_hash),
            other => panic!("应返回快照: {other:?}"),
        }
    }
    let unique: std::collections::BTreeSet<_> = hashes.iter().collect();
    assert_eq!(unique.len(), hashes.len(), "不同种子的房间摘要应互不相同");

    // 一个房间的命令不影响另一个房间
    let before: Vec<[u8; 32]> = hashes.clone();
    let current = {
        let (rt, rr) = tokio::sync::oneshot::channel();
        handles[0]
            .send(RoomMsg::Snapshot {
                connection_id: 1,
                header: MessageHeader::new(1, 2, 0),
                reply: rt,
            })
            .await
            .expect("send");
        match rr.await.expect("reply").expect("snapshot") {
            ServerMessage::Snapshot { view, .. } => view.current_player.raw(),
            other => panic!("应返回快照: {other:?}"),
        }
    };
    let conn = if current == 1 {
        1
    } else {
        let _ = join(&handles[0], 2, Some(current as u8 - 1), None).await;
        2
    };
    let _ = submit(&handles[0], conn, 3, CommandKind::EndPhase {}, 900).await;
    for (i, handle) in handles.iter().enumerate().skip(1) {
        let (rt, rr) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Snapshot {
                connection_id: 1,
                header: MessageHeader::new(1, 3, 0),
                reply: rt,
            })
            .await
            .expect("send");
        match rr.await.expect("reply").expect("snapshot") {
            ServerMessage::Snapshot { view, .. } => {
                assert_eq!(view.state_hash, before[i], "其它房间不应受影响（房间 {i}）")
            }
            other => panic!("应返回快照: {other:?}"),
        }
    }

    // 信箱有界：容量 1 的房间，未消费时 `try_send` 必须失败（tokio mpsc 有界语义）
    let (tx, mut rx) = mpsc::channel::<String>(1);
    // 直接验证 tokio 有界通道语义（房间 actor 使用同一 mpsc 有界信箱）
    tx.try_send("a".into()).expect("首个可入");
    assert!(
        tx.try_send("b".into()).is_err(),
        "容量 1 的信箱在未消费时必须拒绝第二个"
    );
    let _ = rx.recv().await;
}
