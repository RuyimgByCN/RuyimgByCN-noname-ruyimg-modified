//! 卸载/停用必须先保证没有 active 依赖方，且失败时不得删除安装副本。

use content_runtime::{InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency, PackageManifest};
use std::collections::BTreeMap;

fn package(id: &str) -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new("community", id, "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::new(),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "test".into(),
    };
    package.manifest.logic_hash = package.compute_logic_hash();
    package
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn install(store: &PackageStore, package: &ContentPackage) {
    store
        .install(&package.to_json_bytes().unwrap(), None, &options())
        .unwrap();
}

#[test]
fn uninstall_and_disable_reject_active_dependents_without_mutation() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    install(&store, &package("base"));
    let mut dependent = package("dependent");
    dependent.manifest.dependencies = vec![PackageDependency {
        namespace: "community".into(),
        package_id: "base".into(),
        version: "1.0.0".into(),
        logic_hash: [0u8; 32],
    }];
    dependent.manifest.logic_hash = dependent.compute_logic_hash();
    install(&store, &dependent);

    assert!(store.uninstall_slot("community", "base").is_err());
    assert!(store.deactivate_slot("community", "base").is_err());
    assert!(store
        .list_installed()
        .unwrap()
        .iter()
        .any(|p| p.package_id == "base"));

    store.uninstall_slot("community", "dependent").unwrap();
    store.uninstall_slot("community", "base").unwrap();
    assert!(store.list_installed().unwrap().is_empty());
}
