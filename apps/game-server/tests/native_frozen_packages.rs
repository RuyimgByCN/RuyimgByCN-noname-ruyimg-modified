//! 原局完整包副本：原始指纹、VM 与非核心资产，及停用/升级/撤销边界。

#[path = "../src/native.rs"]
mod native;

use native::{FrozenNativePackage, NativeContent};
use std::path::Path;

const DSL: &str =
    "skill frozen_vm(990201) { trigger phase_end owner_is_target { effect recover_hp(self, 1); } }";

fn write_package(root: &Path, version: &str, source: &str) -> Vec<u8> {
    let package = content_compiler::compile_source_named("frozen-vm", source).unwrap();
    let characters = serde_json::to_vec(&serde_json::json!({
        "source": "test", "upstream_path": "frozen-vm",
        "upstream_commit": "0000000000000000000000000000000000000000",
        "pack": { "name": "frozen-vm", "version": 1, "description": "test" },
        "generals": [{
            "id": "frozen_probe", "name": "frozen_probe", "faction": "qun", "hp": 4,
            "gender": "male", "skills": [{ "name": package.trigger_index[0].module }]
        }]
    }))
    .unwrap();
    let mut value = serde_json::to_value(package).unwrap();
    let resource = b"\0binary asset, retained even without a bridge reader\xff";
    value["manifest"]["version"] = version.into();
    value["asset_manifest"] = serde_json::json!({
        "schema_version": 1, "package_id": "frozen-vm",
        "total_compressed_size": 0, "total_decoded_size": 0,
        "entries": [{
            "asset_id": "frozen-vm:resource", "source_hash": blake3::hash(resource).as_bytes(),
            "kind": "Data", "image_variants": [], "audio_variants": [],
            "priority": "Lazy", "depends_on": [], "platforms": []
        }, {
            "asset_id": "frozen-vm:characters", "source_hash": blake3::hash(&characters).as_bytes(),
            "kind": "Data", "image_variants": [], "audio_variants": [],
            "priority": "Lazy", "depends_on": [], "platforms": []
        }]
    });
    value["asset_paths"] = serde_json::json!({
        "frozen-vm:resource": "assets/resource.bin",
        "frozen-vm:characters": "assets/character.json"
    });
    let mut package: content_schema::ContentPackage = serde_json::from_value(value).unwrap();
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    // 保留原始空白，不能以反序列化后重录 JSON 的指纹代替内容锁。
    let mut bytes = package.to_json_bytes().unwrap();
    bytes.extend_from_slice(b"\n  \n");
    std::fs::create_dir_all(root.join("assets")).unwrap();
    std::fs::write(root.join("package.npack.json"), &bytes).unwrap();
    std::fs::write(root.join("assets/resource.bin"), resource).unwrap();
    std::fs::write(root.join("assets/character.json"), characters).unwrap();
    bytes
}

fn fixture() -> (
    tempfile::TempDir,
    NativeContent,
    String,
    FrozenNativePackage,
) {
    let temp = tempfile::tempdir().unwrap();
    let sources = temp.path().join("sources");
    let bytes = write_package(&sources.join("frozen-vm"), "1.0.0", DSL);
    let content = NativeContent::open(sources, temp.path().join("store")).unwrap();
    content.import().unwrap();
    let loaded = content.load("frozen-vm").unwrap();
    assert_eq!(loaded.frozen.package_json.as_bytes(), bytes);
    let reference = format!("{}@{}", loaded.pack.def.name, loaded.pack.def.version);
    (temp, content, reference, loaded.frozen)
}

fn with_rehashed_package(
    copy: &mut FrozenNativePackage,
    edit: impl FnOnce(&mut content_schema::ContentPackage),
) {
    let mut package =
        content_schema::ContentPackage::from_json_bytes(copy.package_json.as_bytes()).unwrap();
    edit(&mut package);
    copy.package_json = String::from_utf8(package.to_json_bytes().unwrap()).unwrap();
    copy.binding.content_hash = content_runtime::content_hash_hex(copy.package_json.as_bytes());
}

