# ADR 0202：第三方交牌否则恢复采用跨观察者选择原语

## 背景

旧 SIXIANG「谙达」在技能拥有者确认发动后，把后续决定交给伤害来源：来源可以交给当前濒死
角色一张手牌或装备牌，否则该角色回复 1 点体力。技能拥有者、Choice 观察者和效果目标可能是
三个不同角色，不能把旧 JavaScript 的 `chooseToGive` 回调直接带入新运行时。

## 决策

Rule VM ABI 129 追加
`choose_give_one_hand_equip_or_recover(owner, giver, target)`。`owner` 只用于绑定并重新验证来源
技能实例，`giver` 是权威选择观察者，`target` 是赠牌接收者或恢复目标。引擎打开可取消的
0..1 张手牌/装备牌选择，并把三名角色、技能实例及原始选项快照写入 continuation 97。

Choice 恢复时重新验证观察者、技能归属与启用状态、三名角色存活、目标仍处于濒死状态，以及
选牌仍由来源持有且仍在手牌区或装备区。合法选牌走普通 `GiveCards` 管线；取消、空选或来源
起始无牌时，对目标执行无来源的 1 点恢复。触发过滤器
`owner_alive_source_alive_event_player_dying_once_per_round` 同时承担每轮限一次与濒死上下文
约束。

## 结果

- 旧 JavaScript 只保留为语义证据，不进入 npack 或运行时。
- 同类“由第三方决定交一张牌，否则恢复”技能可复用同一原语，无需复制角色专用状态机。
- ABI 129 追加 HOST_EFFECT 142、continuation 97 和 `EffectRequest` 哈希标签 179。
