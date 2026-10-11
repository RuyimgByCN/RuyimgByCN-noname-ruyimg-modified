# 2026-10-11 T11 朱厌批末片：巨源（规则 v226，存档 v51）——朱厌四技全齐

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `03e6fce4`；朱厌批末片整技交付）。上游依据：`mode/boss.js` `boss_juyuan`（5836 区：init storage=0 + phaseAfter `storage=hp` + mod selectTarget sha `hp>=storage` return 否则 range[1]++）与 `boss_zhuyan`（890：hp25/maxHp30）。

## 建模

- **巨源记录存储**：`PlayerState.juyuan_storage: i32` 新字段（跨回合持久，shedu 先例同款）——**存档格式 50→51**（PlayerState 尾 i32；canonical_hash 同步入参）。
- **巨源**两面（权限型 Noop+查询）：
  1. **回合结束体力记录**（kernel `Phase::End` begin 臂：`juyuan_storage=hp`——上游 phaseAfter 同口径）；
  2. **杀 selectTarget mod**（engine `max_targets` 计算：`飞刃同款 +1`——持技 ∧ 记录>0 ∧ hp<记录 → +1 目标）。
- **体力上限+5**：由 boss_zhuyan `maxHp30` 表意（登记差异同穷奇族：max_hp 暂=hp=25）。
- FROZEN 尾加「巨源」（82 名）。

## 验证（Prototype 局——boss_stage 局 first=0 与虚失翻面臂的自动推进交互独立登记）

| 项 | 结果 |
|---|---|
| 回归 `boss_zhuyan_juyuan_storage_and_extra_target` | 25 血装配+初始未记录 → 回合结束记录 25 → 体力 20<记录 → 杀双目标 Accepted → 记录随回合结束刷新=当前体力 → v51 存档往返一致 |
| 既有回归 | 虚失/兵先/相柳/梼杌/书精/四凶/关卡全部不变；native_kernel 228/0 |
| 版本回退探针（226→225 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **573 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 225→**226**；存档 50→**51** |

## 登记遗留

- boss_stage 局（first=0）在朱厌虚失已激活时的 advance 自动推进出现 0 号回合循环（虚失翻面臂交互）—— Prototype 局验证无碍，boss 局循环机制**独立登记待查**。
- 朱厌「体力上限+5」以 maxHp30 表意（max_hp 暂=hp=25，同穷奇 maxHp25 差异族）。
