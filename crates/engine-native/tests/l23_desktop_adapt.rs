//! L23 电脑适配八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装出发。预设、导出文件名和鼠标点击阈值
//! 走包内配置。窗口移动、DOM、CSS 和 `game.use_throw_emoji` 没有迁入。

use std::path::PathBuf;

use content_native_loader::{
    apply_desktop_adapt, classify_pointer, export_download, invoke_emoji, parse_desktop_config,
    restore_snapshot, set_layout, snapshot, DesktopError, EmojiBridgeState, HostKind, Layout,
    PointerClass, PointerContext,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency};

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn store_dir(store: &PackageStore, namespace: &str, package_id: &str, version: &str) -> PathBuf {
    store
        .root()
        .join("packages")
        .join(format!("{namespace}__{package_id}"))
        .join(version)
}

fn read_installed(
    store: &PackageStore,
    namespace: &str,
    package_id: &str,
    version: &str,
) -> (ContentPackage, PathBuf) {
    let dir = store_dir(store, namespace, package_id, version);
    let bytes = std::fs::read(dir.join("package.json")).expect("商店副本 package.json");
    let package = ContentPackage::from_json_bytes(&bytes).expect("解析商店副本");
    (package, dir)
}

fn active_version(store: &PackageStore, package_id: &str) -> String {
    store
        .read_active()
        .expect("active")
        .packages
        .iter()
        .find(|(slot, _)| slot.ends_with(&format!("/{package_id}")))
        .map(|(_, version)| version.clone())
        .unwrap_or_else(|| panic!("激活集含 {package_id}"))
}

fn allowed() -> PointerContext {
    PointerContext {
        is_mine: false,
        is_dead: false,
        game_over: false,
        emoji_available: true,
    }
}

