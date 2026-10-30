//! Trusted store membership controls visibility through the real WebSocket actor.
use super::*;
use crate::native_room_store::{NativeRoomStore, StoredRoom};
use crate::native_view::{choice_for, observer_snapshot_for, RoomRole};
use crate::room_native::NativeRoomSession;

fn stored_pending(cfg: &ServerConfig) -> (u64, [String; 4], State) {
    let mut engine = engine_native::Engine::new_identity(1515, 4);
    engine.assign_general_named(1, "刘备").unwrap();
    for _ in 0..3 {
        let result = engine.submit(engine_native::GameCommand {
            actor: 0,
            expected_revision: engine.revision(),
            dedup_id: engine.revision() + 1,
            action: engine_native::GameAction::NextPhase,
        });
        assert!(!matches!(
            result,
            engine_native::CommandOutcome::Rejected(_)
        ));
    }
    assert_eq!(engine.open_choice().unwrap().observer, 0);
    let mut snapshot = engine.save();
    snapshot.core.mode = engine_native::GameMode::Kingdoms;
    snapshot.core.player_mut(1).revealed_generals = [false, false];
    let removed = snapshot.core.deck.pop().unwrap();
    snapshot.core.player_mut(1).off_court.push(removed);
    let expansion = snapshot.core.deck.pop().unwrap();
    snapshot.core.player_mut(2).lianyaohu_store.push(expansion);
    let mut session = NativeRoomSession::new(engine_native::Engine::restore(&snapshot).unwrap(), 4);
    let owner = session.join(1, Some(0), None).unwrap();
    let other = session.join(2, Some(1), None).unwrap();
    session.spectate(90);
    session.spectate(91);
    let mut state = session.export_state(&session.snapshot_hex());
    state.next_dedup = snapshot.last_dedup + 1;
    let referee = state
        .members
        .iter_mut()
        .find(|m| m.connection_id == 90)
        .unwrap();
    referee.role = ClientRole::Referee;
    referee.reconnect_token = uuid::Uuid::new_v4().to_string();
    let referee_token = referee.reconnect_token.clone();
    let spectator = state
        .members
        .iter_mut()
        .find(|m| m.connection_id == 91)
        .unwrap();
    // A valid seat in a trusted record still cannot turn a Spectator into a Player.
    spectator.player_id = PlayerId::new(1);
    spectator.seat = 0;
    spectator.reconnect_token = uuid::Uuid::new_v4().to_string();
    let spectator_token = spectator.reconnect_token.clone();
    let expected = State {
        view: observer_snapshot_for(session.engine(), RoomRole::Referee, None),
        choice: choice_for(session.engine(), RoomRole::Referee, None),
        legal: vec![],
    };
    let room_id = 909;
    NativeRoomStore::new(&cfg.native_room_store)
        .save(&StoredRoom {
            room_id,
            password_hash: crate::native_room_store::password_hash(room_id, PASSWORD),
            state,
            checksum: String::new(),
        })
        .unwrap();
    (
        room_id,
        [
            owner.member.reconnect_token,
            other.member.reconnect_token,
            referee_token,
            spectator_token,
        ],
        expected,
    )
}

async fn reconnect_spectator_with_stored_seat(peer: &mut Peer, room: u64, token: &str) {
    peer.send(ClientMessage::JoinRoom {
        room_id: room,
        password: PASSWORD.into(),
        preferred_seat: Some(0),
        reconnect_token: Some(token.into()),
    })
    .await;
    let message = peer
        .until(|message| matches!(message, ServerMessage::JoinOk { .. }))
        .await;
    let ServerMessage::JoinOk {
        role,
        player_id,
        seat,
        ..
    } = &message
    else {
        unreachable!()
    };
    assert_eq!((*role, *player_id, *seat), (ClientRole::Spectator, 1, 0));
    State::from_message(message).assert_spectator();
    peer.room = room;
    peer.player = PlayerId::new(1);
    peer.token = token.into();
}

fn assert_referee(state: &State) {
    assert_eq!(state.view.observer, ObserverKind::Referee);
    assert!(
        state.legal.is_empty(),
        "visibility must not grant intervention"
    );
    let ids: std::collections::BTreeSet<_> =
        state.view.known_cards.iter().map(|card| card.id).collect();
    assert_eq!(state.view.known_cards.len(), 124);
    assert_eq!(ids.len(), 124);
    assert!(state
        .view
        .known_cards
        .iter()
        .all(|card| card.def.is_some() && card.suit.is_some() && card.rank.is_some()));
    assert!(state
        .view
        .known_cards
        .iter()
        .any(|card| card.zone == engine_api::CardZoneTag::DrawPile));
    assert!(state
        .view
        .known_cards
        .iter()
        .any(|card| card.zone == engine_api::CardZoneTag::Removed));
    assert!(state
        .view
        .known_cards
        .iter()
        .any(|card| card.zone == engine_api::CardZoneTag::Expansion));
    assert_ne!(
        state.view.seats[1].character_def, 0,
        "dark general remains authorized"
    );
}

