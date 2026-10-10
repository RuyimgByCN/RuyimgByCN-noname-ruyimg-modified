use content_runtime::{import_extensions, InstallOptions, PackageStore};
use std::path::Path;

#[test]
fn cold_import_all_26() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../extensions");
    let opts = InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };
    let report = import_extensions(&store, &root, &opts).unwrap();
    println!(
        "installed={:?} skipped={:?}",
        report.installed.len(),
        report.skipped
    );
    assert!(
        report.installed.len() >= 25,
        "expected >=25 installed, got {}",
        report.installed.len()
    );
}
