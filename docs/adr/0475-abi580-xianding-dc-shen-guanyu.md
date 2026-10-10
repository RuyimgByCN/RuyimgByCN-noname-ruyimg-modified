# 0475 ABI 580 XIANDING 新杀神关羽【武神/武魂】

日期：2026-08-03  
状态：已落地  
领取者：B（Codex overflow）

## 决策

旧 JavaScript 仅作为语义合同，不进入内容包或运行时。`dc_shen_guanyu` 在 XIANDING 发布为
5 体力、神势力、男性武将，技能闭包为【武神】`dcwushen(3292885385)` 与【武魂】
`dcwuhun(4234333270)`。

ABI 580 新增过滤器 `owner_source_heart_sha` 与效果
`record_current_sha_all_targets(player, card)`。【武神】通过既有 `card_name`、`card_nature`、
`card_distance_unlimited`、`card_use_unlimited` 修正器实现红桃牌自动视为无属性【杀】且不受
距离、次数限制；效果只把当前杀结算记录中的既有目标绑定到来源技能，后续复用
`owner_source_sha_damage_recorded_by_skill`，不增加专属卡牌状态。

【武魂】复用逐伤害点双端过滤器：拥有者为来源时给受伤者加梦魇，否则给伤害来源加梦魇；死亡
时复用既有“选择梦魇最多者并判定，非桃/桃园则死亡”原语。

## 验收

- Compiler ABI 合同覆盖 DSL → IR → bytecode → VM typed effect。
- Golden 3/3，覆盖完整武将/技能发布、四项修正器、精确实体杀目标绑定与伤害摸牌、双向逐点
  梦魇及死亡判定效果；XIANDING 27 个测试文件共 94 项全绿。
- XIANDING 包更新为 ABI 580、5 名武将、64 项技能、58 个模块；源码产物与
  `extensions/xianding` 两份投放保持字节一致。逻辑哈希
  `0967d34f362a2b8b99369c37bc4d6ae42d0d744254a3745d59f354360ed31780`，文件 SHA-256
  `0eb4495d6e509f631604700b33d500243900dee879b2f3d94973a393ab902c5c`。
- engine-api 4/4、conformance 8/8 与 26 包冷导入通过；ContentLock 为
  `5383c11795ed6cc35d606a61e6afc60a8dfe0f8443a3d3dd2748904b90ec452f`。
- 编译产物按架构计划 §37.10 在批次结束后清理：外部 target 删除 22,943 个文件、11.8 GiB，
  工作区 `target/` 与外部 target 均确认不存在；`.npack` 与 `extensions/` 发布物保留且哈希不变。
