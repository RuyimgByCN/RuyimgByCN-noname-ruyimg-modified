//! L01 AI优化装载器：参数解析与结算口径。

use content_native_loader::{
    benefit, parse_ai_tuning_config, rating_to_weight, threaten, weight, CfMode, QzMode,
};

const CONFIG: &str = r#"{
  "source": "noname-liuli",
  "upstream_path": "noname/extension/AI优化/extension.js",
  "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
  "cf": { "gjcxs_zhugeliang3": 1.9, "gjcxs_caojie1": 0.7 },
  "qz": { "shen_zhugeliang": 1.75 },
  "applyCf": "pj",
  "applyQz": true,
  "ckQz": "cf",
  "qzCf": true,
  "viewAtt": false,
  "nhFriends": true,
  "ntAoe": false,
  "banQh": false
}"#;

#[test]
fn parses_modes_and_tables() {
    let cfg = parse_ai_tuning_config(CONFIG).expect("解析");
    assert_eq!(cfg.apply_cf, CfMode::Auto);
    assert_eq!(cfg.ck_qz, QzMode::Threat);
    assert!(cfg.qz_cf && cfg.apply_qz && cfg.nh_friends);
    assert_eq!(threaten(&cfg, "gjcxs_zhugeliang3"), Some(1.9));
    assert_eq!(threaten(&cfg, "未配置技"), None);
    // 非法模式/越界数值
    let bad_mode = CONFIG.replace("\"applyCf\": \"pj\"", "\"applyCf\": \"xx\"");
    assert!(parse_ai_tuning_config(&bad_mode).is_err());
    let bad_value = CONFIG.replace("\"gjcxs_caojie1\": 0.7", "\"gjcxs_caojie1\": 999");
    assert!(parse_ai_tuning_config(&bad_value).is_err());
    let missing = CONFIG.replace("\"source\": \"noname-liuli\",", "");
    assert!(parse_ai_tuning_config(&missing).is_err());
}

#[test]
fn weight_resolution_follows_ckqz() {
    let mut cfg = parse_ai_tuning_config(CONFIG).expect("解析");
    // 表内有值优先
    assert_eq!(weight(&cfg, "shen_zhugeliang", Some(5.0), &[]), 1.75);
    // Threat：取技能威胁度最大值
    assert_eq!(
        weight(
            &cfg,
            "无记录武将",
            None,
            &["gjcxs_caojie1", "gjcxs_zhugeliang3"]
        ),
        1.9
    );
    assert_eq!(weight(&cfg, "无记录武将", None, &[]), 1.0);
    // Rating：评级线性映射 [0.8, 1.95]
    cfg.ck_qz = QzMode::Rating;
    assert!((weight(&cfg, "无记录武将", Some(0.0), &[]) - 0.8).abs() < 1e-9);
    assert!((weight(&cfg, "无记录武将", Some(5.0), &[]) - 1.95).abs() < 1e-9);
    let mid = weight(&cfg, "无记录武将", Some(2.5), &[]);
    assert!((mid - 1.375).abs() < 1e-9, "中点评级权重 {mid}");
    // 评级越界钳制
    assert!((rating_to_weight(9.0) - 1.95).abs() < 1e-9);
    // Off：恒 1.0
    cfg.ck_qz = QzMode::Off;
    assert_eq!(weight(&cfg, "无记录武将", Some(5.0), &[]), 1.0);
}

#[test]
fn benefit_applies_weight_only_when_enabled() {
    let mut cfg = parse_ai_tuning_config(CONFIG).expect("解析");
    assert!((benefit(&cfg, 2.0, 3.0) - 6.0).abs() < 1e-9, "qzCf 开启：权重×结果");
    cfg.qz_cf = false;
    assert!((benefit(&cfg, 2.0, 3.0) - 3.0).abs() < 1e-9, "关闭：原结果");
}

#[test]
fn yiyi_card_target_selection_and_ai_benefit_matches_upstream() {
    // Upstream `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9216-9253`:
    // - `selectTarget: [-1, -1]` in `guozhan` / `versus`, `[1, 3]` in other modes (`extension.js:9216-9219`)
    // - `ai.wuxie` returns `0` (`extension.js:9225-9227`)
    // - `ai.result.target` (`extension.js:9234-9253`):
    //   `add + Math.sqrt(i / 3.6 + tars) / 2` where `tars = 0.01 * selected_targets_len`, `add = 1.0` if `hasSkillTag('noh')` else `0.0`
    assert_eq!(
        content_native_loader::yiyi_select_target_range("identity"),
        (1, 3)
    );
    assert_eq!(
        content_native_loader::yiyi_select_target_range("guozhan"),
        (-1, -1)
    );
    assert_eq!(
        content_native_loader::yiyi_select_target_range("versus"),
        (-1, -1)
    );
    assert_eq!(content_native_loader::yiyi_wuxie_weight(), 0);
    let self_benefit =
        content_native_loader::evaluate_yiyi_target_benefit(true, 4, 4, 0, true, 1, false);
    let expected_self = ((3.0_f64 / 3.6) + 0.01).sqrt() / 2.0;
    assert!((self_benefit - expected_self).abs() < 1e-9);
    let other_noh_benefit =
        content_native_loader::evaluate_yiyi_target_benefit(false, 2, 2, 0, false, 2, true);
    let expected_other_noh = 1.0 + ((2.0_f64 / 3.6) + 0.02).sqrt() / 2.0;
    assert!((other_noh_benefit - expected_other_noh).abs() < 1e-9);
}

