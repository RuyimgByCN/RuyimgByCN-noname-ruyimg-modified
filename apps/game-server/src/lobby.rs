//! Lobby: create/join rooms, connection id allocation.

use crate::config::ServerConfig;
use crate::native::{NativeGameRequest, NativeGameSummary, NativePackageInfo};
use crate::room::{spawn_room_with_engine, RoomHandle, RoomMsg};
use crate::session::{HandlerError, Session};
use content_runtime::{ActiveAssetInfo, ActiveCatalog, ActiveCatalogPage, ContentLock};
use engine_core::Engine;
use engine_protocol::{
    ClientMessage, ErrorCode, MessageHeader, ServerMessage, NETWORK_PROTOCOL_VERSION,
};
use parking_lot::Mutex;
use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// Global lobby state.
pub struct Lobby {
    cfg: ServerConfig,
    next_conn: AtomicU64,
    next_room: AtomicU64,
    rooms: Mutex<HashMap<u64, RoomHandle>>,
    total_connections: AtomicU64,
    total_commands: AtomicU64,
    rejected_commands: AtomicU64,
    outbox_pending: AtomicU64,
    outbox_delivered: AtomicU64,
    engine_template: Engine,
    content_lock: ContentLock,
    content_catalog: ActiveCatalog,
    content_assets: BTreeMap<(String, String, String), ActiveAssetInfo>,
    native: std::sync::Arc<crate::native::NativeContent>,
    /// NC09：原生房间落盘存储（`None` = 不落盘）。
    native_room_store: Option<crate::native_room_store::NativeRoomStore>,
    /// P1：目录单写者锁；随 Lobby 存活，进程退出/崩溃时由 OS 释放。
    _native_store_lock: Option<crate::native_room_store::StoreWriterLock>,
}

/// Point-in-time lobby statistics for Prometheus export (§40.2).
pub struct LobbyStats {
    /// Active room count.
    pub active_rooms: u64,
    /// Total connections ever accepted.
    pub total_connections: u64,
    /// Total commands processed.
    pub total_commands: u64,
    /// Total commands rejected.
    pub rejected_commands: u64,
    /// Pending outbox messages.
    pub outbox_pending: u64,
    /// Delivered outbox messages.
    pub outbox_delivered: u64,
}

impl Lobby {
    /// Create lobby. 配置了存档目录时先取得排他写者锁，第二写者拒绝启动。
    pub fn new(
        cfg: ServerConfig,
        engine_template: Engine,
        content_lock: ContentLock,
        content_catalog: ActiveCatalog,
        content_assets: BTreeMap<(String, String, String), ActiveAssetInfo>,
        native: std::sync::Arc<crate::native::NativeContent>,
    ) -> Result<Self, String> {
        let cfg_native_store = cfg.native_room_store.clone();
        let (native_room_store, native_store_lock) = if cfg_native_store.is_empty() {
            (None, None)
        } else {
            let lock = crate::native_room_store::StoreWriterLock::acquire(&cfg_native_store)?;
            let store = crate::native_room_store::NativeRoomStore::new(cfg_native_store);
            (Some(store), Some(lock))
        };
        Ok(Self {
            cfg,
            next_conn: AtomicU64::new(1),
            next_room: AtomicU64::new(1),
            rooms: Mutex::new(HashMap::new()),
            total_connections: AtomicU64::new(0),
            total_commands: AtomicU64::new(0),
            rejected_commands: AtomicU64::new(0),
            outbox_pending: AtomicU64::new(0),
            outbox_delivered: AtomicU64::new(0),
            engine_template,
            content_lock,
            content_catalog,
            content_assets,
            native_room_store,
            _native_store_lock: native_store_lock,
            native,
        })
    }

