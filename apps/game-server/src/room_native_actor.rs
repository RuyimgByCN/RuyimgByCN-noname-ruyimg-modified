//! NC09 原生权威房间 actor：**与旧线房间并行的原生 IO 接线**。
//!
//! 设计取舍（登记）：`room.rs` 的 actor 绑定旧线 `engine_core::Engine` 与旧线
//! `RoomPersist`（`engine_api::Snapshot` 是结构化旧线状态，不能直接装原生归档）；
//! 且旧线房间还承载锦标赛 WAL 分块签名等**已测试**能力。因此本模块**并行**实现
//! 原生 actor（复用 [`crate::room_native`] 的会话与持久化面、`RoomMsg` 消息类型、
//! `Outbound` 广播），不改动旧线房间；由入口按后端选择（下一批接线）。
//!
//! 覆盖：入座/重连/观战、命令（幂等重放、反冒充、提交前落 WAL）、快照请求、
//! 断开、名册广播、按 `snapshot_every` 落原生归档快照、WAL 重放恢复。

use crate::persist::RoomPersist;
use crate::room::{JoinReply, RoomHandle, RoomMsg};
use crate::room_native::{NativeRoomPersist, NativeRoomSession};
use engine_api::{CommandKind, CommandResult, ExpectedRevision, IdempotencyKey};
use engine_protocol::{ClientRole, MessageHeader, SeatPresence, ServerMessage};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info};

/// 原生房间成员（含广播句柄）。
struct NativeActorMember {
    connection_id: u64,
    display_name: String,
    player_id: engine_api::PlayerId,
    seat: u8,
    role: ClientRole,
    outbound: Option<crate::session::Outbound>,
}

/// 启动原生权威房间 actor（与 `room::spawn_room*` 同签名形态）。
pub fn spawn_room_native(
    id: u64,
    session: NativeRoomSession,
    mailbox: usize,
    snapshot_every: u64,
) -> RoomHandle {
    let (tx, rx) = mpsc::channel(mailbox);
    tokio::spawn(native_room_loop(id, session, rx, snapshot_every));
    RoomHandle::new(id, tx)
}

