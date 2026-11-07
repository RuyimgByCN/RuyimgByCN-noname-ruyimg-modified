//! Native actor storage failures must not acknowledge or publish an uncommitted action.
#[path = "../src/native.rs"]
mod native;
#[path = "../src/native_room_store.rs"]
mod native_room_store;
#[path = "../src/native_view.rs"]
mod native_view;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/room_native.rs"]
mod room_native;
#[path = "../src/room_native_actor.rs"]
mod room_native_actor;
#[path = "../src/session.rs"]
mod session;

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_protocol::{MessageHeader, ServerMessage};
use native_room_store::NativeRoomStore;
use room::RoomMsg;
use room_native::NativeRoomSession;

async fn join(
    handle: &room::RoomHandle,
    conn: u64,
    token: Option<String>,
) -> Result<room::JoinReply, String> {
    let (tx, _rx) = tokio::sync::mpsc::channel(128);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: conn,
            display_name: format!("p{conn}"),
            preferred_seat: None,
            reconnect_token: token,
            password: String::new(),
            outbound: session::Outbound::new(tx),
            reply,
        })
        .await?;
    response.await.unwrap()
}

async fn end_phase(
    handle: &room::RoomHandle,
    conn: u64,
    key: u64,
) -> Result<ServerMessage, String> {
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Submit {
            connection_id: conn,
            header: MessageHeader::new(handle.id, key, 0),
            command: Command {
                actor: PlayerId::new(1),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::EndPhase {},
            },
            idempotency_key: IdempotencyKey(key),
            replay: false,
            reply,
        })
        .await?;
    response.await.unwrap()
}

async fn snapshot(handle: &room::RoomHandle, conn: u64) -> engine_api::ObserverSnapshot {
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Snapshot {
            connection_id: conn,
            header: MessageHeader::new(handle.id, 0, 0),
            reply,
        })
        .await
        .unwrap();
    match response.await.unwrap().unwrap() {
        ServerMessage::Snapshot { view, .. } => view,
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn nc09_write_failure_does_not_ack_or_advance_and_same_key_can_retry() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("rooms");
    let store = NativeRoomStore::new(&root);
    let session = NativeRoomSession::new(engine_native::Engine::new(4242, 2), 2);
    let handle = room_native_actor::spawn_room_native_persisted(
        10,
        session,
        16,
        5,
        String::new(),
        Some(store.clone()),
    );
    let joined = join(&handle, 1, None).await.unwrap();
    let bytes = std::fs::read(root.join("10.json")).unwrap();
    let original = dir.path().join("rooms-original");
    std::fs::rename(&root, &original).unwrap();
    std::fs::write(&root, b"injected parent write failure").unwrap();
    let failed = end_phase(&handle, 1, 81).await;
    assert!(
        failed.is_err(),
        "storage failure must not acknowledge Accepted: {failed:?}"
    );
    assert_eq!(
        snapshot(&handle, 1).await,
        joined.view,
        "failed write changed confirmed memory"
    );
    std::fs::remove_file(&root).unwrap();
    std::fs::rename(&original, &root).unwrap();
    assert_eq!(std::fs::read(root.join("10.json")).unwrap(), bytes);
    let accepted = end_phase(&handle, 1, 81).await.unwrap();
    assert!(matches!(
        accepted,
        ServerMessage::CommandOutcome {
            result: CommandResult::Accepted {
                new_revision: 1,
                ..
            },
            ..
        }
    ));
    assert_eq!(snapshot(&handle, 1).await.revision, 1);
    let saved = store.load_all().unwrap().pop().unwrap();
    assert_eq!(
        saved.state.next_dedup, 2,
        "failed action must not consume durable dedup"
    );
}