    /// NC09 启动恢复：按精确引用校验原局副本，旧精确档补齐已验证副本。
    ///
    /// 返回恢复的房间数。**单个房间恢复失败不阻断启动**，但会记录告警（并保留落盘记录
    /// 以便人工处理）；恢复成功的房间以**原房间号**重新进入房间表，并把 `next_room`
    /// 推进到最大房间号之后（避免新建房间撞号）。
    pub fn restore_native_rooms(&self) -> Result<usize, String> {
        let Some(store) = self.native_room_store.clone() else {
            return Ok(0);
        };
        let scan = match store.scan() {
            Ok(scan) => scan,
            Err(error) => {
                // 未能扫描旧档时禁止分配房号，避免后续可写时覆盖尚未发现的房间。
                self.next_room.store(0, Ordering::SeqCst);
                return Err(error);
            }
        };
        if let Some(max_id) = scan.reserved_ids.last() {
            let next = max_id.checked_add(1).unwrap_or(0);
            if next == 0 {
                self.next_room.store(0, Ordering::SeqCst);
            } else if self.next_room.load(Ordering::SeqCst) != 0 {
                self.next_room.fetch_max(next, Ordering::SeqCst);
            }
        }
        for issue in scan.issues {
            tracing::warn!(path = %issue.path.display(), room = ?issue.room_id, error = %issue.error,
                uncommitted_temp = issue.uncommitted_temp, "原生房间记录未恢复（保留原文件与房号）");
        }
        let records = scan.records;
        if records.is_empty() {
            return Ok(0);
        }
        let mut restored = 0usize;
        for record in &records {
            if let Some(last) = record.state.members.iter().map(|m| m.connection_id).max() {
                self.next_conn
                    .fetch_max(last.saturating_add(1), Ordering::SeqCst);
            }
            let verified = match self.native.verify_room_locked_content(
                &record.state.pack_refs,
                record.state.canonical_locked_content.as_ref(),
                record.state.frozen_packages.as_deref(),
            ) {
                Ok(verified) => verified,
                Err(e) => {
                    tracing::warn!(room = record.room_id, error = %e, "原生房间包恢复失败（保留落盘记录）");
                    continue;
                }
            };
            let mut packs = Vec::with_capacity(verified.copies().len());
            let mut pack_err = None;
            for copy in verified.copies() {
                match self.native.load_frozen(copy) {
                    Ok(loaded) => packs.push(loaded.pack),
                    Err(e) => {
                        pack_err = Some(e);
                        break;
                    }
                }
            }
            if let Some(e) = pack_err {
                tracing::warn!(room = record.room_id, error = %e, "原生房间包恢复失败（保留落盘记录）");
                continue;
            }
            let mut prepared = record.clone();
            prepared.state.canonical_locked_content = Some(verified.lock().clone());
            prepared.state.frozen_packages = Some(verified.copies().to_vec());
            let session = match crate::room_native::NativeRoomSession::from_saved_locked(
                &prepared.state,
                verified,
                packs,
            ) {
                Ok(session) => session,
                Err(e) => {
                    tracing::warn!(room = record.room_id, error = %e, "原生房间恢复失败（保留落盘记录）");
                    continue;
                }
            };
            if session.engine().outcome().is_some() {
                // 终局记录可能因清理失败而留下，不能把已结束的对局重新发布。
                if let Err(e) = store.remove(record.room_id) {
                    tracing::warn!(room = record.room_id, error = %e, "原生终局记录清理失败（保留记录）");
                }
                continue;
            }
            if let Err(e) = store.save(&prepared) {
                tracing::warn!(room = record.room_id, error = %e, "原生房间恢复写入失败（不发布房间）");
                continue;
            }
            let handle = crate::room_native_actor::spawn_room_native_persisted_with_hash(
                record.room_id,
                session,
                self.cfg.room_mailbox,
                self.cfg.snapshot_every,
                record.password_hash.clone(),
                Some(store.clone()),
            );
            self.rooms.lock().insert(record.room_id, handle);
            restored += 1;
            tracing::info!(room = record.room_id, "原生房间已从落盘恢复");
        }
        Ok(restored)
    }

    #[cfg(test)]
    pub(crate) fn native_room_store_for_test(
        &self,
    ) -> Option<crate::native_room_store::NativeRoomStore> {
        self.native_room_store.clone()
    }

    /// T6③：列出已安装原生包。
    pub fn native_packages(&self) -> Result<Vec<NativePackageInfo>, String> {
        self.native.installed()
    }

    /// 正式生命周期：重新扫描 drop-in 并安装/升级。
    pub fn native_import(&self) -> Result<Vec<NativePackageInfo>, String> {
        self.native.import()
    }

    /// 正式生命周期：激活指定已安装版本。
    pub fn native_activate(
        &self,
        namespace: &str,
        package_id: &str,
        version: &str,
    ) -> Result<(), String> {
        self.native.activate_version(namespace, package_id, version)
    }

    /// 正式生命周期：停用（保留安装副本），可再激活。
    pub fn native_disable(&self, namespace: &str, package_id: &str) -> Result<(), String> {
        self.native.disable_package(namespace, package_id)
    }

