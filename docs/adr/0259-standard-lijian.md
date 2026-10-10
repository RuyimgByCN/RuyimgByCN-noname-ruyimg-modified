# ADR 0259: STANDARD「离间」有序虚拟决斗

ABI 183 增加主动目标约束 `second_can_use_juedou_on_first` 与
`use_virtual_juedou_second_on_first()`（HOST_EFFECT 181）。

目标数组保持旧语义的有序性：第二名男性角色必须能对第一名男性角色使用无来源【决斗】，
并在弃牌费用发生前完成权威校验。结算复用普通独立锦囊队列和【决斗】响应、伤害、归因，
但按旧 `nowuxie` 语义不打开【无懈可击】窗口。未增加 continuation；
`EffectRequest` 稳定哈希标签为 222。貂蝉以「离间、闭月」完整发布，旧 JavaScript 不执行。
