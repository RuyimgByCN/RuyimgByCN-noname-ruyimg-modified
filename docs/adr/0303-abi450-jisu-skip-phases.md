# ADR 0303：ABI 450 原语设计 —— 济溯跳过阶段（collab 叶诗文）

本 ADR 登记 C 段首个 ABI 领取（450）及其原语设计；引擎实现待构建窗口开放后落地。

## 技能语义（旧 JS 仅作证据）

「济溯」（`clbjisu`，collab 包叶诗文）：判定阶段开始时，可选择跳过判定阶段与摸牌阶段，
视为对攻击范围内一名其他角色使用一张【杀】。

## 原语缺口

- `skip_current_phase` 只允许在 Draw/Play/Discard 阶段调用（engine-core
  `skip_current_phase` 显式拒绝 Judge）。
- `skip_judge_phase`/`skip_discard_phase` 为查询时判定的常驻 modifier，但判定阶段的
  跳过查询在 `phase_judge` 触发**之前**执行（turn.rs `Phase::Judge` 分支先查
  `skip_judge_phase` 再 `resolve_judge_zone`），无法由同一阶段的触发设置。
- 无「跳过下一摸牌阶段」原语（`SKIP_DRAW` 回合旗标存在但从未被内容设置）。

## ABI 450 设计（C 段，omp）

新增效果原语：

- `skip_judge_and_draw_phases(player)`：在 `phase_judge` 触发体内调用，设置
  `pending_judge_skips`（复用既有 `pending_judge_skip` 机制，turn.rs 已支持）与
  `turn_flag::SKIP_DRAW`（复用既有旗标，turn.rs 仅重置未设置）。随后
  `choose_use_virtual_sha(player, normal_distance)` 使用虚拟【杀】。

所需改动面：content-schema IR、rule-api 效果枚举、rule-vm bytecode 标签 + VM 分发、
content-compiler typecheck/codegen + ABI 450 门控、engine-core primitives、
hash.rs 与 version.rs 405→450。全部为追加式，不改变既有枚举序数。

## 状态

原语未落地；`clbjisu.skill` 待原语就绪后编写。叶诗文（`yeshiwen`，4195936154）另一技能
「水泳」（`clbshuiyong`，`damage_taken_cancel(fire)`）可先行纯 DSL 移植。
