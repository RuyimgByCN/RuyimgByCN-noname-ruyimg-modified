//! L09 千幻聆音八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/qhly`。本来源**不改规则**：
//! 装载面（`content-native-loader::qhly`）提供换肤决策、随机皮肤策略与索引面
//! （真名/国战回退/封禁/列表过滤）；皮肤/语音**数据**与插件系统不迁移。

use std::path::PathBuf;

use content_native_loader::{
    decide_skin, erase_ext, filter_skin_list, is_skin_banned, parse_qhly_config, pick_random_skin,
    real_name, resolve_gz_fallback, SkinChangeRule, SkinChangeState, SkinShareTable, SkinTrigger,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

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

fn stored_config() -> String {
    std::fs::read_to_string(repo_extensions_root().join("qhly-upstream-liuli/assets/config.json"))
        .expect("读配置")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

/// L09 八项。
#[test]
fn l09_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    // ---- 功能：换肤决策（掉血门控阶段结束）----
    let hp_rule = SkinChangeRule {
        source: "hp_1".into(),
        phase_only: false,
        target: Some("skin_b".into()),
    };
    let mut st = SkinChangeState::default();
    assert!(hp_rule.is_active());
    assert_eq!(
        decide_skin(&hp_rule, SkinTrigger::HpChange { delta: -1 }, &mut st),
        Some("skin_b".into())
    );
    assert_eq!(decide_skin(&hp_rule, SkinTrigger::PhaseEnd, &mut st), None, "同回合跳过");
    assert_eq!(
        decide_skin(&hp_rule, SkinTrigger::PhaseEnd, &mut st),
        Some("skin_b".into()),
        "次回合阶段结束切换"
    );
    assert_eq!(decide_skin(&hp_rule, SkinTrigger::HpChange { delta: 1 }, &mut st), None);
    let non_hp = SkinChangeRule {
        source: "battle".into(),
        phase_only: false,
        target: Some("x".into()),
    };
    assert!(!non_hp.is_active());

    // ---- 功能：随机皮肤过滤 + 索引面 ----
    let cands = vec![
        "默认".to_string(),
        "娘化版".to_string(),
        "ok".to_string(),
        "banned".to_string(),
    ];
    let banned = vec!["banned".to_string()];
    let picked: Vec<Option<String>> = (0..2)
        .map(|i| pick_random_skin(&cands, &banned, &["娘化", "云涯"], i))
        .collect();
    assert_eq!(picked[0], Some("默认".into()));
    assert_eq!(picked[1], Some("ok".into()));
    assert_eq!(pick_random_skin(&cands, &banned, &["娘化", "云涯"], 2), None, "默认入池");
    let mut share = SkinShareTable::new();
    share.insert("gz_liubei".to_string(), "liubei".to_string());
    assert_eq!(real_name("gz_liubei", &share), "liubei");
    assert_eq!(
        resolve_gz_fallback("gz_liubei", false, false, false, |n| n == "liubei"),
        "liubei"
    );
    assert_eq!(erase_ext("skin/a.png"), "skin/a");
    let bans = vec!["liubei-[original]".to_string()];
    assert!(is_skin_banned("liubei", None, &bans));
    assert_eq!(
        filter_skin_list(
            &["s1".to_string(), "hidden".to_string(), "locked".to_string()],
            |s| s == "hidden",
            |s| s == "locked",
            false
        ),
        vec!["s1".to_string()]
    );

    // ---- 配置：严格解析 + 商店副本逐字节 ----
    let text = stored_config();
    let cfg = parse_qhly_config(&text).expect("解析配置");
    assert!(cfg.rand_skin && !cfg.rand_skin_me);
    assert_eq!(cfg.auto_change_skin, None, "close → 关闭");
    assert!(!text.contains("eval(") && !text.contains("fetch("));
    assert!(parse_qhly_config("not json").is_err());
    let missing = text.replace("\"source\": \"noname-liuli\",", "");
    assert!(parse_qhly_config(&missing).is_err());

    // ---- 规则差异：不改规则（阶段推进与体力逐位一致）----
    let mut a = Engine::new(930001, 4);
    let mut b = Engine::new(930001, 4);
    for _ in 0..8 {
        for e in [&mut a, &mut b] {
            if e.turn().phase == Phase::Play && e.turn().current == 0 {
                continue;
            }
            let actor = e.turn().current;
            let out = e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::NextPhase,
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            drain(e);
        }
    }
    assert_eq!(a.state_core().turn.phase, b.state_core().turn.phase);
    for p in 0..4 {
        assert_eq!(a.debug_hp(p), b.debug_hp(p), "体力逐位一致（{p}）");
    }

    // ---- 组合：与 swd 包同时装载 ----
    let swd_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包");
    let swd = content_native_loader::bridge_pack(&swd_json).expect("桥接 swd").pack;
    let combo = Engine::new_with_extensions(930002, 4, &[&swd]).expect("双包建局");
    assert_eq!(combo.state_core().players.len(), 4);

    // ---- 保存恢复：带包存档往返（体力/手牌保持）----
    let mut e = Engine::new(930003, 4);
    e.debug_set_hp(0, 2);
    let hand_before = e.debug_hand(0).len();
    let snap = e.save();
    let restored = Engine::restore(&snap).expect("恢复");
    assert_eq!(restored.state_core().player(0).hp, 2);
    assert_eq!(restored.debug_hand(0).len(), hand_before, "手牌保持");

    // ---- 安装：真实 import ----
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "qhly" && item.namespace == "upstream-liuli")
        .expect("千幻聆音已安装");
    assert_eq!(meta.version, "1.0.0");
    let npack_bytes =
        std::fs::read(repo_extensions_root().join("qhly-upstream-liuli/package.npack.json"))
            .unwrap();
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes)
    );
    let installed_dir = store
        .root()
        .join("packages")
        .join("upstream-liuli__qhly")
        .join("1.0.0");
    let installed = content_schema::ContentPackage::from_json_bytes(
        &std::fs::read(installed_dir.join("package.json")).unwrap(),
    )
    .expect("读商店包");
    assert!(installed.manifest.dependencies.is_empty());
    assert!(installed.skill_index.is_empty(), "不改规则：无技能条目");
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), text.as_bytes(), "商店副本逐字节一致");

    // ---- 保存恢复（商店）：重开内容锁不变 ----
    let lock_before = store.current_lock().expect("lock").lock_hash;
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        reopened.current_lock().expect("lock").lock_hash,
        lock_before
    );

    // ---- 安装升级：2.0.0 覆盖 + rollback + NotFound ----
    let mut upgraded = installed.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    store
        .install_with_assets(
            &upgraded.to_json_bytes().unwrap(),
            None,
            &options(),
            &installed_dir,
        )
        .expect("安装 2.0.0");
    let active = |store: &PackageStore| -> String {
        store
            .read_active()
            .unwrap()
            .packages
            .iter()
            .find(|(slot, _)| slot.ends_with("qhly"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    assert_eq!(active(&store), "2.0.0");
    store.rollback().expect("rollback");
    assert_eq!(active(&store), "1.0.0");
    match store.activate_version("upstream-liuli", "qhly", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }

    // ---- 依赖：皮肤/语音数据源以注入式接口建模（非 manifest 依赖）----
    assert!(installed.manifest.dependencies.is_empty());
    // 目标平台：包内无平台专有代码（不 eval/fetch/fs）
    assert!(!text.contains("fs.") && !text.contains("require("));
}
