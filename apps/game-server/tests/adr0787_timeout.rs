//! ADR0787: actual commands, durable control and the public actor timer.
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
use engine_api::{CommandKind, CommandResult};
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};
use room_native::NativeRoomSession;

fn pending() -> NativeRoomSession {
    let mut session = NativeRoomSession::new(Engine::new(4242, 4), 4);
    for seat in 0..4 {
        session
            .join(10 + u64::from(seat), Some(seat), None)
            .unwrap();
    }
    for _ in 0..1000 {
        if session.engine().open_choice().is_some() {
            return session;
        }
        let seat = session.engine().turn().current;
        let action = session
            .engine()
            .legal_actions(seat)
            .into_iter()
            .next()
            .unwrap();
        let result = session
            .submit_native(seat, action, session.engine().revision())
            .unwrap();
        assert!(matches!(result, CommandResult::Accepted { .. }));
    }
    panic!("fixture must open a real machine choice");
}
fn bytes(session: &NativeRoomSession) -> Vec<u8> {
    let mut saved = session.engine().save();
    saved.auth_generation = 0;
    engine_native::archive::encode_archive(&saved)
}

#[test]
fn timeout_uses_answer_validation_and_recorded_command_replays_full_snapshot() {
    let mut session = pending();
    let choice = session.engine().open_choice().unwrap().clone();
    let before = bytes(&session);
    let deadline = session.timeout_state().deadline.clone().unwrap();
    assert!(session
        .automatic_command(deadline.deadline_unix_ms - 1, true)
        .is_none());
    let (seat, action, takeover) = session
        .automatic_command(deadline.deadline_unix_ms, true)
        .unwrap();
    assert!(takeover);
    assert!(matches!(action, GameAction::TimeoutAction { .. }));
    for (actor, action) in [
        ((seat + 1) % 4, action.clone()),
        (
            seat,
            GameAction::TimeoutAction {
                choice_id: choice.id + 1,
                selection: vec![],
            },
        ),
        (
            seat,
            GameAction::TimeoutAction {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(u32::MAX)],
            },
        ),
    ] {
        assert!(matches!(
            session.submit_automatic(actor, action, true).unwrap(),
            CommandResult::Rejected { .. }
        ));
        assert_eq!(bytes(&session), before);
        assert!(session.timeout_state().managed_seats.is_empty());
    }
    let mut answer = session.engine().clone();
    let GameAction::TimeoutAction {
        choice_id,
        selection,
    } = action.clone()
    else {
        unreachable!()
    };
    let expected = answer.revision();
    let dedup = answer.save().last_dedup + 1;
    assert!(!matches!(
        answer.submit(GameCommand {
            actor: seat,
            expected_revision: expected,
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id,
                selection
            }
        }),
        engine_native::CommandOutcome::Rejected(_)
    ));
    assert!(matches!(
        session.submit_automatic(seat, action, takeover).unwrap(),
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        bytes(&session),
        engine_native::archive::encode_archive(&answer.save())
    );
    assert_eq!(session.engine().log(), answer.log());
    assert!(session.timeout_state().managed_seats.contains(&seat));
    let state = session.export_state(&session.snapshot_hex());
    let json = serde_json::to_value(&state).unwrap();
    assert!(json["wal"]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["action"].get("TimeoutAction").is_some()));
    let restored =
        NativeRoomSession::from_saved(&serde_json::from_value(json.clone()).unwrap(), vec![])
            .unwrap();
    assert_eq!(bytes(&session), bytes(&restored));
    for key in ["actor", "expected_revision", "dedup_id", "revision"] {
        let mut corrupt = json.clone();
        let entries = corrupt["wal"]["entries"].as_array_mut().unwrap();
        entries.last_mut().unwrap()[key] = serde_json::json!(u64::MAX);
        assert!(
            serde_json::from_value::<room_native::NativeRoomState>(corrupt)
                .map_or(true, |state| NativeRoomSession::from_saved(&state, vec![])
                    .is_err()),
            "{key}"
        );
    }
}

