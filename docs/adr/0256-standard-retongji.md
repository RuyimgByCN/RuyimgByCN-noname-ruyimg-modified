# ADR 0256: STANDARD「同疾」弃牌转移【杀】目标

## 决策

ABI 180 增加过滤器 `owner_alive_other_sha_target_can_redirect_to_owner` 与效果
`choose_current_sha_target_discard_redirect_to_owner(owner, source, card, target)`
（HOST_EFFECT 178）。

技能拥有者不在当前【杀】目标中，且当前目标能弃置手牌/装备并能在其攻击范围内攻击拥有者时，
该目标可弃置一张牌。费用及其嵌套触发完成后，保存中的同一目标槽原子替换为技能拥有者，并发出
新增目标事件。转移保持目标数不变，绕过用牌者到新目标的通常距离，但重新校验目标禁用规则；
新目标使用独立的响应与伤害状态。

选择使用 continuation 118；新增 `EffectRequest` 稳定哈希标签 219。旧 JavaScript 的 AI、
音频、提示和动画不进入 npack 或运行时。