    /// 正式生命周期：物理卸载所有版本。
    pub fn native_uninstall(&self, namespace: &str, package_id: &str) -> Result<(), String> {
        self.native.uninstall_package(namespace, package_id)
    }

    /// T6③：安装包 → 装载 → 原生建局（CPU 密集，放阻塞线程池）。
    pub async fn native_game(&self, req: NativeGameRequest) -> Result<NativeGameSummary, String> {
        let native = self.native.clone();
        tokio::task::spawn_blocking(move || crate::native::create_native_game(&native, &req))
            .await
            .map_err(|e| format!("原生建局任务失败: {e}"))?
    }

    /// 已安装扩展进入新对局，返回可恢复的存档。
    pub async fn native_match(
        &self,
        packages: Vec<String>,
        seed: u64,
        seats: u8,
    ) -> Result<crate::native::NativeMatchArchive, String> {
        let native = self.native.clone();
        tokio::task::spawn_blocking(move || {
            crate::native::open_native_match(&native, &packages, seed, seats)
        })
        .await
        .map_err(|e| format!("原生对局任务失败: {e}"))?
    }

    /// 用同一安装集恢复对局。缺包或错版由装载器/内容锁拒绝。
    pub async fn native_restore(
        &self,
        req: crate::native::NativeRestoreRequest,
    ) -> Result<crate::native::NativeRestoreSummary, String> {
        let native = self.native.clone();
        tokio::task::spawn_blocking(move || crate::native::restore_native_match(&native, &req))
            .await
            .map_err(|e| format!("原生恢复任务失败: {e}"))?
    }

