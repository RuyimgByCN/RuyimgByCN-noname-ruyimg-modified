//! L18 更新保护八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/update-guard`。目录识别、
//! 更新源、版本比较、断点名单和检查按钮走本包配置。不请求网络，不写盘，
//! 不 `eval`，不睡眠。不安装如真似幻。

use std::path::PathBuf;

use content_native_loader::{
    apply_update_guard_line, boot_link, can_update, classify_body, compare_update, dedup_broken,
    extension_urls, forced_enables, format_catch, join_changelog, next_error_count, on_remove_link,
    parse_size, parse_update_guard_config, precontent_menu, press_asset_button, press_game_button,
    retry_updates, scan_folders, select_extension_link, select_general_link, settle_broken,
    Attempt, BodyClass, CatchKind, FolderIn, GuardError, RetryClass, UpdatePrompt, CODING_URL,
    FASTGIT_URL, GITHUB_URL, URC_URL,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency};

const DEFAULT_LINE: &str = "link:URC | url:https://unitedrhythmized.club/libccy/noname | remove:coding | scan:甲,丙 | size:1.00KB | cmp:prompt | log:甲；丙 | broken:- | would:0 | game:当前版本已是最新 | asset:当前素材已是最新 | asset_flag:1 | updating:0 | select:github | xuanwu:0 | body:login | retry:manual | lock:十周年UI,手杀ui | can:1 | eval:0 | fetch:0 | write:0 | reload:0 | sleep:0 | dom:0";

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn upstream_extension() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../archive/upstream/noname-liuli/noname/extension/更新保护/extension.js");
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

fn active_version(store: &PackageStore, namespace: &str, package_id: &str) -> String {
    let needle = format!("{namespace}/{package_id}");
    store
        .read_active()
        .expect("active")
        .packages
        .iter()
        .find(|(slot, _)| slot.ends_with(&needle))
        .map(|(_, version)| version.clone())
        .unwrap_or_else(|| panic!("激活集含 {needle}"))
}

fn folder(name: &str, files: &[&str]) -> FolderIn {
    FolderIn {
        name: name.into(),
        files: files.iter().map(|file| (*file).to_string()).collect(),
    }
}