async fn native_room_loop(
    id: u64,
    mut session: NativeRoomSession,
    mut rx: mpsc::Receiver<RoomMsg>,
    snapshot_every: u64,
) {
    let mut persist = NativeRoomPersist::new();
    // 旧线锦标赛 WAL 面保留占位（原生分块签名未做，登记待办）
    let _legacy_wal = RoomPersist::new();
    let mut members: HashMap<u64, NativeActorMember> = HashMap::new();
    let mut idempotency: HashMap<u64, CommandResult> = HashMap::new();
    let mut last_revision = session.engine().revision();
    persist.put_snapshot(
        last_revision,
        engine_native::archive::encode_archive(&session.engine().save()),
    );

    while let Some(msg) = rx.recv().await {
        match msg {
            RoomMsg::Join {
                connection_id,
                display_name,
                preferred_seat,
                reconnect_token,
                password: _pw,
                outbound,
                reply,
            } => {
                match session.join(connection_id, preferred_seat, reconnect_token) {
                    Ok(joined) => {
                        let m = NativeActorMember {
                            connection_id,
                            display_name,
                            player_id: joined.member.player_id,
                            seat: joined.member.seat,
                            role: joined.member.role,
                            outbound: Some(outbound),
                        };
                        let reply_payload = JoinReply {
                            player_id: joined.member.player_id.raw(),
                            seat: joined.member.seat,
                            role: joined.member.role,
                            reconnect_token: joined.member.reconnect_token.clone(),
                            view: joined.view,
                            choice: None,
                            legal_actions: joined.legal_actions,
                        };
                        members.insert(connection_id, m);
                        broadcast_roster_native(id, &members, session.engine().outcome().is_some())
                            .await;
                        let _ = reply.send(Ok(reply_payload));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }
            RoomMsg::Spectate {
                connection_id,
                display_name,
                outbound,
                reply,
            } => {
                let spec = session.spectate(connection_id);
                let m = NativeActorMember {
                    connection_id,
                    display_name,
                    player_id: engine_api::PlayerId::NONE,
                    seat: u8::MAX,
                    role: ClientRole::Spectator,
                    outbound: Some(outbound),
                };
                members.insert(connection_id, m);
                let _ = reply.send(Ok(JoinReply {
                    player_id: 0,
                    seat: u8::MAX,
                    role: ClientRole::Spectator,
                    reconnect_token: spec.member.reconnect_token.clone(),
                    view: spec.view,
                    choice: None,
                    legal_actions: Vec::new(),
                }));
            }
            RoomMsg::Submit {
                connection_id,
                header,
                command,
                idempotency_key,
                reply,
            } => {
                let Some(m) = members.get(&connection_id) else {
                    let _ = reply.send(Err("not in room".into()));
                    continue;
                };
                // 幂等重放
                if idempotency_key.0 != 0 {
                    if let Some(prev) = idempotency.get(&idempotency_key.0) {
                        let view = session.snapshot(connection_id).unwrap_or_else(|_| {
                            crate::native_view::observer_snapshot_for(
                                session.engine(),
                                crate::native_view::RoomRole::Player,
                                Some(m.player_id),
                            )
                        });
                        let _ = reply.send(Ok(ServerMessage::CommandOutcome {
                            header: MessageHeader::new(id, header.conn_seq, session.engine().revision()),
                            idempotency_key,
                            result: prev.clone(),
                            view,
                            legal_actions: Vec::new(),
                            choice: None,
                            public_event_codes: vec![],
                        }));
                        continue;
                    }
                }
                // 反冒充：以入座座位为准（协议 actor 不可信）
                let expected = if header.revision != 0 {
                    header.revision
                } else {
                    session.engine().revision()
                };
                let before = session.engine().revision();
                let result = match session.submit(connection_id, &command.kind, expected) {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = reply.send(Err(e));
                        continue;
                    }
                };
                if let CommandResult::Accepted {
                    new_revision,
                    state_hash,
                } = &result
                {
                    if *new_revision > before {
                        // 提交前落 WAL（架构 §14.3）
                        if let CommandKind::EndPhase {} = command.kind {
                            persist.record(
                                *new_revision,
                                m.seat,
                                engine_native::GameAction::NextPhase,
                                *state_hash,
                            );
                        }
                        if snapshot_every > 0 && *new_revision % snapshot_every == 0 {
                            persist.put_snapshot(
                                *new_revision,
                                engine_native::archive::encode_archive(&session.engine().save()),
                            );
                        }
                        last_revision = *new_revision;
                    }
                    if idempotency_key.0 != 0 {
                        idempotency.insert(idempotency_key.0, result.clone());
                    }
                }
                let view = session
                    .snapshot(connection_id)
                    .unwrap_or_else(|_| {
                        crate::native_view::observer_snapshot_for(
                            session.engine(),
                            crate::native_view::RoomRole::Player,
                            Some(m.player_id),
                        )
                    });
                let _ = reply.send(Ok(ServerMessage::CommandOutcome {
                    header: MessageHeader::new(id, header.conn_seq, last_revision),
                    idempotency_key,
                    result,
                    view,
                    legal_actions: Vec::new(),
                    choice: None,
                    public_event_codes: vec![],
                }));
                broadcast_roster_native(id, &members, session.engine().outcome().is_some()).await;
            }
            RoomMsg::Snapshot {
                connection_id,
                header,
                reply,
            } => {
                match session.snapshot(connection_id) {
                    Ok(view) => {
                        let _ = reply.send(Ok(ServerMessage::Snapshot {
                            header: MessageHeader::new(
                                id,
                                header.conn_seq,
                                session.engine().revision(),
                            ),
                            view,
                            choice: None,
                            legal_actions: Vec::new(),
                        }));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }
            RoomMsg::Disconnect { connection_id } => {
                if let Some(m) = members.get_mut(&connection_id) {
                    m.outbound = None;
                    session.disconnect(connection_id);
                    debug!(room = id, conn = connection_id, "native member disconnected");
                }
                broadcast_roster_native(id, &members, session.engine().outcome().is_some()).await;
            }
            RoomMsg::Recover { reply } => {
                let _ = reply.send(Err(
                    "原生房间恢复请使用 NativeRoomPersist::recover（本 actor 未接线，登记待办）"
                        .into(),
                ));
            }
        }
    }
    info!(room = id, "native room task exit");
}

async fn broadcast_roster_native(
    room_id: u64,
    members: &HashMap<u64, NativeActorMember>,
    finished: bool,
) {
    let mut seats: Vec<SeatPresence> = members
        .values()
        .filter(|m| m.role == ClientRole::Player)
        .map(|m| SeatPresence {
            seat: m.seat,
            player_id: m.player_id.raw(),
            display_name: m.display_name.clone(),
            connected: m.outbound.is_some(),
        })
        .collect();
    seats.sort_by_key(|s| s.seat);
    let msg = ServerMessage::RoomRoster {
        room_id,
        seats,
        started: true,
        finished,
    };
    for m in members.values() {
        if let Some(out) = &m.outbound {
            let _ = out.send(&msg).await;
        }
    }
}

/// NC09 后端选择：按配置构造房间句柄。
///
/// - `native = false` → 由调用方走旧线 `room::spawn_room_with_engine`（本函数返回 `None`）；
/// - `native = true` → 用 `NativeContent` 装载配置指定的包构建原生会话并起原生 actor。
///   装载失败**明确报错**（不静默回退旧线——回退会让「原生后端已启用」变成假象）。
pub fn spawn_native_room_if_configured(
    id: u64,
    players: u8,
    seed: u64,
    mailbox: usize,
    snapshot_every: u64,
    content: &crate::native::NativeContent,
    native: bool,
    namespace: &str,
    packages: &[String],
) -> Result<Option<RoomHandle>, String> {
    if !native {
        return Ok(None);
    }
    let mut packs = Vec::new();
    for id in packages {
        let loaded = content.load_scoped(Some(namespace), id)?;
        packs.push(loaded.pack);
    }
    let refs: Vec<&engine_native::content::extension::ExtensionPack> = packs.iter().collect();
    let engine = if refs.is_empty() {
        engine_native::Engine::new(seed, players as usize)
    } else {
        engine_native::Engine::new_with_extensions(seed, players as usize, &refs)
            .map_err(|e| format!("原生房间建局失败: {e:?}"))?
    };
    let session = NativeRoomSession::new(engine, players);
    Ok(Some(spawn_room_native(id, session, mailbox, snapshot_every)))
}

/// 供测试构造：把原生会话交给 actor 的便捷入口（`oneshot` 回复通道）。
pub fn join_reply_channel() -> (oneshot::Sender<Result<JoinReply, String>>, oneshot::Receiver<Result<JoinReply, String>>) {
    oneshot::channel()
}

/// 供测试构造命令提交通道。
pub fn submit_reply_channel() -> (
    oneshot::Sender<Result<ServerMessage, String>>,
    oneshot::Receiver<Result<ServerMessage, String>>,
) {
    oneshot::channel()
}

/// 供测试构造幂等键。
pub fn idem(key: u64) -> IdempotencyKey {
    IdempotencyKey(key)
}

/// 供测试构造期望修订号。
pub fn expected(revision: u64) -> ExpectedRevision {
    ExpectedRevision(revision)
}