    /// Return one bounded page from the immutable startup content catalog.
    pub fn content_catalog_page(
        &self,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> ActiveCatalogPage {
        self.content_catalog
            .page(&self.content_lock.lock_hash, offset, limit)
    }

    /// Canonical hash of the immutable startup content set.
    pub fn content_lock_hash(&self) -> &str {
        &self.content_lock.lock_hash
    }

    /// Return immutable store metadata for an active package asset.
    pub fn active_asset(
        &self,
        namespace: &str,
        package_id: &str,
        asset_id: &str,
    ) -> Option<ActiveAssetInfo> {
        self.content_assets
            .get(&(
                namespace.to_owned(),
                package_id.to_owned(),
                asset_id.to_owned(),
            ))
            .cloned()
    }

    /// Collect point-in-time stats for Prometheus export (§40.2).
    pub fn stats(&self) -> LobbyStats {
        LobbyStats {
            active_rooms: self.rooms.lock().len() as u64,
            total_connections: self.total_connections.load(Ordering::Relaxed),
            total_commands: self.total_commands.load(Ordering::Relaxed),
            rejected_commands: self.rejected_commands.load(Ordering::Relaxed),
            outbox_pending: self.outbox_pending.load(Ordering::Relaxed),
            outbox_delivered: self.outbox_delivered.load(Ordering::Relaxed),
        }
    }

    /// List active rooms for the /rooms endpoint.
    pub fn room_list(&self) -> Vec<serde_json::Value> {
        let rooms = self.rooms.lock();
        rooms
            .keys()
            .map(|id| {
                serde_json::json!({
                    "room_id": id,
                })
            })
            .collect()
    }

    /// Allocate connection id.
    pub fn alloc_connection_id(&self) -> u64 {
        self.next_conn.fetch_add(1, Ordering::Relaxed)
    }

    /// Handle one client message.
    pub async fn handle_client(
        &self,
        session: &mut Session,
        msg: ClientMessage,
    ) -> Result<(), HandlerError> {
        if !matches!(&msg, ClientMessage::Hello { .. }) && !session.handshake_complete {
            return Err(HandlerError::new(
                ErrorCode::VersionMismatch,
                "WS7 Hello required",
            ));
        }
        let header = match &msg {
            ClientMessage::SubmitCommand { header, .. }
            | ClientMessage::RequestSnapshot { header, .. }
            | ClientMessage::Heartbeat { header } => Some(header),
            _ => None,
        };
        if header.is_some_and(|h| h.protocol_version != NETWORK_PROTOCOL_VERSION) {
            return Err(HandlerError::new(
                ErrorCode::VersionMismatch,
                "protocol version mismatch",
            ));
        }
        match msg {
            ClientMessage::Hello {
                versions,
                display_name,
            } => {
                if versions.network_protocol != NETWORK_PROTOCOL_VERSION {
                    return Err(HandlerError::new(
                        ErrorCode::VersionMismatch,
                        "protocol version mismatch",
                    ));
                }
                session.handshake_complete = true;
                if !display_name.is_empty() {
                    session.display_name = sanitize_name(&display_name);
                }
                Ok(())
            }

            ClientMessage::CreateRoom {
                players,
                seed,
                password,
                choice_timeout_secs,
            } => {
                let secs = choice_timeout_secs.unwrap_or(30);
                crate::room_native::validate_timeout_secs(secs)
                    .map_err(|e| HandlerError::new(ErrorCode::BadMessage, e))?;
                if !self.cfg.native_rooms && secs != 30 {
                    return Err(HandlerError::new(
                        ErrorCode::BadMessage,
                        "legacy rooms do not support choice timeouts",
                    ));
                }
                if !(2..=8).contains(&players) {
                    return Err(HandlerError::new(
                        ErrorCode::BadMessage,
                        "players must be 2..=8",
                    ));
                }
                let room_id = {
                    let mut rooms = self.rooms.lock();
                    if rooms.len() >= self.cfg.max_rooms {
                        return Err(HandlerError::new(
                            ErrorCode::RateLimited,
                            "server room capacity",
                        ));
                    }
                    let id = self
                        .next_room
                        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                            (id != 0).then(|| id.checked_add(1).unwrap_or(0))
                        })
                        .map_err(|_| {
                            HandlerError::new(
                                ErrorCode::Internal,
                                "room storage unavailable or room IDs exhausted",
                            )
                        })?;
                    // NC09：后端选择——原生权威房间（显式启用）或旧线房间。
                    // 原生装载失败**明确报错**，不静默回退（避免「已启用」成假象）。
                    let native_handle = crate::room_native_actor::spawn_native_room_if_configured(
                        id,
                        players,
                        seed,
                        password.clone(),
                        self.cfg.room_mailbox,
                        self.cfg.snapshot_every,
                        &self.native,
                        self.cfg.native_rooms,
                        &self.cfg.native_room_namespace,
                        &self.cfg.native_room_packages,
                        self.native_room_store.clone(),
                        secs,
                    )
                    .map_err(|e| HandlerError::new(ErrorCode::Internal, e))?;
                    let handle = match native_handle {
                        Some(h) => h,
                        None => spawn_room_with_engine(
                            id,
                            players,
                            seed,
                            password.clone(),
                            self.cfg.room_mailbox,
                            self.cfg.snapshot_every,
                            self.engine_template.clone(),
                        ),
                    };
                    rooms.insert(id, handle);
                    id
                }; // MutexGuard dropped here before any await
                info!(room = room_id, players, "room created");
                let _ = session
                    .outbound
                    .send(&ServerMessage::RoomCreated {
                        room_id,
                        host_hint: session.display_name.clone(),
                    })
                    .await;
                self.join_room(session, room_id, password, Some(0), None)
                    .await
            }

            ClientMessage::JoinRoom {
                room_id,
                password,
                preferred_seat,
                reconnect_token,
            } => {
                self.join_room(session, room_id, password, preferred_seat, reconnect_token)
                    .await
            }

            ClientMessage::Spectate { room_id } => {
                let handle = {
                    self.rooms.lock().get(&room_id).cloned().ok_or_else(|| {
                        HandlerError::new(ErrorCode::RoomNotFound, "room not found")
                    })?
                };
                let (tx, rx) = tokio::sync::oneshot::channel();
                handle
                    .send(RoomMsg::Spectate {
                        connection_id: session.connection_id,
                        display_name: session.display_name.clone(),
                        outbound: session.outbound.clone(),
                        reply: tx,
                    })
                    .await
                    .map_err(|e| HandlerError::new(ErrorCode::Internal, e))?;
                let jr = rx
                    .await
                    .map_err(|_| HandlerError::new(ErrorCode::Internal, "room reply lost"))?
                    .map_err(|e| map_room_err(&e))?;
                session.room_id = Some(room_id);
                session.player_id = Some(jr.player_id);
                session.role = Some(jr.role);
                session.reconnect_token = Some(jr.reconnect_token.clone());
                let _ = session
                    .outbound
                    .send(&ServerMessage::JoinOk {
                        room_id,
                        player_id: jr.player_id,
                        seat: jr.seat,
                        role: jr.role,
                        reconnect_token: jr.reconnect_token,
                        view: jr.view,
                        choice: jr.choice,
                        legal_actions: jr.legal_actions,
                    })
                    .await;
                Ok(())
            }

            ClientMessage::SubmitCommand {
                header,
                command,
                idempotency_key,
                replay,
            } => {
                if matches!(
                    command.kind,
                    engine_api::CommandKind::TimeoutAction { .. }
                        | engine_api::CommandKind::AdminAbortMatch { .. }
                ) {
                    return Err(HandlerError::new(
                        ErrorCode::Unauthorized,
                        "internal command is not available to clients",
                    ));
                }
                let room_id = session
                    .room_id
                    .ok_or_else(|| HandlerError::new(ErrorCode::NotInRoom, "not in room"))?;
                if header.room_id != 0 && header.room_id != room_id {
                    return Err(HandlerError::new(ErrorCode::Conflict, "room mismatch"));
                }
                if header.conn_seq != 0 && header.conn_seq <= session.last_conn_seq {
                    // allow equal only for pure retries with same idempotency (room handles)
                    if header.conn_seq < session.last_conn_seq {
                        return Err(HandlerError::new(
                            ErrorCode::Conflict,
                            "conn_seq went backwards",
                        ));
                    }
                }
                session.last_conn_seq = session.last_conn_seq.max(header.conn_seq);

                let handle = {
                    self.rooms.lock().get(&room_id).cloned().ok_or_else(|| {
                        HandlerError::new(ErrorCode::RoomNotFound, "room not found")
                    })?
                };
                let (tx, rx) = tokio::sync::oneshot::channel();
                handle
                    .send(RoomMsg::Submit {
                        connection_id: session.connection_id,
                        header: MessageHeader::new(room_id, header.conn_seq, header.revision),
                        command,
                        idempotency_key,
                        replay,
                        reply: tx,
                    })
                    .await
                    .map_err(|e| HandlerError::new(ErrorCode::Internal, e))?;
                let msg = rx
                    .await
                    .map_err(|_| HandlerError::new(ErrorCode::Internal, "room reply lost"))?
                    .map_err(|e| map_room_err(&e))?;
                let _ = session.outbound.send(&msg).await;
                Ok(())
            }

            ClientMessage::RequestSnapshot { header, .. } => {
                let room_id = session
                    .room_id
                    .ok_or_else(|| HandlerError::new(ErrorCode::NotInRoom, "not in room"))?;
                let handle = {
                    self.rooms.lock().get(&room_id).cloned().ok_or_else(|| {
                        HandlerError::new(ErrorCode::RoomNotFound, "room not found")
                    })?
                };
                let (tx, rx) = tokio::sync::oneshot::channel();
                handle
                    .send(RoomMsg::Snapshot {
                        connection_id: session.connection_id,
                        header: MessageHeader::new(room_id, header.conn_seq, header.revision),
                        reply: tx,
                    })
                    .await
                    .map_err(|e| HandlerError::new(ErrorCode::Internal, e))?;
                let msg = rx
                    .await
                    .map_err(|_| HandlerError::new(ErrorCode::Internal, "room reply lost"))?
                    .map_err(|e| map_room_err(&e))?;
                let _ = session.outbound.send(&msg).await;
                Ok(())
            }

            ClientMessage::Heartbeat { header } => {
                let ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                let _ = session
                    .outbound
                    .send(&ServerMessage::Pong {
                        header: MessageHeader::new(
                            header.room_id,
                            header.conn_seq,
                            header.revision,
                        ),
                        server_time_ms: ms,
                    })
                    .await;
                Ok(())
            }

            ClientMessage::LeaveRoom { room_id } => {
                if session.room_id == Some(room_id) {
                    self.disconnect(room_id, session.connection_id).await;
                    session.room_id = None;
                    session.player_id = None;
                    session.role = None;
                }
                Ok(())
            }
        }
    }

    async fn join_room(
        &self,
        session: &mut Session,
        room_id: u64,
        password: String,
        preferred_seat: Option<u8>,
        reconnect_token: Option<String>,
    ) -> Result<(), HandlerError> {
        let handle = {
            self.rooms
                .lock()
                .get(&room_id)
                .cloned()
                .ok_or_else(|| HandlerError::new(ErrorCode::RoomNotFound, "room not found"))?
        };
        let (tx, rx) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Join {
                connection_id: session.connection_id,
                display_name: session.display_name.clone(),
                preferred_seat,
                reconnect_token,
                password,
                outbound: session.outbound.clone(),
                reply: tx,
            })
            .await
            .map_err(|e| HandlerError::new(ErrorCode::Internal, e))?;
        let jr = rx
            .await
            .map_err(|_| HandlerError::new(ErrorCode::Internal, "room reply lost"))?
            .map_err(|e| map_room_err(&e))?;
        session.room_id = Some(room_id);
        session.player_id = Some(jr.player_id);
        session.role = Some(jr.role);
        session.reconnect_token = Some(jr.reconnect_token.clone());
        let _ = session
            .outbound
            .send(&ServerMessage::JoinOk {
                room_id,
                player_id: jr.player_id,
                seat: jr.seat,
                role: jr.role,
                reconnect_token: jr.reconnect_token,
                view: jr.view,
                choice: jr.choice,
                legal_actions: jr.legal_actions,
            })
            .await;
        Ok(())
    }

    /// Notify room of disconnect.
    pub async fn disconnect(&self, room_id: u64, connection_id: u64) {
        let h = self.rooms.lock().get(&room_id).cloned();
        if let Some(h) = h {
            let _ = h.send(RoomMsg::Disconnect { connection_id }).await;
        }
    }
}

