//! One room = one serial Tokio task + bounded mailbox (architecture §14.2).

use crate::persist::{RoomPersist, WalEntry};
use crate::session::Outbound;
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, ObserverKind, PlayerId,
};
use engine_core::{project_observer, project_player, Engine};
use engine_protocol::{ClientRole, MessageHeader, SeatPresence, ServerMessage};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Messages into the room actor.
#[derive(Debug)]
pub enum RoomMsg {
    /// Seat a player connection.
    Join {
        connection_id: u64,
        display_name: String,
        preferred_seat: Option<u8>,
        reconnect_token: Option<String>,
        password: String,
        outbound: Outbound,
        reply: oneshot::Sender<Result<JoinReply, String>>,
    },
    /// Spectate with public view.
    Spectate {
        connection_id: u64,
        display_name: String,
        outbound: Outbound,
        reply: oneshot::Sender<Result<JoinReply, String>>,
    },
    /// Submit command from a seated player.
    Submit {
        connection_id: u64,
        header: MessageHeader,
        command: Command,
        idempotency_key: IdempotencyKey,
        reply: oneshot::Sender<Result<ServerMessage, String>>,
    },
    /// Snapshot request.
    Snapshot {
        connection_id: u64,
        header: MessageHeader,
        reply: oneshot::Sender<Result<ServerMessage, String>>,
    },
    /// Connection dropped.
    Disconnect { connection_id: u64 },
    /// Debug: force recover from WAL (tests).
    #[allow(dead_code)]
    Recover {
        reply: oneshot::Sender<Result<(), String>>,
    },
}

/// Successful join payload.
#[derive(Debug, Clone)]
pub struct JoinReply {
    /// Player id (0 spectator).
    pub player_id: u32,
    /// Seat.
    pub seat: u8,
    /// Role.
    pub role: ClientRole,
    /// Token.
    pub reconnect_token: String,
    /// Initial view.
    pub view: engine_api::ObserverSnapshot,
    /// Personalized pending choice.
    pub choice: Option<engine_api::ChoiceRequest>,
    /// Personalized legal actions.
    pub legal_actions: Vec<engine_api::LegalAction>,
}

struct Member {
    connection_id: u64,
    display_name: String,
    player_id: PlayerId,
    seat: u8,
    role: ClientRole,
    reconnect_token: String,
    outbound: Option<Outbound>,
    connected: bool,
}

/// Room handle (cloneable sender).
#[derive(Clone)]
pub struct RoomHandle {
    /// Room id.
    pub id: u64,
    tx: mpsc::Sender<RoomMsg>,
}

impl RoomHandle {
    /// Send message to room.
    pub async fn send(&self, msg: RoomMsg) -> Result<(), String> {
        self.tx
            .send(msg)
            .await
            .map_err(|_| "room closed".to_string())
    }
}

/// Spawn a room task. Returns handle.
pub fn spawn_room(
    id: u64,
    players: u8,
    seed: u64,
    password: String,
    mailbox: usize,
    snapshot_every: u64,
) -> RoomHandle {
    spawn_room_with_engine(
        id,
        players,
        seed,
        password,
        mailbox,
        snapshot_every,
        Engine::new(),
    )
}

/// Spawn a room task from a preloaded, content-locked engine template.
pub fn spawn_room_with_engine(
    id: u64,
    players: u8,
    seed: u64,
    password: String,
    mailbox: usize,
    snapshot_every: u64,
    engine_template: Engine,
) -> RoomHandle {
    let (tx, rx) = mpsc::channel(mailbox);
    tokio::spawn(room_loop(
        id,
        players,
        seed,
        password,
        rx,
        snapshot_every,
        engine_template,
    ));
    RoomHandle { id, tx }
}

