# ADR 0754：N11 大模式实现第一批——KOF M1 与国战 M1 数据模型

日期：2026-09-16
状态：已实施（engine-native）
关联：[0753](0753-native-n11-guozhan-kof-mode-design.md)、进度文档 N11

## 背景与决定

按 [0753](0753-native-n11-guozhan-kof-mode-design.md) 里程碑推进 N11 大模式实现。

### 1. KOF M1：2 人局纯命令终局回归（`kof_m1_two_player_pure_command_endgame`）

KOF 会话层的单局基座：Prototype 2 人局纯命令（出杀攻下家/窗口策略应答/推进）
驱动至终局，断言——胜者为存活玩家（`Winner::Player`）、恰一名存活、
全量守恒。零调试击杀。

### 2. 国战 M1：双将数据模型（`guozhan_general_slots_roundtrip`）

`PlayerState` 扩展国战双将字段：

- `general_slots: [Option<u16>; 2]`——双将槽（STANDARD_GENERALS 下标，
  None = 未设置）；
- `revealed_generals: [bool; 2]`——主/副将明置状态。

两字段入状态哈希（防篡改）与 NGSA 存档（opt_u16 编码，确定性遍历）。
验收：设置双将/明置后 save→restore 往返哈希一致；篡改明置或槽位 →
哈希变化（`general_slots`/`revealed_generals` 均参与摘要）。

### 3. 版本 20

PlayerState 编码变化 → `STANDARD_RULESET_VERSION` 19 → 20，
14 golden 重录并通过；runner 复跑退出 0。

## 影响

- 测试 190 项（native_kernel 98+新增 2=100 中的合计），全会话合计 190 项
  （native_kernel 98、boundary_snapshot 59、view_isolation 7、auth_archive 6、
  replay_scenarios 2、源码 2、dbg4 1——以实际输出为准）。
- N11 大模式：KOF M1 与国战 M1 数据模型闭合；
  N11 后续：KOF M2 会话层（轮换/胜负序列）、国战 M2（势力与明置规则）。
