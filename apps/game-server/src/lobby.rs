//! Lobby: create/join rooms, connection id allocation.

use crate::config::ServerConfig;
use crate::room::{spawn_room_with_engine, RoomHandle, RoomMsg};
use crate::session::{HandlerError, Session};
use content_runtime::{ActiveAssetInfo, ActiveCatalog, ActiveCatalogPage, ContentLock};
use engine_core::Engine;
use crate::native::{NativeGameRequest, NativeGameSummary, NativePackageInfo};
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
    /// Create lobby.
    pub fn new(
        cfg: ServerConfig,
        engine_template: Engine,
        content_lock: ContentLock,
        content_catalog: ActiveCatalog,
        content_assets: BTreeMap<(String, String, String), ActiveAssetInfo>,
        native: std::sync::Arc<crate::native::NativeContent>,
    ) -> Self {
        let cfg_native_store = cfg.native_room_store.clone();
        Self {
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
            native_room_store: if cfg_native_store.is_empty() {
                None
            } else {
                Some(crate::native_room_store::NativeRoomStore::new(cfg_native_store))
            },
            native,
        }
    }

    /// NC09 启动恢复：扫描落盘房间并按 `pack_refs` 重新装载包恢复房间。
    ///
    /// 返回恢复的房间数。**单个房间恢复失败不阻断启动**，但会记录告警（并保留落盘记录
    /// 以便人工处理）；恢复成功的房间以**原房间号**重新进入房间表，并把 `next_room`
    /// 推进到最大房间号之后（避免新建房间撞号）。
    pub fn restore_native_rooms(&self) -> Result<usize, String> {
        let Some(store) = self.native_room_store.clone() else {
            return Ok(0);
        };
        let records = store.load_all()?;
        if records.is_empty() {
            return Ok(0);
        }
        let mut restored = 0usize;
        let mut max_id = 0u64;
        for record in &records {
            max_id = max_id.max(record.room_id);
            let content = self.native.clone();
            let namespace = self.cfg.native_room_namespace.clone();
            let mut load_pack = move |r: &str| -> Result<
                engine_native::content::extension::ExtensionPack,
                String,
            > {
                // `包名@版本` → 按名在当前命名空间查找并装载
                let name = r.split('@').next().unwrap_or(r);
                let installed = content.installed()?;
                let found = installed
                    .iter()
                    .find(|p| p.package_id == name || p.package_id.ends_with(name))
                    .ok_or_else(|| format!("恢复房间失败：包未安装 {name}"))?;
                let loaded = content.load_scoped(Some(&namespace), &found.package_id)?;
                Ok(loaded.pack)
            };
            match crate::room_native_actor::restore_room_from_disk(
                record,
                &mut load_pack,
                self.cfg.room_mailbox,
                self.cfg.snapshot_every,
                store.clone(),
            ) {
                Ok(handle) => {
                    self.rooms.lock().insert(record.room_id, handle);
                    restored += 1;
                    tracing::info!(room = record.room_id, "原生房间已从落盘恢复");
                }
                Err(e) => {
                    tracing::warn!(room = record.room_id, error = %e, "原生房间恢复失败（保留落盘记录）");
                }
            }
        }
        self.next_room.fetch_max(max_id + 1, Ordering::SeqCst);
        Ok(restored)
    }

    /// T6③：列出已安装原生包。
    pub fn native_packages(&self) -> Result<Vec<NativePackageInfo>, String> {
        self.native.installed()
    }

    /// T6③：安装包 → 装载 → 原生建局（CPU 密集，放阻塞线程池）。
    pub async fn native_game(
        &self,
        req: NativeGameRequest,
    ) -> Result<NativeGameSummary, String> {
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
        match msg {
            ClientMessage::Hello {
                versions,
                display_name,
            } => {
                if versions.network_protocol != 0
                    && versions.network_protocol != NETWORK_PROTOCOL_VERSION
                {
                    // allow 0 as "unspecified"
                    return Err(HandlerError::new(
                        ErrorCode::VersionMismatch,
                        "protocol version mismatch",
                    ));
                }
                if !display_name.is_empty() {
                    session.display_name = sanitize_name(&display_name);
                }
                Ok(())
            }

            ClientMessage::CreateRoom {
                players,
                seed,
                password,
            } => {
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
                    let id = self.next_room.fetch_add(1, Ordering::Relaxed);
                    // NC09：后端选择——原生权威房间（显式启用）或旧线房间。
                    // 原生装载失败**明确报错**，不静默回退（避免「已启用」成假象）。
                    let native_handle = crate::room_native_actor::spawn_native_room_if_configured(
                        id,
                        players,
                        seed,
                        self.cfg.room_mailbox,
                        self.cfg.snapshot_every,
                        &self.native,
                        self.cfg.native_rooms,
                        &self.cfg.native_room_namespace,
                        &self.cfg.native_room_packages,
                        self.native_room_store.clone(),
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
            } => {
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
        _ => HandlerError::new(ErrorCode::Internal, "room error"),
    }
}
