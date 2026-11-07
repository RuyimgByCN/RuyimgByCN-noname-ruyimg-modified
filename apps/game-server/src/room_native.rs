//! NC09 原生权威房间会话：**同一原生会话契约**下的入座/观战/命令/快照/重连。
//!
//! 现状：`room.rs` 的 actor 循环绑定旧线 `engine_core::Engine`。本模块把「房间的
//! 权威面」（引擎、投影、权威动作、开放选择、命令结果、重连）实现为**原生**会话，
//! 使用 [`crate::native_view`] 的协议映射（唯一映射点）；IO 与广播由
//! [`crate::room_native_actor`] 承接，本模块维护会话、归档与命令重放。
//!
//! 与旧线的差异（登记，不隐藏）：命令载荷来自协议 `CommandKind`，本会话只映射
//! 内核可表达的动作（出牌、阶段、选择、主动技、明将），其余明确报错
//! 而不是静默忽略。

use crate::native_view::{legal_actions_for, observer_snapshot_for, RoomRole};
use engine_api::{
    CommandKind, CommandResult, ObserverSnapshot, PlayerId, RejectReason as ApiRejectReason,
};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use engine_protocol::ClientRole;
use std::collections::HashMap;

#[path = "room_native/receipts.rs"]
mod receipts;
use receipts::CommandReceipts;
#[path = "room_native/wal.rs"]
mod wal;
pub use wal::{NativeWalEntry, NativeWalState};
#[path = "room_native/timeout.rs"]
mod timeout;
pub use receipts::NativeCommandReceipt;
pub(crate) use timeout::unix_ms;
pub use timeout::{validate_timeout_secs, NativeTimeoutState};

/// NC09 落盘状态（JSON 可序列化；**不含** IO 资源）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NativeRoomState {
    /// 引擎归档字节（hex）。
    pub snapshot_hex: String,
    /// 座位上限。
    pub max_seats: u8,
    /// 是否已开局。
    pub started: bool,
    /// 历史入座计数（保留存档字段；重连令牌使用随机 UUID）。
    pub next_token: u64,
    /// 下一个幂等键序号（延续，避免重放旧命令）。
    pub next_dedup: u64,
    /// 扩展包精确引用（`installed:内容锁@原生版本`；与原局副本逐一比对）。
    pub pack_refs: Vec<String>,
    /// 原局规范内容锁（R05 v4/v5）；旧档缺省 None，新局 Some（标准局为空锁）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_locked_content: Option<content_runtime::ContentLock>,
    /// 原局包副本；旧档缺省 None，新局 Some（标准局为空集）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frozen_packages: Option<Vec<crate::native::FrozenNativePackage>>,
    /// 成员（含重连令牌）。
    pub members: Vec<NativeMember>,
    /// 按重连身份固定已接受请求；旧档没有历史收据时保持空集。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command_receipts: Vec<NativeCommandReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<NativeTimeoutState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wal: Option<NativeWalState>,
}

/// hex 编码（小写）。
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// hex 解码（长度奇数或非法字符返回 None）。
fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

/// 房间成员（原生会话内）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NativeMember {
    /// 连接 id。
    pub connection_id: u64,
    /// 协议玩家 id（1 基；观战为 NONE）。
    pub player_id: PlayerId,
    /// 座位。
    pub seat: u8,
    /// 角色。
    pub role: ClientRole,
    /// 重连令牌。
    pub reconnect_token: String,
}

/// 原生权威房间会话（无 IO）。
pub struct NativeRoomSession {
    engine: Engine,
    /// 建局所用扩展包（Recover 重放需要同内容）。
    packs: Vec<engine_native::content::extension::ExtensionPack>,
    verified_content: Option<content_runtime::VerifiedLockedContent>,
    frozen_packages: Option<Vec<crate::native::FrozenNativePackage>>,
    max_seats: u8,
    members: HashMap<u64, NativeMember>,
    by_token: HashMap<String, u64>,
    started: bool,
    next_token: u64,
    next_dedup: u64,
    command_receipts: CommandReceipts,
    /// 已投影给客户端的 native 事件日志长度（会话数据，不入存档）。
    event_cursor: usize,
    timeout: NativeTimeoutState,
    wal: NativeWalState,
}

