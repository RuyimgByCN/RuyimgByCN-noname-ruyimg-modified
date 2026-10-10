# ADR 0052：摸牌后由另一角色获得牌

## 决策

新增通用 DSL effect `draw_then_take_one(player, taker, count)`：先完成摸牌及其触发链，再由仍存活的 `taker` 从 `player` 的手牌或装备牌中强制选择一张获得。选择复用现有 `PickTargetCard` continuation。

新增 `owner_is_target_once` 过滤器，以技能实例的每回合触发计数表达“每回合第一次匹配拥有者的事件”。

## 原因

旧 JS 的 `stdzhuan` 依赖摸牌后的最新区域状态，且获得牌的选择权属于伤害来源。把这段流程做成通用 effect，能保持触发、选择、录像和状态哈希确定性，避免技能 ID 特判。

## 兼容性

新增 VM effect tag 31、`EffectRequest` 变体和触发过滤器；引擎语义版本升至 36。
