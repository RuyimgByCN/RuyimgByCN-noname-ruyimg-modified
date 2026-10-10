# ADR 0037：主动技“其他目标”与“同色选牌”契约

- 状态：已接受
- Engine semantics：21

## 背景

旧 JS 技能常用 `filterTarget: notMe` 排除自己，并在 `filterCard` 中约束多张费用牌的共同属性。若这些条件只存在于界面筛选，网络命令仍可提交自身目标或不符合条件的卡牌。

始计「歃盟」要求每回合一次，弃置两张同色手牌并选择一名其他角色，然后目标摸两张牌、自己摸三张牌。

## 决策

扩展已有主动技契约：

```text
active play_phase uses_per_turn 1 targets 1 1 other select_hand 2 2 same_color {
  ...
}
```

- `other` 将 `PackageTargetSpec.allow_self` 设为否；引擎拒绝技能拥有者出现在目标列表中。
- `same_color` 将 `PackageSelectionSpec.same_color` 设为是；引擎按技能拥有者看到的有效花色计算红黑颜色，并拒绝混色输入。
- 两项限制均进入包逻辑哈希；未声明时保持既有行为。
- 歃盟的弃牌与摸牌仍复用通用 typed effects，不新增技能专用 effect。

## 结果

客户端可读取同一份 npack 元数据构造选择界面，服务器使用同一契约做最终校验。旧扩展缺少新增字段时分别默认允许自身目标、不要求同色，保持向后兼容。
