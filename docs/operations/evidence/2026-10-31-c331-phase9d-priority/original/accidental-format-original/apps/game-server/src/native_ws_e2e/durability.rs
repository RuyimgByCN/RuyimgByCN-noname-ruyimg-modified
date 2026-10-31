//! Durable acknowledgement, failed writes, and abrupt-runtime restart over the production WS path.

use super::*;
use crate::native_room_store::StoreFault;

pub(super) async fn pending_choice(a: &mut Peer, b: &mut Peer, spectator: &mut Peer) -> State {
    let mut state = a.snapshot().await;
    for key in [701, 702, 703] {
        let before = state.view.revision;
        let (result, next) = a.submit(a.player, before, key, CommandKind::EndPhase).await;
        assert!(
            matches!(result, CommandResult::Accepted { new_revision, .. } if new_revision > before)
        );
        assert_synchronized(&next, &b.pushed(key).await, &spectator.pushed(key).await);
        state = next;
    }
    assert!(state.choice.is_some());
    assert!(state
        .legal
        .iter()
        .any(|action| matches!(action, LegalAction::ResolveChoice { .. })));
    state
}

pub(super) fn answer(state: &State, player: PlayerId) -> CommandKind {
    state
        .legal
        .iter()
        .find(|action| {
            matches!(
                action,
                LegalAction::ResolveChoice {
                    cancelled: false,
                    ..
                }
            )
        })
        .expect("pending private answer")
        .to_command(player, state.view.revision)
        .kind
}

