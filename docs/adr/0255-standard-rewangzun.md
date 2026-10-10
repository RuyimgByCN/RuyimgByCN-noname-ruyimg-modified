# ADR 0255: STANDARD「妄尊」准备阶段分支

## 决策

ABI 179 增加 `owner_alive_prepare_player_hp_gt_owner` 与
`owner_alive_prepare_lord_hp_gt_owner`。前者在体力更高的角色准备阶段令技能拥有者摸一张牌；
后者只匹配身份模式的主公，额外摸一张，并给该主公的 `rewangzun2` 标记叠加一层。

`rewangzun2` 继续复用 `hand_limit add -owner_skill_mark`；同名技能只安装一次，标记可由多名
拥有者叠加，并在该主公回合结束时清空标记和临时技能。该 ABI 不增加 HOST_EFFECT、
`EffectRequest` 或 continuation。旧 JavaScript 的模式分支中，当前 STANDARD 身份包只实现
已有权威身份状态的 identity 语义；AI、音频、提示和动画不进入 npack 或运行时。
