# ADR 0321：限定包批次二（精弓/忠节/花火/节行/静隐）

D 段（opencode）第二个批次：五个 XIANDING（限定包）技能全部以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

- 「精弓」（`dcjinggong`，DC 黄祖）：`view_as sha play_phase card_equip from hand_equip
  ignore_distance`，装备牌当无距离限制的【杀】。旧 JS 的伤害值基数改为与首目标距离（至多 5）
  未建模。
- 「忠节」（`dczhongjie`，赵昂）：`dying` 触发 + `owner_alive_other_dying_once_per_round` 过滤器，
  令濒死角色回复 1 点体力并摸一张牌。旧 JS 限定“因失去体力而濒死”（reason == loseHp），
  DSL 以每轮一次近似。
- 「花火」（`dchuahuo`，马伶俐）：`view_as sha play_phase red from hand_equip nature fire
  uses_per_turn 1`，红色手牌当火【杀】。旧 JS 的指定目标后摸牌/视为使用联动未建模。
- 「节行」（`dcjiexing`，袁姬）：`damage` / `hp_recovered` / `hp_lost` 三个触发各摸一张牌。
  旧 JS 的“此牌不计入手牌上限”（gaintag + 临时技能）未建模。
- 「静隐」（`dcjingyin`，李潭）：复用 ABI 151 `sha_resolved` 事件与
  `owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn` 过滤器，其他角色
  回合外使用【杀】结算后获得其实体牌。旧 JS 的“获牌者使用这些牌无次数限制”未建模。

本批次五个技能均为独立移植（武将闭包所需的其他技能尚未完成，不提前写入 characters.json）。
旧 JavaScript 不进入 npack 或运行时。Golden 见 `original_xianding_batch2.rs`（4 例，全过）；
`cargo run -p conformance` 全量通过。
