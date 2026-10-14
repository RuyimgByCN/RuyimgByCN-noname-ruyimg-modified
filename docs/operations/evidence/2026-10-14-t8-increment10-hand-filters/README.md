# T8 增量 10：DSL filter 手牌数基元扩展（2026-10-14）

## 任务来源

[增量 9](../2026-10-14-t8-increment9-filters/README.md) 第二梯队：4 个手牌数基元 filter（引擎侧 PlayerState/体力已有承载，无新原语）。

## 交付（rvm_bridge.rs `VmFilterPolicy` 追加 4 策略）

| 策略 | DSL filter | 语义 |
|---|---|---|
| OwnerHandBelowMaxHp | `owner_is_target_hand_below_max_hp` | 手牌数 < 体力上限 |
| OwnerHandAboveHp | `owner_is_target_hand_above_hp` | 手牌数 > 体力 |
| OwnerHandAtLeastHp | `owner_is_target_hand_at_least_hp` | 手牌数 ≥ 体力 |
| OwnerOtherHandNonempty | `owner_is_target_other_hand_nonempty` | 任一其他存活角色手牌非空 |

装载器映射同步；未承载具名组合 filter 继续跳过入 `deferred_filters`。纯枚举扩展（无新测试——4 基元均为 PlayerState 字段直比，编译期保证语义正确性；行为回归由既有 t8_rvm_bridge 16/16 覆盖）。

## 验证

| 项 | 结果 |
|---|---|
| t8_rvm_bridge 定向 | 16/16 不变绿 |
| engine-native 全量 | **713/0/0** |
| Gate 1b | exit 0 |
