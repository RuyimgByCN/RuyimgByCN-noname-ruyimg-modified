# 0473 ABI 578 XIANDING DC 糜夫人【闺秀/存嗣】

日期：2026-08-03  
状态：已落地  
领取者：B（Codex overflow）

## 决策

旧 JavaScript 仅作为语义合同，不进入内容包或运行时。`dc_mifuren` 在 XIANDING 发布为
3 体力、蜀势力、女性武将，技能闭包为【闺秀】`dcguixiu(2370422642)` 与【存嗣】
`dccunsi(4233990625)`。

ABI 578 新增复合效果
`resolve_dccunsi(player, target, guixiu_def, yongjue_def, stage)`。公开 DSL 只能提交
stage 0：校验技能与双方存活，向目标授予【勇决】；目标不是自己时摸两张牌，并将 stage 1
放入统一 deferred-tail 队列。授技、摸牌及其嵌套触发全部结束后，stage 1 才检查拥有者是否
存活、受伤且仍拥有有效【闺秀】，满足时回复 1 点体力。这保留了旧规则 `useSkillAfter` 的
时序，同时复用既有授技、摸牌、回复和延迟尾机制。

## 验收

- Compiler ABI 合同覆盖 DSL → IR → bytecode → VM typed effect。
- Golden 覆盖精确武将/模块发布、自选时授技且不摸牌、选择他人时摸二及嵌套摸牌触发先于
  【闺秀】回复、失去【闺秀】时不回复，4/4；XIANDING 24 个测试文件共 91 项全绿。
- 修正历史【节行】测试夹具：失去体力分支原先从 1 体力继续扣减而误入濒死清手牌，改为从
  `max_hp - 1` 开始，测试重新只验证非致命失去体力后的摸牌合同，不改变运行时规则。
- XIANDING 包：ABI 578，4 名武将、64 项技能、55 个模块；逻辑哈希
  `b5eb5d012bbd5988676cabc8ee340c2a8de9d41504c3e2346ceb143451607aa6`，文件 SHA-256
  `98dcbe08e9274aca8be8a050a32534fc3fe73783fb87c7d78f03f1605d2172ea`。源码包与
  `extensions/xianding` 两份投放字节一致。
- engine-api 4/4、conformance 8/8 与 26 包冷导入通过。
- 编译产物按架构计划 §37.10 在批次结束后清理：外部 target 删除 16,834 个文件、约
  8.7 GiB，工作区 `target/` 与外部 target 均确认不存在；`.npack` 与 `extensions/` 保留。
