use desktop_client::NativeEngineHost;
use engine_native_wasm::{NativeEngine, NativeSession, MAX_ARCHIVE_HEX_CHARS};
use serde_json::{json, Value};

async fn request(host: &NativeEngineHost, mut body: Value) -> Value {
    body["id"] = json!(1);
    let response = host.request(body).await.unwrap();
    assert_eq!(response["ok"], true, "{response}");
    response["data"].clone()
}

#[tokio::test]
async fn host_requests_initialize_and_reject_invalid_input_without_losing_state() {
    let host = NativeEngineHost::new().unwrap();
    let init = request(&host, json!({"type": "init"})).await;
    assert_eq!(init, "native ready");
    request(
        &host,
        json!({"type": "start", "players": 4, "seed": 42, "observer": 0}),
    )
    .await;
    let saved = request(&host, json!({"type": "save"})).await;
    for mut invalid in [
        json!({"type": "playCard", "actor": 0, "card": -1, "targets": []}),
        json!({"type": "playCard", "actor": 256, "card": 1, "targets": []}),
        json!({"type": "playCard", "actor": 0, "card": 1.5, "targets": []}),
        json!({"type": "playCard", "actor": 0, "card": 1, "targets": [256]}),
        json!({"type": "activateSkill", "actor": 0, "skill": 65536, "targets": [], "cardTargets": []}),
        json!({"type": "activateSkill", "actor": 0, "skill": 1, "targets": [1], "cardTargets": []}),
        json!({"type": "resolveChoice", "actor": 0, "choiceId": 9007199254740992u64, "selections": [], "cancelled": false}),
        json!({"type": "start", "players": 4, "seed": 42, "observer": 255}),
        json!({"type": "start", "players": 0, "seed": 42, "observer": 0}),
        json!({"type": "start", "players": 4, "seed": 9007199254740992u64, "observer": 0}),
        json!({"type": "start", "players": 4, "seed": 42, "observer": 0, "packsJson": "{}"}),
        json!({"type": "restore", "hex": "中"}),
        json!({"type": "restore", "hex": "abc"}),
        json!({"type": "restore", "hex": "zz"}),
        json!({"type": "validatePackage", "packJson": "{}"}),
        json!({"type": "chooseGeneral", "seat": 0, "name": "unknown-general"}),
        json!({"type": "useSkillCard"}),
        json!({"type": "loadReplay"}),
        json!({"type": "seekReplay"}),
        json!({"type": "unknown-request"}),
    ] {
        invalid["id"] = json!(2);
        let rejected = host.request(invalid).await.unwrap();
        assert_eq!(rejected["ok"], false, "{rejected}");
        assert_eq!(saved, request(&host, json!({"type": "save"})).await);
    }
    for field in ["hex", "packsJson", "packJson"] {
        let mut oversized = json!({"id": 3, "type": "restore"});
        oversized[field] = json!("0".repeat(MAX_ARCHIVE_HEX_CHARS + 1));
        assert!(host
            .request(oversized)
            .await
            .unwrap_err()
            .contains("exceeds"));
        assert_eq!(saved, request(&host, json!({"type": "save"})).await);
    }
}

#[tokio::test]
async fn host_full_identity_trajectory_matches_session_and_existing_binding_and_cross_restores() {
    let host = NativeEngineHost::new().unwrap();
    request(&host, json!({"type": "init"})).await;
    let mut native = NativeSession::new_identity(4, 42).unwrap();
    let mut binding = NativeEngine::new_identity(4, 42).unwrap();
    let start = request(
        &host,
        json!({"type": "start", "players": 4, "seed": 42, "observer": 0, "mode": "identity"}),
    )
    .await;
    assert_eq!(start["hash_hex"], native.state_hash_hex());
    let mut finished = false;
    for step in 0..2000 {
        let actual = request(&host, json!({"type": "autoStep"})).await;
        let expected: Value = serde_json::from_str(&native.auto_step().unwrap()).unwrap();
        assert_eq!(actual["result"], expected, "step {step}");
        assert_eq!(
            expected,
            serde_json::from_str::<Value>(&binding.auto_step().unwrap()).unwrap()
        );
        assert_eq!(actual["hash_hex"], native.state_hash_hex(), "step {step}");
        assert_eq!(native.state_hash_hex(), binding.state_hash_hex());
        if step == 30 {
            let save = request(&host, json!({"type": "save"})).await;
            native.restore_hex(save["hex"].as_str().unwrap()).unwrap();
            binding
                .restore_hex_checked(save["hex"].as_str().unwrap())
                .unwrap();
            let restored =
                request(&host, json!({"type": "restore", "hex": binding.save_hex()})).await;
            assert_eq!(actual["hash_hex"], restored["hash_hex"]);
        }
        if native.is_finished() {
            finished = true;
            break;
        }
    }
    assert!(finished, "seed 42 must reach its actual terminal outcome");
    assert!(!request(&host, json!({"type": "observer"})).await["view"]["outcome"].is_null());
}

