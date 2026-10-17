//! U02 / L03 boss「play 包」八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-noname/boss-play` 与
//! `upstream-liuli/boss-play`。挑战/剑阁武将包装配、体力归一（Infinity）、
//! 禁 AI、翻译默认与 intro 别名、`isBossAllowed` 过滤与 `versus_ladder` 排除
//! 走本包配置。不读写 localStorage、不加载模式、不画界面、不播音。

use std::path::PathBuf;

use content_native_loader::{
    exclude_versus_ladder, filter_allowed, init_outcome, merge_translate, parse_boss_play_config,
    parse_storage, BossPlayConfig, HpKind, PlayStorage,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::ContentPackage;

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
        .unwrap_or_else(|| panic!("激活集含 {package_id}"))
}

fn storage_json() -> &'static str {
    r#"{
      "boss": {
        "挑战甲": {"hp": 4, "isBossAllowed": true},
        "挑战乙": {"hp": "3"},
        "挑战丙": {"hp": "3/4"},
        "挑战丁": {"hp": null}
      },
      "versus": {
        "剑阁甲": {"hp": 5, "tags": ["bossallowed"]},
        "剑阁乙": {"hp": "3/4"},
        "剑阁丙": {"hp": 4}
      },
      "translate": {"x": "y", "挑战自定义": "自定义"}
    }"#
}