#[test]
fn durable_deadline_configuration_reconnect_illegal_and_receipt_rules() {
    let mut session = pending();
    let deadline = session.timeout_state().deadline.clone().unwrap();
    for secs in [10, 15, 30, 60, 90] {
        session.set_choice_timeout_secs(secs).unwrap();
    }
    for secs in [0, 9, 11, 91, 255] {
        assert!(session.set_choice_timeout_secs(secs).is_err());
    }
    assert_eq!(session.timeout_state().deadline.as_ref(), Some(&deadline));
    let (seat, action, takeover) = session
        .automatic_command(deadline.deadline_unix_ms, true)
        .unwrap();
    let conn = 10 + u64::from(seat);
    let token = session
        .members()
        .find(|m| m.seat == seat)
        .unwrap()
        .reconnect_token
        .clone();
    session.submit_automatic(seat, action, takeover).unwrap();
    assert!(session.timeout_state().managed_seats.contains(&seat));
    let snapshot = session.snapshot_hex();
    let state = session.export_state(&snapshot);
    let mut restored = NativeRoomSession::from_saved(&state, vec![]).unwrap();
    assert_eq!(
        serde_json::to_value(restored.timeout_state()).unwrap(),
        serde_json::to_value(session.timeout_state()).unwrap()
    );
    let current_deadline = restored.timeout_state().deadline.clone();
    assert!(restored.join(100, None, Some("invalid".into())).is_err());
    assert!(restored.timeout_state().managed_seats.contains(&seat));
    assert!(restored
        .submit(
            conn,
            &CommandKind::TimeoutAction { choice_id: None },
            restored.engine().revision()
        )
        .is_err());
    assert!(restored.timeout_state().managed_seats.contains(&seat));
    restored.join(100, Some(seat), Some(token)).unwrap();
    assert!(!restored.timeout_state().managed_seats.contains(&seat));
    assert_eq!(restored.timeout_state().deadline, current_deadline);
    assert!(restored
        .submit(conn, &CommandKind::EndPhase, restored.engine().revision())
        .is_err());
    // Optional aggregates preserve the exact legacy checksum shape, but cannot restore new rules.
    let mut missing = serde_json::to_value(&state).unwrap();
    missing.as_object_mut().unwrap().remove("timeout");
    missing.as_object_mut().unwrap().remove("wal");
    let old: room_native::NativeRoomState = serde_json::from_value(missing.clone()).unwrap();
    assert_eq!(serde_json::to_value(&old).unwrap(), missing);
    assert!(NativeRoomSession::from_saved(&old, vec![]).is_err());
}

#[tokio::test]
async fn real_actor_ten_seconds_commits_timeout_to_disk() {
    let mut session = pending();
    session.set_choice_timeout_secs(10).unwrap();
    // Fixture already opened a default window: set configuration before creating the actor window.
    let mut state = session.export_state(&session.snapshot_hex());
    let deadline = state.timeout.as_mut().unwrap().deadline.as_mut().unwrap();
    deadline.deadline_unix_ms = room_native::unix_ms().unwrap() + 10_000;
    let seat = deadline.observer;
    let choice_id = deadline.choice_id;
    let session = NativeRoomSession::from_saved(&state, vec![]).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let store = native_room_store::NativeRoomStore::new(dir.path());
    let handle = room_native_actor::spawn_room_native_persisted(
        787,
        session,
        16,
        1000,
        String::new(),
        Some(store.clone()),
    );
    let (tx, mut rx) = tokio::sync::mpsc::channel(128);
    let (reply, response) = tokio::sync::oneshot::channel();
    let token = state
        .members
        .iter()
        .find(|m| m.seat == seat)
        .unwrap()
        .reconnect_token
        .clone();
    handle
        .send(room::RoomMsg::Join {
            connection_id: 999,
            display_name: "timeout".into(),
            preferred_seat: Some(seat),
            reconnect_token: Some(token),
            password: String::new(),
            outbound: session::Outbound::new(tx),
            reply,
        })
        .await
        .unwrap();
    let joined = response.await.unwrap().unwrap();
    let started = std::time::Instant::now();
    let choice = joined.choice.unwrap();
    assert_eq!(u64::from(choice.choice_id.raw()), choice_id);
    assert_eq!(
        choice.deadline_unix_ms,
        state
            .timeout
            .as_ref()
            .unwrap()
            .deadline
            .as_ref()
            .map(|d| d.deadline_unix_ms)
    );
    tokio::time::timeout(std::time::Duration::from_secs(12), async {
        while let Some(text) = rx.recv().await {
            if let engine_protocol::ServerMessage::CommandOutcome { .. } =
                engine_protocol::decode_s2c(&text).unwrap()
            {
                break;
            }
        }
    })
    .await
    .expect("10 second deadline must run actual actor");
    assert!(started.elapsed() >= std::time::Duration::from_secs(9));
    let saved = store.load_all().unwrap().pop().unwrap();
    let wal = serde_json::to_value(&saved.state.wal).unwrap();
    assert_eq!(
        wal["entries"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["action"].get("TimeoutAction").is_some())
            .count(),
        1
    );
    NativeRoomSession::from_saved(&saved.state, vec![]).unwrap();
}

