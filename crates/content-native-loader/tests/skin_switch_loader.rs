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

// ---- 切片 2：其余动皮触发策略 ----

use content_native_loader::{
    damage_transform_allowed, group_changed, group_nature, resolve_transform, TransformFlags,
};
use std::collections::BTreeMap;

#[test]
fn damage_transform_is_once_per_avatar() {
    let mut flags = TransformFlags::default();
    assert!(damage_transform_allowed(&mut flags, true), "主将首次允许");
    assert!(!damage_transform_allowed(&mut flags, true), "主将二次取消");
    assert!(damage_transform_allowed(&mut flags, false), "副将独立首次允许");
    assert!(!damage_transform_allowed(&mut flags, false));
    assert!(flags.damage_primary && flags.damage_deputy);
}

#[test]
fn group_nature_and_change_detection() {
    assert_eq!(group_nature("wei"), Some("wei"));
    assert_eq!(group_nature("shu_x"), Some("shu"));
    assert_eq!(group_nature("wu"), Some("wu"));
    assert_eq!(group_nature("qun"), Some("qun"));
    assert_eq!(group_nature("shen"), None, "神/其它不归类");
    assert!(group_changed("wei", "shu"));
    assert!(!group_changed("wei", "wei_x"), "同归类不算换组");
    assert!(!group_changed("shen", "wei"), "未知归类不触发");
}

#[test]
fn resolves_transform_targets() {
    let mut special = BTreeMap::new();
    special.insert("t1".to_string(), "bone/skin_a".to_string());
    special.insert("t2".to_string(), "bone/".to_string());
    special.insert("t3".to_string(), "noslash".to_string());
    let has_skin = |bone: &str, skin: &str| bone == "bone" && skin == "skin_a";
    assert_eq!(
        resolve_transform(Some("t1"), &special, has_skin),
        Some(("bone".to_string(), "skin_a".to_string()))
    );
    assert_eq!(resolve_transform(Some("t2"), &special, has_skin), None, "空皮肤名");
    assert_eq!(resolve_transform(Some("t3"), &special, has_skin), None, "无分隔符");
    assert_eq!(resolve_transform(Some("missing"), &special, has_skin), None);
    assert_eq!(resolve_transform(None, &special, has_skin), None, "无 transform 配置");
    // 皮肤表中不存在 → 不解析
    assert_eq!(
        resolve_transform(Some("t1"), &special, |_, _| false),
        None
    );
}
