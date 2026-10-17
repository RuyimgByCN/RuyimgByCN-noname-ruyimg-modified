//! L07 全能搜索八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装出发。检索只走公开目录，不读取对局手牌。
//! 界面、分页、高亮皮肤、背景图和语音播放没有迁入；`debug_execute` 拒绝脚本。

use std::collections::BTreeMap;
use std::path::PathBuf;

use content_native_loader::{
    debug_execute, debug_inspect, parse_config, EntryKind, PublicIndex, SearchError,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency, PackageManifest};
use engine_native::content::standard::standard_catalog;
use engine_native::view::HandView;
use engine_native::Engine;
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};

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

fn read_installed(store: &PackageStore, namespace: &str, package_id: &str, version: &str) -> (ContentPackage, PathBuf) {
    let dir = store_dir(store, namespace, package_id, version);
    let bytes = std::fs::read(dir.join("package.json")).expect("商店副本 package.json");
    let package = ContentPackage::from_json_bytes(&bytes).expect("解析商店副本");
    (package, dir)
}

fn zh_cn(package: &ContentPackage) -> BTreeMap<String, String> {
    package
        .translations
        .get("zh-CN")
        .cloned()
        .unwrap_or_default()
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

fn file_hash(bytes: &[u8]) -> [u8; 32] {
    *blake3::Hasher::new().update(bytes).finalize().as_bytes()
}

/// L07 八项：功能、配置、依赖、组合、保存恢复、安装升级、规则差异。
/// 目标平台的字符串辅助在 `engine-native-wasm` 的同名查询里覆盖。
#[test]
fn l07_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let source_root = repo_extensions_root().join("omnisearch-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text = std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "omnisearch")
        .expect("全能搜索已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(meta.namespace, "upstream-liuli");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes),
        "商店记录的 content_hash 来自真实 npack 字节"
    );

    let (installed, installed_dir) = read_installed(&store, "upstream-liuli", "omnisearch", "1.0.0");
    assert!(installed.manifest.dependencies.is_empty(), "依赖正例：不声明外部扩展");
    assert_eq!(
        std::fs::read(installed_dir.join("assets/config.json")).unwrap(),
        config_text.as_bytes(),
        "商店里的配置副本与仓库源一致"
    );
    let config = parse_config(&std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap())
        .expect("配置正例");
    assert_eq!(config.css, "vs2015.min.css");
    assert_eq!(config.background, "none");
    let bad = parse_config(r#"{"css":"nope.min.css","background":"none"}"#);
    assert!(bad.unwrap_err().contains("unknown css"), "配置反例：未知样式");
    let bad_bg = parse_config(r#"{"css":"vs2015.min.css","background":"不存在"}"#);
    assert!(bad_bg.unwrap_err().contains("unknown background"), "配置反例：未知背景");
    assert_eq!(active_version(&store, "omnisearch"), "1.0.0");

    match store.activate_version("upstream-liuli", "omnisearch", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("依赖反例：未安装版本应 NotFound: {other:?}"),
    }
    assert_eq!(active_version(&store, "omnisearch"), "1.0.0");

    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../omnisearch".into();
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
    match store.install_with_assets(&missing.to_json_bytes().unwrap(), None, &options(), &source_root) {
        Err(RuntimeError::Invalid(msg)) => assert!(msg.contains("missing dependency"), "{msg}"),
        other => panic!("依赖反例：缺依赖激活应拒绝: {other:?}"),
    }
    assert_eq!(active_version(&store, "omnisearch"), "1.0.0");

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("omnisearch-disabled");
    std::fs::create_dir_all(disabled_root.join("assets")).unwrap();
    std::fs::copy(source_root.join("manifest.json"), disabled_root.join("manifest.json")).unwrap();
    std::fs::copy(source_root.join("package.npack.json"), disabled_root.join("package.npack.json")).unwrap();
    std::fs::copy(source_root.join("assets/config.json"), disabled_root.join("assets/config.json")).unwrap();
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(disabled_root.join("manifest.json")).unwrap()).unwrap();
    manifest["enabled"] = serde_json::Value::Bool(false);
    std::fs::write(disabled_root.join("manifest.json"), serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    {
        let disabled_store = PackageStore::open(disabled_dir.path().join("store")).unwrap();
        let disabled = import_extensions(&disabled_store, disabled_dir.path(), &options()).expect("disabled import");
        assert!(!disabled.installed.iter().any(|item| item.package_id == "omnisearch"));
        assert!(disabled.skipped.iter().any(|name| name == "omnisearch-disabled"));
    }

    let mut upgraded = installed.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    store
        .install_with_assets(&upgraded.to_json_bytes().unwrap(), None, &options(), &source_root)
        .expect("升级安装");
    assert_eq!(active_version(&store, "omnisearch"), "2.0.0", "安装升级正例");
    let rolled = store.rollback().expect("回滚");
    assert!(
        rolled.packages.iter().any(|(slot, version)| slot.ends_with("/omnisearch") && version == "1.0.0"),
        "回滚回到 1.0.0"
    );
    assert_eq!(active_version(&store, "omnisearch"), "1.0.0");

    let alias_root = dir.path().join("kuwu-alias");
    std::fs::create_dir_all(alias_root.join("assets")).unwrap();
    let alias_cards = r#"{"pack":{"name":"kuwu-alias","version":1,"description":"重名","card_prefix":"alias_"},"cards":[{"name":"kuwu","kind":"sha","suit":"heart","rank":1}]}"#;
    std::fs::write(alias_root.join("assets/cards.json"), alias_cards).unwrap();
    let alias_bytes = alias_cards.as_bytes();
    let mut alias_manifest = AssetManifest::new("kuwu-alias");
    alias_manifest.entries.push(AssetEntry {
        asset_id: "kuwu-alias:cards".into(),
        source_hash: file_hash(alias_bytes),
        kind: AssetKind::Data,
        image_variants: Vec::new(),
        audio_variants: Vec::new(),
        priority: PreloadPriority::Lazy,
        depends_on: Vec::new(),
        platforms: Vec::new(),
    });
    let mut locale = BTreeMap::new();
    locale.insert("kuwu".into(), "苦无".into());
    let mut translations = BTreeMap::new();
    translations.insert("zh-CN".into(), locale);
    let mut alias = ContentPackage {
        manifest: PackageManifest::new("upstream-liuli", "kuwu-alias", "1.0.0"),
        modules: Vec::new(),
        skill_index: Vec::new(),
        character_index: Vec::new(),
        skin_index: Default::default(),
        card_index: Vec::new(),
        trigger_index: Vec::new(),
        modifier_index: Vec::new(),
        active_index: Vec::new(),
        view_as_index: Vec::new(),
        translations,
        asset_manifest: Some(alias_manifest),
        asset_paths: BTreeMap::from([("kuwu-alias:cards".into(), "assets/cards.json".into())]),
        compiler_version: "gen-1".into(),
    };
    alias.manifest.logic_hash = alias.compute_logic_hash();
    alias.manifest.asset_hash = alias.compute_asset_hash();
    alias.validate().expect("重名包校验");
    let alias_install = store
        .install_with_assets(&alias.to_json_bytes().unwrap(), None, &options(), &alias_root)
        .expect("安装重名包");
    assert_eq!(alias_install.meta.package_id, "kuwu-alias");

    let (huanle, huanle_dir) = read_installed(&store, "upstream-noname", "huanle-cards", "1.0.0");
    let (wuxing, wuxing_dir) = read_installed(&store, "upstream-liuli", "wuxing", "1.0.0");
    let (alias_pkg, alias_dir) = read_installed(&store, "upstream-liuli", "kuwu-alias", "1.0.0");
    let mut index = PublicIndex::new();
    index.insert_standard(&standard_catalog());
    index
        .insert_cards_json(
            "huanle-cards",
            &std::fs::read_to_string(huanle_dir.join("assets/cards.json")).unwrap(),
            &zh_cn(&huanle),
        )
        .expect("欢乐索引");
    index
        .insert_cards_json(
            "wuxing",
            &std::fs::read_to_string(wuxing_dir.join("assets/cards.json")).unwrap(),
            &zh_cn(&wuxing),
        )
        .expect("五行索引");
    index
        .insert_cards_json(
            "kuwu-alias",
            &std::fs::read_to_string(alias_dir.join("assets/cards.json")).unwrap(),
            &zh_cn(&alias_pkg),
        )
        .expect("重名索引");
    assert!(
        index.entries().iter().all(|entry| !entry.id.contains("hnl_") && !entry.id.contains("wxl_")),
        "索引不得使用入局后的实例牌名"
    );

    let kuwu = index.search("苦无", &[]).expect("功能正例：苦无");
    let kuwu_ids: Vec<_> = kuwu.iter().map(|entry| entry.package_id.as_str()).collect();
    assert_eq!(kuwu_ids, ["huanle-cards", "kuwu-alias"], "组合正例：同名两条，按包区分");
    assert!(kuwu.iter().all(|entry| entry.kind == EntryKind::Card && entry.id == "kuwu"));
    let huanle_kuwu = index.detail("huanle-cards", "kuwu").expect("苦无详情");
    assert_eq!(huanle_kuwu.fields.get("mapping").map(String::as_str), Some("huanle_kuwu=host:sha_damage_force_discard"));
    assert_eq!(huanle_kuwu.fields.get("copies").map(String::as_str), Some("1"));
    let shouli = index.detail("huanle-cards", "shoulijian").expect("手里剑详情");
    assert_eq!(shouli.fields.get("copies").map(String::as_str), Some("2"), "两张手里剑合并为一条定义");
    let inspected = debug_inspect(huanle_kuwu);
    assert!(inspected.contains("host:sha_damage_force_discard"));
    assert!(!inspected.contains("eval("), "调试查看不包含可执行脚本");
    assert!(
        matches!(debug_execute("eval(game.playAudio('card','male','sha'))"), Err(SearchError::EvalRefused)),
        "规则差异：调试入口拒绝 eval"
    );

    let caocao = index.search("曹操", &[]).unwrap();
    assert!(caocao.iter().any(|entry| entry.kind == EntryKind::General && entry.package_id == "standard" && entry.fields.get("hp").map(String::as_str) == Some("4")));
    let skill = index.search("奸雄", &[]).unwrap();
    assert!(skill.iter().any(|entry| entry.kind == EntryKind::Skill && entry.id == "奸雄"));
    let sha = index.search("杀", &[]).unwrap();
    assert!(sha.iter().any(|entry| entry.package_id == "standard" && entry.id == "sha"));
    let pan = index.search("五行盘", &[]).unwrap();
    assert!(pan.iter().any(|entry| entry.package_id == "wuxing" && entry.id == "wuxingpan"));
    assert!(index.search("kuwu", &[]).unwrap().len() >= 2, "标识后缀匹配");
    assert!(index.search("没有这张牌xyz", &[]).unwrap().is_empty());
    assert!(matches!(index.search("   ", &[]), Err(SearchError::EmptyQuery)));
    assert!(index.insert_cards_json("../x", "{}", &BTreeMap::new()).is_err());

    index.drop_package("huanle-cards");
    let after_disable = index.search("苦无", &[]).unwrap();
    assert_eq!(after_disable.len(), 1);
    assert_eq!(after_disable[0].package_id, "kuwu-alias", "禁用欢乐后索引不再返回该包");
    index.drop_package("wuxing");
    assert!(index.search("五行盘", &[]).unwrap().is_empty(), "禁用五行后五行盘离开索引");
    assert!(index.search("杀", &[]).unwrap().iter().any(|entry| entry.id == "sha"));

    let eng = Engine::new(3417, 3);
    let own = eng.view_for(0);
    let mine = own.players.iter().find(|cell| cell.seat == 0).unwrap();
    let HandView::Own(hidden) = &mine.hand else {
        panic!("自己的手牌应可见");
    };
    assert!(!hidden.is_empty());
    let other = eng.view_for(1);
    let hidden_from_other = other.players.iter().find(|cell| cell.seat == 0).unwrap();
    assert!(matches!(hidden_from_other.hand, HandView::Hidden), "对手视图不得看到手牌");
    for id in hidden {
        match index.search(&id.to_string(), hidden) {
            Err(SearchError::HiddenCard) => {}
            other => panic!("暗牌实例号不得被检索取回: {id} -> {other:?}"),
        }
    }
    let still = index.search("曹操", hidden).unwrap();
    assert!(still.iter().any(|entry| entry.id == "曹操"));
    let dumped = format!("{still:?}");
    for id in hidden {
        assert!(!dumped.contains(&format!("id: \"{id}\"")), "公开结果不含暗牌实例号 {id}");
    }

    let saved = index.search("曹操", &[]).unwrap();
    let store_root = dir.path().join("store");
    let reopened = PackageStore::open(&store_root).unwrap();
    assert_eq!(active_version(&reopened, "omnisearch"), "1.0.0", "重开商店后版本仍是回滚结果");
    let (huanle2, huanle_dir2) = read_installed(&reopened, "upstream-noname", "huanle-cards", "1.0.0");
    let mut rebuilt = PublicIndex::new();
    rebuilt.insert_standard(&standard_catalog());
    rebuilt
        .insert_cards_json(
            "huanle-cards",
            &std::fs::read_to_string(huanle_dir2.join("assets/cards.json")).unwrap(),
            &zh_cn(&huanle2),
        )
        .unwrap();
    assert_eq!(rebuilt.search("曹操", &[]).unwrap(), saved, "保存恢复：重开后标准目录查询不变");
    let config_again = parse_config(&std::fs::read_to_string(store_dir(&reopened, "upstream-liuli", "omnisearch", "1.0.0").join("assets/config.json")).unwrap()).unwrap();
    assert_eq!(config_again, config, "保存恢复：配置仍是安装时的枚举值");
}
