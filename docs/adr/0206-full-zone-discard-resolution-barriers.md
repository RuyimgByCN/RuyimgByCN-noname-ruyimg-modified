# ADR 0206：整区弃牌选择与双结算屏障

## 背景

旧 SIXIANG「躁厉」在准备阶段强制拥有者选择手牌区或装备区，弃置该区全部牌；弃牌完成后，
以实际弃牌数加当前已损失体力值摸牌，摸牌完成后再失去 1 点体力。旧实现依赖
`chooseControl`、两次 `await` 和运行时区域字符串。若简单排成同一个效果队列，摸牌会早于
弃牌触发，失去体力又会早于摸牌触发，语义不等价。

## 决策

Rule VM ABI 133 增加
`choose_discard_all_hand_or_equip_draw_lost_hp_then_lose_hp(player)`：

- 引擎只提供当前至少含一张可弃牌的手牌/装备分支；单分支自动执行，双分支强制选择；
- 选定区域的全部合法牌按一个确定性移动批次弃置；
- movement deferred-tail 在弃牌及其嵌套触发后重新读取已损失体力并摸牌；
- 第二个 deferred-tail 在摸牌及其嵌套触发后执行 1 点失去体力。

ABI 133 追加 HOST_EFFECT 145、continuation 98 和 `EffectRequest` 哈希标签 181。

## 结果

- 旧 JavaScript 的 Promise、区域字符串与 AI 回调不进入 npack 或运行时。
- 选择恢复、卡牌实体、区域所有权和技能上下文都由权威引擎重新验证。
- 嵌套移动触发改变体力时会影响摸牌数；摸牌触发始终早于最终失去体力。
