//! L21 武皇争霸八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/martial`。选池、座位、战绩、
//! 阵亡替换和强敌过滤走本包配置。不画界面，不采样 `Math.random`，不请求
//! `extension/武皇/mightyFoe.js`，不回放，不联机，不 `eval`，不睡眠。

use std::collections::BTreeSet;
use std::path::PathBuf;

use content_native_loader::{
    ai_order, apply_martial_line, ask_join, assign_seats, attitude, check_result, connect_enabled,
    consider_mighty, double_hp_visible, draft, enemy_battle, filter_pool, format_record,
    free_replace, has_zhu_skill, layout_numbers, mighty_script, on_general_dead, online_choose_runs,
    opening_draw, parse_martial_config, playback_runs, redraw_pool, replacement_draw, reselect_pool,
    take_unique, video_name, DeathQuery, DraftInput, MartialError, MightyPair, RecordBook,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency};

const DEFAULT_LINE: &str = "registered:1 | mode:normal | connect:0 | double:0 | hpRule:hejiansan | hpUi:0 | change:0 | free:0 | show:5 | battle:5 | hide:2 | width:5 | num:5 | hiddenN:4 | size:20 | ratio:1.2 | first:0 | side:3 | zhuSeat:1 | fanSeat:0 | zhuSkill:0 | hidden:a,b,c,d | friend:a,b,c,d,e | enemy:j,i,h,g,f | zhu:j,i,h,g,f | fan:a,b,c,d,e | openZhu:j | openFan:a | drawZhu:4 | drawFan:5 | join:- | hp:0 | foe:0 | script:extension/武皇/mightyFoe.js | scriptRan:0 | online:0 | playback:0 | coeff:- | sampled:0 | dom:0 | sleep:0 | eval:0 | fetch:0 | write:0 | reload:0 | dialog:0";

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn upstream_extension() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../archive/upstream/noname-liuli/noname/extension/武皇争霸/extension.js");
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("读上游 {}", path.display()))
}

