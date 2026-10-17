//! L22 段位制裁八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/elo-sanction`。力度、已知牌、
//! 重排、透视和烧条秒数走本包配置。分数由调用方给出，不读 `get.value`，也不接
//! 判定函数。不延迟，不说台词，不筛将池。不安装如真似幻。

use std::path::PathBuf;

use content_native_loader::{
    apply_elo_commands, hand_visible, mark_inserted, mark_opening, parse_elo_config, reorder_draw,
    resolve_difficulty, restore_pile, restrict_pool, snapshot_pile, think_delay, wash, EloCard,
    EloError, EloLevel, EloQuery, EloViewer,
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
    store.root().join("packages").join(format!("{namespace}__{package_id}")).join(version)
}

fn read_installed(store: &PackageStore, namespace: &str, package_id: &str, version: &str) -> (ContentPackage, PathBuf) {
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

fn card(id: u32, known: bool, score: i64) -> EloCard {
    EloCard { id, known, score }
}

fn level(key: &str, online: bool, mode: &str, paiwei_level: Option<i32>) -> EloLevel {
    resolve_difficulty(key, &EloQuery { online, mode: mode.to_string(), paiwei_level })
}

fn ids(pile: &[EloCard]) -> Vec<u32> {
    pile.iter().map(|card| card.id).collect()
}

fn sample() -> Vec<EloCard> {
    vec![card(2, false, 1), card(3, false, 9), card(4, false, 4), card(1, true, 0)]
}

fn viewer_friend() -> EloViewer {
    EloViewer { has_me: true, is_me: false, under_control: false }
}

/// L22 八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l22_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let source_root = repo_extensions_root().join("elo-sanction-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text = std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");
    let npack_text = String::from_utf8(npack_bytes.clone()).expect("npack utf-8");
    assert!(npack_text.contains("段位制裁"));
    assert!(npack_text.contains("\"package_id\":\"elo-sanction\"") || npack_text.contains("\"package_id\": \"elo-sanction\""));

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "elo-sanction" && item.namespace == "upstream-liuli")
        .expect("段位制裁已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(meta.content_hash, content_runtime::content_hash_hex(&npack_bytes), "商店 content_hash 来自本包 npack");
    let liuli_coin = report
        .installed
        .iter()
        .find(|item| item.package_id == "coin" && item.namespace == "upstream-liuli")
        .expect("组合：liuli 金币仍在");
    assert_eq!(liuli_coin.version, "1.0.0");
    let l05_npack = std::fs::read(repo_extensions_root().join("coin-upstream-liuli/package.npack.json")).unwrap();
    assert_eq!(liuli_coin.content_hash, content_runtime::content_hash_hex(&l05_npack), "不改写 L05 npack");
    let noname_coin = report
        .installed
        .iter()
        .find(|item| item.package_id == "coin" && item.namespace == "upstream-noname")
        .expect("组合：noname 金币仍在");
    assert_eq!(noname_coin.version, "1.0.0");
    let u04_npack = std::fs::read(repo_extensions_root().join("coin-upstream-noname/package.npack.json")).unwrap();
    assert_eq!(noname_coin.content_hash, content_runtime::content_hash_hex(&u04_npack), "不改写 U04 npack");
    assert!(!report.installed.iter().any(|item| item.package_id == "如真似幻"), "不把如真似幻计入");

    let (installed, installed_dir) = read_installed(&store, "upstream-liuli", "elo-sanction", "1.0.0");
    assert!(installed.manifest.dependencies.is_empty(), "依赖：不声明外部扩展");
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());
    assert!(stored.contains("\"source\": \"noname-liuli\""));
    assert!(stored.contains("noname/extension/段位制裁/extension.js"));
    assert!(stored.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
    let config = parse_elo_config(&stored).expect("配置");
    assert_eq!(config.source, "noname-liuli");
    assert!(config.elo_players);
    assert_eq!(config.difficulty, "paiwei");
    assert_eq!(config.time_killer, "none");
    assert!(matches!(
        parse_elo_config(&stored.replace("\"noname-liuli\"", "\"\"")),
        Err(EloError::BadConfig(msg)) if msg.contains("source")
    ));
    assert!(matches!(parse_elo_config(&stored.replace("noname-liuli", "bad token")), Err(EloError::BadConfig(_))));
    assert!(matches!(
        parse_elo_config(&stored.replace("\"eloPlayers\": true", "\"eloPlayers\": \"yes\"")),
        Err(EloError::BadConfig(msg)) if msg.contains("eloPlayers")
    ));
    assert!(matches!(
        parse_elo_config(&stored.replace("\"paiwei\"", "1")),
        Err(EloError::BadConfig(msg)) if msg.contains("eloDifficulty")
    ));
    let lock_before = store.current_lock().expect("lock").lock_hash;

    assert_eq!(level("eeasy", false, "identity", None), EloLevel { diff: 4, eh: 1 });
    assert_eq!(level("veasy", true, "identity", None), EloLevel { diff: 3, eh: 1 });
    assert_eq!(level("easy", false, "taixuhuanjing", None), EloLevel { diff: 2, eh: 1 });
    assert_eq!(level("normal", true, "identity", None), EloLevel { diff: 1, eh: 0 });
    assert_eq!(level("hard", true, "identity", None), EloLevel { diff: 2, eh: -1 });
    assert_eq!(level("vhard", true, "identity", None), EloLevel { diff: 3, eh: -1 });
    assert_eq!(level("ehard", false, "identity", None), EloLevel { diff: 4, eh: -1 });
    assert_eq!(level("mystery", true, "identity", None), EloLevel { diff: 4, eh: 0 });
    assert_eq!(level("paiwei", true, "identity", None), EloLevel { diff: 2, eh: 1 });
    assert_eq!(level("paiwei", false, "identity", None), EloLevel { diff: 1, eh: 0 });
    assert_eq!(level("paiwei", true, "taixuhuanjing", Some(0)), EloLevel { diff: 1, eh: 0 });
    assert_eq!(level("paiwei", false, "identity", Some(6)), EloLevel { diff: 1, eh: 0 });
    assert_eq!(level("paiwei", true, "identity", Some(0)), EloLevel { diff: 4, eh: 1 });
    assert_eq!(level("paiwei", true, "identity", Some(1)), EloLevel { diff: 3, eh: 1 });
    assert_eq!(level("paiwei", true, "identity", Some(3)), EloLevel { diff: 1, eh: 0 });
    assert_eq!(level("paiwei", true, "identity", Some(5)), EloLevel { diff: 3, eh: -1 });
    assert_eq!(level("paiwei", true, "identity", Some(6)), EloLevel { diff: 4, eh: -1 });

    let friend = level("eeasy", true, "identity", None);
    let enemy_bias = level("ehard", true, "identity", None);
    let closed = level("normal", true, "identity", None);
    let pile = sample();
    let friend_draw = reorder_draw(&pile, 2, friend, false);
    assert_eq!(ids(&friend_draw), vec![3, 4, 2, 1]);
    assert_eq!(ids(&reorder_draw(&pile, 2, friend, true)), vec![2, 4, 3, 1]);
    assert_eq!(ids(&reorder_draw(&pile, 2, enemy_bias, false)), vec![2, 4, 3, 1]);
    assert_eq!(ids(&reorder_draw(&pile, 2, enemy_bias, true)), vec![3, 4, 2, 1]);
    assert_eq!(ids(&reorder_draw(&pile, 2, closed, false)), vec![2, 3, 4, 1]);
    assert_eq!(ids(&reorder_draw(&pile, 0, friend, false)), vec![2, 3, 4, 1]);
    assert_eq!(ids(&reorder_draw(&pile[..1], 2, friend, false)), vec![2]);
    assert_eq!(ids(&reorder_draw(&pile, 1, level("easy", true, "identity", None), false)), vec![3, 2, 4, 1]);
    for card in &friend_draw {
        let original = pile.iter().find(|item| item.id == card.id).unwrap();
        assert_eq!(card.known, original.known, "重排不改已知标记");
    }
    assert!(friend_draw[3].known);

    let ties = vec![card(10, false, 5), card(11, false, 5), card(12, false, 1)];
    assert_eq!(ids(&reorder_draw(&ties, 3, friend, false)), vec![10, 11, 12]);
    assert_eq!(ids(&reorder_draw(&ties, 3, enemy_bias, false)), vec![12, 11, 10]);
    assert_eq!(ids(&reorder_draw(&ties, 3, friend, true)), vec![12, 10, 11]);
    assert_eq!(ids(&reorder_draw(&ties, 3, enemy_bias, true)), vec![11, 10, 12]);

    let slotted = vec![card(1, false, 1), card(9, true, 100), card(2, false, 8), card(3, false, 3)];
    let kept = reorder_draw(&slotted, 1, friend, false);
    assert_eq!(ids(&kept), vec![2, 9, 3, 1]);
    assert!(kept[1].known);
    assert!(!kept[0].known && !kept[2].known && !kept[3].known);

    let mut opening = vec![card(7, false, 1), card(8, false, 2)];
    mark_opening(&mut opening);
    assert!(opening.iter().all(|card| card.known));
    let saved = snapshot_pile(&opening);
    let restored = restore_pile(&saved).unwrap();
    assert_eq!(restored, opening);
    wash(&mut opening);
    assert!(opening.iter().all(|card| !card.known));
    let mut inserted = card(7, false, 1);
    mark_inserted(&mut inserted, true);
    assert!(!inserted.known, "重排放回不打已知标记");
    mark_inserted(&mut inserted, false);
    assert!(inserted.known);
    mark_inserted(&mut inserted, true);
    assert!(inserted.known, "no_mark 不清掉已有标记");
    assert!(restore_pile(&saved.replace("true", "1")).is_err(), "已知标记损坏拒绝");
    assert!(restore_pile(&saved.replace("\"score\":1", "\"score\":\"x\"")).is_err());
    assert!(restore_pile("{}").is_err());

    let friend_view = viewer_friend();
    let me = EloViewer { has_me: true, is_me: true, under_control: false };
    let control = EloViewer { has_me: true, is_me: false, under_control: true };
    let absent = EloViewer { has_me: false, is_me: false, under_control: false };
    let veasy = level("veasy", true, "identity", None);
    let easy = level("easy", true, "identity", None);
    let vhard = level("vhard", true, "identity", None);
    assert!(!hand_visible(veasy, me, true, Some(1)).unwrap());
    assert!(!hand_visible(veasy, control, true, Some(2)).unwrap());
    assert!(!hand_visible(veasy, absent, true, Some(1)).unwrap());
    assert!(!hand_visible(veasy, friend_view, true, Some(0)).unwrap());
    assert!(hand_visible(veasy, friend_view, true, Some(1)).unwrap());
    assert!(!hand_visible(veasy, friend_view, false, Some(1)).unwrap());
    assert!(matches!(hand_visible(veasy, friend_view, true, Some(2)), Err(EloError::BadRoll(_))));
    assert!(matches!(hand_visible(veasy, friend_view, true, None), Err(EloError::BadRoll(_))));
    assert!(!hand_visible(easy, friend_view, true, Some(7)).unwrap());
    assert!(hand_visible(friend, friend_view, true, None).unwrap());
    assert!(!hand_visible(friend, friend_view, false, Some(9)).unwrap());
    assert!(hand_visible(enemy_bias, friend_view, false, None).unwrap());
    assert!(!hand_visible(enemy_bias, friend_view, true, None).unwrap());
    assert!(!hand_visible(closed, friend_view, true, Some(2)).unwrap());
    assert!(hand_visible(vhard, friend_view, false, Some(1)).unwrap());
    assert!(!hand_visible(vhard, friend_view, true, Some(1)).unwrap());
    assert!(!hand_visible(vhard, friend_view, false, Some(0)).unwrap());

    let mut heavy = config.clone();
    heavy.time_killer = "heavy".into();
    let thinking = think_delay(&heavy, true, 0.5).unwrap();
    assert_eq!(thinking.delay, 2.25);
    assert!(thinking.chat_refused);
    assert!(!thinking.slept);
    assert_eq!(think_delay(&heavy, false, 0.5).unwrap().delay, 0.0);
    assert_eq!(think_delay(&config, true, 0.5).unwrap().delay, 0.0);
    heavy.time_killer = "light".into();
    assert_eq!(think_delay(&heavy, true, 1.0).unwrap().delay, 1.5);
    heavy.time_killer = "middle".into();
    assert_eq!(think_delay(&heavy, true, 0.0).unwrap().delay, 0.0);
    heavy.time_killer = "loud".into();
    assert_eq!(think_delay(&heavy, true, 1.0).unwrap().delay, 0.0);
    assert!(matches!(think_delay(&config, true, 1.5), Err(EloError::BadRandom(_))));
    assert!(matches!(think_delay(&config, true, -0.01), Err(EloError::BadRandom(_))));
    assert!(matches!(think_delay(&config, true, f64::NAN), Err(EloError::BadRandom(_))));
    assert!(matches!(think_delay(&config, true, f64::INFINITY), Err(EloError::BadRandom(_))));

    let names = ["caocao", "liubei"];
    assert_eq!(restrict_pool(&config, &names), names);
    let mut pool_off = config.clone();
    pool_off.elo_players = false;
    assert_eq!(restrict_pool(&pool_off, &names), names);
    assert!(restrict_pool(&config, &[] as &[&str]).is_empty());

    let commanded = apply_elo_commands(&stored, true, "identity", None, "diff;order:friend;order:enemy;pool").unwrap();
    assert_eq!(commanded, "diff:2 eh:1 top:3 top:2 pool:caocao,liubei");
    let level6 = apply_elo_commands(&stored, true, "identity", Some(6), "diff;order:enemy").unwrap();
    assert_eq!(level6, "diff:4 eh:-1 top:3");
    assert!(apply_elo_commands(&stored, true, "identity", None, "nope").is_err());
    assert_eq!(store.current_lock().expect("lock").lock_hash, lock_before, "公式不改内容锁");

    let canvas_dir = tempfile::tempdir().unwrap();
    assert!(canvas_dir.path().read_dir().unwrap().next().is_none(), "不创建画布或音效文件");
    let asset_names: Vec<_> = std::fs::read_dir(installed_dir.join("assets")).unwrap().map(|item| item.unwrap().file_name()).collect();
    assert_eq!(asset_names, vec![std::ffi::OsString::from("config.json")]);

    match store.activate_version("upstream-liuli", "elo-sanction", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../elo-sanction".into();
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
    match store.install_with_assets(&missing.to_json_bytes().unwrap(), None, &options(), &source_root) {
        Err(RuntimeError::Invalid(msg)) => assert!(msg.contains("missing dependency"), "{msg}"),
        other => panic!("缺依赖应拒绝: {other:?}"),
    }
    assert_eq!(active_version(&store, "upstream-liuli", "elo-sanction"), "1.0.0");
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0", "失败安装不撤掉 liuli 金币");

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("elo-sanction-disabled");
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
        assert!(!disabled.installed.iter().any(|item| item.namespace == "upstream-liuli" && item.package_id == "elo-sanction"));
        assert!(disabled.skipped.iter().any(|name| name == "elo-sanction-disabled"));
    }

    let mut upgraded = installed.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    store.install_with_assets(&upgraded.to_json_bytes().unwrap(), None, &options(), &source_root).expect("升级");
    assert_eq!(active_version(&store, "upstream-liuli", "elo-sanction"), "2.0.0");
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0", "升级段位制裁不改 liuli 金币");
    assert_eq!(active_version(&store, "upstream-noname", "coin"), "1.0.0", "升级段位制裁不改 noname 金币");
    let rolled = store.rollback().expect("回滚");
    assert!(rolled.packages.iter().any(|(slot, version)| slot.ends_with("upstream-liuli/elo-sanction") && version == "1.0.0"));
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0", "回滚后 liuli 金币仍是 1.0.0");

    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(active_version(&reopened, "upstream-liuli", "elo-sanction"), "1.0.0");
    let (_, kept) = read_installed(&reopened, "upstream-liuli", "elo-sanction", "1.0.0");
    let kept_text = std::fs::read_to_string(kept.join("assets/config.json")).unwrap();
    let kept_config = parse_elo_config(&kept_text).unwrap();
    assert_eq!(kept_config.source, "noname-liuli");
    assert_eq!(kept_config.difficulty, "paiwei");
    assert!(kept_config.elo_players);
    assert!(kept_text.contains("\"source\": \"noname-liuli\""));
}
