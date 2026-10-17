# swd deferred 消解批 133：此桥邪（`cyqiaoxie`）——失去装备后使用机关牌（v314 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **此桥邪**（`character/swd.js`）
- **规则/存档版本不变**（v314 / 存档 v98；无字段新增——复用批 125 的 `CardsLost` 原语与既有循环 stage）
- 上游回读：`character/swd.js`（`cyqiaoxie` 52 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:'loseEnd'}`，`frequent:true`；`filter`：`event.type!='use'` 且失去牌含装备 | `Cyqiaoxie`（tag 490）：`CardsLost` 触发（恒发动，无确认门）；过滤：来源区含 `Equip` |
| 循环次数 | `event.num` = 失去的装备张数；每次用一张后 `num--` 直至 0 或取消 | `self.cards` 作剩余次数队列；`ShuchuiTargetLoop` stage 逐次开窗（鼠槌同型循环） |
| 候选 | `get.inpile('jiguan')`（全局机关牌名）经 `hasUseTarget` 过滤后 `randomGets(3)` | 牌堆+弃牌堆中**可得**的机关牌名（各名取首张代表），按牌名规则检查可用性，**至多 3 张**（确定性取前 3=上游随机的近似） |
| 使用 | `chooseUseTarget(true, game.createCard(name))` | 实体从所在区入手 → `CardUsed` 帧（机关牌按名路由到既有机关牌机器） |
| 目标规则 | 各牌名 `filterTarget`（引擎 PlayCard 校验同口径） | 表：`jiguantong`=全部其他存活（座位序）；`gouhunluo`/`jiguanfeng`/`jiguanyuan`/`jiguanyaoshu`=恰 1 名其他（带各自近似前置）；`zhiluxiaohu`/`mujiaren`/`jiguanshu`=自目标（木甲人查每回合一次、机关术查自身有装备） |

包数据：`shahai-swd-upstream-noname` 中该将的 `cyqiaoxie` 由 `mapping:null` 改为 `mapping:"host:cyqiaoxie"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 20 → 19**（映射 171 → 172）。
- 台账口径（登记表）：**163/184 → 164/184 映射，deferred 19 → 18**。

## 验证

```
cargo test --offline -p engine-native --test swd_cyqiaoxie_jiguan
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（此桥邪） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：失去装备 → 开候选窗（非空）→ 选一张机关牌 → 实体入手并经 `CardUsed` 结算；三对照（手牌丢失不触发、未持技不触发、取消即止不再开窗） |
| engine-native 全量 | **979 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v314.log](replay-golden-v314.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 487（tag 490 追加） |

## 登记差异

- 候选来源为**当前牌堆/弃牌堆可得**的机关牌（上游 `lib.inpile` 全局牌名池）；候选取前 3（上游 `randomGets(3)` 随机）。
- 单目标机关牌的目标前置按近似（勾魂锣/机关怨查无标记、机关妖术查手牌>体力且装备<5；机关凤任意其他存活）；`hasUseTarget` 的其余全套校验未逐一建模。
