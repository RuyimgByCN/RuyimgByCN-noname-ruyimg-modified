# ADR 0591：限定包批次四十九（僻境）

D 段（opencode）第四十九个批次：XIANDING（限定包）「僻境」（`dcpijing`）以既有
DSL 原语表达，不新增 Rule VM ABI；包 ABI 保持 607。

- 「僻境」：`trigger phase_end owner_is_target optional` +
  `choose_other_add_skill(self, dczimu=4186409591)`——结束阶段开始时，可令一名其他
  角色获得「自牧」（dczimu）。旧 JS 的「任意名角色获得、其余角色失去自牧」全场
  重分配以单目标授予近似（省略失去部分，注释已注明）。

复用批次四十六的 `choose_other_add_skill`（选目标后 `AddSkill`，引擎内
`resolve_hfjieying`）。Golden 见 `original_xianding_batch49.rs`（2 例，全过）。
全量回归（批次 1~48）与 `cargo run -p conformance` 全过。编译遵循架构计划 §37.10：
工作区外 `CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后删除
外部与工作区 `target/`。
