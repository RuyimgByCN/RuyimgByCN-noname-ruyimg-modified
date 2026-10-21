//! O2 全量合并第二批（2026-10-14）：家族 shahai 的 2 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_shahai/original_shahai_from_current_upgrades.rs"]
mod original_shahai_from_current_upgrades;

#[path = "original_merged_shahai/original_shahai_rewrite_batch.rs"]
mod original_shahai_rewrite_batch;
