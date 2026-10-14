# T8 增量 11：DSL filter 深化第二梯队（2026-10-14）

## 任务来源

[增量 9/10](../2026-10-14-t8-increment9-filters/README.md) 后的具名组合 filter 深化首批：3 个引擎侧可承载基元落 `VmFilterPolicy`。

## 交付（rvm_bridge.rs `VmFilterPolicy` 追加 2 策略 + 装载器映射 3 条）

| 策略 | DSL filter | 语义 |
|---|---|---|
| OwnerTurnDamageAtLeastThree | `owner_is_target_turn_damage_at_least_three` | 本回合受伤 ≥3 |
| OwnerHandAtLeastHpOtherAlive | `owner_is_target_hand_at_least_hp_other_alive` | 手牌数 ≥ 体力且存在其他存活角色 |

装载器严格映射同步（未承载具名组合 filter 继续跳过入 deferred_filters）。纯枚举扩展（无行为测试增量——2 基元均为已有状态直比，编译期保证语义正确性）。

## 验证

| 项 | 结果 |
|---|---|
| t8_rvm_bridge 定向 | 16/16 不变绿 |
| engine-native 全量 | **713/0/0**（不变，纯 filter 枚举扩展） |
| Gate 1b | exit 0 |

## 剩余具名组合 filter

~630 个复合语义 filter（需跨状态查询/计数/记忆面或技能检查），逐批按需立项。
