# ADR 0216：主动技手牌上限前置条件与本回合调用次数

## 背景

四象「极奢」可在出牌阶段反复发动：每次摸一张牌并令本回合手牌上限减一，当前手牌上限降至
零后停止。旧 JavaScript 用临时子技能和标记保存次数；引擎已经为每个主动技能实例维护并按
回合清零 `use_count`，无需复制第二份状态。

## 决策

Rule VM ABI 143 增加两个声明式能力：

- active 合同 `hand_limit_positive`，在提交时通过统一 `HandLimit` 查询验证上限严格大于零；
- modifier 值源 `owner_skill_use_count`，读取来源主动技能实例本回合累计调用次数。

两者都复用现有状态和查询路径，不新增 HOST_EFFECT、continuation 或 `EffectRequest`。

## 结果

「极奢」只需一次摸牌效果和一条手牌上限 modifier；新回合自动恢复。Golden 覆盖连续三次发动
令上限从三降至零，以及第四次提交被原子拒绝。
