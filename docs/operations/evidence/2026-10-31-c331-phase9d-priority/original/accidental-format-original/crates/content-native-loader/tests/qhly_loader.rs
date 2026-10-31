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
    let on = CONFIG.replace(
        "\"qhly_autoChangeSkin\": \"close\"",
        "\"qhly_autoChangeSkin\": \"all\"",
    );
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
    assert_eq!(
        decide_skin(&non_hp, SkinTrigger::HpChange { delta: -1 }, &mut st),
        None
    );
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
    assert_eq!(
        decide_skin(&r, SkinTrigger::HpChange { delta: -2 }, &mut st),
        Some("s2".into())
    );
    assert_eq!(
        decide_skin(&r, SkinTrigger::PhaseEnd, &mut st),
        None,
        "同回合阶段结束跳过"
    );
    assert!(!st.switched_on_hp_loss, "标志已清");
    // 未掉血 → 阶段结束切换
    assert_eq!(
        decide_skin(&r, SkinTrigger::PhaseEnd, &mut st),
        Some("s2".into())
    );
    // 回复不切换
    assert_eq!(
        decide_skin(&r, SkinTrigger::HpChange { delta: 1 }, &mut st),
        None
    );
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
    assert_eq!(
        pick_random_skin(&cands, &banned, &["娘化", "云涯"], 2),
        None,
        "默认(false)入池"
    );
    // 越界回绕
    assert_eq!(
        pick_random_skin(&cands, &banned, &["娘化", "云涯"], 3),
        Some("default".into())
    );
    // 全部被过滤 → 仅剩默认
    let all_banned = vec!["a".to_string(), "b".to_string()];
    assert_eq!(
        pick_random_skin(&all_banned, &["a".into(), "b".into()], &[], 0),
        None
    );
}

// ---- 切片 2：索引面（真名/回退/封禁/列表过滤）----

use content_native_loader::{
    erase_ext, filter_skin_list, is_skin_banned, real_name, resolve_gz_fallback, SkinShareTable,
};

#[test]
fn resolves_real_name_and_gz_fallback() {
    let mut share = SkinShareTable::new();
    share.insert("alias_a".to_string(), "real_a".to_string());
    share.insert("alias_empty".to_string(), String::new());
    assert_eq!(real_name("alias_a", &share), "real_a");
    assert_eq!(
        real_name("alias_empty", &share),
        "alias_empty",
        "空别名不生效"
    );
    assert_eq!(real_name("plain", &share), "plain");

    // gz_ 前缀：国战开启且模式匹配且有国战皮肤 → 保持
    assert_eq!(
        resolve_gz_fallback("gz_zhangfei", true, true, true, |_| true),
        "gz_zhangfei"
    );
    // 否则回退到已知角色
    assert_eq!(
        resolve_gz_fallback("gz_zhangfei", false, false, false, |n| n == "zhangfei"),
        "zhangfei"
    );
    // 子名非已知角色 → 保持原名
    assert_eq!(
        resolve_gz_fallback("gz_unknown", false, false, false, |_| false),
        "gz_unknown"
    );
    // 无前缀 → 原样
    assert_eq!(
        resolve_gz_fallback("zhangfei", false, false, false, |_| true),
        "zhangfei"
    );
}

#[test]
fn erases_extension_and_checks_bans() {
    assert_eq!(erase_ext("skin/liubei.png"), "skin/liubei");
    assert_eq!(erase_ext("noext"), "noext");
    assert_eq!(erase_ext("a.b.c"), "a.b");
    assert_eq!(erase_ext(""), "");
    let bans = vec!["liubei-[original]".to_string(), "caocao-skin1".to_string()];
    assert!(is_skin_banned("liubei", None, &bans), "空皮肤=[original]");
    assert!(is_skin_banned("caocao", Some("skin1"), &bans));
    assert!(!is_skin_banned("liubei", Some("skin1"), &bans));
    assert!(!is_skin_banned("liubei", None, &[]), "未配置名单 → 不封禁");
}

#[test]
fn filters_skin_list_by_hide_and_lock() {
    let list = vec![
        "s1".to_string(),
        "hidden".to_string(),
        "locked".to_string(),
        "s2".to_string(),
    ];
    let hidden = |s: &str| s == "hidden";
    let lock = |s: &str| s == "locked";
    // locked=true（默认）：只剔隐藏
    assert_eq!(
        filter_skin_list(&list, hidden, lock, true),
        vec!["s1".to_string(), "locked".to_string(), "s2".to_string()]
    );
    // locked=false：隐藏与锁定都剔
    assert_eq!(
        filter_skin_list(&list, hidden, lock, false),
        vec!["s1".to_string(), "s2".to_string()]
    );
}
