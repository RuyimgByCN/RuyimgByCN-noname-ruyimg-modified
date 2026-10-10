# ADR 0588：限定包批次四十六（毒逝）

D 段（opencode）第四十六个批次：XIANDING（限定包）「毒逝」（`dushi`，弘农王）以既有
DSL 原语表达，不新增 Rule VM ABI；包 ABI 保持 607。

- 「毒逝」：`locked` + `modifier card_target_cancel(tao) override 1`（诅咒被动：拥有者
  不能成为其他角色【桃】的目标——复用批次四十三 dushi2 的桃取消管线）+
  `trigger death owner_is_target` + `effect choose_other_add_skill(self, dushi_id)`
  （死亡时选择一名其他角色，授予其「毒逝」完成诅咒传递）。
- 新发现原语：`choose_other_add_skill(player, skill_def)`（引擎内为
  `resolve_hfjieying`，target 为 NONE 时以 `ContinuationKind::Custom` + 目标选择
  continuation 询问，选择后 `AddSkill`）。注意 dispatch 返回 Ok(effects) 时需先
  `run_effects` 应用才会进入选目标阶段。

Golden 见 `original_xianding_batch46.rs`（2 例，全过）：包契约（card_target_cancel(tao)
override 1 + death 触发器）、死亡传递诅咒（选目标后目标获得 dushi）。全量回归
（批次 1~45）与 `cargo run -p conformance` 全过。编译遵循架构计划 §37.10：工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后删除外部与
工作区 `target/`。
