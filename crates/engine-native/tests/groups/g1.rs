//! engine-native 集成测试分组 `g1`：G1 各包（`g1_*`，沙海除外）。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../g1_cardpile.rs"]
mod g1_cardpile;

#[path = "../g1_d3.rs"]
mod g1_d3;

#[path = "../g1_hearth.rs"]
mod g1_hearth;

#[path = "../g1_huanle.rs"]
mod g1_huanle;

#[path = "../g1_wudian.rs"]
mod g1_wudian;

#[path = "../g1_wuxing.rs"]
mod g1_wuxing;

#[path = "../u01_ddd_eight.rs"]
mod u01_ddd_eight;

#[path = "../l28_thunder_eight_match.rs"]
mod l28_thunder_eight_match;

#[path = "../l28_thunder_eight_triggers.rs"]
mod l28_thunder_eight_triggers;

#[path = "../l28_thunder_eight_batch_b.rs"]
mod l28_thunder_eight_batch_b;

#[path = "../l27_beautify_match.rs"]
mod l27_beautify_match;

#[path = "../l27_beautify_batch_b.rs"]
mod l27_beautify_batch_b;

#[path = "../l19_doomsday_match.rs"]
mod l19_doomsday_match;

#[path = "../l20_mark_supplement_match.rs"]
mod l20_mark_supplement_match;
