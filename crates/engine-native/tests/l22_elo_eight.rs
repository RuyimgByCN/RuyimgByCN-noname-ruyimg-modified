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

fn new_l22_engine(seed: u64, seats: usize, diff_key: &str, tk_key: &str, online: bool) -> engine_native::Engine {
    let pack = content_native_loader::elo_sanction_extension_pack();
    let packs = [&pack];
    let mut engine = engine_native::Engine::new_with_extensions(seed, seats, &packs).expect("l22 engine");
    let cfg = content_native_loader::EloConfig {
        source: "noname-liuli".to_string(),
        elo_players: true,
        difficulty: diff_key.to_string(),
        time_killer: tk_key.to_string(),
    };
    let query = EloQuery {
        online,
        mode: "identity".to_string(),
        paiwei_level: None,
    };
    content_native_loader::elo_apply_engine_config(&mut engine, &cfg, &query);
    engine
}

fn find_card_of_kind(
    engine: &engine_native::Engine,
    kind: engine_native::catalog::CardKind,
) -> engine_native::ids::CardId {
    engine
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == kind)
        .expect("card kind exists") as engine_native::ids::CardId
}

fn submit_action(
    engine: &mut engine_native::Engine,
    dedup: &mut u64,
    actor: u8,
    action: engine_native::GameAction,
) {
    *dedup += 1;
    let outcome = engine.submit(engine_native::GameCommand {
        actor,
        expected_revision: engine.revision(),
        dedup_id: *dedup,
        action,
    });
    assert!(
        !matches!(outcome, engine_native::CommandOutcome::Rejected(_)),
        "unexpected rejection: {outcome:?}"
    );
}

fn advance_to_play_phase(engine: &mut engine_native::Engine, dedup: &mut u64, seat: u8) {
    for _ in 0..24 {
        if engine.turn().current == seat
            && engine.turn().phase == engine_native::event::Phase::Play
            && engine.open_choice().is_none()
        {
            return;
        }
        if let Some(ch) = engine.open_choice().cloned() {
            let sel: Vec<engine_native::ChoiceOption> = ch
                .spec
                .options
                .iter()
                .take(ch.spec.min as usize)
                .copied()
                .collect();
            submit_action(
                engine,
                dedup,
                ch.observer,
                engine_native::GameAction::Answer {
                    choice_id: ch.id,
                    selection: sel,
                },
            );
        } else {
            let cur = engine.turn().current;
            submit_action(engine, dedup, cur, engine_native::GameAction::NextPhase);
        }
    }
}

