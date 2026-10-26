//! R03 英霸目标过滤：active targets 的 `max_hp_gt` 解析/持久化/重复门禁。

use super::*;

const GOOD: &str = r#"
skill yingba_probe(990401) {
  active play_phase uses_per_turn 1 targets 1 1 other max_hp_gt 1 {
    effect draw_cards(self, 1);
  }
}
"#;

#[test]
fn max_hp_gt_is_published_on_active_targets() {
    let package = compile_source_named("max-hp-gt", GOOD).expect("compile");
    let targets = package.active_index[0]
        .targets
        .as_ref()
        .expect("active targets");
    assert_eq!(targets.max_hp_gt, Some(1));
    assert!(!targets.allow_self);
}

#[test]
fn duplicate_max_hp_gt_is_rejected() {
    let bad = GOOD.replace("max_hp_gt 1", "max_hp_gt 1 max_hp_gt 2");
    let err = compile_source_named("max-hp-gt-bad", &bad).unwrap_err();
    assert!(format!("{err:?}").contains("duplicate max_hp_gt"));
}

#[test]
fn max_hp_gt_survives_package_json_roundtrip() {
    let package = compile_source_named("max-hp-gt-roundtrip", GOOD).expect("compile");
    let bytes = serde_json::to_vec(&package).expect("serialize");
    let decoded = content_schema::ContentPackage::from_json_bytes(&bytes).expect("decode");
    assert_eq!(
        decoded.active_index[0]
            .targets
            .as_ref()
            .and_then(|targets| targets.max_hp_gt),
        Some(1)
    );
}
