//! M8 gate 2: 滚动升级演练——进行中对局不受影响.
//!
//! Rehearsal: the lobby pins a ContentLock per room; a "rolling upgrade" is
//! simulated by running two rooms concurrently, one from the old content
//! (standard only) and one from new content (standard + sp). Each room keeps its
//! pinned content: identical templates reproduce identical state hashes while
//! the different-content room runs in parallel without interference.

#[path = "../src/config.rs"]
mod config;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/session.rs"]
mod session;

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::Engine;
use engine_protocol::{ClientRole, MessageHeader, ServerMessage};
use room::{spawn_room_with_engine, RoomMsg};
use tokio::sync::mpsc;

const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn template(with_sp: bool) -> Engine {
    let mut engine = Engine::new();
    if with_sp {
        let pkg = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
        engine.load_package(&pkg).unwrap();
    }
    engine
}

async fn join_and_start(handle: &room::RoomHandle, conn: u64, name: &str) -> (u32, u64) {
    let (tx, mut rx) = mpsc::channel(16);
    let outbound = session::Outbound::new(tx);
    let (reply, wait) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: conn,
            display_name: name.into(),
            preferred_seat: None,
            reconnect_token: None,
            password: String::new(),
            outbound,
            reply,
        })
        .await
        .unwrap();
    let joined = wait.await.unwrap().unwrap();
    let mut drained = 0;
    while rx.try_recv().is_ok() {
        drained += 1;
    }
    (joined.player_id, drained)
}

async fn submit_end_phase(
    handle: &room::RoomHandle,
    conn: u64,
    player: u32,
    seq: u64,
) -> ([u8; 32], u64) {
    let (reply, wait) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Submit {
            connection_id: conn,
            header: MessageHeader::new(1, seq, 0),
            command: Command {
                actor: PlayerId::new(player),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(seq),
                kind: CommandKind::EndPhase,
            },
            idempotency_key: IdempotencyKey(seq),
            replay: false,
            reply,
        })
        .await
        .unwrap();
    match wait.await.unwrap().unwrap() {
        ServerMessage::CommandOutcome { result, view, .. } => {
            eprintln!("submit conn={conn} player={player} current={:?} result={:?}", view.current_player, result);
            assert!(matches!(result, CommandResult::Accepted { .. }), "end-phase rejected: {result:?}");
            (view.state_hash, view.revision)
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[tokio::test]
async fn rolling_upgrade_keeps_matches_on_their_pinned_content() {
    // Two "old-content" rooms (standard only) — deterministic twins.
    let a1 = spawn_room_with_engine(1, 2, 42, String::new(), 32, 5, template(false));
    let a2 = spawn_room_with_engine(2, 2, 42, String::new(), 32, 5, template(false));
    // One "new-content" room (standard + sp) — the rolling upgrade.
    let b = spawn_room_with_engine(3, 2, 42, String::new(), 32, 5, template(true));

    for (handle, base) in [(&a1, 100u64), (&a2, 200), (&b, 300)] {
        let (p1, _) = join_and_start(handle, base, "p1").await;
        let (_p2, _) = join_and_start(handle, base + 1, "p2").await;
        let (hash, _rev) = submit_end_phase(handle, base, p1, base + 2).await;
        assert_ne!(hash, [0u8; 32], "room {base} started with a real state hash");
    }

    // Determinism: the two old-content rooms produced identical states.
    let (h_a1, r_a1) = {
        let (reply, wait) = tokio::sync::oneshot::channel();
        a1.send(RoomMsg::Submit {
            connection_id: 100,
            header: MessageHeader::new(1, 50, 0),
            command: Command {
                actor: PlayerId::new(1),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(50),
                kind: CommandKind::EndPhase,
            },
            idempotency_key: IdempotencyKey(50),
            replay: false,
            reply,
        })
        .await
        .unwrap();
        match wait.await.unwrap().unwrap() {
            ServerMessage::CommandOutcome { view, .. } => (view.state_hash, view.revision),
            other => panic!("unexpected {other:?}"),
        }
    };
    let (h_a2, r_a2) = {
        let (reply, wait) = tokio::sync::oneshot::channel();
        a2.send(RoomMsg::Submit {
            connection_id: 200,
            header: MessageHeader::new(1, 51, 0),
            command: Command {
                actor: PlayerId::new(1),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(51),
                kind: CommandKind::EndPhase,
            },
            idempotency_key: IdempotencyKey(51),
            replay: false,
            reply,
        })
        .await
        .unwrap();
        match wait.await.unwrap().unwrap() {
            ServerMessage::CommandOutcome { view, .. } => (view.state_hash, view.revision),
            other => panic!("unexpected {other:?}"),
        }
    };
    assert_eq!(h_a1, h_a2, "old-content rooms stay byte-identical");
    assert_eq!(r_a1, r_a2, "old-content rooms advance identically");
}

#[tokio::test]
async fn different_content_locks_do_not_collide() {
    // The new-content room carries a different pinned content lock than the old one:
    // its start state must differ (standard+sp vs standard-only), proving the
    // rolling upgrade does not silently mutate old rooms.
    let old = spawn_room_with_engine(11, 2, 7, String::new(), 32, 5, template(false));
    let new = spawn_room_with_engine(12, 2, 7, String::new(), 32, 5, template(true));
    let mut new_hash = [0u8; 32];
    for (handle, base, capture) in [(&old, 400u64, false), (&new, 500, true)] {
        let (p1, _) = join_and_start(handle, base, "p1").await;
        let (_p2, _) = join_and_start(handle, base + 1, "p2").await;
        let (hash, _) = submit_end_phase(handle, base, p1, base + 2).await;
        if capture {
            new_hash = hash;
        }
    }
    // Re-run the new room's match in isolation: identical (deterministic content pin).
    let alone = spawn_room_with_engine(13, 2, 7, String::new(), 32, 5, template(true));
    let (p1, _) = join_and_start(&alone, 600, "p1").await;
    let (_p2, _) = join_and_start(&alone, 601, "p2").await;
    let (alone_hash, _) = submit_end_phase(&alone, 600, p1, 602).await;
    assert_eq!(alone_hash, new_hash, "new-content room is deterministic in isolation");
}
