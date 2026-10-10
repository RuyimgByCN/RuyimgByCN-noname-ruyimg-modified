# ADR 0047：濒死自救请求的 continuation 契约

- 状态：已接受
- Engine semantics：31

## 背景

伤害进入濒死后，引擎先排入自救选择，再让濒死触发技能抢先结算。若强制技能本身打开选择，原先预写在全局解析状态中的 `DyingSave` continuation 会被覆盖，恢复队列时只剩一个无类型的普通选择。

## 决策

- 新增引擎类型化效果 `RequestDyingSave { player, source }`；
- 自救选择实际执行时才写入 `DyingSave` continuation 和击杀来源；
- 濒死触发技能仍插入队首，其选择暂停时，完整的自救请求保留在确定性效果队列中；
- 决斗、群体锦囊等既有被濒死打断的 continuation 继续使用原有 `resume_kind`。

## 结果

「烈斥」等在濒死时先产生选择的技能结算后，玩家仍会进入正确的自救或死亡流程，来源归因不会丢失。
