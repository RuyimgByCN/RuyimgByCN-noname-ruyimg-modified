# swd deferred 消解批 121：神焰（`shenyan`）——火伤 +1 + 邻位蔓延（字段批 v308/存档 v93，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **神焰**（夜鸦熙 `swd_yeyaxi`，`character/swd.js`）
- **规则 v307→308、存档 v92→93**（`PlayerState` 追加 `shenyan_used`/`shenyan_pending`）
- 上游回读：`character/swd.js`（`shenyan` 51 行 + `shenyan2` 17 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发（前置） | `trigger {source:'damageBegin'}`，`filter: !storage.shenyan && event.nature=='fire'` | `SwdShenyan`（tag 476）：`DamageBefore` 面——`d.source == owner && !prevented && nature == Fire && !shenyan_used` |
| 前置效果 | `trigger.num++`；`awakenSkill`（限定技）；`storage.shenyan2` = 原目标距离 ≤1 的其他角色（座位序，不含自己与原目标） | `current_damage_mut().amount += 1`；`shenyan_used = true`；`shenyan_pending` = `seat_order_from(target)` 中距离 ≤1、存活、非来源非目标者（排序后） |
| 触发（结算） | `shenyan2`：`trigger {global:'damageAfter'}`，`forced`——逐个 `storage.shenyan2.shift()` 后 `target.damage('fire')`，随后移除技能 | 同一行为 `Damage`（extra_events）面：`shenyan_used && !shenyan_pending.is_empty()` 时逐个 `ctx.emit(Damage{source:None, num:1, nature:Fire})`（座位序）并清队列 |
| 伤害来源 | `target.damage('fire')` 未给来源 → 无来源 | `damage_data_with(None, …)`（登记差异：无来源同口径） |
| 两面区分 | 上游两个技能（`shenyan` / `shenyan2`）分挂前后事件 | 原生**同行为两面**：按状态区分（未发动→前置面；已发动且有队列→结算面）；同一伤害事件的两个触发阶段各入队一次机器 |

包数据：`shahai-swd-upstream-noname` 中 `swd_yeyaxi.shenyan` 由 `mapping:null` 改为 `mapping:"host:shenyan"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 32 → 31**（映射 159 → 160）。
- 台账口径（登记表）：**151/184 → 152/184 映射，deferred 31 → 30**。

## 验证

```
cargo test --offline -p engine-native --test swd_shenyan_spread
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（神焰） | **5 passed / 0 failed**（[targeted.log](targeted.log)）：火伤 1→2 且邻位（4 人环，排除来源与原目标后仅 2 号）蔓延 1 点、非邻位与来源不受、队列清空、限定位置；对照（非火伤不加成不蔓延、他人火伤不触发）；限定技第二次不触发；存档往返保持限定位与队列 |
| engine-native 全量 | **931 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（308→307 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v308.log](replay-golden-v308.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 473（tag 476 追加） |
