# ADR 0311：DIY 孙翊（国裂）

本批为 DIY 包孙翊（`ns_sunyi`）的单技能完整移植，全部以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

- 「国裂」（`nsguolie`，3456830571）：`phase_draw` 触发 + `skip_current_phase(self)`
  （摸牌阶段在允许集内）+ `add_skill_until_turn_end(nsguolie2)`。
- 「国裂·转化」（`nsguolie2`，4032663402）：`view_as sha play_phase red from hand` +
  `view_as guohe play_phase black from hand` + `modifier card_use_unlimited override 1`。
  旧 JS 的 `cardname` mod（红→杀、黑→过河拆桥）以此等价表达；`cardUsable: Infinity`
  以使用次数无限制修饰器表达。

注：旧 JS 的转化在出牌与响应窗口均生效，DSL 的 view_as 限定 `play_phase` 窗口（主要
使用场景）；响应窗口的转化属文档化近似。旧 JS 以 `phaseJieshuBegin` 清理临时效果，
DSL 的 `add_skill_until_turn_end` 等价。

孙翊（`ns_sunyi`，2855781430，男/吴/4 体力）由单一技能构成完整闭包，加入
`characters.json` 选将池；diy npack 重建为 41 技能 + 10 武将。
旧 JavaScript 不进入 npack 或运行时。
