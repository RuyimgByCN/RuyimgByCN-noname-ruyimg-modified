//! J2 / UP37-I04 / UP37-I01 / UP37-N01 / UP37-R10:
//! Published 0.1.x / 0.2.x package versions, exact content locks, native `.ngsa`
//! locked-content restore & cross-version rejection, and native BoundedPlanner
//! valuation / RNG isolation.

use content_native_loader::{
    bridge_generals_with_known, bridge_vm_skill_decls, prepare_locked_content_with_bridge,
    restore_locked_engine, start_locked_engine, validate_and_freeze_raw_package, LockedStartMode,
};
use content_runtime::{ContentLock, InstallOptions, PackageStore};
use content_schema::ContentPackage;
use engine_native::{
    ai::{BoundedPlanner, ContentTuning},
    archive::{decode_archive, encode_archive},
    content::extension::{ExtensionDef, ExtensionPack},
    ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand, Phase, SnapshotError,
};
use std::collections::BTreeMap;

#[test]
fn shiji_old_lock_retains_published_bytes_and_new_full_package_rejects_missing_native_declarations()
{
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/shiji");
    let v010_bytes = std::fs::read(root.join("archive/shiji-0.1.0.npack.json")).unwrap();
    let old_bytes = std::fs::read(root.join("archive/shiji-0.2.4.npack.json")).unwrap();
    let new_bytes = std::fs::read(root.join("shiji.npack.json")).unwrap();
    let v010 = ContentPackage::from_json_bytes(&v010_bytes).unwrap();
    let old = ContentPackage::from_json_bytes(&old_bytes).unwrap();
    let new = ContentPackage::from_json_bytes(&new_bytes).unwrap();
    assert_eq!(v010.manifest.version, "0.1.0");
    assert_eq!(old.manifest.version, "0.2.4");
    assert_eq!(new.manifest.version, "0.2.5");
    assert_eq!(v010.manifest.rule_vm_abi_version, 502);
    assert_eq!(old.manifest.rule_vm_abi_version, 756);
    assert_eq!(new.manifest.rule_vm_abi_version, 756);
    assert_eq!(old.asset_manifest, new.asset_manifest);
    assert_eq!(old.asset_paths, new.asset_paths);
    assert_eq!(old.manifest.asset_hash, new.manifest.asset_hash);
    assert_eq!(v010.modules.len(), old.modules.len());
    assert_eq!(old.modules.len(), new.modules.len());
    assert_eq!(old.trigger_index.len(), new.trigger_index.len());
    assert_eq!(old.active_index.len(), new.active_index.len());
    assert_ne!(v010.manifest.logic_hash, old.manifest.logic_hash);
    assert_ne!(old.manifest.logic_hash, new.manifest.logic_hash);
    let old_pinghe = old
        .trigger_index
        .iter()
        .find(|entry| entry.module == "pinghe_damage_before")
        .unwrap();
    let new_pinghe = new
        .trigger_index
        .iter()
        .find(|entry| entry.module == "pinghe_damage_before")
        .unwrap();
    assert_eq!(old_pinghe.filter, "owner_is_target");
    assert_eq!(new_pinghe.filter, "owner_is_target_other_source_alive");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let options = InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };
    store
        .install_with_assets(&v010_bytes, None, &options, &root)
        .unwrap();
    let v010_lock: ContentLock =
        serde_json::from_slice(&serde_json::to_vec(&store.current_lock().unwrap()).unwrap())
            .unwrap();
    store
        .install_with_assets(&old_bytes, None, &options, &root)
        .unwrap();
    let old_lock: ContentLock =
        serde_json::from_slice(&serde_json::to_vec(&store.current_lock().unwrap()).unwrap())
            .unwrap();
    store
        .install_with_assets(&new_bytes, None, &options, &root)
        .unwrap();
    let new_lock = store.current_lock().unwrap();
    assert_ne!(v010_lock.lock_hash, old_lock.lock_hash);
    assert_ne!(old_lock.lock_hash, new_lock.lock_hash);
    assert_eq!(store.load_locked_packages(&v010_lock).unwrap(), vec![v010]);
    let restored_old = store.load_locked_packages(&old_lock).unwrap();
    assert_eq!(restored_old.len(), 1);
    assert_eq!(
        restored_old[0], old,
        "an exact old lock never silently selects the new package"
    );
    assert_eq!(
        store.load_locked_packages(&new_lock).unwrap(),
        vec![new.clone()]
    );
    store
        .activate_version("official", "shiji", "0.1.0")
        .unwrap();
    assert_eq!(store.current_lock().unwrap().lock_hash, v010_lock.lock_hash);
    store
        .activate_version("official", "shiji", "0.2.4")
        .unwrap();
    assert_eq!(store.current_lock().unwrap().lock_hash, old_lock.lock_hash);
    assert_eq!(
        store.load_locked_packages(&new_lock).unwrap(),
        vec![new.clone()],
        "rollback preserves the newer frozen copy too"
    );
    let error = content_native_loader::bridge_vm_skill_decls(&new)
        .unwrap_err()
        .to_string();
    // A2 D02（R05 准备阶段真实化）：`phase_prepare` 已可装（Phase::Prepare
    // 门），完整包的拒装锚改为首个仍未映射的 `game_started`。
    assert!(
        error.contains("game_started"),
        "the complete package remains explicitly unsupported: {error}"
    );
}

