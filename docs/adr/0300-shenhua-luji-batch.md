# ADR 0300：SHENHUA 纯 DSL 批次 1（陆绩：怀橘/遗礼/争论）

本批为陆绩（`luji`）三名技能的纯 DSL 移植，全部以既有 DSL 原语组合表达，不新增 Rule VM ABI。

- 「怀橘」（`nzry_huaiju`）：`game_started` 触发 `add_mark(self, 3)` 获得三枚「橘」；
  `damage_before` 时若持有「橘」则 `prevent_current_damage()` 并 `remove_mark(1)`；
  `phase_draw` 时若持有「橘」则 `increase_current_draw_count(1)`。
  旧 JS 的 `tachibana_effect` 以全局 `damageBegin4`/`phaseDrawBegin2` 表达，DSL 以
  拥有者视角的 `owner_is_target` + 标记计数条件近似；遗礼将「橘」转移给他人后的
  全局守护不在拥有者 DSL 中表达，属文档化近似。
- 「遗礼」（`nzry_yili`）：出牌阶段限一次，`choose(0,1)` 分支「失去 1 点体力」或
  「移去一枚「橘」」，然后 `add_mark(event_source, 1)`。旧 JS 以 `phaseUseBegin`
  触发器 + `chooseTarget` 表达，DSL 以 `active play_phase uses_per_turn 1 targets 1 1 other`
  等价表达。
- 「争论」（`nzry_zhenglun`）：`phase_draw` 时若无「橘」则 `skip_current_phase(self)`
  并 `add_mark(self, 1)`。旧 JS 在 `phaseDrawBefore` 取消摸牌阶段，DSL 以
  `skip_current_phase` 等价表达。

陆绩（`luji`，id 3130345384，男/吴/3 体力）由上述三技能构成完整闭包，加入
`characters.json` 选将池。旧 JavaScript 不进入 npack 或运行时。

待办（需新 ABI 原语或跨包依赖）：志继/若愚（觉醒技需 `add_skill` 引用 refresh 包
`reguanxing`/`rejijiang`，尚未移植）；溃诛/掣政/立军等孙亮系技能需新原语。
