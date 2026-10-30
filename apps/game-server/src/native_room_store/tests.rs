use super::*;

pub(super) fn record(room_id: u64, snapshot: &str) -> StoredRoom {
    StoredRoom {
        room_id,
        password_hash: "saved-password-hash".into(),
        checksum: String::new(),
        state: NativeRoomState {
            snapshot_hex: snapshot.into(),
            max_seats: 2,
            started: true,
            next_token: 1,
            next_dedup: 1,
            pack_refs: Vec::new(),
            frozen_packages: Some(Vec::new()),
            members: Vec::new(),
            command_receipts: Vec::new(),
            timeout: None,
            wal: None,
        },
    }
}

#[test]
fn stale_legacy_temp_never_blocks_new_save() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    std::fs::create_dir(dir.path().join("7.json.tmp")).unwrap();
    store.save(&record(7, "first")).unwrap();
    assert_eq!(store.load_all().unwrap()[0].state.snapshot_hex, "first");
    assert!(dir.path().join("7.json.tmp").is_dir());
}

#[test]
fn filename_binding_rejects_record_from_other_room() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    store.save(&record(7, "first")).unwrap();
    std::fs::rename(dir.path().join("7.json"), dir.path().join("8.json")).unwrap();
    assert!(store.load_all().is_err());
}

#[test]
fn pre_replace_faults_preserve_committed_bytes_and_clean_only_own_temp() {
    for fault in [
        StoreFault::PartialWrite,
        StoreFault::FileSync,
        StoreFault::Rename,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let store = NativeRoomStore::new(dir.path());
        store.save(&record(7, "confirmed")).unwrap();
        let before = std::fs::read(dir.path().join("7.json")).unwrap();
        std::fs::write(dir.path().join("7.json.tmp"), b"old incomplete evidence").unwrap();
        store.clone().fail_next_write(fault);
        let error = store.save_durable(&record(7, "unconfirmed")).unwrap_err();
        assert!(!error.replacement_visible, "{fault:?}: {error}");
        assert_eq!(std::fs::read(dir.path().join("7.json")).unwrap(), before);
        assert_eq!(
            std::fs::read(dir.path().join("7.json.tmp")).unwrap(),
            b"old incomplete evidence"
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
        assert_eq!(store.load_all().unwrap()[0].state.snapshot_hex, "confirmed");
        store.save_durable(&record(7, "retry")).unwrap();
        assert_eq!(store.load_all().unwrap()[0].state.snapshot_hex, "retry");
    }
}

#[test]
fn post_replace_failure_restarts_from_confirmed_record_without_repair() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    store.save(&record(7, "confirmed")).unwrap();
    store.fail_next_write(StoreFault::DirectorySync);
    let error = store.save_durable(&record(7, "unconfirmed")).unwrap_err();
    assert!(error.replacement_visible, "{error}");
    assert_eq!(
        NativeRoomStore::new(dir.path()).load_all().unwrap()[0]
            .state
            .snapshot_hex,
        "confirmed"
    );
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
    store.save_durable(&record(7, "confirmed")).unwrap();
    assert_eq!(store.load_all().unwrap()[0].state.snapshot_hex, "confirmed");
}

#[test]
fn failed_first_write_never_becomes_a_committed_record() {
    for fault in [
        StoreFault::PartialWrite,
        StoreFault::FileSync,
        StoreFault::Rename,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let store = NativeRoomStore::new(dir.path());
        store.fail_next_write(fault);
        assert!(
            !store
                .save_durable(&record(7, "candidate"))
                .unwrap_err()
                .replacement_visible
        );
        assert!(store.load_all().unwrap().is_empty());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}

#[test]
fn scan_isolates_bad_records_and_reserves_all_related_room_ids() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    store.save(&record(7, "confirmed")).unwrap();
    std::fs::write(dir.path().join("9.json"), b"{truncated").unwrap();
    std::fs::write(
        dir.path().join("10.json"),
        serde_json::to_vec(&record(11, "wrong identity")).unwrap(),
    )
    .unwrap();
    std::fs::write(dir.path().join("12.json.tmp"), b"{uncommitted").unwrap();
    let tmp_name = format!("13.json.{}.tmp", uuid::Uuid::new_v4());
    std::fs::write(
        dir.path().join(tmp_name),
        serde_json::to_vec(&record(13, "complete but not committed")).unwrap(),
    )
    .unwrap();
    for name in [
        "123notroom.json",
        "x.json",
        "200.json.not-a-uuid.tmp",
        "300.json.something",
        "note.txt",
    ] {
        std::fs::write(dir.path().join(name), b"unknown").unwrap();
    }
    let scan = store.scan().unwrap();
    assert_eq!(scan.records.len(), 1);
    assert_eq!(scan.records[0].room_id, 7);
    assert_eq!(
        scan.reserved_ids,
        [7, 9, 10, 11, 12, 13].into_iter().collect()
    );
    assert_eq!(scan.issues.len(), 4);
    assert_eq!(
        scan.issues
            .iter()
            .filter(|issue| issue.uncommitted_temp)
            .count(),
        2
    );
    assert!(
        store.load_all().is_err(),
        "strict API keeps corruption visible"
    );
    assert_eq!(
        std::fs::read(dir.path().join("9.json")).unwrap(),
        b"{truncated"
    );
}

