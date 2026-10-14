# T8 增量 12：装载器映射补齐——4 个手牌数基元 filter（2026-10-14）

## 任务来源

[增量 10](../2026-10-14-t8-increment10-hand-filters/README.md) 落地了 4 个手牌数基元策略（`VmFilterPolicy` 枚举+行为实现），但装载器 `bridge_vm_skill_decls` 的 filter 严格映射尚未覆盖——本批补齐。

## 交付（content-native-loader/src/lib.rs）

`bridge_vm_skill_decls` filter 映射追加 3 条（第 4 条 `owner_is_target_hand_at_least_hp_other_alive` 已在增量 11 补入）：

| DSL filter | VmFilterPolicy | 语义 |
|---|---|---|
| `owner_is_target_hand_below_max_hp` | OwnerHandBelowMaxHp | 手牌数 < 体力上限 |
| `owner_is_target_hand_above_hp` | OwnerHandAboveHp | 手牌数 > 体力 |
| `owner_is_target_other_hand_nonempty` | OwnerOtherHandNonempty | 任一其他存活角色手牌非空 |

（`owner_is_target_hand_at_least_hp` 由增量 11 的 `OwnerHandAtLeastHpOtherAlive` 承载同语义。）

## 验证

| 项 | 结果 |
|---|---|
| t8_rvm_bridge 定向 | **16/16** 不变绿——[日志](bridge-targeted.log) |
| engine-native 全量 | **715/0/0**（不变，纯映射补齐）——[日志](engine-native-tests.log) |
| content-native-loader | **0/0**——[日志](loader-tests.log) |
| Gate 1b | exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |
