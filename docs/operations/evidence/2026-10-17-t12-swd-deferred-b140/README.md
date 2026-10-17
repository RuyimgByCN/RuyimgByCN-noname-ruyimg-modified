# swd deferred 消解批 140：移花（`yihua`）——单目标牌反弹（v316 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **移花**（`character/swd.js`）；复用批 139 的**帧写回原语**
- **规则/存档版本不变**（v316 / 存档 v100；无字段新增——写回 `CardUseData.player/targets`）
- 上游回读：`character/swd.js`（`yihua` 46 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {target:'useCardToBefore'}`，`direct:true`（可选）；`filter`：`event.targets.length==1`、非自己使用、手牌 ≥2 | `Yihua`（tag 498）：`CardUsed` 触发（AfterTriggers 结算窗）+ 可选确认门；过滤同口径（`targets.len()==1 && targets[0]==owner && player!=owner && hand.len()>=2`） |
| 弃牌 | `chooseToDiscard(get.prompt(...), 2)`（**可取消**） | `SelectCards`（自己手牌，min2/max2，可取消） |
| 反弹 | `trigger.target=trigger.player; trigger.player=player`（使用者/目标对调）+ `untrigger/trigger` 重放 | 写回帧：`u.player = owner; u.targets = vec![原使用者]`——Intrinsic 阶段才构造效果机器，故写回即生效（原使用者成为承受者） |

包数据：`shahai-swd-upstream-noname` 中该将的 `yihua` 由 `mapping:null` 改为 `mapping:"host:yihua"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 13 → 12**（映射 178 → 179）。
- 台账口径（登记表）：**170/184 → 171/184 映射，deferred 12 → 11**。

## 验证

```
cargo test --offline -p engine-native --test swd_yihua_bounce
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（移花） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：被 0 号的杀指定为目标 → 确认 → 弃 2 张手牌 → **反弹**（0 号受伤、1 号无伤）；三对照（放弃照常受伤且手牌未弃、手牌不足 2 张不开窗、未持技不开窗） |
| engine-native 全量 | **1007 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v316.log](replay-golden-v316.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 495（tag 498 追加） |

## 登记差异

- 上游 `event.addedTargets` 的「已追加目标不再触发」标记未建模：被反弹方若同样持有移花可再次反弹（每次消耗 2 张手牌，必然终止）。
- 上游 `untrigger/trigger('useCardToBefore')` 的事件重放以帧写回等价（重放语义面未建模）。
- AI 面（`next.ai`）未迁移。
