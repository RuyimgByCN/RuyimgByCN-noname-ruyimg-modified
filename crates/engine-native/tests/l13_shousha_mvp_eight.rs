//! L13 手杀MVP 八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/shousha-mvp`。MVP 计分、
//! 战绩统计与结算分组走本包配置与调用方给出的权威事实/身份关系。不替换
//! 内核终局，不画界面，不播音，不检测作弊输入，不生成随机名单。事件按
//! `seq` 去重：录像 / 恢复 / 重放重复投递只入账一次。

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use content_native_loader::{
    apply_mvp_line, parse_mvp_config, parse_mvp_request, MvpError, MvpFact, MvpPlayer, MvpSession,
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

fn player(id: u32, identity: &str, alive: bool, friends: &[u32], enemies: &[u32]) -> MvpPlayer {
    MvpPlayer {
        id,
        name: format!("p{id}"),
        identity: identity.to_string(),
        alive,
        friends: friends.iter().copied().collect::<BTreeSet<_>>(),
        enemies: enemies.iter().copied().collect::<BTreeSet<_>>(),
    }
}

/// 四人身份局：0 主公（阵亡）、1 忠臣、2 反贼（阵亡）、3 内奸。
fn identity_players() -> Vec<MvpPlayer> {
    vec![
        player(0, "zhu", false, &[1], &[2, 3]),
        player(1, "zhong", true, &[0], &[2, 3]),
        player(2, "fan", false, &[], &[0, 1]),
        player(3, "nei", true, &[], &[2]),
    ]
}

/// 事实序列（功能面正例；seq 1..=14）。
fn sample_facts() -> Vec<MvpFact> {
    vec![
        MvpFact::UseCard {
            seq: 1,
            player: 0,
            damage_tag: true,
            wuxie: false,
            trick: false,
            to_self: false,
            select_target: 1,
        },
        MvpFact::UseCard {
            seq: 2,
            player: 0,
            damage_tag: false,
            wuxie: true,
            trick: true,
            to_self: false,
            select_target: 1,
        },
        MvpFact::UseCard {
            seq: 3,
            player: 2,
            damage_tag: false,
            wuxie: false,
            trick: true,
            to_self: false,
            select_target: -1,
        },
        MvpFact::UseCard {
            seq: 4,
            player: 1,
            damage_tag: false,
            wuxie: false,
            trick: true,
            to_self: true,
            select_target: -1,
        },
        MvpFact::Damage {
            seq: 5,
            source: 0,
            target: 2,
            num: 3,
            attitude_negative: true,
            source_is_nei: false,
        },
        MvpFact::Damage {
            seq: 6,
            source: 3,
            target: 1,
            num: 6,
            attitude_negative: false,
            source_is_nei: true,
        },
        MvpFact::Gain {
            seq: 7,
            player: 0,
            source: 2,
            cards: 2,
            source_alive: true,
            source_is_nei: false,
            player_enemy: true,
        },
        MvpFact::Gain {
            seq: 8,
            player: 1,
            source: 0,
            cards: 1,
            source_alive: true,
            source_is_nei: false,
            player_enemy: false,
        },
        MvpFact::Discard {
            seq: 9,
            player: 2,
            source: 0,
            cards: 3,
            source_alive: true,
            source_is_nei: false,
            player_enemy: true,
        },
        MvpFact::Recover {
            seq: 10,
            player: 0,
            source: 1,
            num: 2,
            source_alive: true,
            source_is_nei: false,
            friend_or_self: true,
        },
        MvpFact::Recover {
            seq: 11,
            player: 2,
            source: 3,
            num: 7,
            source_alive: true,
            source_is_nei: true,
            friend_or_self: false,
        },
        MvpFact::Die {
            seq: 12,
            victim: 2,
            source: 0,
            source_alive: true,
            source_is_nei: false,
            victim_is_lord: false,
            victim_friend: false,
            victim_enemy: true,
        },
        MvpFact::Die {
            seq: 13,
            victim: 0,
            source: 3,
            source_alive: true,
            source_is_nei: true,
            victim_is_lord: true,
            victim_friend: true,
            victim_enemy: false,
        },
        MvpFact::Die {
            seq: 14,
            victim: 1,
            source: 3,
            source_alive: true,
            source_is_nei: true,
            victim_is_lord: false,
            victim_friend: false,
            victim_enemy: true,
        },
    ]
}

/// L13 八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l13_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let source_root = repo_extensions_root().join("shousha-mvp-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text =
        std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");
    let npack_text = String::from_utf8(npack_bytes.clone()).expect("npack utf-8");
    assert!(npack_text.contains("手杀MVP"));
    assert!(npack_text.contains("\"package_id\":\"shousha-mvp\""));

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "shousha-mvp" && item.namespace == "upstream-liuli")
        .expect("手杀MVP 已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes),
        "商店 content_hash 来自本包 npack"
    );

    // ---- 组合：同一次导入仍安装其他包，且不改写它们的 npack ----
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
    let elo = report
        .installed
        .iter()
        .find(|item| item.package_id == "elo-sanction" && item.namespace == "upstream-liuli")
        .expect("组合：段位制裁仍在");
    assert_eq!(elo.version, "1.0.0");
    assert!(
        !report.installed.iter().any(|item| item.package_id == "如真似幻"),
        "不把如真似幻计入"
    );

    let (installed, installed_dir) =
        read_installed(&store, "upstream-liuli", "shousha-mvp", "1.0.0");
    assert!(installed.manifest.dependencies.is_empty(), "依赖：不声明外部扩展");
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());

    // ---- 配置：真实资产默认值 + 严格解析反例 ----
    assert!(stored.contains("\"source\": \"noname-liuli\""));
    assert!(stored.contains("noname/extension/手杀MVP/extension.js"));
    assert!(stored.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
    assert!(stored.contains("\"package_version\": \"1.1.7\""));
    let config = parse_mvp_config(&stored).expect("配置");
    assert_eq!(config.source, "noname-liuli");
    assert!(config.enabled, "shoushajiesuan 默认开");
    assert!(config.jiesuan_fix, "jiesuanfix 默认开（新版结算）");
    assert_eq!(config.yangshi, "on");
    assert!(config.one_more);
    assert!(config.style_audio);
    assert_eq!(config.package_name, "手杀MVP");
    assert_eq!(config.package_version, "1.1.7");
    assert_eq!(config.author, "清瑶的“徒弟”、神秘喵");
    assert_eq!(config.labels.get("attack").map(String::as_str), Some("攻击"));
    assert_eq!(config.labels.get("kill").map(String::as_str), Some("杀敌"));
    assert!(matches!(
        parse_mvp_config(&stored.replace("\"noname-liuli\"", "\"\"")),
        Err(MvpError::BadField("source"))
    ));
    assert!(matches!(
        parse_mvp_config(&stored.replace("\"package_version\": \"1.1.7\",", "")),
        Err(MvpError::Missing("package_version"))
    ));
    assert!(matches!(
        parse_mvp_config(&"{}}"),
        Err(MvpError::Json(_))
    ));
    assert!(matches!(
        parse_mvp_config(&stored.replace("\"settings\"", "\"settingsX\"")),
        Err(MvpError::Missing("settings"))
    ));
    assert!(matches!(
        parse_mvp_config(&stored.replace("\"shoushajiesuan\": true", "\"shoushajiesuan\": \"yes\"")),
        Err(MvpError::BadField("shoushajiesuan"))
    ));
    assert!(matches!(
        parse_mvp_config(&stored.replace("\"yangshi\": \"on\"", "\"yangshi\": 1")),
        Err(MvpError::BadField("yangshi"))
    ));
    assert!(matches!(
        parse_mvp_config(&stored.replace("\"attack\": \"攻击\"", "\"attack\": 1")),
        Err(MvpError::BadField("labels"))
    ));
    let lock_before = store.current_lock().expect("lock").lock_hash;

    // ---- 功能：MVP 计分与战绩（正例） ----
    let mut session = MvpSession::new(config.clone(), identity_players());
    for fact in sample_facts() {
        assert!(session.apply(fact).expect("入账"), "事实应入账");
    }
    let scores = session.scores();
    let s0 = scores.get(&0).unwrap();
    assert_eq!((s0.attack, s0.heal, s0.support, s0.penalty), (14, 0, 7, 0), "0 号：攻击 14 / 辅助 7");
    let s1 = scores.get(&1).unwrap();
    assert_eq!((s1.attack, s1.heal, s1.support, s1.penalty), (0, 4, 0, 0), "1 号：治疗 4");
    let s2 = scores.get(&2).unwrap();
    assert_eq!((s2.attack, s2.heal, s2.support, s2.penalty), (0, 0, 1, 0), "2 号：多目标锦囊辅助 1");
    let s3 = scores.get(&3).unwrap();
    assert_eq!(
        (s3.attack, s3.heal, s3.support, s3.penalty),
        (18, 10, 0, 5),
        "3 号（内奸）：攻击 18 / 治疗 10 / 惩罚 5"
    );
    let stats = session.stats();
    assert_eq!(
        (
            stats.get(&0).unwrap().damage,
            stats.get(&0).unwrap().gain,
            stats.get(&0).unwrap().play,
            stats.get(&0).unwrap().kill
        ),
        (3, 2, 2, 1),
        "0 号战绩"
    );
    assert_eq!(
        (
            stats.get(&1).unwrap().damaged,
            stats.get(&1).unwrap().gain,
            stats.get(&1).unwrap().play
        ),
        (6, 1, 1),
        "1 号战绩"
    );
    assert_eq!(stats.get(&2).unwrap().damaged, 3);
    assert_eq!((stats.get(&3).unwrap().damage, stats.get(&3).unwrap().kill), (6, 2), "3 号两次击杀");

    // 反例：重复 seq 不入账；未知玩家报错。
    assert!(!session.apply(sample_facts()[0].clone()).expect("重复"), "重复事实不再入账");
    assert_eq!(session.scores().get(&0).unwrap().attack, 14, "重复不改变分数");
    assert!(matches!(
        session.apply(MvpFact::Damage {
            seq: 99,
            source: 9,
            target: 0,
            num: 1,
            attitude_negative: true,
            source_is_nei: false,
        }),
        Err(MvpError::UnknownPlayer(9))
    ));

    // ---- 规则差异：主开关关闭 → 整块不登记 ----
    let mut gated_config = config.clone();
    gated_config.enabled = false;
    let mut gated = MvpSession::new(gated_config, identity_players());
    for fact in sample_facts() {
        assert!(!gated.apply(fact).expect("门控"), "主开关关闭不入账");
    }
    assert_eq!(gated.scores().get(&0).unwrap().attack, 0);
    let gated_settlement = gated.settle("战斗胜利", 3, "identity").unwrap();
    assert!(gated_settlement.victory.is_empty() && gated_settlement.mvp.is_none());

    // ---- 功能/配置：结算基础分、排序与 MVP ----
    let settlement = session.settle("战斗胜利", 3, "identity").expect("结算");
    assert_eq!(settlement.victory, vec![3], "内奸视角胜方只有自己");
    assert_eq!(settlement.fail, vec![1, 0, 2], "其余为负方（存活者在前）");
    assert!(settlement.tie.is_empty());
    assert_eq!(settlement.ranking, vec![3, 1, 0, 2], "总分降序");
    assert_eq!(settlement.mvp, Some(3));
    assert_eq!(settlement.scores.get(&0).unwrap().base, 82, "0 号阵亡 -20、敌阵亡 +2");
    assert_eq!(settlement.scores.get(&1).unwrap().base, 100, "1 号友阵亡 -2");
    assert_eq!(settlement.scores.get(&2).unwrap().base, 82);
    assert_eq!(settlement.scores.get(&3).unwrap().base, 102);
    assert_eq!(settlement.scores.get(&0).unwrap().total(), 103);
    assert_eq!(settlement.scores.get(&1).unwrap().total(), 104);
    assert_eq!(settlement.scores.get(&2).unwrap().total(), 83);
    assert_eq!(settlement.scores.get(&3).unwrap().total(), 125);
    // 重复结算返回首次结果（胜负一次）。
    let again = session.settle("战斗失败", 0, "identity").expect("重复结算");
    assert_eq!(again.victory, settlement.victory, "重复结算不重算");

    // ---- 配置：jiesuanfix 两条口径（失败分支） ----
    let mut fix_on = MvpSession::new(config.clone(), identity_players());
    for fact in sample_facts() {
        fix_on.apply(fact).unwrap();
    }
    let on = fix_on.settle("战斗失败", 0, "identity").unwrap();
    assert_eq!(on.victory, vec![2], "新版：存活反贼为胜方");
    assert_eq!(on.fail, vec![1, 0], "新版：同阵营为负方；内奸不入任何组");
    assert!(on.tie.is_empty());
    let mut legacy_config = config.clone();
    legacy_config.jiesuan_fix = false;
    let mut fix_off = MvpSession::new(legacy_config, identity_players());
    for fact in sample_facts() {
        fix_off.apply(fact).unwrap();
    }
    let off = fix_off.settle("战斗失败", 0, "identity").unwrap();
    assert_eq!(off.victory, vec![3], "旧版：存活内奸为胜方");
    assert_eq!(off.fail, vec![1, 0, 2], "旧版：其余为负方");

    // 国战分支与平局分支。
    let mut guozhan = MvpSession::new(config.clone(), identity_players());
    let gz = guozhan.settle("战斗结束", 0, "guozhan").unwrap();
    assert_eq!(gz.tie, vec![1, 3], "国战非战斗结果：存活者平局");
    assert_eq!(gz.fail, vec![0, 2], "国战：阵亡者负");
    let mut draw = MvpSession::new(config.clone(), identity_players());
    let dr = draw.settle("战斗结束", 0, "identity").unwrap();
    assert_eq!(dr.tie, vec![1, 3], "平局：存活者平局");
    assert_eq!(dr.fail, vec![0, 2]);

    // ---- 规则差异：不建态度模型、不读界面、不检测作弊 ----
    let sample_with_nei_relation = config.clone();
    let mut relation = MvpSession::new(sample_with_nei_relation, identity_players());
    // 来源为内奸但关系非敌：仍按上游「内奸无条件」入账。
    relation
        .apply(MvpFact::Damage {
            seq: 1,
            source: 3,
            target: 1,
            num: 2,
            attitude_negative: false,
            source_is_nei: true,
        })
        .unwrap();
    assert_eq!(relation.scores().get(&3).unwrap().attack, 6, "内奸伤害照记");
    // 来源已阵亡：effect2/3/4 的 isIn() 门拒绝。
    let mut dead_source = MvpSession::new(config.clone(), identity_players());
    dead_source
        .apply(MvpFact::Recover {
            seq: 1,
            player: 0,
            source: 3,
            num: 3,
            source_alive: false,
            source_is_nei: true,
            friend_or_self: false,
        })
        .unwrap();
    dead_source
        .apply(MvpFact::Die {
            seq: 2,
            victim: 1,
            source: 3,
            source_alive: false,
            source_is_nei: true,
            victim_is_lord: false,
            victim_friend: false,
            victim_enemy: true,
        })
        .unwrap();
    assert_eq!(dead_source.scores().get(&3).unwrap().heal, 0, "阵亡来源不记治疗");
    assert_eq!(dead_source.scores().get(&3).unwrap().attack, 0, "阵亡来源不记击杀");

    // ---- 保存恢复：状态导出/恢复 + 重复投递不重复入账 ----
    let state = session.export_state();
    let restored = MvpSession::restore_state(config.clone(), identity_players(), &state).unwrap();
    assert_eq!(restored.scores(), session.scores(), "状态往返保持分数");
    assert_eq!(restored.stats(), session.stats(), "状态往返保持战绩");
    let mut resumed = MvpSession::restore_state(config.clone(), identity_players(), &state).unwrap();
    assert!(!resumed.apply(sample_facts()[0].clone()).unwrap(), "恢复后重复事实不入账");
    assert!(
        resumed
            .apply(MvpFact::Recover {
                seq: 15,
                player: 1,
                source: 0,
                num: 1,
                source_alive: true,
                source_is_nei: false,
                friend_or_self: true,
            })
            .unwrap(),
        "恢复后新事实照常入账"
    );
    assert!(matches!(
        MvpSession::restore_state(config.clone(), identity_players(), "{}"),
        Err(MvpError::BadState(_))
    ));
    assert!(matches!(
        MvpSession::restore_state(
            config.clone(),
            identity_players(),
            &state.replace("\"lastSeq\":", "\"lastSeqX\":")
        ),
        Err(MvpError::BadState(_))
    ));

    // ---- 目标平台：同一入口的紧凑报告（wasm 侧另有字符串辅助测试） ----
    let request = format!(
        "{{\"viewer\":3,\"result\":\"战斗胜利\",\"mode\":\"identity\",\"players\":[{}],\"facts\":[]}}",
        [
            "{\"id\":0,\"name\":\"p0\",\"identity\":\"zhu\",\"alive\":false,\"friends\":[1],\"enemies\":[2,3]}",
            "{\"id\":1,\"name\":\"p1\",\"identity\":\"zhong\",\"alive\":true,\"friends\":[0],\"enemies\":[2,3]}",
            "{\"id\":2,\"name\":\"p2\",\"identity\":\"fan\",\"alive\":false,\"friends\":[],\"enemies\":[0,1]}",
            "{\"id\":3,\"name\":\"p3\",\"identity\":\"nei\",\"alive\":true,\"friends\":[],\"enemies\":[2]}",
        ]
        .join(",")
    );
    let line = apply_mvp_line(&stored, &request).expect("单行入口");
    assert!(line.starts_with("enable:1 fix:1 | facts:0 dup:0 gated:0 | scores:"), "{line}");
    // 无事实时只有基础分：并列 82 分按上游「稳定升序后反转」得 2 在 0 前。
    assert!(line.contains("| rank:3,1,2,0 | mvp:3 | win:3 fail:1,0,2 tie:-"), "{line}");
    let restored_line = apply_mvp_line(&stored, &request.replace(
        "\"facts\":[]",
        &format!("\"facts\":[],\"state\":{}", serde_json::to_string(&state).unwrap()),
    ))
    .expect("带状态入口");
    assert!(restored_line.contains("scores:0:14/0/7/0/82/103"), "{restored_line}");
    assert!(restored_line.contains("dup:1"), "恢复后重复事实计入去重：{restored_line}");
    assert!(apply_mvp_line(&stored, "{}").is_err(), "缺字段拒绝");
    assert!(matches!(parse_mvp_request("{"), Err(MvpError::Json(_))));
    assert!(matches!(parse_mvp_request("[]"), Err(MvpError::Missing("viewer"))));
    // 观察者不在玩家表：结算报未知玩家。
    assert!(matches!(
        apply_mvp_line(&stored, &request.replace("\"viewer\":3", "\"viewer\":9")),
        Err(MvpError::UnknownPlayer(9))
    ));
    let bad_fact = request.replace(
        "\"facts\":[]",
        "\"facts\":[{\"seq\":1,\"kind\":\"wat\"}]",
    );
    assert!(matches!(apply_mvp_line(&stored, &bad_fact), Err(MvpError::UnknownFact(_))));
    let unknown_player_fact = request.replace(
        "\"facts\":[]",
        "\"facts\":[{\"seq\":1,\"kind\":\"gain\",\"player\":9,\"source\":0,\"cards\":1}]",
    );
    assert!(matches!(
        apply_mvp_line(&stored, &unknown_player_fact),
        Err(MvpError::UnknownPlayer(9))
    ), "未知玩家事实拒绝");
    // 事实序号只在完整入账后推进：报错事实不消费序号。
    let error_then_valid = request.replace(
        "\"facts\":[]",
        "\"facts\":[{\"seq\":5,\"kind\":\"gain\",\"player\":9,\"source\":0,\"cards\":1}]",
    );
    assert!(apply_mvp_line(&stored, &error_then_valid).is_err());
    assert_eq!(
        store.current_lock().expect("lock").lock_hash,
        lock_before,
        "结算公式不改内容锁"
    );

    // ---- 规则差异：不画界面 / 不播音 / 不建随机名单 / 不检测作弊 ----
    assert!(!stored.contains(".png") && !stored.contains(".jpg"));
    assert!(!stored.contains("audio") && !stored.contains("playAudio"));
    assert!(!stored.contains("innerHTML") && !stored.contains("commandnode"));
    assert!(!stored.contains("Math.random") && !stored.contains("randomName"));
    assert!(!stored.contains("eval") && !stored.contains("connect_nickname"));
    let asset_names: Vec<_> = std::fs::read_dir(installed_dir.join("assets"))
        .unwrap()
        .map(|item| item.unwrap().file_name())
        .collect();
    assert_eq!(asset_names, vec![std::ffi::OsString::from("config.json")]);

    // ---- 依赖：未安装版本 / 包 id 逃逸 / 缺依赖安装 ----
    match store.activate_version("upstream-liuli", "shousha-mvp", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../shousha-mvp".into();
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
    assert_eq!(active_version(&store, "upstream-liuli", "shousha-mvp"), "1.0.0");
    assert_eq!(
        active_version(&store, "upstream-liuli", "coin"),
        "1.0.0",
        "失败安装不撤掉 liuli 金币"
    );
    assert_eq!(active_version(&store, "upstream-liuli", "elo-sanction"), "1.0.0");

    // ---- 安装升级：禁用清单跳过、2.0.0 升级与回滚 ----
    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("shousha-mvp-disabled");
    std::fs::create_dir_all(disabled_root.join("assets")).unwrap();
    std::fs::copy(source_root.join("manifest.json"), disabled_root.join("manifest.json")).unwrap();
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
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(disabled_root.join("manifest.json")).unwrap())
            .unwrap();
    manifest["enabled"] = serde_json::Value::Bool(false);
    std::fs::write(
        disabled_root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    {
        let disabled_store = PackageStore::open(disabled_dir.path().join("store")).unwrap();
        let disabled =
            import_extensions(&disabled_store, disabled_dir.path(), &options()).expect("disabled import");
        assert!(!disabled
            .installed
            .iter()
            .any(|item| item.namespace == "upstream-liuli" && item.package_id == "shousha-mvp"));
        assert!(disabled
            .skipped
            .iter()
            .any(|name| name == "shousha-mvp-disabled"));
    }

    let mut upgraded = installed.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    store
        .install_with_assets(&upgraded.to_json_bytes().unwrap(), None, &options(), &source_root)
        .expect("升级");
    assert_eq!(active_version(&store, "upstream-liuli", "shousha-mvp"), "2.0.0");
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0");
    let rolled = store.rollback().expect("回滚");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, version)| slot.ends_with("upstream-liuli/shousha-mvp") && version == "1.0.0"));
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0");

    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        active_version(&reopened, "upstream-liuli", "shousha-mvp"),
        "1.0.0"
    );
    let (_, kept) = read_installed(&reopened, "upstream-liuli", "shousha-mvp", "1.0.0");
    let kept_text = std::fs::read_to_string(kept.join("assets/config.json")).unwrap();
    let kept_config = parse_mvp_config(&kept_text).unwrap();
    assert_eq!(kept_config.package_version, "1.1.7");
    assert!(kept_config.enabled);
    let kept_labels: BTreeMap<String, String> = kept_config.labels.clone();
    assert_eq!(kept_labels.get("support").map(String::as_str), Some("辅助"));
}
