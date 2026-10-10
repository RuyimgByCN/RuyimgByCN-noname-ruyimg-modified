# ADR 0213：出牌阶段角色与卡牌类别绑定

## 背景

四象「智诫」要求每轮展示当前出牌阶段角色的一张手牌，并在这个阶段内监听该角色前三次使用
同类别牌。旧实现依赖临时子技能、任意数组 storage 与 `getHistory("useCard")`，不能直接进入
确定性运行时。

## 决策

Rule VM ABI 140 追加：

- `choose_reveal_hand_card_bind_category_target(player, target)`（HOST_EFFECT 151），复用既有公开
  手牌 Choice continuation 58；
- `owner_alive_event_player_hand_nonempty_once_per_round`，复用技能实例的 `round_fire_round`；
- `owner_source_bound_category_first_three_play_phase`，复用 `play_phase_fire_count`，上限为 3；
- 技能实例追加 `BCT1 + PlayerId + category_bit` 状态形状，并在真实阶段切换时清除。

新 `EffectRequest` 只在枚举末尾追加，稳定哈希标签为 187。旧的 `RCG1` 强识类别状态及 ABI 53
效果保持原语义。

## 结果

技能 DSL 只描述阶段窗口、公开并绑定、以及匹配后摸牌；宿主统一处理隐藏手牌选择、类别计算、
每轮/每阶段计数与阶段清理。Golden 覆盖类别不匹配、前三次匹配、第四次停止、阶段清理和同轮
不可重复发动。