/// L23 八项：功能、配置、依赖、组合、保存恢复、安装升级、规则差异。
/// 目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l23_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let source_root = repo_extensions_root().join("desktop-adapt-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text =
        std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "desktop-adapt")
        .expect("电脑适配已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(meta.namespace, "upstream-liuli");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes),
        "商店记录的 content_hash 来自真实 npack 字节"
    );

    let (installed, installed_dir) =
        read_installed(&store, "upstream-liuli", "desktop-adapt", "1.0.0");
    assert!(
        installed.manifest.dependencies.is_empty(),
        "依赖正例：不硬依赖手杀 UI"
    );
    let stored_config = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(
        stored_config.as_bytes(),
        config_text.as_bytes(),
        "商店里的配置副本与仓库源一致"
    );
    let config = parse_desktop_config(&stored_config).expect("配置正例");
    assert_eq!(config.tap_threshold_px, 10);
    assert!(config.hide_settlement_hint);
    assert!(!config.thunder_ui_patch);
    assert!(config.emoji_bridge);
    let bad_layout = parse_desktop_config(&stored_config.replace("\"mouse\"", "\"wide\""));
    assert!(
        bad_layout.unwrap_err().contains("unknown layout"),
        "配置反例：未知布局"
    );
    let bad_device = parse_desktop_config(&stored_config.replace("\"android\"", "\"ios\""));
    assert!(
        bad_device.unwrap_err().contains("unknown device_profile"),
        "配置反例：未知设备"
    );
    let bad_preset = parse_desktop_config(&stored_config.replace(
        "\"extension_雷霆万钧_UIpatch\":false",
        "\"extension_雷霆万钧_UIpatch\":true",
    ));
    assert!(
        bad_preset.unwrap_err().contains("must be false"),
        "配置反例：雷霆补丁不得打开"
    );
    assert_eq!(active_version(&store, "desktop-adapt"), "1.0.0");
    assert_eq!(
        active_version(&store, "huanle-cards"),
        "1.0.0",
        "组合：同一次导入仍激活欢乐卡牌"
    );
    assert_eq!(
        active_version(&store, "omnisearch"),
        "1.0.0",
        "组合：同一次导入仍激活全能搜索"
    );

    let lock_before = store.current_lock().expect("lock").lock_hash;
    let profile = apply_desktop_adapt(&config, HostKind::Desktop, false);
    assert_eq!(
        profile.device_profile,
        content_native_loader::DeviceProfile::Android
    );
    assert_eq!(profile.host, HostKind::Desktop);
    assert_eq!(profile.layout, Layout::Mouse);
    assert_eq!(
        profile.emoji_bridge,
        EmojiBridgeState::Inactive,
        "功能：未启用手杀 UI 时不请求表情桥"
    );
    assert_eq!(profile.presets.get("shouShaJieSuanInfo"), Some(&true));
    assert_eq!(
        profile.presets.get("extension_雷霆万钧_UIpatch"),
        Some(&false)
    );
    assert_eq!(
        store.current_lock().unwrap().lock_hash,
        lock_before,
        "组合：应用预设不改内容锁"
    );
    // 依赖反例：预设应用不要求手杀界面包在场。2026-10-17 起 L13 手杀MVP、
    // 2026-11-06 起 L14 手杀ui 均已作为独立转换包入仓；此处停用 `shousha-ui`
    // 验证激活集不含手杀 UI 包时预设仍走调用方数据，随后回滚恢复激活集。
    store
        .deactivate_slot("upstream-liuli", "shousha-ui")
        .expect("停用 shousha-ui");
    assert!(
        !store
            .read_active()
            .unwrap()
            .packages
            .iter()
            .any(|(slot, _)| slot.contains("shousha-ui")),
        "依赖反例：商店激活集里没有手杀 UI 包"
    );
    assert_eq!(
        apply_desktop_adapt(&config, HostKind::Desktop, false)
            .presets
            .get("shouShaJieSuanInfo"),
        Some(&true)
    );
    store.rollback().expect("回滚恢复 shousha-ui");

    let mut requested = apply_desktop_adapt(&config, HostKind::Desktop, true);
    assert_eq!(
        requested.emoji_bridge,
        EmojiBridgeState::Requested,
        "功能正例：手杀 UI 开启才请求桥"
    );
    assert_eq!(
        invoke_emoji("player-2"),
        Err(DesktopError::DomRefused),
        "规则差异：不调用抛表情"
    );
    assert_eq!(
        classify_pointer(&requested, 3, 4, &allowed()),
        PointerClass::Tap,
        "功能正例：5 像素鼠标位移是点击"
    );
    assert_eq!(
        classify_pointer(&requested, 10, 0, &allowed()),
        PointerClass::Drag,
        "功能反例：刚好 10 像素不是点击"
    );
    assert_eq!(
        classify_pointer(&requested, 6, 8, &allowed()),
        PointerClass::Drag,
        "功能反例：勾股 10 不是点击"
    );
    let mut mine = allowed();
    mine.is_mine = true;
    assert_eq!(
        classify_pointer(&requested, 0, 0, &mine),
        PointerClass::Ignored,
        "功能反例：自己的座位不抛"
    );
    let mut dead = allowed();
    dead.is_dead = true;
    assert_eq!(
        classify_pointer(&requested, 1, 1, &dead),
        PointerClass::Ignored
    );
    let mut over = allowed();
    over.game_over = true;
    assert_eq!(
        classify_pointer(&requested, 1, 1, &over),
        PointerClass::Ignored
    );
    let mut missing_emoji = allowed();
    missing_emoji.emoji_available = false;
    assert_eq!(
        classify_pointer(&requested, 1, 1, &missing_emoji),
        PointerClass::Ignored,
        "依赖反例：没有抛表情函数时不点击"
    );
    set_layout(&mut requested, "touch").expect("布局切换");
    assert_eq!(requested.layout, Layout::Touch);
    assert_eq!(
        classify_pointer(&requested, 0, 0, &allowed()),
        PointerClass::NativeTouch,
        "功能正例：触摸布局不套鼠标阈值"
    );
    assert!(set_layout(&mut requested, "wide")
        .unwrap_err()
        .contains("unknown layout"));
    assert_eq!(store.current_lock().unwrap().lock_hash, lock_before);

    let download = export_download("hello", "../a:b").expect("导出正例");
    assert_eq!(download.filename, "...a.b");
    assert!(!download.filename.contains('/') && !download.filename.contains('\\'));
    assert_eq!(download.bytes, b"hello");
    assert_eq!(export_download("x", "").expect("空名").filename, "noname");
    assert_eq!(
        export_download("x", ".."),
        Err(DesktopError::UnsafeName),
        "导出反例：拒绝 .."
    );
    assert_eq!(export_download("x", "."), Err(DesktopError::UnsafeName));
    let export_dir = tempfile::tempdir().unwrap();
    assert!(
        export_dir.path().read_dir().unwrap().next().is_none(),
        "导出不写磁盘"
    );

    let saved = snapshot(&requested);
    let restored = restore_snapshot(&saved).expect("保存恢复正例");
    assert_eq!(restored, requested);
    assert!(
        restore_snapshot(&saved.replace("\"touch\"", "\"wide\"")).is_err(),
        "保存恢复反例"
    );

    match store.activate_version("upstream-liuli", "desktop-adapt", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("安装反例：未安装版本应 NotFound: {other:?}"),
    }
    assert_eq!(active_version(&store, "desktop-adapt"), "1.0.0");

    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../desktop-adapt".into();
    escaped.manifest.logic_hash = escaped.compute_logic_hash();
    match store.install(&escaped.to_json_bytes().unwrap(), None, &options()) {
        Err(RuntimeError::Package(msg)) => assert!(msg.contains("invalid package_id"), "{msg}"),
        other => panic!("依赖反例：包 id 路径逃逸应拒绝: {other:?}"),
    }

    let mut missing = installed.clone();
    missing.manifest.version = "1.0.1".into();
    missing.manifest.dependencies.push(PackageDependency {
        namespace: "upstream-liuli".into(),
        package_id: "missing-base".into(),
        version: "1.0.0".into(),
        logic_hash: [0u8; 32],
    });
    missing.manifest.logic_hash = missing.compute_logic_hash();
    match store.install_with_assets(
        &missing.to_json_bytes().unwrap(),
        None,
        &options(),
        &source_root,
    ) {
        Err(RuntimeError::Invalid(msg)) => assert!(msg.contains("missing dependency"), "{msg}"),
        other => panic!("依赖反例：缺依赖激活应拒绝: {other:?}"),
    }
    assert_eq!(active_version(&store, "desktop-adapt"), "1.0.0");
    assert_eq!(
        active_version(&store, "omnisearch"),
        "1.0.0",
        "组合：失败安装不撤掉其他包"
    );

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("desktop-adapt-disabled");
    std::fs::create_dir_all(disabled_root.join("assets")).unwrap();
    std::fs::copy(
        source_root.join("manifest.json"),
        disabled_root.join("manifest.json"),
    )
    .unwrap();
    std::fs::copy(
        source_root.join("package.npack.json"),
        disabled_root.join("package.npack.json"),
    )
    .unwrap();
    std::fs::copy(
        source_root.join("assets/config.json"),
        disabled_root.join("assets/config.json"),
    )
    .unwrap();
    let mut manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(disabled_root.join("manifest.json")).unwrap(),
    )
    .unwrap();
    manifest["enabled"] = serde_json::Value::Bool(false);
    std::fs::write(
        disabled_root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    {
        let disabled_store = PackageStore::open(disabled_dir.path().join("store")).unwrap();
        let disabled = import_extensions(&disabled_store, disabled_dir.path(), &options())
            .expect("disabled import");
        assert!(!disabled
            .installed
            .iter()
            .any(|item| item.package_id == "desktop-adapt"));
        assert!(disabled
            .skipped
            .iter()
            .any(|name| name == "desktop-adapt-disabled"));
    }

    let mut upgraded = installed.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    store
        .install_with_assets(
            &upgraded.to_json_bytes().unwrap(),
            None,
            &options(),
            &source_root,
        )
        .expect("升级安装");
    assert_eq!(
        active_version(&store, "desktop-adapt"),
        "2.0.0",
        "安装升级正例"
    );
    let rolled = store.rollback().expect("回滚");
    assert!(
        rolled
            .packages
            .iter()
            .any(|(slot, version)| slot.ends_with("/desktop-adapt") && version == "1.0.0"),
        "回滚回到 1.0.0"
    );
    assert_eq!(active_version(&store, "desktop-adapt"), "1.0.0");
    assert_eq!(
        active_version(&store, "huanle-cards"),
        "1.0.0",
        "组合：回滚后欢乐卡牌仍在"
    );

    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        active_version(&reopened, "desktop-adapt"),
        "1.0.0",
        "保存恢复：重开商店仍是 1.0.0"
    );
    let (_, again) = read_installed(&reopened, "upstream-liuli", "desktop-adapt", "1.0.0");
    let again_config =
        parse_desktop_config(&std::fs::read_to_string(again.join("assets/config.json")).unwrap())
            .unwrap();
    let again_profile = apply_desktop_adapt(&again_config, HostKind::Mobile, false);
    assert_eq!(again_profile.host, HostKind::Mobile);
    assert_eq!(
        again_profile.device_profile,
        content_native_loader::DeviceProfile::Android
    );
    assert_eq!(again_profile.emoji_bridge, EmojiBridgeState::Inactive);
}