#[tokio::test]
async fn host_projects_authority_candidates_and_observer_privacy() {
    let host = NativeEngineHost::new().unwrap();
    request(&host, json!({"type": "init"})).await;
    let data = request(
        &host,
        json!({"type": "start", "players": 4, "seed": 42, "observer": 1, "mode": "identity"}),
    )
    .await;
    let reference = NativeSession::new_identity(4, 42).unwrap();
    let raw: Value = serde_json::from_str(&reference.view_json(1).unwrap()).unwrap();
    let known = data["view"]["known_cards"].as_array().unwrap();
    assert_eq!(
        known.len(),
        raw["players"][1]["hand"]["own"].as_array().unwrap().len()
    );
    assert!(known
        .iter()
        .all(|card| card["owner"] == 1 && card["def"].is_number()));
    for seat in 0..4 {
        assert_eq!(
            data["view"]["seats"][seat]["identity"],
            raw["players"][seat]["identity"]
        );
    }
    let candidates = request(&host, json!({"type": "generalCandidates"})).await;
    assert_eq!(
        candidates,
        serde_json::from_str::<Value>(&reference.general_candidates_json()).unwrap()
    );
    let name = candidates["candidates"][0]["name"].as_str().unwrap();
    let selected = request(
        &host,
        json!({"type": "chooseGeneral", "seat": 1, "name": name}),
    )
    .await;
    assert_eq!(selected["assignment"]["name"], name);
    assert_eq!(
        selected["assignment"]["state_hash"],
        selected["envelope"]["hash_hex"]
    );
    assert_eq!(
        request(&host, json!({"type": "status"})).await["revision"],
        selected["envelope"]["revision"]
    );
    // Invalid restored observer must leave the original four-seat match intact.
    let before = request(&host, json!({"type": "save"})).await;
    let one_seat = NativeSession::new(1, 0).unwrap().save_hex();
    let rejected = host
        .request(json!({"id": 2, "type": "restore", "hex": one_seat}))
        .await
        .unwrap();
    assert_eq!(rejected["ok"], false);
    assert_eq!(before, request(&host, json!({"type": "save"})).await);
}

#[tokio::test]
async fn host_owns_a_single_ordered_command_stream_and_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<NativeEngineHost>();
    let host = NativeEngineHost::new().unwrap();
    request(&host, json!({"type": "init"})).await;
    request(
        &host,
        json!({"type": "start", "players": 4, "seed": 42, "observer": 0}),
    )
    .await;
    let (first, second) = tokio::join!(
        host.request(json!({"id": 2, "type": "autoStep"})),
        host.request(json!({"id": 3, "type": "autoStep"})),
    );
    let first = first.unwrap();
    let second = second.unwrap();
    assert_eq!(first["id"], 2);
    assert_eq!(second["id"], 3);
    assert_eq!(
        first["data"]["revision"].as_u64().unwrap() + 1,
        second["data"]["revision"].as_u64().unwrap()
    );
}

#[tokio::test]
async fn independent_clients_do_not_reset_each_others_sessions_and_close_is_controlled() {
    let battle = NativeEngineHost::new().unwrap();
    let validator = NativeEngineHost::new().unwrap();
    request(&battle, json!({"type": "init"})).await;
    request(
        &battle,
        json!({"type": "start", "players": 4, "seed": 42, "observer": 0}),
    )
    .await;
    request(&battle, json!({"type": "autoStep"})).await;
    let saved = request(&battle, json!({"type": "save"})).await;
    request(&validator, json!({"type": "init"})).await;
    assert_eq!(saved, request(&battle, json!({"type": "save"})).await);
    validator.close();
    validator.close();
    assert!(validator
        .request(json!({"id": 1, "type": "save"}))
        .await
        .unwrap_err()
        .contains("closed"));
    assert_eq!(saved, request(&battle, json!({"type": "save"})).await);
}