/// Dimension ①: Trigger timing & negative conditions across all 8 L22 items in real Engine matches.
#[test]
fn l22_dim1_trigger_timing_and_negative_conditions_real_match() {
    use engine_native::catalog::CardKind;
    use engine_native::kernel::{
        elo_get_global_scalar, elo_get_player_scalar, elo_is_card_known, elo_on_phase_before,
        elo_set_card_known, elo_set_global_scalar, elo_set_player_scalar, elo_try_init_cards,
        elo_try_init_mechanism, L22_ELO_SKILLS,
    };

    let mut engine = new_l22_engine(101, 2, "normal", "none", true);
    let mut dedup = 0u64;

    // Verify all 5 L22 skills are registered in catalog and granted to all seats via extension_global_skills.
    for &(skill_name, expected_ev) in &L22_ELO_SKILLS {
        let def = engine
            .catalog()
            .skill_by_name(skill_name)
            .unwrap_or_else(|| panic!("missing L22 skill {skill_name}"));
        assert_eq!(
            engine.catalog().skills[def as usize]
                .trigger
                .as_ref()
                .unwrap()
                .event,
            expected_ev
        );
        assert!(engine.state_core().has_skill(0, def));
        assert!(engine.state_core().has_skill(1, def));
    }

    // 1. `_eloMechanism` triggered once on `gameStart` for `game.me` (seat 0), usable: 1.
    assert_eq!(elo_get_global_scalar(engine.state_core(), "elo_has_init_function"), Some(1));
    assert_eq!(elo_get_player_scalar(engine.state_core(), 0, "elo_mechanism_init_count"), Some(1));
    assert_eq!(elo_get_player_scalar(engine.state_core(), 1, "elo_mechanism_init_count"), None);
    // Negative: calling `elo_try_init_mechanism` again on seat 0 or seat 1 returns false.
    let cat = engine.catalog().clone();
    assert!(!elo_try_init_mechanism(engine.state_core_mut(), &cat, 0));
    assert!(!elo_try_init_mechanism(engine.state_core_mut(), &cat, 1));
    assert_eq!(elo_get_player_scalar(engine.state_core(), 0, "elo_mechanism_init_count"), Some(1));

    // 2. `_eloInitCards` marked initial hand cards of seat 0 and seat 1 at gameStart (`enterGame`).
    for seat in [0u8, 1u8] {
        assert_eq!(elo_get_player_scalar(engine.state_core(), seat, "elo_init_cards_count"), Some(4));
        for &c in &engine.state_core().player(seat).hand {
            assert!(elo_is_card_known(engine.state_core(), c));
        }
    }
    // Negative: when `phase_number > 0`, `phaseBefore` does NOT mark newly added unknown hand cards.
    let extra_card = engine.state_core_mut().deck.pop().unwrap();
    elo_set_card_known(engine.state_core_mut(), extra_card, false);
    engine.state_core_mut().player_mut(0).hand.push(extra_card);
    engine.state_core_mut().turn.phase_number = 1;
    elo_on_phase_before(engine.state_core_mut(), &cat, 0);
    assert!(!elo_is_card_known(engine.state_core(), extra_card));
    engine.state_core_mut().player_mut(0).hand.pop();
    engine.state_core_mut().deck.push(extra_card);

    // Negative: when `hand` and `equips` are empty (even if `judge` area has a card), `_eloInitCards` returns 0.
    let saved_hand = std::mem::take(&mut engine.state_core_mut().player_mut(1).hand);
    let judge_dummy = engine.state_core_mut().deck.pop().unwrap();
    elo_set_card_known(engine.state_core_mut(), judge_dummy, false);
    engine.state_core_mut().player_mut(1).judge.push(judge_dummy);
    engine.state_core_mut().turn.phase_number = 0;
    assert_eq!(elo_try_init_cards(engine.state_core_mut(), &cat, 1, true), 0);
    assert!(!elo_is_card_known(engine.state_core(), judge_dummy));
    engine.state_core_mut().player_mut(1).judge.pop();
    engine.state_core_mut().deck.push(judge_dummy);
    engine.state_core_mut().player_mut(1).hand = saved_hand;

    // 3. Negative for `_eloDraw` and `_timeKiller`: under `eloDifficulty: "normal"` (`eh == 0`) and `timeKiller: "none"`,
    // stepping through Judge/Draw/Play phases triggers neither `_eloDraw` nor `_timeKiller`.
    let deck_top_two_before: Vec<u32> = engine
        .state_core()
        .deck
        .iter()
        .rev()
        .take(2)
        .copied()
        .collect();
    advance_to_play_phase(&mut engine, &mut dedup, 0);
    assert_eq!(elo_get_player_scalar(engine.state_core(), 0, "elo_draw_triggers"), None);
    assert_eq!(elo_get_player_scalar(engine.state_core(), 0, "elo_judge_triggers"), None);
    assert_eq!(elo_get_player_scalar(engine.state_core(), 0, "elo_time_killer_triggers"), None);
    let drawn_two: Vec<u32> = engine.state_core().player(0).hand.iter().rev().take(2).copied().collect();
    // Since two cards were popped in order, the last drawn is deck_top_two_before[1] and second-to-last is [0].
    assert_eq!(drawn_two, vec![deck_top_two_before[1], deck_top_two_before[0]]);

    // 4. Negative for `_eloHandcard`:
    // - When `source == game.me` (seat 0), or `diff < 3` ("hard", diff = 2), or `diff == 3` ("vhard") with roll = 0,
    //   `Dismantle` does NOT set `elo_handcard_visible = 1` and picks target's first hand card (`Sha`) instead of `Tao`.
    let cfg_vhard = content_native_loader::EloConfig {
        source: "noname-liuli".to_string(),
        elo_players: true,
        difficulty: "vhard".to_string(),
        time_killer: "heavy".to_string(),
    };
    // Offline query disables both `paiwei` and `timeKiller`.
    content_native_loader::elo_apply_engine_config(
        &mut engine,
        &cfg_vhard,
        &EloQuery {
            online: false,
            mode: "identity".to_string(),
            paiwei_level: None,
        },
    );
    // Advance to seat 1's Play phase; with `online: false`, `_timeKiller` must NOT trigger on seat 1.
    submit_action(&mut engine, &mut dedup, 0, engine_native::GameAction::NextPhase);
    advance_to_play_phase(&mut engine, &mut dedup, 1);
    assert_eq!(elo_get_player_scalar(engine.state_core(), 1, "elo_time_killer_triggers"), None);

    // Set `elo_handcard_roll = 0` on seat 1 under `vhard` (`diff = 3`, `eh = -1`): 2/3 roll miss -> blind first card (`Sha`).
    elo_set_global_scalar(engine.state_core_mut(), "elo_online", 1);
    elo_set_player_scalar(engine.state_core_mut(), 1, "elo_handcard_roll", 0);
    let sha_id = find_card_of_kind(&engine, CardKind::Sha);
    let tao_id = find_card_of_kind(&engine, CardKind::Tao);
    let dismantle_id = find_card_of_kind(&engine, CardKind::Dismantle);
    engine.state_core_mut().player_mut(0).hand = vec![sha_id, tao_id];
    engine.state_core_mut().player_mut(1).hand = vec![dismantle_id];
    submit_action(
        &mut engine,
        &mut dedup,
        1,
        engine_native::GameAction::PlayCard {
            card: dismantle_id,
            targets: vec![0],
        },
    );
    assert_eq!(elo_get_player_scalar(engine.state_core(), 1, "elo_handcard_visible"), Some(0));
    assert_eq!(engine.state_core().player(0).hand, vec![tao_id], "blind pick must discard index 0 (Sha), leaving Tao");
}

