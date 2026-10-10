# ADR 0211：受伤后选择其他角色并赠牌

## 背景

旧版 SIXIANG「毅谋」在拥有者受到正数伤害后开放可选窗口；只有拥有者有手牌或装备牌且存在
其他角色时才能发动。发动时同时选择一张自己的手牌/装备牌和一名其他角色，再将该牌交给目标。
旧 JavaScript 的 `chooseCardTarget`、AI 估值及 Promise 控制流不能进入新运行时。

## 决策

Rule VM ABI 138 增加过滤器 `owner_is_target_hand_equip_nonempty_other_alive`，由权威引擎检查
正数伤害、拥有者存活、手牌/装备区非空和其他存活角色。

新增通用效果 `choose_other_give_one_hand_equip(player)`：确认发动后先以 continuation 102
选择一名其他存活角色；恢复时重验技能、观察者、双方存活和拥有者仍有牌，然后构造
`DrawThenGiveOne { count: 0, drawn_only: false, ready_to_give: true }`，复用既有
`GiveOneHandOrEquip` 卡牌选择与 `GiveCards` 移动。卡牌恢复仍会重验实体牌归属和手牌/装备区。

ABI 138 追加 HOST_EFFECT 150、continuation 102 和 `EffectRequest` 哈希标签 186。

## 结果

- 没有新增 JS 执行器，也没有复制赠牌移动和事件语义。
- 目标与卡牌分成两个确定性 Choice 屏障，录像、断线恢复和无头策略可复现。
- 「毅谋」与已迁移的「募讨」共同闭合标鲍信，整将可以安全进入 SIXIANG 包。
