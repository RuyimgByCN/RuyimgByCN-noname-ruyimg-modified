# UP37-R01 涤仇：本回合移除历史、候选范围、全量清标记与伤害顺序（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树。本轮在**旧线 engine-core + 人工 DSL 已接原语**上实现 UP37-R01；原生线仍未注册涤仇，不能把旧线修复写成原生内容等价。当前优先级见[进度主稿](../../../architecture/native-progress.md#delivery-2026-10-24)，UP37 总账见[规则基准](../../../architecture/native-rules-baseline.md#upstream-2026-10-24)。

## 上游语义（`19d86519`）

结束阶段：先移除**所有**仇标记；候选包括当前有仇者与本回合先前已移除仇者；候选非空时至少选一人；对所选角色逐个结算 1 点火伤（座位序）；未选者的标记也已在选择前清除。

## 实现

- `GameState.turn.removed_marks: BTreeSet<(mark_def, player)>`：通用回合内「标记被移除」历史。`RemoveMark` 在计数从正数降到 0/负数时记录；新回合边界与 per-player 计数器一起清空；空集合不序列化、不进哈希（非空时增加 `turn-removed-marks/v1` 段）。见[差异](implementation.diff)。
- `resolve_dcdichou` action 10：候选 = 当前仇 + 本回合移除历史；选择前遍历全部存活玩家清除仇标记并清空该技能历史，把候选快照写入 choice payload；`min=1`、`max=候选数`。
- `resolve_dcdichou_choice` stage 10：只按 payload 校验所选者仍在场并按 `alive_by_seat` 顺序生成火伤；不再二次清标记，也不再把「当前仍有标记」当作选择有效条件。
- 人工 DSL 未改；不动 `ported/**`。

## 验证

| 检查 | 结果 | 证据 |
|---|---|---|
| 涤仇定向 | **3 passed / 0 failed**（含新增 R01 用例） | [日志](dcdichou-targeted.log)、[退出码](dcdichou-targeted.exit) |
| engine-core 完整 | **3780 passed / 0 failed / 1 ignored**（171 个 target） | [最终日志](engine-core-full-final.log)、[计数](engine-core-counts.json) |
| 新增用例 | 本回合已移除者仍可被选；选择前所有标记清空（含未选者）；空选被拒绝且不造成伤害；历史与挂起选择各自序列化/恢复后继续；所选者仅受 1 点火伤 | 同[定向日志](dcdichou-targeted.log) |

新增用例为 `original_xianding_v_guanyinping.rs::dcdichou_end_clears_all_marks_and_offers_history_candidate`。

## 版本、边界与未完成

- 旧线 `GameState` 无独立存档版本号；新字段 `serde(default)` + 空集合跳过序列化，旧 JSON 可读；非空历史的 hash 语义变化仅影响实际移除过标记的回合，属于本轮语义修正。未重录任何 golden 期望值。
- 历史只由通用 `RemoveMark` 记录；若有内容直接改写 `Player.marks` 绕过效果，需单独审计（当前检索未发现涤仇相关绕过）。
- 伤害顺序采用原生既有 `alive_by_seat`；未做「濒死嵌套时逐个等待」的多人时序对照测试，登记为剩余验证。
- **原生线没有涤仇注册**；本轮不等于 native/上游等价，也不改变 37 来源整包计数。R02–R10、N01–N07 仍未完成。
- 本轮未改规则/存档版本常量，未改包内容锁；`engine-native` 不受影响，但仍按主稿要求在全量批次重跑。
