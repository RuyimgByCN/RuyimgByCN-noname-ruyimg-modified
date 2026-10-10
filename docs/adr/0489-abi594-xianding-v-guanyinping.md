# ADR 0489：ABI 594 XIANDING 威关银屏

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。威关银屏（`v_guanyinping`，4134246215）
以新 DSL 和共享引擎原语完成技能闭包：

- 「绍威」读取一次失牌事件的有效花色；方片、红桃分支每回合各限一次。方片分支使用权威 RNG
  从牌堆全部红牌中随机获得至多三张；红桃分支先回复 1 点体力，再恢复一个当前废除的装备栏。
- 「涤仇」用技能自身稳定 ID 记录其他角色对拥有者用牌形成的“仇”。回合开始废除一个可用装备栏，
  再从有“仇”的角色手牌/装备区弃置一张牌，按本地化牌名字数临时增加攻击范围和【杀】使用次数；
  拥有者对“仇”角色用牌时强制摸一张并移去一枚“仇”；结束阶段选择一至全部“仇”角色，依座次
  各造成 1 点火焰伤害并清除其全部“仇”。

ABI 594 新增 `resolve_dcshaowei(player, action)` 与
`resolve_dcdichou(player, action, helper_skill_def)`，复用 CardsMoved 实体上下文、装备栏续体、
权威 RNG、弃牌、名字数、临时技能、修正器与火焰伤害管线；没有执行或转译旧 JS。

验收：威关银屏 Golden 3/3、原批次 Golden 3/3、XIANDING 35 个专项测试目标共 125 项、
content-compiler、engine-api、Rule VM、conformance 与 26 包空 store 冷导入全部通过；内容锁为
`3dee2775a312bcb3a7e59bd066132c22779fada515d19efa273a1141544649c2`。XIANDING 包为 ABI 594、
13 名武将、79 项技能、83 个模块，logic hash 为
`fc070941db5875f2a770e185ec577927ce3d28ce2c36835709ab86b72b11ad5d`；源码与
`extensions/xianding` 四份 npack SHA-256 均为
`2a1990b3eca45a6fcc50cba258810c7d57d30bbb6edab7be43ed699ec4378f31`。

编译产物按架构计划 §37.10 在批次结束后强制清理：工作区 `target/` 删除 2,125 个文件
（Cargo 报告 733.8 MiB），共享 `CARGO_TARGET_DIR` 删除 17,853 个文件（9.8 GiB），两目录均
确认不存在；上述 npack 与 `extensions/` 发布物保留且清理后 SHA-256 不变。
