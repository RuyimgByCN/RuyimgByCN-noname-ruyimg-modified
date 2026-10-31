//! L05 金币八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/coin`。余额、商店、下注和终局
//! 公式走同一 `coin` 服务，但本测试的主体是 liuli 商店副本，不把 U04 再计一次。
//! 画布、音效、提示、CSS 和粒子没有迁入。`unit_random` 由调用方给出。

use std::path::PathBuf;

use content_native_loader::{
    buy_item, change_coin, mark_phase_started, open_coin_session, parse_coin_config,
    persist_config, place_bet, restore_coin, settle_for, snapshot_coin, start_item, stop_item,
    toggle_snow_fall, toggle_snow_size, CoinDisplay, CoinError, CoinGate, CoinMode, IdentityRole,
    SettleRequest, ShopItem, SnowSize,
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

fn play() -> CoinGate {
    CoinGate {
        mode: CoinMode::Identity,
        chess_leader: false,
        video: false,
        connect: false,
        online: false,
        phase_started: false,
    }
}

fn request(victory: bool, unit: f64) -> SettleRequest {
    SettleRequest {
        victory,
        role: IdentityRole::Zhu,
        enhance_zhu: false,
        alive: 8,
        dead: 0,
        ye: false,
        population: 1,
        friends: None,
        chess_combat_extra: false,
        unit_random: unit,
        coin_coeff: None,
        additional_reward: 0,
    }
}

/// L05 八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l05_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let source_root = repo_extensions_root().join("coin-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text =
        std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "coin" && item.namespace == "upstream-liuli")
        .expect("liuli 金币已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes),
        "商店 content_hash 来自 liuli npack"
    );
    let noname_meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "coin" && item.namespace == "upstream-noname")
        .expect("组合：noname 金币仍在");
    assert_eq!(noname_meta.version, "1.0.0");
    let noname_npack =
        std::fs::read(repo_extensions_root().join("coin-upstream-noname/package.npack.json"))
            .unwrap();
    assert_eq!(
        noname_meta.content_hash,
        content_runtime::content_hash_hex(&noname_npack),
        "不改写 U04 npack"
    );

    let (installed, installed_dir) = read_installed(&store, "upstream-liuli", "coin", "1.0.0");
    assert!(
        installed.manifest.dependencies.is_empty(),
        "依赖：不声明外部扩展"
    );
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());
    assert!(stored.contains("\"source\": \"noname-liuli\""));
    assert!(stored.contains("noname/extension/coin/extension.js"));
    assert!(stored.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
    let config = parse_coin_config(&stored).expect("配置");
    assert_eq!(config.source, "noname-liuli");
    assert_eq!(config.balance, 0);
    assert_eq!(config.display, CoinDisplay::Icon);
    assert!(!config.canvas);
    assert!(!config.snow_fall);
    assert!(!config.snow_size_small);
    assert!(matches!(
        parse_coin_config(&stored.replace("\"noname-liuli\"", "\"\"")),
        Err(CoinError::BadConfig(msg)) if msg.contains("source")
    ));
    assert!(matches!(
        parse_coin_config(&stored.replace("noname-liuli", "bad token")),
        Err(CoinError::BadConfig(_))
    ));
    assert!(
        matches!(parse_coin_config(&stored.replace("\"icon\"", "\"wide\"")), Err(CoinError::BadConfig(msg)) if msg.contains("unknown"))
    );
    assert_eq!(ShopItem::Firework.cost(), 50);
    assert_eq!(ShopItem::Snow.cost(), 20);
    assert_eq!(ShopItem::Blink.cost(), 10);

    let (_, noname_dir) = read_installed(&store, "upstream-noname", "coin", "1.0.0");
    let noname_text = std::fs::read_to_string(noname_dir.join("assets/config.json")).unwrap();
    assert_ne!(noname_text.as_bytes(), stored.as_bytes());
    assert!(!noname_text.contains("noname-liuli"));
    assert_eq!(parse_coin_config(&noname_text).unwrap().source, "noname");
    let lock_before = store.current_lock().expect("lock").lock_hash;

    let mut session = open_coin_session(&config, play());
    assert_eq!(session.source(), "noname-liuli");
    assert!(matches!(
        buy_item(&mut session, ShopItem::Blink),
        Err(CoinError::NotEnough {
            balance: 0,
            cost: 10
        })
    ));
    change_coin(&mut session, 50).unwrap();
    buy_item(&mut session, ShopItem::Snow).unwrap();
    assert_eq!(session.balance(), 30);
    stop_item(&mut session, ShopItem::Snow).unwrap();
    assert!(session.running().next().is_none());
    start_item(&mut session, ShopItem::Snow).unwrap();
    assert!(matches!(
        buy_item(&mut session, ShopItem::Snow),
        Err(CoinError::AlreadyOwned(ShopItem::Snow))
    ));
    assert!(matches!(
        start_item(&mut session, ShopItem::Star),
        Err(CoinError::NotOwned(ShopItem::Star))
    ));
    assert_eq!(toggle_snow_size(&mut session).unwrap(), true);
    assert_eq!(session.snow_size(), SnowSize::Small);

    let reloaded = persist_config(&session, false);
    assert_eq!(reloaded.source, "noname-liuli");
    let again = open_coin_session(&reloaded, play());
    assert_eq!(again.balance(), 30, "重启保留余额");
    assert_eq!(again.source(), "noname-liuli");
    assert!(again.owned().next().is_none(), "重启不保留局内购买");
    let fell = toggle_snow_fall(&again);
    assert!(fell.snow_fall);
    assert_eq!(fell.source, "noname-liuli");
    let snowy = open_coin_session(&fell, play());
    assert!(
        snowy.owned().any(|item| item == ShopItem::Snow),
        "常下雪在重载后视为已购买"
    );
    assert_eq!(snowy.balance(), 30, "常下雪不再扣费");

    let mut bet = open_coin_session(&config, play());
    change_coin(&mut bet, 20).unwrap();
    place_bet(&mut bet).unwrap();
    assert_eq!(bet.balance(), 0);
    assert!(matches!(place_bet(&mut bet), Err(CoinError::AlreadyBet)));
    let won = settle_for(&mut bet, CoinMode::Other, request(true, 0.5)).unwrap();
    assert_eq!(won.payout, 40);
    assert_eq!(won.bet_return, 20);
    assert_eq!(bet.balance(), 60);

    let mut plain = open_coin_session(&config, play());
    assert_eq!(
        settle_for(&mut plain, CoinMode::Identity, request(true, 0.0))
            .unwrap()
            .payout,
        36
    );
    let mut fan = open_coin_session(&config, play());
    let mut fan_req = request(true, 0.0);
    fan_req.role = IdentityRole::Fan;
    assert_eq!(
        settle_for(&mut fan, CoinMode::Identity, fan_req)
            .unwrap()
            .payout,
        27
    );
    let mut loss = open_coin_session(&config, play());
    assert_eq!(
        settle_for(&mut loss, CoinMode::Other, request(false, 0.5))
            .unwrap()
            .payout,
        10
    );
    let mut chess = open_coin_session(&config, play());
    let mut chess_req = request(false, 0.0);
    chess_req.chess_combat_extra = true;
    assert_eq!(
        settle_for(&mut chess, CoinMode::Chess, chess_req)
            .unwrap()
            .payout,
        2
    );
    let mut empty_pop = open_coin_session(&config, play());
    let mut pop = request(true, 0.5);
    pop.population = 0;
    assert!(matches!(
        settle_for(&mut empty_pop, CoinMode::Guozhan, pop),
        Err(CoinError::EmptyPopulation)
    ));
    let mut wild = open_coin_session(&config, play());
    assert!(matches!(
        settle_for(&mut wild, CoinMode::Other, request(true, 1.5)),
        Err(CoinError::BadRandom(_))
    ));

    let mut phased = open_coin_session(&config, play());
    mark_phase_started(&mut phased);
    assert!(matches!(place_bet(&mut phased), Err(CoinError::BetClosed)));
    let mut leader = open_coin_session(
        &config,
        CoinGate {
            mode: CoinMode::Chess,
            chess_leader: true,
            ..play()
        },
    );
    assert!(matches!(
        buy_item(&mut leader, ShopItem::Star),
        Err(CoinError::Inactive)
    ));
    let mut video = open_coin_session(
        &config,
        CoinGate {
            video: true,
            ..play()
        },
    );
    let video_end = settle_for(&mut video, CoinMode::Other, request(true, 0.5)).unwrap();
    assert!(!video_end.persisted);
    assert_eq!(video_end.payout, 30);
    assert_eq!(video.balance(), 0, "录像不把终局金币写入余额");
    let mut connect = open_coin_session(
        &config,
        CoinGate {
            connect: true,
            ..play()
        },
    );
    assert!(matches!(
        settle_for(&mut connect, CoinMode::Other, request(true, 0.5)),
        Err(CoinError::Inactive)
    ));

    let mut saved = open_coin_session(&config, play());
    change_coin(&mut saved, 10).unwrap();
    buy_item(&mut saved, ShopItem::Blink).unwrap();
    let text = snapshot_coin(&saved);
    let restored = restore_coin(&text).unwrap();
    assert_eq!(restored.source(), "noname-liuli");
    assert_eq!(restored.balance(), 0);
    assert!(restored.running().any(|item| item == ShopItem::Blink));
    assert!(
        restore_coin(&text.replace("blink", "nope")).is_err(),
        "保存恢复反例"
    );
    assert!(
        restore_coin(&text.replace("noname-liuli", "bad token")).is_err(),
        "来源字段损坏拒绝"
    );
    assert_eq!(
        store.current_lock().expect("lock").lock_hash,
        lock_before,
        "结算不改内容锁"
    );

    let canvas_dir = tempfile::tempdir().unwrap();
    assert!(
        canvas_dir.path().read_dir().unwrap().next().is_none(),
        "不创建画布文件"
    );

    match store.activate_version("upstream-liuli", "coin", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../coin".into();
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
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0");
    assert_eq!(
        active_version(&store, "upstream-noname", "coin"),
        "1.0.0",
        "失败安装不撤掉 noname 金币"
    );

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("coin-liuli-disabled");
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
            .any(|item| item.namespace == "upstream-liuli" && item.package_id == "coin"));
        assert!(disabled
            .skipped
            .iter()
            .any(|name| name == "coin-liuli-disabled"));
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
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "2.0.0");
    assert_eq!(
        active_version(&store, "upstream-noname", "coin"),
        "1.0.0",
        "升级 liuli 不改 noname"
    );
    let rolled = store.rollback().expect("回滚");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, version)| slot.ends_with("upstream-liuli/coin") && version == "1.0.0"));
    assert_eq!(
        active_version(&store, "upstream-noname", "coin"),
        "1.0.0",
        "回滚后 noname 仍是 1.0.0"
    );

    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(active_version(&reopened, "upstream-liuli", "coin"), "1.0.0");
    let (_, kept) = read_installed(&reopened, "upstream-liuli", "coin", "1.0.0");
    let kept_text = std::fs::read_to_string(kept.join("assets/config.json")).unwrap();
    let kept_config = parse_coin_config(&kept_text).unwrap();
    assert_eq!(kept_config.source, "noname-liuli");
    assert_eq!(kept_config.balance, 0);
    assert!(kept_text.contains("\"source\": \"noname-liuli\""));
}
