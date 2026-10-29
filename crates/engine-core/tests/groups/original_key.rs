//! engine-core 集成测试分组 `original_key`：O2 家族文件 key、bingshi、collab、jsrg。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../original_merged_bingshi.rs"]
mod original_merged_bingshi;

#[path = "../original_merged_collab.rs"]
mod original_merged_collab;

#[path = "../original_merged_jsrg.rs"]
mod original_merged_jsrg;

#[path = "../original_merged_key.rs"]
mod original_merged_key;
