//! engine-core 集成测试分组 `original_offline`：O2 家族文件 offline、extension、extra。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../original_merged_extension.rs"]
mod original_merged_extension;

#[path = "../original_merged_extra.rs"]
mod original_merged_extra;

#[path = "../original_merged_offline.rs"]
mod original_merged_offline;
