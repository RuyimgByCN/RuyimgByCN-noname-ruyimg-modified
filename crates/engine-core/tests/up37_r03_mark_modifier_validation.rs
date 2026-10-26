//! R03 平定目标限制 modifier 的权威加载门禁：非法 mode/value 即使编译产出也必须拒绝安装。

use engine_core::state::GameState;

fn load(source: &str) -> Result<(), String> {
    let package = content_compiler::compile_source_named("r03-mark-validation", source)
        .map_err(|error| format!("{error:?}"))?;
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .map_err(|error| format!("{error:?}"))
}

const GOOD: &str = r#"
skill yingba_mark_probe(990502) {
  modifier card_target_cancel_if_other_user_card_use_count_lt_owner_mark override 1;
}
"#;

#[test]
fn valid_mark_restriction_modifier_loads() {
    load(GOOD).expect("override 1 must load");
}

#[test]
fn invalid_mark_restriction_mode_or_value_is_rejected_at_load() {
    let add = GOOD.replace("override 1", "add 1");
    assert!(load(&add).is_err(), "add mode must be rejected at load");
    let two = GOOD.replace("override 1", "override 2");
    assert!(load(&two).is_err(), "override 2 must be rejected at load");
}
