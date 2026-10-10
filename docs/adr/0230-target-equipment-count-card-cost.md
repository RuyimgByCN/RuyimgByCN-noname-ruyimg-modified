# ADR 0230：目标装备数驱动主动技牌成本

## 决策

ABI 157 为主动技目标契约增加
`selected_cards_match_target_equip_count`。它必须与 `targets 1 1 has_equip`
和卡牌选择声明同时使用；引擎在打开选择及提交命令时，以唯一目标当前装备区牌数作为精确
选牌数，并继续复用既有 `discard_selected`、`deal_damage` 与主动技次数通道。

## 理由

旧 JavaScript 的 `ui.selected`、动态 `selectCard` 和 `filterTarget` 只是在表达选牌数与目标
装备数的关系，不需要技能专用效果。把该关系放进声明式选择契约，可由服务器统一校验、选择
续体统一恢复、重放与状态哈希统一覆盖，同时不执行旧 JS。

## 兼容性

字段追加在包目标契约末尾并带默认值，旧包反序列化不变；启用该字段的包要求 ABI 157。
未增加 HOST_EFFECT、continuation 或 `EffectRequest`。
