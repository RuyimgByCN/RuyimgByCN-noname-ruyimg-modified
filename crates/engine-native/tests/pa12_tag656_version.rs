use engine_native::{
    archive::{decode_archive, encode_archive, ArchiveError, ARCHIVE_FORMAT_VERSION},
    Engine, SnapshotError,
};

#[test]
fn v322_archive_is_rejected_and_current_archive_round_trips() {
    let old = include_bytes!(
        "../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/native-v322.ngsa"
    );
    // Decision 29 rejects the original 105 envelope before any historical rules gate.
    assert!(matches!(
        decode_archive(old),
        Err(ArchiveError::UnsupportedFormatVersion(105))
    ));
    // R05-G 批（2026-11-04）：106→107——detached 实体区入档，v106 拒收无迁移。
    assert_eq!(ARCHIVE_FORMAT_VERSION, 107);
    let engine = Engine::new(65101, 4);
    let snapshot = decode_archive(&encode_archive(&engine.save())).unwrap();
    assert_eq!(snapshot.ruleset_version, 333);
    assert_eq!(
        Engine::restore(&snapshot).unwrap().state_hash(),
        engine.state_hash()
    );
}

#[test]
fn historical_ngsa_105_106_and_ruleset_322_to_332_archives_fail_closed() {
    for bytes in [
        include_bytes!("../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/native-v322.ngsa").as_slice(),
        include_bytes!("../../../docs/operations/evidence/2026-10-28-b7-vm-suspend-restore/native-v323.ngsa").as_slice(),
        include_bytes!("../../../docs/operations/evidence/2026-10-28-nc07-lianyaohu-hash/native-v324.ngsa").as_slice(),
        include_bytes!("../../../docs/operations/evidence/2026-10-29-nc04-cards-given-v326/native-v325.ngsa").as_slice(),
        include_bytes!("../../../docs/operations/evidence/2026-10-29-nc07-archive-hash-fields/native-v326.ngsa").as_slice(),
        include_bytes!("fixtures/c331_phase7_genuine_105.ngsa").as_slice(),
        include_bytes!("fixtures/c331-real330-pending.ngsa").as_slice(),
    ] {
        assert!(matches!(
            decode_archive(bytes),
            Err(ArchiveError::UnsupportedFormatVersion(105))
        ));
    }

    for bytes in [
        include_bytes!("fixtures/detached-106.ngsa").as_slice(),
        include_bytes!("fixtures/gaiming-window-331.ngsa").as_slice(),
    ] {
        assert!(matches!(
            decode_archive(bytes),
            Err(ArchiveError::UnsupportedFormatVersion(106))
        ));
    }

    let engine = Engine::new(20261105, 4);
    for old_rule in [322u64, 323, 324, 325, 326, 330, 331, 332] {
        // 1) Header vs payload ruleset mismatch is rejected at archive decode boundary.
        let mut mismatched = engine.save();
        mismatched.ruleset_version = old_rule;
        assert!(matches!(
            decode_archive(&encode_archive(&mismatched)),
            Err(ArchiveError::RuleVersionMismatch {
                header,
                payload: 333
            }) if header == old_rule
        ));

        // 2) Internally consistent old ruleset snapshot is rejected at Engine::restore boundary.
        let mut snap = engine.save();
        snap.ruleset_version = old_rule;
        snap.core.ruleset_version = old_rule;
        let decoded = decode_archive(&encode_archive(&snap)).unwrap();
        assert_eq!(decoded.ruleset_version, old_rule);
        assert!(matches!(
            Engine::restore(&decoded),
            Err(SnapshotError::RuleVersionMismatch {
                snapshot,
                catalog: 333
            }) if snapshot == old_rule
        ));
    }
}
