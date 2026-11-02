use content_schema::ContentPackage;
use engine_native_wasm::{engine_from_packs, validate_pack_json, NativeSession};
use serde_json::{json, Value};
fn package() -> ContentPackage {
    let mut p=content_compiler::compile_source_named("repair-schema5","skill repair_draw(99108020) { trigger damage owner_is_target { effect draw_cards(self,1); } }").unwrap();
    p.skill_index[0].charlotte = Some(false);
    p.manifest.content_schema_version = 5;
    p.manifest.logic_hash = p.compute_logic_hash();
    p
}
fn assets() -> Value {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/huanle-upstream-noname");
    json!({"manifest":serde_json::from_slice::<Value>(&std::fs::read(root.join("package.npack.json")).unwrap()).unwrap()["manifest"],"cards":serde_json::from_slice::<Value>(&std::fs::read(root.join("assets/cards.json")).unwrap()).unwrap()})
}
#[test]
fn omitted_modules_declarations_reject_at_real_validate_admit_restore() {
    let good = assets();
    validate_pack_json(&good.to_string()).unwrap();
    let mut handle = NativeSession::new_with_packs_json(3, 10820, &format!("[{good}]")).unwrap();
    let saved = handle.save_hex();
    let view = handle.view_json(0).unwrap();
    for (version, value) in [(5, json!(true)), (3, json!(false)), (4, Value::Null)] {
        let mut bad = good.clone();
        bad["manifest"]["content_schema_version"] = version.into();
        bad["skill_index"] =
            json!([{"skill_def_id":99108020,"name":"repair_draw","charlotte":value}]);
        let raw = bad.to_string();
        assert!(
            validate_pack_json(&raw).is_err(),
            "omitted modules schema{version}"
        );
        assert!(engine_from_packs(3, 10820, &format!("[{raw}]")).is_err());
        assert!(handle
            .restore_hex_with_packs_json(&saved, &format!("[{raw}]"))
            .is_err());
        assert_eq!(saved, handle.save_hex());
        assert_eq!(view, handle.view_json(0).unwrap());
    }
}
#[test]
fn duplicate_raw_declaration_rejects_single_array_and_preserves_handle() {
    let p = package();
    let raw = String::from_utf8(p.to_json_bytes().unwrap()).unwrap();
    validate_pack_json(&raw).unwrap();
    let mut handle = NativeSession::new_with_packs_json(3, 10821, &format!("[{raw}]")).unwrap();
    let saved = handle.save_hex();
    let view = handle.view_json(0).unwrap();
    let duplicate = raw.replace(
        "\"charlotte\":false",
        "\"charlotte\":true,\"charlotte\":false",
    );
    assert_ne!(raw, duplicate);
    let err = validate_pack_json(&duplicate).expect_err("raw duplicate must reject");
    assert!(
        err.contains("duplicate field") && err.contains("charlotte"),
        "{err}"
    );
    assert!(engine_from_packs(3, 10821, &format!("[{duplicate}]")).is_err());
    assert!(handle
        .restore_hex_with_packs_json(&saved, &format!("[{duplicate}]"))
        .is_err());
    assert_eq!(saved, handle.save_hex());
    assert_eq!(view, handle.view_json(0).unwrap());
}
#[test]
fn export_genuine_schema5_fixture_for_actual_sdk_wasm_path() {
    if let Ok(directory) = std::env::var("C331_REPAIR_EVIDENCE") {
        let directory = std::path::Path::new(&directory);
        std::fs::write(
            directory.join("genuine-schema5.json"),
            package().to_json_bytes().unwrap(),
        )
        .unwrap();
        std::fs::write(
            directory.join("genuine-asset-only.json"),
            assets().to_string(),
        )
        .unwrap();
        let mut active = content_compiler::compile_source_named(
            "repair-active",
            "skill repair_active(99108021) { active play_phase { effect draw_cards(self,1); } }",
        )
        .unwrap();
        active.skill_index[0].charlotte = Some(false);
        active.active_index[0].host_key = Some("host:sanjian_weapon".into());
        active.manifest.content_schema_version = 5;
        active.manifest.logic_hash = active.compute_logic_hash();
        std::fs::write(
            directory.join("active-conflict-recomputed.json"),
            active.to_json_bytes().unwrap(),
        )
        .unwrap();
    }
}
