//! ADR 0632：`has_skill(player, skill_def)` 的第二参必须是 u32 技能定义，
//! 不能被通用表达式检查当成非法标识符而拒绝。

use super::*;

const SOURCE: &str = r#"
skill has_skill_probe(424242) {
  trigger phase_end owner_is_target {
    if has_skill(self, 2187983153) {
      effect draw_cards(self, 1);
    }
  }
}
"#;

#[test]
fn has_skill_accepts_literal_skill_def() {
    let compiled = compile_source_named("has-skill-probe", SOURCE).expect("has_skill 应可编译");
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| { trigger.skill_def_id == 424242 && trigger.event == "phase_end" }));
}
