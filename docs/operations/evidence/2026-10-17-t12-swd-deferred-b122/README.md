# swd deferred 消解批 122：入梦（`rumeng`）——他人出牌阶段前弃牌/跳阶段（字段批 v309/存档 v94，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **入梦**（海大胖 `swd_haidapang`，`character/swd.js`）
- **规则 v308→309、存档 v93→94**（`PlayerState` 追加 `skip_discard_phase`）
- 上游回读：`character/swd.js`（`rumeng` 39 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {global:'phaseUseBefore'}`，`direct:true` | `SwdRumeng`（tag 477）：`PhaseBegin(Play)` 可选触发（全局=任意角色出牌阶段开始） |
| 条件 | `event.player != player && player.countCards('he',{type:'basic'}) < player.countCards('he')` | 触发帧的回合角色 ≠ 持有者，且持有者手牌+装备中**存在非基本牌** |
| ① 弃非基本牌 | `player.chooseToDiscard(card => get.type(card)!='basic', prompt, 'he')`（可取消） | `SelectCards`（持有者手/装备中的非基本牌，min1/max1，`allow_cancel`）；取消即结束 |
| ② 目标二选一 | `trigger.player.chooseToDiscard({type:'basic'}, '入梦：弃置一张基本牌或跳过出牌及弃牌阶段')`（可取消） | `SelectCards`（回合角色的**手牌**基本牌，min1/max1，`allow_cancel`；无可选项即等同取消） |
| ③a 弃基本牌 | 弃置后无事发生 | `discard_from_hand`，阶段照常 |
| ③b 取消 | `trigger.cancel()`（取消出牌阶段）+ `trigger.player.skip('phaseDiscard')` | 置 `skip_discard_phase = true` + 立即发射该出牌阶段的**收束帧**（原生无 phaseBefore 取消位，登记差异）；弃牌阶段开始处检查该位并跳过（一次性、用后清零） |

包数据：`shahai-swd-upstream-noname` 中 `swd_haidapang.rumeng` 由 `mapping:null` 改为 `mapping:"host:rumeng"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 31 → 30**（映射 160 → 161）。
- 台账口径（登记表）：**152/184 → 153/184 映射，deferred 30 → 29**。

## 验证

```
cargo test --offline -p engine-native --test swd_rumeng_phase_skip
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（入梦） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：跳过路径（弃非基本牌 → 目标取消 → 出牌阶段收束且弃牌阶段跳过、跳位用后清零、目标手牌不变）；弃基本牌路径（阶段照常、手牌 -1）；两对照（无非基本牌不开窗；跳位一次性 + 存档往返） |
| engine-native 全量 | **935 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（309→308 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v309.log](replay-golden-v309.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 474（tag 477 追加） |
