# ADR 0314：ABI 453 current_phase_player + SIXIANG 陆通（荐简/仁政）

本批落地 C 段第四个 ABI（453）：`current_phase_player()` 表达式，并以此修正
SIXIANG「仁政」并闭环陆通（`std_dc_luotong`）。

## ABI 453：`current_phase_player()` 表达式

新查询表达式：求值当前回合角色。旧 JS 的 `_status.currentPhase` 在 294 个未移植技能
中出现，为高复用原语。

改动面（全部追加式）：
- rule-api `QueryRequest::CurrentPhasePlayer`（枚举末尾追加）
- engine-core query/mod.rs 分发臂：`state.turn.current.raw() as i32`
- rule-vm bytecode `query_tag::CURRENT_PHASE_PLAYER = 27` + vm.rs 分发
- content-compiler codegen 查询名 + typecheck 零参 + `required_rule_vm_abi` 453 门控
- query/cache.rs 规范化键（raw.push(27)，既有）
- `RULE_VM_ABI_VERSION` 452→453（ABI-COORDINATION.md 已领取 ABI-MAX 453）

## 技能移植

- 「仁政」（`stdrenzheng`，3020306305）：有伤害被防止时当前回合角色摸一张牌。
  既有移植误用 `event_player`（伤害目标）；本次以 `current_phase_player()` 修正为
  旧 JS 语义（当前回合角色）。
- 「荐简」（`stdjinjian`，4177959969）：造成伤害时（`damage_before owner_source`）或
  受到伤害时（`damage_before owner_is_target`）可各防止一次；以单一标记
  （0=未用，1=来源已用，2=目标已用，3=皆用）表达两类各限一次。

陆通（`std_dc_luotong`，3202675342，男/吴/3 体力）由上述两技能构成完整闭包，加入
`characters.json` 选将池；sixiang npack 重建为 69 技能 + 36 武将，ABI 453。
旧 JavaScript 不进入 npack 或运行时。
