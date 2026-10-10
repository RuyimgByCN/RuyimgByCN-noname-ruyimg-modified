# ADR 0010: Card-aware hand-limit selectors

## Decision

- `card_ignored_hand_limit(selector)` 是“不计入手牌上限且不能在弃牌阶段弃置”的通用 DSL 通道。
- 首批静态选择器覆盖【杀】、基本/非基本、锦囊、装备、四种花色和红黑颜色；多个技能选择器按既有 modifier 顺序组合。
- 包格式在 modifier 条目上增加可选 `selector`，未使用选择器的旧 npack 反序列化和规范 JSON 保持兼容。
- 引擎使用 `ModifierTarget::CardIgnoredHandLimit(CardSelector)`；原 `ShaIgnoredHandLimit` 变体和
  `sha_ignored_hand_limit` DSL 名称保留为兼容入口，不再为新牌类继续增加专用目标。

## Compatibility

旧 JS 中同一规则按【杀】、锦囊、基本牌、花色、颜色乃至动态标签反复出现。为每个条件新增
modifier 会扩大查询 API、注册表和扩展 SDK，无法支撑上千技能。本决策先把静态条件收敛到
声明式选择器；动态标签和运行时状态条件后续通过可审计谓词 VM 扩展，不执行旧 JavaScript。
`cangzhuo2` 与 `qiaoshui4` 是首批使用 `trick` 选择器的历史技能，并以普通/延时锦囊的真实
弃牌选项验证。