/// Dimension ②: Target / tier filter & window limits across all `eloDifficulty` tiers,
/// `eloPlayers` scopes, and `_eloDraw` (`drawBegin` & `judgeBegin`) in real matches.
#[test]
fn l22_dim2_tier_filters_and_window_limits_real_match() {
    use engine_native::catalog::{CardKind, Suit};
    use engine_native::kernel::{
        elo_get_player_scalar, elo_is_card_known, elo_mark_deck_insert, elo_on_draw_begin,
        elo_on_judge_begin, elo_set_card_known,
    };

    // 1. Enemy draw in `ehard` (`diff = 4`, `eh = -1`):
    // Top `2 * 4 = 8` cards are reordered so unknown high-value cards move to top while `isEloKnown` slot stays locked!
    let mut engine = new_l22_engine(202, 2, "ehard", "none", true);
    let cat = engine.catalog().clone();

    let tao_id = find_card_of_kind(&engine, CardKind::Tao);
    let snatch_id = find_card_of_kind(&engine, CardKind::Snatch);
    let shandian_id = find_card_of_kind(&engine, CardKind::ShanDian);
    let chain_id = find_card_of_kind(&engine, CardKind::Chain);
    let sha_cards: Vec<u32> = engine
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == CardKind::Sha)
        .take(4)
        .map(|(i, _)| i as u32)
        .collect();

    // Build top 8 cards on `core.deck` (remember: `deck.last()` is top[0], `deck[len - 8]` is top[7]):
    // Put `shandian_id` at top[0] (marked as `isEloKnown` via `elo_mark_deck_insert`),
    // `chain_id` at top[1] (unknown), `sha_cards` at top[2..6] (unknown), and `tao_id`, `snatch_id` at top[6], top[7] (unknown).
    for &c in &[tao_id, snatch_id, shandian_id, chain_id] {
        elo_set_card_known(engine.state_core_mut(), c, false);
    }
    for &c in &sha_cards {
        elo_set_card_known(engine.state_core_mut(), c, false);
    }
    let top_eight_in_draw_order = [
        shandian_id, // top[0]: known (locked!)
        chain_id,    // top[1]: unknown (low value 55)
        sha_cards[0],
        sha_cards[1],
        sha_cards[2],
        sha_cards[3],
        snatch_id, // top[6]: unknown (value 80)
        tao_id,    // top[7]: unknown (value 90)
    ];
    engine
        .state_core_mut()
        .deck
        .retain(|c| !top_eight_in_draw_order.contains(c));
    for &c in top_eight_in_draw_order.iter().rev() {
        engine.state_core_mut().deck.push(c);
    }
    // Mark `shandian_id` via `_eloMechanism`'s deck-insert hook (`no_mark = false`).
    elo_mark_deck_insert(engine.state_core_mut(), shandian_id, false);
    assert!(elo_is_card_known(engine.state_core(), shandian_id));

    // Trigger `_eloDraw` for enemy `seat 1` drawing 2 cards (`window = 2 * 4 = 8`):
    let sorted_n = elo_on_draw_begin(engine.state_core_mut(), &cat, 1, 2);
    assert_eq!(sorted_n, 8);
    assert_eq!(elo_get_player_scalar(engine.state_core(), 1, "elo_draw_last_window"), Some(8));
    let deck_len = engine.state_core().deck.len();
    // Slot 0 (`deck[deck_len - 1]`) was `isEloKnown`, so `shandian_id` stays locked at slot 0!
    // Slot 1 (`deck[deck_len - 2]`) and Slot 2 (`deck[deck_len - 3]`) receive `tao_id` (90) and `snatch_id` (80)!
    assert_eq!(engine.state_core().deck[deck_len - 1], shandian_id);
    assert_eq!(engine.state_core().deck[deck_len - 2], tao_id);
    assert_eq!(engine.state_core().deck[deck_len - 3], snatch_id);
    // Reordered cards must NOT be marked `isEloKnown` (`noMark = true`).
    assert!(!elo_is_card_known(engine.state_core(), tao_id));
    assert!(!elo_is_card_known(engine.state_core(), snatch_id));

    // Now test `game.me` (`seat 0`, friend) drawing 2 cards under `ehard` (`eh = -1`, `diff = 4`):
    // Unlock `shandian_id` so all 8 are unknown; `seat 0` must get the WORST cards (`shandian_id` 30, `chain_id` 55) at top[0], top[1]!
    elo_set_card_known(engine.state_core_mut(), shandian_id, false);
    let _ = elo_on_draw_begin(engine.state_core_mut(), &cat, 0, 2);
    assert_eq!(engine.state_core().deck[deck_len - 1], shandian_id);
    assert_eq!(engine.state_core().deck[deck_len - 2], chain_id);

    // 2. Judge window (`judgeBegin`) for `ShanDian` on `seat 0` (`game.me`) vs `Le` on `seat 1` (enemy) under `ehard` (`diff = 4, eh = -1`):
    let spade_bomb = engine
        .catalog()
        .cards
        .iter()
        .position(|c| c.suit == Suit::Spade && (2..=9).contains(&c.rank))
        .unwrap() as u32;
    let safe_hearts: Vec<u32> = engine
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.suit == Suit::Heart)
        .take(3)
        .map(|(i, _)| i as u32)
        .collect();
    for &c in std::iter::once(&spade_bomb).chain(safe_hearts.iter()) {
        elo_set_card_known(engine.state_core_mut(), c, false);
    }
    // Put 3 safe hearts at top[0..3] and `spade_bomb` at top[3] (4th card from top).
    let judge_four = [safe_hearts[0], safe_hearts[1], safe_hearts[2], spade_bomb];
    engine.state_core_mut().deck.retain(|c| !judge_four.contains(c));
    for &c in judge_four.iter().rev() {
        engine.state_core_mut().deck.push(c);
    }
    // Under `ehard` (`eh = -1`), when `seat 0` (`game.me`) judges `ShanDian`, `_eloDraw` moves `spade_bomb` (-50) to top[0]!
    let j_window = elo_on_judge_begin(engine.state_core_mut(), &cat, 0, CardKind::ShanDian);
    assert_eq!(j_window, 4);
    let dlen = engine.state_core().deck.len();
    assert_eq!(
        engine.state_core().deck[dlen - 1],
        spade_bomb,
        "ehard sanctions game.me by moving Spade 2..9 lightning hit to top of deck"
    );

    // Conversely, when enemy `seat 1` judges `Le` with `spade_bomb` at top[0] and `safe_hearts[0]` at top[3],
    // `_eloDraw` moves `safe_hearts[0]` (+20) to top[0] so enemy passes `Le`!
    let le_four = [spade_bomb, safe_hearts[0], safe_hearts[1], safe_hearts[2]];
    engine.state_core_mut().deck.retain(|c| !le_four.contains(c));
    for &c in le_four.iter().rev() {
        engine.state_core_mut().deck.push(c);
    }
    let _ = elo_on_judge_begin(engine.state_core_mut(), &cat, 1, CardKind::Le);
    assert_eq!(
        engine.catalog().cards[engine.state_core().deck[dlen - 1] as usize].suit,
        Suit::Heart,
        "ehard helps enemy pass Lebusishu by placing Heart at top of deck"
    );
}

