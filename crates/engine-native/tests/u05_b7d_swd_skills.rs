//! U05-7d (`swd/skill.js:4132..8107` 115 技能 + `swd/character.js:3..511` 79 武将) 全量等价与回归测试套。
//! 注册于 `tests/groups/g1_shahai.rs`；零 `debug_execute_effect`。

#[path = "u05_b7d_swd_skills/common.rs"]
mod common;
#[path = "u05_b7d_swd_skills/part1_parity_duijue_tanlin.rs"]
mod part1_parity_duijue_tanlin;
#[path = "u05_b7d_swd_skills/part2_new_mechanics.rs"]
mod part2_new_mechanics;
#[path = "u05_b7d_swd_skills/part3_fixes_and_combos.rs"]
mod part3_fixes_and_combos;
#[path = "u05_b7d_swd_skills/part4_round2_parity.rs"]
mod part4_round2_parity;
#[path = "u05_b7d_swd_skills/part5_round3_fixes.rs"]
mod part5_round3_fixes;
