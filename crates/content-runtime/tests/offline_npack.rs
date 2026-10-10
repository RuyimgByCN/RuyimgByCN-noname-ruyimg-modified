//! The committed OFFLINE artifact imports through the real extension/package store path.

use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency};

fn copy_dir(source: &std::path::Path, target: &std::path::Path) {
    std::fs::create_dir_all(target).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let dst = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &dst);
        } else {
            std::fs::copy(entry.path(), &dst).unwrap();
        }
    }
}

#[test]
fn offline_package_imports_with_current_host_and_exact_dependencies() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let committed_extensions = workspace.join("extensions");
    let store_dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(store_dir.path().join("store")).unwrap();
    let options = InstallOptions {
        activate: false,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };

    let mut dependencies = Vec::new();
    for entry in std::fs::read_dir(&committed_extensions).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() || entry.file_name() == "offline" {
            continue;
        }
        let path = entry.path().join("package.npack.json");
        if !path.is_file() {
            continue;
        }
        let bytes = std::fs::read(path).unwrap();
        let package = ContentPackage::from_json_bytes(&bytes).unwrap();
        store
            .install_with_assets(&bytes, None, &options, &entry.path())
            .unwrap();
        dependencies.push(PackageDependency {
            namespace: package.manifest.namespace,
            package_id: package.manifest.package_id,
            version: package.manifest.version,
            logic_hash: package.manifest.logic_hash,
        });
    }
    store.activate_packages(&dependencies).unwrap();

    let extension_root = store_dir.path().join("extensions");
    let offline_root = extension_root.join("offline");
    std::fs::create_dir_all(&offline_root).unwrap();
    let bytes = std::fs::read(workspace.join("extensions/offline/package.npack.json")).unwrap();
    let package = ContentPackage::from_json_bytes(&bytes).unwrap();
    assert!(package.manifest.rule_vm_abi_version <= engine_api::RULE_VM_ABI_VERSION);
    std::fs::write(offline_root.join("package.npack.json"), &bytes).unwrap();
    // The offline package ships assets (ADR 0610/0611); stage them alongside the npack.
    let committed_offline = workspace.join("extensions/offline");
    let committed_assets = committed_offline.join("assets");
    if committed_assets.is_dir() {
        let target_assets = offline_root.join("assets");
        std::fs::create_dir_all(&target_assets).unwrap();
        for entry in std::fs::read_dir(&committed_assets).unwrap() {
            let entry = entry.unwrap();
            let target = target_assets.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_dir(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), &target).unwrap();
            }
        }
    }

    let report = import_extensions(
        &store,
        &extension_root,
        &InstallOptions {
            activate: true,
            ..options
        },
    )
    .unwrap();
    assert_eq!(report.installed.len(), 1);
    assert_eq!(report.installed[0].package_id, "offline");
    assert!(report
        .content_lock
        .packages
        .iter()
        .any(|package| package.package_id == "offline"));
}
