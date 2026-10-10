# ADR 0049：存活受伤人数阈值查询值源

- 状态：已接受
- Engine semantics：33

## 背景

旧版「兴棹」伤害子技能在伤害结算前统计存活且已受伤的角色：人数为零或不少于四时伤害加一。把该条件写成技能名特判会破坏内容包与引擎的边界。

## 决策

- 新增查询期 modifier 值源 `alive_wounded_count_zero_or_at_least_four`；
- 每次伤害值查询时统计存活且 `hp < max_hp` 的角色，人数为零或至少四时返回一，否则返回零；
- 该值源只允许用于 `damage_amount add`，不扩大到无明确语义需求的 modifier 通道。

## 结果

`xinfu_xingzhao3` 可从旧 JS 精确重写为声明式技能，且后续同阈值技能能复用同一通用值源，无需 Rust 技能名分支。
