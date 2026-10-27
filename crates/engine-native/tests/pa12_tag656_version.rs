use engine_native::{archive::{decode_archive, encode_archive, ARCHIVE_FORMAT_VERSION}, snap::SnapshotError, Engine};

#[test]
fn v322_archive_is_rejected_and_v323_archive_round_trips() {
    let old = include_bytes!("../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/native-v322.ngsa");
    let snapshot = decode_archive(old).unwrap();
    assert_eq!(snapshot.ruleset_version, 322);
    assert!(matches!(Engine::restore(&snapshot), Err(SnapshotError::RuleVersionMismatch { snapshot: 322, catalog: 323 })));
    assert_eq!(ARCHIVE_FORMAT_VERSION, 105);
    let engine = Engine::new(65101, 4);
    let snapshot = decode_archive(&encode_archive(&engine.save())).unwrap();
    assert_eq!(snapshot.ruleset_version, 323);
    assert_eq!(Engine::restore(&snapshot).unwrap().state_hash(), engine.state_hash());
}