/// Dimension ③: Player choice / action window interaction (`_eloHandcard` visible extraction
/// and `_timeKiller` turn choice timer delay & `zuan` chat lines in real match windows).
#[test]
fn l22_dim3_choice_and_action_windows_handcard_and_time_killer() {
    use engine_native::catalog::CardKind;
    use engine_native::kernel::{
        elo_get_player_scalar, elo_set_global_scalar, elo_set_player_scalar,
    };

    let mut engine = new_l22_engine(303, 2, "ehard", "heavy", true);
    let mut dedup = 0u64;
    elo_set_global_scalar(engine.state_core_mut(), "elo_zuan_enable", 1);
    elo_set_player_scalar(engine.state_core_mut(), 1, "elo_time_killer_ran_permille", 800);
    elo_set_player_scalar(engine.state_core_mut(), 1, "elo_time_killer_chat_idx", 2);
    elo_set_player_scalar(engine.state_core_mut(), 1, "elo_sex_female", 1);

    // Advance through seat 0's turn to seat 1's Play phase.
    advance_to_play_phase(&mut engine, &mut dedup, 0);
    submit_action(&mut engine, &mut dedup, 0, engine_native::GameAction::NextPhase);
    advance_to_play_phase(&mut engine, &mut dedup, 1);

    // Entering seat 1's Play phase (`chooseToUseBefore`) triggered `_timeKiller` for AI seat 1:
    // `delay_ms = (3 * 1500 * 800) / 1000 = 3600`, and female chat line 2 (`ran >= 750`).
    assert_eq!(
        elo_get_player_scalar(engine.state_core(), 1, "elo_time_killer_delay_ms"),
        Some(3600)
    );
    assert_eq!(
        elo_get_player_scalar(engine.state_core(), 1, "elo_time_killer_chat_female"),
        Some(2)
    );
    let tk_before = elo_get_player_scalar(engine.state_core(), 1, "elo_time_killer_triggers").unwrap_or(0);
    assert!(tk_before >= 1);

    // Seat 0 (`game.me`) has `[Sha (index 0, value 60), Tao (index 1, value 90)]`.
    // Seat 1 (enemy AI) plays `Snatch` (`ShunShou`) on seat 0.
    // Under `ehard` (`diff = 4, eh = -1`), `_eloHandcard` sets `elo_handcard_visible = 1`
    // and extracts `Tao` (index 1) instead of blind `Sha` (index 0)!
    let sha_id = find_card_of_kind(&engine, CardKind::Sha);
    let tao_id = find_card_of_kind(&engine, CardKind::Tao);
    let snatch_id = find_card_of_kind(&engine, CardKind::Snatch);
    engine.state_core_mut().player_mut(0).hand = vec![sha_id, tao_id];
    engine.state_core_mut().player_mut(1).hand = vec![snatch_id];

    submit_action(
        &mut engine,
        &mut dedup,
        1,
        engine_native::GameAction::PlayCard {
            card: snatch_id,
            targets: vec![0],
        },
    );

    assert_eq!(
        elo_get_player_scalar(engine.state_core(), 1, "elo_handcard_visible"),
        Some(1)
    );
    assert_eq!(
        elo_get_player_scalar(engine.state_core(), 1, "elo_handcard_triggers"),
        Some(1)
    );
    assert_eq!(
        engine.state_core().player(0).hand,
        vec![sha_id],
        "Tao (index 1) must be extracted by _eloHandcard visibility, leaving Sha (index 0)"
    );
    assert!(
        engine.state_core().player(1).hand.contains(&tao_id),
        "seat 1 must gain Tao from Snatch"
    );

    // Now give seat 1 extra hand cards above its HP (e.g. 6 cards vs 4 HP) and advance to Discard phase:
    // Opening the `discard_cards` choice window on seat 1 triggers `_timeKiller` (`chooseToDiscardBefore`).
    let extra_cards: Vec<u32> = engine.state_core_mut().deck.drain(..5).collect();
    engine.state_core_mut().player_mut(1).hand.extend(extra_cards);
    elo_set_player_scalar(engine.state_core_mut(), 1, "elo_sex_female", 0);
    elo_set_player_scalar(engine.state_core_mut(), 1, "elo_time_killer_ran_permille", 900);
    let tk_pre_discard =
        elo_get_player_scalar(engine.state_core(), 1, "elo_time_killer_triggers").unwrap_or(0);
    submit_action(&mut engine, &mut dedup, 1, engine_native::GameAction::NextPhase);
    assert!(engine.open_choice().is_some(), "seat 1 must open discard choice window");
    let tk_post_discard =
        elo_get_player_scalar(engine.state_core(), 1, "elo_time_killer_triggers").unwrap_or(0);
    assert!(tk_post_discard > tk_pre_discard);
    assert_eq!(
        elo_get_player_scalar(engine.state_core(), 1, "elo_time_killer_delay_ms"),
        Some((3 * 1500 * 900) / 1000)
    );
    assert_eq!(
        elo_get_player_scalar(engine.state_core(), 1, "elo_time_killer_chat_male"),
        Some(2)
    );
}

