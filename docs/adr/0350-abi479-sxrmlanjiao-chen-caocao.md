# ADR 0350：ABI 479 揽娇循环查看复合 + sxrm 嗔曹操

本批为 sxrm 包嗔曹操（`sxrm_chen_caocao`，3686299582）的「揽娇」（`sxrmlanjiao`，
3315968200）完整移植，需要 ABI 479 复合 `resolve_sxrmlanjiao(player)`。

## 原语设计（ABI 479）

- DSL 表面：`resolve_sxrmlanjiao(player)`；`trigger phase_play owner_is_target`。
  编译为 `effect_tag::RESOLVE_SXRMLANJIAO`（tag 405）。
- 引擎：`EffectRequest::ResolveSxrmlanjiao { player, target, stage, view, skill }`；
  阶段机（Custom 续体 `RESOLVE_SXRMLANJIAO_CUSTOM_CHOICE`）：
  - 阶段 0：选目标（手牌 ≥ 2 且无「已用」mark；mark key = 目标 id）；
  - 阶段 1：查看目标前两张手牌（`RevealCards`）→ 目标选择 [停止, 重新选择]；
  - 重新选择：目标失去 1 点体力并摸一张 → 重发阶段 1（循环）；
  - 停止：拥有者获得最后查看的两张牌，置目标 mark（每目标每回合一次）。

## 文档化近似

旧 JS 由拥有者自选查看两张且随机；DSL 确定性取前两张。

## 打包与验证

- `sxrmlanjiao.skill` 加入 sxrm README 显式列表；`sxrm_chen_caocao`（男/魏/4 体力，
  技能 [sxrmlanjiao]）加入 sxrm `characters.json`；catalog 标记 `ported`。
- sxrm npack 重建（ABI 479，5 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sxrm/`。
- 运行时测试 `original_sxrm_sxrmlanjiao.rs`（3 用例全绿）：停止获得所查看两张；
  重新选择循环（失 1 体 + 摸 1 后再次查看并停止）；每目标每回合一次（二次触发
  无目标可选）。
- 旧 JavaScript 不进入 npack 或运行时。