#[cfg(feature = "tauri-shell")]
#[tokio::test]
async fn tauri_resource_ids_isolate_clients_and_have_a_real_capacity_and_close_boundary() {
    use desktop_client::{close_native_engine, open_native_engine};
    let mut table = tauri::ResourceTable::default();
    let mut ids = Vec::new();
    for _ in 0..8 {
        ids.push(open_native_engine(&mut table).unwrap());
    }
    assert!(open_native_engine(&mut table)
        .unwrap_err()
        .contains("limit"));
    let first = table.get::<NativeEngineHost>(ids[0]).unwrap();
    let second = table.get::<NativeEngineHost>(ids[1]).unwrap();
    request(&first, json!({"type": "init"})).await;
    request(&first, json!({"type": "autoStep"})).await;
    let saved = request(&first, json!({"type": "save"})).await;
    request(&second, json!({"type": "init"})).await;
    assert_eq!(saved, request(&first, json!({"type": "save"})).await);
    close_native_engine(&mut table, ids[0]).unwrap();
    assert!(close_native_engine(&mut table, ids[0]).is_err());
    assert!(table.get::<NativeEngineHost>(ids[0]).is_err());
    assert!(first
        .request(json!({"id": 1, "type": "save"}))
        .await
        .unwrap_err()
        .contains("closed"));
    for id in ids.into_iter().skip(1) {
        close_native_engine(&mut table, id).unwrap();
    }
    let replacement = open_native_engine(&mut table).unwrap();
    close_native_engine(&mut table, replacement).unwrap();
}

#[cfg(feature = "tauri-shell")]
#[tokio::test]
async fn document_cleanup_retires_only_native_hosts_and_preserves_unrelated_resources() {
    use desktop_client::{close_native_engines, open_native_engine};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    struct UnrelatedResource(Arc<AtomicBool>);
    impl tauri::Resource for UnrelatedResource {
        fn name(&self) -> std::borrow::Cow<'_, str> {
            // Matching a name is insufficient: only NativeEngineHost's concrete type may be closed.
            std::any::type_name::<NativeEngineHost>().into()
        }
        fn close(self: Arc<Self>) {
            self.0.store(true, Ordering::Release);
        }
    }
    let mut table = tauri::ResourceTable::default();
    let other_closed = Arc::new(AtomicBool::new(false));
    let unrelated_id = table.add(UnrelatedResource(other_closed.clone()));
    for _document in 0..20 {
        let first = open_native_engine(&mut table).unwrap();
        let second = open_native_engine(&mut table).unwrap();
        let retained_old_handle = table.get::<NativeEngineHost>(first).unwrap();
        request(&retained_old_handle, json!({"type": "init"})).await;
        assert_eq!(close_native_engines(&mut table), 2);
        assert_eq!(close_native_engines(&mut table), 0);
        assert!(table.get::<NativeEngineHost>(first).is_err());
        assert!(table.get::<NativeEngineHost>(second).is_err());
        assert!(retained_old_handle
            .request(json!({"id": 2, "type": "status"}))
            .await
            .unwrap_err()
            .contains("closed"));
        assert!(table.get::<UnrelatedResource>(unrelated_id).is_ok());
        assert!(!other_closed.load(Ordering::Acquire));
    }
    table.close(unrelated_id).unwrap();
    assert!(other_closed.load(Ordering::Acquire));
}

#[tokio::test]
async fn source_actual_native_choice_reaches_desktop_host_only_for_owner() {
    let mut engine = engine_native::Engine::new(2306, 3);
    engine.debug_set_turn(0, engine_native::Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_grant(1, "遗计");
    engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let hex =
        engine_native_wasm::hex_encode(&engine_native::archive::encode_archive(&engine.save()));
    let host = NativeEngineHost::new().unwrap();
    request(&host, json!({"type":"init"})).await;
    for observer in [1, 0, 2] {
        request(
            &host,
            json!({"type":"start","players":3,"seed":2306,"observer":observer}),
        )
        .await;
        let restored = request(&host, json!({"type":"restore","hex":hex})).await;
        if observer == 1 {
            assert_eq!(restored["choice"]["source_skill"], "遗计");
            assert_eq!(restored["choice"]["prompt"], "trigger_confirm");
        } else {
            assert!(restored["choice"].is_null());
        }
        let saved = request(&host, json!({"type":"save"})).await;
        let projected = request(&host, json!({"type":"observer"})).await;
        assert_eq!(projected["choice"], restored["choice"]);
        assert_eq!(request(&host, json!({"type":"save"})).await, saved);
    }
}
