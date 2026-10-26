use super::tests::record;
use super::*;

fn journal_path(dir: &Path) -> PathBuf {
    dir.join("7.json.journal")
}

fn restart_snapshot(dir: &Path) -> Option<String> {
    NativeRoomStore::new(dir)
        .load_all()
        .unwrap()
        .into_iter()
        .next()
        .map(|r| r.state.snapshot_hex)
}

#[test]
fn every_journal_fault_keeps_the_confirmed_record_across_two_restarts() {
    for fault in [
        StoreFault::PrepareWrite,
        StoreFault::PrepareSync,
        StoreFault::PrepareDirectorySync,
        StoreFault::DecisionWrite,
        StoreFault::DecisionSync,
        StoreFault::DecisionDirectorySync,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let store = NativeRoomStore::new(dir.path());
        store.save(&record(7, "confirmed")).unwrap();
        store.fail_next_write(fault);
        let error = store.save_durable(&record(7, "unconfirmed")).unwrap_err();
        assert!(!error.decision_uncertain, "{fault:?}: {error}");
        for _ in 0..2 {
            assert_eq!(
                restart_snapshot(dir.path()).as_deref(),
                Some("confirmed"),
                "{fault:?}"
            );
        }
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
        store.save(&record(7, "retry accepted")).unwrap();
        assert_eq!(
            restart_snapshot(dir.path()).as_deref(),
            Some("retry accepted")
        );
    }
}

#[test]
fn first_rename_and_decision_failures_preserve_proven_absence() {
    for fault in [
        StoreFault::PrepareWrite,
        StoreFault::PrepareSync,
        StoreFault::PrepareDirectorySync,
        StoreFault::DirectorySync,
        StoreFault::DecisionWrite,
        StoreFault::DecisionSync,
        StoreFault::DecisionDirectorySync,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let store = NativeRoomStore::new(dir.path());
        store.fail_next_write(fault);
        store
            .save_durable(&record(7, "first unconfirmed"))
            .unwrap_err();
        for _ in 0..2 {
            assert_eq!(restart_snapshot(dir.path()), None, "{fault:?}");
        }
        if journal_path(dir.path()).exists() {
            assert!(store.scan().unwrap().reserved_ids.contains(&7));
        }
        store.save(&record(7, "confirmed after retry")).unwrap();
        assert_eq!(
            restart_snapshot(dir.path()).as_deref(),
            Some("confirmed after retry")
        );
    }
}

#[test]
fn consecutive_failures_never_overwrite_the_only_undo_authority() {
    for has_previous in [false, true] {
        for second in [
            StoreFault::PrepareWrite,
            StoreFault::PrepareSync,
            StoreFault::PrepareDirectorySync,
            StoreFault::PartialWrite,
            StoreFault::FileSync,
            StoreFault::Rename,
            StoreFault::DirectorySync,
            StoreFault::DecisionDirectorySync,
        ] {
            let dir = tempfile::tempdir().unwrap();
            let store = NativeRoomStore::new(dir.path());
            if has_previous {
                store.save(&record(7, "confirmed")).unwrap();
            }
            store.fail_next_write(StoreFault::DirectorySync);
            store.save_durable(&record(7, "unconfirmed A")).unwrap_err();
            store.fail_next_write(second);
            store.save_durable(&record(7, "unconfirmed B")).unwrap_err();
            assert_eq!(
                restart_snapshot(dir.path()).as_deref(),
                has_previous.then_some("confirmed"),
                "has_previous={has_previous}, second={second:?}"
            );
        }
    }
}

#[test]
fn prepared_recovery_is_read_only_and_accepts_neither_candidate_nor_corruption() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    store.save(&record(7, "confirmed")).unwrap();
    store.fail_next_write(StoreFault::DirectorySync);
    store
        .save_durable(&record(7, "valid but unconfirmed"))
        .unwrap_err();
    let candidate = dir.path().join("7.json");
    let before = std::fs::read(&candidate).unwrap();
    let undo = std::fs::read(journal_path(dir.path())).unwrap();
    for _ in 0..2 {
        assert_eq!(restart_snapshot(dir.path()).as_deref(), Some("confirmed"));
        assert_eq!(std::fs::read(&candidate).unwrap(), before);
        assert_eq!(std::fs::read(journal_path(dir.path())).unwrap(), undo);
    }
    std::fs::write(&candidate, b"{truncated candidate").unwrap();
    assert_eq!(restart_snapshot(dir.path()).as_deref(), Some("confirmed"));
    assert_eq!(std::fs::read(&candidate).unwrap(), b"{truncated candidate");
    std::fs::remove_file(candidate).unwrap();
    assert_eq!(restart_snapshot(dir.path()).as_deref(), Some("confirmed"));
}

