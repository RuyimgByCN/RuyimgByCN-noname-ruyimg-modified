//! L11（太虚幻境）Batch 2（83 技能：4 侍从全局钩子 + 26 侍从技 B 组 + 53 武将技 D/E/F 组）真实事件链回归。

#[path = "l11_taixu_b2/common.rs"]
mod common;
#[path = "l11_taixu_b2/hooks_servants.rs"]
mod hooks_servants;
#[path = "l11_taixu_b2/round2_fixes.rs"]
mod round2_fixes;
#[path = "l11_taixu_b2/skills_d.rs"]
mod skills_d;
#[path = "l11_taixu_b2/skills_e.rs"]
mod skills_e;
#[path = "l11_taixu_b2/skills_f.rs"]
mod skills_f;