/// IO 提交前保存可变会话；包定义与原局资产在 actor 生命周期内不变。
pub(crate) struct NativeRoomCheckpoint {
    engine: Engine,
    members: HashMap<u64, NativeMember>,
    by_token: HashMap<String, u64>,
    started: bool,
    next_token: u64,
    next_dedup: u64,
    command_receipts: CommandReceipts,
    /// 已投影给客户端的 native 事件日志长度（会话数据，不入存档）。
    event_cursor: usize,
    timeout: NativeTimeoutState,
    wal: NativeWalState,
}

/// 入座/观战结果。
#[derive(Debug, Clone)]
pub struct NativeJoinReply {
    /// 成员。
    pub member: NativeMember,
    /// 该观察者的协议快照。
    pub view: ObserverSnapshot,
    /// 该观察者的权威动作。
    pub legal_actions: Vec<engine_api::LegalAction>,
}

impl NativeRoomSession {
    pub(crate) fn checkpoint(&self) -> NativeRoomCheckpoint {
        NativeRoomCheckpoint {
            engine: self.engine.clone(),
            members: self.members.clone(),
            by_token: self.by_token.clone(),
            started: self.started,
            next_token: self.next_token,
            next_dedup: self.next_dedup,
            command_receipts: self.command_receipts.clone(),
            event_cursor: self.event_cursor,
            timeout: self.timeout.clone(),
            wal: self.wal.clone(),
        }
    }

    pub(crate) fn rollback(&mut self, before: NativeRoomCheckpoint) {
        self.engine = before.engine;
        self.engine.clear_staged_presentation_on_rollback();
        self.members = before.members;
        self.by_token = before.by_token;
        self.started = before.started;
        self.next_token = before.next_token;
        self.next_dedup = before.next_dedup;
        self.command_receipts = before.command_receipts;
        self.event_cursor = before.event_cursor;
        self.timeout = before.timeout;
        self.wal = before.wal;
    }

    /// 以原生引擎为模板建会话（模板状态即开局状态）。
    pub fn new(engine: Engine, max_seats: u8) -> Self {
        Self::new_with_packs(engine, Vec::new(), max_seats)
    }

    /// 建会话并保存扩展包集合（`Recover` 重放需要）。
    pub fn new_with_packs(
        mut engine: Engine,
        packs: Vec<engine_native::content::extension::ExtensionPack>,
        max_seats: u8,
    ) -> Self {
        if std::sync::Arc::strong_count(engine.catalog()) == 1
            || (engine.catalog().extension_skill_range.is_empty()
                && engine.catalog().extension_generals.is_empty())
        {
            let _ = content_native_loader::attach_embedded_original_presentation(&mut engine);
        }
        engine.retire_public_presentation();
        let event_cursor = engine.log().len();
        let wal = NativeWalState::new(&engine);
        let next_dedup = engine.save().last_dedup.saturating_add(1);
        Self {
            timeout: NativeTimeoutState {
                choice_timeout_secs: 30,
                managed_seats: vec![],
                deadline: None,
            },
            wal,
            engine,
            packs,
            verified_content: None,
            frozen_packages: None,
            max_seats,
            members: HashMap::new(),
            by_token: HashMap::new(),
            started: true, // 原生引擎建局即已开局（赛前大厅阶段未建模，登记待办）
            next_token: 1,
            next_dedup,
            command_receipts: CommandReceipts::default(),
            event_cursor,
        }
    }

    /// R05 v4/v5：以已验证的 `VerifiedLockedContent` 建会话。
    pub fn new_locked(
        engine: Engine,
        verified: content_runtime::VerifiedLockedContent,
        packs: Vec<engine_native::content::extension::ExtensionPack>,
        max_seats: u8,
    ) -> Self {
        let mut session = Self::new_with_packs(engine, packs, max_seats);
        session.set_verified_content(verified);
        session
    }

