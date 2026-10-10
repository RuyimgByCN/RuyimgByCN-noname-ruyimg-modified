# ADR 0587：限定包批次四十五（笑武/儒贤/苏慎）

D 段（opencode）第四十五个批次：XIANDING（限定包）三个技能以既有 DSL 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「笑武」（`dcxiaowu`）：`active play_phase uses_per_turn 1 select_he 1 1` +
  `discard_selected(self)` + `gain_card_by_kind(self, sha)`——出牌阶段限一次，弃置一张
  手牌或装备区的牌，从牌堆获得一张【杀】。旧 JS 的「本局发动过则获得两张」以固定
  一张近似（注释已注明）。
- 「儒贤」（`dcruxian`）：`active play_phase once_per_game` + `add_skill(self, dczhangcai_all)`
  ——限定技，本回合内「张采」可对所有点数发动。新增纯标记技能 `dczhangcai_all`
  （`turn_ended` + `remove_skill(self_skill)` 回合结束移除，id 3424574524 按既有
  sha256 规则生成）。因「张采」的 DSL 近似实现本身不区分点数，该标记无实际效果
  （联动保留，注释已注明）。
- 「苏慎」（`dcsbsushen`）：`active play_phase once_per_game` + `add_skill(self, dcsbrushi)`
  ——限定技，获得「覆谋」（dcsbrushi 为空壳，仅授予；旧 JS 的状态记录省略）。

Golden 见 `original_xianding_batch45.rs`（4 例，全过）：dcxiaowu 弃牌得杀、dcruxian
授予/回合末移除、dcsbsushen 授予、包契约（`active_index` 窗口 play_phase、
dczhangcai_all locked）。全量回归（批次 1~44）与 `cargo run -p conformance` 全过。
编译遵循架构计划 §37.10：工作区外 `CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，
批次结束后删除外部与工作区 `target/`。
