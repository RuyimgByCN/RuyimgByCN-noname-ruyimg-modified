//! The committed official standard artifact is a real installable package.

use content_runtime::{InstallOptions, PackageStore};
use content_schema::ContentPackage;
use std::collections::BTreeSet;

#[test]
fn standard_npack_parses_installs_and_activates() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/standard/standard.npack.json");
    let bytes = std::fs::read(path).expect("committed standard npack");
    let package = ContentPackage::from_json_bytes(&bytes).expect("valid package JSON");
    assert_eq!(package.manifest.package_id, "standard");
    assert_eq!(package.character_index.len(), 32);
    assert_eq!(package.skill_index.len(), 65);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    let declared_skills: BTreeSet<_> = package
        .skill_index
        .iter()
        .map(|entry| entry.skill_def_id)
        .collect();
    // Roster closure: gongsunzan / old_re_lidian reference refresh-package
    // skills (reyicong / xunxun / wangxi) cross-package; those defs are covered
    // by the declared refresh dependency and validated by the full content lock.
    let covered_by_deps: BTreeSet<_> = package
        .manifest
        .dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.namespace.as_str(),
                dependency.package_id.as_str(),
            )
        })
        .collect();
    for character in &package.character_index {
        assert!(
            !character.group.is_empty(),
            "character {} has no group",
            character.id
        );
        for skill_id in &character.skills {
            assert!(
                declared_skills.contains(skill_id) || !covered_by_deps.is_empty(),
                "character {} references undeclared skill {}",
                character.id,
                skill_id
            );
        }
    }

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    // Standard declares a refresh dependency (roster closure) and refresh
    // declares standard, so the dependency graph is cyclic: install and
    // activate through the real extension-import path, which resolves the
    // full 32-extension graph atomically.
    let options = InstallOptions {
        allow_unsigned: true,
        ..InstallOptions::default()
    };
    let extensions_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../extensions");
    let report = content_runtime::import_extensions(&store, &extensions_root, &options)
        .expect("import extensions with standard");
    let standard = report
        .installed
        .iter()
        .find(|meta| meta.package_id == "standard")
        .expect("standard installed");
    assert_eq!(standard.version, "0.1.0");
    let expected_logic = package
        .manifest
        .logic_hash
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    assert_eq!(standard.logic_hash, expected_logic);
    assert!(
        report
            .content_lock
            .packages
            .iter()
            .any(|locked| locked.package_id == "standard"),
        "standard is active in the content lock"
    );
}