#[test]
fn noncanonical_duplicate_id_does_not_silently_override_valid_record() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    store.save(&record(7, "confirmed")).unwrap();
    std::fs::write(
        dir.path().join("007.json"),
        serde_json::to_vec(&record(7, "alias")).unwrap(),
    )
    .unwrap();
    let scan = store.scan().unwrap();
    assert_eq!(scan.records.len(), 1);
    assert_eq!(scan.records[0].state.snapshot_hex, "confirmed");
    assert_eq!(scan.issues.len(), 1);
    assert_eq!(scan.reserved_ids, [7].into_iter().collect());
    assert!(store.by_id().is_err());
}

#[test]
fn nested_new_directory_and_real_directory_creation_failure() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path().join("new/nested/rooms"));
    store.save_durable(&record(7, "confirmed")).unwrap();
    assert_eq!(store.load_all().unwrap().len(), 1);
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"preserved").unwrap();
    let failed = NativeRoomStore::new(blocker.join("rooms"));
    assert!(
        !failed
            .save_durable(&record(7, "candidate"))
            .unwrap_err()
            .replacement_visible
    );
    assert_eq!(std::fs::read(blocker).unwrap(), b"preserved");
}

#[test]
fn invalid_checksum_and_non_file_records_are_isolated_without_deletion() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    store.save(&record(7, "confirmed")).unwrap();
    let mut damaged = record(9, "damaged");
    damaged.checksum = "wrong".into();
    std::fs::write(
        dir.path().join("9.json"),
        serde_json::to_vec(&damaged).unwrap(),
    )
    .unwrap();
    std::fs::create_dir(dir.path().join("10.json")).unwrap();
    let scan = store.scan().unwrap();
    assert_eq!(scan.records.len(), 1);
    assert_eq!(scan.issues.len(), 2);
    assert!(scan
        .issues
        .iter()
        .any(|issue| issue.error.contains("校验和")));
    assert!(dir.path().join("10.json").is_dir());
}

#[test]
fn maximum_id_is_reserved_and_zero_id_is_never_written() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    let temp = format!("{}.json.tmp", u64::MAX);
    std::fs::write(dir.path().join(temp), b"partial").unwrap();
    assert!(store.scan().unwrap().reserved_ids.contains(&u64::MAX));
    assert!(
        !store
            .save_durable(&record(0, "invalid"))
            .unwrap_err()
            .replacement_visible
    );
    assert!(!dir.path().join("0.json").exists());
}

#[cfg(windows)]
#[test]
fn windows_write_through_replace_supports_unicode_and_long_paths() {
    let dir = tempfile::tempdir().unwrap();
    let mut root = dir.path().join("房间😀");
    for _ in 0..6 {
        root = root.join("long-room-directory-name-for-original-content-lock");
    }
    let store = NativeRoomStore::new(&root);
    store.save_durable(&record(7, "first")).unwrap();
    store.save_durable(&record(7, "replaced")).unwrap();
    assert_eq!(store.load_all().unwrap()[0].state.snapshot_hex, "replaced");
    assert_eq!(std::fs::read_dir(root).unwrap().count(), 2);
}

#[cfg(windows)]
#[test]
fn windows_invalid_or_blocked_destination_preserves_source() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    std::fs::write(&source, b"original").unwrap();
    let blocked = dir.path().join("destination-directory");
    std::fs::create_dir(&blocked).unwrap();
    for destination in [blocked, dir.path().join("bad\0name")] {
        let mut visible = false;
        assert!(replace_file(&source, &destination, &mut visible).is_err());
        assert!(!visible);
        assert_eq!(std::fs::read(&source).unwrap(), b"original");
    }
}

#[test]
fn multi_room_save_and_load_cost_is_measured_and_bounded() {
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    let rooms = 16u64;
    let rounds = 20u64;
    for room in 1..=rooms {
        store.save(&record(room, "seed")).unwrap();
    }
    let mut save_samples = Vec::new();
    for round in 0..rounds {
        for room in 1..=rooms {
            let started = std::time::Instant::now();
            store
                .save(&record(room, &format!("snapshot-{room}-{round}")))
                .unwrap();
            save_samples.push(started.elapsed());
        }
    }
    save_samples.sort_unstable();
    let median = save_samples[save_samples.len() / 2];
    let p95 = save_samples[(save_samples.len() * 95 / 100).min(save_samples.len() - 1)];
    let load_started = std::time::Instant::now();
    let loaded = store.load_all().unwrap();
    let load_elapsed = load_started.elapsed();
    assert_eq!(loaded.len() as u64, rooms);
    assert!(
        p95 < std::time::Duration::from_millis(500),
        "p95 save {p95:?} 超出测试上限"
    );
    eprintln!(
        "multi-room save median={median:?} p95={p95:?}, load {} rooms={load_elapsed:?}",
        loaded.len()
    );
}
