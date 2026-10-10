# ADR 0105：主动装备牌选择与重铸效果

## 决策

主动技的 `select_hand` / `select_he` 选牌合同新增可选 `card_equip` 过滤器，候选牌按
`CardKind::is_equip()` 的有效类别筛选；过滤器同时进入包逻辑哈希和服务端直接提交校验，客户端
传入未出现在候选集中的牌会被拒绝。DSL 新增通用效果 `recast_selected(player)`，把本次主动技
选中的个人手牌区或装备区牌以 `CardMoveReason::Recast` 移入弃牌堆，再摸等量牌。

`PackageSelectionSpec.filter` 使用反序列化缺省值，未声明过滤器的既有包行为不变；
`CardMoveReason`、`StmtIr`、`EffectRequest`、VM effect tag 46 和散列分支均只在末尾追加。该能力
提升 Rule VM ABI 至 16、Rule IR 至 26、Engine Semantics 至 101；只有使用主动选牌过滤或重铸
效果的包声明 ABI 16，ABI 3–15 包继续兼容加载。

## 迁移边界

KEY「怀柔」的旧 JavaScript 仅用于核对出牌阶段、手牌/装备区、装备类别及重铸语义。运行时只
加载 `active play_phase select_he 1 1 card_equip` 与 `recast_selected(self)` 编译出的类型化包，
不执行旧对象，也不按技能 ID 添加宿主分支。独立的 `Recast` 移动原因避免重铸误触只接受弃置的
技能。
