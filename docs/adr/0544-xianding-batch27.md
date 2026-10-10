# ADR 0544：限定包批次二十七（施仇/壮魄/剑出）

D 段（opencode）第二十七个批次：XIANDING（限定包）三个技能以既有 DSL view_as 原语表达，
不新增 Rule VM ABI；包 ABI 保持 607。

- 「施仇」（`twodcspshichou`，SP 马超 twodc 版）：`view_as sha play_phase any from hand_equip
  no_use_count`——出牌阶段可将手牌/装备牌当【杀】使用且不计次数限制。旧 JS 的“每次需使用
  不同颜色的牌转化、必须指定相同的目标、红杀需目标弃装备否则不可响应、黑杀需目标弃黑色
  手牌否则不可响应”未建模。
- 「壮魄」（`zhuangpo`，神许褚）：`view_as juedou play_phase any from hand_equip`——可将牌
  当【决斗】使用。旧 JS 的“牌面信息包含【杀】的牌”约束（无牌面信息过滤器）与“此决斗
  指定拥有擎标记的目标时可移去任意枚擎令其弃置等量牌、此牌对拥有擎的目标伤害+1”联动
  未建模。
- 「剑出」（`sejianchu`）：`view_as sha play_phase card_shan from hand`——可将【闪】当【杀】
  使用。旧 JS 的觉醒条件（摸牌阶段外累计获得 5 张牌）、讥歌/立仁技能授予与死亡传递未建模
  ——本实现独立提供闪→杀转化。

三个技能均从空壳占位转为实现。Golden 见 `original_xianding_batch27.rs`（2 例，全过）；
`cargo run -p conformance` 全量通过。编译遵循架构计划 §37.10：使用工作区外
`CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，批次结束后执行
`cargo clean --target-dir` 并确认工作区 `target/` 不存在。
