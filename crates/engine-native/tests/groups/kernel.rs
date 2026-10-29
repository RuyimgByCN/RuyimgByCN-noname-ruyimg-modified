//! engine-native 集成测试分组 `kernel`：内核、存档边界、回放场景、目录与守卫类测试。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../a0_tag_registry.rs"]
mod a0_tag_registry;

#[path = "../adr0786_j1_commit_window.rs"]
mod adr0786_j1_commit_window;

#[path = "../auth_archive.rs"]
mod auth_archive;

#[path = "../boundary_snapshot.rs"]
mod boundary_snapshot;

#[path = "../maintenance_guards.rs"]
mod maintenance_guards;

#[path = "../native_kernel.rs"]
mod native_kernel;

#[path = "../replay_scenarios.rs"]
mod replay_scenarios;

#[path = "../view_isolation.rs"]
mod view_isolation;
