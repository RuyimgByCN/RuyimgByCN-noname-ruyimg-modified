# swd deferred 消解批 126：集控（`jikong`）——阶段开始/失去手牌后虚拟雷杀（v312 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **集控**（慕云 `swd_muyun`，`character/swd.js`）
- **规则/存档版本不变**（v312 / 存档 v96；无字段、无事件种类新增——复用批 125 的 `CardsLost` 原语）
- 上游回读：`character/swd.js`（`jikong` 37 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:['loseEnd','phaseBegin']}`，`direct:true`，`usable:1` | `SwdJikong`（tag 481）：`PhaseBegin` 触发 + `extra_events [CardsLost]`；`optional:true`（确认门） |
| 条件（阶段面） | `event.name=='phase'` | 自己阶段开始（任意阶段） |
| 条件（失去牌面） | 手牌为空且失去的牌含 `original=='h'` | `CardsLost{player==自己, origins 含 Hand}` 且手牌已空 |
| 效果 | `chooseTarget`（雷杀可用目标）→ `player.useCard({name:'sha',nature:'thunder'}, targets, false)` | `SelectPlayers`（攻击距离内其他存活角色，min1/max1，`allow_cancel`）→ `use_virtual_card(LeiSha)` |
| 次数 | `usable:1`；取消时 `storage.counttrigger.jikong--` **返还** | **使用时消费**：目标确定后用 `inst.once_used = true`；取消目标窗不消费（与上游返还同口径）；`reset_turn_limits` 每回合清零 |
| 目标合法性 | `lib.filter.targetEnabled({name:'sha',nature:'thunder'})`（含防具/技能禁选等全套） | 攻击距离内近似（登记差异：同既有虚拟杀口径） |

包数据：`shahai-swd-upstream-noname` 中 `swd_muyun.jikong` 由 `mapping:null` 改为 `mapping:"host:jikong"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 27 → 26**（映射 164 → 165）。
- 台账口径（登记表）：**156/184 → 157/184 映射，deferred 26 → 25**。

## 验证

```
cargo test --offline -p engine-native --test swd_jikong_thunder_sha
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（集控） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：阶段面确认 → 选目标 → 虚拟雷杀 1 点雷伤、同回合不再开窗；失去手牌面（仍有手牌不开窗、失去最后一张开窗并命中）；取消目标窗后同回合仍可发动（次数返还）；未持技不开窗 |
| engine-native 全量 | **952 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v312.log](replay-golden-v312.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 478（tag 481 追加） |

## 过程说明

- 测试工具 `debug_discard_card` 改为**发射 `CardsLost` 事件**（与 `debug_take_owned_card` 同口径：克隆状态 → 变更 → 压帧 → `finish_segment`），否则失去牌触发面无法在测试中驱动。
- 「每回合限一次」刻意**不用** `once_per_turn` 自动消费（确认即消费会吞掉上游的取消返还语义），改为使用时手动置位。
