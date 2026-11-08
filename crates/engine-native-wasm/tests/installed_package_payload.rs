//! The production wrapper uses installed manifests and assets, without reading source at runtime.
use engine_native_wasm::test_support::{
    assign_general_for, engine_from_archive, engine_from_packs, engine_identity_from_packs,
    general_candidates_json_for, hex_encode,
};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

fn asset(dir: &str, relative: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(dir)
        .join(relative);
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn installed(dir: &str, key: &str, path: &str) -> Value {
    let mut value = json!({"manifest": asset(dir, "package.npack.json")["manifest"]});
    value[key] = asset(dir, path);
    value
}

fn payload() -> Value {
    json!([
        installed("huanle-upstream-noname", "cards", "assets/cards.json"),
        installed("d3-upstream-noname", "characters", "assets/character.json"),
        installed("cardpile-upstream-noname", "pile", "assets/pile.json")
    ])
}

#[test]
fn installed_assets_enter_catalog_select_and_same_lock_restore() {
    let payload = payload().to_string();
    let baseline = engine_from_packs(4, 42, "[]").unwrap();
    let mut engine = engine_from_packs(4, 42, &payload).unwrap();
    let names: Vec<_> = engine.catalog().cards.iter().map(|c| c.name).collect();
    assert_eq!(names.iter().filter(|n| n.starts_with("hnl_")).count(), 7);
    assert!(
        engine.catalog().cards.len() > baseline.catalog().cards.len() + 7,
        "pile enters deck"
    );
    assert!(general_candidates_json_for(&engine).contains("ddd刘冲"));
    assign_general_for(&mut engine, 0, "ddd刘冲").unwrap();
    assert_eq!(
        engine.view_for(0).players[0].skills,
        vec!["dddjinggou", "dddmoyan"]
    );
    let hex = hex_encode(&engine_native::archive::encode_archive(&engine.save()));
    let restored = engine_from_archive(&hex, &payload).unwrap();
    assert_eq!(restored.state_hash(), engine.state_hash());
    assert_eq!(restored.catalog().cards.len(), engine.catalog().cards.len());
    assert!(engine_from_archive(&hex, "[]").is_err());
    for (index, pointer) in [
        (0, "/cards/cards/0/rank"),
        (1, "/characters/generals/0/hp"),
        (2, "/pile/entries/0/counts/diamond"),
    ] {
        let mut changed: Value = serde_json::from_str(&payload).unwrap();
        *changed[index].pointer_mut(pointer).unwrap() = json!(10);
        assert!(
            engine_from_archive(&hex, &changed.to_string()).is_err(),
            "same manifest with changed asset must refuse restore: {pointer}"
        );
    }

    let mut wrong: Value = serde_json::from_str(&payload).unwrap();
    wrong[0]["manifest"]["version"] = json!("1.0.1");
    assert!(
        engine_from_archive(&hex, &wrong.to_string()).is_err(),
        "exact manifest version binds archive"
    );
}

#[test]
fn identity_preserves_roles_lord_bonus_and_extension_init_hooks() {
    let packs = json!([
        installed("d3-upstream-noname", "characters", "assets/character.json"),
        installed("wuxing-upstream-liuli", "cards", "assets/cards.json")
    ])
    .to_string();
    let mut engine = engine_identity_from_packs(4, 42, &packs).unwrap();
    let standard = engine_native::Engine::try_new_identity(42, 4).unwrap();
    assert_eq!(engine.state_core().mode, standard.state_core().mode);
    assert_eq!(engine.view_for(0).players[0].max_hp, 5);
    assert_eq!(engine.view_for(0).players[0].hp, 5);
    assert!(engine
        .state_core()
        .players
        .iter()
        .all(|p| p.wunature.is_some()));
    assert!(general_candidates_json_for(&engine).contains("ddd刘冲"));
    let globals: Vec<_> = engine
        .state_core()
        .skills
        .keys()
        .filter(|(seat, _)| *seat == 0)
        .copied()
        .collect();
    assert!(!globals.is_empty(), "wuxing grants global skills");
    assign_general_for(&mut engine, 0, "ddd刘冲").unwrap();
    assert_eq!(engine.view_for(0).players[0].max_hp, 5);
    assert!(globals
        .iter()
        .all(|key| engine.state_core().skills.contains_key(key)));
    // Reselecting must remove the old general's skills without clearing extension-global skills.
    assign_general_for(&mut engine, 0, "曹操").unwrap();
    for old in ["dddjinggou", "dddmoyan"] {
        let def = engine.catalog().skill_by_name(old).unwrap();
        assert!(!engine.state_core().skills.contains_key(&(0, def)));
    }
    assert_eq!(engine.view_for(0).players[0].max_hp, 5);
    assert!(globals
        .iter()
        .all(|key| engine.state_core().skills.contains_key(key)));
    let hex = hex_encode(&engine_native::archive::encode_archive(&engine.save()));
    let restored = engine_from_archive(&hex, &packs).unwrap();
    assert_eq!(restored.state_hash(), engine.state_hash());
    assert!(engine_identity_from_packs(3, 42, &packs).is_err());
    assert!(engine_identity_from_packs(9, 42, &packs).is_err());
    let bare = engine_identity_from_packs(4, 42, "[]").unwrap();
    assert_eq!(
        bare.state_hash(),
        standard.state_hash(),
        "empty packs preserve identity constructor"
    );
}

#[test]
fn malformed_installed_payloads_are_not_silently_skipped() {
    let good = payload()[1].clone();
    for field in ["namespace", "package_id", "version"] {
        for bad in [Value::Null, json!(""), json!(42)] {
            let mut item = good.clone();
            item["manifest"][field] = bad;
            let error = engine_from_packs(4, 42, &json!([item]).to_string())
                .err()
                .unwrap();
            assert!(error.contains(field), "{error}");
        }
    }
    for item in [
        json!({"manifest": good["manifest"]}),
        json!({"manifest": good["manifest"], "characters": null}),
        json!({"manifest": good["manifest"], "pile": {"source":"test","total":1,"entries":[{"card":"missing","counts":{"spade":1}}]}}),
    ] {
        assert!(engine_from_packs(4, 42, &json!([item]).to_string()).is_err());
    }
    assert!(engine_from_packs(4, 42, &json!([good.clone(), good]).to_string()).is_err());
}

#[test]
fn duplicate_installed_slots_are_rejected_on_start_and_restore() {
    let good = installed("d3-upstream-noname", "characters", "assets/character.json");
    let single = json!([good.clone()]).to_string();
    let engine = engine_from_packs(4, 42, &single).unwrap();
    let saved = hex_encode(&engine_native::archive::encode_archive(&engine.save()));

    let mut new_version = good.clone();
    new_version["manifest"]["version"] = json!("2.0.0");
    new_version["characters"]["generals"] = json!([]);
    let mut new_bytes = good.clone();
    new_bytes["characters"]["generals"] = json!([]);
    for second in [new_version, new_bytes, good.clone()] {
        let duplicate = json!([good.clone(), second]).to_string();
        for (entry, error) in [
            ("start", engine_from_packs(4, 42, &duplicate).err()),
            (
                "identity",
                engine_identity_from_packs(4, 42, &duplicate).err(),
            ),
            ("restore", engine_from_archive(&saved, &duplicate).err()),
        ] {
            let error =
                error.unwrap_or_else(|| panic!("{entry} accepted a duplicate package slot"));
            assert!(error.contains("重复包槽位"), "{error}");
        }
    }
}

#[test]
fn same_package_id_in_different_namespaces_remains_allowed() {
    let first = installed("d3-upstream-noname", "characters", "assets/character.json");
    let mut second = installed("huanle-upstream-noname", "cards", "assets/cards.json");
    second["manifest"]["namespace"] = json!("another-namespace");
    second["manifest"]["package_id"] = first["manifest"]["package_id"].clone();
    let packs = json!([first, second]).to_string();
    let engine = engine_from_packs(4, 42, &packs).expect("different namespaces are distinct slots");
    assert!(general_candidates_json_for(&engine).contains("ddd刘冲"));
    assert!(engine
        .catalog()
        .cards
        .iter()
        .any(|card| card.name.starts_with("hnl_")));
}

#[test]
fn invalid_installed_identity_fields_are_rejected_before_start_and_restore() {
    let good = installed("d3-upstream-noname", "characters", "assets/character.json");
    let valid_packs = json!([good.clone()]).to_string();
    let valid = engine_from_packs(4, 42, &valid_packs).unwrap();
    let valid_save = hex_encode(&engine_native::archive::encode_archive(&valid.save()));
    let mut accepted = Vec::new();
    for field in ["namespace", "package_id", "version"] {
        for bad in [".", "..", "name/child", "name\\child", "   "] {
            let mut item = good.clone();
            item["manifest"][field] = json!(bad);
            let packs = json!([item]).to_string();
            if !engine_from_archive(&valid_save, &packs)
                .err()
                .is_some_and(|error| error.contains(field))
            {
                accepted.push(format!("restore did not validate {field}={bad:?}"));
            }
            for (entry, result) in [
                ("start", engine_from_packs(4, 42, &packs)),
                ("identity", engine_identity_from_packs(4, 42, &packs)),
            ] {
                match result {
                    Ok(engine) => {
                        accepted.push(format!("{entry} accepted {field}={bad:?}"));
                        let saved =
                            hex_encode(&engine_native::archive::encode_archive(&engine.save()));
                        if engine_from_archive(&saved, &packs).is_ok() {
                            accepted.push(format!("{entry}+restore {field}={bad:?}"));
                        }
                    }
                    Err(error) => assert!(error.contains(field), "{error}"),
                }
            }
        }
    }
    assert!(
        accepted.is_empty(),
        "accepted invalid identities: {accepted:?}"
    );
}

#[test]
fn installed_dependency_lock_is_checked_on_start_and_restore() {
    let base = installed("huanle-upstream-noname", "cards", "assets/cards.json");
    let mut dependent = installed("d3-upstream-noname", "characters", "assets/character.json");
    let required = json!({
        "namespace": base["manifest"]["namespace"],
        "package_id": base["manifest"]["package_id"],
        "version": base["manifest"]["version"],
        "logic_hash": base["manifest"]["logic_hash"]
    });
    dependent["manifest"]["dependencies"] = json!([required]);
    let good = json!([dependent.clone(), base.clone()]).to_string();
    let valid = engine_from_packs(4, 42, &good).unwrap();
    let saved = hex_encode(&engine_native::archive::encode_archive(&valid.save()));
    assert_eq!(
        engine_from_archive(&saved, &good).unwrap().state_hash(),
        valid.state_hash()
    );
    assert!(engine_identity_from_packs(4, 42, &good).is_ok());
    for hash in [Value::Null, json!(vec![0; 32])] {
        let mut wildcard = dependent.clone();
        if hash.is_null() {
            wildcard["manifest"]["dependencies"][0]
                .as_object_mut()
                .unwrap()
                .remove("logic_hash");
        } else {
            wildcard["manifest"]["dependencies"][0]["logic_hash"] = hash;
        }
        assert!(engine_from_packs(4, 42, &json!([wildcard, base]).to_string()).is_ok());
    }

    let mut cycle_base = base.clone();
    cycle_base["manifest"]["dependencies"] = json!([{
        "namespace": dependent["manifest"]["namespace"],
        "package_id": dependent["manifest"]["package_id"],
        "version": dependent["manifest"]["version"]
    }]);
    assert!(engine_from_packs(4, 42, &json!([dependent.clone(), cycle_base]).to_string()).is_ok());

    let mut cases = Vec::new();
    cases.push(("missing", json!([dependent.clone()])));
    for (label, field, value) in [
        ("namespace", "namespace", json!("different")),
        ("version", "version", json!("wrong")),
        ("hash", "logic_hash", json!(vec![17; 32])),
        ("bad hash", "logic_hash", json!([1, 2])),
        ("bad id", "package_id", json!("../bad")),
    ] {
        let mut bad = dependent.clone();
        bad["manifest"]["dependencies"][0][field] = value;
        cases.push((label, json!([bad, base.clone()])));
    }
    let mut duplicate = dependent.clone();
    duplicate["manifest"]["dependencies"] = json!([required, required]);
    cases.push(("duplicate", json!([duplicate, base.clone()])));
    let mut self_dep = dependent.clone();
    self_dep["manifest"]["dependencies"][0]["namespace"] =
        dependent["manifest"]["namespace"].clone();
    self_dep["manifest"]["dependencies"][0]["package_id"] =
        dependent["manifest"]["package_id"].clone();
    cases.push(("self", json!([self_dep, base.clone()])));
    let mut invalid = dependent.clone();
    invalid["manifest"]["dependencies"] = Value::Null;
    cases.push(("not an array", json!([invalid, base])));

    let mut accepted = Vec::new();
    let mut old_saves = Vec::new();
    for (label, values) in cases {
        let packs = values.to_string();
        for (entry, result) in [
            ("start", engine_from_packs(4, 42, &packs)),
            ("identity", engine_identity_from_packs(4, 42, &packs)),
        ] {
            match result {
                Ok(engine) => {
                    accepted.push(format!("{label}/{entry}"));
                    let old_save =
                        hex_encode(&engine_native::archive::encode_archive(&engine.save()));
                    if entry == "start" {
                        old_saves.push(json!({"case": label, "packs": packs, "hex": old_save}));
                    }
                    if engine_from_archive(&old_save, &packs).is_ok() {
                        accepted.push(format!("{label}/{entry}/old-save-restore"));
                    }
                }
                Err(error) => assert!(error.contains("依赖"), "{label}: {error}"),
            }
        }
        let error = engine_from_archive(&saved, &packs).err().unwrap();
        if !error.contains("依赖") {
            accepted.push(format!("{label}/restore-not-validated: {error}"));
        }
    }
    if let Some(path) = std::env::var_os("NC02_DEPENDENCY_OLD_SAVES_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&old_saves).unwrap()).unwrap();
    }
    if let Some(path) = std::env::var_os("NC02_DEPENDENCY_OLD_SAVES_INPUT") {
        let saves: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(saves.len(), 9);
        for save in saves {
            let error = engine_from_archive(
                save["hex"].as_str().unwrap(),
                save["packs"].as_str().unwrap(),
            )
            .err()
            .unwrap();
            assert!(error.contains("依赖"), "{}: {error}", save["case"]);
        }
    }
    assert!(
        accepted.is_empty(),
        "invalid dependency locks accepted: {accepted:?}"
    );
}

#[test]
fn all_37_sources_46_packages_validate_start_step_save_and_restore_on_wasm_session() {
    use engine_native_wasm::{
        prepare_locked_content_json, validate_frozen_package_json, validate_pack_json,
        NativeSession,
    };
    use std::collections::{BTreeMap, BTreeSet};

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let map_path = manifest_dir
        .join("../../docs/operations/evidence/2026-10-19-package-source-map/map.tsv");
    let map_text = fs::read_to_string(&map_path).expect("read map.tsv");
    let ext_root = manifest_dir.join("../../packages/extensions");

    let mut by_source: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut all_dirs: BTreeSet<String> = BTreeSet::new();
    for (idx, line) in map_text.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let src_id = cols[0].trim().to_string();
        let pkgs: Vec<String> = cols[3]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        for p in &pkgs {
            all_dirs.insert(p.clone());
        }
        by_source.insert(src_id, pkgs);
    }
    assert_eq!(by_source.len(), 37, "must cover all 37 upstream sources");
    assert_eq!(all_dirs.len(), 46, "must cover all 46 extension packages");

    let build_pkg_payloads = |dir: &str| -> (Value, Value) {
        let pkg_path = ext_root.join(dir);
        let npack_raw = fs::read_to_string(pkg_path.join("package.npack.json")).unwrap();
        let npack_val: Value = serde_json::from_str(&npack_raw).unwrap();
        let mut pack_item = json!({ "manifest": npack_val["manifest"] });
        let mut raw_assets: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        if let Some(paths) = npack_val.get("asset_paths").and_then(Value::as_object) {
            for rel in paths.values().filter_map(Value::as_str) {
                let bytes = fs::read(pkg_path.join(rel)).unwrap();
                raw_assets.insert(rel.to_string(), bytes);
            }
        }
        for (rel, key) in [
            ("assets/cards.json", "cards"),
            ("assets/character.json", "characters"),
            ("assets/pile.json", "pile"),
            ("assets/config.json", "config"),
        ] {
            let p = pkg_path.join(rel);
            if p.is_file() {
                let bytes = fs::read(&p).unwrap();
                let parsed: Value = serde_json::from_slice(&bytes).unwrap();
                pack_item[key] = parsed;
            }
        }
        let item_str = pack_item.to_string();
        validate_pack_json(&item_str)
            .unwrap_or_else(|e| panic!("{dir}: validate_pack_json failed: {e}"));
        let assets_str = serde_json::to_string(&raw_assets).unwrap();
        let frozen_str = validate_frozen_package_json(&npack_raw, &assets_str)
            .unwrap_or_else(|e| panic!("{dir}: validate_frozen_package_json failed: {e}"));
        let frozen_val: Value = serde_json::from_str(&frozen_str).unwrap();
        (pack_item, frozen_val)
    };

    let empty_locked = prepare_locked_content_json("[]").expect("empty locked content");

    for (src_id, dirs) in &by_source {
        let mut pack_items = Vec::new();
        let mut frozen_items = Vec::new();
        for dir in dirs {
            let (pack_item, frozen_val) = build_pkg_payloads(dir);
            pack_items.push(pack_item);
            frozen_items.push(frozen_val);
        }
        let packs_json = serde_json::to_string(&pack_items).unwrap();
        let frozen_json = serde_json::to_string(&frozen_items).unwrap();
        let locked_json = prepare_locked_content_json(&frozen_json)
            .unwrap_or_else(|e| panic!("{src_id}: prepare_locked_content_json failed: {e}"));

        // 1. packsJson session path: new -> auto_step -> save -> restore -> reject empty
        let mut sess_packs =
            NativeSession::new_with_packs_json(4, 42007, &packs_json)
                .unwrap_or_else(|e| panic!("{src_id}: new_with_packs_json failed: {e}"));
        let rev0 = sess_packs.revision();
        let step_out: Value =
            serde_json::from_str(&sess_packs.auto_step().expect("auto_step")).expect("auto_step json");
        assert!(
            matches!(step_out["kind"].as_str(), Some("plan" | "answer")),
            "{src_id}: auto_step must step, got {step_out}"
        );
        assert!(sess_packs.revision() > rev0, "{src_id}: revision must advance");
        let hash_after = sess_packs.state_hash_hex();
        let saved_hex = sess_packs.save_hex();
        let mut restored_packs = NativeSession::new(4, 1).expect("dummy session");
        restored_packs
            .restore_hex_with_packs_json(&saved_hex, &packs_json)
            .unwrap_or_else(|e| panic!("{src_id}: restore_hex_with_packs_json failed: {e}"));
        assert_eq!(
            restored_packs.state_hash_hex(),
            hash_after,
            "{src_id}: packsJson state_hash_hex roundtrip mismatch"
        );
        assert!(
            restored_packs
                .restore_hex_with_packs_json(&saved_hex, "[]")
                .is_err(),
            "{src_id}: empty packsJson must reject archive restore"
        );

        // 2. lockedContentJson session path: new -> auto_step -> save_locked -> restore_locked -> reject empty
        let mut sess_locked =
            NativeSession::new_with_locked_content_json(4, 42007, &locked_json)
                .unwrap_or_else(|e| panic!("{src_id}: new_with_locked_content_json failed: {e}"));
        let step_locked: Value =
            serde_json::from_str(&sess_locked.auto_step().expect("auto_step locked"))
                .expect("auto_step locked json");
        assert!(
            matches!(step_locked["kind"].as_str(), Some("plan" | "answer")),
            "{src_id}: locked auto_step must step"
        );
        let locked_hash_after = sess_locked.state_hash_hex();
        let saved_locked_obj: Value = serde_json::from_str(
            &sess_locked
                .save_locked_json()
                .unwrap_or_else(|e| panic!("{src_id}: save_locked_json failed: {e}")),
        )
        .expect("save_locked_json parse");
        assert_eq!(saved_locked_obj["kind"], "saveLocked");
        let saved_locked_hex = saved_locked_obj["hex"].as_str().unwrap();
        let mut restored_locked = NativeSession::new(4, 1).expect("dummy session");
        restored_locked
            .restore_hex_with_locked_content_json(&saved_locked_hex, &locked_json)
            .unwrap_or_else(|e| panic!("{src_id}: restore_hex_with_locked_content_json failed: {e}"));
        assert_eq!(
            restored_locked.state_hash_hex(),
            locked_hash_after,
            "{src_id}: lockedContentJson state_hash_hex roundtrip mismatch"
        );
        assert!(
            restored_locked
                .restore_hex_with_locked_content_json(&saved_locked_hex, &empty_locked)
                .is_err(),
            "{src_id}: empty lockedContentJson must reject archive restore"
        );
    }
}

