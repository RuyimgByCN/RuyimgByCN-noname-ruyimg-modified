# swd deferred 消解批 129：颂词（`songci`）——选未记录角色：弃 2 或摸 2（字段批 v313/存档 v97，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **颂词**（定义在 `character/sp.js:22323`——外部包技能但本仓有定义）
- **规则 v312→313、存档 v96→97**（`PlayerState` 追加 `songci_targets`）
- 上游回读：`character/sp.js`（`songci` 主技 + 组技 `songci_draw`）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:'phaseUse'`（无 `usable`：出牌阶段可重复发动） | `Songci`（tag 484）：出牌阶段主动面（`activate`，`turn.phase==Play && current==owner`，且存在未记录角色） |
| 目标 | `filterTarget`：未记录过的角色（**含自己**，上游未排除自身） | `SelectPlayers`（存活且不在 `songci_targets`，含自己，min1/max1，不可取消） |
| 分支① | `target.countCards('h') > target.hp` → `chooseToDiscard(2,'he',true)`（强制弃 2，手/装备） | `SelectCards`（目标手牌+装备，min=max=min(2, 可用数)，不可取消）→ 逐张弃置（手牌 `discard_from_hand`；装备撤技+入弃堆） |
| 分支② | 否则 `target.draw(2)` | `draw_cards(target, 2)` |
| 记录 | `storage.songci.push(target)` + `sortBySeat()` | `songci_record`（去重 + 升序=座位序） |
| 组技 `songci_draw` | `phaseDiscardEnd` forced：`!game.hasPlayer(!storage.songci.contains)` → `player.draw()` | 弃牌阶段**收束**内联臂（`commit_phase` Discard 收束）：持有者全部角色（**含阵亡者**，上游 `game.hasPlayer` 同口径）均已记录 → 摸 1 |

包数据：`shahai-swd-upstream-noname` 中该将的 `songci` 由 `mapping:null` 改为 `mapping:"host:songci"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 24 → 23**（映射 167 → 168）。
- 台账口径（登记表）：**159/184 → 160/184 映射，deferred 23 → 22**。

## 验证

```
cargo test --offline -p engine-native --test swd_songci_targets
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（颂词） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：摸牌分支（目标手牌 ≤ 体力 → 摸 2 且记录，二次发动目标窗排除已记录者）；弃牌分支（手牌 > 体力 → 强制弃 2，min=max=2）；组技（全员已记录 → 弃牌阶段末摸 1；对照未全员记录不摸）；对照（非出牌阶段拒绝）+ 存档往返 |
| engine-native 全量 | **964 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（313→312 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v313.log](replay-golden-v313.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 481（tag 484 追加） |

## 登记差异

- 强制弃置在可用牌不足 2 张时按「弃全部」处理（上游 `chooseToDiscard(2,'he',true)` 在不足时的行为未定义）。
- 组技 `songci_draw` 按上游 `game.hasPlayer` 口径**计入阵亡者**（阵亡且未记录时不触发）。
