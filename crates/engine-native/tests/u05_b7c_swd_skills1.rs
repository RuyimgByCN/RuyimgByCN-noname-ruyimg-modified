//! Batch U05-7c (`swd` 武将技前半 — 115 项) 真实对局与上游等价回归测试。
//!
//! 全部经由真实建局、出牌、响应、主动技与存读档链路验证，不使用调试直执行入口。

#[path = "u05_b7c_swd_skills1/common.rs"]
mod common;
#[path = "u05_b7c_swd_skills1/part1_triggers_and_sha.rs"]
mod part1_triggers_and_sha;
#[path = "u05_b7c_swd_skills1/part2_active_skills.rs"]
mod part2_active_skills;
#[path = "u05_b7c_swd_skills1/part3_round2_fixes.rs"]
mod part3_round2_fixes;
#[path = "u05_b7c_swd_skills1/part4_round3_fixes.rs"]
mod part4_round3_fixes;
