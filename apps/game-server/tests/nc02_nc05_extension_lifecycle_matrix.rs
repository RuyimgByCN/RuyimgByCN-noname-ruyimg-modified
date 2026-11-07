//! NC02 & NC05 扩展生命周期与签名验真闭环矩阵：
//! - Case 2/3：资源/UI 扩展包（`skin-switch` + `select-beautify`）`.sig.json` 签名验真、篡改拒绝、
//!   安装 → 启用 → 装载 → 升级 → 回滚 → 停用 → 卸载 → 旧 `ContentLock` 对局恢复不受影响。
//! - Case 3/3：双包依赖（`base-pack` + `dep-pack` 声明 `PackageDependency` 与 `logic_hash`）+
//!   跨包引用 + 运行期热停用/热卸载阻断 + 顺序卸载后旧 `ContentLock` 对局恢复隔离。

#[path = "../src/native.rs"]
mod native;
#[path = "../src/native_room_store.rs"]
mod native_room_store;
#[path = "../src/native_view.rs"]
mod native_view;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/room_native.rs"]
mod room_native;
#[path = "../src/room_native_actor.rs"]
mod room_native_actor;
#[path = "../src/session.rs"]
mod session;

use content_runtime::{
    import_extensions, sign_package, tooling_host_versions, verify_package, InstallOptions,
    PackageSignature, PackageStore, RuntimeError,
};
use content_schema::{ContentPackage, PackageDependency};
use engine_native::{Engine, GameAction};
use native::{restore_match_locked, save_fresh_match_locked, NativeContent};
use room_native::NativeRoomSession;
use std::path::{Path, PathBuf};

fn repo_packages_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
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

fn sign_dir_package(dir: &Path, key: &str, key_id: &str) -> PackageSignature {
    let pkg_path = dir.join("package.npack.json");
    let bytes = std::fs::read(&pkg_path).expect("read package.npack.json");
    let sig = sign_package(&bytes, key, key_id).expect("sign_package");
    let sig_json = serde_json::to_vec_pretty(&sig).unwrap();
    std::fs::write(dir.join("package.npack.sig.json"), sig_json).unwrap();
    sig
}