/// L18 八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l18_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let upstream = upstream_extension();
    assert!(upstream.contains("alert('当前版本已是最新')"));
    assert!(upstream.contains("alert('当前素材已是最新')"));
    assert!(upstream.contains("function canUpdate()"));
    assert!(upstream.contains("return true;"));
    assert!(upstream.contains("var plays=['coin','boss','wuxing','cardpile']"));
    assert!(upstream.contains("es=='SJ Settings'"));
    assert!(upstream.contains(URC_URL));
    assert!(upstream.contains(FASTGIT_URL));
    assert!(upstream.contains("此更新源已弃用，请不要再使用此更新源"));
    assert!(upstream.contains("达到最大重试次数(5次), 请重试"));
    assert!(can_update());

    let urls = extension_urls();
    assert_eq!(urls.len(), 4);
    assert_eq!(urls[0].key, "coding");
    assert_eq!(urls[0].url, CODING_URL);
    assert_eq!(urls[1].url, GITHUB_URL);
    assert_eq!(urls[2].url, FASTGIT_URL);
    assert_eq!(urls[3].key, "URC");
    assert_eq!(urls[3].url, URC_URL);

    let fresh = boot_link("", CODING_URL);
    assert_eq!(fresh.link, "URC");
    assert_eq!(fresh.url, URC_URL);
    assert!(fresh.url_assigned);
    let kept_url = boot_link("github", CODING_URL);
    assert_eq!(kept_url.link, "github");
    assert_eq!(kept_url.url, CODING_URL);
    assert!(!kept_url.url_assigned);
    let removed = on_remove_link();
    assert_eq!(removed.link, "coding");
    assert_eq!(removed.url, CODING_URL);

    let switched = select_extension_link("URC", "github");
    assert!(switched.accepted);
    assert!(switched.cache_cleared);
    assert_eq!(switched.link, "github");
    assert_eq!(switched.url, GITHUB_URL);
    assert!(switched.alert.is_none());
    let same = select_extension_link("URC", "URC");
    assert!(same.accepted);
    assert!(!same.cache_cleared);
    assert_eq!(same.link, "URC");
    let deprecated = select_extension_link("URC", "xuanwu");
    assert!(!deprecated.accepted);
    assert_eq!(deprecated.link, "URC");
    assert_eq!(
        deprecated.alert.as_deref(),
        Some("此更新源已弃用，请不要再使用此更新源")
    );
    let missing = select_extension_link("URC", "nope");
    assert!(!missing.accepted);
    assert_eq!(missing.alert.as_deref(), Some("选择的更新源(nope)不存在"));
    let general = select_general_link("URC", "xuanwu");
    assert!(!general.accepted);
    assert!(!general.cache_cleared);
    assert_eq!(general.alert.as_deref(), Some("选择的更新源(xuanwu)不存在"));
    assert_ne!(
        general.alert.as_deref(),
        Some("此更新源已弃用，请不要再使用此更新源")
    );

    let scanned = scan_folders(
        true,
        &["已有".into()],
        &[
            folder("coin", &["extension.js"]),
            folder("coin2", &["extension.js"]),
            folder("甲", &["extension.js"]),
            folder("SJ Settings", &["extension.js"]),
            folder("乙", &["readme.md"]),
            folder("丙", &["extension.js"]),
            folder("已有", &["extension.js"]),
        ],
    );
    assert_eq!(
        scanned.added,
        vec!["coin2".to_string(), "甲".into(), "丙".into()]
    );
    assert_eq!(scanned.message, "已识别扩展：coin2、甲、丙");
    let skipped = scan_folders(false, &[], &[folder("甲", &["extension.js"])]);
    assert!(skipped.added.is_empty());
    assert!(skipped.message.is_empty());

    assert_eq!(parse_size(0.0), "0.00B");
    assert_eq!(parse_size(1023.0), "1023.00B");
    assert_eq!(parse_size(1024.0), "1.00KB");
    assert_eq!(parse_size(1536.0), "1.50KB");
    assert_eq!(parse_size(1_048_576.0), "1.00MB");
    assert_eq!(parse_size(1_073_741_824.0), "1.00GB");

    assert_eq!(compare_update("1.2", "1.2"), UpdatePrompt::Same);
    assert_eq!(compare_update("1.2", "1.10"), UpdatePrompt::Prompt);
    assert_eq!(compare_update("1.10", "1.2"), UpdatePrompt::LocalNewer);
    assert_eq!(compare_update("1.2.0", "1.2"), UpdatePrompt::Prompt);
    assert_eq!(compare_update("1.a", "1.1"), UpdatePrompt::Prompt);
    assert_eq!(compare_update("2", "1.9"), UpdatePrompt::LocalNewer);
    assert_eq!(join_changelog(&[]), "");
    assert_eq!(
        join_changelog(&["甲".into(), "https://a".into(), "丙".into()]),
        "甲；丙"
    );
    assert_eq!(
        join_changelog(&["https://a".into(), "乙".into()]),
        "https://a；乙"
    );

    assert_eq!(
        dedup_broken(&["a".into(), "b".into(), "a".into()]),
        vec!["a".to_string(), "b".into()]
    );
    let declined = settle_broken(Some(&["a".into(), "b".into(), "a".into()]), false);
    assert!(declined.files.is_empty());
    assert!(!declined.would_redownload);
    assert!(!declined.download_called);
    assert!(!declined.reload_called);
    let accepted = settle_broken(Some(&["a".into(), "b".into(), "a".into()]), true);
    assert_eq!(accepted.files, vec!["a".to_string(), "b".into()]);
    assert!(accepted.would_redownload);
    assert!(!accepted.download_called);
    assert!(!accepted.reload_called);
    let absent = settle_broken(None, true);
    assert!(absent.files.is_empty());
    assert!(!absent.would_redownload);

    let game = press_game_button("检查游戏更新");
    assert!(game.acted);
    assert!(!game.updating);
    assert!(!game.unwanted);
    assert_eq!(game.alert, "当前版本已是最新");
    assert!(!game.fetch_called && !game.write_called && !game.reload_called);
    assert!(!game.sleep_called && !game.eval_called);
    let ignored = press_game_button("别的");
    assert!(!ignored.acted);
    assert!(ignored.alert.is_empty());
    let asset = press_asset_button("检查素材更新");
    assert!(asset.acted);
    assert!(asset.updating_for_asset);
    assert!(!asset.updating);
    assert!(!asset.unwanted);
    assert_eq!(asset.alert, "当前素材已是最新");
    assert!(!asset.fetch_called && !asset.write_called && !asset.eval_called);
    assert!(!press_asset_button("别的").acted);

    let mut errors = 0;
    for expected in [1, 2, 3, 4, 5, 0] {
        errors = next_error_count(errors);
        assert_eq!(errors, expected);
    }
    let (alert, errors) = format_catch(
        &CatchKind::Object {
            url: format!("{CODING_URL}/master/game/update.js"),
            error: "404".into(),
            progress: false,
            message: "失败".into(),
        },
        CODING_URL,
        0,
    );
    assert_eq!(
        alert.as_deref(),
        Some("更新源:Coding\n网络请求目标：game/update.js\n状态消息或状态码：404\n提示:失败")
    );
    assert_eq!(errors, 1);
    let (progress, _) = format_catch(
        &CatchKind::Object {
            url: format!("{CODING_URL}/master/game/update.js"),
            error: "404".into(),
            progress: true,
            message: "失败".into(),
        },
        CODING_URL,
        0,
    );
    assert_eq!(
        progress.as_deref(),
        Some("更新源:Coding\n网络请求目标：game/update.js\n提示:失败")
    );
    let (generic, _) = format_catch(
        &CatchKind::Object {
            url: format!("{URC_URL}/master/game/update.js"),
            error: "404".into(),
            progress: false,
            message: "失败".into(),
        },
        URC_URL,
        4,
    );
    let generic = generic.expect("URC 提示");
    assert!(generic.starts_with("网络请求目标："), "{generic}");
    assert!(!generic.contains("更新源:"), "{generic}");
    let (timeout, count) = format_catch(&CatchKind::Abort, CODING_URL, 5);
    assert_eq!(timeout.as_deref(), Some("网络连接超时"));
    assert_eq!(count, 0);
    let (failed, _) = format_catch(&CatchKind::FailedFetch, CODING_URL, 0);
    assert_eq!(failed.as_deref(), Some("网络请求失败"));
    let (text, _) = format_catch(&CatchKind::Text("请重试".into()), CODING_URL, 0);
    assert_eq!(text.as_deref(), Some("请重试"));
    let (other, count) = format_catch(&CatchKind::Other, CODING_URL, 0);
    assert!(other.is_none());
    assert_eq!(count, 1);

    let (class, eval_called) = classify_body(r#"{"msg":{"user_not_login":"用户未登录"}}"#);
    assert_eq!(class, BodyClass::UserNotLogin);
    assert!(!eval_called);
    let (class, eval_called) = classify_body("alert(1)");
    assert_eq!(class, BodyClass::EvalRefused);
    assert!(!eval_called);
    assert_eq!(
        classify_body(r#"{"version":"1"}"#).0,
        BodyClass::EvalRefused
    );
    assert_eq!(classify_body(r#"{"msg":null}"#).0, BodyClass::EvalRefused);
    assert_eq!(classify_body(r#"{"msg":{}}"#).0, BodyClass::EvalRefused);

    let (class, message, eval_called) = retry_updates(false, true, &[Attempt::Login]);
    assert_eq!(class, RetryClass::Manual);
    assert_eq!(message, "此版本不支持游戏内更新，请手动更新");
    assert!(!eval_called);
    let (class, message, _) = retry_updates(true, true, &[Attempt::Login]);
    assert_eq!(class, RetryClass::Ready);
    assert_eq!(message, "ready");
    let (class, message, _) = retry_updates(true, false, &[Attempt::Fail; 5]);
    assert_eq!(class, RetryClass::MaxRetries);
    assert_eq!(message, "达到最大重试次数(5次), 请重试");
    let (class, message, _) = retry_updates(true, false, &[Attempt::Fail, Attempt::Login]);
    assert_eq!(class, RetryClass::Login);
    assert_eq!(message, "user_not_login");
    let (class, _, _) = retry_updates(true, false, &[Attempt::Fail, Attempt::Ready]);
    assert_eq!(class, RetryClass::Ready);
    let (class, message, _) = retry_updates(true, false, &[]);
    assert_eq!(class, RetryClass::Other);
    assert_eq!(message, "遇到其他错误, 请重试");

    assert_eq!(
        forced_enables(true, Some(&["十周年UI".into(), "手杀ui".into()])),
        vec!["十周年UI".to_string(), "手杀ui".into()]
    );
    assert!(forced_enables(false, Some(&["十周年UI".into()])).is_empty());
    assert!(forced_enables(true, None).is_empty());
    let menu = precontent_menu(true);
    assert!(menu.delete_removed && menu.enable_removed && menu.lock_enable);
    let unlocked = precontent_menu(false);
    assert!(unlocked.delete_removed && unlocked.enable_removed && !unlocked.lock_enable);

    let source_root = repo_extensions_root().join("update-guard-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text =
        std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");
    let npack_text = String::from_utf8(npack_bytes.clone()).expect("npack utf-8");
    assert!(npack_text.contains("更新保护"));
    assert!(
        npack_text.contains("\"package_id\":\"update-guard\"")
            || npack_text.contains("\"package_id\": \"update-guard\"")
    );

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "update-guard" && item.namespace == "upstream-liuli")
        .expect("更新保护已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes)
    );
    for (package_id, namespace, folder_name) in [
        ("coin", "upstream-liuli", "coin-upstream-liuli"),
        ("coin", "upstream-noname", "coin-upstream-noname"),
        (
            "elo-sanction",
            "upstream-liuli",
            "elo-sanction-upstream-liuli",
        ),
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
        (
            "douzhuan-xingyi",
            "upstream-liuli",
            "douzhuan-xingyi-upstream-liuli",
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

    let (installed, installed_dir) =
        read_installed(&store, "upstream-liuli", "update-guard", "1.0.0");
    assert!(installed.manifest.dependencies.is_empty());
    assert!(installed.character_index.is_empty());
    assert!(installed.card_index.is_empty());
    assert!(installed.skill_index.is_empty());
    let lock_before = store.current_lock().expect("lock").lock_hash;
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());
    assert!(stored.contains("\"source\": \"noname-liuli\""));
    assert!(stored.contains("noname/extension/更新保护/extension.js"));
    assert!(stored.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
    assert!(stored.contains("\"upstream_version\": \"1.5\""));
    assert!(!stored.contains("eval("));
    assert!(!stored.contains("game.reload"));
    assert!(!stored.contains("fetch("));
    let config = parse_update_guard_config(&stored).expect("配置");
    assert_eq!(config.source, "noname-liuli");
    assert_eq!(
        config.upstream_commit,
        "8485dbb29ff16f9b258a766b1a17ef75f883aade"
    );
    assert_eq!(config.upstream_version, "1.5");
    assert!(config.cancel_forbid);
    assert!(config.auto_add_exts);
    assert!(config.update_link.is_empty());
    assert!(config.broken_file.is_empty());
    assert_eq!(
        apply_update_guard_line(&stored, "{}").expect("默认行"),
        DEFAULT_LINE
    );
    let refused = apply_update_guard_line(&stored, r#"{"body":"alert(1)"}"#).unwrap();
    assert!(refused.contains("body:refused |"));
    assert!(refused.contains("eval:0 | fetch:0 | write:0 | reload:0 |"));
    let confirmed = apply_update_guard_line(&stored, r#"{"confirmBroken":true}"#).unwrap();
    assert!(confirmed.contains("broken:a,b | would:1 |"));
    assert!(confirmed.contains("fetch:0 | write:0 | reload:0 |"));
    let closed = stored.replace("\"autoAddExts\": true", "\"autoAddExts\": false");
    assert!(apply_update_guard_line(&closed, "{}")
        .unwrap()
        .contains("scan:- |"));
    let open = stored.replace("\"cancelForbid\": true", "\"cancelForbid\": false");
    assert!(apply_update_guard_line(&open, "{}")
        .unwrap()
        .contains("lock:- |"));
    assert!(matches!(
        apply_update_guard_line(
            &stored.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""),
            "{}"
        ),
        Err(GuardError::BadConfig(_))
    ));

    let asset_names: Vec<_> = std::fs::read_dir(installed_dir.join("assets"))
        .unwrap()
        .map(|item| item.unwrap().file_name())
        .collect();
    assert_eq!(asset_names, vec![std::ffi::OsString::from("config.json")]);
    assert_eq!(store.current_lock().expect("lock").lock_hash, lock_before);

    match store.activate_version("upstream-liuli", "update-guard", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../update-guard".into();
    escaped.manifest.logic_hash = escaped.compute_logic_hash();
    match store.install(&escaped.to_json_bytes().unwrap(), None, &options()) {
        Err(RuntimeError::Package(msg)) => assert!(msg.contains("invalid package_id"), "{msg}"),
        other => panic!("包 id 逃逸应拒绝: {other:?}"),
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
        other => panic!("缺依赖应拒绝: {other:?}"),
    }
    assert_eq!(
        active_version(&store, "upstream-liuli", "update-guard"),
        "1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "douzhuan-xingyi"),
        "1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "shousha-full"),
        "1.0.0"
    );

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("update-guard-disabled");
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
            .any(|item| item.package_id == "update-guard"));
        assert!(disabled
            .skipped
            .iter()
            .any(|name| name == "update-guard-disabled"));
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
        .expect("升级");
    assert_eq!(
        active_version(&store, "upstream-liuli", "update-guard"),
        "2.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "douzhuan-xingyi"),
        "1.0.0"
    );
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0");
    let rolled = store.rollback().expect("回滚");
    assert!(rolled.packages.iter().any(|(slot, version)| slot
        .ends_with("upstream-liuli/update-guard")
        && version == "1.0.0"));
    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        active_version(&reopened, "upstream-liuli", "update-guard"),
        "1.0.0"
    );
    let (_, kept) = read_installed(&reopened, "upstream-liuli", "update-guard", "1.0.0");
    let kept_text = std::fs::read_to_string(kept.join("assets/config.json")).unwrap();
    assert_eq!(
        parse_update_guard_config(&kept_text).unwrap().source,
        "noname-liuli"
    );
    assert_eq!(
        apply_update_guard_line(&kept_text, "{}").unwrap(),
        DEFAULT_LINE
    );
    assert_eq!(
        reopened.current_lock().expect("lock").lock_hash,
        lock_before
    );
}
