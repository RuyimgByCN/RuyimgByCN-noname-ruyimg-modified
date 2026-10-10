# ADR 0439：限定包批次十七（梦魇）

D 段（opencode）第十七个批次：XIANDING（限定包）技能「梦魇」以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 521。

- 「梦魇」（`dcwuhun`，神关羽②）：`damage` + `owner_is_target_other_source_alive_per_damage_point`
  （拥有者是其他角色造成的伤害的目标且来源存活，按伤害点数计数）+ `add_mark(event_source, …)`——
  你受到其他角色造成的伤害后，令伤害来源获得等量“梦魇”标记
  （mark 3698112918 = sha256("noname-next/original/mark/dcwuhun")）。
  旧 JS 的①“你造成伤害后令受伤角色获得标记”无来源视角过滤器未建模；死亡时令拥有最多
  “梦魇”的角色判定（判定杀）未建模。

本技能属独立技（神关羽另一技 dcwushen 武神已移植，梦魇的死亡联动未完成），不提前写入
characters.json。旧 JavaScript 不进入 npack 或运行时。Golden 见
`original_xianding_batch17.rs`（3 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
