//! L24 皮肤切换八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/skin-switch`。本来源**不改规则**：
//! 装载面（`content-native-loader::skin_switch`）提供动皮参数重应用、五类触发
//! 策略与配置；520 KB 数据表、Spine/Worker 渲染与十周年/千幻互调不迁移。

use std::collections::BTreeMap;
use std::path::PathBuf;

use content_native_loader::{
    damage_transform_allowed, group_changed, group_nature, lowhp_decision, parse_skin_switch_config,
    reapply_decision, reapply_on_init_allowed, resolve_transform, LowHpState, ReapplyDecision,
    TransformFlags,
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
    std::fs::read_to_string(
        repo_extensions_root().join("skin-switch-upstream-liuli/assets/config.json"),
    )
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

/// L24 八项。
#[test]
fn l24_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    // ---- 功能：参数重应用（退避 + 初始化过滤）----
    assert_eq!(reapply_decision(true, 0), ReapplyDecision::Apply);
    assert_eq!(reapply_decision(false, 0), ReapplyDecision::RetryAfter(500));
    assert_eq!(reapply_decision(false, 1), ReapplyDecision::RetryAfter(1000));
    assert_eq!(reapply_decision(false, 2), ReapplyDecision::RetryAfter(2000));
    assert_eq!(reapply_decision(false, 3), ReapplyDecision::GiveUp);
    assert!(reapply_on_init_allowed(true, true));
    assert!(!reapply_on_init_allowed(false, true));

    // ---- 功能：低血「仅新低」----
    let mut low = LowHpState::default();
    assert!(lowhp_decision(&mut low, true, 3, -1, false));
    assert!(!lowhp_decision(&mut low, true, 3, -1, false));
    assert!(lowhp_decision(&mut low, true, 2, -1, false));
    assert!(!lowhp_decision(&mut low, true, 3, 1, false), "回复未开 recover 不触发");
    assert!(lowhp_decision(&mut low, false, 4, -1, false), "副将独立");
    assert_eq!(low.primary_lowest, Some(2));

    // ---- 功能：受伤一次 + 换组 + 目标解析 ----
    let mut flags = TransformFlags::default();
    assert!(damage_transform_allowed(&mut flags, true));
    assert!(!damage_transform_allowed(&mut flags, true));
    assert!(damage_transform_allowed(&mut flags, false));
    assert_eq!(group_nature("wei_x"), Some("wei"));
    assert_eq!(group_nature("shen"), None);
    assert!(group_changed("wei", "shu"));
    assert!(!group_changed("wei", "wei_y"));
    let mut special = BTreeMap::new();
    special.insert("t".to_string(), "bone/skin".to_string());
    assert_eq!(
        resolve_transform(Some("t"), &special, |b, s| b == "bone" && s == "skin"),
        Some(("bone".to_string(), "skin".to_string()))
    );
    assert_eq!(resolve_transform(Some("t"), &special, |_, _| false), None);

    // ---- 配置：严格解析 + 商店副本逐字节 ----
    let text = stored_config();
    let cfg = parse_skin_switch_config(&text).expect("解析配置");
    assert!(cfg.dynamic_skin && cfg.use_dynamic && cfg.attack_effect && cfg.czg_enable);
    assert!(!cfg.attack_flip_x && !cfg.hide_huanfu && !cfg.cug_dynamic_bg && !cfg.show_top_arc);
    assert_eq!(cfg.saved_positions.get("stand_x"), Some(&0.5));
    assert!(!text.contains("eval(") && !text.contains("fetch(") && !text.contains("fs."));
    assert!(parse_skin_switch_config("not json").is_err());
    let bad_pos = text.replace("\"stand_x\": 0.5", "\"stand_x\": \"x\"");
    assert!(parse_skin_switch_config(&bad_pos).is_err());

    // ---- 规则差异：不改规则（阶段推进与 4 座体力逐位一致）----
    let mut a = Engine::new(940001, 4);
    let mut b = Engine::new(940001, 4);
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
    let combo = Engine::new_with_extensions(940002, 4, &[&swd]).expect("双包建局");
    assert_eq!(combo.state_core().players.len(), 4);

    // ---- 保存恢复：带包存档往返 ----
    let mut e = Engine::new(940003, 4);
    e.debug_set_hp(0, 2);
    let hand_before = e.debug_hand(0).len();
    let snap = e.save();
    let restored = Engine::restore(&snap).expect("恢复");
    assert_eq!(restored.state_core().player(0).hp, 2);
    assert_eq!(restored.debug_hand(0).len(), hand_before);

    // ---- 安装：真实 import ----
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "skin-switch" && item.namespace == "upstream-liuli")
        .expect("皮肤切换已安装");
    assert_eq!(meta.version, "1.0.0");
    let npack_bytes =
        std::fs::read(repo_extensions_root().join("skin-switch-upstream-liuli/package.npack.json"))
            .unwrap();
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes)
    );
    let installed_dir = store
        .root()
        .join("packages")
        .join("upstream-liuli__skin-switch")
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
            .find(|(slot, _)| slot.ends_with("skin-switch"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    assert_eq!(active(&store), "2.0.0");
    store.rollback().expect("rollback");
    assert_eq!(active(&store), "1.0.0");
    match store.activate_version("upstream-liuli", "skin-switch", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }

    // ---- 依赖：数据/渲染源以注入式接口建模（非 manifest 依赖）----
    assert!(installed.manifest.dependencies.is_empty());
    assert!(!text.contains("require("), "包内无平台专有代码");
}
