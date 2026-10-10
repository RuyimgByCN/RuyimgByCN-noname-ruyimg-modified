# ADR 0351：ABI 480 宽释二伤回复复合 + huicui 阚泽

本批为 huicui 包阚泽（`re_kanze`，3301308945）的「宽释」（`rekuanshi`，
3423208328）完整移植，需要 ABI 480 双阶段复合。

## 原语设计（ABI 480）

- 第一阶段 `resolve_rekuanshi(player)`：`trigger phase_end owner_is_target` → 选目标
  （可取消）→ `AddMark(目标, REKUANSHI_WATCH_MARK, 1)`；同时清零拥有者的每回合
  一次标记（REKUANSHI_USED_MARK）。
- 第二阶段 `resolve_rekuanshi_resolve(player, damaged)`：`trigger damage
  owner_alive_not_target`（vm_host 捕获 `event_player`）→ 目标带 WATCH 标记且
  拥有者未用过 → `turn.damage_taken_count(damaged) == 2` 且受伤 → 回复 1 点体力
  （`RecoverHp`）并置 USED 标记（每回合一次）。

## 打包与验证

- `rekuanshi.skill` 加入 huicui README 显式列表；`re_kanze`（男/吴/3 体力，技能
  [rekuanshi]）加入 huicui `characters.json`；catalog 标记 `ported`。
- huicui npack 重建（ABI 480，28 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/huicui/`。
- 运行时测试 `original_huicui_rekuanshi.rs`（2 用例全绿）：标记后第 2 次受伤回复
  1 点（第 1 次不回复）；连受 4 次伤害仅回复一次。
- 旧 JavaScript 不进入 npack 或运行时。