pub(super) fn decode_snapshot(value: &serde_json::Value) -> engine_native::snap::StateSnapshot {
    let hex = value.as_str().unwrap();
    assert_eq!(hex.len() % 2, 0);
    let bytes: Vec<_> = (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect();
    engine_native::archive::decode_archive(&bytes).unwrap()
}

pub(super) async fn failed_submit(peer: &mut Peer, state: &State, key: u64) {
    peer.seq += 1;
    let sequence = peer.seq;
    peer.send(ClientMessage::SubmitCommand {
        header: MessageHeader::new(peer.room, sequence, state.view.revision),
        command: Command {
            actor: peer.player,
            expected_revision: ExpectedRevision(state.view.revision),
            idempotency_key: IdempotencyKey(key),
            kind: answer(state, peer.player),
        },
        idempotency_key: IdempotencyKey(key),
        replay: false,
    })
    .await;
    let response = peer
        .until(|message| {
            matches!(message, ServerMessage::Error { .. })
                || matches!(message, ServerMessage::CommandOutcome { header, .. } if header.conn_seq == sequence)
        })
        .await;
    assert!(
        matches!(
            &response,
            ServerMessage::Error {
                code: ErrorCode::Internal,
                message,
                ..
            } if message.contains("native room persistence failed:")
        ),
        "a failed durable write must not acknowledge an accepted command: {response:?}"
    );
}

pub(super) async fn unchanged_without_success_push(peer: &mut Peer, expected: &State) {
    peer.seq += 1;
    let sequence = peer.seq;
    peer.send(ClientMessage::RequestSnapshot {
        header: MessageHeader::new(peer.room, sequence, 0),
        last_revision: 0,
    })
    .await;
    let message = peer
        .until(|message| {
            assert!(
                !matches!(message, ServerMessage::CommandOutcome {
                    result: CommandResult::Accepted { new_revision, .. }, ..
                } if *new_revision > expected.view.revision),
                "uncommitted transition reached another socket: {message:?}"
            );
            matches!(message, ServerMessage::Snapshot { header, .. } if header.conn_seq == sequence)
        })
        .await;
    assert_eq!(State::from_message(message), *expected);
}

#[test]
fn native_extensions_ws_write_failure_retries_and_restarts() {
    for fault in [
        None,
        Some(StoreFault::PartialWrite),
        Some(StoreFault::FileSync),
        Some(StoreFault::Rename),
        Some(StoreFault::DirectorySync),
    ] {
        write_failure_retries_and_restarts(fault);
    }
}

#[test]
fn native_extensions_ws_restart_after_uncertain_rename_restores_only_confirmed_choice() {
    for fault in [
        StoreFault::DirectorySync,
        StoreFault::DecisionWrite,
        StoreFault::DecisionSync,
        StoreFault::DecisionDirectorySync,
    ] {
        restart_after_uncertain_rename_restores_only_confirmed_choice(fault);
    }
}

fn restart_after_uncertain_rename_restores_only_confirmed_choice(fault: StoreFault) {
    let directory = tempfile::tempdir().unwrap();
    let cfg = local_config(directory.path());
    let (room, tokens, before_a, before_b, before_spectator, confirmed) =
        runtime().block_on(async {
            let server = Server::start(&cfg, 0).await;
            let mut a = Peer::connect(&server).await;
            a.create().await;
            let room = a.room;
            let mut b = Peer::connect(&server).await;
            b.join(room, 1, None).await;
            let mut spectator = Peer::connect(&server).await;
            spectator.spectate(room).await;
            let before_a = pending_choice(&mut a, &mut b, &mut spectator).await;
            let before_b = b.snapshot().await;
            let before_spectator = spectator.snapshot().await;
            let confirmed: serde_json::Value = serde_json::from_slice(
                &std::fs::read(Path::new(&cfg.native_room_store).join(format!("{room}.json")))
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(
                confirmed["state"]["frozen_packages"]
                    .as_array()
                    .unwrap()
                    .len(),
                2
            );
            server
                .lobby
                .native_room_store_for_test()
                .unwrap()
                .fail_next_write(fault);
            failed_submit(&mut a, &before_a, 704).await;
            let candidate: crate::native_room_store::StoredRoom = serde_json::from_slice(
                &std::fs::read(Path::new(&cfg.native_room_store).join(format!("{room}.json")))
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(
                candidate.checksum,
                crate::native_room_store::state_checksum(&candidate.state)
            );
            assert_ne!(
                serde_json::json!(candidate.state.snapshot_hex),
                confirmed["state"]["snapshot_hex"],
                "the interrupted rename must expose a complete newer candidate before restart"
            );
            let tokens = [a.token.clone(), b.token.clone()];
            // No retry, reconnect, or repair save may turn the failed candidate into a commit.
            drop(a);
            drop(b);
            drop(spectator);
            server.stop().await;
            (
                room,
                tokens,
                before_a,
                before_b,
                before_spectator,
                confirmed,
            )
        });

    disable_packages(&cfg);
    runtime().block_on(async {
        let restarted = Server::start(&cfg, 1).await;
        let mut a = Peer::connect(&restarted).await;
        assert_eq!(a.join(room, 0, Some(tokens[0].clone())).await, before_a,
            "rename without a confirmed commit must preserve the original private choice");
        assert_eq!(a.token, tokens[0]);
        let mut b = Peer::connect(&restarted).await;
        assert_eq!(b.join(room, 1, Some(tokens[1].clone())).await, before_b);
        assert_eq!(b.token, tokens[1]);
        let mut spectator = Peer::connect(&restarted).await;
        assert_eq!(spectator.spectate(room).await, before_spectator);
        assert_synchronized(&a.snapshot().await, &b.snapshot().await, &spectator.snapshot().await);
        let restored: serde_json::Value = serde_json::from_slice(
            &std::fs::read(Path::new(&cfg.native_room_store).join(format!("{room}.json"))).unwrap(),
        )
        .unwrap();
        let mut expected_snapshot = decode_snapshot(&confirmed["state"]["snapshot_hex"]);
        // Engine restore intentionally rotates old authorization tokens once (N11-a).
        expected_snapshot.auth_generation += 1;
        assert_eq!(decode_snapshot(&restored["state"]["snapshot_hex"]), expected_snapshot,
            "recovery preserves the complete core, RNG, dedup, and pending resolution");
        for field in ["pack_refs", "frozen_packages", "next_dedup"] {
            assert_eq!(restored["state"][field], confirmed["state"][field], "restored {field}");
        }
        let (result, after_a) = a
            .submit(a.player, before_a.view.revision, 704, answer(&before_a, a.player))
            .await;
        assert!(matches!(result, CommandResult::Accepted { new_revision, .. }
            if new_revision == before_a.view.revision + 1));
        assert_synchronized(&after_a, &b.pushed(704).await, &spectator.pushed(704).await);
        assert_ne!(after_a.view.state_hash, before_a.view.state_hash);
        println!("NATIVE_WS uncertain_rename=pass fault={fault:?} repair_before_restart=none packages=disabled committed_revision={} retried_same_key=704 continued_revision={}", before_a.view.revision, after_a.view.revision);
        restarted.stop().await;
    });
}

#[test]
fn native_extensions_ws_uncertain_initial_save_is_not_restored_as_a_room() {
    for fault in [
        StoreFault::DirectorySync,
        StoreFault::DecisionWrite,
        StoreFault::DecisionSync,
        StoreFault::DecisionDirectorySync,
    ] {
        uncertain_initial_save_is_not_restored_as_a_room(fault);
    }
}

fn uncertain_initial_save_is_not_restored_as_a_room(fault: StoreFault) {
    let directory = tempfile::tempdir().unwrap();
    let cfg = local_config(directory.path());
    runtime().block_on(async {
        let server = Server::start(&cfg, 0).await;
        server
            .lobby
            .native_room_store_for_test()
            .unwrap()
            .fail_next_write(fault);
        let mut peer = Peer::connect(&server).await;
        peer.send(ClientMessage::CreateRoom {
            choice_timeout_secs: None,
            players: 2,
            seed: SEED,
            password: PASSWORD.into(),
        })
        .await;
        peer.error(ErrorCode::Internal).await;
        assert_eq!(server.lobby.stats().active_rooms, 0);
        drop(peer);
        server.stop().await;
    });
    runtime().block_on(async {
        let restarted = Server::start(&cfg, 0).await;
        let mut peer = Peer::connect(&restarted).await;
        peer.send(ClientMessage::JoinRoom {
            room_id: 1,
            password: PASSWORD.into(),
            preferred_seat: Some(0),
            reconnect_token: None,
        })
        .await;
        peer.error(ErrorCode::RoomNotFound).await;
        peer.create().await;
        assert!(peer.room > 1, "the failed candidate still reserves its room ID");
        println!("NATIVE_WS uncertain_first_save=pass fault={fault:?} unacknowledged_room=not_restored next_room={}", peer.room);
        restarted.stop().await;
    });
}

fn write_failure_retries_and_restarts(fault: Option<StoreFault>) {
    let directory = tempfile::tempdir().unwrap();
    let cfg = local_config(directory.path());
    let (room, tokens, before_a, before_b, before_spectator) = runtime().block_on(async {
        let server = Server::start(&cfg, 0).await;
        let mut a = Peer::connect(&server).await;
        a.create().await;
        let room = a.room;
        let mut b = Peer::connect(&server).await;
        b.join(room, 1, None).await;
        let mut spectator = Peer::connect(&server).await;
        spectator.spectate(room).await;
        let before_a = pending_choice(&mut a, &mut b, &mut spectator).await;
        let before_b = b.snapshot().await;
        let before_spectator = spectator.snapshot().await;
        let rooms = Path::new(&cfg.native_room_store);
        let record_name = format!("{room}.json");
        let saved = std::fs::read(rooms.join(&record_name)).unwrap();
        let original = directory.path().join("confirmed-rooms");
        let store = server.lobby.native_room_store_for_test().unwrap();
        if let Some(fault) = fault {
            store.fail_next_write(fault);
        } else {
            std::fs::rename(rooms, &original).unwrap();
            std::fs::write(rooms, b"fault: room directory is unavailable").unwrap();
        }

        failed_submit(&mut a, &before_a, 704).await;
        unchanged_without_success_push(&mut a, &before_a).await;
        unchanged_without_success_push(&mut b, &before_b).await;
        unchanged_without_success_push(&mut spectator, &before_spectator).await;
        if matches!(fault, Some(StoreFault::DirectorySync)) {
            assert_ne!(std::fs::read(rooms.join(&record_name)).unwrap(), saved,
                "after rename the candidate is visible but durability has not been confirmed");
            store.fail_next_write(StoreFault::Rename);
            failed_submit(&mut a, &before_a, 704).await;
            unchanged_without_success_push(&mut a, &before_a).await;
            unchanged_without_success_push(&mut b, &before_b).await;
            unchanged_without_success_push(&mut spectator, &before_spectator).await;
        } else if fault.is_some() {
            assert_eq!(std::fs::read(rooms.join(&record_name)).unwrap(), saved);
        } else {
            assert_eq!(std::fs::read(original.join(&record_name)).unwrap(), saved);
            std::fs::remove_file(rooms).unwrap();
            std::fs::rename(&original, rooms).unwrap();
        }
        let (result, after_a) = a
            .submit(a.player, before_a.view.revision, 704, answer(&before_a, a.player))
            .await;
        assert!(matches!(result, CommandResult::Accepted { new_revision, .. } if new_revision == before_a.view.revision + 1));
        let after_b = b.pushed(704).await;
        let after_spectator = spectator.pushed(704).await;
        assert_synchronized(&after_a, &after_b, &after_spectator);
        let disk: serde_json::Value =
            serde_json::from_slice(&std::fs::read(rooms.join(&record_name)).unwrap()).unwrap();
        let old: serde_json::Value = serde_json::from_slice(&saved).unwrap();
        assert_eq!(disk["state"]["frozen_packages"], old["state"]["frozen_packages"]);
        assert_eq!(disk["state"]["pack_refs"], old["state"]["pack_refs"]);
        assert_ne!(disk["state"]["snapshot_hex"], old["state"]["snapshot_hex"]);
        let (repeated, repeated_state) = a
            .submit(a.player, before_a.view.revision, 704, answer(&before_a, a.player))
            .await;
        assert!(matches!(repeated, CommandResult::Accepted { new_revision, .. } if new_revision == after_a.view.revision));
        assert_eq!(repeated_state, after_a, "successful retry is cached only after durable commit");
        let tokens = [a.token.clone(), b.token.clone()];
        // No graceful close: destroying this runtime drops its sockets and all room actors.
        drop(a);
        drop(b);
        drop(spectator);
        server.stop().await;
        (room, tokens, after_a, after_b, after_spectator)
    });

    disable_packages(&cfg);
    runtime().block_on(async {
        let restarted = Server::start(&cfg, 1).await;
        let mut a = Peer::connect(&restarted).await;
        assert_eq!(a.join(room, 0, Some(tokens[0].clone())).await, before_a);
        assert_eq!(a.token, tokens[0]);
        let mut b = Peer::connect(&restarted).await;
        assert_eq!(b.join(room, 1, Some(tokens[1].clone())).await, before_b);
        assert_eq!(b.token, tokens[1]);
        let mut spectator = Peer::connect(&restarted).await;
        assert_eq!(spectator.spectate(room).await, before_spectator);
        assert_synchronized(&a.snapshot().await, &b.snapshot().await, &spectator.snapshot().await);
        let action = before_a.legal.first().expect("restored game can continue").to_command(a.player, before_a.view.revision);
        let (result, next) = a.submit(a.player, before_a.view.revision, 705, action.kind).await;
        assert!(matches!(result, CommandResult::Accepted { new_revision, .. } if new_revision > before_a.view.revision));
        assert_synchronized(&next, &b.pushed(705).await, &spectator.pushed(705).await);
        println!("NATIVE_WS durability=pass fault={fault:?} retry=same-key revision={} restart=abrupt-runtime continued_revision={} packages=disabled", before_a.view.revision, next.view.revision);
        restarted.stop().await;
    });
}

#[test]
fn native_extensions_ws_restart_isolates_corrupt_saves_and_ignores_orphan_temps() {
    let directory = tempfile::tempdir().unwrap();
    let cfg = local_config(directory.path());
    let (room, token, before, record) = runtime().block_on(async {
        let server = Server::start(&cfg, 0).await;
        let mut a = Peer::connect(&server).await;
        a.create().await;
        let room = a.room;
        let mut b = Peer::connect(&server).await;
        b.join(room, 1, None).await;
        let mut spectator = Peer::connect(&server).await;
        spectator.spectate(room).await;
        let before = pending_choice(&mut a, &mut b, &mut spectator).await;
        let record: crate::native_room_store::StoredRoom = serde_json::from_slice(
            &std::fs::read(Path::new(&cfg.native_room_store).join(format!("{room}.json"))).unwrap(),
        )
        .unwrap();
        let mut other = Peer::connect(&server).await;
        other.create().await;
        assert_ne!(other.room, room);
        let token = a.token.clone();
        drop(a);
        drop(b);
        drop(spectator);
        drop(other);
        server.stop().await;
        (room, token, before, record)
    });

    let rooms = Path::new(&cfg.native_room_store);
    let content = super::super::native::NativeContent::open(
        cfg.native_packages_root.clone(),
        cfg.native_store_root.clone(),
    )
    .unwrap();
    let (packs, _) = content
        .load_room_packages(
            &record.state.pack_refs,
            record.state.frozen_packages.as_deref(),
        )
        .unwrap();
    // A fully valid, newer candidate models a crash after file sync but before atomic rename.
    let mut candidate =
        crate::room_native::NativeRoomSession::from_saved(&record.state, packs).unwrap();
    let owner = record
        .state
        .members
        .iter()
        .find(|member| member.player_id == PlayerId::new(1))
        .unwrap();
    let outcome = candidate
        .submit(
            owner.connection_id,
            &answer(&before, owner.player_id),
            before.view.revision,
        )
        .unwrap();
    assert!(
        matches!(outcome, CommandResult::Accepted { new_revision, .. } if new_revision > before.view.revision)
    );
    let mut newer = record.clone();
    newer.state = candidate.export_state(&candidate.snapshot_hex());
    newer.checksum = crate::native_room_store::state_checksum(&newer.state);
    let full_temp = serde_json::to_vec_pretty(&newer).unwrap();
    let complete_path = rooms.join(format!(
        "{room}.json.00000000-0000-4000-8000-000000000001.tmp"
    ));
    std::fs::write(&complete_path, &full_temp).unwrap();
    let partial_path = rooms.join(format!("{room}.json.tmp"));
    let partial = b"{\"room_id\":1,\"state\":{\"snapshot_hex\":\"";
    std::fs::write(&partial_path, partial).unwrap();
    let corrupt_path = rooms.join("100.json");
    let corrupt = b"{\"room_id\":100,\"state\":";
    std::fs::write(&corrupt_path, corrupt).unwrap();
    let mismatch_path = rooms.join("101.json");
    let mismatch = serde_json::to_vec_pretty(&record).unwrap();
    std::fs::write(&mismatch_path, &mismatch).unwrap();
    let orphan_path = rooms.join("102.json.00000000-0000-4000-8000-000000000102.tmp");
    let mut orphan = newer;
    orphan.room_id = 102;
    let orphan_bytes = serde_json::to_vec_pretty(&orphan).unwrap();
    std::fs::write(&orphan_path, &orphan_bytes).unwrap();
    disable_packages(&cfg);

    runtime().block_on(async {
        let restarted = Server::start(&cfg, 2).await;
        let mut a = Peer::connect(&restarted).await;
        assert_eq!(a.join(room, 0, Some(token.clone())).await, before,
            "a fully valid temp must not replace the last committed choice");
        assert_eq!(a.token, token);
        let mut unavailable = Peer::connect(&restarted).await;
        for unavailable_room in [100, 101, 102] {
            unavailable.send(ClientMessage::JoinRoom {
                room_id: unavailable_room,
                password: PASSWORD.into(),
                preferred_seat: Some(0),
                reconnect_token: None,
            }).await;
            unavailable.error(ErrorCode::RoomNotFound).await;
        }
        for (path, bytes) in [
            (&complete_path, full_temp.as_slice()),
            (&partial_path, partial.as_slice()),
            (&corrupt_path, corrupt.as_slice()),
            (&mismatch_path, mismatch.as_slice()),
            (&orphan_path, orphan_bytes.as_slice()),
        ] {
            assert_eq!(std::fs::read(path).unwrap(), bytes, "failed/orphaned save remains available for inspection");
        }
        let (outcome, continued) = a.submit(a.player, before.view.revision, 704, answer(&before, a.player)).await;
        assert!(matches!(outcome, CommandResult::Accepted { new_revision, .. } if new_revision > before.view.revision));
        let mut active = Peer::connect(&restarted).await;
        active.send(ClientMessage::CreateRoom { choice_timeout_secs: None, players: 2, seed: SEED, password: PASSWORD.into() }).await;
        active.error(ErrorCode::Internal).await;
        assert_eq!(restarted.lobby.stats().active_rooms, 2, "disabled packs cannot create a new room");
        println!("NATIVE_WS crash_recovery=pass healthy_rooms=2 corrupt_rooms=2 orphan_room=102 committed_revision={} continued_revision={}", before.view.revision, continued.view.revision);
        restarted.stop().await;
    });

    // Reactivating packages permits a new room, while all damaged and uncommitted IDs remain reserved.
    upgrade_packages(&cfg, "1.1.0");
    runtime().block_on(async {
        let restarted = Server::start(&cfg, 2).await;
        let mut new = Peer::connect(&restarted).await;
        new.create().await;
        assert!(
            new.room > 102,
            "new room must not overwrite malformed or uncommitted saves"
        );
        assert_eq!(std::fs::read(&corrupt_path).unwrap(), corrupt);
        assert_eq!(std::fs::read(&mismatch_path).unwrap(), mismatch);
        assert_eq!(std::fs::read(&orphan_path).unwrap(), orphan_bytes);
        println!(
            "NATIVE_WS reserved_ids=pass new_room={} corrupt_and_orphan_bytes=preserved",
            new.room
        );
        restarted.stop().await;
    });
}

#[test]
fn native_extensions_ws_refuses_room_creation_when_recovery_inventory_is_unsafe() {
    for failure in ["unreadable-directory", "exhausted-id"] {
        let directory = tempfile::tempdir().unwrap();
        let cfg = local_config(directory.path());
        let rooms = Path::new(&cfg.native_room_store);
        if failure == "unreadable-directory" {
            std::fs::write(rooms, b"fault: not a directory").unwrap();
        } else {
            std::fs::create_dir_all(rooms).unwrap();
            std::fs::write(rooms.join(format!("{}.json", u64::MAX)), b"{invalid").unwrap();
        }
        // 无法建立目录或许可证时，单写者锁在启动阶段就拒绝，而不是启动一个
        // 扫描/分配房号都不可靠的服务。exhausted-id 仍可启动但拒绝新房间。
        if failure == "unreadable-directory" {
            match crate::try_bootstrap(&cfg) {
                Ok(_) => panic!("存档目录不是目录时必须拒绝启动"),
                Err(error) => {
                    println!("NATIVE_WS recovery_inventory={failure} startup=refused ({error})")
                }
            }
            continue;
        }
        runtime().block_on(async {
            let lobby = bootstrap(&cfg);
            let restore = lobby.restore_native_rooms();
            assert_eq!(restore.unwrap(), 0);
            let app = build_router(lobby.clone(), ServeDir::new(&cfg.original_content_root));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("ws://{}/ws", listener.local_addr().unwrap());
            let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
            let server = Server { url, lobby, task };
            let mut peer = Peer::connect(&server).await;
            peer.send(ClientMessage::CreateRoom {
                choice_timeout_secs: None,
                players: 2,
                seed: SEED,
                password: PASSWORD.into(),
            })
            .await;
            peer.error(ErrorCode::Internal).await;
            assert_eq!(server.lobby.stats().active_rooms, 0);
            println!("NATIVE_WS recovery_inventory={failure} new_room=refused");
            server.stop().await;
        });
    }
}
