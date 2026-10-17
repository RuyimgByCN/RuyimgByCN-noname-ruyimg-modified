# T11 Boss 庞统族批 118：涅槃 + 浴火 + 神威（字段批 v306/存档 v91，2026-10-17）

- 日期：2026-10-17
- 范围：Boss 庞统·涅槃凤雏（`boss_pangtong`）的**涅槃 / 浴火**与浴火授予的**神威**；天狱/栖梧已于 v277 落地
- **规则 v305→306、存档 v90→91**（`PlayerState` 追加 `niepan_used`；`BossStageState` 追加浴火关卡循环型 `loop_type`）
- 上游回读：`character/shenhua.js:5592`（`niepan`）、`character/shenhua.js:5652`（`oldniepan`，同效果异名）、`mode/boss.js:8530`（`boss_yuhuo`）、`mode/boss.js:8779`（`shenwei`）、`mode/boss.js:1305`（`boss_pangtong:['male','shen',4,['boss_tianyu','qiwu','niepan','boss_yuhuo'],…]`）

## 上游规则面与原生接线

| 技 | 上游 | 原生 |
|---|---|---|
| 涅槃 `niepan`（tag 471） | 限定技（`unique`+`limited`，`storage.niepan`）：**濒死时**（`event.type=='dying'` 且自己为濒死者）或**出牌阶段**（`event.parent.name=='phaseUse'`）发动 → 弃置全部牌 `getCards('hej')` → `link(false)` 解除横置 → `turnOver(false)` 翻回正面 → 摸 3 → `hp<3` 时回复至 3 | `HostBehavior::Niepan`：**濒死面**（`EventKind::Dying` 可选触发）+ **出牌阶段主动面**（`activate` 分支，限 `turn.phase==Play && current==owner`），两面共用 `niepan_effect`；限定位 `PlayerState.niepan_used` |
| 浴火 `boss_yuhuo`（tag 472） | `trigger {player:'niepanAfter'}` 锁定：`addSkill('kanpo'/'shenwei'/'zhuyu')` + `game.bossinfo.loopType = 1` + `_status.roundStart = game.boss` | 涅槃收束内**链式执行**（登记差异：上游独立 `niepanAfter` 事件面 → 原生链式）：三技按目录存在性授予（**神威**已落；看破/朱羽未转换 → 跳过）+ `BossStageState.loop_type = 1` |
| 神威 `shenwei`（tag 473） | 锁定：`phaseDrawBegin` `trigger.num += min(3, players-1)`；`mod.maxHandcard` `+min(3, players-1)` | kernel 两臂：摸牌阶段 `draw_cards(2 + min(3, 其他存活数))`；`hand_limit += min(3, 其他存活数)` |
| 目录 | `boss_pangtong` 四技（天狱/栖梧/涅槃/浴火） | `BOSS_GENERALS` 庞统条目扩为 `[天狱, 栖梧, 涅槃, 浴火]`；神威由浴火授予（不入武将技表，与上游一致） |

`oldniepan`（shenhua 同名异技，效果完全相同、仅名称不同）不另立目录项：Boss 条目引用的是 `niepan`；`oldniepan` 作为等价技能登记于规则基准。

## 验证

```
cargo test --offline -p engine-native --test t11_boss_pangtong_niepan
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native-wasm --lib
cargo test --offline -p game-server --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（涅槃族） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：出牌阶段发动（弃手/装/判 + 解横置 + 翻正 + 摸 3 + 回复至 3 + 限一次被拒 + 浴火授予神威与 loopType=1）；濒死自救与放弃阵亡对照；神威摸牌 2+3=5 与手牌上限 4+3=7（含无神威对照摸 2）；存档往返保持限定位与 loopType |
| engine-native 全量 | **918 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 / wasm / 服务端 / 客户端冒烟 | **66/0/0**（[deps-tests.log](deps-tests.log)）、**20/0**（[wasm-tests.log](wasm-tests.log)）、**49/0/0**（[game-server-tests.log](game-server-tests.log)）、**8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| 14 golden | 重录 + 版本回退探针（306→305 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v306.log](replay-golden-v306.log) |
| Gate 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 剩余（`T11-BOSS-PANGTONG-EXTERNAL` 关闭后的边界）

- **看破 `kanpo`**：上游 `offline.js:2448` 为转发桩（`get.info("sbkanpo").subSkill.kanpo`），实义在 sb 包；未转换 → 浴火授予按存在性跳过。
- **朱羽 `zhuyu`**：liuli 中仅 swd 包 `swd.js:7789` 的朱羽（横置伤害 +1），与庞统线授予的 `zhuyu` 同族但非同源；未转换 → 同上。
- **`ylygniepan`**（offline 浴火变体）：本仓库快照中不存在该技能定义（grep 全仓无命中）→ 登记为不可用外部依赖。
- **`_status.roundStart = game.boss`**：Boss 关卡轮次起点重置属 D4 关卡流程面（`BossStageState` 后续批），本批只落 `loopType`。