fn sanitize_name(s: &str) -> String {
    let s: String = s
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(24)
        .collect();
    if s.is_empty() {
        "player".into()
    } else {
        s
    }
}

fn map_room_err(e: &str) -> HandlerError {
    match e {
        "unauthorized" => HandlerError::new(ErrorCode::Unauthorized, "bad password"),
        "room full" | "seat taken" => HandlerError::new(ErrorCode::RoomFull, e),
        "not in room" => HandlerError::new(ErrorCode::NotInRoom, e),
        // 座位未满/对局未开始时提交命令：属**状态冲突**（客户端时序问题），
        // 不应报成 `Internal`（那是服务端故障的语义）。
        "match not started" => HandlerError::new(ErrorCode::Conflict, "match not started"),
        // NC09/NC12：未知房间错误**带上原文**（此前统一成 "room error"，联机排障时看不到原因；
        // 已用 RTT 探针复现过「只看到 room error」的情况，见 2026-10-18-nc12-online-rtt 证据）。
        _ => HandlerError::new(ErrorCode::Internal, format!("room error: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_room_store::{NativeRoomStore, StoredRoom};
    use crate::room_native::NativeRoomSession;

    #[tokio::test]
    async fn finished_saved_room_is_not_published_and_its_id_remains_reserved() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = ServerConfig {
            original_content_root: dir.path().join("original"),
            content_store_root: dir.path().join("content-store"),
            extensions_root: dir.path().join("extensions"),
            native_packages_root: dir.path().join("packages"),
            native_store_root: dir.path().join("native-store"),
            native_room_store: dir.path().join("rooms").to_string_lossy().into_owned(),
            native_rooms: true,
            ..ServerConfig::default()
        };
        for path in [
            &cfg.original_content_root,
            &cfg.extensions_root,
            &cfg.native_packages_root,
        ] {
            std::fs::create_dir_all(path).unwrap();
        }
        let store = NativeRoomStore::new(&cfg.native_room_store);
        let mut finished = engine_native::Engine::new(42, 2).save();
        // 只构造已通过引擎恢复校验的终局存档，测试启动发布而非胜负规则。
        finished.core.outcome = Some(engine_native::state::Outcome {
            winner: engine_native::state::Winner::Player(0),
        });
        let finished = engine_native::Engine::restore(&finished).unwrap();
        assert!(finished.outcome().is_some());
        for (id, engine) in [(3, engine_native::Engine::new(41, 2)), (7, finished)] {
            let mut session = NativeRoomSession::new(engine, 2);
            session.set_frozen_packages(Vec::new());
            store
                .save(&StoredRoom {
                    room_id: id,
                    password_hash: String::new(),
                    state: session.export_state(&session.snapshot_hex()),
                    checksum: String::new(),
                })
                .unwrap();
        }
        let lobby = crate::bootstrap(&cfg);
        assert_eq!(lobby.restore_native_rooms().unwrap(), 1);
        let rooms = lobby.rooms.lock();
        assert!(rooms.contains_key(&3));
        assert!(!rooms.contains_key(&7));
        assert_eq!(lobby.next_room.load(Ordering::SeqCst), 8);
    }
}
