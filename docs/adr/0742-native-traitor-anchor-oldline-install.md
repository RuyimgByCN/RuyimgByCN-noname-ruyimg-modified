# ADR 0742：TRAITOR 终局锚点重选与旧线安装/构建打通

日期：2026-09-16
状态：已实施（锚点）/ 部分完成（N12 旧线批次第一步）
关联：[0738](0738-native-caocao-jianxiong-guojia-jiji.md)、[0737](0737-native-damage-card-field-oldline-feasibility.md)、进度文档 N12/N06

## 1. TRAITOR 终局锚点重选

杀牌弃置泄漏修复（ADR 0738）使杀牌正常进入洗回循环，长局牌序随之确定性改变：
seed 9 的终局由 Traitor 变为 Rebels。本批用与测试完全相同的驱动器扫描
seed 9–120，**seed 28 在 212 条命令内达内奸终局**（主公死亡、内奸唯一存活），
`identity_pure_command_traitor_victory` 锚定 seed 28 并恢复 `Winner::Traitor`
强断言。原 seed 9 断言（TRAITOR）与 Rebel 结果均为合法终局，差异来源已入档。

## 2. N12 旧线批次第一步：依赖安装与构建打通

- 工具链：pnpm 10.34.3（node v26.4.0）；
- `pnpm install`：完成（node_modules 199 个包）；
- `pnpm build`：**成功**——产出完整 `dist`（card/character/game/mode/... 全量
  游戏资源与 `game/game.js`/`game/entry.js` 入口）；
- 构建日志存档：`pnpm_install.log` / `pnpm_build.log`（evidence 目录）。

### 无头驱动器的下一步（专门批次）

旧线游戏核心为 DOM 耦合的非名客户端（`game/game.js` 主循环 + UI 层）。
等价负载无头驱动器需要：

1. DOM/UI shim（或 jsdom 环境）以在 Node 中加载 `game/game.js`；
2. 驱动器以「出杀攻下家/推进阶段/窗口按策略应答」脚本调用旧线出牌接口；
3. 聚合统计（命令数/事件数/守恒/终局）按 n09 口径输出后跨线对照。

## 影响

- 测试 173 项不变；规则版本不变（13）。
- N12：安装与构建已打通，无头驱动器具备开工条件（入口已映射）。
- TRAITOR 锚点恢复强断言（seed 28）。
