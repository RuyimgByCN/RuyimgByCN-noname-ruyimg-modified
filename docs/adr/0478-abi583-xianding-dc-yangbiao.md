# 0478 ABI 583 XIANDING 杨彪【举讹】

日期：2026-08-03  
状态：已落地  
领取者：B（Codex overflow）

## 决策

旧 JavaScript 只作语义合同，不进入内容包或运行时。`dcjue` 的官方译名按上游翻译表统一为
【举讹】：任意角色的结束阶段开始时，若本回合至少一张由玩家区域因“弃置”进入弃牌堆的实体牌
仍在弃牌堆，杨彪每轮可发动一次。若当前回合角色不是杨彪，则确认后固定以该角色为目标；若是
杨彪自己的结束阶段，则以一次可取消的目标选择指定一名其他存活角色。

X 在接受发动时取“上述仍在弃牌堆的牌数”与目标体力上限的较小值。之后最多循环 X 次；每次先
检查目标仍存活，再由权威 RuleRng 随机排列【杀】、【过河拆桥】、【五谷丰登】，选择其中第一个
对杨彪和目标至少有一个合法目标的牌，并只对两者中的合法目标走完整虚拟牌使用、响应、无懈和
伤害管线。目标离场立即停止。

ABI 583 新增 `resolve_dcjue(player, event_player, stage, remaining)`。引擎只在加载
`owner_alive_global_phase_end_discarded_cards_available` 时记录本回合弃置实体牌，未加载该能力的
旧包不增加回合状态或哈希字段。一次选择与循环续体复用现有 Custom Choice、虚拟牌目标查询、
效果调度和技能实例 `round_fire_round`，不建立新的技能专属存储体系。

## 验收

- Compiler 合同覆盖 DSL → IR → bytecode → VM typed effect；content-compiler 492/492、
  engine-api 4/4、rule-vm 15/15 通过。
- 【举讹】Golden 4 项覆盖完整杨彪武将闭包、只统计弃置原因且仍在弃牌堆的实体牌、其他角色结束
  阶段固定目标确认、自己结束阶段单次选择其他目标、权威三项随机与每轮状态。
- XIANDING 28 个测试文件共 102 项全绿；因新增武将改变 bootstrap 角色采样而暴露的【讽言】
  用例已固定其合同前置体力条件，不再依赖包内武将数量。
- XIANDING 包更新为 ABI 583、6 名武将、66 项技能、61 个模块；逻辑哈希
  `f9f4da0574236baeeeb79ed4322be22201cbd94a2a7eb36e68ddf9a9dc39f872`，文件 SHA-256
  `5452112b5aa6be6a255b9ced6715b3c0376dc8065dee5ed572cc160467cee19d`，源码产物与
  `extensions/xianding` 两份投放字节一致。
- conformance 8/8；全部已提交扩展从空临时 store 经真实离线导入/激活路径通过。
- 编译产物遵循架构计划 §37.10：并发测试释放后已删除本批 6.4 GiB 临时 target 和统一共享
  target，工作区 `target/` 也不存在；`.npack` 与 `extensions/` 发布物不在清理范围内。
