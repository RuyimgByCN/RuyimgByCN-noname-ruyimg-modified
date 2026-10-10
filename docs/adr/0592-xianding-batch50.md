# ADR 0592：限定包批次五十（施仇·OL版）

D 段（opencode）第五十个批次：XIANDING（限定包）「施仇·OL 版」（`onedcspshichou`）
以既有 DSL 原语表达，不新增 Rule VM ABI；包 ABI 保持 607。

- 「施仇·OL 版」：`view_as juedou play_phase any from hand_equip`——可将一张手牌/
  装备区的牌当【决斗】使用。旧 JS 的「使用【杀】未造成伤害后」触发条件与「对此杀
  的目标使用」限定无对应原语，以常驻 view_as 近似（注释已注明）。

探索记录（本批未采用）：「妙弦」的「任意普通锦囊」需要动态族 view_as，pack 校验要求
`ordinary_trick ... from none cost <skill>`（材料由 cost 子技能管理，仅 dczhaowen 模式），
现有原语无法直接表达，保持空壳。另确认 xianding 包依赖 sp/huicui/sp2 且
characters.json 的 24 个跨包技能引用全部可解析（依赖链完整）。Golden 见
`original_xianding_batch50.rs`（2 例，全过）；全量回归（批次 1~49）与
`cargo run -p conformance` 全过。编译遵循架构计划 §37.10：工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后删除外部与
工作区 `target/`。
