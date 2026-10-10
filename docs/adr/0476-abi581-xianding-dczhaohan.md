# 0476 ABI 581 XIANDING 杨彪【昭汉】

日期：2026-08-03  
状态：已落地  
领取者：B（Codex overflow）

## 决策

旧 JavaScript 仅作为语义合同，不进入内容包或运行时。【昭汉】在未固定摸牌数的摸牌阶段可令
本次权威摸牌数增加 2；接受后添加一次性辅助技能 `dczhaohan_choose(3308549699)`。摸牌阶段
结束时，若拥有者至少有两张手牌，强制选择：将两张手牌交给一名当前没有手牌的存活角色，或
弃置两张手牌；没有合法赠牌目标时直接进入弃牌选择。辅助技能在本回合结束时移除。

ABI 581 新增通用效果
`choose_give_or_discard_hand(player, payment_count, hand_empty)`。实现复用 ABI 116 的交牌/弃牌
选择续体和既有 `hand_empty` 目标范围，在分支、目标及实体牌选择屏障重新验证技能、存活状态、
目标手牌数和卡牌所有权；不增加【昭汉】专属引擎状态。

## 验收

- Compiler 合同覆盖 DSL → IR → bytecode → VM typed effect，content-compiler 490/490、
  engine-api 4/4、rule-vm 15/15 通过。
- 【昭汉】Golden 3 项覆盖摸牌数 +2、交两张给无手牌角色、无合法目标时弃两张；XIANDING
  27 个测试文件共 96 项全绿，conformance 8/8 通过。
- XIANDING 包更新为 ABI 581、5 名武将、65 项技能、60 个模块；逻辑哈希
  `2239f6af9e977b2f6fcd9f3963574c95d2922c6ca932d74c424068d9106da1ec`，文件 SHA-256
  `fe1ce4ace299d6512f248a0e08d808f249740ea3860a2e36ac2a5563df651516`，源码产物与
  `extensions/xianding` 两份投放字节一致。
- 26 个扩展从空 store 冷导入成功，ContentLock 为
  `c7c70cc9b803d0ac1e57fa61ca999c13e937fc08861f695609973a11bb6385ad`。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 发布物不在清理范围。
