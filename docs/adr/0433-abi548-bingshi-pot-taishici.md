# ADR 0433：ABI 548 兵势太史慈闭环

本批迁移 `pot_taishici`（4160925913）及「酣战」`pothanzhan`（2573360121）、
「战烈」`potzhanlie`（3829642123）、「振锋」`potzhenfeng`（2197448518）。旧
JavaScript 仅作语义来源，发布物由 DSL、Rust 原语和 npack 构成。

## 决策

- ABI 548 新增 `resolve_pottaishici(player,target,action,hanzhan_def,zhanlie_def)`，
  以及战烈收集弃牌堆中实体杀、追踪本次杀结算后摸三的两个过滤器。
- 酣战按当前 X 将双方手牌补至上限三张，再以虚拟决斗指定目标；抽牌与决斗通过
  effect 队列串联，抽牌触发选择不会截断后续。
- 战烈每回合按 X 记录前 X 张进入弃牌堆的杀并累计至六枚「烈」。出牌阶段结束可
  清空全部「烈」使用不计次数的虚拟杀，每两枚选择一项：额外目标、伤害 +1、目标
  弃牌后方可响应、结算后摸三。响应代价位于 CardUsed/目标触发之后、实际响应之前，
  并复用既有杀续体保证嵌套触发可恢复。
- 振锋限定技可回复两点体力，或分别将酣战/战烈的 X 改为当前体力、已损失体力、
  存活角色数；两个技能的来源独立持久化。

## 发布与验证

- `official/bingshi@0.1.0`：ABI 548，26 名武将、62 项技能、1 张卡；logic hash
  `c67822f41fc253153284f2b62bf1652d1c702536ad5cb2b95e02e75d0ef62e41`，文件
  SHA-256 `334cd665a58e7f621786d0f727e6339bc01775dd011ade5c6bfbf4453ffae16e`。
- 三份发布包逐字节一致，独立重打包一致；26 个扩展冷启动导入成功，ContentLock
  `7fd058c69af1b357b166b8e0a8f10a80753e2ed981c7534faaffbe1ce0e42411`。
- 势太史慈 4 个 Golden、全部 25 个 Bingshi 测试文件、content-compiler 479 项测试、
  rule-api/rule-vm/content-schema/content-runtime、conformance 与 engine-ai 编译通过。
- 按架构计划 §37.10 收尾删除本批外部 `CARGO_TARGET_DIR`；工作区 `target/` 不保留。
