//! R03 平定目标限制：modifier target 的发布与模式门禁。

use super::*;

const GOOD: &str = r#"
skill yingba_mark_probe(990501) {
  modifier card_target_cancel_if_other_user_card_use_count_lt_owner_mark override 1;
}
"#;

#[test]
fn mark_target_restriction_is_published_with_override_one() {
    let package = compile_source_named("yingba-mark-probe", GOOD).expect("compile");
    let modifier = package.modifier_index[0].clone();
    assert_eq!(
        modifier.target,
        "card_target_cancel_if_other_user_card_use_count_lt_owner_mark"
    );
    assert_eq!(modifier.mode, "override");
    assert_eq!(modifier.value, 1);
}


#[test]
fn mark_target_restriction_rejects_add_mode_or_other_value() {
    let bad_mode = GOOD.replace("override 1", "add 1");
    assert!(compile_source_named("yingba-mark-bad-mode", &bad_mode).is_err());
    let bad_value = GOOD.replace("override 1", "override 2");
    assert!(compile_source_named("yingba-mark-bad-value", &bad_value).is_err());
}
