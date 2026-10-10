# ADR 0201：主动技目标的固定手牌上限约束

## 状态

已接受。

## 背景

SIXIANG「助国」要求主动技只能选择当前手牌少于两张的存活角色，再令其补至两张。旧 JavaScript
在 `filter` 与 `filterTarget` 中重复执行 `countCards("h") < 2`。运行时不能执行这些回调，
而现有主动目标合同只支持与技能拥有者比较手牌，不能表达固定阈值。

若把目标选择封装成「助国」专用 Host 效果，会绕过既有 `ActivateSkill` 目标合同，也不利于后续
大量固定手牌阈值技能复用。

## 决策

ABI 128 为 `ActiveTargetsAst`、`ActiveTargetsIr` 和 `PackageTargetSpec` 追加可选字段
`hand_lt`，DSL 写作 `targets ... hand_lt N`。该约束表示每名选中目标的当前手牌数必须严格小于
无符号字面量 `N`。

服务端在 `ActivateSkill` 提交路径与其他目标约束一起验证该条件；不合法或陈旧目标会原子拒绝。
省略字段的旧包按 `None` 读取，保持向后兼容。

「助国」声明为 `targets 1 1 hand_lt 2`，复用现有 `hand_count` 查询和 `draw_cards` 效果计算
`2 - hand_count(event_source)`，不新增 Host 效果或续体。

## 后果

- 固定手牌阈值成为可组合、可检查、可序列化的通用主动目标约束。
- 目标可以包括技能拥有者，精确保留旧 `filterTarget` 语义。
- 旧 JavaScript 只作语义证据，不进入 npack 或运行时。
