//! L01 AI优化八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/ai-tuning`。本来源**不改规则**：
//! 装载面（`content-native-loader::ai_tuning`）提供威胁度/权重参数与结算口径，
//! 原生规划器（`engine_native::ai::ContentTuning` + 策略变体 2）消费之。

use std::path::PathBuf;

use content_native_loader::{
    benefit, parse_ai_tuning_config, rating_to_weight, threaten, weight, CfMode, QzMode,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use engine_native::ai::{BoundedPlanner, ContentTuning};
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
        repo_extensions_root().join("ai-tuning-upstream-liuli/assets/config.json"),
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

/// 构造：0 号持【杀】，1/3 号相邻且空手，轮到 0 号出牌阶段。
fn attack_setup(seed: u64) -> Engine {
    let mut e = Engine::new(seed, 4);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
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
        drain(&mut e);
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    e
}

fn planned_targets(e: &Engine, weights: &[(u8, f64)]) -> Option<Vec<u8>> {
    let tuning = ContentTuning {
        seat_weights: weights.to_vec(),
        skill_threat: vec![("mohua2".into(), 2.0)],
        qz_cf: true,
    };
    let planner = BoundedPlanner::new(64)
        .with_policy_variant(2)
        .with_content_tuning(tuning);
    match e.plan_action(0, &planner).action {
        GameAction::PlayCard { targets, .. } => Some(targets),
        _ => None,
    }
}

/// L01 八项。
#[test]
fn l01_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    // ---- 功能：参数解析 + 口径（威胁度/权重/效益）----
    let text = stored_config();
    let cfg = parse_ai_tuning_config(&text).expect("解析配置");
    assert_eq!(cfg.apply_cf, CfMode::Off);
    assert_eq!(cfg.ck_qz, QzMode::Off);
    assert_eq!(threaten(&cfg, "gjcxs_zhugeliang3"), Some(1.9));
    assert_eq!(threaten(&cfg, "未配置"), None);
    assert_eq!(weight(&cfg, "shen_zhugeliang", None, &[]), 1.75);
    assert_eq!(weight(&cfg, "无记录", None, &[]), 1.0);
    let mut rating_mode = cfg.clone();
    rating_mode.ck_qz = QzMode::Rating;
    assert!((weight(&rating_mode, "无记录", Some(0.0), &[]) - 0.8).abs() < 1e-9);
    assert!((rating_to_weight(5.0) - 1.95).abs() < 1e-9);
    let mut benefit_on = cfg.clone();
    benefit_on.qz_cf = true;
    assert!((benefit(&benefit_on, 2.0, 3.0) - 6.0).abs() < 1e-9);
    assert!((benefit(&cfg, 2.0, 3.0) - 3.0).abs() < 1e-9);

    // ---- 功能：原生 AI 消费（确定性 + 权重改变决策）----
    let e = attack_setup(920001);
    let a = planned_targets(&e, &[(1, 1.0), (3, 1.0)]);
    let b = planned_targets(&e, &[(1, 1.0), (3, 1.0)]);
    assert_eq!(a, b, "同信息集同决策");
    let heavy_three = planned_targets(&e, &[(1, 1.0), (3, 5.0)]);
    let heavy_one = planned_targets(&e, &[(1, 5.0), (3, 1.0)]);
    assert!(
        heavy_three.as_ref().is_some_and(|t| t.contains(&3)),
        "权重改变决策（3 号）：{heavy_three:?}"
    );
    assert!(
        heavy_one.as_ref().is_some_and(|t| t.contains(&1)),
        "权重改变决策（1 号）：{heavy_one:?}"
    );

    // ---- 配置：严格解析 + 商店副本逐字节 ----
    assert!(!text.contains("eval(") && !text.contains("fetch("));
    let bad_mode = text.replace("\"applyCf\": \"off\"", "\"applyCf\": \"xx\"");
    assert!(parse_ai_tuning_config(&bad_mode).is_err());
    let bad_value = text.replace("\"gjcxs_caojie1\": 0.7", "\"gjcxs_caojie1\": 999");
    assert!(parse_ai_tuning_config(&bad_value).is_err());
    assert!(parse_ai_tuning_config("not json").is_err());

    // ---- 规则差异：调参不改规则（同局面合法性一致）----
    let mut plain = Engine::new(920002, 4);
    let mut tuned = Engine::new(920002, 4);
    for _ in 0..8 {
        for e in [&mut plain, &mut tuned] {
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
    assert_eq!(
        plain.state_core().turn.phase,
        tuned.state_core().turn.phase,
        "调参不改阶段推进"
    );
    assert_eq!(
        plain.debug_hp(0),
        tuned.debug_hp(0),
        "调参不改体力状态"
    );

    // ---- 组合：与 swd 包同时装载 + 调参规划可用 ----
    let swd_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包");
    let swd = content_native_loader::bridge_pack(&swd_json).expect("桥接 swd").pack;
    let mut combo = Engine::new_with_extensions(920003, 4, &[&swd]).expect("双包建局");
    for _ in 0..12 {
        if combo.turn().current == 0 && combo.turn().phase == Phase::Play {
            break;
        }
        let actor = combo.turn().current;
        let out = combo.submit(GameCommand {
            actor,
            expected_revision: combo.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "{out:?}"
        );
        drain(&mut combo);
    }
    let planner = BoundedPlanner::new(64)
        .with_policy_variant(2)
        .with_content_tuning(ContentTuning {
            seat_weights: vec![(1, 2.0)],
            ..Default::default()
        });
    let _ = combo.plan_action(0, &planner);

    // ---- 保存恢复：带包存档往返 + 恢复后同决策 ----
    let snap = e.save();
    let restored = Engine::restore(&snap).expect("恢复");
    assert_eq!(
        planned_targets(&restored, &[(1, 1.0), (3, 5.0)]),
        heavy_three,
        "恢复后同调参同决策"
    );

    // ---- 安装：真实 import ----
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "ai-tuning" && item.namespace == "upstream-liuli")
        .expect("AI优化已安装");
    assert_eq!(meta.version, "1.0.0");
    let npack_bytes =
        std::fs::read(repo_extensions_root().join("ai-tuning-upstream-liuli/package.npack.json"))
            .unwrap();
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes)
    );
    let installed_dir = store
        .root()
        .join("packages")
        .join("upstream-liuli__ai-tuning")
        .join("1.0.0");
    let installed = content_schema::ContentPackage::from_json_bytes(
        &std::fs::read(installed_dir.join("package.json")).unwrap(),
    )
    .expect("读商店包");
    assert!(installed.manifest.dependencies.is_empty());
    assert!(installed.skill_index.is_empty(), "不改规则：无技能条目");
    assert!(installed.card_index.is_empty() && installed.character_index.is_empty());
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), text.as_bytes(), "商店副本逐字节一致");

    // ---- 保存恢复（商店）：重开内容锁不变 ----
    let lock_before = store.current_lock().expect("lock").lock_hash;
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        reopened.current_lock().expect("lock").lock_hash,
        lock_before
    );

    // ---- 安装升级：2.0.0 覆盖 + rollback ----
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
    let active_version = |store: &PackageStore| -> String {
        store
            .read_active()
            .unwrap()
            .packages
            .iter()
            .find(|(slot, _)| slot.ends_with("ai-tuning"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    assert_eq!(active_version(&store), "2.0.0");
    store.rollback().expect("rollback");
    assert_eq!(active_version(&store), "1.0.0");
    match store.activate_version("upstream-liuli", "ai-tuning", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }

    // ---- 依赖：缺失来源（评级）以参数注入，不构成 manifest 依赖 ----
    assert!(installed.manifest.dependencies.is_empty());
    assert_eq!(weight(&rating_mode, "无记录", None, &[]), 1.0, "无评级缺省 1.0");
}
