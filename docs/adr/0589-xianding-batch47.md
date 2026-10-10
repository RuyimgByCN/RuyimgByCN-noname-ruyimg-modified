# ADR 0589：限定包批次四十七（巡使/王抗/纫霜·旧版）

D 段（opencode）第四十七个批次：XIANDING（限定包）三个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「巡使」（`xunshi`）：`locked` + `modifier card_name(trick) override sha` +
  `card_distance_unlimited(trick)` + `card_use_unlimited(trick)`——锦囊牌均视为
  【杀】，无距离、无次数限制。旧 JS 的「仅手牌中的锦囊」「无花色」「神裁标记+1/
  额外目标」未建模（card_name 作用于全区域，注释已注明）。
- 「王抗」（`dcsbwankang`）：`trigger dying owner_is_target` + `recover_hp(self, 1)` +
  `draw_cards(self, 2)`——濒死时回复 1 点体力并摸两张牌。「标记+1」「回合结束对伤害
  来源用【杀】」子技能未建模。
- 「纫霜·旧版」（`old_dcrenshuang`）：`locked` + `trigger dying owner_is_target` +
  `recover_hp(self, 1 - hp(self))` + `change_max_hp(self, 1)`——濒死时回复至 1 并
  增加体力上限（dckunli 同款模式）。「首次/最多 3 次」次数限制与「解除连环/翻面、
  视为使用锦囊」未建模。

注：非 locked 且未写 optional 的 trigger 默认强制执行（dispatch 返回 Ok(effects)，
需 `run_effects` 应用后生效；dying 测试需先置 hp=0）。Golden 见
`original_xianding_batch47.rs`（3 例，全过）；全量回归（批次 1~46）与
`cargo run -p conformance` 全过。编译遵循架构计划 §37.10：工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后删除外部与
工作区 `target/`。
