# ADR 0561：限定包批次三十四（追猎/夺月）

D 段（opencode）第三十四个批次：XIANDING（限定包）两个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「追猎」（`spzhuilie`，sp 王双）：`modifier card_distance_unlimited(sha) override 1` +
  `modifier ignore_armor override 1`——锁定技，你使用【杀】无距离限制并无视防具。
  旧 JS 的“使用【杀】指定攻击范围外的目标后判定：装备牌则此【杀】伤害基数改为目标体力
  值且不计次数、其他非基本牌则你失去 1 点体力并获得判定牌”未建模。
- 「夺月」（`dcduoyue`，DC 神张辽）：`phase_play` + `owner_is_target` +
  `choose_hand_compare(self, win, lose)`（复用 chengzhao 拼点模式）——出牌阶段开始可选一名
  角色拼点；赢则获得子技能（`card_distance_unlimited(sha)` + `sha_use_limit add 999` 近似
  不计次数），结束阶段移除；输则移除子技能。旧 JS 的“拼点赢后可继续、输的后续”未建模。

两个技能均从空壳占位转为实现（dcduoyue 含 2 个子技能定义）。Golden 见
`original_xianding_batch34.rs`（3 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