    /// 写入已验证的规范内容锁与原局完整包副本。
    pub fn set_verified_content(&mut self, verified: content_runtime::VerifiedLockedContent) {
        self.frozen_packages = Some(verified.copies().to_vec());
        self.verified_content = Some(verified);
    }

    /// 固定原局完整包副本；调用方已逐包校验后，在 spawn 前写入。
    pub fn set_frozen_packages(&mut self, packages: Vec<crate::native::FrozenNativePackage>) {
        self.verified_content =
            content_native_loader::prepare_locked_content_with_bridge(packages.clone()).ok();
        self.frozen_packages = Some(packages);
    }

    /// 只读引擎（测试/宿主查询）。
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn members(&self) -> impl Iterator<Item = &NativeMember> {
        self.members.values()
    }

    /// 建局所用扩展包引用（`Recover` 重放用）。
    pub fn pack_refs(&self) -> Vec<&engine_native::content::extension::ExtensionPack> {
        self.packs.iter().collect()
    }

    /// 用恢复出的引擎替换当前引擎（`Recover` 接线；成员/令牌/座位保持不变）。
    pub fn replace_engine(&mut self, mut engine: Engine) {
        if std::sync::Arc::strong_count(engine.catalog()) == 1
            || (engine.catalog().extension_skill_range.is_empty()
                && engine.catalog().extension_generals.is_empty())
        {
            let _ = content_native_loader::attach_embedded_original_presentation(&mut engine);
        }
        engine.retire_public_presentation();
        // WAL 重放的去重高水位可高于实时计数，下一条命令不能撞到已重放的键。
        self.next_dedup = self
            .next_dedup
            .max(engine.save().last_dedup.saturating_add(1));
        self.event_cursor = engine.log().len();
        self.engine = engine;
    }

    /// I02（裁决 30）：基线响应路径显式退役未读的已封存/已释放展示区间（保留开放段 Held）。
    pub(crate) fn retire_public_presentation(&mut self) {
        self.engine.retire_public_presentation();
    }

    /// I02（裁决 30）：持久化提交成功后释放并一次性消费本次已封存的公开展示批次。
    pub(crate) fn release_and_take_public_presentation(
        &mut self,
    ) -> Option<engine_api::PublicPresentationBatch> {
        self.engine.release_committed_presentation();
        self.engine.take_public_presentation()
    }

    /// 取自上一条命令以来新增的公开事件码（只码，不含牌 id/选项）。
    pub(crate) fn take_public_event_codes(&mut self) -> Vec<u32> {
        let end = self.engine.log().len();
        let start = self.event_cursor.min(end);
        let codes = self
            .engine
            .public_event_codes_from(start)
            .into_iter()
            .map(u32::from)
            .collect();
        self.event_cursor = end;
        codes
    }

    /// 入座（满座或座位被占时返回错误；同令牌为重连）。
    pub fn join(
        &mut self,
        connection_id: u64,
        preferred_seat: Option<u8>,
        reconnect_token: Option<String>,
    ) -> Result<NativeJoinReply, String> {
        // 重连：换绑连接、保留座位与令牌
        if let Some(token) = reconnect_token {
            if let Some(old) = self.by_token.get(&token).copied() {
                if old != connection_id && self.members.contains_key(&connection_id) {
                    return Err("connection already in room".into());
                }
                if let Some(mut m) = self.members.remove(&old) {
                    m.connection_id = connection_id;
                    if m.role == ClientRole::Player {
                        self.timeout.managed_seats.retain(|s| *s != m.seat);
                    }
                    let reply = self.reply_for(&m);
                    self.by_token.insert(token, connection_id);
                    self.members.insert(connection_id, m);
                    return Ok(reply);
                }
            }
            return Err("invalid reconnect token".into());
        }
        if self.members.contains_key(&connection_id) {
            return Err("connection already in room".into());
        }
        let seat = preferred_seat
            .filter(|s| *s < self.max_seats && !self.seat_taken(*s))
            .or_else(|| (0..self.max_seats).find(|s| !self.seat_taken(*s)))
            .ok_or_else(|| "room full".to_string())?;
        let token = uuid::Uuid::new_v4().to_string();
        self.next_token += 1;
        let member = NativeMember {
            connection_id,
            player_id: PlayerId::new(seat as u32 + 1),
            seat,
            role: ClientRole::Player,
            reconnect_token: token.clone(),
        };
        self.by_token.insert(token, connection_id);
        let reply = self.reply_for(&member);
        self.members.insert(connection_id, member);
        Ok(reply)
    }

