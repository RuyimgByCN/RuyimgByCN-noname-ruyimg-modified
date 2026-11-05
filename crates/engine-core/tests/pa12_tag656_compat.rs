use content_runtime::{ContentLock, InstallOptions, PackageStore};
use content_schema::ContentPackage;
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, ManifestRejectionReason,
    PlayerId,
};
use engine_core::{
    snapshot::{Snapshot, SnapshotError},
    Engine,
};

const OLD_ONLYOL_010: &[u8] =
    include_bytes!("../../../content/original/packages/onlyOL.npack.json");
const OLD_ONLYOL: &[u8] =
    include_bytes!("../../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/onlyOL-0.1.4.npack.json");
const OLD_ONLYOL_012: &[u8] =
    include_bytes!("../../../docs/operations/evidence/2026-10-24-up37-n01-zhuhuan/onlyOL-0.1.2.npack.json");
const NEW_ONLYOL: &[u8] =
    include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");
const SHIJI_010: &[u8] =
    include_bytes!("../../../content/original/ported/shiji/archive/shiji-0.1.0.npack.json");
const SHIJI_024: &[u8] =
    include_bytes!("../../../content/original/ported/shiji/archive/shiji-0.2.4.npack.json");
const SHIJI_025: &[u8] =
    include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");
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
            assert!(Engine::new()
                .load_package(&package)
                .unwrap_err()
                .to_string()
                .contains("ambiguous effect 656"));
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
    assert_eq!(
        (new.manifest.version.as_str(), new.manifest.rule_vm_abi_version),
        ("0.1.5", 758)
    );
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
        assert!(
            matches!(result, Err(SnapshotError::ContentCompatibility(_))),
            "ABI {abi}"
        );
    }
}

fn start_locked_match(store: &PackageStore, lock: &ContentLock, seed: u64) -> Engine {
    let mut engine = Engine::from_content_lock(store, lock).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(1),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    engine
}

#[test]
fn shiji_010_024_025_content_lock_upgrade_audit_and_snapshot_matrix() {
    let p010 = ContentPackage::from_json_bytes(SHIJI_010).unwrap();
    let p024 = ContentPackage::from_json_bytes(SHIJI_024).unwrap();
    let p025 = ContentPackage::from_json_bytes(SHIJI_025).unwrap();
    assert_eq!(
        (p010.manifest.version.as_str(), p010.manifest.rule_vm_abi_version),
        ("0.1.0", 502)
    );
    assert_eq!(
        (p024.manifest.version.as_str(), p024.manifest.rule_vm_abi_version),
        ("0.2.4", 756)
    );
    assert_eq!(
        (p025.manifest.version.as_str(), p025.manifest.rule_vm_abi_version),
        ("0.2.5", 756)
    );
    assert_eq!(p010.character_index.len(), 48);
    assert_eq!(p024.character_index.len(), 48);
    assert_eq!(p025.character_index.len(), 48);
    assert_eq!(p010.skill_index.len(), 180);
    assert_eq!(p024.skill_index.len(), 180);
    assert_eq!(p025.skill_index.len(), 180);
    assert_eq!(p010.modules.len(), 173);
    assert_eq!(p024.modules.len(), 173);
    assert_eq!(p025.modules.len(), 173);
    assert_ne!(p010.manifest.logic_hash, p024.manifest.logic_hash);
    assert_ne!(p024.manifest.logic_hash, p025.manifest.logic_hash);

    // 0.2.4 -> 0.2.5 delta audit: only pinghe_damage_before trigger filter changed.
    let diff_024_025: Vec<&str> = p024
        .modules
        .iter()
        .zip(&p025.modules)
        .filter(|(a, b)| a.logic_hash != b.logic_hash)
        .map(|(a, _)| a.name.as_str())
        .collect();
    assert!(
        diff_024_025.is_empty(),
        "0.2.4 -> 0.2.5 keeps module bytecode identical and only tightens trigger_index filter"
    );
    let trig_024 = p024
        .trigger_index
        .iter()
        .find(|t| t.module == "pinghe_damage_before")
        .unwrap();
    let trig_025 = p025
        .trigger_index
        .iter()
        .find(|t| t.module == "pinghe_damage_before")
        .unwrap();
    assert_eq!(trig_024.filter, "owner_is_target");
    assert_eq!(trig_025.filter, "owner_is_target_other_source_alive");

    let shiji_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/original/ported/shiji");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let opts = InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };
    store
        .install_with_assets(SHIJI_010, None, &opts, &shiji_root)
        .unwrap();
    let lock_010 = store.current_lock().unwrap();
    store
        .install_with_assets(SHIJI_024, None, &opts, &shiji_root)
        .unwrap();
    let lock_024 = store.current_lock().unwrap();
    store
        .install_with_assets(SHIJI_025, None, &opts, &shiji_root)
        .unwrap();
    let lock_025 = store.current_lock().unwrap();

    assert_ne!(lock_010.lock_hash, lock_024.lock_hash);
    assert_ne!(lock_024.lock_hash, lock_025.lock_hash);
    assert_ne!(lock_010.lock_hash, lock_025.lock_hash);

    // Exact frozen copies survive upgrades in PackageStore.
    assert_eq!(store.load_locked_packages(&lock_010).unwrap(), vec![p010]);
    assert_eq!(store.load_locked_packages(&lock_024).unwrap(), vec![p024]);
    assert_eq!(store.load_locked_packages(&lock_025).unwrap(), vec![p025]);

    let eng_010 = start_locked_match(&store, &lock_010, 20261105);
    let eng_024 = start_locked_match(&store, &lock_024, 20261105);
    let eng_025 = start_locked_match(&store, &lock_025, 20261105);

    let snap_010 = eng_010.snapshot();
    let snap_024 = eng_024.snapshot();
    let snap_025 = eng_025.snapshot();
    let man_010 = eng_010.manifest().unwrap().clone();
    let man_024 = eng_024.manifest().unwrap().clone();
    let man_025 = eng_025.manifest().unwrap().clone();

    assert_ne!(man_010.content_lock_hash, man_024.content_lock_hash);
    assert_ne!(man_024.content_lock_hash, man_025.content_lock_hash);
    assert_ne!(man_010.manifest_hash(), man_024.manifest_hash());
    assert_ne!(man_024.manifest_hash(), man_025.manifest_hash());

    // Exact-lock recovery succeeds for each version even after 0.2.5 is active.
    for (lock, snap, man, expected_hash) in [
        (&lock_010, &snap_010, &man_010, eng_010.state_hash()),
        (&lock_024, &snap_024, &man_024, eng_024.state_hash()),
        (&lock_025, &snap_025, &man_025, eng_025.state_hash()),
    ] {
        let restored_state = snap.restore_with_manifest(man.manifest_hash()).unwrap();
        assert_eq!(
            engine_core::hash::canonical_state_hash(&restored_state),
            expected_hash
        );
        let mut fresh = Engine::from_content_lock(&store, lock).unwrap();
        fresh
            .restore_snapshot_with_manifest(snap, man.clone())
            .unwrap();
        assert_eq!(fresh.state_hash(), expected_hash);
    }

    // Cross-version content lock restoration fails closed across all 6 mismatched pairs.
    for (snap, wrong_man) in [
        (&snap_010, &man_024),
        (&snap_010, &man_025),
        (&snap_024, &man_010),
        (&snap_024, &man_025),
        (&snap_025, &man_010),
        (&snap_025, &man_024),
    ] {
        assert!(matches!(
            snap.restore_with_manifest(wrong_man.manifest_hash()),
            Err(SnapshotError::ManifestMismatch { .. })
        ));
        let mut fresh = Engine::new();
        assert_eq!(
            fresh.restore_snapshot_with_manifest(snap, wrong_man.clone()),
            Err(ManifestRejectionReason::SnapshotInconsistent)
        );
    }
}

