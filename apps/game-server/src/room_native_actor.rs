//! NC09 原生权威房间 actor：**与旧线房间并行的原生 IO 接线**。
//!
//! 设计取舍（登记）：`room.rs` 的 actor 绑定旧线 `engine_core::Engine` 与旧线
//! `RoomPersist`（`engine_api::Snapshot` 是结构化旧线状态，不能直接装原生归档）；
//! 且旧线房间还承载锦标赛 WAL 分块签名等**已测试**能力。因此本模块**并行**实现
//! 原生 actor（复用 [`crate::room_native`] 的会话与持久化面、`RoomMsg` 消息类型、
//! `Outbound` 广播）；大厅按 `native_rooms` 配置选择原生或旧线房间。
//!
//! 覆盖：入座/重连/观战、命令（幂等重放、反冒充、接受后记 WAL）、快照请求、
//! 断开、名册广播、按 `snapshot_every` 落原生归档快照、WAL 重放恢复。

use crate::native_room_store::{NativeRoomStore, StoredRoom};
use crate::native_view::{legal_actions_for, observer_snapshot_for, RoomRole};
use crate::persist::RoomPersist;
use crate::room::{JoinReply, RoomHandle, RoomMsg};
use crate::room_native::NativeRoomSession;
use crate::session::SendStatus;
use engine_api::{CommandResult, ExpectedRevision, IdempotencyKey};
use engine_protocol::{ClientRole, MessageHeader, SeatPresence, ServerMessage};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info};

/// NC09 缺席座位回收时限（秒；0 = 关闭）。由 `spawn_*` 侧从配置/环境写入，
/// 使 actor 循环不直接读环境变量（生产源码门禁：禁止 env::var）。
pub static SEAT_TIMEOUT_SECS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(600);

/// 设置缺席座位回收时限（秒；0 = 关闭）。
pub fn set_seat_timeout_secs(secs: u64) {
    SEAT_TIMEOUT_SECS.store(secs, std::sync::atomic::Ordering::Relaxed);
}

#[path = "room_native_actor/durability.rs"]
mod durability;
use durability::RoomDurability;
#[path = "room_native_actor/timeout.rs"]
mod timeout;

/// 原生房间成员（含广播句柄）。
struct NativeActorMember {
    connection_id: u64,
    display_name: String,
    player_id: engine_api::PlayerId,
    seat: u8,
    role: ClientRole,
    outbound: Option<crate::session::Outbound>,
    /// NC09 缺席座位回收：该成员最后一次活动的时刻（惰性超时判据）。
    last_seen: std::time::Instant,
}

/// 启动原生权威房间 actor（与 `room::spawn_room*` 同签名形态）。

