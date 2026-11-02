use content_runtime::{content_hash_hex, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageManifest};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use serde_json::{json, Value};

const CATEGORIES: [&str; 3] = ["character_index", "skill_index", "card_index"];
const AUDIO: &[u8] = b"package-local audio payload";

fn seal(package: &mut ContentPackage) {
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
}

fn package() -> ContentPackage {
    let mut package: ContentPackage = serde_json::from_value(json!({
        "manifest": PackageManifest::new("sdk", "voices", "1.0.0"),
        "modules": [],
        "character_index": [{"id": 7, "name_key": "same.name", "max_hp": 4, "skills": [7]}],
        "skill_index": [{"skill_def_id": 7, "name": "same.name", "locked": false}],
        "card_index": [{"card_def_id": 7, "name": "same.name", "kind": "weapon"}],
        "trigger_index": [], "modifier_index": [], "active_index": [], "view_as_index": [],
        "compiler_version": "test"
    }))
    .unwrap();
    let mut manifest = AssetManifest::new("voices");
    for (asset_id, kind, payload) in [
        ("audio", AssetKind::Audio, AUDIO),
        ("image", AssetKind::Image, b"image".as_slice()),
    ] {
        manifest.add_entry(AssetEntry {
            asset_id: asset_id.into(),
            source_hash: *blake3::hash(payload).as_bytes(),
            kind,
            image_variants: vec![],
            audio_variants: vec![],
            priority: PreloadPriority::Lazy,
            depends_on: vec![],
            platforms: vec![],
        });
        package
            .asset_paths
            .insert(asset_id.into(), format!("assets/{asset_id}"));
    }
    package.asset_manifest = Some(manifest);
    seal(&mut package);
    package.validate().unwrap();
    package
}

fn with_voice(package: &ContentPackage, category: &str, voices: Value) -> ContentPackage {
    let mut value = serde_json::to_value(package).unwrap();
    value[category][0]["voice_assets"] = voices;
    serde_json::from_value(value).unwrap()
}

#[test]
fn genuine_definition_voices_roundtrip_and_change_only_asset_hash() {
    let legacy = package();
    for category in CATEGORIES {
        let mut voiced = with_voice(
            &legacy,
            category,
            json!({"die": "audio", "activate": "audio"}),
        );
        assert_ne!(
            voiced.compute_asset_hash(),
            legacy.compute_asset_hash(),
            "{category}"
        );
        assert_eq!(voiced.compute_logic_hash(), legacy.compute_logic_hash());
        assert!(voiced
            .validate()
            .unwrap_err()
            .contains("asset_hash mismatch"));
        seal(&mut voiced);
        let bytes = voiced.to_json_bytes().unwrap();
        let restored = ContentPackage::from_json_bytes(&bytes).unwrap();
        restored.validate().unwrap();
        let value = serde_json::to_value(&restored).unwrap();
        assert_eq!(value[category][0]["voice_assets"]["die"], "audio");
        assert_eq!(
            restored.asset_manifest.as_ref().unwrap().entries[0].kind,
            AssetKind::Audio
        );
        assert!(restored.asset_paths.contains_key("audio"));
        assert_ne!(
            content_hash_hex(&bytes),
            content_hash_hex(&legacy.to_json_bytes().unwrap())
        );
        let empty = with_voice(&legacy, category, json!({}));
        assert_eq!(
            empty.to_json_bytes().unwrap(),
            legacy.to_json_bytes().unwrap()
        );
        assert_eq!(empty.compute_asset_hash(), legacy.compute_asset_hash());
        let mut rule_change = voiced.clone();
        rule_change.character_index[0].max_hp += 1;
        assert_ne!(
            rule_change.compute_logic_hash(),
            voiced.compute_logic_hash()
        );
    }
}

#[test]
fn definition_voices_require_safe_keys_and_local_audio_membership() {
    for category in CATEGORIES {
        for voices in [
            json!({"die": "image"}),
            json!({"die": "missing"}),
            json!({"die": "other/package/audio"}),
            json!({"../die": "audio"}),
            json!({"": "audio"}),
            json!({"die": ""}),
            json!({"die": "../audio"}),
        ] {
            let mut invalid = with_voice(&package(), category, voices.clone());
            seal(&mut invalid);
            let bytes = invalid.to_json_bytes().unwrap();
            assert!(
                ContentPackage::from_json_bytes(&bytes)
                    .unwrap()
                    .validate()
                    .is_err(),
                "{category}: {voices}"
            );
        }
        let mut invalid = with_voice(&package(), category, json!({"die": "audio"}));
        invalid
            .asset_paths
            .insert("audio".into(), "../escape".into());
        seal(&mut invalid);
        assert!(invalid
            .validate()
            .unwrap_err()
            .contains("invalid asset path"));
        let mut invalid = with_voice(&package(), category, json!({"die": "audio"}));
        invalid.asset_manifest = None;
        invalid.asset_paths.clear();
        seal(&mut invalid);
        assert!(invalid.validate().is_err());
    }
}

