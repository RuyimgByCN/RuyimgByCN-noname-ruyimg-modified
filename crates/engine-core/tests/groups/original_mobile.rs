//! engine-core 集成测试分组 `original_mobile`：O2 家族文件 mobile、tw、sixiang、shiji。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../original_merged_mobile.rs"]
mod original_merged_mobile;

#[path = "../original_merged_shiji.rs"]
mod original_merged_shiji;

#[path = "../original_merged_sixiang.rs"]
mod original_merged_sixiang;

#[path = "../original_merged_tw.rs"]
mod original_merged_tw;