#[test]
fn native_ws_referee_restore_reconnect_choice_fanout_and_restart() {
    let directory = tempfile::tempdir().unwrap();
    let cfg = config(directory.path());
    let (room, tokens, expected) = stored_pending(&cfg);
    let before_restart = runtime().block_on(async {
        let server = Server::start(&cfg, 1).await;
        let mut owner = Peer::connect(&server).await;
        let initial = owner.join(room, 0, Some(tokens[0].clone())).await;
        let mut other = Peer::connect(&server).await;
        other
            .join(room, 1, Some(tokens[1].clone()))
            .await
            .assert_player(PlayerId::new(2));
        let mut referee = Peer::connect(&server).await;
        let joined = referee.join(room, 0, Some(tokens[2].clone())).await;
        assert_eq!(joined, expected);
        assert_referee(&joined);
        assert_eq!(referee.snapshot().await, expected);
        assert_eq!(
            joined.choice, initial.choice,
            "referee sees the current owner's full choice"
        );
        let mut old_referee = referee;
        let mut referee = Peer::connect(&server).await;
        assert_eq!(
            referee.join(room, 0, Some(tokens[2].clone())).await,
            expected
        );
        old_referee
            .send(ClientMessage::RequestSnapshot {
                header: MessageHeader::new(room, 1, 0),
                last_revision: 0,
            })
            .await;
        let stale = old_referee
            .until(|message| {
                matches!(
                    message,
                    ServerMessage::Error { .. } | ServerMessage::Snapshot { .. }
                )
            })
            .await;
        assert!(
            matches!(stale, ServerMessage::Error { .. }),
            "old referee connection retained authority"
        );
        let mut spectator = Peer::connect(&server).await;
        reconnect_spectator_with_stored_seat(&mut spectator, room, &tokens[3]).await;
        spectator.snapshot().await.assert_spectator();
        assert!(spectator.snapshot().await.view.seats[1].character_def == 0);
        let mut newcomer = Peer::connect(&server).await;
        newcomer.spectate(room).await.assert_spectator();
        // A rejected player intent still fans out the unchanged pending private choice.
        let (rejected, unchanged) = owner
            .submit(
                owner.player,
                initial.view.revision,
                8999,
                CommandKind::EndPhase,
            )
            .await;
        assert!(matches!(rejected, CommandResult::Rejected { .. }));
        assert_eq!(unchanged, initial);
        assert_eq!(referee.pushed(8999).await, expected);
        spectator.pushed(8999).await.assert_spectator();
        newcomer.pushed(8999).await.assert_spectator();
        old_referee
            .send(ClientMessage::RequestSnapshot {
                header: MessageHeader::new(room, 2, 0),
                last_revision: 0,
            })
            .await;
        let stale_push = old_referee
            .until(|message| {
                matches!(
                    message,
                    ServerMessage::Error { .. } | ServerMessage::CommandOutcome { .. }
                )
            })
            .await;
        assert!(
            matches!(stale_push, ServerMessage::Error { .. }),
            "stale referee received private choice fanout"
        );
        // Seeing the full options cannot authorize a referee to answer them, including replay.
        let answer = durability::answer(&initial, owner.player);
        double_failure::submit_error(&mut referee, 9001, answer.clone()).await;
        double_failure::submit_error(&mut spectator, 9001, answer.clone()).await;
        referee.seq += 1;
        referee
            .send(ClientMessage::SubmitCommand {
                header: MessageHeader::new(room, referee.seq, initial.view.revision),
                command: Command {
                    actor: owner.player,
                    expected_revision: ExpectedRevision(initial.view.revision),
                    idempotency_key: IdempotencyKey(9001),
                    kind: answer.clone(),
                },
                idempotency_key: IdempotencyKey(9001),
                replay: true,
            })
            .await;
        let rejected = referee
            .until(|message| {
                matches!(
                    message,
                    ServerMessage::Error { .. } | ServerMessage::CommandOutcome { .. }
                )
            })
            .await;
        assert!(matches!(rejected, ServerMessage::Error { .. }));
        assert_eq!(referee.snapshot().await, expected);
        let (result, _) = owner
            .submit(owner.player, initial.view.revision, 9002, answer)
            .await;
        assert!(
            matches!(result, CommandResult::Accepted { .. }),
            "{result:?}"
        );
        let pushed = referee.pushed(9002).await;
        assert_referee(&pushed);
        spectator.pushed(9002).await.assert_spectator();
        newcomer.pushed(9002).await.assert_spectator();
        let before_restart = referee.snapshot().await;
        assert_eq!(pushed, before_restart);
        owner.close().await;
        other.close().await;
        referee.close().await;
        old_referee.close().await;
        spectator.close().await;
        newcomer.close().await;
        server.stop().await;
        before_restart
    });
    runtime().block_on(async {
        let restarted = Server::start(&cfg, 1).await;
        let mut referee = Peer::connect(&restarted).await;
        assert_eq!(
            referee.join(room, 0, Some(tokens[2].clone())).await,
            before_restart
        );
        assert_eq!(referee.snapshot().await, before_restart);
        let mut spectator = Peer::connect(&restarted).await;
        reconnect_spectator_with_stored_seat(&mut spectator, room, &tokens[3]).await;
        restarted.stop().await;
    });
}
