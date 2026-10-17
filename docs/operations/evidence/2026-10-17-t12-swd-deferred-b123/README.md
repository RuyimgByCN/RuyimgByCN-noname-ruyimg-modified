# swd deferred 消解批 123：义山（`yishan`）——受伤取回最近失去的牌（字段批 v310/存档 v95，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **义山**（`swd_septem`，`character/swd.js`）
- **规则 v309→310、存档 v94→95**（`PlayerState` 追加 `yishan_lost`）
- 上游回读：`character/swd.js`（`yishan` 85 行 + `yishan2` 10 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 记录（`yishan2`） | `trigger {player:'loseEnd'}`，`silent`：`storage.yishan.unshift(card)`（新在前） | `machine::note_cards_lost`（自由函数，Ctx 与测试工具共用）：持技者失去牌时 `yishan_lost.insert(0, card)`，表长上限 16；**记录点**=弃置手牌 / 交出手牌 / 装备离场（替换）三类原生丢失路径（登记差异：其余丢失路径如移出游戏不记） |
| 触发 | `trigger {player:'damageEnd'}`，`filter`：存在「不属于自己且不在特殊区」的记录牌 | `SwdYishan`（tag 478）：`Damage` 触发（自己受伤、未防止）且存在可回收记录牌；`optional:true`（上游 `notemp` 直发，原生可选窗，登记差异） |
| 效果 | 先剔除自己持有/在特殊区者；`splice(0,2)` 取最近 2 张 `player.gain(cards)` | 依记录序（新在前）取至多 2 张可回收牌，从所在区（**弃牌堆/桌面/他人手牌·装备·判定区**）取出并入手；已取回者从记录表移除 |
| 装备区取回 | 上游 `get.position` 不区分装备；取回后随 `gain` 入手 | 从他人装备区取回时同步 `revoke_equip_skills`（装备技撤销管线） |

包数据：`shahai-swd-upstream-noname` 中 `swd_septem.yishan` 由 `mapping:null` 改为 `mapping:"host:yishan"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 30 → 29**（映射 161 → 162）。
- 台账口径（登记表）：**153/184 → 154/184 映射，deferred 29 → 28**。

## 验证

```
cargo test --offline -p engine-native --test swd_yishan_recover
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（义山） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：弃 3 张后记录新在前、受伤取回最近 2 张且第三张不取回、已取回者出表；对照（无可回收牌不开窗、未持技不记录）；跨区取回他人手牌 + 存档往返 |
| engine-native 全量 | **939 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（310→309 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v310.log](replay-golden-v310.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 475（tag 478 追加） |
