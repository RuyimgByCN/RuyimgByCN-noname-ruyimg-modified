# T8 增量 13：VmFilterPolicy 家族语义收敛（2026-10-14）

## 任务来源

增量 9–11 落地的 12 个 `owner_is_target_*` 派生 filter 策略的**定向复核**：
lsp 权威描述（`content-compiler/src/lsp.rs`，如 `owner_is_target_hand_at_least_hp_other_alive` =
"the living skill owner is the event player, has at least as many hand cards as current HP,
and another living player exists"）表明家族语义是**「事件主玩家=持有者」∧ 状态谓词**的合取；
而实现只承载了谓词半边。filter 是触发调度里唯一的持有者域闸
（`resolution.rs` `plan_triggers` 只按 EventKind 找候选），漏合取段会让**他人事件过闸**
（炙焚 holder 门同类缺陷，v262 批曾由「他人准备阶段不触发」回归抓出同类问题）。

## 缺陷清单（本次全部修复）

| # | 缺陷 | 影响 |
|---|---|---|
| 1 | **家族性**：12 个派生策略（Wounded/Linked/HandNonempty/HandEmpty/NoTurnDamage/TurnDamage/HandBelowMaxHp/HandAboveHp/HandAtLeastHp/OtherHandNonempty/TurnDamageAtLeastThree/HandAtLeastHpOtherAlive）全部丢「事件主玩家=持有者」合取段 | 持有者谓词态满足时，任意他人的 damage/phase 事件都会触发其 VM 技能 |
| 2 | `OwnerHandAtLeastHpOtherAlive` 丢「手牌数≥体力」半边（实现只剩"存在其他存活角色"） | 谓词完全失效 |
| 3 | 增量 9 证据把 turn_damage 系描述为"受到伤害"；内核（kernel.rs 记来源侧）与 lsp 权威语义均为"**造成**"。实现正确，**证据描述笔误** | 文档失真，已补正注 |
| 4 | Gate 1b 拦截：上一批入库的诊断脚本 `t12_dual_diag.rs` 有未用导入 `PileSpec` | -D warnings 下编译失败 |

另核实：装载器映射的 `owner_is_target_hand_empty` 不在编译器 FILTERS 词表（编译产物不可能
携带该名）——装载器映射保留为前进兼容臂，策略行为由直注测试覆盖；词表内 10 个具名映射
以正向锁面测试固定。

## 修复

- **`rvm_bridge.rs`**：`SkillBehavior::filter` 引入 `is_event_player = trig.event_player == Some(owner)`
  合取段，前置到全部 12 个派生策略臂；`OwnerHandAtLeastHpOtherAlive` 补「手牌≥体力」半边；
  枚举文档逐项更正（含 turn_damage「造成」口径）。
- **定向回归**（`t8_rvm_bridge.rs` +2 测）：
  - `t8_filter_family_owner_gate_and_predicates`：12 策略逐策略双闸（他人受伤不过闸/本人
    受伤过闸）；turn_damage 系以「持有者对他人造成伤害」负例兼布置「造成」计数；
    `AtLeastHpOtherAlive` 另测「体力 6>手牌 4 谓词假不触发」反例（增量 11 丢半边的直接反例）。
  - `t8_loader_maps_all_carried_named_filters`：词表内 10 个具名 filter 编译→装载器逐名映射
    断言 + `deferred_filters` 为空。
- **`t12_dual_diag.rs`**：删未用导入。

## 反例实证（修复必须先被新测试抓住）

`git checkout` 暂回退 src 修复、仅保留新测试：`t8_filter_family_owner_gate_and_predicates`
**FAILED**（旧实现下他人事件过闸）；重新 apply 修复后转绿。测试对缺陷的判别力已实证。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | t8_rvm_bridge **18/18**（16 既有 + 2 新）——[日志](bridge-targeted.log) |
| engine-native 全量 | **718/0/0**，14 golden 联动守护（replay_scenarios）通过——哈希不变符合预期：VM filter 语义只作用于扩展装载技能，标准目录不经 `VmFilterPolicy`——[日志](engine-native-tests.log) |
| 装载/包管理 | content-runtime + content-native-loader **37/0** |
| Gate 1b | exit 0（独立 CARGO_TARGET_DIR，`-D warnings` --all-targets；拦截项 #4 已修）——[日志](gate1b-all-targets.log) |

## 登记边界

- 增量 9–11 的"定向 16/16"实为既有套件通过数：12 派生策略在当时**无任何直接测试**，
  本批家族回归为其首个真实覆盖——后续 filter 深化增量一律要求新策略携带双闸回归。
- `OwnerHandAtLeastHp` 策略变体为词表超集（词表无裸名），暂由直注测试覆盖；词表若收录
  裸名则装载器需补映射（现 loader 无该臂）。
- 余项不变：具名组合 filter 深化（~630 复合语义逐批）、剩余效果查询变体（需内核新原语）。