    /// 观战（公开投影；无座位、无权威动作）。
    pub fn spectate(&mut self, connection_id: u64) -> NativeJoinReply {
        let member = NativeMember {
            connection_id,
            player_id: PlayerId::NONE,
            seat: u8::MAX,
            role: ClientRole::Spectator,
            reconnect_token: format!("nc09-spec-{connection_id}"),
        };
        let reply = self.reply_for(&member);
        self.members.insert(connection_id, member);
        reply
    }

    /// 断开（保留成员与令牌，供重连）。
    /// 座位上限（NC09 惰性缺席回收判据）。
    pub fn max_seats(&self) -> u8 {
        self.max_seats
    }

    /// NC09 缺席座位回收：把某成员从名册与令牌表中**彻底移除**（座位释放）。
    ///
    /// 与 `disconnect` 的差别：`disconnect` 只清 outbound（保留座位与令牌，供重连），
    /// 本方法用于「长时间缺席」的座位回收——**令牌随之失效**（调用方须先广播名册）。
    /// 返回被移除的成员（不存在则 None）。
    pub fn reclaim_seat(&mut self, connection_id: u64) -> Option<NativeMember> {
        let m = self.members.remove(&connection_id)?;
        self.by_token.remove(&m.reconnect_token);
        Some(m)
    }

    pub fn disconnect(&mut self, connection_id: u64) {
        if let Some(m) = self.members.get(&connection_id).cloned() {
            self.by_token
                .insert(m.reconnect_token.clone(), connection_id);
        }
    }

    /// NC05：正式选将（按入座座位；观战者拒绝）。返回 `(revision, state_hash_hex)`。
    ///
    /// 注意：内核 `assign_general_named` **不推进 revision**（状态变化体现在 state_hash），
    /// 调用方（客户端/房间）必须显式失效基于 revision 的缓存。
    pub fn assign_general(
        &mut self,
        connection_id: u64,
        name: &str,
    ) -> Result<(u64, String), String> {
        let m = self
            .members
            .get(&connection_id)
            .cloned()
            .ok_or("unknown connection")?;
        if m.role != ClientRole::Player {
            return Err("spectator cannot choose general".into());
        }
        self.engine.retire_public_presentation();
        self.engine
            .assign_general_named(m.seat, name)
            .map_err(|e| format!("选将被拒绝: {e:?}"))?;
        self.engine.retire_public_presentation();
        self.timeout.managed_seats.retain(|s| *s != m.seat);
        self.wal = NativeWalState::new(&self.engine);
        self.sync_deadline()?;
        Ok((
            self.engine.revision(),
            self.engine
                .state_hash()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
        ))
    }

    /// NC09 跨进程/跨重启持久化：导出可落盘的房间状态。
    ///
    /// 内容：引擎归档字节（hex）+ 成员（含**重连令牌**）+ 座位数/开局位/令牌计数
    /// + 扩展包引用及原局完整副本。**不含**连接句柄与 IO 资源。
    pub fn export_state(&self, snapshot_hex: &str) -> NativeRoomState {
        NativeRoomState {
            snapshot_hex: snapshot_hex.to_string(),
            max_seats: self.max_seats,
            started: self.started,
            next_token: self.next_token,
            next_dedup: self.next_dedup,
            pack_refs: self
                .packs
                .iter()
                .map(|p| format!("{}@{}", p.def.name, p.def.version))
                .collect(),
            canonical_locked_content: self
                .verified_content
                .as_ref()
                .map(|v| v.lock().clone()),
            frozen_packages: self.frozen_packages.clone(),
            members: self.members.values().cloned().collect(),
            command_receipts: self.command_receipts.entries(),
            timeout: Some(self.timeout.clone()),
            wal: Some(self.wal.clone()),
        }
    }

