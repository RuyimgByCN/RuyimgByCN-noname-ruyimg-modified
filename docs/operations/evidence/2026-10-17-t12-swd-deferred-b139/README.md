# swd deferred 消解批 139：贞烈（`zhenlie`）——成为杀/锦囊目标时失体力并移出目标（v316 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **贞烈**（`character/yijiang.js:11678`——外部包技能但本仓有定义）；新增**「移出目标」写回原语**
- **规则/存档版本不变**（v316 / 存档 v100；无字段新增——写回 `CardUseData.targets`）
- 上游回读：`character/yijiang.js`（`zhenlie` 45 行）

## 上游语义与原生接线

| 步 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {target:'useCardToTargeted'}`，`direct:true`（可选）；`filter`：他人使用【杀】或锦囊且自己为目标 | `Zhenlie`（tag 497）：`CardUsed` 触发（AfterTriggers 结算窗）+ 可选确认门；过滤：非自己使用、未取消、`targets` 含自己且牌种为杀类或锦囊 |
| ① 失体力 | `player.loseHp()` | `LoseHp` 帧（1 点） |
| ② 移出目标 | `trigger.getParent().excluded.add(player)` | **写回 `CardUseData.targets`**（`retain != owner`）——原生 Intrinsic 阶段才构造效果机器，故 AfterTriggers 写回即生效；目标清空时「恰一名目标」牌种不再结算（新护栏，避免机器对空目标越界） |
| ③ 弃牌 | `player.discardPlayerCard(trigger.player,'he',true)` | `SelectCards`（使用者手牌+装备，min1/max1，不可取消）→ 手牌 `discard_from_hand` / 装备撤技后入弃堆 |

包数据：`shahai-swd-upstream-noname` 中该将的 `zhenlie` 由 `mapping:null` 改为 `mapping:"host:zhenlie"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 14 → 13**（映射 177 → 178）。
- 台账口径（登记表）：**169/184 → 170/184 映射，deferred 13 → 12**。

## 验证

```
cargo test --offline -p engine-native --test swd_zhenlie_target_exclude
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（贞烈） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：被杀指定为目标 → 确认 → 失 1 体力 + **未受杀伤害**（已移出目标）+ 使用者被弃置一张牌；三对照（放弃照常受伤、非目标不开窗、未持技不开窗） |
| engine-native 全量 | **1003 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)）——空目标护栏按「恰一名目标」牌种收窄后回归全绿（初版宽护栏曾致 30 项群体/无目标牌用例失败，已修正并复跑） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v316.log](replay-golden-v316.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 494（tag 497 追加） |

## 登记差异

- 「移出目标」以写回 `CardUseData.targets` 承载（上游 `excluded` 集合另有「不可再被指定」的语义面，未建模）。
- 目标清空后仅「恰一名目标」牌种跳过结算（杀族/决斗/拆顺/乐兵/借刀/铁索）；群体与无目标牌种照常（其机器自行处理空列表）。
- 上游 `check`（AI 是否发动）未迁移（无 AI 面）。