#[tokio::test]
async fn nc09_all_write_stages_rollback_and_uncertain_commit_blocks_until_reconciled() {
    use native_room_store::StoreFault;
    for fault in [
        StoreFault::PartialWrite,
        StoreFault::FileSync,
        StoreFault::Rename,
        StoreFault::DirectorySync,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let store = NativeRoomStore::new(dir.path());
        let handle = room_native_actor::spawn_room_native_persisted(
            20,
            NativeRoomSession::new(engine_native::Engine::new(4242, 2), 2),
            16,
            5,
            String::new(),
            Some(store.clone()),
        );
        let joined = join(&handle, 1, None).await.unwrap();
        let before = std::fs::read(dir.path().join("20.json")).unwrap();
        store.fail_next_write(fault);
        let error = end_phase(&handle, 1, 91).await.unwrap_err();
        assert!(
            error.starts_with("native room persistence failed:"),
            "{fault:?}: {error}"
        );
        assert_eq!(snapshot(&handle, 1).await, joined.view, "{fault:?}");
        let confirmed_before: native_room_store::StoredRoom =
            serde_json::from_slice(&before).unwrap();
        let recovery = NativeRoomStore::new(dir.path()).load_all().unwrap();
        assert_eq!(recovery.len(), 1);
        assert_eq!(
            native_room_store::state_checksum(&recovery[0].state),
            native_room_store::state_checksum(&confirmed_before.state),
            "{fault:?}: a fresh store must recover only the confirmed actor state"
        );
        if fault == StoreFault::DirectorySync {
            assert_ne!(
                std::fs::read(dir.path().join("20.json")).unwrap(),
                before,
                "rename already replaced the candidate, so disk rollback cannot be claimed"
            );
            // Reconciliation itself fails: no later mutation may bypass the uncertainty guard.
            store.fail_next_write(StoreFault::Rename);
            assert!(end_phase(&handle, 1, 91).await.is_err());
            assert_eq!(snapshot(&handle, 1).await, joined.view);
        } else {
            assert_eq!(
                std::fs::read(dir.path().join("20.json")).unwrap(),
                before,
                "{fault:?} damaged committed record"
            );
        }
        let accepted = end_phase(&handle, 1, 91).await.unwrap();
        let hash = match accepted {
            ServerMessage::CommandOutcome {
                result:
                    CommandResult::Accepted {
                        new_revision: 1,
                        state_hash,
                    },
                ..
            } => state_hash,
            other => panic!("{fault:?}: {other:?}"),
        };
        let mut reference = NativeRoomSession::new(engine_native::Engine::new(4242, 2), 2);
        reference.join(1, Some(0), None).unwrap();
        let expected = reference.submit(1, &CommandKind::EndPhase {}, 0).unwrap();
        assert_eq!(
            expected,
            CommandResult::Accepted {
                new_revision: 1,
                state_hash: hash
            },
            "failed attempt consumed RNG/engine state"
        );
        let saved = store.load_all().unwrap().pop().unwrap();
        assert_eq!(saved.state.next_dedup, 2);
        assert_eq!(saved.state.next_token, 2);
        assert_eq!(snapshot(&handle, 1).await.state_hash, hash);
        assert!(
            matches!(
                end_phase(&handle, 1, 91).await.unwrap(),
                ServerMessage::CommandOutcome {
                    result: CommandResult::Accepted {
                        new_revision: 1,
                        ..
                    },
                    ..
                }
            ),
            "successful retry was applied twice"
        );
        println!("NATIVE_ACTOR fault={fault:?} rollback=confirmed retry=once");
    }
}

async fn spectate(handle: &room::RoomHandle, conn: u64) -> Result<room::JoinReply, String> {
    let (tx, _rx) = tokio::sync::mpsc::channel(128);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Spectate {
            connection_id: conn,
            display_name: "spectator".into(),
            outbound: session::Outbound::new(tx),
            reply,
        })
        .await?;
    response.await.unwrap()
}

