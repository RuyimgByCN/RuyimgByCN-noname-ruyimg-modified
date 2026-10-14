# T8 增量 17：并列最大查询族 + targeted_by_other_sha（2026-10-14）

## 任务来源

[T8 余项](../../../architecture/native-progress.md)「具名组合 filter 深化」第五梯队，
配同日**全词表分类扫描**（642 名：18 已承载 + 624 待办——待办细分：复合查询
461【内含大量特定技能跟踪面】/once 实例旗标 52/移动批次上下文 51/pot 族 24/
丢失历史 16/技能牌堆 9/标记 9/转化杀溯源 1/实例记忆 1）：

| DSL filter | 权威语义（lsp） | 承载 |
|---|---|---|
| `owner_is_target_tied_max_hand`（ABI 55） | 事件主=持有者，手牌数与存活玩家最大值并列 | 存活集最大值比较 |
| `owner_is_target_tied_max_hp`（ABI 55） | 同上，当前体力 | 同上 |
| `owner_is_target_tied_max_equip_nonempty`（ABI 55） | 同上，装备数最大并列且非空 | 同上 |
| `owner_is_target_not_unique_max_current_hp` | owner 非严格体力最高（存在并列或更高者） | 存在性比较 |
| `owner_targeted_by_other_sha` | **持有者被另一玩家的杀指定**（Card-target 面） | 原生无独立 targeted 帧型——以 CardUsed+targets 表达（`VmTriggerCtx.targets` 新增捕获），登记差异 |

装载器：事件别名 `card_targeted → EventKind::CardUsed`（登记差异）；filter
映射 +5（锁面 19 名）。`VmTriggerCtx` 弃 Copy 保 Clone（targets 堆面）。

## 定向回归（t8_rvm_bridge +2 测，28/28；反例实证）

- `t8_filter_tied_max_family`：4 策略正/负例（并列触发、他人更高/严格最高
  负例、空装备负例）。断言经 `trigger_draws_of`（日志摸牌计数）——布阵账
  会被 np_settled 一步内的 Draw 摸牌与后续 phase_end 帧污染（实测两轮修正：
  断言通道换日志、负例布阵幅度加大到恒不并列）。
- `t8_filter_targeted_by_other_sha`：p2 杀 p1 触发 / 杀 p0 不触发 / p1 自己
  出杀不触发（使用者本人排除）。
- **反例实证**：TiedMaxHand 与 TargetedByOtherSha 两臂桩化恒 false → 2 测
  FAILED（5 passed/2 failed）。

## 语义勘误（测试驱动）

`OwnerTargetedByOtherSha` 首版误加 `is_event_player` 门——lsp 权威语义的
owner 域是**目标集**（"the living owner is targeted by…"）而非事件主玩家；
修正为「使用者≠持有者 ∧ 事件卡杀族 ∧ targets 含持有者」。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | t8_rvm_bridge **28/28** |
| engine-native 全量 | **728/0/0**（14 golden 不变）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0（拦截测试闭包多余 `mut`，已修）——[日志](gate1b-all-targets.log) |

## 登记边界

- `card_targeted` 事件别名：原生以 CardUsed 帧表达 targeted 语义，无独立
  帧型（上游 cardTargeted 的逐目标独立触发时序差异登记）。
- 并列最大族在伤害/阶段等任意携带 event_player 的帧上评估（词表标注
  Phase trigger，装载器不限制事件-filter 配对——与既有严格面一致）。

## 过程记录（操作纪律重申）

反例实验第二次发生 `git checkout` 误回退未提交改动（增量 17 rvm_bridge 短暂
丢失，经 `git apply --include` 从预存 patch 恢复）——**未提交工作区禁用
checkout**，恢复一律走 patch/精确 python 反向替换。
