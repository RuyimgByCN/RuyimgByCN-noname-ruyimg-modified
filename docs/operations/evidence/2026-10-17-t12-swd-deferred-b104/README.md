# T12 swd deferred 消解批 104：灵弦（2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 443）——**140/184 技条目已映射**（deferred 43→42）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（lingxian 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 灵弦 lingxian（慕月 `swd_muyue`） | `trigger: {player: ["respond", "useCard"]}`：非你回合时，你使用或打出牌后，与一名攻击距离大于 1 的角色各摸 1 | `SwdLingxian`（tag 443）=CardUsed 触发 + `extra_events [CardResponded]`；filter：牌由自己使用/打出 ∧ 非自己回合 ∧ 存在攻击距离 >1 的其他存活者；效果步取座位序首个符合者，双方 `draw_cards(1)`。**登记差异**：自选目标→座位序首个确定性近似；可放弃→恒发动近似 |

## 验证

- **定向回归** `swd_deferred_lingxian_draws_with_far_target`：1 号持灵弦；0 号出杀，1 号打出闪 → 1 号摸 1（净持平）、座位 3（距 1 号 2）摸 1、1 号未受伤。5 passed / 0 failed（[targeted.log](targeted.log)，含 b100–b103 回归）。
- **全量**：engine-native **886 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 440（tag 443）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算。

## 结论

- **140/184 映射，deferred 43→42**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 42 项）。
- 首次用 `extra_events` 合并 `CardResponded`（使用/打出双事件）。