#[tokio::test]
async fn nc09_membership_failures_preserve_seats_tokens_and_reconnect_authority() {
    use native_room_store::StoreFault;
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    let handle = room_native_actor::spawn_room_native_persisted(
        30,
        NativeRoomSession::new(engine_native::Engine::new(4242, 2), 2),
        16,
        5,
        String::new(),
        Some(store.clone()),
    );
    let owner = join(&handle, 1, None).await.unwrap();
    let before = std::fs::read(dir.path().join("30.json")).unwrap();
    store.fail_next_write(StoreFault::Rename);
    assert!(join(&handle, 2, None).await.is_err());
    assert_eq!(std::fs::read(dir.path().join("30.json")).unwrap(), before);
    let peer = join(&handle, 2, None).await.unwrap();
    assert_eq!(peer.seat, 1);
    let record = store.load_all().unwrap().pop().unwrap();
    assert_eq!(record.state.members.len(), 2);
    assert_eq!(
        record.state.next_token, 3,
        "failed join consumed a token/seat"
    );
    store.fail_next_write(StoreFault::PartialWrite);
    assert!(join(&handle, 11, Some(owner.reconnect_token.clone()))
        .await
        .is_err());
    assert_eq!(
        snapshot(&handle, 1).await,
        owner.view,
        "failed reconnect revoked the confirmed connection"
    );
    let rebound = join(&handle, 11, Some(owner.reconnect_token.clone()))
        .await
        .unwrap();
    assert_eq!(rebound.reconnect_token, owner.reconnect_token);
    assert_eq!(rebound.seat, owner.seat);
    let record = store.load_all().unwrap().pop().unwrap();
    assert!(record.state.members.iter().any(|m| m.connection_id == 11));
    assert!(!record.state.members.iter().any(|m| m.connection_id == 1));
    store.fail_next_write(StoreFault::FileSync);
    assert!(spectate(&handle, 3).await.is_err());
    assert_eq!(
        store.load_all().unwrap().pop().unwrap().state.members.len(),
        2
    );
    assert_eq!(
        spectate(&handle, 3).await.unwrap().role,
        engine_protocol::ClientRole::Spectator
    );
    assert_eq!(
        store.load_all().unwrap().pop().unwrap().state.members.len(),
        3
    );
}

async fn choose_general(handle: &room::RoomHandle, name: &str) -> Result<String, String> {
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::ChooseGeneral {
            connection_id: 1,
            name: name.into(),
            reply,
        })
        .await?;
    response.await.unwrap()
}

#[tokio::test]
async fn nc09_general_failure_does_not_publish_or_replace_recovery_point() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    let engine = engine_native::Engine::new(4242, 2);
    let name = engine
        .general_candidates()
        .into_iter()
        .filter(|def| *def != 0)
        .find_map(|def| engine.catalog().general_meta(def).map(|m| m.0.to_owned()))
        .unwrap();
    let handle = room_native_actor::spawn_room_native_persisted(
        40,
        NativeRoomSession::new(engine, 2),
        16,
        5,
        String::new(),
        Some(store.clone()),
    );
    let owner = join(&handle, 1, None).await.unwrap();
    let before = std::fs::read(dir.path().join("40.json")).unwrap();
    store.fail_next_write(native_room_store::StoreFault::Rename);
    assert!(choose_general(&handle, &name).await.is_err());
    assert_eq!(snapshot(&handle, 1).await, owner.view);
    assert_eq!(std::fs::read(dir.path().join("40.json")).unwrap(), before);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle.send(RoomMsg::Recover { reply }).await.unwrap();
    response.await.unwrap().unwrap();
    assert_eq!(
        snapshot(&handle, 1).await,
        owner.view,
        "failed selection poisoned the in-memory recovery point"
    );
    choose_general(&handle, &name).await.unwrap();
    let selected = snapshot(&handle, 1).await;
    assert_eq!(selected.revision, owner.view.revision);
    assert_ne!(selected.state_hash, owner.view.state_hash);
    let record = store.load_all().unwrap().pop().unwrap();
    let restored = NativeRoomSession::from_saved(&record.state, Vec::new()).unwrap();
    assert_eq!(restored.engine().state_hash(), selected.state_hash);
}