fn copy_reference(copy: &FrozenNativePackage) -> String {
    format!(
        "installed:{}@1",
        serde_json::to_string(&copy.binding).unwrap()
    )
}

#[test]
fn frozen_vm_and_binary_assets_restore_without_sources_or_installed_versions() {
    let (temp, content, reference, frozen) = fixture();
    let pack = content.load_frozen_reference(&reference, &frozen).unwrap();
    assert_eq!(pack.vm_skills.len(), 1);
    let engine = engine_native::Engine::new_with_extensions(4242, 2, &[&pack]).unwrap();
    let snapshot = engine.save();
    std::fs::remove_dir_all(temp.path().join("sources")).unwrap();
    content.import().unwrap();
    std::fs::remove_dir_all(temp.path().join("store/packages")).unwrap();
    assert!(content.load("frozen-vm").is_err());
    assert!(content.load_reference(&reference).is_err());
    let (packs, retained) = content
        .load_room_packages(&[reference], Some(&[frozen]))
        .unwrap();
    assert_eq!(retained[0].assets.len(), 2);
    assert_eq!(packs[0].vm_skills.len(), 1);
    let restored = engine_native::Engine::restore_with_extensions(&snapshot, &[&packs[0]]).unwrap();
    assert_eq!(restored.state_hash(), engine.state_hash());
}

#[test]
fn vm_character_survives_upgrade_rollback_and_uninstall_via_frozen_copy() {
    let (temp, content, reference, frozen) = fixture();
    let original = content.load("frozen-vm").unwrap();
    let mut engine =
        engine_native::Engine::new_with_extensions(990_003, 4, &[&original.pack]).unwrap();
    engine.assign_general_named(0, "frozen_probe").unwrap();
    assert!(engine.view_for(0).players[0]
        .skills
        .contains(&"frozen_vm_phase_end"));
    engine.debug_set_hp(0, 2);
    let snapshot = engine.save();

    let upgraded_dsl = DSL.replace("recover_hp(self, 1)", "recover_hp(self, 2)");
    write_package(
        &temp.path().join("sources/frozen-vm"),
        "2.0.0",
        &upgraded_dsl,
    );
    content.import().unwrap();
    let upgraded = content.load("frozen-vm").unwrap();
    assert_eq!(upgraded.frozen.binding.version, "2.0.0");
    assert_ne!(
        upgraded.frozen.binding.logic_hash,
        frozen.binding.logic_hash
    );
    assert!(matches!(
        engine_native::Engine::restore_with_extensions(&snapshot, &[&upgraded.pack]),
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. })
    ));
    let mut upgraded_engine =
        engine_native::Engine::new_with_extensions(990_003, 4, &[&upgraded.pack]).unwrap();
    upgraded_engine
        .assign_general_named(0, "frozen_probe")
        .unwrap();
    upgraded_engine.debug_set_hp(0, 2);
    advance_until_recovery(&mut upgraded_engine, 4);
    let (upgrade_packs, _) = content
        .load_room_packages(&[reference.clone()], Some(&[frozen.clone()]))
        .unwrap();
    let mut during_upgrade =
        engine_native::Engine::restore_with_extensions(&snapshot, &[&upgrade_packs[0]]).unwrap();
    content.rollback().unwrap();
    let rolled_back = content.load("frozen-vm").unwrap();
    assert_eq!(rolled_back.frozen.binding, frozen.binding);
    let mut after_rollback =
        engine_native::Engine::restore_with_extensions(&snapshot, &[&rolled_back.pack]).unwrap();

    content
        .uninstall_package(&frozen.binding.namespace, "frozen-vm")
        .unwrap();
    assert!(content.load("frozen-vm").is_err());
    assert!(content
        .load_room_packages(&[reference.clone()], None)
        .is_err());
    let (packs, retained) = content
        .load_room_packages(&[reference], Some(&[frozen.clone()]))
        .unwrap();
    assert_eq!(retained[0].binding, frozen.binding);
    let mut restored =
        engine_native::Engine::restore_with_extensions(&snapshot, &[&packs[0]]).unwrap();

    for branch in [
        &mut engine,
        &mut during_upgrade,
        &mut after_rollback,
        &mut restored,
    ] {
        advance_until_recovery(branch, 3);
    }
    assert_eq!(during_upgrade.state_hash(), engine.state_hash());
    assert_eq!(after_rollback.state_hash(), engine.state_hash());
    assert_eq!(restored.state_hash(), engine.state_hash());
}

