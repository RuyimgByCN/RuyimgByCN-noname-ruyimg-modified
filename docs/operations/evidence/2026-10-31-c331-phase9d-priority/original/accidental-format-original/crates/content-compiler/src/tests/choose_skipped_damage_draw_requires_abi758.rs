//! N01 平寇摸牌效果：新 DSL 关键字的 ABI/参数门禁。

use super::*;

const GOOD: &str = r#"
skill skipped_draw(990301) {
  trigger phase_end owner_is_target {
    effect choose_other_targets_by_skipped_phase_count_damage_draw(self, 1);
  }
}
"#;

#[test]
fn skipped_damage_draw_compiles_at_abi758_and_emits_target_choice() {
    let package = compile_source_named("skipped-draw", GOOD).expect("compile");
    assert_eq!(package.manifest.rule_vm_abi_version, 758);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == 990301 && trigger.event == "phase_end"));
}

#[test]
fn skipped_damage_draw_rejects_missing_amount() {
    let bad = GOOD.replace("(self, 1)", "(self)");
    let err = compile_source_named("skipped-draw-bad", &bad).unwrap_err();
    let text = format!("{err:?}");
    assert!(
        text.contains("choose_other_targets_by_skipped_phase_count_damage_draw")
            || text.contains("expects 2")
    );
}