fn run_variant(dir_name: &str, namespace: &str, liuli: bool) {
    use content_runtime::RuntimeError;

    let source_root = repo_extensions_root().join(dir_name);
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text =
        std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");
    let npack_text = String::from_utf8(npack_bytes.clone()).expect("npack utf-8");
    assert!(npack_text.contains("\"package_id\":\"boss-play\"")
        || npack_text.contains("\"package_id\": \"boss-play\""));

    let config: BossPlayConfig = parse_boss_play_config(&config_text).expect("配置");
    assert_eq!(config.source, if liuli { "noname-liuli" } else { "noname" });
    assert_eq!(config.intro_aliases.len(), if liuli { 8 } else { 15 });
    if liuli {
        assert_eq!(config.tag_boss.as_deref(), Some("mode:boss"));
        assert_eq!(config.tag_versus.as_deref(), Some("mode:versus"));
        assert_eq!(config.boss_pack_key, "mode_extension_boss");
        assert_eq!(config.boss_translate_key, "mode_extension_boss_character_config");
    } else {
        assert!(config.tag_boss.is_none() && config.tag_versus.is_none());
        assert_eq!(config.boss_pack_key, "boss");
        assert_eq!(config.boss_translate_key, "boss_character_config");
    }

    let storage = parse_storage(storage_json());
    assert_eq!(storage.boss.len(), 4);
    assert_eq!(storage.versus.len(), 3);

    // ---- 功能：装配 + 体力归一 + 禁 AI + 翻译 + intro ----
    let out = init_outcome(&config, "prototype", "", "", &storage, true);
    assert!(!out.skipped);
    assert_eq!(out.boss_pack.len(), 4);
    assert_eq!(out.jiange_pack.len(), 3);
    assert_eq!(out.characters.len(), 7);
    assert!(out.forbidai.is_empty());
    let boss_hp = |name: &str| {
        out.boss_pack
            .iter()
            .find(|e| e.name == name)
            .map(|e| e.hp.clone())
            .unwrap()
    };
    assert_eq!(boss_hp("挑战甲"), HpKind::Number(4));
    assert_eq!(boss_hp("挑战乙"), HpKind::Infinity, "数字字符串应归一为 Infinity");
    assert_eq!(boss_hp("挑战丙"), HpKind::Fraction("3/4".into()));
    assert_eq!(boss_hp("挑战丁"), HpKind::Infinity);
    let jiange_hp = |name: &str| {
        out.jiange_pack
            .iter()
            .find(|e| e.name == name)
            .map(|e| e.hp.clone())
            .unwrap()
    };
    assert_eq!(jiange_hp("剑阁甲"), HpKind::Number(5));
    assert_eq!(jiange_hp("剑阁乙"), HpKind::Infinity, "剑阁分数应归一为 Infinity");
    assert_eq!(jiange_hp("剑阁丙"), HpKind::Number(4));
    // 标签：liuli 追加，noname 不加。
    let boss_tag = out.boss_pack.iter().find(|e| e.name == "挑战甲").unwrap();
    if liuli {
        assert!(boss_tag.tags.iter().any(|t| t == "mode:boss"));
    } else {
        assert!(!boss_tag.tags.iter().any(|t| t == "mode:boss"));
    }
    // 翻译默认与 storage 值。
    let tr = |k: &str| {
        out.translate
            .iter()
            .find(|(key, _)| key == k)
            .map(|(_, v)| v.clone())
    };
    assert_eq!(tr(&config.boss_translate_key).as_deref(), Some("挑战武将"));
    assert_eq!(tr(&config.jiange_translate_key).as_deref(), Some("剑阁武将"));
    assert_eq!(tr("x").as_deref(), Some("y"));
    assert_eq!(out.intro.len(), config.intro_aliases.len());
    assert!(out
        .intro
        .iter()
        .any(|(a, b)| a == "boss_liedixuande" && b == "liubei"));

    // ---- 功能：模式门控 ----
    let boss_mode = init_outcome(&config, "boss", "", "", &storage, true);
    assert!(boss_mode.boss_pack.is_empty(), "boss 模式下不装挑战包");
    assert_eq!(boss_mode.jiange_pack.len(), 3);
    let tafang = init_outcome(&config, "tafang", "", "", &storage, true);
    assert!(tafang.skipped && tafang.boss_pack.is_empty() && tafang.translate.is_empty());
    let vs_jiange = init_outcome(&config, "versus", "jiange", "prototype", &storage, true);
    assert!(vs_jiange.jiange_pack.is_empty(), "versus+jiange 不装剑阁包");
    assert_eq!(vs_jiange.characters.len(), 7, "boss 块无条件装配 + else 分支覆盖剑阁");
    assert!(vs_jiange.intro.is_empty());
    let vs_jiange_in = init_outcome(&config, "versus", "jiange", "jiange", &storage, true);
    assert_eq!(vs_jiange_in.characters.len(), 4, "status_mode=jiange 时不覆盖剑阁，仅 boss 块");

    // ---- 功能：禁 AI ----
    let no_ai = init_outcome(&config, "prototype", "", "", &storage, false);
    assert_eq!(no_ai.forbidai.len(), 7);
    assert!(no_ai.forbidai.iter().any(|n| n == "挑战甲"));

    // ---- 功能：允许过滤 / 翻译合并 / versus_ladder ----
    let allowed = filter_allowed(&out.jiange_pack);
    assert_eq!(allowed.len(), 1);
    assert_eq!(allowed[0].name, "剑阁甲");
    assert_eq!(
        merge_translate(
            &[("x".into(), "old".into())],
            &[("x".into(), "new".into()), ("z".into(), "nz".into())]
        ),
        vec![
            ("x".to_string(), "old".into()),
            ("z".to_string(), "nz".into())
        ]
    );
    assert_eq!(
        exclude_versus_ladder(&["a".into(), "versus_ladder".into(), "b".into()]),
        vec!["a".to_string(), "b".into()]
    );

    // ---- 规则差异：storage 非法输入视为空；配置不含 localStorage/eval/fs ----
    assert_eq!(parse_storage("not json"), PlayStorage::default());
    assert_eq!(parse_storage("[1,2]"), PlayStorage::default());
    assert!(!config_text.contains("localStorage"));
    assert!(!config_text.contains("eval("));
    assert!(!config_text.contains("fs."));

    // ---- 安装 ----
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "boss-play" && item.namespace == namespace)
        .expect("boss-play 已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes)
    );
    // 组合：两来源 boss-play 都在，且各自内容哈希独立。
    for ns in ["upstream-noname", "upstream-liuli"] {
        let kept = report
            .installed
            .iter()
            .find(|item| item.package_id == "boss-play" && item.namespace == ns)
            .unwrap_or_else(|| panic!("组合：{ns}/boss-play 仍在"));
        assert_eq!(kept.version, "1.0.0");
    }

    let (installed, installed_dir) = read_installed(&store, namespace, "boss-play", "1.0.0");
    assert!(installed.manifest.dependencies.is_empty());
    assert!(installed.character_index.is_empty());
    assert!(installed.card_index.is_empty());
    let lock_before = store.current_lock().expect("lock").lock_hash;
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());
    let _ = init_outcome(&parse_boss_play_config(&stored).unwrap(), "prototype", "", "", &storage, true);
    assert_eq!(store.current_lock().expect("lock").lock_hash, lock_before);

    // ---- 保存恢复 ----
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(active_version(&reopened, namespace, "boss-play"), "1.0.0");

    // ---- 依赖 ----
    match store.activate_version(namespace, "boss-play", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }

    // ---- 安装升级 ----
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
    assert_eq!(active_version(&store, namespace, "boss-play"), "2.0.0");
    store.rollback().expect("rollback");
    assert_eq!(active_version(&store, namespace, "boss-play"), "1.0.0");
}

/// U02（noname）八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn u02_eight_dimension_from_fresh_install() {
    run_variant("boss-play-upstream-noname", "upstream-noname", false);
}

/// L03（noname-liuli）八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l03_eight_dimension_from_fresh_install() {
    run_variant("boss-play-upstream-liuli", "upstream-liuli", true);
}
