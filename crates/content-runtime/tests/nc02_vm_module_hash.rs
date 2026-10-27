//! Installation checks each module even when the package has a valid signature.
use content_runtime::{sign_package, InstallOptions, PackageStore, RuntimeError};
use content_schema::ContentPackage;

#[test]
fn signed_invalid_module_is_rejected_without_changing_active_content() {
    let source =
        include_bytes!("../../../apps/web-client/scripts/fixtures/vm-trigger/package.npack.json");
    let mut package = ContentPackage::from_json_bytes(source).unwrap();
    package.asset_manifest = None;
    package.asset_paths.clear();
    package.manifest.asset_hash = package.compute_asset_hash();
    package.validate().unwrap();
    let bytes = package.to_json_bytes().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let opts = InstallOptions {
        activate: true,
        allow_unsigned: false,
        verify_key: Some("test-key".into()),
        host: content_runtime::tooling_host_versions(),
    };
    let signature = sign_package(&bytes, "test-key", "test").unwrap();
    store.install(&bytes, Some(&signature), &opts).unwrap();
    let before = store.current_lock().unwrap();
    package.manifest.version = "0.1.1".into();
    package.manifest.logic_hash = package.compute_logic_hash();
    package.modules[0].local_count += 1;
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    let invalid = package.to_json_bytes().unwrap();
    let signature = sign_package(&invalid, "test-key", "test").unwrap();
    let result = store.install(&invalid, Some(&signature), &opts);
    assert!(
        matches!(result, Err(RuntimeError::Package(ref error)) if error.contains("logic_hash mismatch")),
        "unexpected install result: {result:?}"
    );
    assert_eq!(store.current_lock().unwrap(), before);
}
