# ADR 0270：SIXIANG「豹变」的失去体力、弃牌与条件杀

ABI 194 增加通用过滤器 `owner_is_target_any_alive_self_discardable_hand`，只在阶段角色为
存活技能拥有者且至少一名存活角色能自行弃置手牌时开放发动窗口。

`choose_target_lose_hp_discard_hand_sha_if_basic` 先选择目标并令拥有者失去 1 点体力，再由
目标自行弃置一张手牌；移动触发完成后，若弃牌有效类别为基本牌且双方仍存活，则复用普通
【杀】管线，忽略距离但保留通常使用次数限制。标夏侯霸以该单技能形成完整闭包；旧
JavaScript 仅作为顺序与条件的语义证据。