fn advance_until_recovery(engine: &mut engine_native::Engine, expected_hp: i32) {
    for step in 0..80u64 {
        if engine.view_for(0).players[0].hp == expected_hp {
            assert_eq!(engine.turn().current, 0);
            assert_eq!(engine.turn().phase, engine_native::Phase::End);
            return;
        }
        let (actor, action) = if let Some(choice) = engine.open_choice() {
            let selection = choice
                .spec
                .options
                .iter()
                .take(choice.spec.min as usize)
                .cloned()
                .collect();
            (
                choice.observer,
                engine_native::GameAction::Answer {
                    choice_id: choice.id,
                    selection,
                },
            )
        } else {
            (engine.turn().current, engine_native::GameAction::NextPhase)
        };
        let result = engine.submit(engine_native::GameCommand {
            actor,
            expected_revision: engine.revision(),
            dedup_id: 30_000 + step,
            action,
        });
        assert!(
            !matches!(result, engine_native::CommandOutcome::Rejected(_)),
            "{result:?}"
        );
    }
    panic!("VM 技能未在结束阶段回复至 {expected_hp} 点体力");
}

#[test]
fn frozen_copy_rejects_tampering_missing_extra_assets_and_invalid_manifest() {
    let (_temp, content, reference, frozen) = fixture();
    let reject = |copy: &FrozenNativePackage| {
        assert!(content.load_frozen_reference(&reference, copy).is_err());
    };
    let mut raw_changed = frozen.clone();
    raw_changed.package_json.push('\n');
    reject(&raw_changed);
    let mut asset_changed = frozen.clone();
    asset_changed.assets.get_mut("assets/resource.bin").unwrap()[0] ^= 1;
    reject(&asset_changed);
    let mut missing = frozen.clone();
    missing.assets.clear();
    reject(&missing);
    let mut extra = frozen.clone();
    extra.assets.insert("assets/undeclared.bin".into(), vec![0]);
    reject(&extra);
    let mut wrong_logic = frozen.clone();
    wrong_logic.binding.logic_hash = "00".repeat(32);
    assert!(content
        .load_frozen_reference(&copy_reference(&wrong_logic), &wrong_logic)
        .is_err());
    let mut invalid_manifest = frozen.clone();
    with_rehashed_package(&mut invalid_manifest, |pkg| {
        pkg.manifest.logic_hash = [0; 32]
    });
    assert!(content
        .load_frozen_reference(&copy_reference(&invalid_manifest), &invalid_manifest)
        .is_err());
    let mut traversal = frozen.clone();
    with_rehashed_package(&mut traversal, |pkg| {
        pkg.asset_paths
            .insert("frozen-vm:resource".into(), "../outside".into());
        pkg.manifest.asset_hash = pkg.compute_asset_hash();
    });
    assert!(content
        .load_frozen_reference(&copy_reference(&traversal), &traversal)
        .is_err());
}

#[test]
fn room_packages_are_exact_ordered_and_do_not_fallback_to_active_packages() {
    let (_temp, content, reference, frozen) = fixture();
    assert!(content
        .load_room_packages(&[reference.clone()], Some(&[]))
        .is_err());
    assert!(content
        .load_room_packages(&[], Some(&[frozen.clone()]))
        .is_err());
    assert!(content
        .load_room_packages(
            &[reference.clone(), reference.clone()],
            Some(&[frozen.clone(), frozen.clone()])
        )
        .is_err());
    let mut wrong = frozen.clone();
    wrong.binding.namespace = "different-namespace".into();
    assert!(content
        .load_room_packages(&[reference.clone()], Some(&[wrong]))
        .is_err());
    assert!(content
        .load_room_packages(&[reference.replace("@1", "@2")], Some(&[frozen]))
        .is_err());
    assert!(content
        .load_room_packages(&["frozen-vm@1".into()], None)
        .is_err());
    assert!(content.load_room_packages(&[], None).unwrap().0.is_empty());
    assert!(content
        .load_room_packages(&[], Some(&[]))
        .unwrap()
        .1
        .is_empty());
}

