//! Per-connection session state.

use engine_protocol::{encode_s2c, ClientRole, ServerMessage};
use tokio::sync::{mpsc, watch};

/// 有界发送结果。`Full` 表示客户端持续不读，房间不得为它等待。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendStatus {
    Queued,
    Full,
    Closed,
}

/// Outbound text channel to a WebSocket.
#[derive(Clone)]
pub struct Outbound {
    tx: mpsc::Sender<String>,
    /// 置 `true` 后，该连接的 session 读循环退出并走正常断开/重连。
    slow: watch::Sender<bool>,
}

impl std::fmt::Debug for Outbound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Outbound(..)")
    }
}

impl Outbound {
    /// Create.
    pub fn new(tx: mpsc::Sender<String>) -> Self {
        let (slow, _) = watch::channel(false);
        Self { tx, slow }
    }

    /// 供 session 读循环观察「写端已超时/队列已满」。
    pub fn slow_signal(&self) -> watch::Receiver<bool> {
        self.slow.subscribe()
    }

    /// 只含慢信号发送端，供 socket writer 使用；**不得**把它扩展成
    /// `Outbound` 克隆——那会让 writer 自己持有出站 `Sender`，session 退出后
    /// 队列永远不关闭（连接无法结束）。
    pub fn slow_marker(&self) -> watch::Sender<bool> {
        self.slow.clone()
    }

    /// 有界非阻塞发送：队列满时标记慢连接并返回 `Full`，绝不等待。
    pub fn try_send(&self, msg: &ServerMessage) -> SendStatus {
        let Ok(text) = encode_s2c(msg) else {
            return SendStatus::Closed;
        };
        match self.tx.try_send(text) {
            Ok(()) => SendStatus::Queued,
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.mark_slow();
                SendStatus::Full
            }
            Err(mpsc::error::TrySendError::Closed(_)) => SendStatus::Closed,
        }
    }

    /// 发送并等待队列空位（仅用于连接建立阶段的少量消息；房间广播必须
    /// 使用 [`Self::try_send`]，否则慢客户端会阻塞整个 room actor）。
    pub async fn send(&self, msg: &ServerMessage) -> bool {
        match self.try_send(msg) {
            SendStatus::Queued => true,
            SendStatus::Full | SendStatus::Closed => false,
        }
    }

    /// 由 socket writer 超时或写失败调用：让 session 读循环尽快退出。
    pub fn mark_slow(&self) {
        let _ = self.slow.send(true);
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
    /// Only an exact WS6 Hello enables public room operations.
    pub handshake_complete: bool,
    /// Outbound.
    pub outbound: Outbound,
    /// 慢连接信号：socket writer 超时或房间广播队列满时置 `true`。
    pub slow: watch::Receiver<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server_error() -> ServerMessage {
        ServerMessage::Error {
            code: engine_protocol::ErrorCode::BadMessage,
            message: "test".into(),
            room_id: None,
        }
    }

    #[tokio::test]
    async fn full_queue_marks_slow_instead_of_waiting() {
        let (tx, _rx) = mpsc::channel(1);
        let outbound = Outbound::new(tx);
        let mut slow = outbound.slow_signal();
        assert_eq!(outbound.try_send(&server_error()), SendStatus::Queued);
        assert_eq!(outbound.try_send(&server_error()), SendStatus::Full);
        assert!(slow.has_changed().unwrap(), "满载必须通知 session 退出");
        assert!(*slow.borrow_and_update());
    }

    #[tokio::test]
    async fn closed_writer_reports_closed_without_slow_mark() {
        let (tx, rx) = mpsc::channel(1);
        drop(rx);
        let outbound = Outbound::new(tx);
        assert_eq!(outbound.try_send(&server_error()), SendStatus::Closed);
    }
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
