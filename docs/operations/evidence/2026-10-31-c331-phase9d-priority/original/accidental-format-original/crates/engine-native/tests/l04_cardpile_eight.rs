//! L04 cardpile 八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/cardpile`。倍率走同一
//! `scale_plan`，但本测试的主体是 liuli 商店副本，不把 U03 再计一次。
//! 点数由调用方给出的 1..=13 序列填写，不是 `Math.random`。

use std::collections::BTreeMap;
use std::path::PathBuf;

use content_native_loader::parse_pile_factors;
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency};
use engine_native::content::extension::ExtensionDef;
use engine_native::content::pile::{
    extension_pack_from_pile, extension_pack_from_scale, restore_scale, scale_plan, snapshot_scale,
    ScaleStatus,
};
use engine_native::Engine;

const ONES: &str = r#"{"guohe":1,"huosha":1,"jiu":1,"leisha":1,"nanman":1,"sha":1,"shan":1,"shunshou":1,"tao":1,"tiesuo":1,"wanjian":1,"wuxie":1}"#;

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

fn bucket_count(status: &ScaleStatus, source: &str, suit: &str) -> u32 {
    match status {
        ScaleStatus::Added(buckets) => buckets
            .iter()
            .find(|bucket| bucket.source == source && bucket.suit == suit)
            .map(|bucket| bucket.count)
            .unwrap_or(0),
        _ => 0,
    }
}

fn ones() -> BTreeMap<String, f64> {
    parse_pile_factors(ONES).expect("十二项倍率")
}

