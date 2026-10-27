use content_schema::ContentPackage;
use engine_core::{snapshot::{Snapshot, SnapshotError}, Engine};

const OLD_ONLYOL: &[u8] = include_bytes!("../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/onlyOL-0.1.4.npack.json");
const OLD_ONLYOL_012: &[u8] = include_bytes!("../../../docs/operations/evidence/2026-10-24-up37-n01-zhuhuan/onlyOL-0.1.2.npack.json");
const NEW_ONLYOL: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn ambiguous_packages_reject_all_four_abi_values_without_breaking_sp() {
    for bytes in [OLD_ONLYOL, OLD_ONLYOL_012] {
        let original = ContentPackage::from_json_bytes(bytes).unwrap();
        for abi in 754..=757 {
            let mut package = original.clone();
            package.manifest.rule_vm_abi_version = abi;
            package.manifest.logic_hash = package.compute_logic_hash();
            assert!(package.validate().unwrap_err().contains("ambiguous effect 656"));
            assert!(Engine::new().load_package(&package).unwrap_err().to_string().contains("ambiguous effect 656"));
        }
    }
    let sp = ContentPackage::from_json_bytes(SP).unwrap();
    assert_eq!(sp.manifest.rule_vm_abi_version, 745);
    sp.validate().unwrap();
    let mut engine = Engine::new();
    engine.load_package(&sp).unwrap();
    let bytes = engine.snapshot().to_bytes().unwrap();
    Snapshot::from_bytes(&bytes).unwrap().restore().unwrap();

    let mut new = ContentPackage::from_json_bytes(NEW_ONLYOL).unwrap();
    assert_eq!((new.manifest.version.as_str(), new.manifest.rule_vm_abi_version), ("0.1.5", 758));
    new.validate().unwrap();
    new.manifest.rule_vm_abi_version = 757;
    new.manifest.logic_hash = new.compute_logic_hash();
    assert!(new.validate().unwrap_err().contains("effect 772 below ABI 758"));
}

#[test]
fn pre_fix_match_snapshots_with_ambiguous_or_unrecorded_abi_fail_closed() {
    for bytes in [
        include_bytes!("../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/onlyOL-0.1.2-legacy-match-snapshot.json").as_slice(),
        include_bytes!("../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/onlyOL-0.1.4-legacy-match-snapshot.json").as_slice(),
        include_bytes!("../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/sp-0.1.0-legacy-match-snapshot.json").as_slice(),
    ] {
        let snapshot = Snapshot::from_bytes(bytes).unwrap();
        assert!(matches!(snapshot.restore(), Err(SnapshotError::ContentCompatibility(_))));
    }
}

#[test]
fn frozen_snapshot_with_explicit_ambiguous_abi_is_rejected() {
    let original: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/onlyOL-0.1.4-legacy-match-snapshot.json"
    ))
    .unwrap();
    for abi in 754..=757 {
        let mut snapshot = original.clone();
        snapshot["state"]["content"]["packages"][0]["rule_vm_abi_version"] = abi.into();
        let bytes = serde_json::to_vec(&snapshot).unwrap();
        let result = Snapshot::from_bytes(&bytes).unwrap().restore();
        assert!(matches!(result, Err(SnapshotError::ContentCompatibility(_))), "ABI {abi}");
    }
}
