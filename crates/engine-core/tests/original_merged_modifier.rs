//! O2 全量合并第二批（2026-10-14）：家族 modifier 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

#[path = "original_merged_modifier/original_modifier_temp_skills.rs"]
mod original_modifier_temp_skills;
