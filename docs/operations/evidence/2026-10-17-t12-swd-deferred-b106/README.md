# T12 swd deferred 消解批 106：蛇魂（2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 445）——**142/184 技条目已映射**（deferred 41→40）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（shehun 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 蛇魂 shehun（山小小 `swd_shanxiaoxiao`） | `enable: "phaseUse"`, usable 1：弃 1–4 张花色互异的手牌并选一名有 he 牌角色：其弃等量 he 牌，若其中有与弃牌同花色者则其受 1 伤 | `SwdShehun`（tag 445）=主动技（`activate()` 门：手牌有牌；`once_per_turn`）：多步机 Act→SelectCard（弃牌）→TargetChoice（目标）→CardsAsk（目标弃 he 牌）→花色交集则 `emit(Damage)`。**登记差异**：花色互异约束未强制（任意 1–4 张） |

## 验证

- **定向回归** `swd_deferred_shehun_suit_match_damage`：花色匹配（0 号与 1 号弃同花色）→ 1 号受 1 伤（4→3）、双方弃牌离手；花色不匹配 → 无伤害。7 passed / 0 failed（[targeted.log](targeted.log)，含 b100–b105 回归）。
- **全量**：engine-native **888 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 442（tag 445）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算。

## 结论

- **142/184 映射，deferred 41→40**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 40 项）。