#[tokio::test]
async fn nc09_configured_creation_rejects_failed_initial_save() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        native::NativeContent::open(dir.path().join("sources"), dir.path().join("packages"))
            .unwrap();
    let store = NativeRoomStore::new(dir.path().join("rooms"));
    store.fail_next_write(native_room_store::StoreFault::FileSync);
    let created = room_native_actor::spawn_native_room_if_configured(
        50,
        2,
        4242,
        String::new(),
        16,
        5,
        &content,
        true,
        "test",
        &[],
        Some(store.clone()),
        30,
    );
    assert!(
        matches!(created,Err(error) if error.starts_with("native room persistence failed:")),
        "initial persistence failure must not publish a room handle"
    );
    assert!(store.load_all().unwrap().is_empty());
}

async fn recover(handle: &room::RoomHandle) -> Result<(), String> {
    let (reply, response) = tokio::sync::oneshot::channel();
    handle.send(RoomMsg::Recover { reply }).await?;
    response.await.unwrap()
}

#[tokio::test]
async fn nc09_failed_recover_keeps_confirmed_dedup_and_successful_recover_can_continue() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    let handle = room_native_actor::spawn_room_native_persisted(
        60,
        NativeRoomSession::new(engine_native::Engine::new(4242, 2), 2),
        16,
        0,
        String::new(),
        Some(store.clone()),
    );
    join(&handle, 1, None).await.unwrap();
    end_phase(&handle, 1, 11).await.unwrap();
    let before = snapshot(&handle, 1).await;
    let saved = std::fs::read(dir.path().join("60.json")).unwrap();
    store.fail_next_write(native_room_store::StoreFault::Rename);
    assert!(recover(&handle).await.is_err());
    assert_eq!(snapshot(&handle, 1).await, before);
    assert_eq!(std::fs::read(dir.path().join("60.json")).unwrap(), saved);
    recover(&handle).await.unwrap();
    assert_eq!(snapshot(&handle, 1).await, before);
    assert!(
        matches!(
            end_phase(&handle, 1, 12).await.unwrap(),
            ServerMessage::CommandOutcome {
                result: CommandResult::Accepted {
                    new_revision: 2,
                    ..
                },
                ..
            }
        ),
        "replayed internal dedup must not collide with the next live command"
    );
}

#[tokio::test]
async fn nc09_failed_lazy_reclaim_keeps_original_seat_and_token() {
    room_native_actor::set_seat_timeout_secs(1);
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    let handle = room_native_actor::spawn_room_native_persisted(
        70,
        NativeRoomSession::new(engine_native::Engine::new(4242, 2), 2),
        16,
        0,
        String::new(),
        Some(store.clone()),
    );
    let owner = join(&handle, 1, None).await.unwrap();
    join(&handle, 2, None).await.unwrap();
    handle
        .send(RoomMsg::Disconnect { connection_id: 1 })
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let before = std::fs::read(dir.path().join("70.json")).unwrap();
    store.fail_next_write(native_room_store::StoreFault::Rename);
    assert!(join(&handle, 3, None).await.is_err());
    assert_eq!(std::fs::read(dir.path().join("70.json")).unwrap(), before);
    let rebound = join(&handle, 11, Some(owner.reconnect_token.clone()))
        .await
        .unwrap();
    assert_eq!(rebound.seat, owner.seat);
    assert_eq!(rebound.reconnect_token, owner.reconnect_token);
    handle
        .send(RoomMsg::Disconnect { connection_id: 11 })
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let replacement = join(&handle, 3, None).await.unwrap();
    assert_eq!(replacement.seat, owner.seat);
    assert_ne!(replacement.reconnect_token, owner.reconnect_token);
    assert!(join(&handle, 12, Some(owner.reconnect_token))
        .await
        .is_err());
    room_native_actor::set_seat_timeout_secs(600);
}

