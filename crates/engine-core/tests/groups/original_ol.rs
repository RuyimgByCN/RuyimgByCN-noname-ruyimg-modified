//! engine-core 集成测试分组 `original_ol`：O2 家族文件 huicui、old、onlyol、diy、clan、shahai、modifier。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../original_merged_clan.rs"]
mod original_merged_clan;

#[path = "../original_merged_diy.rs"]
mod original_merged_diy;

#[path = "../original_merged_huicui.rs"]
mod original_merged_huicui;

#[path = "../original_merged_modifier.rs"]
mod original_merged_modifier;

#[path = "../original_merged_old.rs"]
mod original_merged_old;

#[path = "../original_merged_onlyol.rs"]
mod original_merged_onlyol;

#[path = "../original_merged_shahai.rs"]
mod original_merged_shahai;
