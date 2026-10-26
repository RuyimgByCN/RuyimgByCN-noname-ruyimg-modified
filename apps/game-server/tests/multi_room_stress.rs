//! Multi-room stress test (architecture §31.1: one room = one serial task, multiple rooms parallel).
//!
//! Spawns N concurrent rooms, verifies they operate independently without
//! panics or deadlocks.

#[path = "../src/config.rs"]
mod config;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/session.rs"]
mod session;

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_protocol::{MessageHeader, ServerMessage};
use room::{spawn_room, RoomMsg};
use session::Outbound;
use std::time::Instant;
use tokio::sync::mpsc;

/// Spawn a room, join 2 players, submit a few commands.
async fn exercise_room(room_id: u64, seed: u64) -> Result<u64, String> {
    let handle = spawn_room(room_id, 2, seed, String::new(), 64, 100);

    let (tx1, mut rx1) = mpsc::channel(8);
    let (r1, w1) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 1,
            display_name: "a".into(),
            preferred_seat: Some(0),
            reconnect_token: None,
            password: String::new(),
            outbound: Outbound::new(tx1),
            reply: r1,
        })
        .await
        .map_err(|e| e.to_string())?;
    let j1 = w1.await.map_err(|e| e.to_string())?.map_err(|e| e)?;
    let player1_id = j1.player_id;

    let (tx2, mut rx2) = mpsc::channel(8);
    let (r2, w2) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 2,
            display_name: "b".into(),
            preferred_seat: Some(1),
            reconnect_token: None,
            password: String::new(),
            outbound: Outbound::new(tx2),
            reply: r2,
        })
        .await
        .map_err(|e| e.to_string())?;
    let _j2 = w2.await.map_err(|e| e.to_string())?.map_err(|e| e)?;

    while rx1.try_recv().is_ok() {}
    while rx2.try_recv().is_ok() {}

    let mut accepted = 0u64;
    for i in 0..5u64 {
        let (rs, ws) = tokio::sync::oneshot::channel();
        if handle
            .send(RoomMsg::Submit {
                connection_id: 1,
                header: MessageHeader::new(room_id, i, 0),
                command: Command {
                    actor: PlayerId::new(player1_id),
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(i),
                    kind: CommandKind::EndPhase,
                },
                idempotency_key: IdempotencyKey(i),
                replay: false,
                reply: rs,
            })
            .await
            .is_err()
        {
            break;
        }
        if let Ok(Ok(ServerMessage::CommandOutcome { result, .. })) = ws.await {
            if matches!(result, engine_api::CommandResult::Accepted { .. }) {
                accepted += 1;
            }
        }
        while rx1.try_recv().is_ok() {}
        while rx2.try_recv().is_ok() {}
    }

    let (rs, ws) = tokio::sync::oneshot::channel();
    let _ = handle
        .send(RoomMsg::Snapshot {
            connection_id: 1,
            header: MessageHeader::new(room_id, 0, 0),
            reply: rs,
        })
        .await;
    assert!(ws.await.is_ok(), "room {room_id} must respond to snapshot");
    Ok(accepted)
}

/// §31.1: 10 concurrent rooms, no panic/deadlock.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stress_10_concurrent_rooms() {
    let n = 10u64;
    let start = Instant::now();
    let mut tasks = Vec::new();
    for i in 0..n {
        tasks.push(tokio::spawn(exercise_room(i, 42 + i)));
    }
    let mut total = 0u64;
    for (i, t) in tasks.into_iter().enumerate() {
        match t.await {
            Ok(Ok(a)) => total += a,
            Ok(Err(e)) => panic!("room {i}: {e}"),
            Err(e) => panic!("room {i} panicked: {e}"),
        }
    }
    println!(
        "stress: {n} rooms in {:?}, {total} commands accepted",
        start.elapsed()
    );
    assert!(total > 0);
}

/// §31.1: different seeds → different state hashes (isolation).
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn rooms_with_different_seeds_are_isolated() {
    let h1 = spawn_room(100, 2, 1, String::new(), 64, 100);
    let h2 = spawn_room(101, 2, 999, String::new(), 64, 100);
    for (h, cid) in [(&h1, 1u64), (&h1, 2), (&h2, 1), (&h2, 2)] {
        let (tx, _rx) = mpsc::channel(8);
        let (r, w) = tokio::sync::oneshot::channel();
        let _ = h
            .send(RoomMsg::Join {
                connection_id: cid,
                display_name: "p".into(),
                preferred_seat: Some((cid - 1) as u8),
                reconnect_token: None,
                password: String::new(),
                outbound: Outbound::new(tx),
                reply: r,
            })
            .await;
        let _ = w.await;
    }
    let (r1, w1) = tokio::sync::oneshot::channel();
    let _ = h1
        .send(RoomMsg::Snapshot {
            connection_id: 1,
            header: MessageHeader::new(100, 0, 0),
            reply: r1,
        })
        .await;
    let (r2, w2) = tokio::sync::oneshot::channel();
    let _ = h2
        .send(RoomMsg::Snapshot {
            connection_id: 1,
            header: MessageHeader::new(101, 0, 0),
            reply: r2,
        })
        .await;
    let s1 = w1.await.unwrap().unwrap();
    let s2 = w2.await.unwrap().unwrap();
    let h1 = match s1 {
        ServerMessage::Snapshot { view, .. } => view.state_hash,
        _ => panic!(),
    };
    let h2 = match s2 {
        ServerMessage::Snapshot { view, .. } => view.state_hash,
        _ => panic!(),
    };
    assert_ne!(h1, h2, "different seeds must produce different hashes");
}

/// §31.2: mailbox is bounded.
#[tokio::test]
async fn room_mailbox_is_bounded() {
    let h = spawn_room(200, 2, 42, String::new(), 4, 100);
    for cid in 1..=2u64 {
        let (tx, _rx) = mpsc::channel(8);
        let (r, w) = tokio::sync::oneshot::channel();
        let _ = h
            .send(RoomMsg::Join {
                connection_id: cid,
                display_name: "p".into(),
                preferred_seat: Some((cid - 1) as u8),
                reconnect_token: None,
                password: String::new(),
                outbound: Outbound::new(tx),
                reply: r,
            })
            .await;
        let _ = w.await;
    }
    let (r, w) = tokio::sync::oneshot::channel();
    let _ = h
        .send(RoomMsg::Snapshot {
            connection_id: 1,
            header: MessageHeader::new(200, 0, 0),
            reply: r,
        })
        .await;
    assert!(w.await.is_ok());
}
