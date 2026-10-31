//! L16 拖拽读取八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/drag-read`。拖入类型判定、
//! 默认扩展名、`extension.js` 存在性与公共前缀校验、原生 play 名单拒绝、
//! 一键完整包布局与 `.`/`_` 前缀过滤走本包配置。不 eval、不读写文件、不请求
//! 网络、不启动 Electron。

use std::path::PathBuf;

use content_native_loader::{
    complete_package_tree, default_extension_name, detect_drop, dir_entry_visible, list_dir,
    parse_drag_config, plan_archive_import, plan_single_extension_js, DragError, DropKind,
    ERR_BAD_EXTENSION_JS, ERR_NATIVE_PLAY, ERR_NOT_EXTENSION, ERR_NO_EXTENSION_JS,
    SEVENZ_UNSUPPORTED,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::ContentPackage;

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn upstream_extension() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../archive/upstream/noname-liuli/noname/extension/拖拽读取/extension.js");
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("读上游 {}", path.display()))
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
    let needle = format!("/{package_id}");
    store
        .read_active()
        .expect("active")
        .packages
        .iter()
        .find(|(slot, _)| slot.ends_with(&needle))
        .map(|(_, version)| version.clone())
        .unwrap_or_else(|| panic!("激活集含 {package_id}"))
}

