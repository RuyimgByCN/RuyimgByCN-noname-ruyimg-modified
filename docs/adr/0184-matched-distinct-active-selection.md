# ADR 0184：匹配数量的不同类别主动选择

## 背景

旧版 COLLAB「未羊」通过 `ui.selected.cards` 动态限制牌类别，并把目标数实时绑定到已选牌数。
新运行时不执行旧 JavaScript，也不能把 UI 临时状态当作规则权威。

## 决策

Rule VM ABI 109 增加三项通用声明式约束：

- `select_hand|select_he ... distinct_categories`：有效基本、锦囊、装备类别两两不同；
- `targets ... wounded`：每个目标必须存活且当前体力低于体力上限；
- `targets ... match_selected_card_count`：目标数与选牌数相同，两个声明须使用相同范围。

自动选牌 Choice 会按已选目标数收紧为精确数量；直接提交与 Choice 恢复都会重新验证牌归属、
区域、类别、目标状态和数量。新增 `recover_selected_targets(amount)`，消费主动命令的权威目标
快照，去重并按座次逐一回复。

## 版本与兼容

Rule VM ABI 从 108 提升到 109，Engine Semantics 从 155 提升到 156，新增 HOST_EFFECT 127，
并在 `EffectRequest` 尾部追加公开变体。Rule IR 保持 v76；内容 schema、包格式、网络协议、
回放和快照 schema 不变。旧 ABI 包继续由 ABI 109 主机加载。
