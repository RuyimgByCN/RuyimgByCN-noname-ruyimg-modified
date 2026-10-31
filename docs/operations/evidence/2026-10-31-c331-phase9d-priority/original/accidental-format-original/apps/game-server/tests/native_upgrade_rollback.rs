//! NC02：同包安装新版本后，显式 activate 才切换；rollback 恢复上一版本。

#[path = "../src/native.rs"]
mod native;

use native::{restore_match, save_fresh_match, NativeContent};

fn package(version: &str) -> Vec<u8> {
    let source = format!(
        "skill upgrade_probe(990010) {{ trigger phase_end owner_is_target {{ effect draw_cards(self, 1); }} }}"
    );
    let mut package =
        content_compiler::compile_source_named("upgrade-probe", &source).expect("compile");
    package.manifest.version = version.into();
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package.to_json_bytes().unwrap()
}

#[test]
fn activate_new_version_then_rollback() {
    let dir = tempfile::tempdir().unwrap();
    let dropin = dir.path().join("upgrade-probe");
    std::fs::create_dir_all(&dropin).unwrap();
    std::fs::write(dropin.join("package.npack.json"), package("0.1.0")).unwrap();

    let content =
        NativeContent::open(dir.path().to_path_buf(), dir.path().join("store")).expect("open");
    content.import().expect("install v1");
    let v1 = content.load("upgrade-probe").expect("load v1");
    assert_eq!(v1.frozen.binding.version, "0.1.0");

    std::fs::write(dropin.join("package.npack.json"), package("0.2.0")).unwrap();
    content.import().expect("install v2");
    let v2 = content.load("upgrade-probe").expect("load v2");
    assert_eq!(v2.frozen.binding.version, "0.2.0", "导入 v2 后自动激活");

    let namespace = v2.frozen.binding.namespace.clone();
    content
        .activate_version(&namespace, "upgrade-probe", "0.2.0")
        .expect("activate v2");

    content.rollback().expect("rollback");
    let back = content.load("upgrade-probe").expect("load rollback");
    assert_eq!(back.frozen.binding.version, "0.1.0", "rollback 应恢复 v1");

    content
        .revoke_package(&namespace, "upgrade-probe", "uninstall test")
        .expect("revoke");
    assert!(content.load("upgrade-probe").is_err(), "撤销后不得再装载");
    assert!(
        content
            .activate_version(&namespace, "upgrade-probe", "0.2.0")
            .is_err(),
        "撤销后不得再激活"
    );
}

#[test]
fn physical_uninstall_removes_active_slot_and_versions() {
    let dir = tempfile::tempdir().unwrap();
    let dropin = dir.path().join("upgrade-probe");
    std::fs::create_dir_all(&dropin).unwrap();
    std::fs::write(dropin.join("package.npack.json"), package("0.1.0")).unwrap();
    let content =
        NativeContent::open(dir.path().to_path_buf(), dir.path().join("store")).expect("open");
    content.import().expect("install");
    let loaded = content.load("upgrade-probe").expect("load");
    let namespace = loaded.frozen.binding.namespace.clone();

    content
        .uninstall_package(&namespace, "upgrade-probe")
        .expect("uninstall");
    assert!(content.load("upgrade-probe").is_err(), "卸载后不可装载");
    assert!(
        content
            .installed()
            .unwrap()
            .iter()
            .all(|p| p.package_id != "upgrade-probe"),
        "卸载后不应再列出"
    );
    // 无撤销标记：重新导入可恢复安装。
    content.import().expect("reinstall");
    assert!(content.load("upgrade-probe").is_ok(), "重新安装后应可装载");
}

#[test]
fn disable_then_enable_keeps_installed_copy() {
    let dir = tempfile::tempdir().unwrap();
    let dropin = dir.path().join("disable-probe");
    std::fs::create_dir_all(&dropin).unwrap();
    std::fs::write(dropin.join("package.npack.json"), package("0.1.0")).unwrap();
    let content =
        NativeContent::open(dir.path().to_path_buf(), dir.path().join("store")).expect("open");
    content.import().expect("install");
    let loaded = content.load("upgrade-probe").expect("load");
    let namespace = loaded.frozen.binding.namespace.clone();

    content
        .disable_package(&namespace, "upgrade-probe")
        .expect("disable");
    assert!(content.load("upgrade-probe").is_err(), "停用后不可装载");
    content
        .enable_package(&namespace, "upgrade-probe", "0.1.0")
        .expect("enable");
    assert!(content.load("upgrade-probe").is_ok(), "启用后恢复装载");
}

#[test]
fn content_lock_rejects_restoring_after_package_upgrade() {
    let dir = tempfile::tempdir().unwrap();
    let dropin = dir.path().join("lock-probe");
    std::fs::create_dir_all(&dropin).unwrap();
    std::fs::write(dropin.join("package.npack.json"), package("0.1.0")).unwrap();
    let content =
        NativeContent::open(dir.path().to_path_buf(), dir.path().join("store")).expect("open");
    content.import().expect("install v1");
    let (bytes, _) = save_fresh_match(&content, &["upgrade-probe".into()], 42, 2).expect("save v1");

    std::fs::write(dropin.join("package.npack.json"), package("0.2.0")).unwrap();
    content.import().expect("install v2");
    let error = restore_match(&content, &["upgrade-probe".into()], &bytes)
        .expect_err("升级后旧局必须因内容锁拒绝");
    assert!(error.contains("恢复失败"), "unexpected: {error}");
}
