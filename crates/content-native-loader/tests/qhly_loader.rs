//! L09 千幻聆音装载器：换肤决策与随机皮肤策略。

use content_native_loader::{
    decide_skin, parse_qhly_config, pick_random_skin, SkinChangeRule, SkinChangeState, SkinTrigger,
};

const CONFIG: &str = r#"{
  "source": "noname-liuli",
  "upstream_path": "noname/extension/千幻聆音/extension.js",
  "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
  "qhly_randskin": true,
  "qhly_randskin_me": false,
  "qhly_autoChangeSkin": "close"
}"#;

fn rule(source: &str, phase_only: bool, target: &str) -> SkinChangeRule {
    SkinChangeRule {
        source: source.into(),
        phase_only,
        target: Some(target.into()),
    }
}

#[test]
fn parses_config_and_auto_switch_flag() {
    let cfg = parse_qhly_config(CONFIG).expect("解析");
    assert!(cfg.rand_skin && !cfg.rand_skin_me);
    assert_eq!(cfg.auto_change_skin, None, "close → 关闭");
    let on = CONFIG.replace("\"qhly_autoChangeSkin\": \"close\"", "\"qhly_autoChangeSkin\": \"all\"");
    assert_eq!(
        parse_qhly_config(&on).expect("解析").auto_change_skin,
        Some("all".into())
    );
    let missing = CONFIG.replace("\"source\": \"noname-liuli\",", "");
    assert!(parse_qhly_config(&missing).is_err());
    assert!(parse_qhly_config("not json").is_err());
}

#[test]
fn only_hp_rules_participate() {
    let mut st = SkinChangeState::default();
    let non_hp = rule("battle", false, "x");
    assert!(!non_hp.is_active());
    assert_eq!(decide_skin(&non_hp, SkinTrigger::HpChange { delta: -1 }, &mut st), None);
    let hp = rule("hp_1", false, "s1");
    assert!(hp.is_active());
    assert_eq!(
        decide_skin(&hp, SkinTrigger::HpChange { delta: -1 }, &mut st),
        Some("s1".into())
    );
    assert!(st.switched_on_hp_loss, "掉血置位");
}

#[test]
fn hp_loss_gates_phase_end_switch() {
    // 掉血换过 → 阶段结束不再换（并清标志）
    let mut st = SkinChangeState::default();
    let r = rule("hp_2", false, "s2");
    assert_eq!(decide_skin(&r, SkinTrigger::HpChange { delta: -2 }, &mut st), Some("s2".into()));
    assert_eq!(decide_skin(&r, SkinTrigger::PhaseEnd, &mut st), None, "同回合阶段结束跳过");
    assert!(!st.switched_on_hp_loss, "标志已清");
    // 未掉血 → 阶段结束切换
    assert_eq!(decide_skin(&r, SkinTrigger::PhaseEnd, &mut st), Some("s2".into()));
    // 回复不切换
    assert_eq!(decide_skin(&r, SkinTrigger::HpChange { delta: 1 }, &mut st), None);
}

#[test]
fn random_skin_filters_banned_and_terms() {
    let cands = vec![
        "default".to_string(),
        "banned_one".to_string(),
        "娘化版".to_string(),
        "云涯皮肤".to_string(),
        "ok_skin".to_string(),
    ];
    let banned = vec!["banned_one".to_string()];
    // 过滤后候选 = [default, ok_skin, 默认(false)]（保持原序，默认追加在末尾）
    assert_eq!(
        pick_random_skin(&cands, &banned, &["娘化", "云涯"], 0),
        Some("default".into())
    );
    assert_eq!(
        pick_random_skin(&cands, &banned, &["娘化", "云涯"], 1),
        Some("ok_skin".into())
    );
    assert_eq!(pick_random_skin(&cands, &banned, &["娘化", "云涯"], 2), None, "默认(false)入池");
    // 越界回绕
    assert_eq!(
        pick_random_skin(&cands, &banned, &["娘化", "云涯"], 3),
        Some("default".into())
    );
    // 全部被过滤 → 仅剩默认
    let all_banned = vec![
        "a".to_string(),
        "b".to_string(),
    ];
    assert_eq!(
        pick_random_skin(&all_banned, &["a".into(), "b".into()], &[], 0),
        None
    );
}
