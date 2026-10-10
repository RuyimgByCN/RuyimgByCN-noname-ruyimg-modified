//! Per-connection session state.

use engine_protocol::{encode_s2c, ClientRole, ServerMessage};
use tokio::sync::mpsc;

/// Outbound text channel to a WebSocket.
#[derive(Clone)]
pub struct Outbound {
    tx: mpsc::Sender<String>,
}

impl std::fmt::Debug for Outbound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Outbound(..)")
    }
}

impl Outbound {
    /// Create.
    pub fn new(tx: mpsc::Sender<String>) -> Self {
        Self { tx }
    }

    /// Send a server message (drops if client is slow / closed).
    pub async fn send(&self, msg: &ServerMessage) -> bool {
        match encode_s2c(msg) {
            Ok(s) => self.tx.send(s).await.is_ok(),
            Err(_) => false,
        }
    }
}

/// Live session.
pub struct Session {
    /// Connection id.
    pub connection_id: u64,
    /// Display name.
    pub display_name: String,
    /// Current room.
    pub room_id: Option<u64>,
    /// Seated player id.
    pub player_id: Option<u32>,
    /// Role.
    pub role: Option<ClientRole>,
    /// Reconnect token.
    pub reconnect_token: Option<String>,
    /// Last accepted conn_seq from client.
    pub last_conn_seq: u64,
    /// Outbound.
    pub outbound: Outbound,
}

/// Handler error returned to wire as public Error.
#[derive(Debug)]
pub struct HandlerError {
    /// Code.
    pub code: engine_protocol::ErrorCode,
    /// Message.
    pub message: String,
}

impl HandlerError {
    /// Construct.
    pub fn new(code: engine_protocol::ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}