#[test]
fn real_329_archive_rejects_without_rewriting_its_original_room_checksum() {
    let raw = include_bytes!("fixtures/adr0787-room329.json");
    let record: native_room_store::StoredRoom = serde_json::from_slice(raw).unwrap();
    assert_eq!(
        native_room_store::state_checksum(&record.state),
        record.checksum
    );
    let error = match NativeRoomSession::from_saved(&record.state, vec![]) {
        Ok(_) => panic!("329 must reject"),
        Err(e) => e,
    };
    // 存档格式已由 105 升到 106：这份真实 329 档在格式门就先被拒
    // （UnsupportedFormatVersion(105)），早于规则版本比较。两者都表示
    // 「旧档必须被拒且不得改写房间校验和」，本测试的意图不变。
    assert!(
        error.contains("RuleVersionMismatch") || error.contains("UnsupportedFormatVersion(105)"),
        "{error}"
    );
    assert!(record.state.timeout.is_none());
    assert!(record.state.wal.is_none());
}

#[test]
fn timeout_and_human_races_commit_once_and_replayed_receipt_does_not_release_control() {
    let mut session = pending();
    let deadline = session.timeout_state().deadline.clone().unwrap();
    let (seat, timeout, _) = session
        .automatic_command(deadline.deadline_unix_ms, true)
        .unwrap();
    let GameAction::TimeoutAction {
        choice_id,
        selection,
    } = timeout.clone()
    else {
        unreachable!()
    };
    let conn = 10 + u64::from(seat);
    let before = session.export_state(&session.snapshot_hex());
    let command = CommandKind::ResolveChoice {
        choice_id: engine_api::ChoiceId::new(choice_id as u32),
        selections: (0..selection.len() as u32).collect(),
        cancelled: selection.is_empty(),
    };
    let result = session
        .submit(conn, &command, session.engine().revision())
        .unwrap();
    assert!(matches!(result, CommandResult::Accepted { .. }));
    session
        .remember_command(conn, 787, &command, &result)
        .unwrap();
    assert!(matches!(
        session
            .submit_automatic(seat, timeout.clone(), true)
            .unwrap(),
        CommandResult::Rejected { .. }
    ));
    assert!(session.timeout_state().managed_seats.is_empty());
    let mut managed = session.export_state(&session.snapshot_hex());
    managed.timeout.as_mut().unwrap().managed_seats = vec![seat];
    let mut managed = NativeRoomSession::from_saved(&managed, vec![]).unwrap();
    assert_eq!(
        managed.command_receipt(conn, 787, &command).unwrap(),
        Some(result)
    );
    assert!(managed.timeout_state().managed_seats.contains(&seat));
    assert!(managed
        .submit(
            conn,
            &CommandKind::TimeoutAction { choice_id: None },
            managed.engine().revision()
        )
        .is_err());
    assert!(managed.timeout_state().managed_seats.contains(&seat));
    let mut timeout_first = NativeRoomSession::from_saved(&before, vec![]).unwrap();
    timeout_first.submit_automatic(seat, timeout, true).unwrap();
    let committed = timeout_first.snapshot_hex();
    assert!(timeout_first
        .submit(
            conn,
            &command,
            before.wal.unwrap().entries.last().unwrap().revision
        )
        .map_or(true, |r| matches!(r, CommandResult::Rejected { .. })));
    assert_eq!(timeout_first.snapshot_hex(), committed);
    assert!(timeout_first.timeout_state().managed_seats.contains(&seat));
    let resumed = timeout_first
        .submit(
            conn,
            &CommandKind::StartGame {
                player_count: 4,
                rng_seed: 4242,
                ruleset: String::new(),
            },
            timeout_first.engine().revision(),
        )
        .unwrap();
    assert!(matches!(resumed, CommandResult::Accepted { .. }));
    assert!(!timeout_first.timeout_state().managed_seats.contains(&seat));
}

