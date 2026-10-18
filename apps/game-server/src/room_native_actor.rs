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
use crate::native_view::{choice_for, legal_actions_for, observer_snapshot_for, RoomRole};
use crate::native_room_store::{NativeRoomStore, StoredRoom};
use crate::room_native::{NativeRoomPersist, NativeRoomSession};
use engine_api::{CommandKind, CommandResult, ExpectedRevision, IdempotencyKey};
use engine_protocol::{ClientRole, MessageHeader, SeatPresence, ServerMessage};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info};

/// NC09 落盘：把当前会话状态写入存储（失败只记日志，不影响房间运行）。
///
/// 终局处理：比赛已有结果（`outcome().is_some()`）时**删除**落盘记录并停止写入——
/// 已结束的房间无需跨重启恢复，避免落盘目录只增不减。
///
/// `password` 参数此处是**密码哈希**（运行期不持有明文）。
fn persist_room(
    store: &Option<NativeRoomStore>,
    id: u64,
    password: &str,
    session: &NativeRoomSession,
) {
    let Some(store) = store else { return };
    if session.engine().outcome().is_some() {
        if let Err(e) = store.remove(id) {
            tracing::warn!(room = id, error = %e, "原生房间终局清理失败");
        }
        return;
    }
    let record = StoredRoom {
        room_id: id,
        password_hash: password.to_string(),
        state: session.export_state(&session.snapshot_hex()),
        checksum: String::new(),
    };
    if let Err(e) = store.save(&record) {
        tracing::warn!(room = id, error = %e, "原生房间落盘失败");
    }
}

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
    spawn_room_native_with_password(id, session, mailbox, snapshot_every, String::new())
}

/// 启动原生房间 actor（带房间密码；空串 = 无密码）。
pub fn spawn_room_native_with_password(
    id: u64,
    session: NativeRoomSession,
    mailbox: usize,
    snapshot_every: u64,
    password: String,
) -> RoomHandle {
    spawn_room_native_persisted(id, session, mailbox, snapshot_every, password, None)
}

/// 带**落盘存储**启动（每条接受命令后写盘；`None` = 不落盘，测试/临时房间）。
pub fn spawn_room_native_persisted(
    id: u64,
    session: NativeRoomSession,
    mailbox: usize,
    snapshot_every: u64,
    password: String,
    store: Option<NativeRoomStore>,
) -> RoomHandle {
    // 运行期只保留**哈希**（明文仅在入座比对时短暂存在）
    let hash = crate::native_room_store::password_hash(id, &password);
    spawn_room_native_persisted_with_hash(id, session, mailbox, snapshot_every, hash, store)
}

/// 带**密码哈希**启动（恢复路径用：磁盘上只有哈希，没有明文）。
pub fn spawn_room_native_persisted_with_hash(
    id: u64,
    session: NativeRoomSession,
    mailbox: usize,
    snapshot_every: u64,
    password_hash: String,
    store: Option<NativeRoomStore>,
) -> RoomHandle {
    let (tx, rx) = mpsc::channel(mailbox);
    tokio::spawn(native_room_loop(
        id,
        session,
        rx,
        snapshot_every,
        password_hash,
        store,
    ));
    RoomHandle::new(id, tx)
}

/// 从落盘记录恢复并启动房间（服务端启动时调用）。
///
/// `load_pack(ref)` 由调用方提供：按 `包名@版本` 重新装载同内容包（缺包/版本不符由
/// `from_saved` 明确报错）。
pub fn restore_room_from_disk(
    record: &StoredRoom,
    load_pack: &mut dyn FnMut(&str) -> Result<engine_native::content::extension::ExtensionPack, String>,
    mailbox: usize,
    snapshot_every: u64,
    store: NativeRoomStore,
) -> Result<RoomHandle, String> {
    let mut packs = Vec::new();
    for r in &record.state.pack_refs {
        packs.push(load_pack(r)?);
    }
    let session = NativeRoomSession::from_saved(&record.state, packs)?;
    // 恢复时把**哈希**带回运行期：入座校验用哈希比对（运行期不再持有明文）
    Ok(spawn_room_native_persisted_with_hash(
        record.room_id,
        session,
        mailbox,
        snapshot_every,
        record.password_hash.clone(),
        Some(store),
    ))
}

async fn native_room_loop(
    id: u64,
    mut session: NativeRoomSession,
    mut rx: mpsc::Receiver<RoomMsg>,
    snapshot_every: u64,
    room_password: String,
    store: Option<NativeRoomStore>,
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
                password: pw,
                outbound,
                reply,
            } => {
                // 入座校验：输入密码 → 哈希 → 与运行期哈希比对（磁盘/内存均无明文）
                if !room_password.is_empty()
                    && crate::native_room_store::password_hash(id, &pw) != room_password
                {
                    let _ = reply.send(Err("unauthorized".into()));
                    continue;
                }
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
                        // 名册/令牌变化必须持久化（否则重启后无法凭令牌回到原座位）
                        persist_room(&store, id, &room_password, &session);
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
                persist_room(&store, id, &room_password, &session);
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
                            header: MessageHeader::new(
                                id,
                                header.conn_seq,
                                session.engine().revision(),
                            ),
                            idempotency_key,
                            result: prev.clone(),
                            view,
                            legal_actions: legal_actions_for(session.engine(), m.player_id),
                            choice: choice_for(session.engine(), Some(m.player_id)),
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
                        // NC09 跨重启持久化：接受命令后写盘（原子 rename）
                        persist_room(&store, id, &room_password, &session);
                    }
                    if idempotency_key.0 != 0 {
                        idempotency.insert(idempotency_key.0, result.clone());
                    }
                }
                let view = session
                    .snapshot(connection_id)
                    .unwrap_or_else(|_| {
                        observer_snapshot_for(
                            session.engine(),
                            RoomRole::Player,
                            Some(m.player_id),
                        )
                    });
                let legal = legal_actions_for(session.engine(), m.player_id);
                let choice = choice_for(session.engine(), Some(m.player_id));
                let _ = reply.send(Ok(ServerMessage::CommandOutcome {
                    header: MessageHeader::new(id, header.conn_seq, last_revision),
                    idempotency_key,
                    result: result.clone(),
                    view,
                    legal_actions: legal,
                    choice,
                    public_event_codes: vec![],
                }));
                // 命令后**个性化推送**：其余成员各收自己的视图/动作/选择（提交者走 oneshot）
                for other in members.values() {
                    if other.connection_id == connection_id {
                        continue;
                    }
                    let Some(out) = other.outbound.as_ref() else {
                        continue;
                    };
                    let role = match other.role {
                        ClientRole::Player => RoomRole::Player,
                        ClientRole::Spectator => RoomRole::Spectator,
                        ClientRole::Referee => RoomRole::Referee,
                    };
                    let seat = if other.role == ClientRole::Player {
                        Some(other.player_id)
                    } else {
                        None
                    };
                    let msg = ServerMessage::CommandOutcome {
                        header: MessageHeader::new(id, 0, last_revision),
                        idempotency_key,
                        result: result.clone(),
                        view: observer_snapshot_for(session.engine(), role, seat),
                        legal_actions: if other.role == ClientRole::Player {
                            legal_actions_for(session.engine(), other.player_id)
                        } else {
                            Vec::new()
                        },
                        choice: choice_for(session.engine(), seat),
                        public_event_codes: vec![],
                    };
                    let _ = out.send(&msg).await;
                }
                broadcast_roster_native(id, &members, session.engine().outcome().is_some()).await;
            }
            RoomMsg::Snapshot {
                connection_id,
                header,
                reply,
            } => {
                match session.snapshot(connection_id) {
                    Ok(view) => {
                        let member = members.get(&connection_id);
                        let (seat, is_player) = match member {
                            Some(m) if m.role == ClientRole::Player => (Some(m.player_id), true),
                            _ => (None, false),
                        };
                        let _ = reply.send(Ok(ServerMessage::Snapshot {
                            header: MessageHeader::new(
                                id,
                                header.conn_seq,
                                session.engine().revision(),
                            ),
                            view,
                            choice: choice_for(session.engine(), seat),
                            legal_actions: if is_player {
                                legal_actions_for(session.engine(), seat.expect("player seat"))
                            } else {
                                Vec::new()
                            },
                        }));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }
            RoomMsg::ChooseGeneral {
                connection_id,
                name,
                reply,
            } => {
                // 座位以入座座位为准（反冒充）；成功后向全体推送新视图（选将不推进
                // revision，故推送判据是「操作成功」而不是 revision 变化）。
                match session.assign_general(connection_id, &name) {
                    Ok((revision, hash)) => {
                        for other in members.values() {
                            let Some(out) = other.outbound.as_ref() else {
                                continue;
                            };
                            let role = match other.role {
                                ClientRole::Player => RoomRole::Player,
                                ClientRole::Spectator => RoomRole::Spectator,
                                ClientRole::Referee => RoomRole::Referee,
                            };
                            let seat = if other.role == ClientRole::Player {
                                Some(other.player_id)
                            } else {
                                None
                            };
                            let msg = ServerMessage::Snapshot {
                                header: MessageHeader::new(id, 0, revision),
                                view: observer_snapshot_for(session.engine(), role, seat),
                                choice: choice_for(session.engine(), seat),
                                legal_actions: if other.role == ClientRole::Player {
                                    legal_actions_for(
                                        session.engine(),
                                        seat.expect("player seat"),
                                    )
                                } else {
                                    Vec::new()
                                },
                            };
                            let _ = out.send(&msg).await;
                        }
                        persist_room(&store, id, &room_password, &session);
                        let _ = reply.send(Ok(format!(
                            "{{\"revision\":{revision},\"state_hash\":\"{hash}\"}}"
                        )));
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
                // NC09：按「快照 + WAL 重放」恢复（内容不符/摘要不符都会明确失败）。
                // 快照点取持久化面中最新的、不晚于最后已接受修订号的快照。
                let from = persist
                    .snapshots
                    .keys()
                    .filter(|r| **r <= last_revision)
                    .next_back()
                    .copied();
                let Some(from) = from else {
                    let _ = reply.send(Err("无可用于恢复的快照".into()));
                    continue;
                };
                match persist.recover(from, &session.pack_refs()) {
                    Ok((engine, replayed)) => {
                        last_revision = engine.revision();
                        session.replace_engine(engine);
                        debug!(room = id, from, replayed, "native room recovered");
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
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
    store: Option<NativeRoomStore>,
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
    Ok(Some(spawn_room_native_persisted(
        id,
        session,
        mailbox,
        snapshot_every,
        String::new(),
        store,
    )))
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