#[test]
fn voice_hash_uses_category_and_stable_id_and_normalized_map_order() {
    let base = package();
    let mut hashes = Vec::new();
    for category in CATEGORIES {
        let voiced = with_voice(&base, category, json!({"z": "audio", "a": "audio"}));
        let reordered = with_voice(&base, category, json!({"a": "audio", "z": "audio"}));
        assert_eq!(
            voiced.to_json_bytes().unwrap(),
            reordered.to_json_bytes().unwrap()
        );
        assert_eq!(voiced.compute_asset_hash(), reordered.compute_asset_hash());
        hashes.push(voiced.compute_asset_hash());
        let mut value = serde_json::to_value(&voiced).unwrap();
        let id_key = match category {
            "character_index" => "id",
            "skill_index" => "skill_def_id",
            _ => "card_def_id",
        };
        value[category][0][id_key] = json!(8);
        let changed: ContentPackage = serde_json::from_value(value.clone()).unwrap();
        assert_ne!(changed.compute_asset_hash(), voiced.compute_asset_hash());
        if category == "skill_index" {
            // C331 逻辑技能身份要求 skill_index 名称唯一，第二份须改名再比较哈希序无关性。
            value[category][0]["name"] = json!("other.name");
        }
        value[category]
            .as_array_mut()
            .unwrap()
            .push(serde_json::to_value(&voiced).unwrap()[category][0].clone());
        let mut ordered: ContentPackage = serde_json::from_value(value.clone()).unwrap();
        value[category].as_array_mut().unwrap().reverse();
        let mut reversed: ContentPackage = serde_json::from_value(value).unwrap();
        assert_eq!(ordered.compute_asset_hash(), reversed.compute_asset_hash());
        seal(&mut ordered);
        seal(&mut reversed);
        ordered.validate().unwrap();
        reversed.validate().unwrap();
    }
    assert_ne!(hashes[0], hashes[1]);
    assert_ne!(hashes[1], hashes[2]);
    assert_ne!(hashes[0], hashes[2]);
    let mut digest_changed = with_voice(&base, CATEGORIES[0], json!({"die": "audio"}));
    let before = digest_changed.compute_asset_hash();
    digest_changed.asset_manifest.as_mut().unwrap().entries[0].source_hash =
        *blake3::hash(b"changed payload").as_bytes();
    assert_ne!(digest_changed.compute_asset_hash(), before);
}

#[test]
fn sdk_voice_package_installs_and_restores_exact_locked_bytes_and_assets() {
    let mut voiced = package();
    for category in CATEGORIES {
        voiced = with_voice(&voiced, category, json!({"die": "audio"}));
    }
    seal(&mut voiced);
    let bytes = voiced.to_json_bytes().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let assets = dir.path().join("source/assets");
    std::fs::create_dir_all(&assets).unwrap();
    std::fs::write(assets.join("audio"), AUDIO).unwrap();
    std::fs::write(assets.join("image"), b"image").unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let options = InstallOptions {
        allow_unsigned: true,
        ..InstallOptions::default()
    };
    std::fs::write(assets.join("audio"), b"tampered").unwrap();
    assert!(store
        .install_with_assets(&bytes, None, &options, assets.parent().unwrap())
        .unwrap_err()
        .to_string()
        .contains("asset hash mismatch"));
    assert!(store.current_lock().unwrap().packages.is_empty());
    std::fs::write(assets.join("audio"), AUDIO).unwrap();
    let installed = store
        .install_with_assets(&bytes, None, &options, assets.parent().unwrap())
        .unwrap();
    assert_eq!(installed.meta.content_hash, content_hash_hex(&bytes));
    let lock = store.current_lock().unwrap();
    let frozen_lock = serde_json::to_vec(&lock).unwrap();
    let info = store
        .resolve_active_asset_info("sdk", "voices", "audio")
        .unwrap();
    assert_eq!(info.kind, AssetKind::Audio);
    assert_eq!(info.source_hash, *blake3::hash(AUDIO).as_bytes());
    assert_eq!(std::fs::read(&info.path).unwrap(), AUDIO);
    let exported_bytes = std::fs::read(installed.store_path).unwrap();
    assert_eq!(exported_bytes, bytes);
    drop(store);
    std::fs::remove_dir_all(assets.parent().unwrap()).unwrap();
    let restored = PackageStore::open(dir.path().join("store")).unwrap();
    let restored_lock = serde_json::from_slice(&frozen_lock).unwrap();
    let loaded = restored.load_locked_packages(&restored_lock).unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].to_json_bytes().unwrap(), exported_bytes);
    let value = serde_json::to_value(&loaded[0]).unwrap();
    for category in CATEGORIES {
        assert_eq!(value[category][0]["voice_assets"], json!({"die": "audio"}));
    }
    assert_eq!(std::fs::read(info.path).unwrap(), AUDIO);
}