fn ids(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
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

fn draft_of(names: &[&str], ranks: &[f64], hidden: &[usize], width: usize, first: u8, skip: bool) -> content_native_loader::DraftOutput {
    draft(&DraftInput {
        names: ids(names),
        ranks: ranks.to_vec(),
        hidden: hidden.to_vec(),
        width,
        first,
        skip,
        player_choices: Vec::new(),
        ai_indices: Vec::new(),
    })
    .expect("draft")
}

#[test]
fn l21_eight_dimension_from_fresh_install() {
    let source = upstream_extension();
    assert!(source.contains("game.addMode('martial'"));
    assert!(source.contains("lib.mode['martial'].connect = false"));
    assert!(source.contains("_status.mode = 'normal'"));
    assert!(source.contains("extension/武皇"));
    assert!(source.contains("//     player.maxHp = maxHp1 + maxHp2 - 3;"));
    assert!(!online_choose_runs());
    assert!(!playback_runs());
    assert!(!connect_enabled());
    let script = mighty_script();
    assert_eq!(script.url, "extension/武皇/mightyFoe.js");
    assert!(!script.ran);
    let sibling = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../archive/upstream/noname-liuli/noname/extension/武皇争霸/mightyFoe.js");
    let sibling_text = std::fs::read_to_string(&sibling).expect("同目录 mightyFoe.js");
    assert!(sibling_text.starts_with("window.mightyFoe"));
    assert!(!source.contains("extension/武皇争霸/mightyFoe.js"));

    let standard = layout_numbers(5, 5, 2);
    assert_eq!(standard.width, 5);
    assert_eq!(standard.num, 5);
    assert_eq!(standard.hidden, 4);
    assert_eq!(standard.height, 4);
    assert_eq!(standard.size, 20);
    assert!((standard.ratio - 1.2).abs() < 1e-9);
    let wide = layout_numbers(2, 6, 8);
    assert_eq!((wide.width, wide.num, wide.hidden, wide.size), (6, 6, 12, 24));
    let shown = layout_numbers(8, 3, 2);
    assert_eq!((shown.width, shown.num, shown.hidden), (8, 3, 4));
    let zero_battle = layout_numbers(5, 0, 2);
    assert_eq!((zero_battle.width, zero_battle.num, zero_battle.hidden), (5, 0, 4));

    let first = assign_seats(0).unwrap();
    assert_eq!((first.zhu, first.fan, first.side, first.zhu_takes_friend), (1, 0, 3, false));
    let second = assign_seats(1).unwrap();
    assert_eq!((second.zhu, second.fan, second.side), (0, 1, 1));
    assert!(second.zhu_takes_friend);
    assert!(matches!(assign_seats(2), Err(MartialError::BadConfig(_))));

    let pool = ids(&["a", "b", "c", "d"]);
    let mut disabled = BTreeSet::new();
    disabled.insert("c".into());
    assert_eq!(filter_pool(&pool, &disabled), ids(&["a", "b", "d"]));
    assert_eq!(take_unique(&pool, &[1, 0]).unwrap(), ids(&["b", "a"]));

    let ranked = ids(&["a", "b", "c"]);
    assert_eq!(
        ai_order(&ranked, &[10.0, 0.0, 100.0], &[2]).unwrap(),
        ids(&["a", "c", "b"])
    );
    assert_eq!(
        ai_order(&ids(&["a", "b"]), &[9.0, 1.0], &[0, 1]).unwrap(),
        ids(&["a", "b"])
    );

    let later = draft_of(&["w", "x", "y", "z"], &[1.0, 1.0, 1.0, 1.0], &[], 2, 1, false);
    assert_eq!(later.side_start, 1);
    assert_eq!(later.friend, ids(&["w", "z"]));
    assert_eq!(later.enemy, ids(&["x", "y"]));
    let earlier = draft_of(&["w", "x", "y", "z"], &[1.0, 1.0, 1.0, 1.0], &[], 2, 0, false);
    assert_eq!(earlier.side_start, 3);
    assert_eq!(earlier.friend, ids(&["x", "y"]));
    assert_eq!(earlier.enemy, ids(&["w", "z"]));
    let skipped = draft_of(
        &["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"],
        &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        &[0, 1, 2, 3],
        5,
        0,
        true,
    );
    assert_eq!(skipped.enemy, ids(&["j", "i", "h", "a", "b"]));
    assert_eq!(skipped.friend, ids(&["c", "d", "g", "f", "e"]));
    assert_eq!(skipped.hidden_ids, ids(&["a", "b", "c", "d"]));

    let lineup = enemy_battle(&ids(&["p", "q", "r"]), 3, 2, &[5, 3], &[1, 0]).unwrap();
    assert_eq!(lineup, ids(&["q", "r"]));
    assert!(matches!(
        enemy_battle(&ids(&["p"]), 3, 2, &[], &[]),
        Err(MartialError::BadConfig(_))
    ));

    assert_eq!(opening_draw("fan"), 5);
    assert_eq!(opening_draw("zhu"), 4);
    assert_eq!(attitude(true), 10);
    assert_eq!(attitude(false), -10);
    assert!(!has_zhu_skill());
    assert_eq!(
        video_name("刘备", Some("关羽"), "normal", "zhu"),
        ["刘备/关羽".to_string(), "新1v1 - 先手".to_string()]
    );

    let mut book = RecordBook::new();
    book.add("normal", "fan", Some(true));
    book.add("normal", "zhu", Some(true));
    book.add("normal", "zhu", Some(false));
    book.add("normal", "zhu", None);
    assert_eq!(
        format_record(&book),
        "新1v1：<br>先手：1胜 1负<br>后手：1胜 0负<br>"
    );

    let qiangdi_alive = check_result(true, true, true);
    assert!(qiangdi_alive.win && qiangdi_alive.coin == 150);
    let qiangdi_dead = check_result(false, true, true);
    assert!(qiangdi_dead.win && qiangdi_dead.coin == 0);
    let plain_loss = check_result(false, false, true);
    assert!(!plain_loss.win && plain_loss.coin == 0);
    let no_coin = check_result(true, true, false);
    assert!(no_coin.win && no_coin.coin == 0);

    let me_dead = on_general_dead(&DeathQuery {
        backups_left: 0,
        dead_is_me: true,
        me_dying: false,
        me_has_name2: true,
        dead_has_qiangdi: false,
    });
    assert!(me_dead.check_result && me_dead.try_mighty && !me_dead.replace);
    let enemy_double = on_general_dead(&DeathQuery {
        backups_left: 0,
        dead_is_me: false,
        me_dying: false,
        me_has_name2: true,
        dead_has_qiangdi: false,
    });
    assert!(!enemy_double.check_result && enemy_double.try_mighty);
    let no_second = on_general_dead(&DeathQuery {
        backups_left: 0,
        dead_is_me: false,
        me_dying: false,
        me_has_name2: false,
        dead_has_qiangdi: false,
    });
    assert!(no_second.check_result);
    let still_queued = on_general_dead(&DeathQuery {
        backups_left: 2,
        dead_is_me: true,
        me_dying: false,
        me_has_name2: false,
        dead_has_qiangdi: false,
    });
    assert!(still_queued.replace && !still_queued.check_result && !still_queued.try_mighty);

    let replaced = replacement_draw(&ids(&["a", "b", "c"]), true).unwrap();
    assert_eq!(replaced.entrant, "a");
    assert_eq!(replaced.backups, ids(&["b", "c"]));
    assert_eq!(replaced.draw, 4);
    assert_eq!(replacement_draw(&ids(&["a"]), true).unwrap().draw, 2);
    assert_eq!(replacement_draw(&ids(&["a"]), false).unwrap().draw, 4);

    let joined = ask_join(true, "j", None, &ids(&["i", "h"]), true);
    assert!(joined.offered && joined.accepted && !joined.hp_changed);
    assert_eq!(joined.name2.as_deref(), Some("i"));
    assert_eq!(joined.backups, ids(&["h"]));
    let declined = ask_join(true, "j", None, &ids(&["i"]), false);
    assert!(declined.offered && !declined.accepted && declined.name2.is_none());
    assert!(!ask_join(false, "j", None, &ids(&["i"]), true).offered);
    assert!(!ask_join(true, "j", Some("i"), &ids(&["h"]), true).offered);
    assert!(!double_hp_visible(false));
    assert!(double_hp_visible(true));

    let mut known = BTreeSet::new();
    for name in [
        "jin_simashi",
        "re_huanggai",
        "jlsgsr_huangyueying",
        "re_zhangjiao",
        "jlsgsoul_simayi",
        "maliang",
        "jlsgsoul_zhangjiao",
        "caorui",
        "wangyuanji",
        "ol_dongzhuo",
        "jlsgsoul_sp_zhugeliang",
        "re_sunquan",
        "hs_malfurion",
        "hs_xuanzhuanjijia",
        "mtg_jiding",
        "guanyinping",
        "jlsgsoul_guanyu",
        "hs_mojinbaozi",
    ] {
        known.insert(name.to_string());
    }
    let pairs = vec![
        MightyPair { name1: "jin_simashi".into(), name2: Some("re_huanggai".into()) },
        MightyPair { name1: "jlsgsr_huangyueying".into(), name2: Some("re_zhangjiao".into()) },
        MightyPair { name1: "jlsgsoul_simayi".into(), name2: Some("maliang".into()) },
        MightyPair { name1: "jlsgsoul_zhangjiao".into(), name2: Some("caorui".into()) },
        MightyPair { name1: "wangyuanji".into(), name2: Some("ol_dongzhuo".into()) },
        MightyPair { name1: "jlsgsoul_sp_zhugeliang".into(), name2: None },
        MightyPair { name1: "re_sunquan".into(), name2: None },
        MightyPair { name1: "hs_malfurion".into(), name2: Some("hs_xuanzhuanjijia".into()) },
        MightyPair { name1: "mtg_jiding".into(), name2: Some("guanyinping".into()) },
        MightyPair { name1: "jlsgsoul_guanyu".into(), name2: Some("hs_mojinbaozi".into()) },
    ];
    let accepted = consider_mighty(&pairs, &known, 0, true, true).unwrap();
    assert!(accepted.accepted && accepted.skill && accepted.phase_skipped);
    assert_eq!(accepted.draw, 4);
    assert!(!accepted.check_result);
    assert_eq!(accepted.name1.as_deref(), Some("jin_simashi"));
    assert_eq!(accepted.name2.as_deref(), Some("re_huanggai"));
    let declined_foe = consider_mighty(&pairs, &known, 7, false, false).unwrap();
    assert!(declined_foe.offered && declined_foe.check_result && !declined_foe.skill);
    assert_eq!(declined_foe.name1.as_deref(), Some("jlsgsoul_guanyu"));
    let empty = consider_mighty(&pairs, &BTreeSet::new(), 0, true, false).unwrap();
    assert!(!empty.offered && empty.check_result && !empty.skill);

    assert!(matches!(
        reselect_pool(false, &ids(&["a"]), &ids(&["b"]), &ids(&["c"])),
        Err(MartialError::Unavailable)
    ));
    assert_eq!(
        reselect_pool(true, &ids(&["a"]), &ids(&["b"]), &ids(&["c"])).unwrap(),
        ids(&["a", "b", "c"])
    );
    assert!(matches!(redraw_pool(false), Err(MartialError::Unavailable)));
    let cleared = redraw_pool(true).unwrap();
    assert!(cleared.list_dropped && cleared.friend.is_empty() && cleared.enemy.is_empty());
    assert!(matches!(
        free_replace(false, true, "liubei"),
        Err(MartialError::Unavailable)
    ));
    let waiting = free_replace(true, false, "liubei").unwrap();
    assert!(!waiting.replaced && waiting.name.is_none());
    assert_eq!(free_replace(true, true, "liubei").unwrap().name.as_deref(), Some("liubei"));

    let source_root = repo_extensions_root().join("martial-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text = std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");
    let npack_text = String::from_utf8(npack_bytes.clone()).expect("npack utf-8");
    assert!(npack_text.contains("武皇争霸"));
    assert!(
        npack_text.contains("\"package_id\":\"martial\"")
            || npack_text.contains("\"package_id\": \"martial\"")
    );
    let config = parse_martial_config(&config_text).expect("配置");
    assert_eq!(config.source, "noname-liuli");
    assert_eq!(config.upstream_commit, "8485dbb29ff16f9b258a766b1a17ef75f883aade");
    assert_eq!(config.upstream_version, "1.3");
    assert!(config.enable);
    assert_eq!((config.show, config.battle, config.hide), (5, 5, 2));
    assert!(!config.double_character && !config.free_choose && !config.change_choice);
    assert_eq!(config.double_hp, "hejiansan");
    let show_zero = config_text.replace("\"show_number\": \"5\"", "\"show_number\": 0");
    assert_eq!(parse_martial_config(&show_zero).unwrap().show, 5);
    let show_text_zero = config_text.replace("\"show_number\": \"5\"", "\"show_number\": \"0\"");
    assert_eq!(parse_martial_config(&show_text_zero).unwrap().show, 0);
    let battle_zero = config_text.replace("\"battle_number\": \"5\"", "\"battle_number\": 0");
    assert_eq!(parse_martial_config(&battle_zero).unwrap().battle, 0);
    assert_eq!(apply_martial_line(&config_text, "{}").unwrap(), DEFAULT_LINE);
    let quiet = apply_martial_line(&config_text, r#"{"join":true}"#).unwrap();
    assert!(quiet.contains("join:-"));
    assert!(matches!(
        apply_martial_line(&config_text.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""), "{}"),
        Err(MartialError::BadConfig(_))
    ));

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "martial" && item.namespace == "upstream-liuli")
        .expect("武皇争霸已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(meta.content_hash, content_runtime::content_hash_hex(&npack_bytes));
    for (package_id, namespace, folder_name) in [
        ("coin", "upstream-liuli", "coin-upstream-liuli"),
        ("coin", "upstream-noname", "coin-upstream-noname"),
        ("elo-sanction", "upstream-liuli", "elo-sanction-upstream-liuli"),
        ("sj-settings", "upstream-liuli", "sj-settings-upstream-liuli"),
        ("shousha-full", "upstream-liuli", "shousha-full-upstream-liuli"),
        ("douzhuan-xingyi", "upstream-liuli", "douzhuan-xingyi-upstream-liuli"),
        ("update-guard", "upstream-liuli", "update-guard-upstream-liuli"),
        ("fool-plugin", "upstream-liuli", "fool-plugin-upstream-liuli"),
    ] {
        let kept = report
            .installed
            .iter()
            .find(|item| item.package_id == package_id && item.namespace == namespace)
            .unwrap_or_else(|| panic!("组合：{namespace}/{package_id} 仍在"));
        assert_eq!(kept.version, "1.0.0");
        let bytes = std::fs::read(repo_extensions_root().join(folder_name).join("package.npack.json")).unwrap();
        assert_eq!(kept.content_hash, content_runtime::content_hash_hex(&bytes));
    }
    assert!(!report.installed.iter().any(|item| item.package_id == "如真似幻"));

    let (installed, installed_dir) = read_installed(&store, "upstream-liuli", "martial", "1.0.0");
    assert_eq!(installed.manifest.namespace, "upstream-liuli");
    assert_eq!(installed.manifest.package_id, "martial");
    assert_eq!(installed.manifest.version, "1.0.0");
    assert!(installed.manifest.dependencies.is_empty());
    assert!(installed.character_index.is_empty());
    assert!(installed.card_index.is_empty());
    assert!(installed.skill_index.is_empty());
    let lock_before = store.current_lock().expect("lock").lock_hash;
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());
    assert!(stored.contains("noname/extension/武皇争霸/extension.js"));
    assert!(!stored.contains("eval("));
    assert!(!stored.contains("game.reload"));
    assert!(!stored.contains("fetch("));
    assert_eq!(apply_martial_line(&stored, "{}").unwrap(), DEFAULT_LINE);
    let asset_names: Vec<_> = std::fs::read_dir(installed_dir.join("assets"))
        .unwrap()
        .map(|item| item.unwrap().file_name())
        .collect();
    assert_eq!(asset_names, vec![std::ffi::OsString::from("config.json")]);
    assert_eq!(store.current_lock().expect("lock").lock_hash, lock_before);

    use content_runtime::RuntimeError;
    match store.activate_version("upstream-liuli", "martial", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../martial".into();
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
    assert_eq!(active_version(&store, "upstream-liuli", "martial"), "1.0.0");
    assert_eq!(active_version(&store, "upstream-liuli", "fool-plugin"), "1.0.0");

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("martial-disabled");
    std::fs::create_dir_all(disabled_root.join("assets")).unwrap();
    for name in ["manifest.json", "package.npack.json"] {
        std::fs::copy(source_root.join(name), disabled_root.join(name)).unwrap();
    }
    std::fs::copy(source_root.join("assets/config.json"), disabled_root.join("assets/config.json")).unwrap();
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(disabled_root.join("manifest.json")).unwrap()).unwrap();
    manifest["enabled"] = serde_json::Value::Bool(false);
    std::fs::write(disabled_root.join("manifest.json"), serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    {
        let disabled_store = PackageStore::open(disabled_dir.path().join("store")).unwrap();
        let disabled = import_extensions(&disabled_store, disabled_dir.path(), &options()).expect("disabled import");
        assert!(!disabled.installed.iter().any(|item| item.package_id == "martial"));
        assert!(disabled.skipped.iter().any(|name| name == "martial-disabled"));
    }

    let mut upgraded = installed.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    store
        .install_with_assets(&upgraded.to_json_bytes().unwrap(), None, &options(), &source_root)
        .expect("升级");
    assert_eq!(active_version(&store, "upstream-liuli", "martial"), "2.0.0");
    assert_eq!(active_version(&store, "upstream-liuli", "fool-plugin"), "1.0.0");
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0");
    let rolled = store.rollback().expect("回滚");
    assert!(rolled.packages.iter().any(|(slot, version)| slot.ends_with("upstream-liuli/martial") && version == "1.0.0"));
    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(active_version(&reopened, "upstream-liuli", "martial"), "1.0.0");
    let (_, kept) = read_installed(&reopened, "upstream-liuli", "martial", "1.0.0");
    let kept_text = std::fs::read_to_string(kept.join("assets/config.json")).unwrap();
    assert_eq!(parse_martial_config(&kept_text).unwrap().source, "noname-liuli");
    assert_eq!(apply_martial_line(&kept_text, "{}").unwrap(), DEFAULT_LINE);
    assert_eq!(reopened.current_lock().expect("lock").lock_hash, lock_before);
}
