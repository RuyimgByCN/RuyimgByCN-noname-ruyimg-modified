//! A valid package signature cannot make malformed VM control flow installable.
use content_runtime::{sign_package, InstallOptions, PackageStore, RuntimeError};
use content_schema::ContentPackage;
use rule_vm::{op, BytecodeModule};
use serde_json::Value;

#[test]
fn signed_invalid_upgrade_does_not_replace_active_package() {
    let saves: Vec<Value> = serde_json::from_slice(include_bytes!(
        "../../../apps/web-client/scripts/fixtures/vm-control-flow/old-native-saves.json"
    ))
    .unwrap();
    for case in saves {
        let packs: Vec<ContentPackage> =
            serde_json::from_str(case["packs"].as_str().unwrap()).unwrap();
        let mut invalid = packs.into_iter().next().unwrap();
        let mut good = invalid.clone();
        let module = &mut good.modules[0];
        module.code = vec![op::HALT];
        module.entry_pc = 0;
        module.logic_hash = BytecodeModule::compute_logic_hash(
            &module.name,
            &module.code,
            module.local_count,
            module.caps_mask,
            module.entry_pc,
        );
        good.manifest.logic_hash = good.compute_logic_hash();
        good.validate().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = PackageStore::open(dir.path()).unwrap();
        let opts = InstallOptions {
            activate: true,
            allow_unsigned: false,
            verify_key: Some("test-key".into()),
            host: content_runtime::tooling_host_versions(),
        };
        let bytes = good.to_json_bytes().unwrap();
        let signature = sign_package(&bytes, "test-key", "test").unwrap();
        store.install(&bytes, Some(&signature), &opts).unwrap();
        let before = store.current_lock().unwrap();
        invalid.manifest.version = "0.1.1".into();
        invalid.manifest.logic_hash = invalid.compute_logic_hash();
        let bytes = invalid.to_json_bytes().unwrap();
        let signature = sign_package(&bytes, "test-key", "test").unwrap();
        let result = store.install(&bytes, Some(&signature), &opts);
        assert!(
            matches!(result, Err(RuntimeError::Package(ref e)) if e.contains("instruction boundary")),
            "{}: {result:?}",
            case["case"]
        );
        assert_eq!(store.current_lock().unwrap(), before);
    }
}