#[test]
fn recorded_timeout_selection_matches_current_wasm_answer_full_archive() {
    let session = pending();
    let before = session.engine().save();
    let choice = session.engine().open_choice().unwrap();
    let (_, action, _) = session.automatic_command(session.timeout_state().deadline.as_ref().unwrap().deadline_unix_ms, true).unwrap();
    let GameAction::TimeoutAction { selection, .. } = &action else { unreachable!() };
    let selection = selection.clone();
    let tokens: Vec<u32> = selection.iter().map(|value| choice.spec.options.iter().position(|option| option == value).unwrap() as u32).collect();
    let mut native = Engine::restore(&before).unwrap();
    let mut answer = Engine::restore(&before).unwrap();
    // Existing WASM adapter owns this command counter; use the same counter for exact archive parity.
    let dedup = native.revision().wrapping_mul(2).max(1);
    let revision = native.revision();
    let result = native.submit(GameCommand { actor: choice.observer, expected_revision: revision, dedup_id: dedup, action });
    assert!(!matches!(result, engine_native::CommandOutcome::Rejected(_)));
    answer.submit(GameCommand { actor: choice.observer, expected_revision: revision, dedup_id: dedup, action: GameAction::Answer { choice_id: choice.id, selection: selection.clone() } });
    assert_eq!(native.save(), answer.save());
    assert_eq!(native.log(), answer.log());
    if let Some(path) = std::env::var_os("A330_WASM_PARITY_OUT") {
        let hex = |bytes: Vec<u8>| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({
            "before_hex": hex(engine_native::archive::encode_archive(&before)),
            "after_hex": hex(engine_native::archive::encode_archive(&native.save())),
            "hash": hex(native.state_hash().to_vec()), "codes": native.public_event_codes_from(0),
            "actor": choice.observer, "choice_id": choice.id, "tokens": tokens
        })).unwrap()).unwrap();
    }
}

#[test]
fn all_five_configurations_create_a_deadline_once_and_preserve_that_window() {
    let pending = pending();
    for secs in [10, 15, 30, 60, 90] {
        let mut session = NativeRoomSession::new(Engine::restore(&pending.engine().save()).unwrap(), 4);
        assert_eq!(session.timeout_state().choice_timeout_secs, 30);
        session.set_choice_timeout_secs(secs).unwrap();
        let start = room_native::unix_ms().unwrap();
        session.sync_deadline().unwrap();
        let deadline = session.timeout_state().deadline.clone().unwrap();
        assert!(deadline.deadline_unix_ms >= start + u64::from(secs) * 1000);
        assert!(deadline.deadline_unix_ms <= room_native::unix_ms().unwrap() + u64::from(secs) * 1000);
        session.sync_deadline().unwrap();
        assert_eq!(session.timeout_state().deadline.as_ref(), Some(&deadline));
        let restored = NativeRoomSession::from_saved(&session.export_state(&session.snapshot_hex()), vec![]).unwrap();
        assert_eq!(restored.timeout_state().deadline.as_ref(), Some(&deadline));
    }
}

