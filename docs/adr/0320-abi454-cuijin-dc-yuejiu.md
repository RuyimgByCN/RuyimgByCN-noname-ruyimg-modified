# ADR 0320：ABI 454 目标过滤器 + HUICUI 界沮授（崔进）

本批落地 C 段第五个 ABI（454）：`owner_source_or_within_distance_one_sha_or_juedou`
触发器过滤器，并以此移植 HUICUI 界沮授（`dc_yuejiu`）的「崔进」。

## ABI 454：`owner_source_or_within_distance_one_sha_or_juedou`

新触发器过滤器：当前事件为【杀】或【决斗】的卡牌使用，且使用者是拥有者或
使用者与拥有者距离不超过 1（复用 `player_in_attack_range` 判定）。

改动面（全部追加式）：
- engine-core trigger/catalog.rs：`TriggerFilter::OwnerSourceOrWithinDistanceOneShaOrJuedou` 变体
- trigger/dispatch.rs：分发臂 `(card_is_sha || card_is_juedou) && (owner_is_source || player_in_attack_range(ctx.source, active.owner))`
- content/registry.rs 字符串映射 + typecheck.rs 过滤器列表
- content-compiler `required_rule_vm_abi` 454 门控
- `RULE_VM_ABI_VERSION` 453→454（ABI-COORDINATION.md 已领取 ABI-MAX 454）

## 技能移植

「崔进」（`dccuijin`，3710121415）：任意角色使用【杀】或【决斗】时（使用者是你或
你与其距离不超过 1）→ 弃一张手牌/装备使该牌造成的伤害 +1。DSL 以 ABI 454 过滤器 +
`discard_hand_equip(self, 1)` + `increase_current_card_base_damage(self, event_card, 1)`。

文档化近似：旧 JS 手动选择弃牌（chooseToDiscard），DSL 由引擎自动选择弃牌。

界沮授（`dc_yuejiu`，2596857442，男/群/4 体力）由单一技能构成完整闭包，加入
`characters.json` 选将池；huicui npack 重建为 23 技能 + 3 武将，ABI 454。
旧 JavaScript 不进入 npack 或运行时。