#[test]
fn legacy_exact_reference_migrates_disabled_old_version_and_upgrade_leaves_original_copy() {
    let (temp, content, reference, frozen) = fixture();
    let source = temp.path().join("sources/frozen-vm");
    std::fs::write(
        source.join("manifest.json"),
        br#"{"schema_version":1,"enabled":false}"#,
    )
    .unwrap();
    assert!(content.import().unwrap().is_empty(), "全停用是合法配置");
    assert!(content.load("frozen-vm").is_err(), "停用包不能开新局");
    let (old, migrated) = content
        .load_room_packages(&[reference.clone()], None)
        .unwrap();
    assert_eq!(migrated[0].binding, frozen.binding);
    assert_eq!(old[0].def.name, reference.rsplit_once('@').unwrap().0);
    std::fs::remove_file(source.join("manifest.json")).unwrap();
    write_package(&source, "2.0.0", DSL);
    content.import().unwrap();
    let current = content.load("frozen-vm").unwrap();
    assert_eq!(current.frozen.binding.version, "2.0.0");
    assert_ne!(current.frozen.binding, frozen.binding);
    let (_, migrated_again) = content
        .load_room_packages(&[reference.clone()], None)
        .unwrap();
    assert_eq!(migrated_again[0].binding.version, "1.0.0");
    let old_root = temp
        .path()
        .join("store/packages")
        .join(format!(
            "{}__{}",
            frozen.binding.namespace, frozen.binding.package_id
        ))
        .join("1.0.0");
    std::fs::remove_dir_all(old_root).unwrap();
    assert!(
        content
            .load_room_packages(&[reference.clone()], None)
            .is_err(),
        "旧档不得猜新版"
    );
    content
        .load_room_packages(&[reference], Some(&[frozen]))
        .unwrap();
}

#[test]
fn frozen_copies_obey_current_hash_and_slot_revocations() {
    for by_hash in [true, false] {
        let (temp, content, reference, frozen) = fixture();
        let store = content_runtime::PackageStore::open(temp.path().join("store")).unwrap();
        if by_hash {
            store
                .revoke_content_hash(&frozen.binding.content_hash, "test", true)
                .unwrap();
        } else {
            store
                .revoke_slot(
                    &frozen.binding.namespace,
                    &frozen.binding.package_id,
                    "test",
                    true,
                )
                .unwrap();
        }
        assert!(content
            .load_frozen_reference(&reference, &frozen)
            .err()
            .expect("撤销应拒绝")
            .contains("已撤销"));
        assert!(content
            .freeze_reference(&reference)
            .unwrap_err()
            .contains("已撤销"));
    }
}

#[cfg(unix)]
#[test]
fn freezing_installed_assets_rejects_symlinks_outside_package_root() {
    use std::os::unix::fs::symlink;
    let (temp, content, reference, frozen) = fixture();
    let installed = temp
        .path()
        .join("store/packages")
        .join(format!(
            "{}__{}",
            frozen.binding.namespace, frozen.binding.package_id
        ))
        .join(&frozen.binding.version);
    let asset = installed.join("assets/resource.bin");
    let outside = temp.path().join("outside.bin");
    std::fs::write(&outside, frozen.assets.get("assets/resource.bin").unwrap()).unwrap();
    std::fs::remove_file(&asset).unwrap();
    symlink(&outside, &asset).unwrap();
    assert!(content
        .freeze_reference(&reference)
        .unwrap_err()
        .contains("越界"));
}