#[tokio::test]
async fn nc09_salted_password_hash_v1_and_v2_verification() {
    let v1 = native_room_store::password_hash(42, "hunter2");
    assert!(!v1.starts_with("v2:"));
    assert!(native_room_store::verify_password_hash(42, "hunter2", &v1));
    assert!(!native_room_store::verify_password_hash(42, "wrong", &v1));
    assert!(!native_room_store::verify_password_hash(43, "hunter2", &v1));

    let v2_a = native_room_store::salted_password_hash(42, "hunter2");
    let v2_b = native_room_store::salted_password_hash(42, "hunter2");
    assert!(v2_a.starts_with("v2:"));
    assert!(v2_b.starts_with("v2:"));
    assert_ne!(v2_a, v2_b, "random per-room salt must differentiate two v2 hashes");
    assert!(native_room_store::verify_password_hash(42, "hunter2", &v2_a));
    assert!(native_room_store::verify_password_hash(42, "hunter2", &v2_b));
    assert!(!native_room_store::verify_password_hash(42, "wrong", &v2_a));
    assert!(!native_room_store::verify_password_hash(43, "hunter2", &v2_a));
    assert_eq!(native_room_store::salted_password_hash(42, ""), "");
    assert!(native_room_store::verify_password_hash(42, "", ""));

    let handle = room_native_actor::spawn_room_native_with_password(
        42,
        NativeRoomSession::new(engine_native::Engine::new(4242, 2), 2),
        16,
        0,
        "hunter2".into(),
    );
    assert!(join(&handle, 1, None).await.is_err());
    let (tx, _rx) = tokio::sync::mpsc::channel(128);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 2,
            display_name: "p2".into(),
            preferred_seat: None,
            reconnect_token: None,
            password: "hunter2".into(),
            outbound: session::Outbound::new(tx),
            reply,
        })
        .await
        .unwrap();
    assert!(response.await.unwrap().is_ok());
}

#[tokio::test]
async fn nc12_append_only_wal_segment_compaction_and_crash_replay() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    let handle = room_native_actor::spawn_room_native_persisted(
        80,
        NativeRoomSession::new(engine_native::Engine::new(4242, 2), 2),
        16,
        0,
        String::new(),
        Some(store.clone()),
    );
    join(&handle, 1, None).await.unwrap();
    join(&handle, 2, None).await.unwrap();
    assert!(store.room_path(80).exists());
    assert!(!store.wal_segment_path(80).exists());

    // Advance 3 steps (revisions 1, 2, 3 < DEFAULT_WAL_CHECKPOINT_EVERY = 16).
    for step in 1..=3 {
        let current = snapshot(&handle, 1).await.current_player.raw();
        let conn = u64::from(current);
        assert!(matches!(
            end_phase(&handle, conn, 100 + step).await.unwrap(),
            ServerMessage::CommandOutcome {
                result: CommandResult::Accepted { new_revision, .. },
                ..
            } if new_revision == step
        ));
    }
    let seg_path = store.wal_segment_path(80);
    assert!(seg_path.exists(), "steps 1..=3 must append to 80.wal.jsonl");
    let seg_text = std::fs::read_to_string(&seg_path).unwrap();
    assert_eq!(seg_text.lines().count(), 3);
    let expected_snap = snapshot(&handle, 1).await;

    // Simulate a torn/truncated tail line after crash and verify load_all replays 1..=3 cleanly.
    {
        let mut f = std::fs::OpenOptions::new().append(true).open(&seg_path).unwrap();
        f.write_all(b"{\"revision\":999,\"truncated").unwrap();
    }
    let loaded = store
        .load_all()
        .unwrap()
        .into_iter()
        .find(|r| r.room_id == 80)
        .expect("room 80 loaded with WAL segment replay");
    assert_eq!(loaded.state.wal.as_ref().unwrap().entries.len(), 3);
    assert_eq!(
        loaded.state.wal.as_ref().unwrap().entries.last().unwrap().revision,
        3
    );
    let restored_session =
        NativeRoomSession::from_saved(&loaded.state, Vec::new()).expect("restore room 80 session");
    let restored = room_native_actor::spawn_room_native_persisted(
        80,
        restored_session,
        16,
        0,
        loaded.password_hash.clone(),
        None,
    );
    assert_eq!(snapshot(&restored, 1).await, expected_snap);

    // Repair segment file to clean lines and advance to revision 16 (checkpoint compaction).
    std::fs::write(&seg_path, seg_text).unwrap();
    for step in 4..=native_room_store::DEFAULT_WAL_CHECKPOINT_EVERY {
        let mut chosen = None;
        for c in [1u64, 2u64] {
            let (reply, response) = tokio::sync::oneshot::channel();
            handle
                .send(RoomMsg::Snapshot {
                    connection_id: c,
                    header: MessageHeader::new(handle.id, 0, 0),
                    reply,
                })
                .await
                .unwrap();
            if let ServerMessage::Snapshot {
                view,
                choice: Some(ask),
                ..
            } = response.await.unwrap().unwrap()
            {
                let selections = (0..ask.min_count).map(u32::from).collect();
                chosen = Some((
                    c,
                    view.revision,
                    ask.observer,
                    CommandKind::ResolveChoice {
                        choice_id: ask.choice_id,
                        selections,
                        cancelled: false,
                    },
                ));
                break;
            }
        }
        let (conn, rev, actor, kind) = if let Some(t) = chosen {
            t
        } else {
            let snap = snapshot(&handle, 1).await;
            let c = u64::from(snap.current_player.raw());
            (c, snap.revision, snap.current_player, CommandKind::EndPhase {})
        };
        let key = 200 + step;
        let (reply, response) = tokio::sync::oneshot::channel();
        handle
            .send(RoomMsg::Submit {
                connection_id: conn,
                header: MessageHeader::new(handle.id, key, rev),
                command: Command {
                    actor,
                    expected_revision: ExpectedRevision(rev),
                    idempotency_key: IdempotencyKey(key),
                    kind,
                },
                idempotency_key: IdempotencyKey(key),
                replay: false,
                reply,
            })
            .await
            .unwrap();
        let res = response.await.unwrap().unwrap();
        assert!(
            matches!(
                res,
                ServerMessage::CommandOutcome {
                    result: CommandResult::Accepted { new_revision, .. },
                    ..
                } if new_revision == step
            ),
            "step {step} failed: {res:?}"
        );
    }
    assert_eq!(
        snapshot(&handle, 1).await.revision,
        native_room_store::DEFAULT_WAL_CHECKPOINT_EVERY
    );
    assert!(
        !seg_path.exists(),
        "checkpoint at revision 16 must compact into 80.json and unlink 80.wal.jsonl"
    );
}

