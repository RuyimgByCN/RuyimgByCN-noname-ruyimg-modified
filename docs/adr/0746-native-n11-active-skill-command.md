# ADR 0746：N11 主动技能命令面——GameAction::ActivateSkill 与黄盖苦肉

日期：2026-09-16
状态：已实施（engine-native）
关联：[0736](0736-native-simayi-guicai-fankui.md)、[0740](0740-native-wusheng-longdan-viewas.md)、进度文档 N11/N08

## 背景与决定

此前技能只能随事件被动触发——主动技（制衡/苦肉/奇策类）没有命令面。本批新增
**`GameAction::ActivateSkill { skill: SkillDefId }`** 平台命令面，并以首个主动技
**黄盖苦肉**（出牌阶段：失去 1 点体力，摸两张牌，每回合限一次）交付端到端验收。

### 实现

- `SkillBehavior` trait 增加默认 `activate(ctx, owner) -> Option<Machine>`——
  主动技形态（默认 None，既有行为实现零改动）；
- `apply_command` 新增 `ActivateSkill` 臂，信任边界校验链：
  ①非当前回合玩家 → `NotYourTurn`；②技能未授予 → `NoSuchSkill`；
  ③已禁用 → `SkillDisabled`；④本回合已发动（once_per_turn）→
  `SkillAlreadyUsed`；⑤行为拒绝激活 → `InvalidChoice`；
- `once_used` 经既有每回合重置机制自动复位（turn_opened）；
- 新拒绝原因 `NoSuchSkill`/`SkillDisabled`/`SkillAlreadyUsed`
  （信任边界扩展，runner 聚合同步）。

### 苦肉机器（KuRouMachine）

两阶段：Lose（失去 1 点体力，含濒死内联检查——体力 ≤0 进入濒死流程）
→ Draw（摸两张）→ Done。快照变体 `MachineSnapshot::KuRou { owner, stage }`
（六处同步：枚举/快照/校验/哈希/存档编码/解码）。

### 验收（2 项）

- `huanggai_kurou_lose_hp_draw_two`：发动后体力 -1、手牌 +2；同回合重复发动
  → `SkillAlreadyUsed`；守恒；
- `activate_skill_without_grant_rejected`：未授予技能发动 → `NoSuchSkill`。

## 影响

- 测试 178 项（native_kernel 101），净增 2。规则版本 14 → 15：
  Duel.second_pending（同批无双决斗补全）与 KuRou 机器变体一并入状态
  编码与哈希，14 golden 重录并通过。
- N11 主动技能命令面就位：制衡/奇策等后续主动技按模板扩展。
