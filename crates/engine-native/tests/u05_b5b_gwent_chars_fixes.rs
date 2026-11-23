//! U05-5b（`杀海拾遗 · 昆特牌 52 武将 + 69 技能`）全量真实事件测试套件。
//! 覆盖全部 52 名 `gw_*` 武将装载、69 个角色/衍生/全局技能正反分支、选择分支、取消、
//! 回合/轮次/死亡/空堆边界、AI 公开信息启发式、及挂起存档恢复。

#[path = "u05_b5b_gwent_chars/common.rs"]
mod common;
#[path = "u05_b5b_gwent_chars/group_a.rs"]
mod group_a;
#[path = "u05_b5b_gwent_chars/group_b.rs"]
mod group_b;
#[path = "u05_b5b_gwent_chars/group_c.rs"]
mod group_c;
#[path = "u05_b5b_gwent_chars/group_d.rs"]
mod group_d;
#[path = "u05_b5b_gwent_chars/round2.rs"]
mod round2;
#[path = "u05_b5b_gwent_chars/round3.rs"]
mod round3;
