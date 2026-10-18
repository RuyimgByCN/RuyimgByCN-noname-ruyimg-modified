# swd deferred 消解批 149：昆仑镜（`kunlunjing`）——阶段收束快照 / 体力下降回滚（字段批 v321/存档 v105，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **昆仑镜**（`character/swd.js` 组技两面 `kunlunjing1`/`kunlunjing2` + 父技）
- **规则 v320→321、存档 v104→105**（`TurnState` 追加快照两字段）
- 上游回读：`character/swd.js`（`kunlunjing` 23 行 + `kunlunjing1` 110 行 + `kunlunjing2` 41 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 录（快照） | `kunlunjing2`：`trigger {player:'phaseAfter'}`，**silent/forced**——记录全场（手牌/装备/判定区，含 `viewAs`）与 `storage.kunlunjing2 = player.hp` | 同一行为 `Kunlunjing`（tag 507）的**阶段收束面**（`extra_events:[PhaseEnd]`，`optional:false`）→ 逐玩家记录 `(hand, equips, judge)` 与当前体力 |
| 回滚 | `kunlunjing1`：`trigger {player:'phaseBegin'}`，priority 10，**可选**——`player.hp < storage.kunlunjing2` 时回滚（`directgain` 拉回快照牌、`$equip`/判定区复位） | 同一行为的**阶段开始面**：过滤同口径（快照非空且体力低于快照值）→ **机器内确认门**（`ChoiceSpec::confirm("trigger_confirm")`）→ 确认后回滚 |
| 回滚实现 | `directgain`/`$equip`/判定区 `appendChild` | `kunlunjing_move_to`：把快照牌从**任意区**（任意玩家手牌/装备/判定区、桌面、牌堆、弃牌堆）移除后放回快照区；装备区变更同步**授予/撤销装备技** |

**可选性差异的处置**：上游两面可选性不同（录=锁定、回滚=可选），而 `TriggerMeta.optional` 为单一标记——故行为注册为锁定触发，回滚在**机器内**开确认门（登记差异）。

包数据：`shahai-swd-upstream-noname` 中该将的 `kunlunjing` 由 `mapping:null` 改为 `mapping:"host:kunlunjing"`，npack 已 rehash。

## 计数

- 武将技能条目：**deferred 4 → 3**（映射 **178/184 → 179/184**）。

## 验证

```
cargo test --offline -p engine-native --test swd_kunlunjing_rewind
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（昆仑镜） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：出牌阶段收束记录快照 → 破坏现场（弃手牌、拆装备入弃堆）+ 体力降到快照值以下 → 下一阶段开始确认回滚 → **手牌与装备回原位、装备离开弃牌堆**；对照（体力未降不出现回滚窗）；两字段存档往返 |
| engine-native 全量 | **1035 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（321→320 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 504（tag 507 追加） |

## 登记差异

- 回滚为「把快照牌拉回原位」的**增量回滚**（上游 `directgain` 同口径——快照后新得的牌留在原处，非完整状态倒带）。
- 快照未记录 `viewAs`（原生 `declared_kinds`/`declared_slots` 未纳入快照）。
- 回滚时机：上游 priority 10（阶段开始窗内优先）；原生按普通时机 + 机器内确认门。
- 上游 `check`（AI 是否回滚）未迁移。
