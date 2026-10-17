# swd deferred 消解批 130：天香（`tianxiang`）+ 天香·受（`tianxiang2`）（v313 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **天香**（定义在 `character/shenhua.js:6501`——外部包技能但本仓有定义）+ 其组技 `tianxiang2`
- **规则/存档版本不变**（v313 / 存档 v97；无字段新增——天香·受以**运行时技能实例**承载）
- 上游回读：`character/shenhua.js`（`tianxiang` 主技 + `tianxiang2` 组技）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:'damageBegin3'}`，`direct:true`，`filter`：有红桃手牌且 `num>0` | `Tianxiang`（tag 485）：`DamageBefore` 触发（自己为承受者、未防止、手中有红桃）+ 可选确认门 |
| ① 选牌 | `chooseCardTarget`：红桃手牌 + 目标（≠自己） | `SelectCards`（红桃手牌，min1/max1，可取消）→ `SelectPlayers`（其他存活，min1/max1，可取消）；两窗拆分（登记差异：上游为合并窗） |
| ② 转移 | `trigger.player = result.targets[0]`（伤害承受者改为目标） | 帧改写：`d.redirect_chain.push(d.target); d.target = target`（转移链记录） |
| ③ 弃牌 | `player.discard(result.cards[0])` | `discard_from_hand` |
| ④ 授予 | `trigger.player.addSkill('tianxiang2')` | 运行时插入技能实例（目录技能 `tianxiang2` 由包级 `skills` 声明 → `host:tianxiang2`） |
| 组技 | `tianxiang2`：`damageAfter`/`damageCancelled`/`damageZero` forced → 按失去体力摸牌 + 移除 | `Tianxiang2`（tag 486）：`Damage` 触发（自己为承受者、未防止）→ 按本次伤害量摸牌 + 移除实例 |

包数据：`shahai-swd-upstream-noname` 中该将的 `tianxiang` 由 `mapping:null` 改为 `mapping:"host:tianxiang"`，并新增包级技能声明 `tianxiang2 → host:tianxiang2`；npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 23 → 22**（映射 168 → 169）。
- 台账口径（登记表）：**160/184 → 161/184 映射，deferred 22 → 21**。

## 验证

```
cargo test --offline -p engine-native --test swd_tianxiang_redirect
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（天香） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：2 点伤害转移给 1 号（自己不受伤害、1 号受 2 并按失去体力摸 2、红桃离手、天香·受用后移除）；三对照（无红桃不开窗、放弃选牌不转移、未持技不开窗）；运行时授予实例存档往返保持 |
| engine-native 全量 | **968 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v313.log](replay-golden-v313.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 483（tag 485/486 追加） |

## 登记差异

- 选牌与选目标拆为两窗（上游 `chooseCardTarget` 合并窗）。
- 天香·受按**本次伤害量**摸牌（上游 `getDamagedHp()` 为实际失去体力——溢出致死等边界可能少于伤害量）。