#[test]
fn onlyol_010_to_015_package_store_and_native_locked_content_matrix() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let onlyol_root = manifest_dir.join("../../content/original/ported/onlyOL");
    let v010_bytes =
        std::fs::read(manifest_dir.join("../../content/original/packages/onlyOL.npack.json"))
            .unwrap();
    let v012_bytes = std::fs::read(
        manifest_dir
            .join("../../docs/operations/evidence/2026-10-24-up37-n01-zhuhuan/onlyOL-0.1.2.npack.json"),
    )
    .unwrap();
    let v014_bytes = std::fs::read(
        manifest_dir
            .join("../../docs/operations/evidence/2026-10-27-pa12-tag656-fix/onlyOL-0.1.4.npack.json"),
    )
    .unwrap();
    let v015_bytes = std::fs::read(onlyol_root.join("onlyOL.npack.json")).unwrap();

    // 0.1.2 and 0.1.4 fail closed across all native validation & loader entry points.
    for bad in [&v012_bytes, &v014_bytes] {
        let pkg = ContentPackage::from_json_bytes(bad).unwrap();
        assert!(pkg.validate().unwrap_err().contains("ambiguous effect 656"));
        assert!(bridge_vm_skill_decls(&pkg)
            .unwrap_err()
            .to_string()
            .contains("ambiguous effect 656"));
        let raw_str = String::from_utf8(bad.clone()).unwrap();
        assert!(validate_and_freeze_raw_package(raw_str, BTreeMap::new())
            .unwrap_err()
            .contains("ambiguous effect 656"));
    }

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path()).unwrap();
    let opts = InstallOptions {
        activate: false,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    };
    let r010 = store
        .install_with_assets(&v010_bytes, None, &opts, &onlyol_root)
        .unwrap();
    for bad in [&v012_bytes, &v014_bytes] {
        assert!(store
            .install_with_assets(bad, None, &opts, &onlyol_root)
            .unwrap_err()
            .to_string()
            .contains("ambiguous effect 656"));
    }
    let r015 = store
        .install_with_assets(&v015_bytes, None, &opts, &onlyol_root)
        .unwrap();
    assert_eq!(r010.meta.version, "0.1.0");
    assert_eq!(r015.meta.version, "0.1.5");
    assert_ne!(r010.meta.logic_hash, r015.meta.logic_hash);

    // Native VerifiedLockedContent 0.1.0 vs 0.1.5 (onlyOL N01 slice) and 0.2.4 vs 0.2.5 (shiji R03 slice):
    let fenli_src = include_str!("../../../content/original/ported/onlyOL/ol_fenli.skill");
    let pingkou_src = include_str!("../../../content/original/ported/onlyOL/ol_pingkou.skill");
    let danshou_src = include_str!("../../../content/original/ported/onlyOL/ol_jie_danshou.skill");
    let pinghe_src = include_str!("../../../content/original/ported/shiji/pinghe.skill");

    let build_verified = |pkg_id: &str, version: &str, source: &str| {
        let mut pkg = content_compiler::compile_source_named(pkg_id, source).unwrap();
        pkg.manifest.namespace = "official".into();
        pkg.manifest.package_id = pkg_id.into();
        pkg.manifest.version = version.into();
        pkg.manifest.logic_hash = pkg.compute_logic_hash();
        let raw = String::from_utf8(pkg.to_json_bytes().unwrap()).unwrap();
        let frozen = validate_and_freeze_raw_package(raw, BTreeMap::new()).unwrap();
        prepare_locked_content_with_bridge(vec![frozen]).unwrap()
    };

    let onlyol_v010 = build_verified("onlyOL", "0.1.0", &format!("{fenli_src}\n{pingkou_src}"));
    let onlyol_v015 = build_verified(
        "onlyOL",
        "0.1.5",
        &format!("{fenli_src}\n{pingkou_src}\n{danshou_src}"),
    );
    let shiji_v024 = build_verified(
        "shiji",
        "0.2.4",
        &pinghe_src.replace("owner_is_target_other_source_alive", "owner_is_target"),
    );
    let shiji_v025 = build_verified("shiji", "0.2.5", pinghe_src);

    assert_ne!(onlyol_v010.lock().lock_hash, onlyol_v015.lock().lock_hash);
    assert_ne!(shiji_v024.lock().lock_hash, shiji_v025.lock().lock_hash);

    for (v_old, v_new) in [(&onlyol_v010, &onlyol_v015), (&shiji_v024, &shiji_v025)] {
        let eng_old = start_locked_engine(20261105, 4, LockedStartMode::Prototype, v_old).unwrap();
        let eng_new = start_locked_engine(20261105, 4, LockedStartMode::Prototype, v_new).unwrap();
        let bytes_old = encode_archive(&eng_old.save());
        let bytes_new = encode_archive(&eng_new.save());
        let snap_old = decode_archive(&bytes_old).unwrap();
        let snap_new = decode_archive(&bytes_new).unwrap();

        // Exact locked content restores with identical state_hash, core, resolution, and rotated auth_generation.
        assert_eq!(encode_archive(&snap_old), bytes_old);
        let restored_old = restore_locked_engine(&snap_old, v_old).unwrap();
        assert_eq!(restored_old.state_hash(), eng_old.state_hash());
        assert_eq!(restored_old.save().core, snap_old.core);
        assert_eq!(restored_old.save().resolution, snap_old.resolution);
        assert_eq!(
            restored_old.save().auth_generation,
            snap_old.auth_generation + 1
        );

        assert_eq!(encode_archive(&snap_new), bytes_new);
        let restored_new = restore_locked_engine(&snap_new, v_new).unwrap();
        assert_eq!(restored_new.state_hash(), eng_new.state_hash());
        assert_eq!(restored_new.save().core, snap_new.core);
        assert_eq!(restored_new.save().resolution, snap_new.resolution);
        assert_eq!(
            restored_new.save().auth_generation,
            snap_new.auth_generation + 1
        );

        // Cross-version restore (0.1.0 <-> 0.1.5 or 0.2.4 <-> 0.2.5) and standard catalog restore fail closed.
        assert!(matches!(
            restore_locked_engine(&snap_old, v_new),
            Err(SnapshotError::RuleVersionMismatch { .. })
        ));
        assert!(matches!(
            restore_locked_engine(&snap_new, v_old),
            Err(SnapshotError::RuleVersionMismatch { .. })
        ));
        assert!(matches!(
            Engine::restore(&snap_old),
            Err(SnapshotError::RuleVersionMismatch { .. })
        ));
    }
}

