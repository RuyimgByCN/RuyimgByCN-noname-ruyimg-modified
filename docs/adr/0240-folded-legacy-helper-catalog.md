# ADR 0240: 旧 helper 折叠覆盖

旧 JS 的 `group`/`global` helper 若全部权威行为已由父技能 DSL 承载，不再生成重复的运行时
技能。同步目录将该 helper 标为 `ported`，并用 `covered_by` 指向已迁移父技能。

映射必须逐项审计；同步校验要求 helper 与父技能都存在、父技能已迁移，且 helper 未被武将
直接引用。除首批 SHENHUA helper 外，已审计闭合 `langmie_damage`、`kongcheng1`、
`wushuang1/2`、`chunlao2`、`lihuo2`、`yanyu2`。

后续审计确认以下 helper 的全部权威行为也已由父技能 DSL/Host effect 承载：

- `beizhu_draw` → `beizhu`
- `shanjia2`、`hanyong3`、`fengpo2`、`qiangwu3` → 各自同名父技能
- `reshangshi_2nd`、`lihuo3`、`chunlao2_old`、`yonglve2`、`qiangzhi_draw`
  → 各自同名父技能
