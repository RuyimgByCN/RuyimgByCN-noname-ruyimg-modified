//! NC09 原生权威房间会话：**同一原生会话契约**下的入座/观战/命令/快照/重连。
//!
//! 现状：`room.rs` 的 actor 循环绑定旧线 `engine_core::Engine`。本模块把「房间的
//! 权威面」（引擎、投影、权威动作、开放选择、命令结果、重连）实现为**原生**会话，
//! 使用 [`crate::native_view`] 的协议映射（唯一映射点），供 actor 循环后续替换接入；
//! 本模块自身不含 tokio/持久化/广播（下一批接线）。
//!
//! 与旧线的差异（登记，不隐藏）：命令载荷来自协议 `CommandKind`，本会话只映射
//! 内核可表达的子集（`PlayCard`/`NextPhase`/`Answer`），其余返回 `Unsupported`
//! 而不是静默忽略。

use crate::native_view::{legal_actions_for, observer_snapshot_for, RoomRole};
use engine_api::{
    CommandKind, CommandResult, ObserverSnapshot, PlayerId, RejectReason as ApiRejectReason,
};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use engine_protocol::ClientRole;
use std::collections::HashMap;

/// 房间成员（原生会话内）。
#[derive(Debug, Clone)]
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
    max_seats: u8,
    members: HashMap<u64, NativeMember>,
    by_token: HashMap<String, u64>,
    started: bool,
    next_token: u64,
    next_dedup: u64,
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
    /// 以原生引擎为模板建会话（模板状态即开局状态）。
    pub fn new(engine: Engine, max_seats: u8) -> Self {
        Self {
            engine,
            max_seats,
            members: HashMap::new(),
            by_token: HashMap::new(),
            started: true, // 原生引擎建局即已开局（赛前大厅阶段未建模，登记待办）
            next_token: 1,
            next_dedup: 1,
        }
    }

    /// 只读引擎（测试/宿主查询）。
    pub fn engine(&self) -> &Engine {
        &self.engine
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
                if let Some(mut m) = self.members.remove(&old) {
                    m.connection_id = connection_id;
                    let reply = self.reply_for(&m);
                    self.by_token.insert(token, connection_id);
                    self.members.insert(connection_id, m);
                    return Ok(reply);
                }
            }
            return Err("invalid reconnect token".into());
        }
        let seat = preferred_seat
            .filter(|s| *s < self.max_seats && !self.seat_taken(*s))
            .or_else(|| (0..self.max_seats).find(|s| !self.seat_taken(*s)))
            .ok_or_else(|| "room full".to_string())?;
        let token = format!("nc09-{}", self.next_token);
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
    pub fn disconnect(&mut self, connection_id: u64) {
        if let Some(m) = self.members.get(&connection_id).cloned() {
            self.by_token.insert(m.reconnect_token.clone(), connection_id);
        }
    }

    /// 快照（按角色投影）。
    pub fn snapshot(&self, connection_id: u64) -> Result<ObserverSnapshot, String> {
        let m = self.members.get(&connection_id).ok_or("unknown connection")?;
        Ok(self.snapshot_for(m))
    }

    /// 提交命令（协议载荷 → 内核命令；不支持形态明确报错）。
    pub fn submit(
        &mut self,
        connection_id: u64,
        kind: &CommandKind,
        expected_revision: u64,
    ) -> Result<CommandResult, String> {
        let m = self.members.get(&connection_id).cloned().ok_or("unknown connection")?;
        if m.role != ClientRole::Player {
            return Err("spectator cannot submit".into());
        }
        let action = match kind {
            CommandKind::PlayCard { card, targets, .. } => GameAction::PlayCard {
                card: card.raw(),
                targets: targets
                    .iter()
                    .map(|t| t.raw().saturating_sub(1) as u8)
                    .collect(),
            },
            CommandKind::EndPhase {} => GameAction::NextPhase,
            CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            } => GameAction::Answer {
                choice_id: choice_id.raw() as u64,
                selection: if *cancelled {
                    Vec::new()
                } else {
                    selections
                        .iter()
                        .map(|s| engine_native::ChoiceOption::Tag(*s as u8))
                        .collect()
                },
            },
            other => {
                return Err(format!("unsupported command kind for native authority: {other:?}"))
            }
        };
        let dedup = self.next_dedup;
        self.next_dedup += 1;
        let outcome = self.engine.submit(GameCommand {
            actor: m.seat,
            expected_revision,
            dedup_id: dedup,
            action,
        });
        Ok(match outcome {
            CommandOutcome::Accepted { state_hash } => CommandResult::Accepted {
                new_revision: self.engine.revision(),
                state_hash,
            },
            CommandOutcome::Pending { .. } | CommandOutcome::Suspended => {
                CommandResult::Accepted {
                    new_revision: self.engine.revision(),
                    state_hash: self.engine.state_hash(),
                }
            }
            CommandOutcome::Rejected(reason) => CommandResult::Rejected {
                reason: map_reject(&reason, self.engine.revision(), expected_revision),
            },
        })
    }

    fn seat_taken(&self, seat: u8) -> bool {
        self.members.values().any(|m| m.seat == seat)
    }

    fn snapshot_for(&self, m: &NativeMember) -> ObserverSnapshot {
        match m.role {
            ClientRole::Player => observer_snapshot_for(
                &self.engine,
                RoomRole::Player,
                Some(m.player_id),
            ),
            ClientRole::Spectator => {
                observer_snapshot_for(&self.engine, RoomRole::Spectator, None)
            }
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
fn map_reject(reason: &engine_native::RejectReason, current: u64, expected: u64) -> ApiRejectReason {
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
