//! L02 SJ Settings 八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/sj-settings`。启动名单、
//! 配置导入、版本比较、目录标记和错误文本走本包配置。不请求网络，不执行
//! `update.js`，不打开 PDF，不启动 eruda，也不调用 `game.reload` 或 `game.loop`。

use std::collections::BTreeMap;
use std::path::PathBuf;

use content_native_loader::{
    apply_external, apply_sj_commands, boot, catalog_mark, choose_download, classify_fault,
    compare_version, decode_text, decode_uri, encode_text, eval_update_script, filter_downloads,
    finish_download, format_window_error, import_config, load_remote_catalog, open_asset_file,
    parse_sj_config, plan_files, request_console, Actor, BootInput, CatalogEntry, CatalogMark,
    ExternalKind, FaultAction, Frame, GameEvent, HttpCode, ImportInput, InstalledPack, SelectInput,
    SjError, TransferFault, VersionOrder, WindowError,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency};
use serde_json::Value;

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

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|text| (*text).to_string()).collect()
}

fn store_map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

fn boot_of(
    storage: BTreeMap<String, String>,
    extensions: &[&str],
    enables: &[(&str, bool)],
    asset: Option<&str>,
) -> Result<content_native_loader::BootOutput, SjError> {
    let mut enabled = BTreeMap::new();
    for (name, on) in enables {
        enabled.insert((*name).to_string(), *on);
    }
    boot(&BootInput {
        storage,
        extensions: names(extensions),
        enables: enabled,
        asset_version: asset.map(str::to_string),
    })
}

fn run_import(
    payload: &str,
    db: bool,
    keep: bool,
    extensions: &[&str],
    storage: &[(&str, &str)],
    prefix: &str,
) -> Result<content_native_loader::ImportOutput, SjError> {
    import_config(&ImportInput {
        payload: payload.to_string(),
        db,
        keep_existing: keep,
        extensions: names(extensions),
        storage: store_map(storage),
        config_prefix: prefix.to_string(),
        config: BTreeMap::new(),
        data: BTreeMap::new(),
    })
}

fn encoded(text: &str) -> String {
    encode_text(text).expect("encode")
}

fn entry(version: Option<&str>, noname: Option<&str>, files: Option<Vec<&str>>) -> CatalogEntry {
    CatalogEntry {
        version: version.map(str::to_string),
        noname_version: noname.map(str::to_string),
        files: files.map(|list| list.into_iter().map(str::to_string).collect()),
    }
}

fn window(
    message: &str,
    source: Option<&str>,
    event: Option<GameEvent>,
    translate: &[(&str, &str)],
    stack: Option<&str>,
) -> WindowError {
    let mut map = BTreeMap::new();
    for (key, value) in translate {
        map.insert((*key).to_string(), (*value).to_string());
    }
    WindowError {
        message: Some(message.to_string()),
        source: source.map(str::to_string),
        line: Some("1".into()),
        column: Some("2".into()),
        stack: stack.map(str::to_string),
        event,
        translate: map,
    }
}

fn frame(name: &str, step: &str, parent: Option<Frame>) -> Frame {
    Frame {
        name: name.into(),
        step: step.into(),
        parent: parent.map(Box::new),
    }
}

fn event_of(
    frame: Frame,
    player: Option<Actor>,
    skill: Option<&str>,
    card: Option<&str>,
) -> GameEvent {
    GameEvent {
        frame,
        player,
        target: None,
        source: None,
        skill: skill.map(str::to_string),
        card: card.map(str::to_string),
    }
}