#[test]
fn onlyol_010_to_015_content_lock_matrix_and_ambiguous_rejection() {
    let p010 = ContentPackage::from_json_bytes(OLD_ONLYOL_010).unwrap();
    let p012 = ContentPackage::from_json_bytes(OLD_ONLYOL_012).unwrap();
    let p014 = ContentPackage::from_json_bytes(OLD_ONLYOL).unwrap();
    let p015 = ContentPackage::from_json_bytes(NEW_ONLYOL).unwrap();

    assert_eq!(
        (
            p010.manifest.version.as_str(),
            p010.manifest.rule_vm_abi_version,
            p010.character_index.len(),
            p010.skill_index.len(),
            p010.modules.len()
        ),
        ("0.1.0", 688, 84, 190, 30)
    );
    assert_eq!(
        (
            p012.manifest.version.as_str(),
            p012.manifest.rule_vm_abi_version,
            p012.character_index.len(),
            p012.skill_index.len(),
            p012.modules.len()
        ),
        ("0.1.2", 754, 85, 192, 34)
    );
    assert_eq!(
        (
            p014.manifest.version.as_str(),
            p014.manifest.rule_vm_abi_version,
            p014.character_index.len(),
            p014.skill_index.len(),
            p014.modules.len()
        ),
        ("0.1.4", 757, 86, 193, 36)
    );
    assert_eq!(
        (
            p015.manifest.version.as_str(),
            p015.manifest.rule_vm_abi_version,
            p015.character_index.len(),
            p015.skill_index.len(),
            p015.modules.len()
        ),
        ("0.1.5", 758, 86, 193, 36)
    );

    // Per-version delta audit (UP37-I01 / UP37-N01): existing 84 characters and 190 skills
    // in 0.1.0 preserve their exact relative order across 0.1.2, 0.1.4, and 0.1.5.
    let chars_010: Vec<&str> = p010
        .character_index
        .iter()
        .map(|c| c.name_key.as_str())
        .collect();
    let skills_010: Vec<&str> = p010.skill_index.iter().map(|s| s.name.as_str()).collect();
    for pkg in [&p012, &p014, &p015] {
        let kept_chars: Vec<&str> = pkg
            .character_index
            .iter()
            .map(|c| c.name_key.as_str())
            .filter(|name| chars_010.contains(name))
            .collect();
        assert_eq!(kept_chars, chars_010);
        let kept_skills: Vec<&str> = pkg
            .skill_index
            .iter()
            .map(|s| s.name.as_str())
            .filter(|name| skills_010.contains(name))
            .collect();
        assert_eq!(kept_skills, skills_010);
    }

    let onlyol_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/onlyOL");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let opts = InstallOptions {
        activate: false,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };

    // 0.1.0 installs cleanly into PackageStore.
    let res_010 = store
        .install_with_assets(OLD_ONLYOL_010, None, &opts, &onlyol_root)
        .unwrap();
    assert_eq!(res_010.meta.version, "0.1.0");

    // 0.1.2 and 0.1.4 fail closed at PackageStore install due to retired ambiguous tag 656.
    for bad_bytes in [OLD_ONLYOL_012, OLD_ONLYOL] {
        let err = store
            .install_with_assets(bad_bytes, None, &opts, &onlyol_root)
            .unwrap_err()
            .to_string();
        assert!(err.contains("ambiguous effect 656"), "{err}");
    }
    assert_eq!(store.list_installed().unwrap().len(), 1);

    // 0.1.5 installs cleanly alongside 0.1.0 without overwriting the frozen 0.1.0 copy.
    let res_015 = store
        .install_with_assets(NEW_ONLYOL, None, &opts, &onlyol_root)
        .unwrap();
    assert_eq!(res_015.meta.version, "0.1.5");
    assert_ne!(res_010.meta.logic_hash, res_015.meta.logic_hash);
    assert_ne!(res_010.meta.content_hash, res_015.meta.content_hash);
    let installed: Vec<String> = store
        .list_installed()
        .unwrap()
        .into_iter()
        .map(|m| format!("{}/{}@{}", m.namespace, m.package_id, m.version))
        .collect();
    assert_eq!(
        installed,
        vec![
            "official/onlyOL@0.1.0".to_string(),
            "official/onlyOL@0.1.5".to_string()
        ]
    );

    // Engine snapshot & manifest binding between 0.1.0 and 0.1.5:
    let mut eng_010 = Engine::new();
    eng_010.load_package(&p010).unwrap();
    assert!(matches!(
        eng_010.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20261105,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let mut eng_015 = Engine::new();
    eng_015.load_package(&p015).unwrap();
    assert!(matches!(
        eng_015.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20261105,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let snap_010 = eng_010.snapshot();
    let snap_015 = eng_015.snapshot();
    assert_ne!(
        snap_010.state_hash, snap_015.state_hash,
        "0.1.0 and 0.1.5 frozen package descriptors produce distinct canonical state hashes"
    );
    assert_eq!(
        engine_core::hash::canonical_state_hash(&snap_010.restore().unwrap()),
        eng_010.state_hash()
    );
    assert_eq!(
        engine_core::hash::canonical_state_hash(&snap_015.restore().unwrap()),
        eng_015.state_hash()
    );

    // Tampering a 0.1.0 snapshot's frozen package descriptor with 0.1.5's state_hash fails closed.
    let mut mismatched = snap_010.clone();
    mismatched.state_hash = snap_015.state_hash;
    assert_eq!(mismatched.restore(), Err(SnapshotError::HashMismatch));
}

#[test]
fn all_ported_and_original_packages_validate_and_preserve_audit_invariants() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/original");
    let mut ported_count = 0usize;
    for entry in std::fs::read_dir(root.join("ported")).unwrap() {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }
        let name = dir.file_name().unwrap().to_str().unwrap();
        let npack = dir.join(format!("{name}.npack.json"));
        if !npack.exists() {
            continue;
        }
        let pkg = ContentPackage::from_json_bytes(&std::fs::read(&npack).unwrap()).unwrap();
        pkg.validate().unwrap();
        let char_ids: std::collections::BTreeSet<u32> =
            pkg.character_index.iter().map(|c| c.id).collect();
        assert_eq!(char_ids.len(), pkg.character_index.len(), "{name} char ids");
        let skill_ids: std::collections::BTreeSet<u32> =
            pkg.skill_index.iter().map(|s| s.skill_def_id).collect();
        assert_eq!(skill_ids.len(), pkg.skill_index.len(), "{name} skill ids");
        ported_count += 1;
    }
    assert_eq!(ported_count, 31, "expected all 31 ported packages");

    // content/original/packages/ holds pre-upgrade 0.1.0 snapshots for offline and onlyOL.
    let offline_pkg = ContentPackage::from_json_bytes(
        &std::fs::read(root.join("packages/offline.npack.json")).unwrap(),
    )
    .unwrap();
    let offline_ported = ContentPackage::from_json_bytes(
        &std::fs::read(root.join("ported/offline/offline.npack.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(offline_pkg, offline_ported);
}