/// Dimension ④: Multi-step state changes across a real match turn (`GameStart` `_eloMechanism` +
/// `_eloInitCards` -> `Phase::Judge` `_eloDraw` -> `Phase::Draw` `_eloDraw` -> `Phase::Play`
/// `_eloHandcard` + `_timeKiller` -> deck exhaustion `washCard` clearing `isEloKnown`).
#[test]
fn l22_dim4_multi_step_turn_chain_and_wash_deck_real_match() {
    use engine_native::catalog::{CardKind, Suit};
    use engine_native::kernel::{
        elo_get_global_scalar, elo_get_player_scalar, elo_is_card_known, elo_set_card_known,
    };

    let mut engine = new_l22_engine(404, 2, "eeasy", "light", true);
    let mut dedup = 0u64;
    // Advance to seat 1's Play phase so the next turn transition enters seat 0's `Phase::Judge (begin: true)` from scratch.
    advance_to_play_phase(&mut engine, &mut dedup, 0);
    submit_action(&mut engine, &mut dedup, 0, engine_native::GameAction::NextPhase);
    advance_to_play_phase(&mut engine, &mut dedup, 1);

    // In `eeasy` (`diff = 4, eh = 1`), `seat 0` (`game.me`) is helped:
    // - `Phase::Judge` on `Le` places a Heart card at top of deck so `seat 0` passes `Le` and enters `Phase::Play`.
    // - `Phase::Draw` places the highest-value unknown cards (`Tao`, `Snatch`) at top of deck for `seat 0`.
    let le_id = find_card_of_kind(&engine, CardKind::Le);
    let heart_judge = engine
        .catalog()
        .cards
        .iter()
        .position(|c| c.suit == Suit::Heart && c.kind == CardKind::Sha)
        .unwrap() as u32;
    let spade_judge = engine
        .catalog()
        .cards
        .iter()
        .position(|c| c.suit == Suit::Spade && c.kind == CardKind::Sha)
        .unwrap() as u32;
    let tao_id = find_card_of_kind(&engine, CardKind::Tao);
    let snatch_id = find_card_of_kind(&engine, CardKind::Snatch);
    let chain_id = find_card_of_kind(&engine, CardKind::Chain);

    for &c in &[heart_judge, spade_judge, tao_id, snatch_id, chain_id, le_id] {
        elo_set_card_known(engine.state_core_mut(), c, false);
        engine.state_core_mut().deck.retain(|&x| x != c);
        engine.state_core_mut().discard.retain(|&x| x != c);
        for p in 0..2 {
            engine.state_core_mut().player_mut(p).hand.retain(|&x| x != c);
        }
    }
    // Put `Le` in seat 0's judge area.
    engine.state_core_mut().player_mut(0).judge.push(le_id);

    // Arrange top of deck:
    // For Judge (`window = 4`): `spade_judge` at top[0], `heart_judge` at top[1], `chain_id` at top[2], `tao_id` at top[3], `snatch_id` at top[4].
    // `_eloDraw` `judgeBegin` will move `heart_judge` (+20) to top[0] so `Le` passes!
    // Then `Phase::Draw` (`window = 2 * 4 = 8`) will move `tao_id` (90) and `snatch_id` (80) to top[0..2] so `seat 0` draws both!
    let top_five = [spade_judge, heart_judge, chain_id, tao_id, snatch_id];
    for &c in top_five.iter().rev() {
        engine.state_core_mut().deck.push(c);
    }

    submit_action(&mut engine, &mut dedup, 1, engine_native::GameAction::NextPhase);
    advance_to_play_phase(&mut engine, &mut dedup, 0);
    assert_eq!(engine.turn().current, 0);
    assert_eq!(engine.turn().phase, engine_native::event::Phase::Play);
    assert_eq!(elo_get_player_scalar(engine.state_core(), 0, "elo_judge_triggers"), Some(1));
    assert_eq!(elo_get_player_scalar(engine.state_core(), 0, "elo_draw_triggers"), Some(2));
    assert!(engine.state_core().player(0).hand.contains(&tao_id));
    assert!(engine.state_core().player(0).hand.contains(&snatch_id));

    // Now test `_eloMechanism`'s `washCard` hook during a real draw when `core.deck` is empty and `core.discard` has `isEloKnown` cards!
    let discard_cards: Vec<u32> = engine.state_core_mut().deck.drain(..).collect();
    for &c in &discard_cards {
        elo_set_card_known(engine.state_core_mut(), c, true);
    }
    engine.state_core_mut().discard.extend(discard_cards.iter().copied());
    assert!(elo_is_card_known(engine.state_core(), discard_cards[0]));

    // Advance to seat 1's turn so seat 1's `Phase::Draw` triggers a deck reshuffle (`washCard`)!
    submit_action(&mut engine, &mut dedup, 0, engine_native::GameAction::NextPhase);
    advance_to_play_phase(&mut engine, &mut dedup, 1);

    assert_eq!(
        elo_get_global_scalar(engine.state_core(), "elo_wash_count"),
        Some(1),
        "reshuffling discard into deck must trigger _eloMechanism washCard hook"
    );
    for &c in &discard_cards {
        assert!(
            !elo_is_card_known(engine.state_core(), c),
            "washCard must clear isEloKnown on all reshuffled cards"
        );
    }
}

