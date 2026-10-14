# T8 增量 15：参数化/计数族 filter 四策略（2026-10-14）

## 任务来源

[T8 余项](../../../architecture/native-progress.md)「具名组合 filter 深化」第三梯队：
词表**参数化族**与计数具名中有 4 个可由现役状态承载（无新字段、无存档/哈希变化、
无 golden 重录）：

| DSL filter | 权威语义（lsp） | 承载 |
|---|---|---|
| `owner_is_target_hp_below_{N}` | 事件主玩家=持有者且当前体力 < N | `PlayerState.hp`（事件时点口径：damage 帧 commit 扣血后评估） |
| `owner_source_play_card_count_eq_{N}` | 主动使用本出牌阶段第 N 张牌时（响应不计、Play 开始清零） | `cards_used_play_phase`==N（filter 于帧 commit 后评估，计数=本次用牌后序数） |
| `owner_source_play_card_count_eq_hp` | 同上，N=当前体力 | 同上==hp |
| `owner_is_target_card_used_at_least_hp` | 本回合主动用牌 ≥ 当前体力 | 同上≥hp（本引擎主动用牌仅发生在出牌阶段且 Play begin 清零=回合级化归，同增量 14 口径登记） |

**跳过登记**：`owner_is_target_active_used_at_least_{N}_this_play_phase`
（语义=主动技实例发动计数，需 per-instance 状态原语，归「剩余效果查询变体」）、
`owner_source_play_skill_mark_mode_{N}`/`*_mark_positive_*`（标记面）、
`*_skill_pile_*`（技能牌堆面）。

## 交付

- **`VmFilterPolicy` 扩至 18 策略**：`OwnerHpBelow(u32)`/`OwnerPlayCountEq(u32)`/
  `OwnerPlayCountEqHp`/`OwnerCardUsedAtLeastHp`（首现带载荷变体；Copy/Eq 保持）。
- **装载器参数化解析**：精确名 match 后前缀解析（`eq_hp` 先于 `eq_` 前缀防吞）；
  解析失败按未承载落 `deferred_filters`（防御性，词表本保证后缀合法）。
- 无存档/规则版本变化（策略与声明不进 NGSA 字节，目录随扩展装载重建）。

## 实测记录（反例实证先行）

| 项 | 结果 |
|---|---|
| 反例 1 | 4 行为臂桩化恒 false → `t8_filter_param_hp_below…`/`t8_filter_param_play…` **2 测 FAILED** |
| 反例 2 | 装载器参数化分支桩化 → 参数名落 deferred（`t8_loader_maps_parameterized` 变体/deferred 断言必失败；实施中断未单独跑绿——装载器映射缺失由定向 24/24 复跑兜底） |
| 定向 | t8_rvm_bridge **24/24**（+3 测：hp_below 双闸+阈值边；play_count/eq_hp/at_least_hp 出牌序数；装载器 4 名参数化锁面） |
| engine-native 全量 | **724/0/0**（14 golden 不变——无哈希面变化）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 语义口径实测确认（回归驱动出的两处）

- **damage 帧的 `current HP` 是事件时点（扣血后）**：`hp_below` 在帧 commit 后
  评估 filter（Normal after 窗），阈值边断言按扣血后布置。
- **card_used 帧的计数是 commit 后序数**：第 N 张牌使用时 `cards_used_play_phase`
  已 =N；响应打牌走 CardResponded 不经出牌提交=「Responses do not count」同口径。

## 登记边界

- `card_used_at_least_hp` 的「本回合」以出牌阶段计数化归（机器直发虚拟牌经
  `commit_card_used` 亦计数；响应不计）；上游 countUsed 的逐项口径未对照，
  登记差异同增量 14。
- 装备牌不计数（Equipped 帧）同增量 14。
