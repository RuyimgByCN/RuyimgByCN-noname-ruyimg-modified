# ADR 0054：每回合一次成为他人【杀】目标

新增通用过滤器 `owner_targeted_by_other_sha_once`：仅匹配 `card_targeted`，要求拥有者为本目标、使用者为其他角色、逻辑牌为【杀】，并复用技能实例的每回合触发计数。

用于重写 `stdshiren`，后续摸牌与交牌直接复用 `draw_then_give_one`，不新增 effect 或技能 ID 特判。引擎语义版本升至 38。