#[test]
fn up37_n01_and_r10_native_bounded_planner_tuning_and_rng_isolation() {
    let source = format!(
        "{}\n{}\n{}\n{}",
        include_str!("../../../content/original/ported/onlyOL/ol_jie_danshou.skill"),
        include_str!("../../../content/original/ported/onlyOL/ol_fenli.skill"),
        include_str!("../../../content/original/ported/onlyOL/ol_pingkou.skill"),
        include_str!("../../../content/original/ported/shiji/pinghe.skill"),
    );
    let pkg = content_compiler::compile_source_named("up37-n01-r10-native-ai", &source).unwrap();
    let bridged = bridge_vm_skill_decls(&pkg).unwrap();
    let known: std::collections::BTreeSet<String> =
        bridged.decls.iter().map(|d| d.name.clone()).collect();
    let generals = bridge_generals_with_known(
        &serde_json::json!({
            "pack": {"name": "up37-n01-r10-native-ai", "version": 1},
            "generals": [{
                "name": "up37-ai-hero",
                "hp": 4,
                "gender": "male",
                "faction": "wu",
                "skills": bridged.decls.iter().map(|d| serde_json::json!({"name": d.name})).collect::<Vec<_>>()
            }]
        })
        .to_string(),
        &known,
    )
    .unwrap();
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "up37-n01-r10-native-ai",
            version: 1,
            description: "UP37 N01/R10 native AI",
        },
        generals: generals.generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: bridged.decls,
        init_hook: Default::default(),
        enabled: true,
    };

    let mut engine = Engine::new_with_extensions(20261105, 4, &[&pack]).unwrap();
    engine.assign_general_named(0, "up37-ai-hero").unwrap();
    for _ in 0..20 {
        if engine.turn().current == 0
            && engine.turn().phase == Phase::Play
            && engine.open_choice().is_none()
        {
            break;
        }
        if let Some(c) = engine.open_choice().cloned() {
            let sel = if c.spec.options.contains(&O::Pass) {
                vec![O::Pass]
            } else if c.spec.allow_cancel {
                vec![]
            } else {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            };
            let rev = engine.revision();
            engine.submit(GameCommand {
                actor: c.observer,
                expected_revision: rev,
                dedup_id: rev + 100,
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
        } else {
            let rev = engine.revision();
            engine.submit(GameCommand {
                actor: engine.turn().current,
                expected_revision: rev,
                dedup_id: rev + 100,
                action: GameAction::NextPhase,
            });
        }
    }
    assert_eq!((engine.turn().current, engine.turn().phase), (0, Phase::Play));
    let danshou_def = engine.catalog().skill_by_name("ol_jie_danshou").unwrap();
    assert!(engine
        .legal_actions(0)
        .contains(&GameAction::ActivateSkill { skill: danshou_def }));

    let before_hash = engine.state_hash();
    let before_rng = engine.state_core().rng;

    let planner_a = BoundedPlanner::new(16).with_policy_variant(1);
    let planner_b = BoundedPlanner::new(16).with_policy_variant(1);
    let plan_a = engine.plan_action(0, &planner_a);
    let plan_b = engine.plan_action(0, &planner_b);
    assert_eq!(
        plan_a.action, plan_b.action,
        "BoundedPlanner is deterministic"
    );
    assert_eq!(plan_a.sim_faults, 0);
    assert_eq!(
        engine.state_hash(),
        before_hash,
        "BoundedPlanner must not mutate authority state_hash"
    );
    assert_eq!(
        engine.state_core().rng,
        before_rng,
        "Simulation RNG must stay isolated from authority RuleRng"
    );

    let tuning = ContentTuning {
        seat_weights: vec![(0, 1.5), (1, f64::INFINITY)],
        skill_threat: vec![
            ("ol_jie_danshou".to_string(), 2.5),
            ("ol_pingkou".to_string(), 2.0),
            ("yongjian".to_string(), f64::INFINITY),
        ],
        qz_cf: true,
    };
    assert_eq!(tuning.weight_of(0), 1.5);
    assert_eq!(
        tuning.weight_of(1),
        1.0,
        "non-finite seat weight clamps to finite 1.0"
    );
    assert_eq!(
        tuning.threat_of_skills(["ol_jie_danshou", "yongjian"]),
        2.5,
        "non-finite skill threat falls back to finite 1.0"
    );
    let tuned = BoundedPlanner::new(16)
        .with_policy_variant(2)
        .with_content_tuning(tuning);
    let tuned_plan = engine.plan_action(0, &tuned);
    assert_eq!(tuned_plan.sim_faults, 0);
    assert_eq!(engine.state_hash(), before_hash);
    assert_eq!(engine.state_core().rng, before_rng);

    // Activate ol_jie_danshou and verify NC10 ChoiceView::Own vs ChoiceView::Other boundary holds.
    let rev = engine.revision();
    let out = engine.submit(GameCommand {
        actor: 0,
        expected_revision: rev,
        dedup_id: rev + 500,
        action: GameAction::ActivateSkill { skill: danshou_def },
    });
    assert!(matches!(out, CommandOutcome::Pending { .. }));
    let choice_hash = engine.state_hash();
    let choice_rng = engine.state_core().rng;

    let own_plan = engine.plan_action(0, &planner_a);
    assert!(
        matches!(own_plan.action, GameAction::Answer { .. }),
        "ChoiceView::Own produces projected Answer"
    );
    let other_plan = engine.plan_action(1, &planner_a);
    assert_eq!(
        other_plan.action,
        GameAction::NextPhase,
        "ChoiceView::Other preserves NC10 boundary without reading private choice options"
    );
    assert_eq!(engine.state_hash(), choice_hash);
    assert_eq!(engine.state_core().rng, choice_rng);
}
