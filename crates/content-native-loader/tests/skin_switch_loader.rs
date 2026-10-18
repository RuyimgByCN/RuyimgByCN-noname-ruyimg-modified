//! L24 皮肤切换装载器：参数重应用退避与低血变身策略。

use content_native_loader::{
    lowhp_decision, parse_skin_switch_config, reapply_decision, reapply_on_init_allowed,
    LowHpState, ReapplyDecision, REAPPLY_BACKOFF_MS,
};

const CONFIG: &str = r#"{
  "source": "noname-liuli",
  "upstream_path": "noname/extension/皮肤切换/extension.js",
  "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
  "dynamicSkin": true,
  "useDynamic": true,
  "attackEffect": true,
  "isAttackFlipX": false,
  "hideHuanFu": false,
  "czgEnable": true,
  "cugDynamicBg": false,
  "showTopArc": false,
  "savedPositions": { "p1": 1.5, "p2": -0.25 }
}"#;

#[test]
fn parses_config_and_saved_positions() {
    let cfg = parse_skin_switch_config(CONFIG).expect("解析");
    assert!(cfg.dynamic_skin && cfg.use_dynamic && cfg.attack_effect && cfg.czg_enable);
    assert!(!cfg.attack_flip_x && !cfg.hide_huanfu && !cfg.cug_dynamic_bg && !cfg.show_top_arc);
    assert_eq!(cfg.saved_positions.get("p1"), Some(&1.5));
    assert_eq!(cfg.saved_positions.len(), 2);
    let bad = CONFIG.replace("\"p1\": 1.5", "\"p1\": \"x\"");
    assert!(parse_skin_switch_config(&bad).is_err(), "位置值非法应报错");
    let missing = CONFIG.replace("\"source\": \"noname-liuli\",", "");
    assert!(parse_skin_switch_config(&missing).is_err());
}

#[test]
fn reapply_backoff_follows_upstream_schedule() {
    assert_eq!(reapply_decision(true, 0), ReapplyDecision::Apply);
    // 依赖未就绪：按 500/1000/2000 退避，第 4 次放弃
    assert_eq!(reapply_decision(false, 0), ReapplyDecision::RetryAfter(500));
    assert_eq!(reapply_decision(false, 1), ReapplyDecision::RetryAfter(1000));
    assert_eq!(reapply_decision(false, 2), ReapplyDecision::RetryAfter(2000));
    assert_eq!(reapply_decision(false, 3), ReapplyDecision::GiveUp);
    assert_eq!(REAPPLY_BACKOFF_MS, [0, 500, 1000, 2000]);
    // 角色初始化过滤：需有 dynamic 且为主将
    assert!(reapply_on_init_allowed(true, true));
    assert!(!reapply_on_init_allowed(true, false));
    assert!(!reapply_on_init_allowed(false, true));
}

#[test]
fn lowhp_triggers_only_on_new_low() {
    let mut st = LowHpState::default();
    // 初值 = hp + 1000 → 首次受伤必为新低
    assert!(lowhp_decision(&mut st, true, 3, -1, false));
    assert_eq!(st.primary_lowest, Some(3));
    // 同值不触发
    assert!(!lowhp_decision(&mut st, true, 3, -1, false));
    // 更低触发
    assert!(lowhp_decision(&mut st, true, 2, -1, false));
    assert_eq!(st.primary_lowest, Some(2));
    // 回复且未开启 recover → 不触发（也不更新记录）
    assert!(!lowhp_decision(&mut st, true, 3, 1, false));
    assert_eq!(st.primary_lowest, Some(2));
    // 回复但开启 recover → 走新低判定（3 ≥ 2 → 不触发）
    assert!(!lowhp_decision(&mut st, true, 3, 1, true));
    // 主/副将各自独立记录
    assert!(lowhp_decision(&mut st, false, 4, -1, false));
    assert_eq!(st.deputy_lowest, Some(4));
    assert_eq!(st.primary_lowest, Some(2), "主将记录不受副将影响");
}
