# ADR 0011: All-card use-limit override

## Decision

- `card_use_unlimited` 表示技能拥有者使用所有牌时跳过普通的每回合/阶段次数上限。
- `card_use_unlimited(selector)` 仅豁免匹配牌名族、类别、花色或颜色的牌；无 selector 的既有
  语义保持不变。
- `sha_use_cancel` 等禁用通道优先于次数豁免；“不能使用”不会被“可无限使用”反向覆盖。
- 当前引擎只有【杀】实施阶段次数限制，因此通用查询由 `sha_use_limit_for` 消费，按牌查询由
  `sha_use_limit_for_card` 消费；后续新增其他卡牌次数规则时必须复用同一通道。
- `ModifierTarget::CardUseUnlimitedFor(CardSelector)` 追加在枚举末尾，保留已有序列化变体编号。

## Compatibility

旧技能 `yuiko_fenglun2` 的 `cardUsable()` 对所有牌返回无限，并同时跳过距离限制。将它仅编译成
`sha_use_limit` 会在未来新增锦囊或装备次数规则时产生语义漂移。本决策以全牌 modifier 保留旧
JS 的原始范围，并用“已使用一次【杀】后，命令从拒绝变为接受”的真实流程验证当前可观察行为。