#[test]
fn missing_or_malformed_journal_cannot_downgrade_new_records_to_legacy() {
    for corruption in [None, Some(b"{truncated".as_slice()), Some(b"{}".as_slice())] {
        let dir = tempfile::tempdir().unwrap();
        let store = NativeRoomStore::new(dir.path());
        store.save(&record(7, "confirmed")).unwrap();
        match corruption {
            Some(bytes) => std::fs::write(journal_path(dir.path()), bytes).unwrap(),
            None => std::fs::remove_file(journal_path(dir.path())).unwrap(),
        }
        let scan = store.scan().unwrap();
        assert!(scan.records.is_empty());
        assert!(scan.reserved_ids.contains(&7));
        assert_eq!(scan.issues.len(), 1);
        assert!(store
            .save(&record(7, "must not overwrite evidence"))
            .is_err());
    }
}

#[test]
fn stale_prepared_journal_cannot_rewind_a_later_committed_generation() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    store.save(&record(7, "confirmed A")).unwrap();
    store.fail_next_write(StoreFault::DirectorySync);
    store.save_durable(&record(7, "unconfirmed B")).unwrap_err();
    let stale = std::fs::read(journal_path(dir.path())).unwrap();
    store.save(&record(7, "confirmed C")).unwrap();
    std::fs::write(journal_path(dir.path()), &stale).unwrap();
    let scan = store.scan().unwrap();
    assert!(scan.records.is_empty());
    assert!(scan.issues[0].error.contains("陈旧日志"));
    assert_eq!(std::fs::read(journal_path(dir.path())).unwrap(), stale);
}

#[test]
fn removing_journal_before_terminal_body_never_exposes_its_previous_game() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    store.save(&record(7, "ongoing")).unwrap();
    store.save(&record(7, "terminal")).unwrap();
    std::fs::remove_file(journal_path(dir.path())).unwrap();
    assert!(store.scan().unwrap().records.is_empty());
    assert!(store.scan().unwrap().reserved_ids.contains(&7));
    store.remove(7).unwrap();
    assert!(store.load_all().unwrap().is_empty());
}

#[test]
fn committed_record_must_match_every_byte_and_cannot_fall_back_if_missing() {
    for mutation in ["password", "state", "identity", "missing"] {
        let dir = tempfile::tempdir().unwrap();
        let store = NativeRoomStore::new(dir.path());
        store.save(&record(7, "old")).unwrap();
        store.save(&record(7, "committed")).unwrap();
        let path = dir.path().join("7.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        match mutation {
            "password" => value["password_hash"] = "other password".into(),
            "state" => value["state"]["snapshot_hex"] = "tampered".into(),
            "identity" => value["room_id"] = 99.into(),
            "missing" => std::fs::remove_file(&path).unwrap(),
            _ => unreachable!(),
        }
        if mutation != "missing" {
            std::fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        }
        let scan = store.scan().unwrap();
        assert!(scan.records.is_empty(), "{mutation}");
        assert!(!scan.issues.is_empty(), "{mutation}");
        assert!(scan.reserved_ids.contains(&7));
        if mutation == "identity" {
            assert!(scan.reserved_ids.contains(&99));
        }
    }
}

#[test]
fn legacy_without_journal_migrates_and_pending_migration_recovers_original_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    let legacy = serde_json::to_vec(&record(7, "legacy confirmed")).unwrap();
    std::fs::write(dir.path().join("7.json"), &legacy).unwrap();
    assert_eq!(
        restart_snapshot(dir.path()).as_deref(),
        Some("legacy confirmed")
    );
    store.fail_next_write(StoreFault::DirectorySync);
    store
        .save_durable(&record(7, "new unconfirmed"))
        .unwrap_err();
    assert_eq!(
        restart_snapshot(dir.path()).as_deref(),
        Some("legacy confirmed")
    );
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(journal_path(dir.path())).unwrap()).unwrap();
    assert_eq!(value["previous"].as_str().unwrap().as_bytes(), legacy);
    store.save(&record(7, "new committed")).unwrap();
    assert_eq!(
        restart_snapshot(dir.path()).as_deref(),
        Some("new committed")
    );
}

