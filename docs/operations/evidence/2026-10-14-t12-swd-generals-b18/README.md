# T12 swd 武将面批 18：铁壁 + 大同（韩罗/习嫣双整将转正，2026-10-14）

## 承载

| 技能 | 上游 | 原生承载 |
|---|---|---|
| 铁壁 tiebi（韩罗 qun 5 唯一技→整将，global shaBefore direct） | 其他角色成为杀目标且与持有者距离 ≤1 时弃 1 张黑牌给目标 +1 护甲 | `SwdTiebi`（tag 264）：CardUsed 帧等价承载（sha 目标距离 ≤1/无护甲/持有者有黑牌）+ 弃首张黑牌（登记差异：上游 chooseToDiscard 玩家选）→ `change_hujia(t, 1)` |
| 大同 datong（习嫣，global phaseEnd frequent） | 任意角色阶段结束全场手牌极差 ≤1 则自己摸 2 | `SwdDatong`（tag 265）：Phase{begin:false} 任意角色任意阶段结束 + 极差计算 filter + 摸 2 |

**语义勘正**（回归驱动）：原生护甲=受伤时抵 1 伤——铁壁 +1 护甲在本次杀伤害
时消耗（hp 不减、护甲清零），断言按消耗语义修正。

## 实施要点

- 铁壁为 **CardUsed 帧等价承载**（上游 target shaBefore 无独立事件种——
  复仇同路径），filter 含目标活/距/护甲面，提取臂取首个非持有者目标
  （machine() 无 ctx——活/距校验在 filter）。
- **接线清单第 10 触点再证**：filter 命中但 start() 无臂 → `_ => None`
  吞机（效果静默失效）——探针（plan entry→step→black）三段定位后补
  start 臂修复。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | g1_shahai_swd **29/29**（铁壁护甲抵伤语义/大同极差摸 2/目录面） |
| engine-native 全量 | **747/0/0**（golden 不变——CardUsed/PhaseEnd 触发面标准场景无 swd 持有者）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**2/79 名、2/241 技转正**（韩罗整将、习嫣 1/2 部分转正——
  jiefen 多步交牌机待后续批）。
- 铁壁登记差异：上游玩家自选弃牌、原生首张黑牌；上游 direct 可取消，
  原生自动（有黑牌必弃）。
