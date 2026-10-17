# T12 swd deferred 消解批 103：幻音（2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 442）——**139/184 技条目已映射**（deferred 44→43）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（huanyin 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 幻音 huanyin（琴儿 `swd_qiner`） | `trigger: {target: "useCardToBefore"}`，forced，priority 5.9：他人对你使用牌时判定——**黑桃取消该牌**、**红桃摸 1**、其余无事 | `SwdHuanyin`（tag 442）=CardUsed 触发（filter：`u.player!=owner ∧ u.targets.contains(owner) ∧ !u.cancelled`，priority 6）；`machine()` 提取牌的使用者为关联目标、牌为关联卡；效果步 `draw_judge_card()` → 黑桃 `current_card_use_mut().cancelled=true`、红桃 `draw_cards(1)`、判定牌入弃堆 |

## 验证

- **定向回归** `swd_deferred_huanyin_judge_cancel_or_draw`：0 号对 1 号用铁索连环——判定黑桃 → 取消（1 号不横置）；判定红桃 → 1 号摸 1 且铁索照常结算（1 号横置）。4 passed / 0 failed（[targeted.log](targeted.log)，含 b100–b102 回归）。
- **全量**：engine-native **885 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 439（tag 442）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算。

## 结论

- **139/184 映射，deferred 44→43**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 43 项）。
- 首次在 HostBehavior 内用 `draw_judge_card()` 做独立判定 + `current_card_use_mut().cancelled` 取消使用。
