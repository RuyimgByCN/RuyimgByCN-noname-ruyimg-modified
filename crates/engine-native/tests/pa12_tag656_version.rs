use engine_native::{archive::{decode_archive, encode_archive, ARCHIVE_FORMAT_VERSION}, Engine};

#[test]
fn v322_archive_is_rejected_and_current_archive_round_trips() {
    let old = include_bytes!("../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/native-v322.ngsa");
    // Decision29 rejects the original105 envelope before any historical rules gate.
    assert!(matches!(decode_archive(old), Err(engine_native::archive::ArchiveError::UnsupportedFormatVersion(105))));
    // R05-G 批（2026-11-04）：106→107——detached 实体区入档，v106 拒收无迁移。
    assert_eq!(ARCHIVE_FORMAT_VERSION, 107);
    let engine = Engine::new(65101, 4);
    let snapshot = decode_archive(&encode_archive(&engine.save())).unwrap();
    assert_eq!(snapshot.ruleset_version, 333);
    assert_eq!(Engine::restore(&snapshot).unwrap().state_hash(), engine.state_hash());
}
