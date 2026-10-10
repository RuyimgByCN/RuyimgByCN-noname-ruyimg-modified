# ADR 0449：ABI 561 兵势周瑜

旧 JavaScript 只作为「炽沄、焰洄、焚涛、雄姿」的语义证据，不进入包或运行时。

- DSL 增加四个事件过滤器和 `resolve_potzhouyu(player, target, card, action)`；固定四参数由 VM host
  注入当前技能、牌标志与完整目标列表。
- 引擎以技能实例保存阶段/回合历史、焚涛待重新横置目标和雄姿分支锁；所有可恢复选择重新验证
  技能、角色、实体牌、区域及焰洄阶段末允许受伤的角色集合。
- 焚涛复用当前伤害增量、弃牌和连环标记；传播火伤按同一连锁累计增量，直接火伤重新计数。
- 雄姿只改变三项技能的两个既有分支，不新增表现层状态；旧皮肤切换不进入规则状态。
- Golden 覆盖炽沄、焰洄目标约束、焚涛弃牌后重新横置、雄姿统一锁分支和包清单。
- 势周瑜 Golden 6/6、BINGSHI 29 个测试目标、content-compiler 482/482、ABI/schema
  以及 conformance 8/8 通过。
- BINGSHI 包为 ABI 561、30 名武将、84 项技能、1 张卡，logic hash
  `781d4ee4ac8eda06a749e1a304c1fea7518f54d8f45d77edb003a9b47b2129d7`；三个发布副本字节一致。
- 完整 `extensions/` 在全新 store 冷导入 26 个包成功，ContentLock
  `39d2983aa24cdef7cecc35f66b339814d9045b61b64950bd4a528ac6bd529d1e`。
- 编译统一复用外部 `CARGO_TARGET_DIR`，批次门禁结束后按架构计划 §37.10 删除。
