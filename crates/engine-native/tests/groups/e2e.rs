//! engine-native 集成测试分组 `e2e`：端到端契约、会话生命周期与 4 阶 E2E 验证。
//! 登记规则与守卫见 `test_registry.rs`；本文件只登记模块，测试全名为 `<原文件名>::<函数名>`。

#[path = "../e2e/e2e_harness.rs"]
pub mod e2e_harness;

#[path = "../e2e/e2e_tier1_part1.rs"]
pub mod e2e_tier1_part1;

#[path = "../e2e/e2e_tier1_part2.rs"]
pub mod e2e_tier1_part2;

#[path = "../e2e/e2e_tier2_part1.rs"]
pub mod e2e_tier2_part1;

#[path = "../e2e/e2e_tier2_part2.rs"]
pub mod e2e_tier2_part2;

#[path = "../e2e/e2e_tier3_interactions.rs"]
pub mod e2e_tier3_interactions;

#[path = "../e2e/e2e_tier4_scenarios.rs"]
pub mod e2e_tier4_scenarios;

#[path = "../e2e/e2e_tier5_adversarial_c1.rs"]
pub mod e2e_tier5_adversarial_c1;
