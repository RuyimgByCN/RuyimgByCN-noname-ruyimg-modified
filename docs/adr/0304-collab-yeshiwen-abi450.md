# ADR 0304：collab 叶诗文（济溯/水泳）与 ABI 450 落地

本批为 collab 包叶诗文（`yeshiwen`）的完整移植，并落地 C 段首个 ABI（450）。

## ABI 450：`skip_judge_and_draw_phases`

新效果原语 `skip_judge_and_draw_phases(player)`：在判定阶段触发体内调用，一次性设置
`pending_judge_skips`（复用既有 `skip_next_judge_phase` 队列机制，turn.rs 判定入口消费）
与 `turn_flag::SKIP_DRAW`（复用既有旗标，turn.rs/ judge.rs 已读取）。

改动面（全部追加式，不改变既有枚举序数）：
- rule-api `EffectRequest::SkipJudgeAndDrawPhases { player, skill }`
- content-schema `StmtIr::EffectSkipJudgeAndDrawPhases`
- rule-vm bytecode 标签 340 + vm.rs 分发
- content-compiler typecheck/codegen + `required_rule_vm_abi` 450 门控
- engine-core primitives `skip_judge_and_draw_phases`（限定 Prepare/Judge 阶段与当前回合角色）
- hash.rs 标签 401（pending_judge_skips 与回合旗标本已在 canonical hash 内）
- `RULE_VM_ABI_VERSION` 405→450（已在 ABI-COORDINATION.md 领取 ABI-MAX 450）

## 技能移植

- 「济溯」（`clbjisu`，3764359885）：`phase_judge` 触发 + `skip_judge_and_draw_phases(self)`
  + `choose_use_virtual_sha(self, normal_distance)`。旧 JS 的 `phaseJudgeBefore` 取消判定+
  摸牌阶段由此等价表达；虚拟【杀】目标仍按攻击范围校验。
- 「水泳」（`clbshuiyong`，4055999596，此前已移植）：`modifier damage_taken_cancel(fire)`。

叶诗文（`yeshiwen`，4195936154，女/吴/3 体力）由上述两技能构成完整闭包，加入
`characters.json` 选将池；collab npack 重建为 30 技能 + 17 武将，ABI 450。
旧 JavaScript 不进入 npack 或运行时。
