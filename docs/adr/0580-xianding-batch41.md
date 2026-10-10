# ADR 0580：限定包批次四十一（天匠）

D 段（opencode）第四十一个批次：XIANDING（限定包）技能「天匠」以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「天匠」（`pytianjiang`，PY）：`game_started` + `owner_alive` +
  `gain_equipment_from_draw_pile(self)`——锁定技，游戏开始时从牌堆获得一张装备牌（复用
  xuyuan/xinpingkou 的装备获取管线）。旧 JS 的“获得两种不同类别的装备牌并立即装备一张、
  每轮开始时重复”省略。

本技能从空壳占位转为实现。Golden 见 `original_xianding_batch41.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