    pub(crate) fn last_wal_entry(&self) -> Option<&NativeWalEntry> {
        self.wal.entries.last()
    }

    pub(crate) fn wal_segment_line(&self) -> crate::native_room_store::WalSegmentLine {
        crate::native_room_store::WalSegmentLine {
            revision: self.engine.revision(),
            next_dedup: self.next_dedup,
            snapshot_hex: self.snapshot_hex(),
            entry: self.wal.entries.last().cloned(),
            command_receipts: self.command_receipts.entries(),
            timeout: Some(self.timeout.clone()),
        }
    }

    /// R05 v4/v5：以已验证的 `VerifiedLockedContent` 从落盘状态恢复会话。
    pub fn from_saved_locked(
        state: &NativeRoomState,
        verified: content_runtime::VerifiedLockedContent,
        packs: Vec<engine_native::content::extension::ExtensionPack>,
    ) -> Result<Self, String> {
        Self::from_saved_inner(state, Some(verified), packs)
    }

    /// 从落盘状态恢复会话（引擎由归档字节重建；`packs` 必须与导出时同内容）。
    ///
    /// 恢复后：成员与令牌映射重建（重连令牌可直接命中**原座位**）、`started` 与
    /// 令牌/幂等计数器延续——不得重置为初始值，否则会重复接受旧令牌或重放命令。
    pub fn from_saved(
        state: &NativeRoomState,
        packs: Vec<engine_native::content::extension::ExtensionPack>,
    ) -> Result<Self, String> {
        let verified = match (&state.canonical_locked_content, &state.frozen_packages) {
            (Some(lock), Some(frozen)) => {
                Some(content_native_loader::verify_locked_content_with_bridge(lock, frozen)?)
            }
            (Some(_), None) => {
                return Err("房间状态含 canonical_locked_content 但缺少 frozen_packages".into());
            }
            (None, Some(frozen)) if !frozen.is_empty() || packs.is_empty() => {
                Some(content_native_loader::prepare_locked_content_with_bridge(frozen.clone())?)
            }
            (None, _) => None,
        };
        Self::from_saved_inner(state, verified, packs)
    }

    fn from_saved_inner(
        state: &NativeRoomState,
        verified: Option<content_runtime::VerifiedLockedContent>,
        packs: Vec<engine_native::content::extension::ExtensionPack>,
    ) -> Result<Self, String> {
        let bytes = hex_decode(&state.snapshot_hex).ok_or("快照 hex 解码失败")?;
        let snapshot =
            engine_native::archive::decode_archive(&bytes).map_err(|e| format!("{e:?}"))?;
        let engine = if let Some(ref verified) = verified {
            content_native_loader::restore_locked_engine(&snapshot, verified)
                .map_err(|e| format!("{e:?}"))?
        } else {
            let refs: Vec<&engine_native::content::extension::ExtensionPack> =
                packs.iter().collect();
            let mut e = if refs.is_empty() {
                Engine::restore(&snapshot).map_err(|e| format!("{e:?}"))?
            } else {
                Engine::restore_with_extensions(&snapshot, &refs).map_err(|e| format!("{e:?}"))?
            };
            content_native_loader::attach_embedded_original_presentation(&mut e)?;
            e
        };
        let mut members = HashMap::new();
        let mut by_token = HashMap::new();
        let command_receipts =
            CommandReceipts::restore(&state.command_receipts, engine.revision(), state.max_seats)?;
        let mut seats = std::collections::HashSet::new();
        if state.max_seats as usize != engine.state_core().players.len() {
            return Err("落盘座位数与引擎不一致".into());
        }
        for m in &state.members {
            if m.role == ClientRole::Player
                && (m.seat >= state.max_seats
                    || m.player_id.raw() != m.seat as u32 + 1
                    || !seats.insert(m.seat))
            {
                return Err("落盘成员座位无效或重复".into());
            }
            if members.insert(m.connection_id, m.clone()).is_some()
                || by_token
                    .insert(m.reconnect_token.clone(), m.connection_id)
                    .is_some()
            {
                return Err("落盘成员连接或令牌重复".into());
            }
        }
        let event_cursor = engine.log().len();
        let timeout = state
            .timeout
            .clone()
            .ok_or("missing native timeout metadata")?;
        let wal = state.wal.clone().ok_or("missing native WAL")?;
        let frozen_packages = verified
            .as_ref()
            .map(|v| v.copies().to_vec())
            .or_else(|| state.frozen_packages.clone());
        let restored = Self {
            timeout,
            wal,
            engine,
            packs,
            verified_content: verified,
            frozen_packages,
            max_seats: state.max_seats,
            members,
            by_token,
            started: state.started,
            next_token: state.next_token,
            next_dedup: state.next_dedup,
            command_receipts,
            event_cursor,
        };
        if restored.next_dedup
            != restored
                .engine
                .save()
                .last_dedup
                .checked_add(1)
                .ok_or("dedup exhausted")?
        {
            return Err("invalid next dedup".into());
        }
        restored.validate_timeout()?;
        restored.validate_wal()?;
        Ok(restored)
    }

