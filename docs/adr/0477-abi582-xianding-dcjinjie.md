# 0477 ABI 582 XIANDING 杨彪【尽节】

日期：2026-08-03  
状态：已落地  
领取者：B（Codex overflow）

## 决策

旧 JavaScript 仅作为语义合同，不进入内容包或运行时。【尽节】改用现有
`choose_range(0, 3)` 声明 X，任意角色（包括拥有者）进入濒死时每轮限一次：X 大于零时先令
该角色摸 X 张，待摸牌及嵌套触发完成后，拥有者可以取消或弃置恰好 X 张当前手牌/装备牌；
成功支付及移动触发完成后，该濒死角色回复 1 点体力。X 为零时跳过摸牌和弃牌，直接询问是否
回复。触发改用既有 `owner_alive_once_per_round`，不再错误排除拥有者自身。

ABI 582 新增通用效果
`choose_discard_hand_equip_then_recover_target(chooser, target, count, amount)`。实现复用 ABI 134
的动态精确弃牌选择续体，通过向后兼容的扩展载荷增加“可取消、指定回复目标”模式；ABI 134
旧载荷仍保持强制弃牌并回复自己。Choice 恢复时重验技能、观察者、原始选项、当前可弃牌、目标
存活/受伤状态和回复量，不增加【尽节】专属状态机。

## 验收

- Compiler 合同覆盖 DSL → IR → bytecode → VM typed effect；content-compiler 491/491、
  engine-api 4/4、rule-vm 15/15 通过。
- 【尽节】Golden 5 项覆盖完整包合同、X=2 摸牌/等量弃牌/回复、X=0 自身濒死回复、取消弃牌、
  每轮限次；XIANDING 27 个测试文件共 98 项全绿。
- ABI 134 旧【聚进】弃牌后回复时序回归 1/1，证明扩展载荷未改变旧调用。
- XIANDING 包更新为 ABI 582、5 名武将、65 项技能、60 个模块；逻辑哈希
  `e10fb9ecdd7085fbd5f6012acddaf117a522a9f8812e2e126daa17b9feeb714f`，文件 SHA-256
  `ce1dc3b5ca827417b74e0287dbe65ef36b2afbe779f3c50df591141ebe2ef932`，源码产物与
  `extensions/xianding` 两份投放字节一致。
- conformance 8/8；当前 26 个扩展从空临时 store 经真实导入/激活路径通过。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 发布物不在清理范围。
