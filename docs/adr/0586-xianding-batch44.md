# ADR 0586：限定包批次四十四（决·旧版/张采）

D 段（opencode）第四十四个批次：XIANDING（限定包）两个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「决·旧版」（`oldjue`）：`phase_prepare owner_is_target optional` +
  `choose_use_virtual_sha(self, ignore_distance)`——准备阶段开始时，可视为对一名角色
  使用一张【杀】。旧 JS 的「体力值或手牌数大于你」目标筛选无对应原语，以任意目标近似
  （注释已注明）。
- 「张采」（`dczhangcai`）：`card_used owner_source optional` + `draw_cards(self, 1)`——
  使用牌时可摸一张牌。旧 JS 的「点数 8 过滤（或 dczhangcai_all 任意）」「摸同点数手牌
  数张且至少 1」以固定摸一张近似，「打出」时触发省略（注释已注明）。

两个技能均从空壳占位转为实现（A 段同款"核心效果 + 省略注明"近似风格）。
Golden 见 `original_xianding_batch44.rs`（3 例，全过）；全量回归（含批次 1~43）与
`cargo run -p conformance` 全过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
