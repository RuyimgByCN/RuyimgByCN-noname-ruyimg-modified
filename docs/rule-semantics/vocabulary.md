# 核心规则词汇表（阶段 0 草案）

> 本文件随标准模式实现持续冻结。词汇以核心机制为准；标准包（M1/M1+）的
> 完整牌/技能语义见 `phase5-identity.md` 与 `docs/content-sdk/phase4-dsl-vm.md`。

## 实体

| 术语 | 含义 |
|---|---|
| Player / Seat | 座位上的玩家实体；稳定 `PlayerId` |
| Card | 牌实例；稳定 `CardId`，含定义、花色、点数、所在区域 |
| Zone | 有序区域：牌堆、弃牌堆、手牌、装备、判定、处理区、移出游戏 |
| Skill | 技能实例；稳定 `SkillId` |
| Mark | 标记计数（按定义 ID 聚合） |
| Ruleset | 内容包/规则集引用与逻辑哈希 |

## 回合

`Idle → Prepare → Judge → Draw → Play → Discard → End → (下一座位 Prepare)`

## 命令

`StartGame` · `PlayCard` · `ActivateSkill` · `ResolveChoice` · `EndPhase` · `Surrender` · `TimeoutAction` · `AdminAbortMatch`

## 原语 Effect

见架构计划 §10（五类规则语义对象）与 `docs/content-sdk/phase4-dsl-vm.md`；已实现摸牌、移动、伤害、回复、濒死、死亡、选择请求等核心路径。

## 选择屏障

`ChoiceRequest` 是规则状态机输入屏障，不是 UI Promise。选择前状态与 continuation 先提交；`ResolveChoice` 开启下一段事务。

## 故障

| 类型 | 行为 |
|---|---|
| CommandRejected | 不改状态 |
| ContentFault | 回滚当前事务段 |
| EngineFault | 回滚 + 诊断 + 终止对局 |
| BudgetExceeded | 作为确定性 ContentFault |
