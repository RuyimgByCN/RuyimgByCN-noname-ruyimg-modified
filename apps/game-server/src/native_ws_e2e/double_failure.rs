//! A failed decision and failed compensation must not let stale RAM rewrite the authority.

use super::durability::{
    answer, decode_snapshot, failed_submit, pending_choice, unchanged_without_success_push,
};
use super::*;
use crate::native_room_store::{NativeRoomStore, StoreFault, StoredRoom};

const RETRY_KEY: u64 = 904;

fn committed(primary: StoreFault, compensation: StoreFault) -> bool {
    primary == StoreFault::DecisionDirectorySync && compensation != StoreFault::PrepareDirectorySync
}

pub(super) fn saved_state(cfg: &ServerConfig, record: &StoredRoom, token: &str) -> State {
    let content = crate::native::NativeContent::open(
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
    let session = crate::room_native::NativeRoomSession::from_saved(&record.state, packs).unwrap();
    let member = session
        .members()
        .find(|member| member.reconnect_token == token)
        .unwrap();
    let player = (member.role == ClientRole::Player).then_some(member.player_id);
    State {
        view: session.snapshot(member.connection_id).unwrap(),
        legal: player
            .map(|id| crate::native_view::legal_actions_for(session.engine(), id))
            .unwrap_or_default(),
        choice: crate::native_view::choice_for(session.engine(), member.role.into(), player),
    }
}

pub(super) async fn submit_error(peer: &mut Peer, key: u64, kind: CommandKind) {
    peer.seq += 1;
    let sequence = peer.seq;
    peer.send(ClientMessage::SubmitCommand {
        header: MessageHeader::new(peer.room, sequence, 0),
        command: Command {
            actor: peer.player,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
        idempotency_key: IdempotencyKey(key),
    replay: false,
})
    .await;
    let response = peer.until(|message| matches!(message, ServerMessage::Error { .. })
        || matches!(message, ServerMessage::CommandOutcome { header, .. } if header.conn_seq == sequence)).await;
    assert!(
        matches!(response, ServerMessage::Error { .. }),
        "{response:?}"
    );
}

#[test]
fn native_extensions_ws_double_failure_fences_writes_and_recovers_by_decision() {
    for primary in [
        StoreFault::DecisionWrite,
        StoreFault::DecisionSync,
        StoreFault::DecisionDirectorySync,
    ] {
        for compensation in [
            StoreFault::PrepareWrite,
            StoreFault::PrepareSync,
            StoreFault::PrepareDirectorySync,
        ] {
            double_failure_fences_writes_and_recovers_by_decision(primary, compensation);
        }
    }
}

fn double_failure_fences_writes_and_recovers_by_decision(
    primary: StoreFault,
    compensation: StoreFault,
) {
    let directory = tempfile::tempdir().unwrap();
    let cfg = local_config(directory.path());
    let decision_committed = committed(primary, compensation);
    let (room, tokens, selected, intent) = runtime().block_on(async {
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
        let before_s = spectator.snapshot().await;
        let intent = answer(&before_a, a.player);
        let store = server.lobby.native_room_store_for_test().unwrap();
        store.fail_next_write_with_compensation(primary, compensation);
        failed_submit(&mut a, &before_a, RETRY_KEY).await;
        let record_path = Path::new(&cfg.native_room_store).join(format!("{room}.json"));
        let journal_path = Path::new(&cfg.native_room_store).join(format!("{room}.json.journal"));
        let candidate = std::fs::read(&record_path).unwrap();
        let journal = std::fs::read(&journal_path).unwrap();
        let selected = NativeRoomStore::new(&cfg.native_room_store)
            .load_all()
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(
            saved_state(&cfg, &selected, &a.token).view.revision,
            3 + u64::from(decision_committed)
        );
        unchanged_without_success_push(&mut a, &before_a).await;
        unchanged_without_success_push(&mut b, &before_b).await;
        unchanged_without_success_push(&mut spectator, &before_s).await;
        // The old RAM state must never be written over a visible Committed decision.
        failed_submit(&mut a, &before_a, RETRY_KEY).await;
        let mut newcomer = Peer::connect(&server).await;
        newcomer
            .send(ClientMessage::Spectate { room_id: room })
            .await;
        newcomer.error(ErrorCode::Internal).await;
        assert_eq!(std::fs::read(&record_path).unwrap(), candidate);
        assert_eq!(std::fs::read(&journal_path).unwrap(), journal);
        unchanged_without_success_push(&mut a, &before_a).await;
        let tokens = [a.token.clone(), b.token.clone(), spectator.token.clone()];
        drop((a, b, spectator, newcomer));
        server.stop().await;
        (room, tokens, selected, intent)
    });
    disable_packages(&cfg);
    runtime().block_on(async {
        let expected_a = saved_state(&cfg, &selected, &tokens[0]);
        let expected_b = saved_state(&cfg, &selected, &tokens[1]);
        let expected_s = saved_state(&cfg, &selected, &tokens[2]);
        let restarted = Server::start(&cfg, 1).await;
        let mut a = Peer::connect(&restarted).await;
        assert_eq!(a.join(room, 0, Some(tokens[0].clone())).await, expected_a);
        let mut b = Peer::connect(&restarted).await;
        assert_eq!(b.join(room, 1, Some(tokens[1].clone())).await, expected_b);
        let mut spectator = Peer::connect(&restarted).await;
        assert_eq!(spectator.spectate(room).await, expected_s);
        let restored = NativeRoomStore::new(&cfg.native_room_store).load_all().unwrap().pop().unwrap();
        let mut expected_snapshot = decode_snapshot(&serde_json::json!(selected.state.snapshot_hex));
        expected_snapshot.auth_generation += 1;
        assert_eq!(decode_snapshot(&serde_json::json!(restored.state.snapshot_hex)), expected_snapshot);
        assert_eq!(serde_json::to_value(&restored.state.frozen_packages).unwrap(), serde_json::to_value(&selected.state.frozen_packages).unwrap());
        let (result, after_a) = a.submit(a.player, 0, RETRY_KEY, intent.clone()).await;
        assert!(matches!(result, CommandResult::Accepted { new_revision: 4, .. }));
        assert_eq!(after_a.view.revision, 4);
        if decision_committed {
            assert_eq!(after_a, expected_a, "committed retries must only replay their receipt");
        } else {
            assert_synchronized(&after_a, &b.pushed(RETRY_KEY).await, &spectator.pushed(RETRY_KEY).await);
        }
        let (again, replayed) = a.submit(a.player, 0, RETRY_KEY, intent.clone()).await;
        assert_eq!(again, result);
        assert_eq!(replayed, after_a);
        submit_error(&mut a, RETRY_KEY, CommandKind::EndPhase).await;
        assert_eq!(a.snapshot().await, after_a);
        let (other_result, _) = b.submit(b.player, 0, RETRY_KEY, intent.clone()).await;
        assert!(matches!(other_result, CommandResult::Rejected { .. }), "another player cannot replay the owner's receipt");
        let (advance, next) = a.submit(a.player, 4, RETRY_KEY + 1, CommandKind::EndPhase).await;
        assert!(matches!(advance, CommandResult::Accepted { new_revision: 5, .. }));
        assert_synchronized(&next, &b.pushed(RETRY_KEY + 1).await, &spectator.pushed(RETRY_KEY + 1).await);
        let (old_result, current) = a.submit(a.player, 0, RETRY_KEY, intent).await;
        assert_eq!(old_result, result, "later commands must not discard the resolved receipt");
        assert_eq!(current, next);
        println!("NATIVE_WS double_failure=pass decision={primary:?} compensation={compensation:?} authority={} restart_revision={} same_key=once continued_revision={}", if decision_committed { "Committed" } else { "Prepared" }, expected_a.view.revision, next.view.revision);
        restarted.stop().await;
    });
}

#[test]
fn native_extensions_ws_double_initial_failure_restores_only_committed_creation() {
    for primary in [
        StoreFault::DecisionWrite,
        StoreFault::DecisionSync,
        StoreFault::DecisionDirectorySync,
    ] {
        for compensation in [
            StoreFault::PrepareWrite,
            StoreFault::PrepareSync,
            StoreFault::PrepareDirectorySync,
        ] {
            let directory = tempfile::tempdir().unwrap();
            let cfg = local_config(directory.path());
            let decision_committed = committed(primary, compensation);
            runtime().block_on(async {
                let server = Server::start(&cfg, 0).await;
                server
                    .lobby
                    .native_room_store_for_test()
                    .unwrap()
                    .fail_next_write_with_compensation(primary, compensation);
                let mut peer = Peer::connect(&server).await;
                peer.send(ClientMessage::CreateRoom {
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
                let restarted = Server::start(&cfg, usize::from(decision_committed)).await;
                let mut peer = Peer::connect(&restarted).await;
                if decision_committed {
                    assert_eq!(peer.join(1, 0, None).await.view.revision, 0);
                } else {
                    peer.send(ClientMessage::JoinRoom { room_id: 1, password: PASSWORD.into(), preferred_seat: Some(0), reconnect_token: None }).await;
                    peer.error(ErrorCode::RoomNotFound).await;
                }
                let mut next = Peer::connect(&restarted).await;
                next.create().await;
                assert_eq!(next.room, 2);
                println!("NATIVE_WS double_initial_failure=pass decision={primary:?} compensation={compensation:?} restored={} next_room=2", usize::from(decision_committed));
                restarted.stop().await;
            });
        }
    }
}
