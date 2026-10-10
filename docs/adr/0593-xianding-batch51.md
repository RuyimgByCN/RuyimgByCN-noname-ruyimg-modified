# ADR 0593：限定包批次五十一（佩娘）

D 段（opencode）第五十一个批次：XIANDING（限定包）「佩娘」（`dcpeiniang`）以既有
DSL 原语表达，不新增 Rule VM ABI；包 ABI 保持 607。

- 「佩娘」：`view_as jiu play_phase any from hand_equip` + `modifier
  card_use_unlimited(jiu) override 1`——可将一张手牌或装备区的牌当【酒】使用，
  且【酒】无使用次数限制。旧 JS 的「dcyitong 记录花色」过滤与「濒死时亦可使用」
  联动未建模（注释已注明）。

Golden 见 `original_xianding_batch51.rs`（2 例，全过）。全量回归（批次 1~50）与
`cargo run -p conformance` 全过。编译遵循架构计划 §37.10：工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后删除外部与
工作区 `target/`。