    /// 当前引擎归档字节（hex）——落盘用。
    pub fn snapshot_hex(&self) -> String {
        hex_encode(&engine_native::archive::encode_archive(&self.engine.save()))
    }

    /// 快照（按角色投影）。
    pub fn snapshot(&self, connection_id: u64) -> Result<ObserverSnapshot, String> {
        let m = self
            .members
            .get(&connection_id)
            .ok_or("unknown connection")?;
        Ok(self.snapshot_for(m))
    }

    /// 提交命令（协议载荷 → 内核命令；不支持形态明确报错）。
    pub fn submit(
        &mut self,
        connection_id: u64,
        kind: &CommandKind,
        expected_revision: u64,
    ) -> Result<CommandResult, String> {
        self.engine.retire_public_presentation();
        let m = self
            .members
            .get(&connection_id)
            .cloned()
            .ok_or("unknown connection")?;
        if m.role != ClientRole::Player {
            return Err("spectator cannot submit".into());
        }
        if expected_revision != self.engine.revision() {
            return Ok(CommandResult::Rejected {
                reason: ApiRejectReason::StaleRevision {
                    current: self.engine.revision(),
                    expected: expected_revision,
                },
            });
        }
        if matches!(kind, CommandKind::ResolveChoice { .. })
            && self.engine.open_choice().map(|choice| choice.observer) != Some(m.seat)
        {
            return Ok(CommandResult::Rejected {
                reason: ApiRejectReason::InvalidActor,
            });
        }
        if let CommandKind::StartGame { player_count, .. } = kind {
            if *player_count as usize != self.engine.state_core().players.len() {
                return Err("StartGame 座位数与已创建的原生对局不一致".into());
            }
            self.timeout.managed_seats.retain(|s| *s != m.seat);
            return Ok(CommandResult::Accepted {
                new_revision: self.engine.revision(),
                state_hash: self.engine.state_hash(),
            });
        }
        let action = self.action_for(kind)?;
        let result = self.submit_native(m.seat, action, expected_revision)?;
        if matches!(result, CommandResult::Accepted { .. }) {
            self.timeout.managed_seats.retain(|s| *s != m.seat);
        }
        Ok(result)
    }

    pub(crate) fn submit_native(
        &mut self,
        seat: u8,
        action: GameAction,
        expected_revision: u64,
    ) -> Result<CommandResult, String> {
        let next = self.next_dedup.checked_add(1).ok_or("dedup exhausted")?;
        let command = GameCommand {
            actor: seat,
            expected_revision,
            dedup_id: self.next_dedup,
            action,
        };
        let before = self.checkpoint();
        let outcome = self.engine.submit(command.clone());
        if let CommandOutcome::Rejected(reason) = outcome {
            return Ok(CommandResult::Rejected {
                reason: map_reject(&reason, self.engine.revision(), expected_revision),
            });
        }
        self.next_dedup = next;
        self.wal
            .entries
            .push(NativeWalEntry::new(command, &self.engine));
        if let Err(error) = self.sync_deadline() {
            self.rollback(before);
            return Err(error);
        }
        Ok(CommandResult::Accepted {
            new_revision: self.engine.revision(),
            state_hash: self.engine.state_hash(),
        })
    }