/// Dimension ⑤: Archive save/restore (`encode_archive` -> `decode_archive` -> `restore_with_extensions`)
/// preserves all L22 state, `isEloKnown` marks, config tiers, and deterministic `state_hash`.
#[test]
fn l22_dim5_archive_save_restore_preserves_l22_state_and_hash() {
    use engine_native::archive::{decode_archive, encode_archive};
    use engine_native::catalog::CardKind;
    use engine_native::kernel::{
        elo_get_global_scalar, elo_get_player_scalar, elo_is_card_known, elo_mark_deck_insert,
    };

    let pack = content_native_loader::elo_sanction_extension_pack();
    let packs = [&pack];
    let mut engine = new_l22_engine(505, 2, "ehard", "middle", true);
    let mut dedup = 0u64;
    advance_to_play_phase(&mut engine, &mut dedup, 0);

    let marked_card = engine.state_core().deck[0];
    elo_mark_deck_insert(engine.state_core_mut(), marked_card, false);
    assert!(elo_is_card_known(engine.state_core(), marked_card));

    let hash_before = engine.state_hash();
    let snap = engine.save();
    let bytes = encode_archive(&snap);
    let decoded = decode_archive(&bytes).expect("decode l22 archive");
    let mut restored =
        engine_native::Engine::restore_with_extensions(&decoded, &packs).expect("restore l22 engine");

    assert_eq!(restored.state_hash(), hash_before);
    assert_eq!(elo_get_global_scalar(restored.state_core(), "elo_has_init_function"), Some(1));
    assert_eq!(elo_get_global_scalar(restored.state_core(), "elo_eh"), Some(-1));
    assert_eq!(elo_get_global_scalar(restored.state_core(), "elo_difficulty_val"), Some(4));
    assert_eq!(elo_get_global_scalar(restored.state_core(), "elo_time_killer_val"), Some(2));
    assert!(elo_is_card_known(restored.state_core(), marked_card));

    // Step both `engine` and `restored` through seat 1's turn with a `Dismantle` (`_eloHandcard`) and verify identical `state_hash`.
    let sha_id = find_card_of_kind(&engine, CardKind::Sha);
    let tao_id = find_card_of_kind(&engine, CardKind::Tao);
    let dismantle_id = find_card_of_kind(&engine, CardKind::Dismantle);

    let base_dedup = dedup;
    for e in [&mut engine, &mut restored] {
        let mut d = base_dedup;
        submit_action(e, &mut d, 0, engine_native::GameAction::NextPhase);
        advance_to_play_phase(e, &mut d, 1);
        e.state_core_mut().player_mut(0).hand = vec![sha_id, tao_id];
        e.state_core_mut().player_mut(1).hand = vec![dismantle_id];
        submit_action(
            e,
            &mut d,
            1,
            engine_native::GameAction::PlayCard {
                card: dismantle_id,
                targets: vec![0],
            },
        );
    }

    assert_eq!(
        elo_get_player_scalar(restored.state_core(), 1, "elo_handcard_visible"),
        Some(1)
    );
    assert_eq!(restored.state_core().player(0).hand, vec![sha_id]);
    assert_eq!(restored.state_hash(), engine.state_hash());
}