#[test]
fn nc12_native_wal_block_signing_and_chain_verification() {
    let mut session = NativeRoomSession::new(engine_native::Engine::new(4242, 2), 2);
    session.join(1, Some(0), None).unwrap();
    session.join(2, Some(1), None).unwrap();
    let mut persist = persist::RoomPersist::new();
    let h0 = persist::native_snapshot_state_hash(&session.snapshot_hex());
    assert_ne!(h0, [0u8; 32]);

    for step in 1..=4u64 {
        let (conn, kind) = if let Some(ask) = session.engine().open_choice() {
            let c = u64::from(ask.observer) + 1;
            let selections = (0..ask.spec.min).map(|i| i as u32).collect();
            (
                c,
                CommandKind::ResolveChoice {
                    choice_id: engine_api::ChoiceId::new(ask.id as u32),
                    selections,
                    cancelled: false,
                },
            )
        } else {
            let active = session.engine().state_core().turn.current;
            (u64::from(active) + 1, CommandKind::EndPhase {})
        };
        let res = session.submit(conn, &kind, step - 1).unwrap();
        assert!(matches!(res, CommandResult::Accepted { .. }));
        room_native_actor::record_committed_native_step(&mut persist, 99, &session, step, 2);
    }
    assert_eq!(persist.blocks().len(), 2);
    assert_eq!(persist.block_signatures().len(), 2);
    assert_eq!(
        persist::RoomPersist::verify_block_chain(persist.blocks()),
        Ok(())
    );
    let (sk, _key_id) = persist::native_wal_signing_key(99);
    assert_eq!(
        persist::RoomPersist::verify_block_signatures(
            persist.blocks(),
            persist.block_signatures(),
            &sk
        ),
        Ok(())
    );
}

