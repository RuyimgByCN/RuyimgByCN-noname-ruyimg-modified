# ADR 0457：限定包批次二十二（下策）

D 段（opencode）第二十二个批次：XIANDING（限定包）技能「下策」以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 521。

- 「下策」（`dcxiace`，白绫韵①）：`damage` + `owner_is_target` +
  `choose_discard_hand_equip_then_recover(self, 1, 1)`——当你受到伤害后可弃置一张手牌/
  装备牌并回复 1 点体力（该 effect 恰好一张可弃牌时直接弃牌回体、多张时打开选牌选择）。
  旧 JS 的“每回合每项各限一次”限次省略；②“当你造成伤害后可令一名其他角色的非锁定技于
  本回合失效”无全部非锁定技失效原语（仅有按技能定义失效），未建模。

> 后续状态：本批近似实现已由 ADR 0488 / ABI 593 取代；`dclinghui`、`dcxiace`、`dcyuxin`
> 均已按旧 JS 语义闭环，柏灵筠已写入 `characters.json`。
旧 JavaScript 不进入 npack 或运行时。Golden 见 `original_xianding_batch22.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