/// Case 2/3：资源/UI 扩展包（`skin-switch` + `select-beautify`）`.sig.json` 签名验真、篡改拒绝、
/// 安装 → 启用 → 装载 → 升级 → 回滚 → 停用 → 卸载 → 旧 `ContentLock` 对局恢复不受影响。
#[test]
fn nc02_nc05_case2_resource_ui_pack_signed_lifecycle_and_old_content_lock_restore() {
    let temp = tempfile::tempdir().unwrap();
    let dropin = temp.path().join("dropin");
    let store_dir = temp.path().join("store");
    std::fs::create_dir_all(&dropin).unwrap();

    let key = "nc02-prod-publisher-secret-key-2026";
    let wrong_key = "nc02-wrong-publisher-secret-key";

    // 1. 拷贝真实资源/UI 包 skin-switch-upstream-liuli 与 select-beautify-upstream-liuli 到临时 drop-in 目录
    let skin_dir = dropin.join("skin-switch-upstream-liuli");
    let ui_dir = dropin.join("select-beautify-upstream-liuli");
    copy_dir_recursive(
        &repo_packages_root().join("skin-switch-upstream-liuli"),
        &skin_dir,
    );
    copy_dir_recursive(
        &repo_packages_root().join("select-beautify-upstream-liuli"),
        &ui_dir,
    );

    // 2. 验证未签名包在 allow_unsigned = false 时被拒绝
    let store = PackageStore::open(&store_dir).unwrap();
    let strict_opts = InstallOptions {
        activate: true,
        allow_unsigned: false,
        verify_key: Some(key.to_string()),
        host: tooling_host_versions(),
    };
    let unsigned_err = import_extensions(&store, &dropin, &strict_opts).unwrap_err();
    assert!(
        matches!(unsigned_err, RuntimeError::UnsignedNotAllowed),
        "allow_unsigned=false 时未附带 .sig.json 的包必须拒绝安装: {unsigned_err:?}"
    );

    // 3. 生成 .sig.json，验证 verify_package 正常通过、篡改字节或错钥立即拒绝
    let skin_sig_v1 = sign_dir_package(&skin_dir, key, "nc02-key-v1");
    let _ui_sig_v1 = sign_dir_package(&ui_dir, key, "nc02-key-v1");

    let skin_bytes_v1 = std::fs::read(skin_dir.join("package.npack.json")).unwrap();
    assert!(verify_package(&skin_bytes_v1, key, &skin_sig_v1).unwrap());
    assert!(
        !verify_package(&skin_bytes_v1, wrong_key, &skin_sig_v1).unwrap(),
        "错误密钥必须验签失败"
    );
    let mut tampered_bytes = skin_bytes_v1.clone();
    tampered_bytes.push(b' ');
    assert!(
        !verify_package(&tampered_bytes, key, &skin_sig_v1).unwrap(),
        "篡改包字节必须验签失败"
    );
    let tamper_install_err = store
        .install_with_assets(
            &tampered_bytes,
            Some(&skin_sig_v1),
            &strict_opts,
            &skin_dir,
        )
        .unwrap_err();
    assert!(
        matches!(tamper_install_err, RuntimeError::Signature(_)),
        "篡改后的包字节安装必须返回 Signature 错误: {tamper_install_err:?}"
    );

    // 4. 严格验签模式安装 2 个资源/UI 包并激活
    let report = import_extensions(&store, &dropin, &strict_opts).unwrap();
    assert_eq!(report.installed.len(), 2);
    assert!(report.installed.iter().all(|m| m.signed));
    assert!(report
        .installed
        .iter()
        .all(|m| m.key_id.as_deref() == Some("nc02-key-v1")));

    let content = NativeContent::open(dropin.clone(), store_dir.clone()).unwrap();
    let loaded_skin_v1 = content
        .load_scoped(Some("upstream-liuli"), "skin-switch")
        .unwrap();
    let loaded_ui_v1 = content
        .load_scoped(Some("upstream-liuli"), "select-beautify")
        .unwrap();
    assert!(loaded_skin_v1.pack.cards.is_empty());
    assert!(loaded_ui_v1.pack.cards.is_empty());

    let asset_info_v1 = store
        .resolve_active_asset_info("upstream-liuli", "skin-switch", "skin-switch:config")
        .unwrap();
    let config_raw_v1 = std::fs::read_to_string(&asset_info_v1.path).unwrap();
    assert!(config_raw_v1.contains('{'));

    // 5. 用 v1.0.0 内容锁建立进行中对局并保存状态（含 canonical_locked_content 与 frozen_packages）
    let (fresh_bytes_v1, fresh_cards_v1, verified_v1) = save_fresh_match_locked(
        &content,
        &["skin-switch".to_string(), "select-beautify".to_string()],
        20261106,
        2,
    )
    .unwrap();
    let lock_v1 = store.current_lock().unwrap();
    let v1_entry = lock_v1
        .packages
        .iter()
        .find(|p| p.package_id == "skin-switch")
        .unwrap()
        .clone();

    let engine_v1 = Engine::new_with_extensions(
        20261106,
        2,
        &[&loaded_skin_v1.pack, &loaded_ui_v1.pack],
    )
    .unwrap();
    let mut session_v1 = NativeRoomSession::new_locked(
        engine_v1,
        verified_v1,
        vec![loaded_skin_v1.pack, loaded_ui_v1.pack],
        2,
    );
    session_v1.join(1, Some(0), None).unwrap();
    session_v1.join(2, Some(1), None).unwrap();
    let cur = session_v1.engine().turn().current;
    session_v1
        .submit_native(cur, GameAction::NextPhase, session_v1.engine().revision())
        .unwrap();
    let saved_v1_state = session_v1.export_state(&session_v1.snapshot_hex());

    // 6. 升级 skin-switch 到签名版 1.0.1（更新资源与签名）→ 验证激活、回滚、再激活
    let mut changed_cfg = config_raw_v1.clone();
    changed_cfg.push('\n');
    let new_cfg_hash: Vec<u8> = blake3::hash(changed_cfg.as_bytes()).as_bytes().to_vec();
    let mut val_v2: serde_json::Value = serde_json::from_slice(&skin_bytes_v1).unwrap();
    if let Some(entries) = val_v2["asset_manifest"]["entries"].as_array_mut() {
        for e in entries.iter_mut() {
            if e["asset_id"] == "skin-switch:config" {
                e["source_hash"] = serde_json::json!(new_cfg_hash);
            }
        }
    }
    let mut pkg_v2 =
        ContentPackage::from_json_bytes(serde_json::to_string(&val_v2).unwrap().as_bytes())
            .unwrap();
    pkg_v2.manifest.version = "1.0.1".to_string();
    pkg_v2.manifest.asset_hash = pkg_v2.compute_asset_hash();
    pkg_v2.manifest.logic_hash = pkg_v2.compute_logic_hash();
    let bytes_v2 = pkg_v2.to_json_bytes().unwrap();
    let sig_v2 = sign_package(&bytes_v2, key, "nc02-key-v1").unwrap();

    let skin_v2_src = temp.path().join("skin-v2-src");
    copy_dir_recursive(&skin_dir, &skin_v2_src);
    std::fs::write(skin_v2_src.join("assets/config.json"), &changed_cfg).unwrap();
    store
        .install_with_assets(&bytes_v2, Some(&sig_v2), &strict_opts, &skin_v2_src)
        .unwrap();

    let lock_v2 = store.current_lock().unwrap();
    let v2_entry = lock_v2
        .packages
        .iter()
        .find(|p| p.package_id == "skin-switch")
        .unwrap();
    assert_eq!(v2_entry.version, "1.0.1");
    assert_ne!(v2_entry.content_hash, v1_entry.content_hash);

    // 回滚到 1.0.0
    content.rollback().unwrap();
    let lock_after_rb = store.current_lock().unwrap();
    assert_eq!(
        lock_after_rb
            .packages
            .iter()
            .find(|p| p.package_id == "skin-switch")
            .unwrap()
            .version,
        v1_entry.version
    );

    // 重新激活 1.0.1
    content
        .activate_version("upstream-liuli", "skin-switch", "1.0.1")
        .unwrap();

    // 7. 停用 → 不可装载 → 重新启用 → 可装载
    content
        .disable_package("upstream-liuli", "skin-switch")
        .unwrap();
    assert!(
        content
            .load_scoped(Some("upstream-liuli"), "skin-switch")
            .is_err(),
        "停用后必须不可装载"
    );
    content
        .enable_package("upstream-liuli", "skin-switch", "1.0.1")
        .unwrap();
    assert!(content
        .load_scoped(Some("upstream-liuli"), "skin-switch")
        .is_ok());

    // 8. 物理卸载两个包后，先前基于旧 ContentLock 落盘的对局与存档仍可完整恢复并一致推进
    content
        .uninstall_package("upstream-liuli", "skin-switch")
        .unwrap();
    content
        .uninstall_package("upstream-liuli", "select-beautify")
        .unwrap();
    assert!(content.installed().unwrap().is_empty());

    let restored_cards = restore_match_locked(
        &content,
        saved_v1_state.canonical_locked_content.as_ref().unwrap(),
        saved_v1_state.frozen_packages.as_deref().unwrap(),
        &fresh_bytes_v1,
    )
    .unwrap();
    assert_eq!(restored_cards, fresh_cards_v1);

    let restored_v1 = NativeRoomSession::from_saved(&saved_v1_state, vec![]).unwrap();
    assert_eq!(
        restored_v1.engine().state_hash(),
        session_v1.engine().state_hash()
    );
}

