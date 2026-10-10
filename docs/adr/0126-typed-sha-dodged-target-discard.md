# ADR 0126：类型化【杀】闪避与目标区域弃牌

## 背景

旧「猛进」订阅 `shaMiss`：一张【杀】被目标以足量【闪】完全抵消后，使用者可以弃置目标手牌区
或装备区的一张牌。现有 `CardResponded` 同时表示【杀】、【万箭齐发】、【决斗】等响应，且无双
等规则会要求多张【闪】；把任意【闪】响应当作 `shaMiss` 会误触发并可能在第一张【闪】后提前发动。

## 决策

- 领域事件追加 `ShaDodged { source, target, card }`，规则事件名为 `sha_dodged`。单目标、多目标、
  实体【闪】、八卦和无来源视为【闪】统一在满足本次全部响应数量后发出一次；【万箭齐发】不发出。
- 过滤器 `owner_source_sha_dodged_target_alive_has_discardable_hand_equip` 要求技能拥有者是存活的
  【杀】来源、目标仍存活，且统一弃置保护查询至少返回一张手牌区或装备区候选。
- Rule VM ABI 36 追加
  `choose_discard_target_card(player, target, equip|hand_equip|hand_equip_judge)`，host effect tag 为 54。
  强制选择使用新的续体种类 39；打开选择与恢复时都重检参与者、所有权、区域和弃置保护。
- 该效果只产生语义为 `Discard` 的移动，不包含「趫猛」的坐骑回收；两项能力共享候选查询和最终
  `DiscardTargetCards` 原语，但使用独立续体，避免策略泄漏。

## 版本

- Rule VM ABI：36
- Rule IR：47
- Engine semantics：122
