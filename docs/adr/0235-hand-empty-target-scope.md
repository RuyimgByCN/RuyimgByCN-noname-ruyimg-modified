# ADR 0235：空手牌角色共享目标域

## 决策

ABI 162 为现有 `DrawTargetScope` / `DrawTargetScopeIr` 尾部追加 `HandEmpty`，稳定载荷标签为
6。DSL 名称为 `hand_empty`，匹配所有当前没有手牌的存活角色，并允许选择者选择自己。

`choose_target_draw`、`choose_targets_draw` 和 `choose_targets_damage` 继续共用既有候选生成与
continuation 校验；恢复时重新检查目标存活且手牌仍为空。结束阶段过滤器
`owner_is_target_any_alive_hand_empty` 只在拥有者是阶段角色且场上至少有一个合法候选时匹配。

## 结果

YIJIANG「绝策」只需组合可选结束阶段触发与既有多目标伤害效果，不增加技能专属
HOST_EFFECT、`EffectRequest`、continuation 或持久状态。旧 JS 仅作语义证据，不进入包或运行时。