#[cfg(unix)]
#[test]
fn symlink_journal_is_not_a_recovery_authority_and_cleanup_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    store.save(&record(7, "confirmed")).unwrap();
    let external = dir.path().join("external-evidence");
    std::fs::rename(journal_path(dir.path()), &external).unwrap();
    std::os::unix::fs::symlink(&external, journal_path(dir.path())).unwrap();
    assert!(store.scan().unwrap().records.is_empty());
    let original = std::fs::read(&external).unwrap();
    store.remove(7).unwrap();
    store.remove(7).unwrap();
    assert!(store.load_all().unwrap().is_empty());
    assert_eq!(std::fs::read(external).unwrap(), original);
}

#[test]
fn double_faults_preserve_the_actual_decision_for_read_only_restart() {
    for has_previous in [false, true] {
        for primary in [
            StoreFault::DecisionWrite,
            StoreFault::DecisionSync,
            StoreFault::DecisionDirectorySync,
        ] {
            for compensation in [
                StoreFault::PrepareWrite,
                StoreFault::PrepareSync,
                StoreFault::PrepareDirectorySync,
            ] {
                let dir = tempfile::tempdir().unwrap();
                let store = NativeRoomStore::new(dir.path());
                if has_previous {
                    store.save(&record(7, "previous confirmed")).unwrap();
                }
                store
                    .clone()
                    .fail_next_write_with_compensation(primary, compensation);
                let error = store.save_durable(&record(7, "candidate")).unwrap_err();
                assert!(
                    error.replacement_visible,
                    "{primary:?}/{compensation:?}: {error}"
                );
                assert!(
                    error.decision_uncertain,
                    "{primary:?}/{compensation:?}: {error}"
                );
                assert!(error.message.contains("固定撤销决定失败"));
                let committed = primary == StoreFault::DecisionDirectorySync
                    && compensation != StoreFault::PrepareDirectorySync;
                let expected = if committed {
                    Some("candidate")
                } else {
                    has_previous.then_some("previous confirmed")
                };
                let candidate_path = dir.path().join("7.json");
                let candidate_bytes = std::fs::read(&candidate_path).unwrap();
                let journal_bytes = std::fs::read(journal_path(dir.path())).unwrap();
                let journal: serde_json::Value = serde_json::from_slice(&journal_bytes).unwrap();
                assert_eq!(
                    journal["phase"],
                    if committed { "Committed" } else { "Prepared" }
                );
                for _ in 0..2 {
                    assert_eq!(
                        restart_snapshot(dir.path()).as_deref(),
                        expected,
                        "has_previous={has_previous}, {primary:?}/{compensation:?}"
                    );
                    assert_eq!(std::fs::read(&candidate_path).unwrap(), candidate_bytes);
                    assert_eq!(
                        std::fs::read(journal_path(dir.path())).unwrap(),
                        journal_bytes
                    );
                    assert!(NativeRoomStore::new(dir.path())
                        .scan()
                        .unwrap()
                        .reserved_ids
                        .contains(&7));
                }
                assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
                // 双重故障仅消耗本次计划；后续明确保存仍沿用磁盘确认的前态。
                store.save(&record(7, "next confirmed")).unwrap();
                assert_eq!(
                    restart_snapshot(dir.path()).as_deref(),
                    Some("next confirmed")
                );
            }
        }
    }
}

#[test]
fn replacing_a_double_fault_plan_with_a_single_fault_does_not_leak_compensation() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    store.save(&record(7, "confirmed")).unwrap();
    store.fail_next_write_with_compensation(
        StoreFault::DecisionDirectorySync,
        StoreFault::PrepareWrite,
    );
    store.fail_next_write(StoreFault::DecisionDirectorySync);
    let error = store.save_durable(&record(7, "candidate")).unwrap_err();
    assert!(error.replacement_visible);
    assert!(
        !error.decision_uncertain,
        "single-fault compensation must remain healthy: {error}"
    );
    assert_eq!(restart_snapshot(dir.path()).as_deref(), Some("confirmed"));
}