    /// 协议 token 是候选下标；提交与 WAL 必须保存同一条原生动作。
    pub fn action_for(&self, kind: &CommandKind) -> Result<GameAction, String> {
        Ok(match kind {
            CommandKind::PlayCard {
                card,
                targets,
                card_targets,
            } if card_targets.is_empty() => GameAction::PlayCard {
                card: card.raw(),
                targets: targets
                    .iter()
                    .map(|t| {
                        t.raw()
                            .checked_sub(1)
                            .filter(|s| *s < self.max_seats as u32)
                            .map(|s| s as u8)
                            .ok_or_else(|| "目标座位越界".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            },
            CommandKind::EndPhase => GameAction::NextPhase,
            CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets,
            } if targets.is_empty() && card_targets.is_empty() => GameAction::ActivateSkill {
                skill: u16::try_from(skill.raw()).map_err(|_| "技能编号越界")?,
            },
            CommandKind::RevealGeneral { slot } => GameAction::RevealGeneral { slot: *slot },
            CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            } => {
                let open = self.engine.open_choice().ok_or("没有待应答的选择")?;
                if open.id != choice_id.raw() as u64 || (*cancelled && !selections.is_empty()) {
                    return Err("选择编号或取消载荷无效".into());
                }
                GameAction::Answer {
                    choice_id: choice_id.raw() as u64,
                    selection: if *cancelled {
                        Vec::new()
                    } else {
                        selections
                            .iter()
                            .map(|s| {
                                open.spec
                                    .options
                                    .get(*s as usize)
                                    .copied()
                                    .ok_or_else(|| format!("选择 token 越界: {s}"))
                            })
                            .collect::<Result<Vec<_>, _>>()?
                    },
                }
            }
            other => {
                return Err(format!(
                    "unsupported command kind for native authority: {other:?}"
                ))
            }
        })
    }

    fn seat_taken(&self, seat: u8) -> bool {
        self.members.values().any(|m| m.seat == seat)
    }

    fn snapshot_for(&self, m: &NativeMember) -> ObserverSnapshot {
        match m.role {
            ClientRole::Player => {
                observer_snapshot_for(&self.engine, RoomRole::Player, Some(m.player_id))
            }
            ClientRole::Spectator => observer_snapshot_for(&self.engine, RoomRole::Spectator, None),
            ClientRole::Referee => observer_snapshot_for(&self.engine, RoomRole::Referee, None),
        }
    }

    fn reply_for(&self, m: &NativeMember) -> NativeJoinReply {
        let view = self.snapshot_for(m);
        let legal_actions = if m.role == ClientRole::Player {
            legal_actions_for(&self.engine, m.player_id)
        } else {
            Vec::new()
        };
        NativeJoinReply {
            member: m.clone(),
            view,
            legal_actions,
        }
    }
}

/// 内核拒绝原因 → 协议拒绝原因（协议枚举较小，未覆盖形态归入 `IllegalAction`
/// 并带稳定短码；不得把故障伪装成成功）。
fn map_reject(
    reason: &engine_native::RejectReason,
    current: u64,
    expected: u64,
) -> ApiRejectReason {
    use engine_native::RejectReason as R;
    match reason {
        R::NotYourTurn | R::WrongPlayer | R::UnknownActor => ApiRejectReason::InvalidActor,
        R::StaleRevision => ApiRejectReason::StaleRevision { current, expected },
        R::DuplicateCommand => ApiRejectReason::DuplicateIdempotency,
        R::NoSuchChoice | R::InvalidChoice | R::ChoicePending => ApiRejectReason::InvalidChoice,
        R::MatchOver => ApiRejectReason::MatchOver,
        R::VmFault { skill, detail } => ApiRejectReason::Fault {
            kind: engine_api::FaultKind::ContentFault,
            message: format!("vm fault in {skill}: {detail}"),
        },
        other => ApiRejectReason::IllegalAction {
            code: format!("{other:?}"),
        },
    }
}