async fn room_loop(
    id: u64,
    players: u8,
    seed: u64,
    password: String,
    mut rx: mpsc::Receiver<RoomMsg>,
    snapshot_every: u64,
    engine_template: Engine,
) {
    let mut engine = engine_template.clone();
    let mut persist = RoomPersist::new();
    let mut members: HashMap<u64, Member> = HashMap::new(); // by connection
    let mut by_token: HashMap<String, u64> = HashMap::new(); // token -> connection
    let mut seat_owner: HashMap<u8, u64> = HashMap::new(); // seat -> connection
    let mut idempotency: HashMap<u64, CommandResult> = HashMap::new();
    let mut started = false;
    let mut next_seat: u8 = 0;
    let max_seats = players;

    // Pre-create empty seats only after joins; start game when full or host starts via first StartGame command
    info!(room = id, players, seed, "room started");

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
                if !password.is_empty() && pw != password {
                    let _ = reply.send(Err("unauthorized".into()));
                    continue;
                }

                // Reconnect path
                if let Some(token) = reconnect_token {
                    if let Some(old_conn) = by_token.get(&token).copied() {
                        if let Some(mut m) = members.remove(&old_conn) {
                            seat_owner.insert(m.seat, connection_id);
                            by_token.insert(token.clone(), connection_id);
                            m.connection_id = connection_id;
                            m.outbound = Some(outbound);
                            m.connected = true;
                            m.display_name = display_name;
                            let view = project_for(&engine, m.role, m.player_id);
                            let jr = JoinReply {
                                player_id: m.player_id.raw(),
                                seat: m.seat,
                                role: m.role,
                                reconnect_token: m.reconnect_token.clone(),
                                view,
                                choice: choice_for(&engine, m.role, m.player_id),
                                legal_actions: legal_for(&engine, m.role, m.player_id),
                            };
                            members.insert(connection_id, m);
                            broadcast_roster(
                                id,
                                &members,
                                started,
                                engine.state().outcome.is_finished(),
                            )
                            .await;
                            let _ = reply.send(Ok(jr));
                            continue;
                        }
                    }
                }

                if next_seat >= max_seats && seat_owner.len() >= max_seats as usize {
                    let _ = reply.send(Err("room full".into()));
                    continue;
                }

                let seat = preferred_seat
                    .filter(|s| *s < max_seats && !seat_owner.contains_key(s))
                    .unwrap_or_else(|| {
                        (0..max_seats)
                            .find(|s| !seat_owner.contains_key(s))
                            .unwrap_or(next_seat)
                    });
                if seat_owner.contains_key(&seat) {
                    let _ = reply.send(Err("seat taken".into()));
                    continue;
                }

                // Player ids are 1-based after StartGame; before start use seat+1 provisional
                let player_id = PlayerId::new(seat as u32 + 1);
                let token = Uuid::new_v4().to_string();
                seat_owner.insert(seat, connection_id);
                by_token.insert(token.clone(), connection_id);
                next_seat = next_seat.max(seat.saturating_add(1));

                let member = Member {
                    connection_id,
                    display_name,
                    player_id,
                    seat,
                    role: ClientRole::Player,
                    reconnect_token: token.clone(),
                    outbound: Some(outbound),
                    connected: true,
                };

                // Auto-start when all seats filled
                if !started && seat_owner.len() == max_seats as usize {
                    let start = Command {
                        actor: PlayerId::NONE,
                        expected_revision: ExpectedRevision(0),
                        idempotency_key: IdempotencyKey(0),
                        kind: CommandKind::StartGame {
                            player_count: max_seats,
                            rng_seed: seed,
                            ruleset: "identity-standard".into(),
                        },
                    };
                    match engine.submit(start.clone()) {
                        CommandResult::Accepted {
                            new_revision,
                            state_hash,
                        } => {
                            // Pin the manifest before any WAL append (Phase 15 / §5).
                            if let Some(m) = engine.manifest() {
                                persist.pin_manifest(m.clone());
                            }
                            persist.append(WalEntry {
                                revision: new_revision,
                                state_hash,
                                command: start,
                                idempotency_key: 0,
                                manifest_hash: engine_api::MatchManifestHash::ZERO,
                            });
                            persist.put_snapshot(engine.snapshot());
                            started = true;
                            info!(
                                room = id,
                                rev = new_revision,
                                manifest = %hex_manifest(&engine.manifest_hash()),
                                "match started"
                            );
                        }
                        CommandResult::Rejected { reason } => {
                            warn!(?reason, "failed to start match");
                        }
                    }
                }

                let view = project_for(&engine, ClientRole::Player, player_id);
                let choice = choice_for(&engine, ClientRole::Player, player_id);
                let legal_actions = legal_for(&engine, ClientRole::Player, player_id);
                members.insert(connection_id, member);
                broadcast_roster(id, &members, started, engine.state().outcome.is_finished()).await;
                let _ = reply.send(Ok(JoinReply {
                    player_id: player_id.raw(),
                    seat,
                    role: ClientRole::Player,
                    reconnect_token: token,
                    view,
                    choice,
                    legal_actions,
                }));
            }

            RoomMsg::Spectate {
                connection_id,
                display_name,
                outbound,
                reply,
            } => {
                let token = Uuid::new_v4().to_string();
                let view = project_observer(engine.state(), ObserverKind::Spectator);
                members.insert(
                    connection_id,
                    Member {
                        connection_id,
                        display_name,
                        player_id: PlayerId::NONE,
                        seat: 255,
                        role: ClientRole::Spectator,
                        reconnect_token: token.clone(),
                        outbound: Some(outbound),
                        connected: true,
                    },
                );
                let _ = reply.send(Ok(JoinReply {
                    player_id: 0,
                    seat: 255,
                    role: ClientRole::Spectator,
                    reconnect_token: token,
                    view,
                    choice: None,
                    legal_actions: vec![],
                }));
            }

            RoomMsg::Submit {
                connection_id,
                header,
                mut command,
                idempotency_key,
                reply,
            } => {
                let Some(member) = members.get(&connection_id) else {
                    let _ = reply.send(Err("not in room".into()));
                    continue;
                };
                if member.role != ClientRole::Player {
                    let _ = reply.send(Err("spectators cannot submit".into()));
                    continue;
                }
                if !started {
                    let _ = reply.send(Err("match not started".into()));
                    continue;
                }

                // Idempotency: replay prior result without re-executing
                if idempotency_key.0 != 0 {
                    if let Some(prev) = idempotency.get(&idempotency_key.0) {
                        let view = project_for(&engine, member.role, member.player_id);
                        let msg = ServerMessage::CommandOutcome {
                            header: MessageHeader::new(
                                id,
                                header.conn_seq,
                                engine.state().revision,
                            ),
                            idempotency_key,
                            result: prev.clone(),
                            view,
                            legal_actions: legal_for(&engine, member.role, member.player_id),
                            choice: choice_for(&engine, member.role, member.player_id),
                            public_event_codes: vec![],
                        };
                        let _ = reply.send(Ok(msg));
                        continue;
                    }
                }

                // Force actor to seated player (anti-spoof)
                command.actor = member.player_id;
                command.idempotency_key = idempotency_key;
                if header.revision != 0 {
                    command.expected_revision = ExpectedRevision(header.revision);
                }

                let result = engine.submit(command.clone());
                match &result {
                    CommandResult::Accepted {
                        new_revision,
                        state_hash,
                    } => {
                        // Persist BEFORE publish (architecture §14.3)
                        persist.append(WalEntry {
                            revision: *new_revision,
                            state_hash: *state_hash,
                            command: command.clone(),
                            idempotency_key: idempotency_key.0,
                            manifest_hash: engine_api::MatchManifestHash::ZERO,
                        });
                        if *new_revision % snapshot_every == 0 {
                            persist.put_snapshot(engine.snapshot());
                        }
                        if idempotency_key.0 != 0 {
                            idempotency.insert(idempotency_key.0, result.clone());
                        }
                        // Fan-out personalized views to everyone except submitter (submitter gets oneshot)
                        fanout_after_command(
                            id,
                            &engine,
                            &members,
                            connection_id,
                            idempotency_key,
                            result.clone(),
                        )
                        .await;
                        let view = project_for(&engine, member.role, member.player_id);
                        let choice = choice_for(&engine, member.role, member.player_id);
                        let _ = reply.send(Ok(ServerMessage::CommandOutcome {
                            header: MessageHeader::new(id, header.conn_seq, *new_revision),
                            idempotency_key,
                            result,
                            view,
                            legal_actions: legal_for(&engine, member.role, member.player_id),
                            choice,
                            public_event_codes: public_codes(engine.last_events()),
                        }));
                    }
                    CommandResult::Rejected { .. } => {
                        // Reject: no state change, only to submitter
                        let view = project_for(&engine, member.role, member.player_id);
                        let _ = reply.send(Ok(ServerMessage::CommandOutcome {
                            header: MessageHeader::new(
                                id,
                                header.conn_seq,
                                engine.state().revision,
                            ),
                            idempotency_key,
                            result,
                            view,
                            legal_actions: legal_for(&engine, member.role, member.player_id),
                            choice: choice_for(&engine, member.role, member.player_id),
                            public_event_codes: vec![],
                        }));
                    }
                }
            }

            RoomMsg::Snapshot {
                connection_id,
                header,
                reply,
            } => {
                let Some(member) = members.get(&connection_id) else {
                    let _ = reply.send(Err("not in room".into()));
                    continue;
                };
                let view = project_for(&engine, member.role, member.player_id);
                let _ = reply.send(Ok(ServerMessage::Snapshot {
                    header: MessageHeader::new(id, header.conn_seq, engine.state().revision),
                    view,
                    choice: choice_for(&engine, member.role, member.player_id),
                    legal_actions: legal_for(&engine, member.role, member.player_id),
                }));
            }

            RoomMsg::Disconnect { connection_id } => {
                if let Some(m) = members.get_mut(&connection_id) {
                    m.connected = false;
                    m.outbound = None;
                    debug!(room = id, conn = connection_id, "member disconnected");
                }
                broadcast_roster(id, &members, started, engine.state().outcome.is_finished()).await;
            }

            RoomMsg::Recover { reply } => {
                match crate::persist::recover_engine_from(&persist, engine_template.clone()) {
                    Ok(e) => {
                        engine = e;
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }
        }
    }
    info!(room = id, "room task exit");
}

