# ADR 0026: 伤害来源属性筛选与唯一最多手牌

- 状态：接受
- 日期：2026-07-14

## 决策

`damage_amount` 接受 `normal`、`fire`、`thunder` 属性选择器，按伤害进入来源侧规则时的原始属性筛选；`damage_taken_amount` 继续按目标侧已经解析的属性筛选。

增加通用动态值 `owner_unique_max_hand`：仅当技能拥有者存活，且其手牌数严格大于其他所有存活角色时为 1，否则为 0。SP 包的 `jiaozi` 在来源侧与目标侧复用该值；OFFLINE 包的 `hm_yingzhan` 组合火、雷两类属性选择器，不增加技能专用分支。

无来源伤害不应用任何来源侧 modifier。旧 JS 仅用于语义对照，不进入运行时。

## 兼容性

新 `ModifierTarget` 与 `ModifierValueSource` 变体追加在枚举末尾，不改变旧 npack 的序列化变体编号。规则可表达范围和无来源伤害行为发生变化，`EngineSemanticsId` 从修订 9 升至 10；Native 与浏览器 WASM 哈希必须重新一致通过。