/// A reconnect replay for which the room has no saved receipt cannot prove the
/// command was not already executed. Reject it instead of risking a duplicate.
fn legacy_replay_rejection(
    session: &NativeRoomSession,
    connection_id: u64,
    player_id: engine_api::PlayerId,
    id: u64,
    conn_seq: u64,
    idempotency_key: IdempotencyKey,
) -> Result<ServerMessage, String> {
    let view = session.snapshot(connection_id).unwrap_or_else(|_| {
        crate::native_view::observer_snapshot_for(
            session.engine(),
            crate::native_view::RoomRole::Player,
            Some(player_id),
        )
    });
    Ok(ServerMessage::CommandOutcome {
        header: MessageHeader::new(id, conn_seq, session.engine().revision()),
        idempotency_key,
        result: CommandResult::Rejected {
            reason: engine_api::RejectReason::IllegalAction {
                code: "legacy_replay_no_receipt".into(),
            },
        },
        view,
        legal_actions: legal_actions_for(session.engine(), player_id),
        choice: session.choice_for(RoomRole::Player, Some(player_id)),
        public_event_codes: vec![],
        presentation: None,
    })
}

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
    load_pack: &mut dyn FnMut(
        &str,
    )
        -> Result<engine_native::content::extension::ExtensionPack, String>,
    mailbox: usize,
    snapshot_every: u64,
    store: NativeRoomStore,
) -> Result<RoomHandle, String> {
    let mut packs = Vec::new();
    for r in &record.state.pack_refs {
        packs.push(load_pack(r)?);
    }
    let session = NativeRoomSession::from_saved(&record.state, packs)?;
    if session.engine().outcome().is_some() {
        return Err("原生房间已结束，不恢复为运行中房间".into());
    }
    store.save(record)?;
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
    // NC09 缺席座位回收（惰性）：入座满员且新客户端要座位时，回收**离线**且超过
    // 该时限的成员（0 = 关闭）。时限来自环境变量 `NONAME_NATIVE_SEAT_TIMEOUT_SECS`
    // （默认 600 秒），由 spawn 侧读取后传入，避免在生产源码里直接读环境变量。
    let seat_timeout_secs = SEAT_TIMEOUT_SECS.load(std::sync::atomic::Ordering::Relaxed);
    let mut durability = RoomDurability::new(id, room_password.clone(), store);
    // 旧线锦标赛 WAL 面保留占位（原生分块签名未做，登记待办）
    let _legacy_wal = RoomPersist::new();
    let mut members: HashMap<u64, NativeActorMember> = session
        .members()
        .map(|m| {
            (
                m.connection_id,
                NativeActorMember {
                    connection_id: m.connection_id,
                    display_name: "player".into(),
                    player_id: m.player_id,
                    seat: m.seat,
                    role: m.role,
                    outbound: None,
                    last_seen: std::time::Instant::now(),
                },
            )
        })
        .collect();
    let mut last_revision = session.engine().revision();
    if let Err(error) = session
        .sync_deadline()
        .and_then(|_| durability.commit(&mut session, None))
    {
        tracing::error!(room = id, %error, "initial choice deadline could not be persisted");
        return;
    }
    let mut automatic_allowed = true;
    let mut paused_revision = None;
    loop {
        let revision = session.engine().revision();
        let now = crate::room_native::unix_ms();
        if paused_revision != Some(revision) {
            match now {
                Ok(now) => {
                    if let Some(candidate) = session.automatic_command(now, automatic_allowed) {
                        if let Err(error) = timeout::automatic_step(
                            id,
                            &mut session,
                            &mut durability,
                            &mut members,
                            snapshot_every,
                            candidate,
                        )
                        .await
                        {
                            tracing::warn!(room = id, %error, "automatic input paused");
                            paused_revision = Some(session.engine().revision());
                        }
                        last_revision = session.engine().revision();
                        automatic_allowed = false;
                        continue;
                    }
                }
                Err(ref error) => {
                    tracing::error!(room = id, %error, "automatic clock unavailable");
                    paused_revision = Some(revision);
                }
            }
        }
        let now = now.ok();
        let managed_ready = now.is_some_and(|now| session.automatic_command(now, true).is_some());
        let wait = if paused_revision == Some(revision) {
            None
        } else if managed_ready {
            Some(std::time::Duration::ZERO)
        } else {
            session.timeout_state().deadline.as_ref().map(|d| {
                std::time::Duration::from_millis(
                    d.deadline_unix_ms
                        .saturating_sub(now.expect("clock checked before timer")),
                )
            })
        };
        // Expiry was checked before removing a message; managed successors yield to queued people.
        let msg = tokio::select! {
            biased;
            _ = async {
                if !managed_ready {
                    match wait {
                        Some(delay) => tokio::time::sleep(delay).await,
                        None => std::future::pending::<()>().await,
                    }
                } else {
                    std::future::pending::<()>().await
                }
            } => {
                automatic_allowed = true;
                continue;
            },
            msg = rx.recv() => match msg {
                Some(msg) => msg,
                None => break,
            },
            _ = async {
                match wait {
                    Some(delay) => tokio::time::sleep(delay).await,
                    None => std::future::pending::<()>().await,
                }
            } => {
                automatic_allowed = true;
                continue;
            }
        };
        automatic_allowed = true;
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
                if !room_password.is_empty()
                    && crate::native_room_store::password_hash(id, &pw) != room_password
                {
                    let _ = reply.send(Err("unauthorized".into()));
                    continue;
                }
                let mut before = match durability.checkpoint(&session) {
                    Ok(before) => before,
                    Err(error) => {
                        let _ = reply.send(Err(error));
                        continue;
                    }
                };
                let mut reclaimed = None;
                // 回收与新入座同一次提交，失败不能丢失原座位或令牌。
                if seat_timeout_secs > 0
                    && reconnect_token.is_none()
                    && !members.contains_key(&connection_id)
                {
                    let idle = std::time::Duration::from_secs(seat_timeout_secs);
                    let now = std::time::Instant::now();
                    let occupied = members
                        .values()
                        .filter(|m| m.role == ClientRole::Player)
                        .count();
                    if occupied >= session.max_seats() as usize {
                        let victim = members
                            .values()
                            .filter(|m| m.role == ClientRole::Player && m.outbound.is_none())
                            .min_by_key(|m| m.last_seen)
                            .map(|m| (m.connection_id, m.seat, m.last_seen));
                        if let Some((conn, seat, seen)) = victim {
                            if now.duration_since(seen) >= idle {
                                if before.is_none() {
                                    before = Some(session.checkpoint());
                                }
                                session.reclaim_seat(conn);
                                reclaimed = Some((conn, seat));
                            }
                        }
                    }
                }
                match session.join(connection_id, preferred_seat, reconnect_token) {
                    Ok(joined) => {
                        paused_revision = None;
                        if let Err(error) = durability.commit(&mut session, before) {
                            let _ = reply.send(Err(error));
                            continue;
                        }
                        if let Some((conn, seat)) = reclaimed {
                            members.remove(&conn);
                            info!(room = id, conn, seat, "原生房间回收缺席座位（惰性超时）");
                        }
                        let m = NativeActorMember {
                            last_seen: std::time::Instant::now(),
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
                            choice: session.choice_for(
                                joined.member.role.into(),
                                Some(joined.member.player_id),
                            ),
                            legal_actions: joined.legal_actions,
                        };
                        members.retain(|connection, _| {
                            session.members().any(|member| member.connection_id == *connection)
                        });
                        members.insert(connection_id, m);
                        broadcast_roster_native(id, &mut members, &session).await;
                        let _ = reply.send(Ok(reply_payload));
                    }
                    Err(e) => {
                        if let Some(before) = before {
                            session.rollback(before);
                        }
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
                if members.contains_key(&connection_id) {
                    let _ = reply.send(Err("connection already in room".into()));
                    continue;
                }
                let before = match durability.checkpoint(&session) {
                    Ok(before) => before,
                    Err(error) => {
                        let _ = reply.send(Err(error));
                        continue;
                    }
                };
                let spec = session.spectate(connection_id);
                if let Err(error) = durability.commit(&mut session, before) {
                    let _ = reply.send(Err(error));
                    continue;
                }
                let m = NativeActorMember {
                    last_seen: std::time::Instant::now(),
                    connection_id,
                    display_name,
                    player_id: engine_api::PlayerId::NONE,
                    seat: u8::MAX,
                    role: ClientRole::Spectator,
                    outbound: Some(outbound),
                };
                members.insert(connection_id, m);
                broadcast_roster_native(id, &mut members, &session).await;
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
                replay,
                reply,
            } => {
                if let Some(m) = members.get_mut(&connection_id) {
                    m.last_seen = std::time::Instant::now();
                }
                let Some(m) = members.get(&connection_id) else {
                    let _ = reply.send(Err("not in room".into()));
                    continue;
                };
                if m.role != ClientRole::Player {
                    let _ = reply.send(Err("spectator cannot submit".into()));
                    continue;
                }
                let checkpoint = match durability.checkpoint(&session) {
                    Ok(before) => before,
                    Err(error) => {
                        let _ = reply.send(Err(error));
                        continue;
                    }
                };
                // 幂等重放
                match session.command_receipt(connection_id, idempotency_key.0, &command.kind) {
                    Ok(Some(prev)) => {
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
                            result: prev,
                            view,
                            legal_actions: legal_actions_for(session.engine(), m.player_id),
                            choice: session.choice_for(m.role.into(), Some(m.player_id)),
                            public_event_codes: vec![],
                            presentation: None,
                        }));
                        continue;
                    }
                    Err(error) => {
                        let _ = reply.send(Err(error));
                        continue;
                    }
                    Ok(None) => {}
                }
                if replay {
                    let _ = reply.send(legacy_replay_rejection(
                        &session,
                        connection_id,
                        m.player_id,
                        id,
                        header.conn_seq,
                        idempotency_key,
                    ));
                    continue;
                }
                // 反冒充：以入座座位为准（协议 actor 不可信）
                let expected = if header.revision != 0 {
                    header.revision
                } else if command.expected_revision.0 != 0 {
                    command.expected_revision.0
                } else {
                    session.engine().revision()
                };
                if header.revision != 0
                    && command.expected_revision.0 != 0
                    && header.revision != command.expected_revision.0
                {
                    let _ = reply.send(Err("conflicting command revisions".into()));
                    continue;
                }
                let before = session.engine().revision();
                let result = match session.submit(connection_id, &command.kind, expected) {
                    Ok(r) => r,
                    Err(e) => {
                        if let Some(before) = checkpoint {
                            session.rollback(before);
                        }
                        let _ = reply.send(Err(e));
                        continue;
                    }
                };
                if let CommandResult::Accepted {
                    new_revision,
                    state_hash: _,
                } = &result
                {
                    if let Err(error) = session.remember_command(
                        connection_id,
                        idempotency_key.0,
                        &command.kind,
                        &result,
                    ) {
                        if let Some(before) = checkpoint {
                            session.rollback(before);
                        }
                        let _ = reply.send(Err(error));
                        continue;
                    }
                    session.compact_wal(snapshot_every);
                    if let Err(error) = durability.commit(&mut session, checkpoint) {
                        let _ = reply.send(Err(error));
                        continue;
                    }
                    paused_revision = None;
                    if *new_revision > before {
                        last_revision = *new_revision;
                    }
                } else if let Some(before) = checkpoint {
                    session.rollback(before);
                }
                let public_event_codes = session.take_public_event_codes();
                let view = session.snapshot(connection_id).unwrap_or_else(|_| {
                    observer_snapshot_for(session.engine(), RoomRole::Player, Some(m.player_id))
                });
                let legal = legal_actions_for(session.engine(), m.player_id);
                let choice = session.choice_for(m.role.into(), Some(m.player_id));
                let _ = reply.send(Ok(ServerMessage::CommandOutcome {
                    header: MessageHeader::new(id, header.conn_seq, last_revision),
                    idempotency_key,
                    result: result.clone(),
                    view,
                    legal_actions: legal,
                    choice,
                    public_event_codes: public_event_codes.clone(),
                    presentation: None,
                }));
                // 命令后**个性化推送**：其余成员各收自己的视图/动作/选择（提交者走 oneshot）。
                // 发送必须有界：慢客户端的队列满时立即标记/移出广播，绝不 await。
                let mut stalled = Vec::new();
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
                        choice: session.choice_for(role, seat),
                        public_event_codes: public_event_codes.clone(),
                        presentation: None,
                    };
                    if out.try_send(&msg) == SendStatus::Full {
                        stalled.push(other.connection_id);
                    }
                }
                drop_stalled(&mut members, stalled);
                broadcast_roster_native(id, &mut members, &session).await;
            }
            RoomMsg::Snapshot {
                connection_id,
                header,
                reply,
            } => match session.snapshot(connection_id) {
                Ok(view) => {
                    let member = members.get(&connection_id);
                    let role = member.map(|m| m.role.into()).unwrap_or(RoomRole::Spectator);
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
                        choice: session.choice_for(role, seat),
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
            },
            RoomMsg::ChooseGeneral {
                connection_id,
                name,
                reply,
            } => {
                // 座位以入座座位为准（反冒充）；成功后向全体推送新视图（选将不推进
                // revision，故推送判据是「操作成功」而不是 revision 变化）。
                let before = match durability.checkpoint(&session) {
                    Ok(before) => before,
                    Err(error) => {
                        let _ = reply.send(Err(error));
                        continue;
                    }
                };
                match session.assign_general(connection_id, &name) {
                    Ok((revision, hash)) => {
                        if let Err(error) = durability.commit(&mut session, before) {
                            let _ = reply.send(Err(error));
                            continue;
                        }

                        paused_revision = None;
                        let mut stalled = Vec::new();
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
                                choice: session.choice_for(role, seat),
                                legal_actions: if other.role == ClientRole::Player {
                                    legal_actions_for(session.engine(), seat.expect("player seat"))
                                } else {
                                    Vec::new()
                                },
                            };
                            if out.try_send(&msg) == SendStatus::Full {
                                stalled.push(other.connection_id);
                            }
                        }
                        drop_stalled(&mut members, stalled);
                        let _ = reply.send(Ok(format!(
                            "{{\"revision\":{revision},\"state_hash\":\"{hash}\"}}"
                        )));
                    }
                    Err(e) => {
                        if let Some(before) = before {
                            session.rollback(before);
                        }
                        let _ = reply.send(Err(e));
                    }
                }
            }

            RoomMsg::Disconnect { connection_id } => {
                if let Some(m) = members.get_mut(&connection_id) {
                    m.outbound = None;
                    m.last_seen = std::time::Instant::now();
                    session.disconnect(connection_id);
                    debug!(
                        room = id,
                        conn = connection_id,
                        "native member disconnected"
                    );
                }
                broadcast_roster_native(id, &mut members, &session).await;
            }
            RoomMsg::Recover { reply } => {
                let before = match durability.checkpoint(&session) {
                    Ok(before) => before,
                    Err(error) => {
                        let _ = reply.send(Err(error));
                        continue;
                    }
                };
                match session.recover_wal() {
                    Ok(()) => {
                        if let Err(error) = durability.commit(&mut session, before) {
                            let _ = reply.send(Err(error));
                            continue;
                        }
                        paused_revision = None;
                        last_revision = session.engine().revision();
                        debug!(room = id, "native room recovered");
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

fn roster_message(
    room_id: u64,
    members: &HashMap<u64, NativeActorMember>,
    session: &NativeRoomSession,
) -> ServerMessage {
    let mut seats: Vec<SeatPresence> = members
        .values()
        .filter(|m| m.role == ClientRole::Player)
        .map(|m| SeatPresence {
            seat: m.seat,
            player_id: m.player_id.raw(),
            display_name: m.display_name.clone(),
            connected: m.outbound.is_some(),
            managed: session.timeout_state().managed_seats.contains(&m.seat),
        })
        .collect();
    seats.sort_by_key(|s| s.seat);
    ServerMessage::RoomRoster {
        room_id,
        seats,
        started: true,
        finished: session.engine().outcome().is_some(),
        choice_timeout_secs: Some(session.timeout_state().choice_timeout_secs),
    }
}

/// 慢连接移出房间广播（但保留座位/令牌，供重连时恢复）。
fn drop_stalled(members: &mut HashMap<u64, NativeActorMember>, stalled: Vec<u64>) {
    for connection_id in stalled {
        if let Some(m) = members.get_mut(&connection_id) {
            if m.outbound.take().is_some() {
                m.last_seen = std::time::Instant::now();
                tracing::warn!(
                    conn = connection_id,
                    "房间广播队列已满，断开慢连接并要求重连同步"
                );
            }
        }
    }
}

async fn broadcast_roster_native(
    room_id: u64,
    members: &mut HashMap<u64, NativeActorMember>,
    session: &NativeRoomSession,
) {
    let msg = roster_message(room_id, members, session);
    let mut stalled = Vec::new();
    for m in members.values() {
        if let Some(out) = &m.outbound {
            if out.try_send(&msg) == SendStatus::Full {
                stalled.push(m.connection_id);
            }
        }
    }
    if stalled.is_empty() {
        return;
    }
    drop_stalled(members, stalled);
    // 已断开的连接不应再出现在名册的 connected=true 中；仅对剩余连接重发。
    let msg = roster_message(room_id, members, session);
    for m in members.values() {
        if let Some(out) = &m.outbound {
            let _ = out.try_send(&msg);
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
    password: String,
    mailbox: usize,
    snapshot_every: u64,
    content: &crate::native::NativeContent,
    native: bool,
    namespace: &str,
    packages: &[String],
    store: Option<NativeRoomStore>,
    choice_timeout_secs: u8,
) -> Result<Option<RoomHandle>, String> {
    if !native {
        return Ok(None);
    }
    let mut packs = Vec::new();
    let mut frozen = Vec::new();
    for id in packages {
        let loaded = content.load_scoped(Some(namespace), id)?;
        packs.push(loaded.pack);
        frozen.push(loaded.frozen);
    }
    let refs: Vec<&engine_native::content::extension::ExtensionPack> = packs.iter().collect();
    let engine = if refs.is_empty() {
        engine_native::Engine::new(seed, players as usize)
    } else {
        engine_native::Engine::new_with_extensions(seed, players as usize, &refs)
            .map_err(|e| format!("原生房间建局失败: {e:?}"))?
    };
    let mut session = NativeRoomSession::new_with_packs(engine, packs, players);
    session.set_frozen_packages(frozen);
    session.set_choice_timeout_secs(choice_timeout_secs)?;
    session.sync_deadline()?;
    let hash = crate::native_room_store::password_hash(id, &password);
    let mut durability = RoomDurability::new(id, hash, store.clone());
    durability.commit(&mut session, None)?;
    Ok(Some(spawn_room_native_persisted(
        id,
        session,
        mailbox,
        snapshot_every,
        password,
        store,
    )))
}

/// 供测试构造：把原生会话交给 actor 的便捷入口（`oneshot` 回复通道）。
pub fn join_reply_channel() -> (
    oneshot::Sender<Result<JoinReply, String>>,
    oneshot::Receiver<Result<JoinReply, String>>,
) {
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

#[cfg(test)]
mod replay_tests {
    use super::*;

    #[test]
    fn replay_without_receipt_is_rejected_not_executed() {
        let mut session = NativeRoomSession::new(engine_native::Engine::new(7, 2), 2);
        session.join(1, Some(0), None).unwrap();
        let before = session.engine().revision();
        let message = legacy_replay_rejection(
            &session,
            1,
            engine_api::PlayerId::new(1),
            9,
            3,
            IdempotencyKey(77),
        )
        .unwrap();
        let ServerMessage::CommandOutcome { result, .. } = message else {
            panic!("expected outcome");
        };
        assert!(matches!(
            result,
            CommandResult::Rejected {
                reason: engine_api::RejectReason::IllegalAction { code }
            } if code == "legacy_replay_no_receipt"
        ));
        assert_eq!(session.engine().revision(), before);
    }
}
