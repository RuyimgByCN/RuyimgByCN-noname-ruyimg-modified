# swd 批 151：魔焰（`moyan`）——魔化前置技之二（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 范围：**魔化（`mohua2`）前置技** `moyan`（`character/swd.js`，55 行）——撒旦形态四技之二
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- 上游回读：`character/swd.js`（`moyan` 全量）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:'phaseUse'`，`usable:1`；`filter`：`hp<maxHp` 且有红色手牌 | `Moyan`（tag 510）：出牌阶段主动面（`activate`；体力未满 + 有红/方块手牌；`host_skill_def` 每回合一次） |
| 选牌 | `selectCard`：`[1, min(其他角色数, maxHp-hp)]`，`filterCard`：红色 | `SelectCards`（红桃/方块手牌，min1，max = `min(其他存活数, maxHp-hp)`，可取消） |
| 选目标 | `selectTarget`：目标数 = 已选牌数；`filterTarget`：非自己 | `SelectPlayers`（其他存活，min=max=选牌数，不可取消） |
| 效果 | 每个目标 `target.damage('fire')`（1 点火焰） | 逐个目标发 `Damage` 帧（火焰 1 点，`source=自己`）；**每帧后让出**以保持目标序（LIFO 反序规避） |
| 代价 | `phaseUse` 用牌消耗（弃所选牌） | 选牌后立即逐张 `discard_from_hand` |

包数据：包级声明 `moyan → host:moyan`（属魔化形态，随 `mohua2` 批次入将）；npack 已 rehash。

## 计数

- swd 包 deferred 计数不变（前置技落地）；台账口径仍 **179/184**。
- 登记项 `T12-SWD-MOHUA2-PREREQ`：`jifeng` ✅（批 150）、`moyan` ✅（本批）、`miedao`、`swd_xiuluo` 待落。

## 验证

```
cargo test --offline -p engine-native --test swd_moyan_multi_fire
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（魔焰） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：体力 2（上限 4）→ 选牌窗上限 2 → 弃 2 红牌 → 目标窗 min=max=2 → 两名目标各受 1 点火焰伤害且红牌离手；两对照（体力已满拒绝、无红牌拒绝、未持技拒绝） |
| engine-native 全量 | **1041 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（无版本变化）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 506（tag 510 追加） |

## 登记差异

- 目标数校验在机器内（原生无「目标数 = 选牌数」的声明式联动；已按同口径强制 min=max=选牌数）。
- 上游 `check`/`result`/`effect`（AI 与受害侧评估）未迁移。
- 伤害为无载体牌伤害（`card=None`；上游 `target.damage('fire')` 同口径）。
