# ADR 0371：ABI 500 胆守目标历史计数、弃牌伤害 + 手杀界朱然

本批完整移植 MOBILE 手杀界朱然（`xin_zhuran`，4142079595）的「胆守」
（`mobiledanshou`，2790027559）。旧 JavaScript 仅作为语义来源，不进入运行时。

## 原语设计（ABI 500）

- `owner_alive_other_phase_end_mobiledanshou` 仅在加载相关内容时启用本回合
  “用牌者—目标”计数，并验证零次摸牌或等量弃牌分支当前可执行。
- `resolve_mobiledanshou(player, target)` 在计数为零时自动摸一张；计数非零时允许取消，
  接受后精确弃置等量手牌或装备牌，再通过尾续接对结束阶段角色造成一点伤害。
- 选择恢复时重新校验回合、计数、技能实例与可弃牌，避免客户端提交过期选择。

## 打包与验证

- `mobiledanshou.skill` 与 `xin_zhuran` 加入 MOBILE 包并同步 `extensions/mobile/`。
- 黄金测试覆盖零次摸牌、两次指定后的等量弃牌伤害及角色技能闭包。
