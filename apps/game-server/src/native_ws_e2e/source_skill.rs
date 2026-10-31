//! Approved27 metadata over the production WS6 handshake, actor and reconnect path.
use super::*;
use crate::native_room_store::{NativeRoomStore, StoredRoom};
use crate::room_native::NativeRoomSession;
#[path = "../../tests/support/ws6_choice_before_source.rs"]
mod old;

async fn raw_snapshot(peer: &mut Peer, label: &str) -> serde_json::Value {
    peer.seq += 1;
    let seq = peer.seq;
    peer.send(ClientMessage::RequestSnapshot {
        header: MessageHeader::new(peer.room, seq, 0),
        last_revision: 0,
    })
    .await;
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(10), peer.ws.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        if let Message::Text(text) = frame {
            let value: serde_json::Value = serde_json::from_str(&text).unwrap();
            if value["type"] == "Snapshot" && value["payload"]["header"]["conn_seq"] == seq {
                assert_eq!(value["payload"]["header"]["protocol_version"], 6);
                if let Ok(directory) = std::env::var("SOURCE27_EVIDENCE") {
                    std::fs::create_dir_all(&directory).unwrap();
                    std::fs::write(
                        Path::new(&directory).join(format!("actual-ws-{label}.json")),
                        text.as_bytes(),
                    )
                    .unwrap();
                }
                return value["payload"].clone();
            }
        }
    }
}
#[test]
fn source_actual_ws_roles_reconnect_deadline_and_wire_compatibility() {
    let directory = tempfile::tempdir().unwrap();
    let cfg = local_config(directory.path());
    let mut engine = engine_native::Engine::new(2306, 3);
    engine.debug_set_turn(0, engine_native::Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_grant(1, "遗计");
    engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let mut session = NativeRoomSession::new(engine, 3);
    let owner = session.join(1, Some(1), None).unwrap();
    let other = session.join(2, Some(0), None).unwrap();
    session.spectate(90);
    session.sync_deadline().unwrap();
    let mut state = session.export_state(&session.snapshot_hex());
    let referee = state
        .members
        .iter_mut()
        .find(|member| member.connection_id == 90)
        .unwrap();
    referee.role = ClientRole::Referee;
    referee.reconnect_token = uuid::Uuid::new_v4().to_string();
    let token = referee.reconnect_token.clone();
    NativeRoomStore::new(&cfg.native_room_store)
        .save(&StoredRoom {
            room_id: 927,
            password_hash: crate::native_room_store::password_hash(927, PASSWORD),
            state,
            checksum: String::new(),
        })
        .unwrap();
    runtime().block_on(async {
        let server = Server::start(&cfg, 1).await;
        let mut a = Peer::connect(&server).await;
        let initial = a
            .join(927, 1, Some(owner.member.reconnect_token.clone()))
            .await;
        assert_eq!(
            initial.choice.as_ref().unwrap().source_skill.as_deref(),
            Some("遗计")
        );
        let mut b = Peer::connect(&server).await;
        b.join(927, 0, Some(other.member.reconnect_token)).await;
        let mut r = Peer::connect(&server).await;
        r.join(927, 0, Some(token)).await;
        let mut s = Peer::connect(&server).await;
        s.spectate(927).await;
        let own = raw_snapshot(&mut a, "owner").await;
        assert_eq!(own["choice"]["source_skill"], "遗计");
        assert_eq!(own["choice"]["prompt"], "trigger_confirm");
        assert!(own["choice"]["deadline_unix_ms"].as_u64().unwrap() > 0);
        assert_eq!(
            raw_snapshot(&mut r, "referee").await["choice"],
            own["choice"]
        );
        for (peer, label) in [(&mut b, "other"), (&mut s, "spectator")] {
            assert!(raw_snapshot(peer, label).await["choice"].is_null());
        }
        let mut reconnect = Peer::connect(&server).await;
        let joined = reconnect
            .join(927, 1, Some(owner.member.reconnect_token))
            .await;
        assert_eq!(joined.choice, initial.choice);
        assert_eq!(
            raw_snapshot(&mut reconnect, "reconnect").await["choice"],
            own["choice"]
        );
        // Freeze the exact WS6 DTO before this one field, including its deadline.
        let old: old::ChoiceRequest = serde_json::from_value(own["choice"].clone()).unwrap();
        let mut expected_old = own["choice"].clone();
        expected_old.as_object_mut().unwrap().remove("source_skill");
        assert_eq!(serde_json::to_value(old).unwrap(), expected_old);
        let mut wire = own["choice"].clone();
        wire.as_object_mut().unwrap().remove("source_skill");
        assert_eq!(
            serde_json::from_value::<ChoiceRequest>(wire.clone())
                .unwrap()
                .source_skill,
            None
        );
        wire["source_skill"] = serde_json::Value::Null;
        assert_eq!(
            serde_json::from_value::<ChoiceRequest>(wire.clone())
                .unwrap()
                .source_skill,
            None
        );
        for bad in [
            serde_json::json!(""),
            serde_json::json!(1),
            serde_json::json!({}),
            serde_json::json!([]),
            serde_json::json!(true),
        ] {
            wire["source_skill"] = bad;
            assert!(serde_json::from_value::<ChoiceRequest>(wire.clone()).is_err());
        }
        let (rejected, unchanged) = reconnect
            .submit(
                reconnect.player,
                initial.view.revision,
                9271,
                CommandKind::EndPhase,
            )
            .await;
        assert!(matches!(rejected, CommandResult::Rejected { .. }));
        assert_eq!(unchanged.choice, initial.choice);
        assert_eq!(r.pushed(9271).await.choice, initial.choice);
        assert!(b.pushed(9271).await.choice.is_none());
        assert!(s.pushed(9271).await.choice.is_none());
        reconnect.close().await;
        a.close().await;
        b.close().await;
        r.close().await;
        s.close().await;
        server.stop().await;
    });
}