#[test]
fn compensatory_reveal_keeps_same_choice_deadline_and_timeout_uses_latest_revision() {
    let mut engine = Engine::new_identity(2, 4);
    engine.assign_general_named(1, "刘备").unwrap();
    engine.debug_set_turn(0, engine_native::Phase::Play, 1);
    let sha = engine.debug_place(0, "sha");
    engine.debug_place(1, "shan");
    let mut snapshot = engine.save();
    snapshot.core.mode = engine_native::GameMode::Kingdoms;
    snapshot.core.player_mut(1).revealed_generals[0] = false;
    let mut session = NativeRoomSession::new(Engine::restore(&snapshot).unwrap(), 4);
    assert!(matches!(session.submit_native(0, GameAction::PlayCard { card: sha, targets: vec![1] }, 0).unwrap(), CommandResult::Accepted { .. }));
    let deadline = session.timeout_state().deadline.clone().unwrap();
    let revision = session.engine().revision();
    assert!(matches!(session.submit_native(1, GameAction::RevealGeneral { slot: 0 }, revision).unwrap(), CommandResult::Accepted { .. }));
    assert!(session.engine().revision() > revision);
    assert_eq!(session.timeout_state().deadline.as_ref(), Some(&deadline));
    let (seat, action, takeover) = session.automatic_command(deadline.deadline_unix_ms, true).unwrap();
    assert!(matches!(session.submit_automatic(seat, action, takeover).unwrap(), CommandResult::Accepted { .. }));
}

#[tokio::test]
async fn continuous_managed_steps_yield_to_queued_human_snapshot() {
    let session = NativeRoomSession::new(Engine::new(4242, 4), 4);
    let mut state = session.export_state(&session.snapshot_hex());
    state.timeout.as_mut().unwrap().managed_seats = vec![0, 1, 2, 3];
    let session = NativeRoomSession::from_saved(&state, vec![]).unwrap();
    let handle = room_native_actor::spawn_room_native(789, session, 16, 1000);
    let (tx, _rx) = tokio::sync::mpsc::channel(128);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle.send(room::RoomMsg::Join { connection_id: 1, display_name: "human".into(), preferred_seat: Some(0), reconnect_token: None, password: String::new(), outbound: session::Outbound::new(tx), reply }).await.unwrap();
    response.await.unwrap().unwrap();
    let (reply, response) = tokio::sync::oneshot::channel();
    handle.send(room::RoomMsg::Snapshot { connection_id: 1, header: engine_protocol::MessageHeader::new(789, 1, 0), reply }).await.unwrap();
    let result = tokio::time::timeout(std::time::Duration::from_secs(1), response).await.unwrap().unwrap().unwrap();
    let engine_protocol::ServerMessage::Snapshot { view, .. } = result else { unreachable!() };
    assert!(view.revision <= 4, "automatic processing must yield every command: {}", view.revision);
}

async fn control_snapshot(handle: &room::RoomHandle) -> engine_protocol::ServerMessage {
    let (reply, response) = tokio::sync::oneshot::channel();
    handle.send(room::RoomMsg::Snapshot { connection_id: 999,
        header: engine_protocol::MessageHeader::new(handle.id, 1, 0), reply }).await.unwrap();
    response.await.unwrap().unwrap()
}

async fn control_recover(handle: &room::RoomHandle) -> Result<(), String> {
    let (reply, response) = tokio::sync::oneshot::channel();
    handle.send(room::RoomMsg::Recover { reply }).await.unwrap();
    response.await.unwrap()
}

#[tokio::test]
async fn recover_after_automatic_write_failure_retries_original_expired_choice_once() {
    automatic_failure_then_recover(false).await;
}

#[tokio::test]
async fn recover_cannot_release_automatic_pause_when_restart_is_required() {
    automatic_failure_then_recover(true).await;
}

