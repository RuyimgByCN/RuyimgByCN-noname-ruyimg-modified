# ADR 0422：限定包批次十三（金玠）

> 本 ADR 记录历史近似实现；【尽节】精确实现已由 ADR 0477（ABI 582）取代，杨彪完整闭包与
> 【举讹】精确实现见 ADR 0478（ABI 583）。

D 段（opencode）第十三个批次：XIANDING（限定包）技能「金玠」以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 451。

- 「金玠」（`dcjinjie`，杨彪）：`dying` 事件 + `owner_alive_other_dying_once_per_round`
  过滤器（每轮限一次）+ `choose_target_draw(self, 2, any)`（令其摸两张牌）+
  `choose_recover_target(self, event_player, 1)`（令其回复 1 点体力，沿用
  ChooseDrawOrRecover 结算）。旧 JS 为“声明整数 X∈[0,3] 令其摸 X 张，然后可弃置 X 张牌令其
  回复 1 点体力”——整数声明以固定 X=2 近似，“弃置 X 张牌”代价省略直接回复，已注明。

杨彪另一技 dcjue（举讹，结束阶段随机使用杀/过拆/五谷至弃置牌数）当时未建模，现已由
ADR 0478 补全。旧 JavaScript
不进入 npack 或运行时。Golden 见 `original_xianding_batch13.rs`（3 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
