//! M8 gate 1: 官方内容包均有许可/来源元数据.
//!
//! Every ported package's `metadata.json` must declare a `license` block and
//! the catalog must record the upstream source for every skill. This test walks
//! the committed ported packages at build time and fails when any package lacks
//! license provenance — the M8 governance acceptance evidence.

use std::path::Path;

#[test]
fn every_ported_package_declares_license_metadata() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/original/ported");
    let mut checked = 0usize;
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(&root).unwrap() {
        let entry = entry.unwrap();
        if !entry.path().is_dir() {
            continue;
        }
        let metadata = entry.path().join("metadata.json");
        if !metadata.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&metadata).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        if value.get("license").is_some() {
            checked += 1;
        } else {
            missing.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    assert!(
        checked >= 30,
        "expected license metadata across the ported packages, got {checked}"
    );
    assert!(
        missing.is_empty(),
        "packages without license metadata: {missing:?}"
    );
}

#[test]
fn catalog_records_upstream_source_for_every_skill() {
    let catalog_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/original/catalog.json");
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&catalog_path).unwrap()).unwrap();
    let skills = value
        .get("skills")
        .and_then(|v| v.as_array())
        .expect("catalog skills array");
    let missing_source = skills
        .iter()
        .filter(|skill| skill.get("source").and_then(|s| s.as_str()).is_none())
        .count();
    assert_eq!(
        missing_source, 0,
        "every catalog skill must record its upstream source"
    );
    assert!(skills.len() > 5000, "full-library inventory present");
}