fn project_for(
    engine: &Engine,
    role: ClientRole,
    player_id: PlayerId,
) -> engine_api::ObserverSnapshot {
    match role {
        ClientRole::Player => project_player(engine.state(), player_id),
        ClientRole::Spectator => project_observer(engine.state(), ObserverKind::Spectator),
        ClientRole::Referee => project_observer(engine.state(), ObserverKind::Referee),
    }
}

fn legal_for(
    engine: &Engine,
    role: ClientRole,
    player_id: PlayerId,
) -> Vec<engine_api::LegalAction> {
    if role == ClientRole::Player {
        engine_ai::legal_actions(engine, player_id)
    } else {
        vec![]
    }
}

fn choice_for(
    engine: &Engine,
    role: ClientRole,
    player_id: PlayerId,
) -> Option<engine_api::ChoiceRequest> {
    if role == ClientRole::Spectator {
        return None;
    }
    engine
        .last_choice()
        .cloned()
        .filter(|choice| role == ClientRole::Referee || choice.observer == player_id)
}

fn public_codes(events: &[engine_api::DomainEvent]) -> Vec<u32> {
    // Stable coarse codes for UI; no card identities of hidden hands
    events
        .iter()
        .map(|e| match e {
            engine_api::DomainEvent::GameStarted { .. } => 1,
            engine_api::DomainEvent::CardsDrawn { .. } => 2,
            engine_api::DomainEvent::CardUsed { .. } => 3,
            engine_api::DomainEvent::DamageDealt { .. } => 4,
            engine_api::DomainEvent::HpRecovered { .. } => 5,
            engine_api::DomainEvent::EnteredDying { .. } => 6,
            engine_api::DomainEvent::PlayerKilled { .. } => 7,
            engine_api::DomainEvent::TurnStateChanged { .. } => 8,
            engine_api::DomainEvent::MatchEnded { .. } => 9,
            engine_api::DomainEvent::ChoiceRequested { .. } => 10,
            _ => 0,
        })
        .filter(|c| *c != 0)
        .collect()
}

