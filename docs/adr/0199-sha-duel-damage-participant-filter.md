# ADR 0199：杀/决斗伤害参与者过滤器

## 状态

已接受。

## 背景

SIXIANG「纵计」在任意逻辑【杀】或【决斗】造成伤害后询问技能拥有者，并依次允许其弃置
受伤角色和伤害来源各一张手牌或装备牌。旧 JavaScript 通过 `damageEnd` 上的 `event.card`、
`event.player` 和 `event.source` 判断；运行时不能执行该回调。

已有 ABI 36 的 `choose_discard_target_card` 已能权威选择、重验并弃置指定角色的一张合法牌，
所以无需新增弃牌效果。缺口是全局 Damage 过滤器，以及【决斗】伤害没有像【杀】伤害一样携带
逻辑牌来源。

## 决策

ABI 126 追加
`owner_alive_sha_or_duel_damage_source_target_alive_hand_equip_nonempty`。过滤器要求：

- 技能拥有者、伤害来源和受伤角色均存活；
- 伤害来自逻辑【杀】或【决斗】；
- 来源与目标的手牌、装备牌合计非空。

「纵计」的 VM 模块按旧语义先对 `event_player`、再对 `event_source` 调用
`choose_discard_target_card(..., hand_equip)`。效果在每个 Choice 屏障恢复时重验牌的所有权、
区域和弃置保护；没有合法牌的一方直接跳过。

【决斗】结算改用 `DealCardDamage` 传递独立锦囊队列中保存的物理牌 ID 和逻辑
`TrickJuedou` 类别。没有队列来源的内部旧路径继续使用普通 `DealDamage`，避免伪造实体牌。

## 后果

- 旧 JavaScript 仅作语义证据，不进入 npack 或运行时。
- 【杀】与【决斗】的 DamageBefore / Damage 触发都能读取正确逻辑牌类别。
- 不新增专用「纵计」Host 效果，弃牌仍走共享、可回放的权威状态机。