/// Case 3/3：双包依赖（`base-pack` + `dep-pack` 声明 `PackageDependency` 与 `logic_hash`）+
/// 跨包引用 + 运行期热停用/热卸载阻断 + 顺序卸载后旧 `ContentLock` 恢复隔离。
#[test]
fn nc02_nc05_case3_hybrid_two_pack_dependency_cross_ref_and_hot_uninstall_isolation() {
    let temp = tempfile::tempdir().unwrap();
    let dropin = temp.path().join("dropin");
    let store_dir = temp.path().join("store");
    std::fs::create_dir_all(&dropin).unwrap();

    let key = "nc05-hybrid-dep-signing-key-2026";

    // 1. 构造带签名与资产的基座包 upstream-liuli/wuxing 与依赖它的混合包 upstream-noname/shahai-swd
    let base_dir = dropin.join("wuxing-upstream-liuli");
    let dep_dir = dropin.join("shahai-swd-upstream-noname");
    copy_dir_recursive(
        &repo_packages_root().join("wuxing-upstream-liuli"),
        &base_dir,
    );
    copy_dir_recursive(
        &repo_packages_root().join("shahai-swd-upstream-noname"),
        &dep_dir,
    );

    let mut base_pkg = ContentPackage::from_json_bytes(
        &std::fs::read(base_dir.join("package.npack.json")).unwrap(),
    )
    .unwrap();
    let base_ns = base_pkg.manifest.namespace.clone();
    let base_id = base_pkg.manifest.package_id.clone();
    let base_ver = base_pkg.manifest.version.clone();
    base_pkg.manifest.asset_hash = base_pkg.compute_asset_hash();
    let base_logic_hash = base_pkg.compute_logic_hash();
    base_pkg.manifest.logic_hash = base_logic_hash;
    let base_bytes = base_pkg.to_json_bytes().unwrap();
    std::fs::write(base_dir.join("package.npack.json"), &base_bytes).unwrap();
    let base_sig = sign_package(&base_bytes, key, "nc05-hybrid-key").unwrap();

    let mut dep_pkg = ContentPackage::from_json_bytes(
        &std::fs::read(dep_dir.join("package.npack.json")).unwrap(),
    )
    .unwrap();
    let dep_ns = dep_pkg.manifest.namespace.clone();
    let dep_id = dep_pkg.manifest.package_id.clone();
    let dep_ver = dep_pkg.manifest.version.clone();
    dep_pkg.manifest.dependencies = vec![PackageDependency {
        namespace: base_ns.clone(),
        package_id: base_id.clone(),
        version: base_ver.clone(),
        logic_hash: base_logic_hash,
    }];
    dep_pkg.manifest.asset_hash = dep_pkg.compute_asset_hash();
    dep_pkg.manifest.logic_hash = dep_pkg.compute_logic_hash();
    let dep_bytes = dep_pkg.to_json_bytes().unwrap();
    std::fs::write(dep_dir.join("package.npack.json"), &dep_bytes).unwrap();
    let dep_sig = sign_package(&dep_bytes, key, "nc05-hybrid-key").unwrap();

    let store = PackageStore::open(&store_dir).unwrap();
    let install_inactive = InstallOptions {
        activate: false,
        allow_unsigned: false,
        verify_key: Some(key.to_string()),
        host: tooling_host_versions(),
    };
    store
        .install_with_assets(&base_bytes, Some(&base_sig), &install_inactive, &base_dir)
        .unwrap();
    store
        .install_with_assets(&dep_bytes, Some(&dep_sig), &install_inactive, &dep_dir)
        .unwrap();

    // 2. 单独激活 dep-pack 必须因缺少激活态依赖 base-pack 而失败
    let dep_only_err = store
        .activate_version(&dep_ns, &dep_id, &dep_ver)
        .unwrap_err();
    assert!(
        dep_only_err.to_string().contains(&base_id),
        "未激活 base-pack 时单独激活 dep-pack 必须报错指明缺失依赖: {dep_only_err:?}"
    );

    // 3. 原子激活 [base-pack, dep-pack] 成功
    store
        .activate_packages(&[
            PackageDependency {
                namespace: base_ns.clone(),
                package_id: base_id.clone(),
                version: base_ver.clone(),
                logic_hash: base_logic_hash,
            },
            PackageDependency {
                namespace: dep_ns.clone(),
                package_id: dep_id.clone(),
                version: dep_ver.clone(),
                logic_hash: [0u8; 32],
            },
        ])
        .unwrap();

    let content = NativeContent::open(dropin.clone(), store_dir.clone()).unwrap();
    let loaded_base = content.load_scoped(Some(&base_ns), &base_id).unwrap();
    let loaded_dep = content.load_scoped(Some(&dep_ns), &dep_id).unwrap();
    assert!(!loaded_base.pack.cards.is_empty());
    assert!(!loaded_dep.pack.cards.is_empty() || !loaded_dep.pack.generals.is_empty());

    // 4. 组合双包建局并推进，记录含完整锁与副本的房间存档
    let (fresh_bytes, fresh_cards, verified) = save_fresh_match_locked(
        &content,
        &[base_id.clone(), dep_id.clone()],
        20261107,
        4,
    )
    .unwrap();
    let engine = Engine::new_with_extensions(
        20261107,
        4,
        &[&loaded_base.pack, &loaded_dep.pack],
    )
    .unwrap();
    let mut session = NativeRoomSession::new_locked(
        engine,
        verified,
        vec![loaded_base.pack, loaded_dep.pack],
        4,
    );
    session.join(1, Some(0), None).unwrap();
    let cur = session.engine().turn().current;
    session
        .submit_native(cur, GameAction::NextPhase, session.engine().revision())
        .unwrap();
    let saved_state = session.export_state(&session.snapshot_hex());

    // 5. 当 dep-pack 仍处于 active 时，停用或物理卸载 base-pack 必须在任何变更前被阻断
    let disable_base_err = content.disable_package(&base_ns, &base_id);
    assert!(
        disable_base_err.is_err(),
        "dep-pack 激活时停用被依赖方 base-pack 必须失败"
    );
    let uninstall_base_err = content.uninstall_package(&base_ns, &base_id);
    assert!(
        uninstall_base_err.is_err(),
        "dep-pack 激活时卸载被依赖方 base-pack 必须失败"
    );
    // 验证阻断后 base-pack 与 dep-pack 均保持已安装与激活可装载
    assert_eq!(content.installed().unwrap().len(), 2);
    assert!(content.load_scoped(Some(&base_ns), &base_id).is_ok());
    assert!(content.load_scoped(Some(&dep_ns), &dep_id).is_ok());

    // 6. 按依赖拓扑顺序卸载（先 dep-pack 后 base-pack）成功，且已开局房间由冻结 ContentLock 完整恢复
    content.uninstall_package(&dep_ns, &dep_id).unwrap();
    content.uninstall_package(&base_ns, &base_id).unwrap();
    assert!(content.installed().unwrap().is_empty());

    let restored_cards = restore_match_locked(
        &content,
        saved_state.canonical_locked_content.as_ref().unwrap(),
        saved_state.frozen_packages.as_deref().unwrap(),
        &fresh_bytes,
    )
    .unwrap();
    assert_eq!(restored_cards, fresh_cards);

    let restored = NativeRoomSession::from_saved(&saved_state, vec![]).unwrap();
    assert_eq!(
        restored.engine().state_hash(),
        session.engine().state_hash()
    );
}
