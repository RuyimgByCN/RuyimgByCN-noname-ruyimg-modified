# ADR 0501：ABI 598 XIANDING 卞玥

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。卞玥（`bianyue`，3005636206）的「庇族」
（`dcbizu`，2914157062）在每次发动时精确计算所有与拥有者当前手牌数相同的存活角色，按座次
各摸一张；若本次目标集合与当前回合此前一次发动完全相同，则在摸牌及嵌套触发结束后本回合
禁用该主动技并回复 1 点体力。「无胁」（`dcwuxie`，3514743897）在自己的结束阶段且双方均有
手牌时选择一名其他角色，交换双方全部实体手牌，再把拥有者新手牌中的全部伤害牌置于牌堆底。

ABI 598 新增 `resolve_bianyue(player, target, action)` 和精确结束阶段过滤器。实现复用摸牌、主动技
回合禁用、回复、标准目标选择屏障、实体移牌及牌堆底顺序；技能实例只保存当前回合已出现的
规范化目标集合，不扫描或保存旧 JS 事件树。

验收：卞玥 Golden 2/2、XIANDING 40 个专项测试目标共 138 项、content-compiler 496/496、
Rule VM 10/10、engine-api 4/4、conformance 8/8 均通过。打包结果为 ABI 598、17 名武将、
87 项技能、92 个模块，logic hash 为
`ae27922dc07d72a4c8681ea89d74db8787c452efcab7277e9c6dc5194cfe2bad`；源码与
`extensions/xianding` 四份 npack SHA-256 均为
`5f654e57393a3d04d290c2fa2b651fa001daf3c9a5048e96becd3a55d0b57207`。26 个官方包从空 store
冷导入通过，ContentLock 为
`5301583a57d830892a43685553d9956d96b6708a78acec7e4ec56fc7fa864792`。全局同步器复验当前被共享
工作区 SB 包 `sbyangwei` 的重复 definition ID 阻断，待对应并发批次修复；不从卞玥侧绕过校验。
本批冷导入临时目录已删除；工作区 `target/` 与共享 `CARGO_TARGET_DIR` 正被并发构建占用，释放后
按架构计划 §37.10 删除，`.npack` 与 `extensions/` 发布物保留。
