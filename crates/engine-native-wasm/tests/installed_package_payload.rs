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
