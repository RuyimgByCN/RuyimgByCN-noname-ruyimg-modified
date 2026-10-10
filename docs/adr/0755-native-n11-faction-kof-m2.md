# ADR 0755：N11 国战 M2 势力模型与 KOF M2 会话层

日期：2026-09-16
状态：已实施（engine-native）
关联：[0753](0753-native-n11-guozhan-kof-mode-design.md)、[0754](0754-native-kof-m1-guozhan-data-model.md)、进度文档 N11

## 背景与决定

按 [0753](0753-native-n11-guozhan-kof-mode-design.md) 里程碑继续：国战 M2
（势力模型）与 KOF M2（会话层多局轮换/胜负序列）。

### 1. 国战 M2：势力模型（`Faction` 枚举 + 武将→势力映射）

- `catalog.rs` 新增 `Faction` 枚举（魏/蜀/吴/群）；
- `STANDARD_GENERALS` 扩展为三元组 `(武将名, Faction, 技能集)`——全部
  二十一名武将（含机制样例=群）归属完成，消费方适配（`debug_grant_general`
  与 general_table 测试的元组解构）；
- 验收 `general_faction_mapping_complete`：势力合法性（四选一）、武将名
  唯一、每将至少一个技能且技能均在目录、官方势力抽查（曹操魏/关羽蜀/
  孙权吴/吕布群）。

### 2. KOF M2：会话层驱动器（`examples/n11_kof.rs`）

KOF = 「多局 + 淘汰」的宿主侧会话状态机——单局用 Prototype 2 人局表达
（内核不感知轮换）。驱动器连赛 N 局（种子逐局递增），登记胜负序列与比分：

- 单局：纯命令驱动（出杀攻下家/窗口策略应答/推进阶段）至终局；
- 会话：逐局登记 `round/winner=seatN` 胜负序列，汇总比分（match_score）；
- 8 局 seed 8000 实跑：胜负序列 3-5（seat1 领先），全部单局收敛。

### 3. 明置门控（国战 M3 前置说明）

`revealed_generals` 已入哈希与存档（M1）；技能明置门控（未明置武将的技能
不可发动/触发）需技能→武将归属映射，列为国战 M3 前置。

## 影响

- 测试 191 项（native_kernel 105），净增 1。规则版本不变（20）。
- N11：国战 M2 势力模型、KOF M2 会话层交付；
  N11 剩余：国战 M3 势力胜负 + 回归矩阵、明置门控、KOF 淘汰规则细化。
