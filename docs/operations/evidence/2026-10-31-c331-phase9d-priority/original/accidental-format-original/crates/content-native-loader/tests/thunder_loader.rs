//! L28 雷霆万钧装载器：配置、同名替换表、标记文本补丁。

use content_native_loader::{
    mark_labels, parse_thunder_config, CHG_HERO, EXCLUSIVE_PACK, SAME_HERO,
};

const CONFIG: &str = r#"{
  "source": "noname-liuli",
  "upstream_path": "noname/extension/雷霆万钧/extension.js",
  "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
  "gameSpeed": 4,
  "replaceCha": true,
  "hidePack": true,
  "forbidSame": false,
  "UIpatch": true,
  "jinyu": true,
  "shhbiaoji": true,
  "shh_new": false,
  "jiufatext": true,
  "guanning": false
}"#;

#[test]
fn parses_config_and_speed_bounds() {
    let cfg = parse_thunder_config(CONFIG).expect("解析");
    assert_eq!(cfg.game_speed, 4);
    assert!(cfg.replace_cha && cfg.hide_pack && cfg.ui_patch);
    assert!(!cfg.forbid_same && !cfg.shh_new && !cfg.guanning);
    let bad = CONFIG.replace("\"gameSpeed\": 4", "\"gameSpeed\": 99");
    assert!(parse_thunder_config(&bad).is_err(), "速度越界应报错");
    let missing = CONFIG.replace("\"source\": \"noname-liuli\",", "");
    assert!(parse_thunder_config(&missing).is_err(), "缺 source 应报错");
}

#[test]
fn replace_plan_pairs_parallel_tables() {
    assert_eq!(SAME_HERO.len(), CHG_HERO.len(), "两表平行");
    let mut cfg = parse_thunder_config(CONFIG).expect("解析");
    let plan = cfg.replace_plan();
    assert_eq!(plan.pairs.len(), SAME_HERO.len());
    assert_eq!(plan.pairs[0], ("majun".to_string(), "th_majun".to_string()));
    assert_eq!(
        plan.pairs[SAME_HERO.len() - 1],
        ("shen_zhouyu".to_string(), "th_shen_zhouyu".to_string())
    );
    assert!(cfg.hides_exclusive_pack());
    assert_eq!(EXCLUSIVE_PACK, "thunder");
    // 关闭替换 → 空计划
    cfg.replace_cha = false;
    assert!(cfg.replace_plan().pairs.is_empty());
}

#[test]
fn mark_labels_follow_shh_new_switch() {
    let mut cfg = parse_thunder_config(CONFIG).expect("解析");
    let plain = mark_labels(&cfg, 0, 1);
    assert_eq!(
        plain,
        vec![
            ("miaojian".to_string(), "妙剑1级".to_string()),
            ("shhlianhua".to_string(), "莲华2级".to_string()),
        ]
    );
    cfg.shh_new = true;
    let super_script = mark_labels(&cfg, 0, 1);
    assert_eq!(super_script[0].1, "妙剑¹");
    assert_eq!(super_script[1].1, "莲华²");
    cfg.shh_biaoji = false;
    assert!(mark_labels(&cfg, 0, 1).is_empty(), "补丁关闭 → 空");
}
