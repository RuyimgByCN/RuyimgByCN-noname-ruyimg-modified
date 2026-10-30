//! Typed client/server messages.

use engine_api::{
    ChoiceRequest, Command, CommandResult, IdempotencyKey, LegalAction, ObserverSnapshot, PlayerId,
    VersionBundle,
};
use serde::{Deserialize, Serialize};

use crate::NETWORK_PROTOCOL_VERSION;

/// Envelope header common to room-scoped messages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Protocol version.
    pub protocol_version: u32,
    /// Room id (0 if not in room).
    pub room_id: u64,
    /// Per-connection monotonic sequence (client→server).
    pub conn_seq: u64,
    /// Associated authority revision when applicable.
    pub revision: u64,
}

impl MessageHeader {
    /// New header.
    pub fn new(room_id: u64, conn_seq: u64, revision: u64) -> Self {
        Self {
            protocol_version: NETWORK_PROTOCOL_VERSION,
            room_id,
            conn_seq,
            revision,
        }
    }
}

/// Client → Server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum ClientMessage {
    /// Capability / version hello.
    Hello {
        /// Client-reported versions.
        versions: VersionBundle,
        /// Display name (not trusted for auth).
        display_name: String,
    },
    /// Create a new identity room.
    CreateRoom {
        /// Native choice window, default 30 seconds.
        #[serde(default)]
        choice_timeout_secs: Option<u8>,
        /// Seat count 2..=8.
        players: u8,
        /// Rule RNG seed.
        seed: u64,
        /// Optional soft password (Phase 7 toy auth).
        password: String,
    },
    /// Join existing room.
    JoinRoom {
        /// Room id.
        room_id: u64,
        /// Password if required.
        password: String,
        /// Preferred seat (0-based); server may reassign.
        preferred_seat: Option<u8>,
        /// Reconnect token from prior JoinOk.
        reconnect_token: Option<String>,
    },
    /// Spectate (public projection only).
    Spectate {
        /// Room id.
        room_id: u64,
    },
    /// Submit a normalized command.
    SubmitCommand {
        /// Header with room + conn_seq + expected revision.
        header: MessageHeader,
        /// Command body.
        command: Command,
        /// Idempotency key (required for network).
        idempotency_key: IdempotencyKey,
        /// True when this is a reconnect replay. A legacy room without a saved
        /// receipt must reject it instead of executing a possibly-duplicate command.
        #[serde(default)]
        replay: bool,
    },
    /// Request full observer snapshot (after gap or reconnect).
    RequestSnapshot {
        /// Header.
        header: MessageHeader,
        /// Last revision client has (0 if none).
        last_revision: u64,
    },
    /// Heartbeat / keepalive.
    Heartbeat {
        /// Header.
        header: MessageHeader,
    },
    /// Leave room gracefully.
    LeaveRoom {
        /// Room id.
        room_id: u64,
    },
}

/// Server → Client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    /// Server hello / welcome.
    Welcome {
        /// Assigned connection id.
        connection_id: u64,
        /// Protocol version.
        protocol_version: u32,
        /// Server version bundle.
        versions: VersionBundle,
    },
    /// Room created.
    RoomCreated {
        /// Room id.
        room_id: u64,
        /// Host seat player id once seated.
        host_hint: String,
    },
    /// Join accepted.
    JoinOk {
        /// Room id.
        room_id: u64,
        /// Assigned player id (0 for spectator).
        player_id: u32,
        /// Seat index.
        seat: u8,
        /// Role.
        role: ClientRole,
        /// Token for reconnect.
        reconnect_token: String,
        /// Initial observer view.
        view: ObserverSnapshot,
        /// Current personalized choice barrier, if any.
        choice: Option<ChoiceRequest>,
        /// Personalized actions currently legal for this client.
        legal_actions: Vec<LegalAction>,
    },
    /// Command accepted or rejected (no hidden info in reject messages).
    CommandOutcome {
        /// Header echo.
        header: MessageHeader,
        /// Idempotency key echo.
        idempotency_key: IdempotencyKey,
        /// Result.
        result: CommandResult,
        /// Observer view for *this* client only.
        view: ObserverSnapshot,
        /// Personalized actions legal after applying (or rejecting) the command.
        legal_actions: Vec<LegalAction>,
        /// Pending choice if any (already filtered to this observer when personal).
        choice: Option<ChoiceRequest>,
        /// Public event codes only (optional sanitized list).
        public_event_codes: Vec<u32>,
    },
    /// Room public roster update.
    RoomRoster {
        /// Native room window; legacy rooms have no choice timer.
        choice_timeout_secs: Option<u8>,
        /// Room id.
        room_id: u64,
        /// Seats filled.
        seats: Vec<SeatPresence>,
        /// Match started?
        started: bool,
        /// Finished?
        finished: bool,
    },
    /// Snapshot response.
    Snapshot {
        /// Header.
        header: MessageHeader,
        /// Full observer view for this client.
        view: ObserverSnapshot,
        /// Current personalized choice barrier, if any.
        choice: Option<ChoiceRequest>,
        /// Personalized actions legal at the snapshot revision.
        legal_actions: Vec<LegalAction>,
    },
    /// Heartbeat reply.
    Pong {
        /// Header.
        header: MessageHeader,
        /// Server monotonic millis (not used by rules).
        server_time_ms: u64,
    },
    /// Error that is safe to show (never includes hidden cards/identity of others).
    Error {
        /// Machine code.
        code: ErrorCode,
        /// Safe message.
        message: String,
        /// Optional room.
        room_id: Option<u64>,
    },
}

/// Client role in a room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientRole {
    /// Seated player.
    Player,
    /// Public spectator.
    Spectator,
    /// Referee (debug; full view) — not granted by default join.
    Referee,
}

/// Presence of a seat (public).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatPresence {
    /// Seat index.
    pub seat: u8,
    /// Player id if seated.
    pub player_id: u32,
    /// Display name (sanitized).
    pub display_name: String,
    /// Connected now.
    pub connected: bool,
    /// Server-controlled seat takeover.
    pub managed: bool,
}

/// Safe error codes (no hidden-state leakage).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Protocol / parse.
    BadMessage,
    /// Auth / password.
    Unauthorized,
    /// Room missing.
    RoomNotFound,
    /// Room full.
    RoomFull,
    /// Not in room.
    NotInRoom,
    /// Rate limited.
    RateLimited,
    /// Internal.
    Internal,
    /// Version mismatch.
    VersionMismatch,
    /// Duplicate / stale handled as command result usually.
    Conflict,
}

/// Helper: sanitize error for wire (strip anything that might look like state dumps).
pub fn public_error(code: ErrorCode, message: impl Into<String>) -> ServerMessage {
    let message = message.into();
    // Hard cap length; never attach dumps.
    let message = if message.len() > 200 {
        message.chars().take(200).collect()
    } else {
        message
    };
    ServerMessage::Error {
        code,
        message,
        room_id: None,
    }
}

/// Map player id for JSON (engine uses PlayerId).
pub fn player_raw(p: PlayerId) -> u32 {
    p.raw()
}
