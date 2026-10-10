# ADR 0295：SHIJI 卞夫人「挽危」「约俭」语义与救护原语（ABI 401）

ABI 401 增加
`spwanwei_active(player, target)`、`spwanwei_dying(player, target)` 权威原语，
以及触发过滤器 `owner_alive_other_dying_once_per_round` 和
`owner_is_target_discardable_hand_equip_gte_two`。

「挽危」（`spwanwei`）在出牌阶段限一次或当其他角色进入濒死状态时可发动：
- 主动阶段：目标回复 `player.hp` 点体力，拥有者失去 `player.hp` 点体力。
- 濒死救援：目标回复 `max(player.hp + 1, 1 - target.hp)` 点体力，拥有者失去 `player.hp` 点体力。
- 每轮限定触发一次（`once_per_round`）。

「约俭」（`spyuejian`）修饰手牌上限基数等于体力上限（`hand_limit_base_max_hp override 1`）；
进入濒死状态时若手牌/装备区可弃置牌不小于两张，可选弃置两张并回复 1 点体力。

卞夫人（`sp_bianfuren`）与「挽危」「约俭」组成第 347 名完整 3 体力魏势力武将闭包发布。
旧 JavaScript 不进入 npack 或运行时。
