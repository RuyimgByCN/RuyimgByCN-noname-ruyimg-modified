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
