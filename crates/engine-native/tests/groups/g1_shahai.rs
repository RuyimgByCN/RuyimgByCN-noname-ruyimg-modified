//! engine-native 集成测试分组 `g1_shahai`：G1 沙海各包（`g1_shahai_*`）。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../g1_shahai_gujian.rs"]
mod g1_shahai_gujian;

#[path = "../g1_shahai_gwent.rs"]
mod g1_shahai_gwent;

#[path = "../g1_shahai_mtg.rs"]
mod g1_shahai_mtg;

#[path = "../g1_shahai_swd.rs"]
mod g1_shahai_swd;

#[path = "../g1_shahai_yunchou.rs"]
mod g1_shahai_yunchou;

#[path = "../g1_shahai_zhenfa.rs"]
mod g1_shahai_zhenfa;

#[path = "../decision34_round4_u05_shahai.rs"]
mod decision34_round4_u05_shahai;

#[path = "../u05_b1_fixes.rs"]
mod u05_b1_fixes;

#[path = "../u05_b2_gujian_fixes.rs"]
mod u05_b2_gujian_fixes;

#[path = "../u05_b3_ow_fixes.rs"]
mod u05_b3_ow_fixes;

#[path = "../u05_b6a_hearth_fixes.rs"]
mod u05_b6a_hearth_fixes;

#[path = "../u05_b6b_hearth_fixes.rs"]
mod u05_b6b_hearth_fixes;

#[path = "../u05_b5a_gwent_cards_fixes.rs"]
mod u05_b5a_gwent_cards_fixes;

#[path = "../u05_b5b_gwent_chars_fixes.rs"]
mod u05_b5b_gwent_chars_fixes;

#[path = "../u05_b4_xianjian_fixes.rs"]
mod u05_b4_xianjian_fixes;

#[path = "../u05_b7a_swd_cards.rs"]
mod u05_b7a_swd_cards;

#[path = "../u05_b7b_swd_cards.rs"]
mod u05_b7b_swd_cards;

#[path = "../u05_b7c_swd_skills1.rs"]
mod u05_b7c_swd_skills1;
