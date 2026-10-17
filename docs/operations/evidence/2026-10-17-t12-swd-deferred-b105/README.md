# T12 swd deferred 消解批 105：拼点（2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 444）——**141/184 技条目已映射**（deferred 42→41）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（nlianji 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 拼点 nlianji（独孤宁珂 `swd_duguningke`） | `enable: "phaseUse"`, usable 1：弃一张手牌，令两名角色拼点——赢者获得该牌、输者受赢者 1 伤 | `SwdNlianji`（tag 444）=主动技（`activate()` 门：手牌有牌；`once_per_turn`）：多步机 Act→SelectCard（弃出牌）→TargetChoice（目标0）→CardsAsk（目标1）→CompareTargetPick（目标0 选拼点牌）→ComparePlayerPick（目标1 拼点牌取座位序首个）→比大小：赢者 `give_hand_card` 获得弃出牌、输者 `emit(Damage)` 受 1 伤。**登记差异**：拼点牌两目标自选→确定性/座位序首个近似 |

## 验证

- **定向回归** `swd_deferred_nlianji_compare_reward_and_damage`：0 号弃桃；目标 1（rank13）与 2（rank1）拼点——1 号获得桃、2 号受 1 伤（4→3）、弃出牌离开 0 号手牌。6 passed / 0 failed（[targeted.log](targeted.log)，含 b100–b104 回归）。
- **全量**：engine-native **887 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 441（tag 444）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算。

## 结论

- **141/184 映射，deferred 42→41**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 41 项）。
- 首个多步主动技机器（Act→选择→拼点→结算，含 `Flow::Yield` 阶段链），复用 `HostStage` 既有槽位。
