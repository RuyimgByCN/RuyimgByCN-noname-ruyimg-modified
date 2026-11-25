//! 37 来源八维仓内签署公共测试脚手架。

use std::path::{Path, PathBuf};

use content_runtime::{
    import_extensions, sign_package, tooling_host_versions, verify_package, InstallOptions,
    PackageSignature, PackageStore, RuntimeError,
};
use content_schema::{ContentPackage, PackageDependency};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::Engine;

use super::native::{restore_match_locked, save_fresh_match_locked, NativeContent};
use super::room_native::NativeRoomSession;

pub const TEST_KEY: &str = "up37-eight-dim-signoff-key-2026";
pub const WRONG_KEY: &str = "up37-eight-dim-wrong-key-2026";

pub fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &target);
        } else if ty.is_file() {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

pub fn sign_dir_package(dir: &Path, key: &str, key_id: &str) -> PackageSignature {
    let pkg_path = dir.join("package.npack.json");
    let bytes = std::fs::read(&pkg_path).expect("read package.npack.json");
    let sig = sign_package(&bytes, key, key_id).expect("sign_package");
    let sig_json = serde_json::to_vec_pretty(&sig).unwrap();
    std::fs::write(dir.join("package.npack.sig.json"), sig_json).unwrap();
    sig
}

pub fn normalize_json_lf(dir: &Path) {
    let pkg_path = dir.join("package.npack.json");
    if let Ok(raw) = std::fs::read_to_string(&pkg_path) {
        let lf = raw.replace("\r\n", "\n");
        let _ = std::fs::write(&pkg_path, lf);
    }
}

pub fn bump_patch_version(ver: &str) -> String {
    let mut parts: Vec<String> = ver.split('.').map(|s| s.to_string()).collect();
    if let Some(last) = parts.last_mut() {
        if let Ok(n) = last.parse::<u64>() {
            *last = (n + 1).to_string();
            return parts.join(".");
        }
    }
    format!("{ver}.1")
}

/// 逐来源验证全部仓内可验维度（① 签名安装/幂等/启停重启/升级回滚/卸载重启；
/// ③ 依赖声明/宿主版本门禁/被依赖保护/缺依赖拒绝；⑤ 锁建局/升级及卸载后旧锁无损恢复/错版拒恢复）。
pub fn verify_source_full_lifecycle_and_deps(source_id: &str, pkg_dirs: &[&str], seed: u64) {
    assert!(!pkg_dirs.is_empty(), "{source_id}: must have >= 1 package");
    let temp = tempfile::tempdir().unwrap();
    let dropin = temp.path().join("dropin");
    let store_dir = temp.path().join("store");
    std::fs::create_dir_all(&dropin).unwrap();

    for dir in pkg_dirs {
        let dst = dropin.join(dir);
        copy_dir_recursive(&repo_extensions_root().join(dir), &dst);
        normalize_json_lf(&dst);
    }

    // ①a 未签名包在 allow_unsigned=false 时必被拒
    let store = PackageStore::open(&store_dir).unwrap();
    let strict_opts = InstallOptions {
        activate: true,
        allow_unsigned: false,
        verify_key: Some(TEST_KEY.to_string()),
        host: tooling_host_versions(),
    };
    let unsigned_err = import_extensions(&store, &dropin, &strict_opts).unwrap_err();
    assert!(
        matches!(unsigned_err, RuntimeError::UnsignedNotAllowed),
        "{source_id}: unsigned package must be rejected: {unsigned_err:?}"
    );

    // ①b 为全部包签名，并校验篡改字节与错密钥拒装
    let mut primary_meta = Vec::new();
    for dir in pkg_dirs {
        let pdir = dropin.join(dir);
        let sig = sign_dir_package(&pdir, TEST_KEY, "up37-key-v1");
        let bytes = std::fs::read(pdir.join("package.npack.json")).unwrap();
        assert!(verify_package(&bytes, TEST_KEY, &sig).unwrap());
        assert!(!verify_package(&bytes, WRONG_KEY, &sig).unwrap());
        let mut tampered = bytes.clone();
        tampered.push(b' ');
        assert!(!verify_package(&tampered, TEST_KEY, &sig).unwrap());
        let tamper_err = store
            .install_with_assets(&tampered, Some(&sig), &strict_opts, &pdir)
            .unwrap_err();
        assert!(
            matches!(tamper_err, RuntimeError::Signature(_)),
            "{source_id}/{dir}: tampered package must fail signature check: {tamper_err:?}"
        );
        let pkg = ContentPackage::from_json_bytes(&bytes).unwrap();
        primary_meta.push((
            dir.to_string(),
            pkg.manifest.namespace.clone(),
            pkg.manifest.package_id.clone(),
            pkg.manifest.version.clone(),
            pkg.compute_logic_hash(),
        ));
    }

    // ③a 不兼容宿主版本门禁（max_engine_api < host.engine_api）
    {
        let incompatible_store = PackageStore::open(temp.path().join("bad-host-store")).unwrap();
        let mut bad_host = tooling_host_versions();
        bad_host.engine_api = u32::MAX;
        let bad_opts = InstallOptions {
            activate: true,
            allow_unsigned: true,
            verify_key: None,
            host: bad_host,
        };
        let host_err = import_extensions(&incompatible_store, &dropin, &bad_opts).unwrap_err();
        assert!(
            matches!(host_err, RuntimeError::Incompatible(_)),
            "{source_id}: incompatible host must be rejected: {host_err:?}"
        );
    }

    // ①c 正式验签导入 + 幂等
    let report1 = import_extensions(&store, &dropin, &strict_opts).unwrap();
    assert_eq!(report1.installed.len(), pkg_dirs.len());
    assert!(report1.installed.iter().all(|m| m.signed));
    let report2 = import_extensions(&store, &dropin, &strict_opts).unwrap();
    assert_eq!(report2.installed.len(), pkg_dirs.len());

    // ①d 停用 -> 重启store验证停用持久 -> 启用
    let content = NativeContent::open(dropin.clone(), store_dir.clone()).unwrap();
    for (_, ns, pid, ver, _) in &primary_meta {
        content.disable_package(ns, pid).unwrap();
        assert!(content.load_scoped(Some(ns), pid).is_err());
        let reopened = NativeContent::open(dropin.clone(), store_dir.clone()).unwrap();
        assert!(
            reopened.load_scoped(Some(ns), pid).is_err(),
            "{source_id}/{pid}: disabled state must persist across reopen"
        );
        reopened.enable_package(ns, pid, ver).unwrap();
        assert!(reopened.load_scoped(Some(ns), pid).is_ok());
    }

    // ⑤a 在 v1 激活态创建冻结锁对局并导出完整存档
    let pkg_ids: Vec<String> = primary_meta.iter().map(|m| m.2.clone()).collect();
    let ns_scope = if primary_meta
        .iter()
        .all(|m| m.1 == primary_meta[0].1)
    {
        Some(primary_meta[0].1.as_str())
    } else {
        None
    };
    let verified_v1 = content.freeze_room_selection(ns_scope, &pkg_ids).unwrap();
    let (fresh_bytes_v1, fresh_cards_v1, _) =
        save_fresh_match_locked(&content, &pkg_ids, seed, 4).unwrap();
    let mut engine_v1 = content_native_loader::start_locked_engine(
        seed,
        4,
        content_native_loader::LockedStartMode::Prototype,
        &verified_v1,
    )
    .unwrap();
    let planner = BoundedPlanner::new(64);
    for _ in 0..8 {
        if engine_v1.outcome().is_some() {
            break;
        }
        let seat = engine_v1
            .open_choice()
            .map(|c| c.observer)
            .unwrap_or_else(|| engine_v1.turn().current);
        let plan = engine_v1.plan_action(seat, &planner);
        engine_v1.apply_plan(&plan).unwrap();
    }
    let snap_v1 = engine_v1.save();
    let hash_v1 = engine_v1.state_hash();
    let archive_v1 = encode_archive(&snap_v1);
    let mut session_v1 = NativeRoomSession::new_locked(engine_v1, verified_v1, vec![], 4);
    let saved_v1_state = session_v1.export_state(&session_v1.snapshot_hex());

    // ⑤b 错版/无包恢复拒绝校验
    let decoded_v1 = decode_archive(&archive_v1).unwrap();
    assert!(
        Engine::restore(&decoded_v1).is_err(),
        "{source_id}: bare Engine::restore without extensions must fail"
    );

    // ①e 升级首包到 patch+1（重算 hash 与签名）-> 验证 lock 升级 -> 回滚 -> 再升级
    let (first_dir, first_ns, first_pid, first_ver, _) = &primary_meta[0];
    let next_ver = bump_patch_version(first_ver);
    let v2_dir = temp.path().join("v2_src");
    copy_dir_recursive(&dropin.join(first_dir), &v2_dir);
    let mut pkg_v2 = ContentPackage::from_json_bytes(
        &std::fs::read(v2_dir.join("package.npack.json")).unwrap(),
    )
    .unwrap();
    pkg_v2.manifest.version = next_ver.clone();
    pkg_v2.manifest.asset_hash = pkg_v2.compute_asset_hash();
    pkg_v2.manifest.logic_hash = pkg_v2.compute_logic_hash();
    let bytes_v2 = pkg_v2.to_json_bytes().unwrap();
    let sig_v2 = sign_package(&bytes_v2, TEST_KEY, "up37-key-v1").unwrap();
    store
        .install_with_assets(&bytes_v2, Some(&sig_v2), &strict_opts, &v2_dir)
        .unwrap();
    let lock_v2 = store.current_lock().unwrap();
    assert_eq!(
        lock_v2
            .packages
            .iter()
            .find(|p| p.namespace == *first_ns && p.package_id == *first_pid)
            .unwrap()
            .version,
        next_ver
    );
    content.rollback().unwrap();
    let lock_rb = store.current_lock().unwrap();
    assert_eq!(
        lock_rb
            .packages
            .iter()
            .find(|p| p.namespace == *first_ns && p.package_id == *first_pid)
            .unwrap()
            .version,
        *first_ver
    );
    content
        .activate_version(first_ns, first_pid, &next_ver)
        .unwrap();

    // ⑤c 升级后旧锁恢复仍 100% 一致
    let restored_after_upgrade = restore_match_locked(
        &content,
        saved_v1_state.canonical_locked_content.as_ref().unwrap(),
        saved_v1_state.frozen_packages.as_deref().unwrap(),
        &fresh_bytes_v1,
    )
    .unwrap();
    assert_eq!(restored_after_upgrade, fresh_cards_v1);

    // ③b 构造下游依赖包 `dep-probe`：缺依赖单独激活必被拒，激活后阻止卸载与停用主包
    let mut dep_pkg = pkg_v2.clone();
    dep_pkg.manifest.namespace = "up37-probe".to_string();
    dep_pkg.manifest.package_id = format!("dep-{source_id}").to_lowercase();
    dep_pkg.manifest.version = "1.0.0".to_string();
    dep_pkg.manifest.dependencies = vec![PackageDependency {
        namespace: first_ns.clone(),
        package_id: first_pid.clone(),
        version: next_ver.clone(),
        logic_hash: pkg_v2.manifest.logic_hash,
    }];
    dep_pkg.asset_paths.clear();
    dep_pkg.asset_manifest = None;
    dep_pkg.manifest.asset_hash = dep_pkg.compute_asset_hash();
    dep_pkg.manifest.logic_hash = dep_pkg.compute_logic_hash();
    let dep_bytes = dep_pkg.to_json_bytes().unwrap();
    let dep_sig = sign_package(&dep_bytes, TEST_KEY, "up37-key-v1").unwrap();
    let empty_assets = temp.path().join("empty-assets");
    std::fs::create_dir_all(&empty_assets).unwrap();

    // 先验证缺失依赖安装/激活拒绝
    let mut missing_dep_pkg = dep_pkg.clone();
    missing_dep_pkg.manifest.package_id = format!("missing-dep-{source_id}").to_lowercase();
    missing_dep_pkg.manifest.dependencies[0].package_id = "nonexistent-dep-pkg".to_string();
    missing_dep_pkg.manifest.logic_hash = missing_dep_pkg.compute_logic_hash();
    let missing_bytes = missing_dep_pkg.to_json_bytes().unwrap();
    let missing_sig = sign_package(&missing_bytes, TEST_KEY, "up37-key-v1").unwrap();
    let missing_err = store
        .install_with_assets(&missing_bytes, Some(&missing_sig), &strict_opts, &empty_assets)
        .unwrap_err();
    assert!(
        missing_err.to_string().contains("nonexistent-dep-pkg"),
        "{source_id}: missing dependency must be rejected: {missing_err:?}"
    );

    store
        .install_with_assets(&dep_bytes, Some(&dep_sig), &strict_opts, &empty_assets)
        .unwrap();
    assert!(
        content.disable_package(first_ns, first_pid).is_err(),
        "{source_id}: disabling active dependency target must be blocked"
    );
    assert!(
        content.uninstall_package(first_ns, first_pid).is_err(),
        "{source_id}: uninstalling active dependency target must be blocked"
    );

    // ①f 先卸载依赖方与失败残留包，再物理卸载全部原包并重开 PackageStore 确认彻底清空
    content
        .uninstall_package(&dep_pkg.manifest.namespace, &dep_pkg.manifest.package_id)
        .unwrap();
    content
        .uninstall_package(
            &missing_dep_pkg.manifest.namespace,
            &missing_dep_pkg.manifest.package_id,
        )
        .unwrap();
    for (_, ns, pid, _, _) in &primary_meta {
        content.uninstall_package(ns, pid).unwrap();
    }
    let reopened_after_uninstall = NativeContent::open(dropin, store_dir).unwrap();
    assert!(reopened_after_uninstall.installed().unwrap().is_empty());

    // ⑤d 物理卸载后旧冻结锁对局仍可无损恢复且 hash 100% 一致
    let restored_cards_after_uninstall = restore_match_locked(
        &reopened_after_uninstall,
        saved_v1_state.canonical_locked_content.as_ref().unwrap(),
        saved_v1_state.frozen_packages.as_deref().unwrap(),
        &fresh_bytes_v1,
    )
    .unwrap();
    assert_eq!(restored_cards_after_uninstall, fresh_cards_v1);
    let restored_session = NativeRoomSession::from_saved(&saved_v1_state, vec![]).unwrap();
    assert_eq!(restored_session.engine().state_hash(), hash_v1);
}

/// 校验多包组合建局 + 反向加载序 + 推进 + 全区牌守恒 + 存档哈希一致。
pub fn verify_combo_match(label: &str, pkg_dirs: &[&str], seed: u64) {
    let temp = tempfile::tempdir().unwrap();
    let dropin = temp.path().join("dropin");
    let store_dir = temp.path().join("store");
    std::fs::create_dir_all(&dropin).unwrap();
    for dir in pkg_dirs {
        let dst = dropin.join(dir);
        copy_dir_recursive(&repo_extensions_root().join(dir), &dst);
        normalize_json_lf(&dst);
    }
    let content = NativeContent::open(dropin, store_dir).unwrap();
    let installed = content.import().unwrap();
    assert_eq!(installed.len(), pkg_dirs.len(), "{label}: import count");

    let mut pids = Vec::with_capacity(pkg_dirs.len());
    for dir in pkg_dirs {
        let raw = std::fs::read_to_string(repo_extensions_root().join(dir).join("package.npack.json")).unwrap();
        let pkg = ContentPackage::from_json_bytes(raw.as_bytes()).unwrap();
        pids.push(pkg.manifest.package_id);
    }
    let verified = content.freeze_room_selection(None, &pids).unwrap();
    let mut e = content_native_loader::start_locked_engine(
        seed,
        4,
        content_native_loader::LockedStartMode::Prototype,
        &verified,
    )
    .unwrap();
    let initial_hash = e.state_hash();
    let initial_cards = e.catalog().cards.len();
    let planner = BoundedPlanner::new(64);
    for _ in 0..8 {
        if e.outcome().is_some() {
            break;
        }
        let seat = e.open_choice().map(|c| c.observer).unwrap_or_else(|| e.turn().current);
        let plan = e.plan_action(seat, &planner);
        e.apply_plan(&plan).unwrap();
    }
    let snap = e.save();
    let mut ids = Vec::new();
    for p in &snap.core.players {
        ids.extend(p.hand.iter().copied());
        ids.extend(p.equips.iter().copied());
        ids.extend(p.judge.iter().copied());
    }
    ids.extend(snap.core.deck.iter().copied());
    ids.extend(snap.core.discard.iter().copied());
    ids.extend(snap.core.table.iter().copied());
    let raw_len = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), raw_len, "{label}: card ownership conservation");
    let restored = content_native_loader::restore_locked_engine(&snap, &verified).unwrap();
    assert_eq!(restored.state_hash(), e.state_hash(), "{label}: state_hash");

    // 反向输入列表亦经规范内容锁排序正常建局并保持一致
    let mut pids_rev = pids.clone();
    pids_rev.reverse();
    let verified_rev = content.freeze_room_selection(None, &pids_rev).unwrap();
    let e_rev = content_native_loader::start_locked_engine(
        seed,
        4,
        content_native_loader::LockedStartMode::Prototype,
        &verified_rev,
    )
    .unwrap();
    assert_eq!(e_rev.catalog().cards.len(), initial_cards, "{label}: rev card count");
    assert_eq!(e_rev.state_hash(), initial_hash, "{label}: rev initial state_hash");
}
