//! Shared authenticated production WebSocket test peer.
use super::*;

impl Peer {
    pub(super) async fn connect(server: &Server) -> Self {
        Self::connect_url(&server.url).await
    }

    pub(super) async fn connect_url(url: &str) -> Self {
        let (ws, response) = tokio_tungstenite::connect_async(url).await.unwrap();
        assert_eq!(response.status().as_u16(), 101);
        Self::from_ws(ws).await
    }

    pub(super) async fn from_ws(ws: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        let mut peer = Self {
            ws,
            room: 0,
            player: PlayerId::NONE,
            token: String::new(),
            seq: 0,
        };
        peer.until(|message| matches!(message, ServerMessage::Welcome { .. }))
            .await;
        peer.send(ClientMessage::Hello {
            versions: engine_api::VersionBundle::current(),
            display_name: "ws-test".into(),
        })
        .await;
        peer
    }

    pub(super) async fn send(&mut self, message: ClientMessage) {
        record_wire("client", &encode_c2s(&message).unwrap());
        self.ws
            .send(Message::Text(encode_c2s(&message).unwrap().into()))
            .await
            .unwrap();
    }

    pub(super) async fn until(
        &mut self,
        matches: impl Fn(&ServerMessage) -> bool,
    ) -> ServerMessage {
        for _ in 0..40 {
            let frame = tokio::time::timeout(Duration::from_secs(10), self.ws.next())
                .await
                .expect("WS response timeout")
                .expect("WS remains open")
                .unwrap();
            if let Message::Text(text) = frame {
                record_wire("server", text.as_ref());
                let message = decode_s2c(text.as_ref()).expect("valid server protocol");
                if matches(&message) {
                    return message;
                }
                assert!(
                    !matches!(message, ServerMessage::Error { .. }),
                    "unexpected {message:?}"
                );
            }
        }
        panic!("expected WS response within 40 frames");
    }

    pub(super) async fn join_result(&mut self) -> State {
        let message = self
            .until(|message| matches!(message, ServerMessage::JoinOk { .. }))
            .await;
        if let ServerMessage::JoinOk {
            room_id,
            player_id,
            reconnect_token,
            role,
            seat,
            ..
        } = &message
        {
            match role {
                ClientRole::Player => assert_eq!(*seat as u32 + 1, *player_id),
                ClientRole::Spectator | ClientRole::Referee => {
                    assert_eq!(*player_id, 0);
                    assert_eq!(*seat, u8::MAX);
                }
            }
            self.room = *room_id;
            self.player = PlayerId::new(*player_id);
            self.token = reconnect_token.clone();
        }
        State::from_message(message)
    }

    pub(super) async fn create(&mut self) -> State {
        self.send(ClientMessage::CreateRoom {
            choice_timeout_secs: None,
            players: 2,
            seed: SEED,
            password: PASSWORD.into(),
        })
        .await;
        self.join_result().await
    }

    pub(super) async fn join(&mut self, room_id: u64, seat: u8, token: Option<String>) -> State {
        self.send(ClientMessage::JoinRoom {
            room_id,
            password: PASSWORD.into(),
            preferred_seat: Some(seat),
            reconnect_token: token,
        })
        .await;
        self.join_result().await
    }

    pub(super) async fn spectate(&mut self, room_id: u64) -> State {
        self.send(ClientMessage::Spectate { room_id }).await;
        self.join_result().await
    }

    pub(super) async fn snapshot(&mut self) -> State {
        self.seq += 1;
        let sequence = self.seq;
        self.send(ClientMessage::RequestSnapshot {
            header: MessageHeader::new(self.room, sequence, 0),
            last_revision: 0,
        })
        .await;
        State::from_message(self.until(|message| {
            matches!(message, ServerMessage::Snapshot { header, .. } if header.conn_seq == sequence)
        }).await)
    }

    pub(super) async fn submit(
        &mut self,
        actor: PlayerId,
        expected: u64,
        key: u64,
        kind: CommandKind,
    ) -> (CommandResult, State) {
        self.seq += 1;
        let sequence = self.seq;
        self.send(ClientMessage::SubmitCommand {
            header: MessageHeader::new(self.room, sequence, expected),
            command: Command {
                actor,
                expected_revision: ExpectedRevision(expected),
                idempotency_key: IdempotencyKey(key),
                kind,
            },
            idempotency_key: IdempotencyKey(key),
            replay: false,
        })
        .await;
        let message = self.until(|message| {
            matches!(message, ServerMessage::CommandOutcome { header, .. } if header.conn_seq == sequence)
        }).await;
        let result = match &message {
            ServerMessage::CommandOutcome { result, .. } => result.clone(),
            _ => unreachable!(),
        };
        (result, State::from_message(message))
    }

    pub(super) async fn submit_replay(
        &mut self,
        actor: PlayerId,
        expected: u64,
        key: u64,
        kind: CommandKind,
    ) -> (CommandResult, State) {
        self.seq += 1;
        let sequence = self.seq;
        self.send(ClientMessage::SubmitCommand {
            header: MessageHeader::new(self.room, sequence, expected),
            command: Command {
                actor,
                expected_revision: ExpectedRevision(expected),
                idempotency_key: IdempotencyKey(key),
                kind,
            },
            idempotency_key: IdempotencyKey(key),
            replay: true,
        })
        .await;
        let message = self
            .until(|message| {
                matches!(message, ServerMessage::CommandOutcome { header, .. } if header.conn_seq == sequence)
            })
            .await;
        let result = match &message {
            ServerMessage::CommandOutcome { result, .. } => result.clone(),
            _ => unreachable!(),
        };
        (result, State::from_message(message))
    }

    pub(super) async fn pushed(&mut self, key: u64) -> State {
        State::from_message(
            self.until(|message| {
                matches!(message, ServerMessage::CommandOutcome { header, idempotency_key, .. }
                if header.conn_seq == 0 && idempotency_key.0 == key)
            })
            .await,
        )
    }

    pub(super) async fn error(&mut self, expected: ErrorCode) {
        let message = self
            .until(|message| matches!(message, ServerMessage::Error { .. }))
            .await;
        assert!(
            matches!(message, ServerMessage::Error { code, .. } if code == expected),
            "{message:?}"
        );
    }

    pub(super) async fn close(mut self) {
        self.ws.close(None).await.unwrap();
    }
}

fn record_wire(direction: &str, wire: &str) {
    if let Ok(root) = std::env::var("C331_PHASE3_EVIDENCE") {
        use std::io::Write;
        static WIRE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _serial = WIRE_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut line =
            serde_json::to_vec(&serde_json::json!({"direction":direction,"wire":wire})).unwrap();
        line.push(b'\n');
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(std::path::Path::new(&root).join("server-ws-wire.jsonl"))
            .unwrap();
        file.write_all(&line).unwrap();
    }
}
