# ADR 0309：限定包批次一（DC 甘夫人与六技能）

D 段（opencode）首个批次：六个 XIANDING（限定包）技能全部以既有 DSL 原语组合表达，
不新增 Rule VM ABI；DC 甘夫人（`dc_ganfuren`）完成闭环，作为完整武将进入选将池。

- 「神智」（`dcshenzhi`）：`phase_prepare` + `owner_is_target_hand_above_hp` 过滤器，弃置一张
  手牌并回复 1 点体力。旧 JS 的择弃成本以 `discard_hand(self, 1)` 表达。
- 「淑慎」（`dcshushen`）：复用 ABI 101 的 `choose_other_draw_or_recover(self, 2, 1, event_amount)`，
  与 SP「淑慎」、标「淑慎」同一权威语义，逐回复点触发。旧 JS“双方各摸一张”的分支按引擎统一
  语义（目标摸两张或回复一点）近似。
- 「诋敌」（`dcjidi`）：`damage_before` + `owner_is_target_source_alive`，在触发器内比较来源与
  拥有者的体力、手牌数；来源较多则失去 1 点体力或弃置两张手牌。旧 JS 的随机弃置以手牌区
  顺序弃置近似。
- 「制略」（`dczhijue`）：`distance_from add -2` 修正；`damage` 触发器内 `hand_count < max_hp`
  时 `adjust_hand_to(self, max_hp)` 摸至体力上限，并以 `ban_active_until_turn_end(self, self_skill)`
  近似旧 JS 的 `tempBanSkill`。
- 「闺秀」（`dcguixiu`）：`phase_begin` 时若本局未发动过则加标记并摸两张。第二段“发动存嗣后
  回复”依赖 `dccunsi` 技能，暂未建模。
- 「昭汉」（`dczhaohan`）：`phase_draw` + `increase_current_draw_count(self, 2)`。旧 JS 摸牌后的
  “交给无手牌角色或弃置两张”分支依赖本回合摸牌后结算，暂未建模。

DC 甘夫人（`dc_ganfuren`，3 体力蜀）由「淑慎」+「神智」组成技能闭包，写入
`characters.json` 并在 `extensions/xianding/` 发布 npack。旧 JavaScript 不进入 npack 或运行时。