async fn fanout_after_command(
    room_id: u64,
    engine: &Engine,
    members: &HashMap<u64, Member>,
    submitter_conn: u64,
    idem: IdempotencyKey,
    result: CommandResult,
) {
    let rev = engine.state().revision;
    for m in members.values() {
        if !m.connected || m.connection_id == submitter_conn {
            continue;
        }
        let Some(out) = m.outbound.as_ref() else {
            continue;
        };
        let view = project_for(engine, m.role, m.player_id);
        let choice = choice_for(engine, m.role, m.player_id);
        let msg = ServerMessage::CommandOutcome {
            header: MessageHeader::new(room_id, 0, rev),
            idempotency_key: idem,
            result: result.clone(),
            view,
            legal_actions: legal_for(engine, m.role, m.player_id),
            choice,
            public_event_codes: public_codes(engine.last_events()),
        };
        let _ = out.send(&msg).await;
    }
}

async fn broadcast_roster(
    room_id: u64,
    members: &HashMap<u64, Member>,
    started: bool,
    finished: bool,
) {
    let mut seats: Vec<SeatPresence> = members
        .values()
        .filter(|m| m.role == ClientRole::Player)
        .map(|m| SeatPresence {
            seat: m.seat,
            player_id: m.player_id.raw(),
            display_name: m.display_name.clone(),
            connected: m.connected,
        })
        .collect();
    seats.sort_by_key(|s| s.seat);
    let msg = ServerMessage::RoomRoster {
        room_id,
        seats,
        started,
        finished,
    };
    for m in members.values() {
        if let Some(out) = m.outbound.as_ref() {
            let _ = out.send(&msg).await;
        }
    }
}

/// Format the first 8 bytes of a manifest hash for compact tracing.
fn hex_manifest(h: &engine_api::MatchManifestHash) -> String {
    let s: String = h.0.iter().take(8).map(|b| format!("{b:02x}")).collect();
    s
}
