# ADR 0531：欢乐卡牌归档扩展迁移（huanle）

- 领取：C 段 omp
- 来源：`archive/upstream/noname/apps/core/extension/欢乐卡牌`（GNU GPL v3）
- 交付：`extensions/huanle`（6 卡 / 5 技能）

## 内容

欢乐卡牌（huanle）是含自定义卡牌的扩展。5 张装备卡注册定义（monkey/jiuwei/
xuelunyang/mianju/kuwu），2 项真实技能 + 3 项元数据登记，依赖 standard 包。

## 已发布

- 卡牌：`monkey`（宝物）、`jiuwei`（宝物）、`xuelunyang`（宝物）、`mianju`
  （防具）、`kuwu`（武器，攻击范围 1、进攻距离 -1）
- `mianju`（面具）：锁定技不能被翻面（turn_over_cancel）
- `jiuwei`（九尾）：结束阶段回复 1 点体力或摸一张牌（旧 JS 按已受伤自动选回复，
  DSL 以自选近似）

## 元数据登记（无运行时行为）

- `monkey`（猴子偷桃）：取消他人【桃】并获牌——需整张牌使用取消原语
- `kuwu`（苦无）：【杀】伤害后目标弃牌——需伤害后目标弃牌原语
- `xuelunyang`（血轮眼）：复制其他角色一项技能——需动态复制技能机制

## 未迁移

`shoulijian`（手里剑）：自定义基本牌，引擎卡牌种类仅支持装备类，不可迁移；
效果保留于译文。自定义装备卡不进默认牌堆（与 wandianlunsha 派生卡处理一致）。

## 验收

content-compiler 514/514、conformance 8/8、30 包空 store 冷导入通过。