#[test]
fn voice_definition_ids_must_be_nonzero_and_unambiguous() {
    for category in CATEGORIES {
        let voiced = with_voice(&package(), category, json!({"die": "audio"}));
        let id_key = match category {
            "character_index" => "id",
            "skill_index" => "skill_def_id",
            _ => "card_def_id",
        };
        let mut value = serde_json::to_value(&voiced).unwrap();
        value[category][0][id_key] = json!(0);
        let mut invalid: ContentPackage = serde_json::from_value(value).unwrap();
        seal(&mut invalid);
        assert!(invalid
            .validate()
            .unwrap_err()
            .contains("invalid voice definition id"));
        let mut value = serde_json::to_value(&voiced).unwrap();
        let mut duplicate = value[category][0].clone();
        duplicate.as_object_mut().unwrap().remove("voice_assets");
        value[category].as_array_mut().unwrap().push(duplicate);
        let mut invalid: ContentPackage = serde_json::from_value(value).unwrap();
        seal(&mut invalid);
        let error = invalid.validate().unwrap_err();
        if category == "skill_index" {
            // C331 的逻辑技能身份检查先于声明语音检查：重复 skill_def_id 已在该层被拒。
            assert!(error.contains("conflicting logical skill identity"), "{error}");
        } else {
            assert!(error.contains("ambiguous"), "{error}");
        }
        for malformed in [Value::Null, json!([]), json!({"die": 1})] {
            let mut value = serde_json::to_value(&voiced).unwrap();
            value[category][0]["voice_assets"] = malformed;
            assert!(ContentPackage::from_json_bytes(&serde_json::to_vec(&value).unwrap()).is_err());
        }
    }
}

#[test]
fn absent_and_empty_voices_keep_legacy_zero_asset_hash() {
    let mut legacy = package();
    legacy.asset_manifest = None;
    legacy.asset_paths.clear();
    seal(&mut legacy);
    assert_eq!(legacy.compute_asset_hash(), [0; 32]);
    legacy.validate().unwrap();
    for category in CATEGORIES {
        let empty = with_voice(&legacy, category, json!({}));
        assert_eq!(
            empty.to_json_bytes().unwrap(),
            legacy.to_json_bytes().unwrap()
        );
        assert_eq!(empty.compute_asset_hash(), [0; 32]);
        let mut missing_audio = with_voice(&legacy, category, json!({"die": "audio"}));
        assert_ne!(missing_audio.compute_asset_hash(), [0; 32]);
        seal(&mut missing_audio);
        assert!(missing_audio.validate().is_err());
    }
}

#[test]
fn definition_voice_hash_cannot_be_absorbed_by_legacy_sort_key() {
    let mut legacy = package();
    legacy.character_index[0].sort_key =
        "xdefinition-voice-assets/v1[[\"character\",7,{\"die\":\"audio\"}]]".into();
    let mut voiced = package();
    voiced.character_index[0].sort_key = "x".into();
    voiced.character_index[0]
        .voice_assets
        .insert("die".into(), "audio".into());
    seal(&mut legacy);
    seal(&mut voiced);
    let legacy_bytes = legacy.to_json_bytes().unwrap();
    let voiced_bytes = voiced.to_json_bytes().unwrap();
    ContentPackage::from_json_bytes(&legacy_bytes)
        .unwrap()
        .validate()
        .unwrap();
    ContentPackage::from_json_bytes(&voiced_bytes)
        .unwrap()
        .validate()
        .unwrap();
    assert_eq!(legacy.compute_logic_hash(), voiced.compute_logic_hash());
    assert_ne!(
        content_hash_hex(&legacy_bytes),
        content_hash_hex(&voiced_bytes)
    );
    assert_ne!(legacy.compute_asset_hash(), voiced.compute_asset_hash());
}