/// L02 八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l02_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let source_root = repo_extensions_root().join("sj-settings-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text =
        std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");
    let npack_text = String::from_utf8(npack_bytes.clone()).expect("npack utf-8");
    assert!(npack_text.contains("SJ Settings"));
    assert!(
        npack_text.contains("\"package_id\":\"sj-settings\"")
            || npack_text.contains("\"package_id\": \"sj-settings\"")
    );

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "sj-settings" && item.namespace == "upstream-liuli")
        .expect("SJ Settings 已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes),
        "商店 content_hash 来自本包 npack"
    );
    let liuli_coin = report
        .installed
        .iter()
        .find(|item| item.package_id == "coin" && item.namespace == "upstream-liuli")
        .expect("组合：liuli 金币仍在");
    assert_eq!(liuli_coin.version, "1.0.0");
    let l05_npack =
        std::fs::read(repo_extensions_root().join("coin-upstream-liuli/package.npack.json"))
            .unwrap();
    assert_eq!(
        liuli_coin.content_hash,
        content_runtime::content_hash_hex(&l05_npack),
        "不改写 L05 npack"
    );
    let noname_coin = report
        .installed
        .iter()
        .find(|item| item.package_id == "coin" && item.namespace == "upstream-noname")
        .expect("组合：noname 金币仍在");
    assert_eq!(noname_coin.version, "1.0.0");
    let u04_npack =
        std::fs::read(repo_extensions_root().join("coin-upstream-noname/package.npack.json"))
            .unwrap();
    assert_eq!(
        noname_coin.content_hash,
        content_runtime::content_hash_hex(&u04_npack),
        "不改写 U04 npack"
    );
    let elo = report
        .installed
        .iter()
        .find(|item| item.package_id == "elo-sanction" && item.namespace == "upstream-liuli")
        .expect("组合：段位制裁仍在");
    assert_eq!(elo.version, "1.0.0");
    let l22_npack = std::fs::read(
        repo_extensions_root().join("elo-sanction-upstream-liuli/package.npack.json"),
    )
    .unwrap();
    assert_eq!(
        elo.content_hash,
        content_runtime::content_hash_hex(&l22_npack),
        "不改写 L22 npack"
    );
    assert!(
        !report
            .installed
            .iter()
            .any(|item| item.package_id == "如真似幻"),
        "不把如真似幻计入"
    );

    let (installed, installed_dir) =
        read_installed(&store, "upstream-liuli", "sj-settings", "1.0.0");
    assert!(
        installed.manifest.dependencies.is_empty(),
        "依赖：不声明外部扩展"
    );
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());
    assert!(stored.contains("\"source\": \"noname-liuli\""));
    assert!(stored.contains("noname/extension/SJ Settings/extension.js"));
    assert!(stored.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
    assert!(stored.contains("\"upstream_version\": \"1.4204\""));
    assert!(stored.contains("\"openTools\": false"));
    let config = parse_sj_config(&stored).expect("配置");
    assert_eq!(config.source, "noname-liuli");
    assert_eq!(config.upstream_version, "1.4204");
    assert!(!config.open_tools);
    assert!(
        matches!(parse_sj_config(&stored.replace("\"noname-liuli\"", "\"\"")), Err(SjError::BadConfig(msg)) if msg.contains("source"))
    );
    assert!(matches!(
        parse_sj_config(&stored.replace("noname-liuli", "bad token")),
        Err(SjError::BadConfig(_))
    ));
    assert!(
        matches!(parse_sj_config(&stored.replace("\"openTools\": false", "\"openTools\": \"yes\"")), Err(SjError::BadConfig(msg)) if msg.contains("openTools"))
    );
    assert!(
        matches!(parse_sj_config(&stored.replace("\"1.4204\"", "\"\"")), Err(SjError::BadConfig(msg)) if msg.contains("upstream_version"))
    );
    let lock_before = store.current_lock().expect("lock").lock_hash;

    let absent = boot_of(BTreeMap::new(), &[], &[], None).unwrap();
    assert_eq!(absent.extensions, names(&["SJ Settings"]));
    assert_eq!(absent.enables.get("SJ Settings"), Some(&true));
    assert_eq!(absent.asset_version, "无");
    assert!(!absent.android_key_removed);

    let mut blank_key = BTreeMap::new();
    blank_key.insert("noname_android_extension".into(), String::new());
    let blank = boot_of(blank_key, &[], &[], Some("")).unwrap();
    assert_eq!(blank.extensions, names(&["SJ Settings"]));
    assert_eq!(
        blank.storage.get("noname_android_extension"),
        Some(&String::new())
    );
    assert!(!blank.android_key_removed);
    assert_eq!(blank.asset_version, "无");

    let mut kept_asset = BTreeMap::new();
    kept_asset.insert("noname_android_extension".into(), "[\"甲\"]".into());
    let kept = boot_of(
        kept_asset,
        &["SJ Settings"],
        &[("SJ Settings", true), ("甲", false)],
        Some("0"),
    )
    .unwrap();
    assert_eq!(kept.extensions, names(&["SJ Settings", "甲"]));
    assert!(kept.android_key_removed);
    assert!(!kept.storage.contains_key("noname_android_extension"));
    assert_eq!(kept.enables.get("甲"), Some(&true));
    assert_eq!(kept.enables.get("SJ Settings"), Some(&true));
    assert_eq!(kept.asset_version, "0");
    let only_named = boot_of(
        store_map(&[("noname_android_extension", "[\"甲\"]")]),
        &[],
        &[],
        Some("无"),
    )
    .unwrap();
    assert_eq!(only_named.extensions, names(&["甲"]));
    assert!(!only_named
        .extensions
        .iter()
        .any(|name| name == "SJ Settings"));
    let empty_list = boot_of(
        store_map(&[("noname_android_extension", "[]")]),
        &[],
        &[],
        None,
    )
    .unwrap();
    assert!(empty_list.extensions.is_empty());
    assert!(empty_list.android_key_removed);
    assert_eq!(empty_list.asset_version, "无");
    let dup = boot_of(
        store_map(&[("noname_android_extension", "[\"甲\",\"甲\"]")]),
        &[],
        &[],
        Some("1.2"),
    )
    .unwrap();
    assert_eq!(dup.extensions, names(&["甲"]));
    assert_eq!(dup.asset_version, "1.2");

    assert!(
        matches!(
            boot_of(
                store_map(&[("noname_android_extension", "[")]),
                &["甲"],
                &[],
                Some("9")
            ),
            Err(SjError::BadPayload(_))
        ),
        "解析失败不返回已删除键的结果"
    );
    assert!(matches!(
        boot_of(
            store_map(&[("noname_android_extension", "null")]),
            &[],
            &[],
            None
        ),
        Err(SjError::BadPayload(_))
    ));
    assert!(matches!(
        boot_of(
            store_map(&[("noname_android_extension", "[1]")]),
            &[],
            &[],
            None
        ),
        Err(SjError::BadPayload(_))
    ));
    assert!(matches!(
        boot_of(
            store_map(&[("noname_android_extension", "\"SJ Settings\"")]),
            &[],
            &[],
            None
        ),
        Err(SjError::BadPayload(_))
    ));

    assert_eq!(encode_text(r#"{"a":1}"#).unwrap(), "eyJhIjoxfQ==");
    assert_eq!(decode_text("eyJhIjoxfQ==").unwrap(), r#"{"a":1}"#);
    let chinese = r#"{"config":{"extensions":["甲","SJ Settings"],"asset_version":"无"},"data":{"note":"诗笺"}}"#;
    assert_eq!(
        encode_text(chinese).unwrap(),
        "eyJjb25maWciOnsiZXh0ZW5zaW9ucyI6WyLnlLIiLCJTSiBTZXR0aW5ncyJdLCJhc3NldF92ZXJzaW9uIjoi5pegIn0sImRhdGEiOnsibm90ZSI6Iuivl+esuiJ9fQ=="
    );
    assert_eq!(
        decode_text(&encode_text(chinese).unwrap()).unwrap(),
        chinese
    );
    assert!(encode_text("😀").is_err(), "非 BMP 不伪装成上游编码");
    assert!(decode_text("!!!!").is_err());
    assert!(decode_text("abc").is_err());

    let empty = run_import("", true, true, &["甲"], &[("temp", "1")], "noname_").unwrap();
    assert!(!empty.applied);
    assert!(!empty.reload_requested);
    assert!(!empty.restarted);
    assert!(!empty.cleared);
    assert_eq!(empty.extensions, names(&["甲"]));
    assert_eq!(empty.storage.get("temp"), Some(&"1".to_string()));

    let db_body = r#"{"config":{"extensions":["B"],"asset_version":"9"},"data":{"note":"诗笺"}}"#;
    let kept_ext = run_import(
        &encoded(db_body),
        true,
        true,
        &["A", "SJ Settings"],
        &[("temp", "1")],
        "noname_",
    )
    .unwrap();
    assert!(kept_ext.applied);
    assert!(kept_ext.reload_requested);
    assert!(!kept_ext.restarted);
    assert!(!kept_ext.cleared);
    assert_eq!(kept_ext.extensions, names(&["B", "A", "SJ Settings"]));
    assert_eq!(
        kept_ext.storage.get("temp"),
        Some(&"1".to_string()),
        "数据库模式不清存储"
    );
    assert_eq!(
        kept_ext.config.get("asset_version").and_then(Value::as_str),
        Some("9")
    );
    assert_eq!(
        kept_ext.data.get("note").and_then(Value::as_str),
        Some("诗笺")
    );
    let dropped_ext = run_import(
        &encoded(db_body),
        true,
        false,
        &["A", "SJ Settings"],
        &[("temp", "1")],
        "noname_",
    )
    .unwrap();
    assert_eq!(dropped_ext.extensions, names(&["B", "SJ Settings"]));
    assert_eq!(dropped_ext.storage.get("temp"), Some(&"1".to_string()));
    let no_key = r#"{"config":{"asset_version":"1"},"data":{}}"#;
    assert_eq!(
        run_import(&encoded(no_key), true, false, &["A", "B"], &[], "noname_")
            .unwrap()
            .extensions,
        names(&["A", "B", "SJ Settings"])
    );
    assert_eq!(
        run_import(&encoded(no_key), true, true, &["A", "B"], &[], "noname_")
            .unwrap()
            .extensions,
        names(&["A", "B"])
    );
    let dups = r#"{"config":{"extensions":["B","B","A"]},"data":{}}"#;
    assert_eq!(
        run_import(&encoded(dups), true, true, &["A", "C"], &[], "noname_")
            .unwrap()
            .extensions,
        names(&["B", "A", "C"])
    );

    let flat = r#"{"z":"1"}"#;
    let nodb_keep = run_import(
        &encoded(flat),
        false,
        true,
        &["甲"],
        &[("temp", "1")],
        "noname_",
    )
    .unwrap();
    let nodb_drop = run_import(
        &encoded(flat),
        false,
        false,
        &["甲"],
        &[("temp", "1")],
        "noname_",
    )
    .unwrap();
    assert_eq!(
        nodb_keep.extensions,
        names(&["甲"]),
        "非数据库模式忽略保留确认"
    );
    assert_eq!(nodb_drop.extensions, nodb_keep.extensions);
    assert!(nodb_keep.cleared);
    assert!(nodb_keep.reload_requested);
    assert!(!nodb_keep.restarted);
    assert!(!nodb_keep.storage.contains_key("temp"));
    assert_eq!(nodb_keep.storage.get("z"), Some(&"1".to_string()));

    let typed = serde_json::json!({"s": "ab", "b": true, "n": null, "i": 12, "f": 1.5, "a": [1, "x", null], "o": {"k": 1}});
    let typed_out = run_import(
        &encoded(&typed.to_string()),
        false,
        false,
        &[],
        &[],
        "noname_",
    )
    .unwrap();
    assert_eq!(typed_out.storage.get("s"), Some(&"ab".to_string()));
    assert_eq!(typed_out.storage.get("b"), Some(&"true".to_string()));
    assert_eq!(typed_out.storage.get("n"), Some(&"null".to_string()));
    assert_eq!(typed_out.storage.get("i"), Some(&"12".to_string()));
    assert_eq!(typed_out.storage.get("f"), Some(&"1.5".to_string()));
    assert_eq!(typed_out.storage.get("a"), Some(&"1,x,".to_string()));
    assert_eq!(
        typed_out.storage.get("o"),
        Some(&"[object Object]".to_string())
    );

    let preserved = run_import(
        &encoded(r#"{"noname_inited":"no"}"#),
        false,
        false,
        &["甲"],
        &[
            ("noname_inited", "yes"),
            ("noname_key", "secret"),
            ("temp", "1"),
        ],
        "noname_",
    )
    .unwrap();
    assert_eq!(
        preserved.storage,
        store_map(&[("noname_inited", "no"), ("noname_key", "secret")])
    );
    assert_eq!(preserved.extensions, names(&["甲"]));
    let forgotten = run_import(
        &encoded(r#"{"z":"1"}"#),
        false,
        false,
        &[],
        &[("noname_inited", ""), ("noname_key", "")],
        "noname_",
    )
    .unwrap();
    assert_eq!(forgotten.storage, store_map(&[("z", "1")]));
    let prefixed = import_config(&ImportInput {
        payload: encoded("{}"),
        db: false,
        keep_existing: false,
        extensions: names(&["甲"]),
        storage: store_map(&[("game_key", "secret"), ("noname_key", "other")]),
        config_prefix: "game_".into(),
        config: BTreeMap::new(),
        data: BTreeMap::new(),
    })
    .unwrap();
    assert_eq!(prefixed.storage, store_map(&[("game_key", "secret")]));
    assert_eq!(prefixed.extensions, names(&["甲"]));
    let overwritten = run_import(
        &encoded(r#"{"noname_key":"new"}"#),
        false,
        true,
        &[],
        &[("noname_key", "old")],
        "noname_",
    )
    .unwrap();
    assert_eq!(
        overwritten.storage.get("noname_key"),
        Some(&"new".to_string())
    );

    let indexed = run_import(
        &encoded(r#"["x",2]"#),
        false,
        false,
        &["甲"],
        &[("temp", "1")],
        "noname_",
    )
    .unwrap();
    assert_eq!(indexed.storage, store_map(&[("0", "x"), ("1", "2")]));
    assert_eq!(indexed.extensions, names(&["甲"]));
    assert!(indexed.cleared);
    for bad in [
        "null",
        "1",
        "\"hi\"",
        "true",
        "{\"config\":{}}",
        "{\"config\":null,\"data\":{}}",
        "{\"config\":{\"extensions\":[1]},\"data\":{}}",
    ] {
        let err = run_import(
            &encoded(bad),
            true,
            true,
            &["甲"],
            &[("temp", "1")],
            "noname_",
        );
        assert!(err.is_err(), "{bad} 应拒绝");
    }
    assert!(matches!(
        run_import(
            &encoded("[]"),
            true,
            true,
            &["甲"],
            &[("temp", "1")],
            "noname_"
        ),
        Err(SjError::Format)
    ));
    assert!(matches!(
        run_import(" ", false, false, &[], &[], "noname_"),
        Err(SjError::BadEncoding(_))
    ));
    assert!(matches!(
        run_import("!!!!", true, true, &["甲"], &[("temp", "1")], "noname_"),
        Err(SjError::BadEncoding(_))
    ));

    let orders = [
        ("1.2", "1.2", VersionOrder::Equal),
        ("", "", VersionOrder::Equal),
        ("1.2", "1.10", VersionOrder::Older),
        ("1.10", "1.2", VersionOrder::Newer),
        ("1.10", "1.1", VersionOrder::Equal),
        ("1", "1.0", VersionOrder::Equal),
        ("", "1.2", VersionOrder::Newer),
        ("1.2", "", VersionOrder::Older),
        ("10", "2", VersionOrder::Older),
        ("2", "10", VersionOrder::Newer),
        ("2.0", "1.9", VersionOrder::Older),
        ("无", "1.2", VersionOrder::Newer),
        ("1.2", "无", VersionOrder::Newer),
        ("无", "0", VersionOrder::Unordered),
        ("0", "无", VersionOrder::Unordered),
        ("a", "b", VersionOrder::Unordered),
        ("1.a", "1.2", VersionOrder::Newer),
        ("1.2", "1.a", VersionOrder::Older),
        ("1.0.0", "1.0", VersionOrder::Unordered),
        ("1.0", "1.0.1", VersionOrder::Unordered),
        ("1.2.0", "1.2", VersionOrder::Unordered),
        ("1.0.0", "1.0.1", VersionOrder::Newer),
        ("1.2", "a", VersionOrder::Newer),
    ];
    for (local, remote, order) in orders {
        assert_eq!(compare_version(local, remote), order, "{local} vs {remote}");
    }

    assert_eq!(
        catalog_mark("1.0", Some("1.0"), Some("1.2"), Some("1.0.0")),
        CatalogMark::Incompatible
    );
    assert_eq!(
        catalog_mark("1.0.0", Some("1.0"), Some("1.2"), Some("1.0.0")),
        CatalogMark::Updatable
    );
    assert_eq!(
        catalog_mark("1.0", Some("1.0"), Some("1.0"), None),
        CatalogMark::Same
    );
    assert_eq!(
        catalog_mark("1.0", Some("1.2"), Some("1.0"), None),
        CatalogMark::LocalHigher
    );
    assert_eq!(
        catalog_mark("1.0", Some("1.0.0"), Some("1.0"), None),
        CatalogMark::LocalHigher
    );
    assert_eq!(
        catalog_mark("1.0", None, Some("1.2"), None),
        CatalogMark::Unmarked
    );
    assert_eq!(
        catalog_mark("1.0", Some("1.0"), Some(""), None),
        CatalogMark::Unmarked
    );
    assert_eq!(
        catalog_mark("1.0", Some(""), Some("1.2"), None),
        CatalogMark::Updatable
    );
    assert_eq!(
        catalog_mark("1.0", Some("1.0"), Some("1.0"), Some("1.0")),
        CatalogMark::Same
    );

    let reject_engine = SelectInput {
        engine_version: "1.0",
        local_version: Some("1.0"),
        remote_version: Some("1.2"),
        noname_version: Some("1.0.0"),
        allow_same: true,
        allow_higher: true,
    };
    assert!(!choose_download(&reject_engine));
    assert!(!choose_download(&SelectInput {
        engine_version: "1.0",
        local_version: Some("1.0"),
        remote_version: Some("1.0"),
        noname_version: None,
        allow_same: false,
        allow_higher: true
    }));
    assert!(choose_download(&SelectInput {
        engine_version: "1.0",
        local_version: Some("1.0"),
        remote_version: Some("1.0"),
        noname_version: None,
        allow_same: true,
        allow_higher: false
    }));
    assert!(!choose_download(&SelectInput {
        engine_version: "1.0",
        local_version: Some("1.2"),
        remote_version: Some("1.0"),
        noname_version: None,
        allow_same: true,
        allow_higher: false
    }));
    assert!(choose_download(&SelectInput {
        engine_version: "1.0",
        local_version: Some("1.2"),
        remote_version: Some("1.0"),
        noname_version: None,
        allow_same: false,
        allow_higher: true
    }));
    assert!(choose_download(&SelectInput {
        engine_version: "1.0",
        local_version: Some("1.0"),
        remote_version: Some("1.2"),
        noname_version: None,
        allow_same: false,
        allow_higher: false
    }));
    assert!(choose_download(&SelectInput {
        engine_version: "1.0",
        local_version: Some("1.0.0"),
        remote_version: Some("1.0"),
        noname_version: None,
        allow_same: false,
        allow_higher: false
    }));
    assert!(choose_download(&SelectInput {
        engine_version: "1.0",
        local_version: None,
        remote_version: Some("1.2"),
        noname_version: None,
        allow_same: false,
        allow_higher: false
    }));
    assert!(!choose_download(&SelectInput {
        engine_version: "1.0",
        local_version: Some("1.2"),
        remote_version: None,
        noname_version: None,
        allow_same: true,
        allow_higher: false
    }));

    let mut catalog = BTreeMap::new();
    catalog.insert("甲".into(), entry(Some("1.0"), None, Some(vec!["a.js"])));
    catalog.insert("乙".into(), entry(Some("1.0"), None, Some(vec![])));
    catalog.insert("丙".into(), entry(Some("2.0"), None, None));
    catalog.insert("丁".into(), entry(None, None, Some(vec!["d.js"])));
    catalog.insert("戊".into(), entry(Some("1.0"), None, Some(vec!["e.js"])));
    let mut packs = BTreeMap::new();
    packs.insert(
        "甲".into(),
        InstalledPack {
            version: Some("1.0".into()),
        },
    );
    packs.insert(
        "戊".into(),
        InstalledPack {
            version: Some("9.0".into()),
        },
    );
    packs.insert("丁".into(), InstalledPack { version: None });
    let requested = names(&["缺失", "丙", "甲", "乙", "丁", "戊", "甲"]);
    let installed_names = names(&["甲", "乙", "丁", "戊"]);
    assert_eq!(
        filter_downloads(&requested, &catalog, &installed_names, &packs, false),
        names(&["乙", "戊"])
    );
    assert_eq!(
        filter_downloads(&requested, &catalog, &installed_names, &packs, true),
        names(&["甲", "乙", "丁", "戊", "甲"])
    );
    let empty_plan = plan_files(&[]);
    assert!(empty_plan.names.is_empty());
    assert!(!empty_plan.fetched);
    assert!(!empty_plan.evaled);
    let one_plan = plan_files(&["a.js".into()]);
    assert_eq!(one_plan.names, names(&["a.js"]));
    assert!(!one_plan.fetched);
    assert!(!one_plan.evaled);
    assert!(load_remote_catalog(false).is_err());
    assert!(matches!(load_remote_catalog(false), Err(SjError::Offline)));
    assert!(matches!(
        load_remote_catalog(true),
        Err(SjError::NetworkRefused)
    ));
    assert!(matches!(
        eval_update_script("window.noname_android_extension={}"),
        Err(SjError::EvalRefused)
    ));

    assert_eq!(
        classify_fault(&TransferFault {
            exception: Some("Unable to resolve host: example".into()),
            http_status: HttpCode::Number(500)
        }),
        FaultAction::Skip
    );
    assert_eq!(
        classify_fault(&TransferFault {
            exception: Some("unable to resolve host".into()),
            http_status: HttpCode::Number(500)
        }),
        FaultAction::Retry
    );
    assert_eq!(
        classify_fault(&TransferFault {
            exception: None,
            http_status: HttpCode::Null
        }),
        FaultAction::Skip
    );
    assert_eq!(
        classify_fault(&TransferFault {
            exception: None,
            http_status: HttpCode::Number(404)
        }),
        FaultAction::Skip
    );
    assert_eq!(
        classify_fault(&TransferFault {
            exception: None,
            http_status: HttpCode::Text("404".into())
        }),
        FaultAction::Skip
    );
    assert_eq!(
        classify_fault(&TransferFault {
            exception: None,
            http_status: HttpCode::Text("0404".into())
        }),
        FaultAction::Retry
    );
    assert_eq!(
        classify_fault(&TransferFault {
            exception: None,
            http_status: HttpCode::Number(500)
        }),
        FaultAction::Retry
    );
    assert_eq!(
        classify_fault(&TransferFault {
            exception: None,
            http_status: HttpCode::Absent
        }),
        FaultAction::Retry
    );

    let finished = finish_download(
        &names(&["甲", "丙", "乙"]),
        &catalog,
        &names(&["甲"]),
        &BTreeMap::new(),
        true,
    );
    assert_eq!(finished.extensions, names(&["甲", "乙"]));
    assert_eq!(finished.enables.get("甲"), Some(&true));
    assert_eq!(finished.enables.get("乙"), Some(&true));
    assert!(!finished.enables.contains_key("丙"));
    assert!(finished.reload_requested);
    assert!(!finished.restarted);
    let no_reload = finish_download(&[], &catalog, &[], &BTreeMap::new(), true);
    assert!(!no_reload.reload_requested);
    assert!(!no_reload.restarted);

    let added = apply_external(
        &ExternalKind::Extension { name: "甲".into() },
        &names(&["SJ Settings"]),
        &BTreeMap::new(),
        &BTreeMap::new(),
    );
    assert_eq!(added.extensions, names(&["SJ Settings", "甲"]));
    assert_eq!(added.enables.get("甲"), Some(&true));
    assert!(added.reload_requested);
    assert!(!added.restarted);
    let again = apply_external(
        &ExternalKind::Extension { name: "甲".into() },
        &added.extensions,
        &added.enables,
        &added.storage,
    );
    assert_eq!(again.extensions, names(&["SJ Settings", "甲"]));
    let packaged = apply_external(
        &ExternalKind::Package {
            message: "blob".into(),
        },
        &names(&["SJ Settings"]),
        &BTreeMap::new(),
        &BTreeMap::new(),
    );
    assert_eq!(packaged.extensions, names(&["SJ Settings"]));
    assert_eq!(
        packaged.storage.get("noname_inited"),
        Some(&"blob".to_string())
    );
    assert!(packaged.reload_requested);
    assert!(!packaged.restarted);
    let other = apply_external(
        &ExternalKind::Other,
        &names(&["SJ Settings"]),
        &BTreeMap::new(),
        &packaged.storage,
    );
    assert_eq!(other.extensions, names(&["SJ Settings"]));
    assert!(!other.reload_requested);
    assert_eq!(
        other.storage.get("noname_inited"),
        Some(&"blob".to_string())
    );

    let idle = request_console(false, false, false);
    assert!(!idle.requested);
    assert!(!idle.eruda_started);
    let asked = request_console(true, false, false);
    assert!(asked.requested);
    assert!(!asked.eruda_started);
    assert!(!request_console(true, true, false).requested);
    assert!(!request_console(true, false, true).requested);
    assert!(matches!(
        open_asset_file("apk_tutorial.pdf"),
        Err(SjError::AssetRefused)
    ));
    assert!(matches!(
        open_asset_file("noname_tutorial.pdf"),
        Err(SjError::AssetRefused)
    ));

    assert_eq!(decode_uri("a%20b").unwrap(), "a b");
    assert_eq!(decode_uri("%E7%94%B2").unwrap(), "甲");
    assert_eq!(decode_uri("%3B").unwrap(), "%3B");
    assert_eq!(decode_uri("%3b").unwrap(), "%3b");
    assert_eq!(decode_uri("%23").unwrap(), "%23");
    assert_eq!(decode_uri("%25").unwrap(), "%");
    assert!(decode_uri("%").is_err());
    assert!(decode_uri("%2").is_err());
    assert!(decode_uri("%ZZ").is_err());
    assert!(decode_uri("%E7%94").is_err());
    assert!(decode_uri("%80").is_err());

    let plain = format_window_error(&WindowError {
        message: Some("boom".into()),
        source: None,
        line: Some("3".into()),
        column: Some("4".into()),
        stack: None,
        event: None,
        translate: BTreeMap::new(),
    })
    .unwrap();
    assert_eq!(
        plain.text,
        "错误文件: undefined\n错误信息: boom\n行号: 3\n列号: 4\n-------------"
    );
    assert!(!plain.continued);
    assert!(!plain.looped);
    let bare = window(
        "E",
        Some("a.js"),
        Some(event_of(frame("use", "3", None), None, None, None)),
        &[],
        None,
    );
    let bare_text = format_window_error(&bare).unwrap();
    assert_eq!(bare_text.text, "错误文件: a.js\n错误信息: E\n行号: 1\n列号: 2\nevent.name: use\nevent.step: 3\n-------------");
    assert!(!bare_text.continued && !bare_text.looped);
    let nested = window(
        "E",
        Some("a.js"),
        Some(event_of(
            frame(
                "use",
                "3",
                Some(frame("respond", "1", Some(frame("phase", "0", None)))),
            ),
            None,
            None,
            None,
        )),
        &[],
        None,
    );
    assert_eq!(
        format_window_error(&nested).unwrap().text,
        "错误文件: a.js\n错误信息: E\n行号: 1\n列号: 2\nevent.name: use\nevent.step: 3\nevent.parent.name: respond\nevent.parent.step: 1\nevent.parent.parent.name: phase\nevent.parent.parent.step: 0\n-------------"
    );
    let hidden = window(
        "E",
        Some("a.js"),
        Some(event_of(
            frame("use", "3", None),
            Some(Actor {
                name: "liubei".into(),
                distance: Some(0),
            }),
            None,
            None,
        )),
        &[],
        None,
    );
    assert_eq!(
        format_window_error(&hidden).unwrap().text,
        "错误文件: a.js\n错误信息: E\n行号: 1\n列号: 2\nevent.name: use\nevent.step: 3\n-------------\n-------------"
    );
    let seated = window(
        "E",
        Some("a.js"),
        Some(event_of(
            frame("use", "3", None),
            Some(Actor {
                name: "caocao".into(),
                distance: Some(2),
            }),
            None,
            None,
        )),
        &[("caocao", "曹操")],
        None,
    );
    assert_eq!(
        format_window_error(&seated).unwrap().text,
        "错误文件: a.js\n错误信息: E\n行号: 1\n列号: 2\nevent.name: use\nevent.step: 3\n-------------\nplayer: 曹操[caocao]\n座位号: 3"
    );
    let infinite = window(
        "E",
        Some("a.js"),
        Some(event_of(
            frame("use", "3", None),
            Some(Actor {
                name: "caocao".into(),
                distance: None,
            }),
            None,
            None,
        )),
        &[("caocao", "曹操")],
        None,
    );
    assert_eq!(
        format_window_error(&infinite).unwrap().text,
        "错误文件: a.js\n错误信息: E\n行号: 1\n列号: 2\nevent.name: use\nevent.step: 3\n-------------\nplayer: 曹操[caocao]"
    );
    assert!(!format_window_error(&infinite)
        .unwrap()
        .text
        .contains("座位号"));
    let blank_label = window(
        "E",
        Some("a.js"),
        Some(event_of(
            frame("use", "3", None),
            Some(Actor {
                name: "caocao".into(),
                distance: Some(0),
            }),
            None,
            None,
        )),
        &[("caocao", "")],
        None,
    );
    assert!(format_window_error(&blank_label)
        .unwrap()
        .text
        .ends_with("-------------\n-------------"));
    let skilled = window(
        "E",
        Some("a.js"),
        Some(event_of(
            frame("use", "3", None),
            None,
            Some("rende"),
            Some("sha"),
        )),
        &[("rende", "仁德"), ("sha", "杀")],
        None,
    );
    assert_eq!(
        format_window_error(&skilled).unwrap().text,
        "错误文件: a.js\n错误信息: E\n行号: 1\n列号: 2\nevent.name: use\nevent.step: 3\n-------------\nskill: 仁德[rende]\ncard: 杀[sha]"
    );
    let mixed = window(
        "E",
        Some("a.js"),
        Some(event_of(
            frame("use", "3", None),
            Some(Actor {
                name: "liubei".into(),
                distance: Some(1),
            }),
            Some("rende"),
            None,
        )),
        &[("rende", "仁德")],
        None,
    );
    let mixed_text = format_window_error(&mixed).unwrap().text;
    assert!(mixed_text.contains("\n-------------\nskill: 仁德[rende]"));
    assert!(!mixed_text.ends_with("-------------"));
    assert_eq!(
        format_window_error(&window("E", Some("%20a"), None, &[], None))
            .unwrap()
            .text
            .lines()
            .next(),
        Some("错误文件:  a")
    );
    assert_eq!(
        format_window_error(&window("E", Some("%3B"), None, &[], None))
            .unwrap()
            .text
            .lines()
            .next(),
        Some("错误文件: %3B")
    );
    assert_eq!(
        format_window_error(&window("E", Some("%E7%94%B2.js"), None, &[], None))
            .unwrap()
            .text
            .lines()
            .next(),
        Some("错误文件: 甲.js")
    );
    assert_eq!(
        format_window_error(&window("E", Some(""), None, &[], None))
            .unwrap()
            .text
            .lines()
            .next(),
        Some("错误文件: undefined")
    );
    assert!(format_window_error(&window("E", Some("%"), None, &[], None)).is_err());
    let stacked = format_window_error(&WindowError {
        message: Some("boom".into()),
        source: None,
        line: None,
        column: None,
        stack: Some("%20x%3B".into()),
        event: None,
        translate: BTreeMap::new(),
    })
    .unwrap();
    assert_eq!(stacked.text, "错误文件: undefined\n错误信息: boom\n行号: undefined\n列号: undefined\n-------------\n x%3B");
    assert!(!stacked.continued && !stacked.looped);
    assert!(format_window_error(&WindowError {
        stack: Some("%".into()),
        ..stacked_source()
    })
    .is_err());

    let chain = apply_sj_commands(
        &stored,
        "boot;asset;version:1.2:1.10;version:1.10:1.2;version:1:1.0;version:无:1.2;version:1.2:无;version:1.0.0:1.0;version:1.0.0:1.0.1;version::1.2;pdf;console;fault:404;fault:other",
    )
    .unwrap();
    assert_eq!(
        chain,
        "boot:SJ Settings en:SJ Settings=1 key:absent asset:无 version:false version:true version:equal version:true version:true version:undefined version:true version:true asset:refused console:0 eruda:0 fault:skip fault:retry"
    );
    let marks = apply_sj_commands(&stored, "mark:1.0:1.0:1.2:1.0.0;mark:1.0:1.0:1.2:-;mark:1.0:1.2:1.0:-;mark:1.0:1.0:1.0:-;mark:1.0:-:1.2:-;mark:1.0:1.0.0:1.0:-;select:1.0:1.0:1.2:1.0.0:1:1;select:1.0:1.0:1.0:-:0:1;select:1.0:1.0:1.0:-:1:0;select:1.0:1.2:1.0:-:1:0;select:1.0:1.2:1.0:-:1:1;select:1.0:1.0:1.2:-:0:0;select:1.0:-:1.2:-:0:0;select:1.0:1.0.0:1.0:-:0:0").unwrap();
    assert_eq!(marks, "mark:incompatible mark:updatable mark:higher mark:same mark:unmarked mark:higher select:0 select:0 select:1 select:0 select:1 select:1 select:1 select:1");
    let payload = encoded(db_body);
    let keep_cmd = format!("seed-ext:甲;boot;import:db:keep:{payload}");
    let drop_cmd = format!("seed-ext:甲;boot;import:db:drop:{payload}");
    assert_eq!(
        apply_sj_commands(&stored, &keep_cmd).unwrap(),
        "boot:甲,SJ Settings en:SJ Settings=1 key:absent import:ok exts:B,甲,SJ Settings asset:9 note:诗笺 reload:1 restarted:0"
    );
    assert_eq!(
        apply_sj_commands(&stored, &drop_cmd).unwrap(),
        "boot:甲,SJ Settings en:SJ Settings=1 key:absent import:ok exts:B,SJ Settings asset:9 note:诗笺 reload:1 restarted:0"
    );
    assert_eq!(
        apply_sj_commands(&stored, "seed-store:noname_inited:yes;seed-store:noname_key:secret;seed-store:temp:1;import:nodb:keep:eyJhIjoxfQ==").unwrap(),
        "import:ok exts:- asset:- storage:a=1,noname_inited=yes,noname_key=secret reload:1 restarted:0"
    );
    assert_eq!(
        apply_sj_commands(&stored, "import:nodb:drop:").unwrap(),
        "import:noop reload:0 restarted:0"
    );
    assert_eq!(
        apply_sj_commands(&stored, "import:db:keep:!!!!").unwrap(),
        "import:bad reload:0 restarted:0"
    );
    assert_eq!(
        apply_sj_commands(&stored, "import:db:keep:bnVsbA==").unwrap(),
        "import:format reload:0 restarted:0"
    );
    assert_eq!(
        apply_sj_commands(&stored, "boot:[]").unwrap(),
        "boot:- key:removed"
    );
    assert_eq!(
        apply_sj_commands(&stored, "boot:[\"甲\"]").unwrap(),
        "boot:甲 en:甲=1 key:removed"
    );
    assert!(apply_sj_commands(&stored, "boot:null").is_err());
    assert!(apply_sj_commands(&stored, "boot:[1]").is_err());
    assert!(apply_sj_commands("{}", "boot").is_err());
    assert!(apply_sj_commands(&stored, "nope").is_err());
    assert!(apply_sj_commands(&stored, "onerror:bad").is_err());
    let tools = stored.replace("\"openTools\": false", "\"openTools\": true");
    assert_eq!(
        apply_sj_commands(&tools, "console;console;console:env").unwrap(),
        "console:1 eruda:0 console:0 eruda:0 console:0 eruda:0"
    );
    assert_eq!(
        apply_sj_commands(&stored, "boot;external:extension:甲;external:package:blob;external:other").unwrap(),
        "boot:SJ Settings en:SJ Settings=1 key:absent external:extension exts:SJ Settings,甲 reload:1 restarted:0 external:package exts:SJ Settings,甲 inited:blob reload:1 restarted:0 external:other exts:SJ Settings,甲 inited:blob reload:0 restarted:0"
    );
    assert_eq!(
        apply_sj_commands(
            &stored,
            "fault:404;fault:text;fault:null;fault:host;fault:lower;fault:missing;fault:500"
        )
        .unwrap(),
        "fault:skip fault:skip fault:skip fault:skip fault:retry fault:retry fault:retry"
    );
    assert_eq!(
        apply_sj_commands(
            &stored,
            "eval:window.noname_android_extension={};catalog:0;catalog:1"
        )
        .unwrap(),
        "eval:refused catalog:offline catalog:refused"
    );
    assert_eq!(
        apply_sj_commands(&stored, "onerror:plain;onerror:stack").unwrap(),
        "onerror:错误文件: undefined\\n错误信息: boom\\n行号: 3\\n列号: 4\\n------------- continued:0 looped:0 onerror:错误文件: undefined\\n错误信息: boom\\n行号: undefined\\n列号: undefined\\n-------------\\n x%3B continued:0 looped:0"
    );
    let pinned = apply_sj_commands(
        &stored,
        "boot;import:db:keep:eyJjb25maWciOnsiZXh0ZW5zaW9ucyI6WyLnlLIiLCJTSiBTZXR0aW5ncyJdLCJhc3NldF92ZXJzaW9uIjoi5pegIn0sImRhdGEiOnsibm90ZSI6Iuivl+esuiJ9fQ==",
    )
    .unwrap();
    assert_eq!(pinned, "boot:SJ Settings en:SJ Settings=1 key:absent import:ok exts:甲,SJ Settings asset:无 note:诗笺 reload:1 restarted:0");
    assert_eq!(
        store.current_lock().expect("lock").lock_hash,
        lock_before,
        "公式不改内容锁"
    );

    let canvas_dir = tempfile::tempdir().unwrap();
    assert!(
        canvas_dir.path().read_dir().unwrap().next().is_none(),
        "不创建画布、进度条或 PDF"
    );
    let asset_names: Vec<_> = std::fs::read_dir(installed_dir.join("assets"))
        .unwrap()
        .map(|item| item.unwrap().file_name())
        .collect();
    assert_eq!(asset_names, vec![std::ffi::OsString::from("config.json")]);

    match store.activate_version("upstream-liuli", "sj-settings", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../sj-settings".into();
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
        active_version(&store, "upstream-liuli", "sj-settings"),
        "1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "coin"),
        "1.0.0",
        "失败安装不撤掉 liuli 金币"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "elo-sanction"),
        "1.0.0",
        "失败安装不撤掉段位制裁"
    );

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("sj-settings-disabled");
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
            .any(|item| item.namespace == "upstream-liuli" && item.package_id == "sj-settings"));
        assert!(disabled
            .skipped
            .iter()
            .any(|name| name == "sj-settings-disabled"));
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
        active_version(&store, "upstream-liuli", "sj-settings"),
        "2.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "coin"),
        "1.0.0",
        "升级 SJ Settings 不改 liuli 金币"
    );
    assert_eq!(
        active_version(&store, "upstream-noname", "coin"),
        "1.0.0",
        "升级 SJ Settings 不改 noname 金币"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "elo-sanction"),
        "1.0.0",
        "升级 SJ Settings 不改段位制裁"
    );
    let rolled = store.rollback().expect("回滚");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, version)| slot.ends_with("upstream-liuli/sj-settings") && version == "1.0.0"));
    assert_eq!(
        active_version(&store, "upstream-liuli", "coin"),
        "1.0.0",
        "回滚后 liuli 金币仍是 1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "elo-sanction"),
        "1.0.0",
        "回滚后段位制裁仍是 1.0.0"
    );

    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        active_version(&reopened, "upstream-liuli", "sj-settings"),
        "1.0.0"
    );
    let (_, kept_dir) = read_installed(&reopened, "upstream-liuli", "sj-settings", "1.0.0");
    let kept_text = std::fs::read_to_string(kept_dir.join("assets/config.json")).unwrap();
    let kept_config = parse_sj_config(&kept_text).unwrap();
    assert_eq!(kept_config.source, "noname-liuli");
    assert_eq!(kept_config.upstream_version, "1.4204");
    assert!(!kept_config.open_tools);
    assert!(kept_text.contains("\"source\": \"noname-liuli\""));
}

fn stacked_source() -> WindowError {
    WindowError {
        message: Some("boom".into()),
        source: None,
        line: None,
        column: None,
        stack: None,
        event: None,
        translate: BTreeMap::new(),
    }
}
