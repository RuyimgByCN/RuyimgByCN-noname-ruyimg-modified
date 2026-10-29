//! engine-core 集成测试分组 `original_shenhua`：O2 家族文件 shenhua、refresh、standard。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../original_merged_refresh.rs"]
mod original_merged_refresh;

#[path = "../original_merged_shenhua.rs"]
mod original_merged_shenhua;

#[path = "../original_merged_standard.rs"]
mod original_merged_standard;
