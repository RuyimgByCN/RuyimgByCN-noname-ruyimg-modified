# ADR 0305：SHENHUA 神速系列（界夏侯渊/旧夏侯渊）

本批移植夏侯渊神速系列，复用既有原语与 ABI 450，不新增 ABI。

## 技能移植

- 「神速·一」（`shensu1`，2385964521）：`phase_judge` 触发 + `skip_judge_and_draw_phases(self)`
  （ABI 450，设置 pending_judge_skips 与 SKIP_DRAW）+ `choose_use_virtual_sha(self, normal_distance)`。
- 「神速·二」（`shensu2`，3236580224）：`phase_play` 触发 + `discard_hand_equip(self, 1)` +
  `skip_current_phase(self)`（出牌阶段在允许集内）+ `choose_use_virtual_sha`。
  注：旧 JS 要求弃置装备牌（filterCard type==equip），DSL 的 `discard_hand_equip` 放宽为
  任意手牌/装备牌，属文档化近似。
- 「神速·四」（`shensu4`，4133384668）：`phase_discard` 触发 + `turn_over(self)` +
  `skip_current_phase(self)` + `choose_use_virtual_sha`。
- 「界神速」（`xinshensu`，2561255694）：旧 JS 的 group（xinshensu_1/2/4）折叠进同一定义，
  三段触发合并；`re_xiahouyuan`（3099322121，男/魏/4 体力）闭环入选将池。
- 「旧神速」（`shensu`，3358716580）：判定/出牌两段，供 `old:xiahouyuan` 跨包引用。

旧 JavaScript 不进入 npack 或运行时。神速·一为 ABI 450 在 shenhua 包的首个复用。