async fn automatic_failure_then_recover(restart_required: bool) {
    use native_room_store::StoreFault;
    let pending = pending();
    let mut initial = pending.export_state(&pending.snapshot_hex());
    // A short absolute test deadline drives the production actor timer, not a synthetic submit.
    let deadline = initial.timeout.as_mut().unwrap().deadline.as_mut().unwrap();
    deadline.deadline_unix_ms = room_native::unix_ms().unwrap() + 500;
    let deadline = deadline.clone();
    let token = initial.members.iter().find(|m| m.seat == deadline.observer).unwrap().reconnect_token.clone();
    let session = NativeRoomSession::from_saved(&initial, vec![]).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let store = native_room_store::NativeRoomStore::new(dir.path());
    let handle = room_native_actor::spawn_room_native_persisted(790, session, 16, 1000, String::new(), Some(store.clone()));
    let (tx, mut rx) = tokio::sync::mpsc::channel(128);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle.send(room::RoomMsg::Join { connection_id: 999, display_name: "recover".into(), preferred_seat: Some(deadline.observer), reconnect_token: Some(token), password: String::new(), outbound: session::Outbound::new(tx), reply }).await.unwrap();
    let joined = response.await.unwrap().unwrap();
    let confirmed = std::fs::read(dir.path().join("790.json")).unwrap();
    if restart_required { store.fail_next_write_with_compensation(StoreFault::DecisionWrite, StoreFault::PrepareSync); }
    else { store.fail_next_write(StoreFault::FileSync); }
    tokio::time::sleep(std::time::Duration::from_millis(deadline.deadline_unix_ms.saturating_sub(room_native::unix_ms().unwrap()))).await;
    let failed = control_snapshot(&handle).await;
    let engine_protocol::ServerMessage::Snapshot { view, choice, .. } = failed else { unreachable!() };
    assert_eq!(view, joined.view);
    assert_eq!(choice.as_ref().unwrap().deadline_unix_ms, Some(deadline.deadline_unix_ms));
    if restart_required {
        let bytes = std::fs::read(dir.path().join("790.json")).unwrap();
        assert!(control_recover(&handle).await.unwrap_err().contains("restart required"));
        assert_eq!(std::fs::read(dir.path().join("790.json")).unwrap(), bytes);
        assert_eq!(control_snapshot(&handle).await, engine_protocol::ServerMessage::Snapshot {
            header: engine_protocol::MessageHeader::new(790, 1, view.revision), view, choice,
            legal_actions: joined.legal_actions,
        });
        return;
    }
    assert_eq!(std::fs::read(dir.path().join("790.json")).unwrap(), confirmed);
    // A failed Recover must not clear the pause or retry an expired timer.
    store.fail_next_write(StoreFault::Rename);
    assert!(control_recover(&handle).await.is_err());
    let engine_protocol::ServerMessage::Snapshot { view: retry_view, .. } = control_snapshot(&handle).await else { unreachable!() };
    assert_eq!(retry_view, joined.view);
    let saved = store.load_all().unwrap().pop().unwrap();
    assert_eq!(saved.state.timeout.as_ref().unwrap().deadline.as_ref(), Some(&deadline));
    control_recover(&handle).await.unwrap();
    let update = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while let Some(text) = rx.recv().await {
            let message = engine_protocol::decode_s2c(&text).unwrap();
            if matches!(message, engine_protocol::ServerMessage::CommandOutcome { .. }) { return message; }
        }
        panic!("actor closed before retry");
    }).await.expect("successful durable Recover must release the original expired timer");
    let saved = store.load_all().unwrap().pop().unwrap();
    let entries = saved.state.wal.as_ref().unwrap().entries.iter().filter(|entry| matches!(entry.command().action, GameAction::TimeoutAction { .. })).collect::<Vec<_>>();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].expected_revision, joined.view.revision);
    assert!(matches!(entries[0].command().action, GameAction::TimeoutAction { choice_id, .. } if choice_id == deadline.choice_id));
    if let Some(output) = std::env::var_os("A330_RECOVER_PROOF") {
        std::fs::write(output, serde_json::to_vec_pretty(&serde_json::json!({"original_deadline": deadline, "before": initial, "after": saved, "pushed": update})).unwrap()).unwrap();
    }
    NativeRoomSession::from_saved(&saved.state, vec![]).unwrap();
}