/// L16 八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l16_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let upstream = upstream_extension();
    assert!(upstream.contains("检测到extension.js"));
    assert!(upstream.contains("没有extension.js，无法导入"));
    assert!(upstream.contains("禁止安装游戏原生扩展"));
    assert!(upstream.contains("你导入的不是扩展！请选择正确的文件"));
    assert!(upstream.contains("extension.js代码有错误，无法导入！"));
    assert!(upstream.contains("还不支持读取7z格式的扩展"));
    assert!(upstream.contains("一键生成完整包"));
    assert!(upstream.contains("node_modules"));

    // ---- 功能：拖入类型判定 ----
    assert_eq!(
        detect_drop("我的扩展.zip", "application/zip"),
        DropKind::Zip
    );
    assert_eq!(detect_drop("我的扩展.zip", ""), DropKind::Ignored);
    assert_eq!(detect_drop("我的扩展.7z", ""), DropKind::SevenZ);
    assert_eq!(
        detect_drop("我的扩展.7z", "application/zip"),
        DropKind::SevenZ
    );
    assert_eq!(detect_drop("archive", "application/zip"), DropKind::SevenZ);
    assert_eq!(detect_drop("extension.js", ""), DropKind::ExtensionJs);
    assert_eq!(detect_drop("readme.txt", ""), DropKind::Ignored);

    // ---- 功能：默认扩展名 ----
    assert_eq!(default_extension_name("我的扩展.zip"), "我的扩展");
    assert_eq!(default_extension_name("a.7z"), "a");
    assert_eq!(default_extension_name("a.TAR"), "a.TAR");
    assert_eq!(default_extension_name("a.zip.7z"), "a.zip");

    // ---- 功能：解压导入计划（前缀 / 原生 play / 缺 extension.js / 非扩展）----
    let native = vec![
        "coin".to_string(),
        "boss".into(),
        "wuxing".into(),
        "cardpile".into(),
    ];
    let plan = plan_archive_import(
        &["我的扩展/extension.js", "我的扩展/card/a.json"],
        "我的扩展.zip",
        "我的扩展",
        &native,
    )
    .expect("正常扩展");
    assert_eq!(plan.extension_name, "我的扩展");
    assert_eq!(plan.default_name, "我的扩展");
    assert_eq!(plan.prefix, "我的扩展/");
    assert_eq!(plan.candidates, vec!["我的扩展/extension.js".to_string()]);

    let root = plan_archive_import(&["extension.js"], "裸.zip", "裸", &native).expect("根级");
    assert_eq!(root.prefix, "");
    assert_eq!(root.default_name, "裸");

    assert_eq!(
        plan_archive_import(&["我的扩展/readme.md"], "x.zip", "x", &native),
        Err(DragError::NoExtensionJs)
    );
    assert_eq!(
        plan_archive_import(&["boss/extension.js"], "x.zip", "boss", &native),
        Err(DragError::NativePlay("boss".into()))
    );
    assert_eq!(
        plan_archive_import(&["a/extension.js", "b/extension.js"], "x.zip", "x", &native),
        Err(DragError::NotExtension)
    );
    // 更深层扩展：最短候选决定前缀，同前缀候选通过。
    let nested = plan_archive_import(
        &["outer/inner/extension.js", "outer/inner/sub/extension.js"],
        "x.zip",
        "inner",
        &native,
    )
    .expect("嵌套扩展");
    assert_eq!(nested.prefix, "outer/inner/");
    assert_eq!(nested.candidates.len(), 2);

    // ---- 功能：目录项过滤（`.`/`_` 前缀）----
    assert!(dir_entry_visible("card"));
    assert!(!dir_entry_visible(".git"));
    assert!(!dir_entry_visible("_cache"));
    assert_eq!(
        list_dir(&["card", ".git", "_x", "image"]),
        vec!["card".to_string(), "image".into()]
    );

    // ---- 规则差异：单文件 extension.js 不 eval；7z 扩展包不支持 ----
    assert_eq!(
        plan_single_extension_js("extension.js"),
        Err(DragError::BadExtensionJs)
    );
    assert_eq!(DragError::BadExtensionJs.message(), ERR_BAD_EXTENSION_JS);
    assert_eq!(DragError::SevenZUnsupported.message(), SEVENZ_UNSUPPORTED);
    assert_eq!(DragError::NoExtensionJs.message(), ERR_NO_EXTENSION_JS);
    assert_eq!(
        DragError::NativePlay("boss".into()).message(),
        format!("{ERR_NATIVE_PLAY}（boss）")
    );
    assert_eq!(DragError::NotExtension.message(), ERR_NOT_EXTENSION);

    // ---- 配置 / 一键完整包布局 ----
    let source_root = repo_extensions_root().join("drag-read-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text =
        std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");
    let npack_text = String::from_utf8(npack_bytes.clone()).expect("npack utf-8");
    assert!(npack_text.contains("拖拽读取"));
    assert!(
        npack_text.contains("\"package_id\":\"drag-read\"")
            || npack_text.contains("\"package_id\": \"drag-read\"")
    );

    let config = parse_drag_config(&config_text).expect("配置");
    assert_eq!(config.source, "noname-liuli");
    assert_eq!(config.upstream_version, "1.81");
    assert_eq!(config.author, "诗笺");
    assert_eq!(config.native_plays, native);
    assert_eq!(config.complete_folders.len(), 7);
    assert_eq!(config.complete_files.len(), 4);
    assert_eq!(config.audio_subfolders.len(), 6);
    assert_eq!(config.image_subfolders.len(), 6);
    assert_eq!(config.extension_subfolders.len(), 4);
    assert_eq!(config.node_modules_subfolders.len(), 3);

    let tree = complete_package_tree(&config);
    for expected in [
        "app.html",
        "index.html",
        "LICENSE",
        "README.md",
        "card/",
        "character/",
        "font/",
        "game/",
        "layout/",
        "mode/",
        "theme/",
        "cache/",
        "audio/",
        "audio/voice/",
        "image/",
        "image/splash/",
        "extension/",
        "extension/wuxing/",
        "node_modules/",
        "node_modules/ws/",
    ] {
        assert!(tree.iter().any(|t| t == expected), "完整包应含 {expected}");
    }
    assert!(
        !tree.iter().any(|t| t == "image/skin/"),
        "上游注释掉的 skin 目录不应导出"
    );
    assert!(parse_drag_config(r#"{"source":""}"#).is_err());
    assert!(parse_drag_config("not json").is_err());

    // ---- 安装：真实 import ----
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "drag-read" && item.namespace == "upstream-liuli")
        .expect("拖拽读取已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes)
    );

    // ---- 组合：同一次导入仍装其他包，纯函数不改内容锁 ----
    for (package_id, namespace, folder_name) in [
        (
            "update-guard",
            "upstream-liuli",
            "update-guard-upstream-liuli",
        ),
        ("coin", "upstream-liuli", "coin-upstream-liuli"),
        (
            "sj-settings",
            "upstream-liuli",
            "sj-settings-upstream-liuli",
        ),
        (
            "shousha-full",
            "upstream-liuli",
            "shousha-full-upstream-liuli",
        ),
    ] {
        let kept = report
            .installed
            .iter()
            .find(|item| item.package_id == package_id && item.namespace == namespace)
            .unwrap_or_else(|| panic!("组合：{namespace}/{package_id} 仍在"));
        assert_eq!(kept.version, "1.0.0");
        let bytes = std::fs::read(
            repo_extensions_root()
                .join(folder_name)
                .join("package.npack.json"),
        )
        .unwrap();
        assert_eq!(kept.content_hash, content_runtime::content_hash_hex(&bytes));
    }
    assert!(!report
        .installed
        .iter()
        .any(|item| item.package_id == "如真似幻"));

    let (installed, installed_dir) = read_installed(&store, "upstream-liuli", "drag-read", "1.0.0");
    assert!(installed.manifest.dependencies.is_empty());
    assert!(installed.character_index.is_empty());
    assert!(installed.card_index.is_empty());
    assert!(installed.skill_index.is_empty());
    let lock_before = store.current_lock().expect("lock").lock_hash;
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());
    assert!(stored.contains("noname/extension/拖拽读取/extension.js"));
    assert!(stored.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
    assert!(!stored.contains("eval("));
    assert!(!stored.contains("fetch("));
    assert!(!stored.contains("fs."));
    let config = parse_drag_config(&stored).expect("商店配置");
    assert_eq!(
        config.upstream_commit,
        "8485dbb29ff16f9b258a766b1a17ef75f883aade"
    );
    // 纯函数不触碰商店：布局与导入计划前后内容锁不变。
    let _ = complete_package_tree(&config);
    let _ = plan_archive_import(
        &["我的扩展/extension.js"],
        "x.zip",
        "我的扩展",
        &config.native_plays,
    );
    assert_eq!(store.current_lock().expect("lock").lock_hash, lock_before);

    // ---- 保存恢复：重开商店仍 1.0.0 ----
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(active_version(&reopened, "drag-read"), "1.0.0");
    assert_eq!(
        reopened.current_lock().expect("lock").lock_hash,
        lock_before
    );

    // ---- 依赖：缺失版本 / 逃逸 id / 非法依赖 ----
    match store.activate_version("upstream-liuli", "drag-read", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../drag-read".into();
    escaped.manifest.logic_hash = escaped.compute_logic_hash();
    match store.install(&escaped.to_json_bytes().unwrap(), None, &options()) {
        Err(RuntimeError::Package(msg)) => assert!(msg.contains("invalid package_id"), "{msg}"),
        other => panic!("包 id 逃逸应拒绝: {other:?}"),
    }

    // ---- 安装升级：2.0.0 覆盖 + rollback 回 1.0.0 ----
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
        .expect("安装 2.0.0");
    assert_eq!(active_version(&store, "drag-read"), "2.0.0");
    store.rollback().expect("rollback");
    assert_eq!(active_version(&store, "drag-read"), "1.0.0");
}
