//! J2 content release: published package bytes and exact old locks survive an upgrade.
use content_runtime::{ContentLock, InstallOptions, PackageStore};
use content_schema::ContentPackage;

#[test]
fn shiji_old_lock_retains_published_bytes_and_new_full_package_rejects_missing_native_declarations()
{
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/shiji");
    let old_bytes = std::fs::read(root.join("archive/shiji-0.2.4.npack.json")).unwrap();
    let new_bytes = std::fs::read(root.join("shiji.npack.json")).unwrap();
    let old = ContentPackage::from_json_bytes(&old_bytes).unwrap();
    let new = ContentPackage::from_json_bytes(&new_bytes).unwrap();
    assert_eq!(old.manifest.version, "0.2.4");
    assert_eq!(new.manifest.version, "0.2.5");
    assert_eq!(old.manifest.rule_vm_abi_version, 756);
    assert_eq!(new.manifest.rule_vm_abi_version, 756);
    assert_eq!(old.asset_manifest, new.asset_manifest);
    assert_eq!(old.asset_paths, new.asset_paths);
    assert_eq!(old.manifest.asset_hash, new.manifest.asset_hash);
    assert_eq!(old.modules.len(), new.modules.len());
    assert_eq!(old.trigger_index.len(), new.trigger_index.len());
    assert_eq!(old.active_index.len(), new.active_index.len());
    assert_ne!(old.manifest.logic_hash, new.manifest.logic_hash);
    let old_pinghe = old
        .trigger_index
        .iter()
        .find(|entry| entry.module == "pinghe_damage_before")
        .unwrap();
    let new_pinghe = new
        .trigger_index
        .iter()
        .find(|entry| entry.module == "pinghe_damage_before")
        .unwrap();
    assert_eq!(old_pinghe.filter, "owner_is_target");
    assert_eq!(new_pinghe.filter, "owner_is_target_other_source_alive");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let options = InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };
    store
        .install_with_assets(&old_bytes, None, &options, &root)
        .unwrap();
    let old_lock: ContentLock =
        serde_json::from_slice(&serde_json::to_vec(&store.current_lock().unwrap()).unwrap())
            .unwrap();
    store
        .install_with_assets(&new_bytes, None, &options, &root)
        .unwrap();
    let new_lock = store.current_lock().unwrap();
    assert_ne!(old_lock.lock_hash, new_lock.lock_hash);
    let restored_old = store.load_locked_packages(&old_lock).unwrap();
    assert_eq!(restored_old.len(), 1);
    assert_eq!(
        restored_old[0], old,
        "an exact old lock never silently selects the new package"
    );
    assert_eq!(
        store.load_locked_packages(&new_lock).unwrap(),
        vec![new.clone()]
    );
    store
        .activate_version("official", "shiji", "0.2.4")
        .unwrap();
    assert_eq!(store.current_lock().unwrap().lock_hash, old_lock.lock_hash);
    assert_eq!(
        store.load_locked_packages(&new_lock).unwrap(),
        vec![new.clone()],
        "rollback preserves the newer frozen copy too"
    );
    let error = content_native_loader::bridge_vm_skill_decls(&new)
        .unwrap_err()
        .to_string();
    // A2 D02（R05 准备阶段真实化）：`phase_prepare` 已可装（Phase::Prepare
    // 门），完整包的拒装锚改为首个仍未映射的 `game_started`。
    assert!(
        error.contains("game_started"),
        "the complete package remains explicitly unsupported: {error}"
    );
}
