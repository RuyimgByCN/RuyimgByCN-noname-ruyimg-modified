//! Atomic install / rollback / revoke / staging GC.

use content_runtime::{sign_package, ContentLock, InstallOptions, PackageStore, RuntimeError};
use content_schema::{ContentPackage, PackageDependency, PackageManifest};
use rule_vm::{bytecode::op, BytecodeModule};

fn sample_package(version: &str, package_id: &str) -> Vec<u8> {
    let mut pkg = ContentPackage {
        manifest: PackageManifest::new("core", package_id, version),
        modules: vec![BytecodeModule {
            name: "m".into(),
            code: vec![op::HALT],
            local_count: 4,
            max_stack: 16,
            entry_pc: 0,
            logic_hash: [0u8; 32],
            caps_mask: 0,
        }],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: Default::default(),
        asset_manifest: None,
        asset_paths: Default::default(),
        compiler_version: "test".into(),
    };
    pkg.modules[0].logic_hash = *blake3::hash(&pkg.modules[0].code).as_bytes();
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg.to_json_bytes().unwrap()
}

#[test]
fn signed_install_activate_rollback() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let bytes = sample_package("1.0.0", "standard");
    let sig = sign_package(&bytes, "secret", "dev").unwrap();
    let opts = InstallOptions {
        activate: true,
        allow_unsigned: false,
        verify_key: Some("secret".into()),
        host: content_runtime::tooling_host_versions(),
    };
    let r = store.install(&bytes, Some(&sig), &opts).unwrap();
    assert!(r.meta.signed);
    assert_eq!(r.meta.version, "1.0.0");
    let lock1 = store.current_lock().unwrap();
    assert_eq!(lock1.packages.len(), 1);

    // upgrade
    let bytes2 = sample_package("1.1.0", "standard");
    let sig2 = sign_package(&bytes2, "secret", "dev").unwrap();
    store.install(&bytes2, Some(&sig2), &opts).unwrap();
    let lock2 = store.current_lock().unwrap();
    assert_eq!(lock2.packages[0].version, "1.1.0");
    assert_ne!(lock1.lock_hash, lock2.lock_hash);

    let rolled = store.rollback().unwrap();
    assert_eq!(
        rolled.packages.get("core/standard").map(String::as_str),
        Some("1.0.0")
    );
    let lock3 = store.current_lock().unwrap();
    assert_eq!(lock3.lock_hash, lock1.lock_hash);
}

#[test]
fn unsigned_rejected_unless_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let bytes = sample_package("1.0.0", "std");
    let opts = InstallOptions {
        activate: true,
        allow_unsigned: false,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };
    let err = store.install(&bytes, None, &opts).unwrap_err();
    assert!(matches!(err, RuntimeError::UnsignedNotAllowed));

    let opts2 = InstallOptions {
        allow_unsigned: true,
        ..opts
    };
    store.install(&bytes, None, &opts2).unwrap();
}

#[test]
fn revoke_blocks_activate() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let bytes = sample_package("1.0.0", "evil");
    let sig = sign_package(&bytes, "secret", "dev").unwrap();
    let opts = InstallOptions {
        activate: true,
        allow_unsigned: false,
        verify_key: Some("secret".into()),
        host: content_runtime::tooling_host_versions(),
    };
    let r = store.install(&bytes, Some(&sig), &opts).unwrap();
    store
        .revoke_content_hash(&r.meta.content_hash, "policy", true)
        .unwrap();
    let active = store.read_active().unwrap();
    assert!(active.packages.is_empty());

    // reinstall same content blocked
    let err = store.install(&bytes, Some(&sig), &opts).unwrap_err();
    assert!(matches!(err, RuntimeError::Revoked(_)));
}

#[test]
fn staging_gc_clears_orphan() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let orphan = store.root().join("staging").join("crash-partial");
    std::fs::create_dir_all(&orphan).unwrap();
    std::fs::write(orphan.join("package.json"), b"partial").unwrap();
    assert_eq!(store.gc_staging().unwrap(), 1);
    assert!(!orphan.exists());
}

#[test]
fn lock_empty_stable() {
    let l = ContentLock::empty();
    assert!(l.packages.is_empty());
    assert!(!l.lock_hash.is_empty());
}

#[test]
fn bad_signature_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let bytes = sample_package("1.0.0", "std");
    let mut sig = sign_package(&bytes, "secret", "dev").unwrap();
    sig.signature = "00".repeat(32);
    let opts = InstallOptions {
        activate: true,
        allow_unsigned: false,
        verify_key: Some("secret".into()),
        host: content_runtime::tooling_host_versions(),
    };
    let err = store.install(&bytes, Some(&sig), &opts).unwrap_err();
    assert!(matches!(err, RuntimeError::Signature(_)));
}

#[test]
fn activation_requires_exact_dependencies() {
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let mut dependent = ContentPackage::from_json_bytes(&sample_package("1.0.0", "hero")).unwrap();
    dependent.manifest.dependencies.push(PackageDependency {
        namespace: "core".into(),
        package_id: "base".into(),
        version: "1.0.0".into(),
        logic_hash: [0u8; 32],
    });
    dependent.manifest.logic_hash = dependent.compute_logic_hash();
    let dependent = dependent.to_json_bytes().unwrap();
    let opts = InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };
    assert!(matches!(
        store.install(&dependent, None, &opts),
        Err(RuntimeError::Invalid(message)) if message.contains("missing dependency")
    ));
    assert!(store.read_active().unwrap().packages.is_empty());

    store
        .install(&sample_package("1.0.0", "base"), None, &opts)
        .unwrap();
    store.activate_version("core", "hero", "1.0.0").unwrap();
    assert_eq!(store.current_lock().unwrap().packages.len(), 2);
}
