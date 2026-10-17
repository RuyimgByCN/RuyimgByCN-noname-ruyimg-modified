# swd deferred 消解批 134：冰封（`bingfeng`）+ 冰封·封（`bingfeng2`）—— 翻面封锁限定技（v314 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **冰封**（`character/swd.js`）+ 其授予技 `bingfeng2`；新增**出牌封锁原语** `SkillBehavior::blocks_all_cards`
- **规则/存档版本不变**（v314 / 存档 v98；无字段新增——觉醒以实例禁用承载，封锁以运行时实例 + 查询钩承载）
- 上游回读：`character/swd.js`（`bingfeng` 82 行 + `bingfeng2` 24 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:'phaseUse'`，`unique` + `awakenSkill`（限定技，`!storage.bingfeng`） | `Bingfeng`（tag 491）：出牌阶段主动面（`activate`，Play + current==owner + 实例启用 + 存在未翻面其他角色） |
| 目标 | `selectTarget [1,3]`，`filterTarget`：其他且未翻面 | `SelectPlayers`（其他未翻面存活，min1/max=min(可用,3)） |
| 觉醒/玄咒 | `awakenSkill` + `removeSkill('xuanzhou')` | 实例 `enabled = false`（无字段）+ 目录查 `xuanzhou` 存在则移除 |
| 失上限 | `player.loseMaxHp()` | `lose_max_hp(owner, 1)` |
| 翻面 | `player.turnOver()` + 各目标 `turnOver()` | `TurnOverBefore{ cancelled:false }` 提交帧（自己 + 各目标；凶兽/神裔/隐月·判等取消技照常生效） |
| 授予 | `addSkill('bingfeng2')`（自己 + 各目标） | 运行时插入实例（包级 `skills` 声明 `bingfeng2 → host:bingfeng2`） |
| 封锁 | `bingfeng2` mod：`cardEnabled`/`cardUsable`/`cardRespondable`/`cardSavable` 全 false | 新钩 `SkillBehavior::blocks_all_cards` + `Catalog::blocks_all_cards(core, p)`：① `PlayCard` 校验拒绝（新 `RejectReason::CardBlocked`）② 闪响应窗无可选项（必中）③ 濒死救援（桃/酒）无可选项 ④ 决斗响应杀 ⑤ AOE 响应——**其余响应窗（改判等）登记差异** |
| 自移除 | `trigger {player:'turnOverAfter'}` forced，`!isTurnedOver()` → `removeSkill('bingfeng2')` | `Bingfeng2`（tag 492）：`TurnOvered` 触发（自己、已翻回正面）→ 移除实例 |

包数据：`shahai-swd-upstream-noname` 中该将的 `bingfeng` 由 `mapping:null` 改为 `mapping:"host:bingfeng"`，并新增包级技能声明 `bingfeng2 → host:bingfeng2`；npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 19 → 18**（映射 172 → 173）。
- 台账口径（登记表）：**164/184 → 165/184 映射，deferred 18 → 17**。

## 验证

```
cargo test --offline -p engine-native --test swd_bingfeng_lock
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（冰封） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：翻面自己与两目标（未选目标不翻）、失去 1 体力上限、授予封锁（自己+两目标、第三名无）、封锁者 `PlayCard` 被拒（`CardBlocked`）、限定技二次被拒；封锁者持闪不能闪避（必中、闪留手）；对照未封锁者闪避成功；翻回正面后封锁自移除 |
| engine-native 全量 | **983 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v314.log](replay-golden-v314.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)）——同批补 `n09_runner` 新拒绝因由映射 |
| 键表守护 | `HostBehavior` 489（tag 491/492 追加） |

## 登记差异

- 封锁覆盖面：出牌 + 闪响应 + 濒死救援 + 决斗响应杀 + AOE 响应五处；改判（鬼道/鬼才）等其余牌类窗口未逐一封锁。
- `bingfeng2` 的 `cardSavable`（求桃）面以濒死救援窗无选项承载（含自救与救人）。
