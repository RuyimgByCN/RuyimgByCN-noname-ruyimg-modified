# swd deferred 消解批 145：燎原（`liaoyuan`）——弃同花色牌换额外闪需求与加伤（字段批 v319/存档 v103，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **燎原**（`character/swd.js` 主技 + 组技 `liaoyuan2`/`liaoyuan3`）
- **规则 v318→319、存档 v102→103**（`TurnState` 追加 `liaoyuan_count`）
- 上游回读：`character/swd.js`（`liaoyuan` 58 行 + `liaoyuan2` 12 行 + `liaoyuan3` 7 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:'shaBegin'}`，`direct:true`（可选）；`filter`：杀有实体牌且自己持有同花色牌 | `Liaoyuan`（tag 502）：`CardUsed` 触发（杀类、自己使用、未取消）+ 可选确认门；过滤同口径（手/装备同花色） |
| 弃牌循环 | `chooseCard('he', 同花色)` 可取消；**首张必问**，弃后余 `>1` 张才续问 | 同一机器内循环：`Act` 开窗 → `SelectCard` 消费应答（弃置并计数）→ 回 `Act`；余 ≤1 即止 |
| 额外闪 | 目标需额外打出 `num` 张【闪】（`chooseToRespond` 循环） | Sha 机器 `needed = (无双?2:1) + turn.liaoyuan_count`（无双/燎原需求合并） |
| 直接命中 | 未能全打出 → `untrigger(); directHit=true`；`storage.liaoyuan=num` | 闪避数 < 燎原计数 → 走 `emit_sha_damage`（命中）而非 `ShaDodged`/miss |
| 加伤 | `liaoyuan2`：`source damageBegin` → `trigger.num += storage.liaoyuan`（杀伤害） | `emit_sha_damage` 统一 `+ turn.liaoyuan_count` 并清零（覆盖「无闪自动命中」与直命两条路径） |
| 清零 | `liaoyuan3`：`shaEnd` → `storage.liaoyuan = 0` | 杀需求满足路径与伤害路径均清零；`emit_sha_damage` 清零 |

包数据：`shahai-swd-upstream-noname` 中该将的 `liaoyuan` 由 `mapping:null` 改为 `mapping:"host:liaoyuan"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 8 → 7**（映射 183 → 184）。
- 台账口径（登记表）：**175/184 → 176/184 映射，deferred 7 → 6**。

## 验证

```
cargo test --offline -p engine-native --test swd_liaoyuan_extra_shan
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（燎原） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：弃 2 张同花色牌 → 目标无闪 → **直接命中且加伤 2**（1+2=3）；对照（放弃则正常 1 伤且不弃牌；无同花色牌不开窗）；`liaoyuan_count` 存档往返 |
| engine-native 全量 | **1025 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（319→318 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v319.log](replay-golden-v319.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 499（tag 502 追加） |

## 登记差异

- 无双与燎原的需求合并为 `needed`（上游为独立子技各自追加闪需求）。
- 加伤在 `emit_sha_damage` 统一计入（上游为 `damageBegin` 时机子技）；`liaoyuan_count` 以回合字段承载（上游 `storage`），杀结束/伤害时清零。
- 弃牌循环「余 >1 才续问」按上游 `countCards>1` 条件；AI（`next.ai`）未迁移。
