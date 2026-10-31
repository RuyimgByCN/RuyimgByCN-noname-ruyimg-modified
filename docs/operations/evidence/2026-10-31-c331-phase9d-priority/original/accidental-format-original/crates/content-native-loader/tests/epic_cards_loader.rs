//! L10 史诗卡牌装载器：配置解析与十常侍面依赖判定。

use content_native_loader::{
    parse_epic_config, ShichangshiPlan, EXTERNAL_SKILLS, OVERRIDE_CHARACTER, REQUIRED_EXTENSION,
};

const CONFIG: &str = r#"{
  "source": "noname-liuli",
  "upstream_path": "noname/extension/史诗卡牌/extension.js",
  "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
  "cardEffects": true,
  "skillEffects": true,
  "skillEffectsScale": 1.0,
  "cardEmoji": false,
  "cardFold": false,
  "cardPhantom": false,
  "meihuaskills": false,
  "shichangshi": false,
  "skinEffects": false
}"#;

#[test]
fn parses_config_and_presentation_flags() {
    let cfg = parse_epic_config(CONFIG).expect("解析");
    assert_eq!(cfg.source, "noname-liuli");
    assert!(cfg.card_effects && cfg.skill_effects);
    assert!(!cfg.card_emoji && !cfg.card_fold && !cfg.card_phantom && !cfg.skin_effects);
    let p = cfg.presentation();
    assert!(p.card_effects && p.skill_effects);
    assert!(!p.card_emoji);
}

#[test]
fn rejects_bad_scale_and_missing_fields() {
    let bad = CONFIG.replace("\"skillEffectsScale\": 1.0", "\"skillEffectsScale\": 99.0");
    assert!(parse_epic_config(&bad).is_err(), "缩放越界应报错");
    let missing = CONFIG.replace("\"source\": \"noname-liuli\",", "");
    assert!(parse_epic_config(&missing).is_err(), "缺 source 应报错");
}

#[test]
fn shichangshi_plan_tracks_external_dependency() {
    let mut cfg = parse_epic_config(CONFIG).expect("解析");
    assert_eq!(
        cfg.shichangshi_plan(true, |_| true),
        ShichangshiPlan::Disabled
    );
    cfg.shichangshi = true;
    // 武将不在目录
    assert_eq!(
        cfg.shichangshi_plan(false, |_| true),
        ShichangshiPlan::MissingCharacter(OVERRIDE_CHARACTER.to_string())
    );
    // 外部技能缺失（本仓未定义）
    assert_eq!(
        cfg.shichangshi_plan(true, |_| false),
        ShichangshiPlan::MissingSkill(EXTERNAL_SKILLS[0].to_string())
    );
    // 齐备 → 可执行覆盖
    match cfg.shichangshi_plan(true, |_| true) {
        ShichangshiPlan::Apply {
            character,
            skills,
            required_extension,
        } => {
            assert_eq!(character, OVERRIDE_CHARACTER);
            assert_eq!(skills.len(), EXTERNAL_SKILLS.len());
            assert_eq!(required_extension, REQUIRED_EXTENSION);
        }
        other => panic!("应为 Apply，实为 {other:?}"),
    }
}