/// L04 八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l04_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let source_root = repo_extensions_root().join("cardpile-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "cardpile" && item.namespace == "upstream-liuli")
        .expect("liuli 牌堆已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes),
        "商店 content_hash 来自 liuli npack"
    );
    assert!(
        report
            .installed
            .iter()
            .any(|item| item.package_id == "cardpile" && item.namespace == "upstream-noname"),
        "组合：同一次导入还有 noname 牌堆"
    );

    let (installed, installed_dir) = read_installed(&store, "upstream-liuli", "cardpile", "1.0.0");
    assert!(
        installed.manifest.dependencies.is_empty(),
        "依赖：不声明外部扩展"
    );
    let pile_text = std::fs::read_to_string(installed_dir.join("assets/pile.json")).unwrap();
    assert!(
        pile_text.contains("\"source\": \"noname-liuli\""),
        "读到的是 liuli 比例表"
    );
    let spec = content_native_loader::bridge_pile(&pile_text).expect("商店副本可装载");
    assert_eq!(spec.source, "noname-liuli");
    assert_eq!(spec.total, 160);
    let factors = ones();
    let lock_before = store.current_lock().unwrap().lock_hash;

    let added = scale_plan(&spec, &factors, 207).expect("功能正例");
    assert_eq!(bucket_count(&added, "sha", "diamond"), 6);
    assert_eq!(bucket_count(&added, "shan", "diamond"), 18);
    assert_eq!(bucket_count(&added, "tiesuo", "club"), 4);
    let ScaleStatus::Added(buckets) = &added else {
        panic!("应追加")
    };
    assert_eq!(
        buckets
            .iter()
            .map(|bucket| bucket.count as usize)
            .sum::<usize>(),
        113
    );

    let doubled = {
        let factors = parse_pile_factors(&ONES.replace("\"sha\":1", "\"sha\":2")).unwrap();
        scale_plan(&spec, &factors, 207).unwrap()
    };
    assert_eq!(bucket_count(&doubled, "sha", "diamond"), 33);
    assert_eq!(bucket_count(&doubled, "huosha", "heart"), 8);
    let capped = scale_plan(&spec, &factors, 600).unwrap();
    assert_eq!(bucket_count(&capped, "sha", "diamond"), 53);
    assert_eq!(bucket_count(&capped, "tiesuo", "club"), 35);
    assert_eq!(
        scale_plan(&spec, &factors, 160).unwrap(),
        ScaleStatus::NoAddition
    );
    assert_eq!(
        scale_plan(&spec, &factors, 100).unwrap(),
        ScaleStatus::NoAddition
    );
    let mut missing = factors.clone();
    missing.remove("jiu");
    assert_eq!(
        scale_plan(&spec, &missing, 207).unwrap(),
        ScaleStatus::Unconfigured
    );
    assert!(parse_pile_factors(r#"{"sha":1,"nope":1}"#)
        .unwrap_err()
        .contains("unknown pile factor"));
    assert!(parse_pile_factors(r#"{"sha":-1}"#)
        .unwrap_err()
        .contains("non-negative"));
    assert_eq!(
        store.current_lock().unwrap().lock_hash,
        lock_before,
        "计算倍率不改内容锁"
    );

    let (_, noname_dir) = read_installed(&store, "upstream-noname", "cardpile", "1.0.0");
    let noname_text = std::fs::read_to_string(noname_dir.join("assets/pile.json")).unwrap();
    assert!(noname_text.contains("\"source\": \"noname\""));
    assert!(!noname_text.contains("noname-liuli"));
    let noname = content_native_loader::bridge_pile(&noname_text).unwrap();
    assert_eq!(
        snapshot_scale(&scale_plan(&noname, &factors, 207).unwrap()),
        snapshot_scale(&added)
    );

    let ratio = extension_pack_from_pile(
        ExtensionDef {
            name: "cardpile-upstream-liuli",
            version: 1,
            description: "ratio",
        },
        "clp",
        &spec,
    )
    .expect("比例表");
    assert_eq!(ratio.card_count(), 113);
    assert!(ratio
        .cards
        .iter()
        .any(|card| card.name.starts_with("clp_sha_")));
    let ranks: Vec<u8> = (0..113).map(|i| (i % 13) as u8 + 1).collect();
    let scaled = extension_pack_from_scale(
        ExtensionDef {
            name: "cardpile-liuli-scale",
            version: 1,
            description: "scale",
        },
        "clp",
        buckets,
        &ranks,
    )
    .expect("追加牌");
    assert_eq!(scaled.card_count(), 113);
    assert!(scaled
        .cards
        .iter()
        .any(|card| card.name.contains("_scale_huosha_") && card.kind.key() == "huosha"));
    assert!(extension_pack_from_scale(
        ExtensionDef {
            name: "cardpile-liuli-scale",
            version: 1,
            description: "scale"
        },
        "clp",
        buckets,
        &vec![14; 113],
    )
    .is_err());
    let base = Engine::new(1, 3).catalog().cards.len();
    let mixed = Engine::new_with_extensions(31018, 3, &[&ratio, &scaled]).expect("建局");
    assert_eq!(mixed.catalog().cards.len(), base + 226);

    let saved = snapshot_scale(&added);
    assert_eq!(restore_scale(&saved).unwrap(), added);
    assert!(restore_scale("added\nsha diamond 0\n").is_err());
    assert_eq!(
        active_version(&store, "upstream-liuli", "cardpile"),
        "1.0.0"
    );

    match store.activate_version("upstream-liuli", "cardpile", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    assert_eq!(
        active_version(&store, "upstream-liuli", "cardpile"),
        "1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-noname", "cardpile"),
        "1.0.0",
        "失败激活不撤 noname"
    );

    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../cardpile".into();
    escaped.manifest.logic_hash = escaped.compute_logic_hash();
    match store.install(&escaped.to_json_bytes().unwrap(), None, &options()) {
        Err(RuntimeError::Package(msg)) => assert!(msg.contains("invalid package_id"), "{msg}"),
        other => panic!("包 id 逃逸应拒绝: {other:?}"),
    }

    let mut missing_dep = installed.clone();
    missing_dep.manifest.version = "1.0.1".into();
    missing_dep.manifest.dependencies.push(PackageDependency {
        namespace: "upstream-liuli".into(),
        package_id: "missing-base".into(),
        version: "1.0.0".into(),
        logic_hash: [0u8; 32],
    });
    missing_dep.manifest.logic_hash = missing_dep.compute_logic_hash();
    match store.install_with_assets(
        &missing_dep.to_json_bytes().unwrap(),
        None,
        &options(),
        &source_root,
    ) {
        Err(RuntimeError::Invalid(msg)) => assert!(msg.contains("missing dependency"), "{msg}"),
        other => panic!("缺依赖应拒绝: {other:?}"),
    }
    assert_eq!(
        active_version(&store, "upstream-noname", "cardpile"),
        "1.0.0"
    );

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("cardpile-liuli-disabled");
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
        source_root.join("assets/pile.json"),
        disabled_root.join("assets/pile.json"),
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
            .any(|item| item.package_id == "cardpile"));
        assert!(disabled
            .skipped
            .iter()
            .any(|name| name == "cardpile-liuli-disabled"));
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
        active_version(&store, "upstream-liuli", "cardpile"),
        "2.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-noname", "cardpile"),
        "1.0.0",
        "升级 liuli 不改 noname"
    );
    let rolled = store.rollback().expect("回滚");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, version)| slot.ends_with("upstream-liuli/cardpile") && version == "1.0.0"));
    assert_eq!(
        active_version(&store, "upstream-noname", "cardpile"),
        "1.0.0",
        "回滚 liuli 不撤 noname"
    );

    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        active_version(&reopened, "upstream-liuli", "cardpile"),
        "1.0.0"
    );
    let (_, again) = read_installed(&reopened, "upstream-liuli", "cardpile", "1.0.0");
    let again_text = std::fs::read_to_string(again.join("assets/pile.json")).unwrap();
    assert!(again_text.contains("\"source\": \"noname-liuli\""));
    let again_spec = content_native_loader::bridge_pile(&again_text).unwrap();
    assert_eq!(
        snapshot_scale(&scale_plan(&again_spec, &ones(), 207).unwrap()),
        saved
    );
}