/// Dimension ⑥: AI `BoundedPlanner` in an L22-enabled Identity match completes legally
/// without reading unrevealed identities/cards or leaking `elo_known:*` deck knowledge.
#[test]
fn l22_dim6_ai_bounded_planner_advise_redacts_elo_known_and_completes_legally() {
    use engine_native::ai::{redact_opponent_hands, BoundedPlanner};
    use engine_native::kernel::elo_is_card_known;

    let mut engine = new_l22_engine(606, 4, "ehard", "middle", true);
    let mut dedup = 0u64;

    // Precondition: initial hand cards are marked `elo_known:*` in authoritative engine state.
    let seat0_card = engine.state_core().player(0).hand[0];
    assert!(elo_is_card_known(engine.state_core(), seat0_card));

    // Verify `redact_opponent_hands` scrubs `elo_known:*` for ALL seats (including seat 0!).
    for observer in 0..4u8 {
        let mut redacted = engine.clone();
        redact_opponent_hands(&mut redacted, observer);
        assert!(
            !redacted
                .state_core()
                .vm_aux
                .flags
                .keys()
                .any(|(_, k)| k.starts_with("elo_known:")),
            "observer {observer} redacted view must not retain elo_known:* marks"
        );
    }

    // Step through real match decisions using `BoundedPlanner` and verify every advised action is legal.
    let planner = BoundedPlanner::new(32);
    for _ in 0..12 {
        if engine.outcome().is_some() {
            break;
        }
        let actor = engine
            .open_choice()
            .map(|c| c.observer)
            .unwrap_or_else(|| engine.turn().current);
        let hash_before = engine.state_hash();
        let plan = engine.plan_action(actor, &planner);
        assert_eq!(
            engine.state_hash(),
            hash_before,
            "BoundedPlanner must not mutate authoritative engine state"
        );
        submit_action(&mut engine, &mut dedup, actor, plan.action);
    }
}

