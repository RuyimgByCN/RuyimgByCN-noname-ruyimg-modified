# ADR 0333：ABI 462 无懈过滤器 + 举荐获牌原语 + SIXIANG 标徐庶

本批为 SIXIANG 包标徐庶（`std_xushu`）的「举荐」（`stdjujian`，2772808467）完整
移植，需要新的过滤器 `owner_source_wuxie_used` 与复合原语
`choose_other_gain_event_cards(player)`（ABI 462）。

## 原语设计（ABI 462）

- 过滤器 `owner_source_wuxie_used`：`card_used` 事件中拥有者为使用者、牌类为
  `CardKind::TrickWuxie` 且本回合未发动（`turn_fire_count == 0`，内建 once-per-turn
  —— 旧 JS 的 usable: 1 语义，因 DSL 过滤器槽位仅一个而折叠进过滤器）。
- 复合原语 `choose_other_gain_event_cards(player)`：编译为
  `effect_tag::RESOLVE_STDJUJIAN`（tag 377）；vm_host 从使用事件捕获被用牌
  （`event_card`）；选择结算走 `ContinuationKind::Custom` +
  `RESOLVE_STDJUJIAN_CUSTOM_CHOICE`（0xDCED_4620）：选择其他存活角色后复用
  `gain_event_cards` 将该牌（仍在弃牌堆时）移入其手牌。

## 技能移植

「举荐」（`stdjujian`，2772808467）：
- `trigger card_used owner_source_wuxie_used optional`：
  `choose_other_gain_event_cards(self)`。

## 打包与验证

- `stdjujian.skill` 加入 `content/original/ported/sixiang/`（README 显式列表补充）；
  标徐庶（4024740751，男/蜀/3 体力，技能 [stdwuyan 3164633937,
  stdjujian 2772808467]）加入 sixiang `characters.json`；catalog 标记
  `stdjujian` 为 `ported`。
- sixiang npack 重建（ABI 462，69 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sixiang/`。
- 运行时测试 `original_sixiang_stdjujian.rs` 全绿：首次使用无懈后目标获牌；
  同回合再次使用无懈不再触发（once-per-turn）。
- 旧 JavaScript 不进入 npack 或运行时。
