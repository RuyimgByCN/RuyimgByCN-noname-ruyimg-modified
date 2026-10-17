# swd deferred 消解批 124：守印（`shouyin`）——濒死全员回复限定技（字段批 v311/存档 v96，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **守印**（慕月 `swd_muyue`，`character/swd.js`）
- **规则 v310→311、存档 v95→96**（`PlayerState` 追加 `shouyin_used`）
- 上游回读：`character/swd.js`（`shouyin` 63 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:'chooseToUse'` + `save:true`（濒死救助窗），`unique` + `awakenSkill`（限定技） | `SwdShouyin`（tag 479）：`Dying` 触发 + `optional:true`（确认门） |
| 条件 | `event.type=='dying'` 且 `!storage.shouyin` 且 `!player.isTurnedOver()` | 任意角色濒死、持有者未发动、未翻面 |
| ① 自翻面 | `player.turnOver()` | `ctx.emit(TurnOverBefore { player, cancelled: false })`（翻面提交帧；凶兽/神裔等取消技照常生效） |
| ② 全员回复 | 座位序逐人：`if(target.hp<target.maxHp) target.recover(target.maxHp-target.hp)` | `seat_order_from(owner)` 逐人：存活且 `hp < max_hp` 时 `recover(max_hp - hp)` |
| 限定位 | `player.storage.shouyin = true`（`awakenSkill`） | `PlayerState.shouyin_used` |

包数据：`shahai-swd-upstream-noname` 中 `swd_muyue.shouyin` 由 `mapping:null` 改为 `mapping:"host:shouyin"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 29 → 28**（映射 162 → 163）。
- 台账口径（登记表）：**154/184 → 155/184 映射，deferred 28 → 27**。

## 验证

```
cargo test --offline -p engine-native --test swd_shouyin_full_heal
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（守印） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：任意角色濒死开窗（观察者=持技者）→ 确认后自翻面、0/1/2 号全部回复至满、濒死被救回、限定位置；限一次（再次濒死不再开窗）；两对照（已翻面不开窗、未持技不开窗）；存档往返 |
| engine-native 全量 | **943 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（311→310 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v311.log](replay-golden-v311.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 476（tag 479 追加） |
