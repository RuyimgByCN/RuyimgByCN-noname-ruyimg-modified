# swd deferred 消解批 127：林韵（`linyun`）——两张牌当【杀】（v312 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **林韵**（司空玉 `swd_sikongyu`，`character/swd.js`）
- **规则/存档版本不变**（v312 / 存档 v96；无字段、无事件种类新增）
- 上游回读：`character/swd.js`（`linyun` 18 行 + 组技 `linyun2` 27 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:'chooseToUse'`，`filterCard:true`，`selectCard:2`，`position:'he'`，`viewAs:{name:'sha'}` | `SwdLinyun`（tag 482）：出牌阶段主动面（`activate` 分支，`turn.phase==Play && current==owner` 且手牌+装备 ≥2） |
| ① 选两张 | `selectCard:2`（手牌或装备） | `SelectCards`（手牌+装备，min2/max2，不可取消） |
| ② 选目标 | viewAs 杀的目标规则（射程/次数） | `SelectPlayers`（攻击距离内其他存活角色，min1/max1） |
| ③ 结算 | 两张牌为转化素材，视为【杀】 | 次张弃置（手牌走 `discard_from_hand`；装备走撤技+入弃堆），首张为载荷牌：装备则先入手，`declared_kinds` 声明为 `Sha` → `CardUsed` 帧（`card_use_as`）→ 杀结算（响应闪/伤害） |
| 组技 `linyun2` | `shaBegin`：目标须出闪，否则 `untrigger` + `directHit` | 原生杀结算天然「无闪即命中」；`directHit` 的**防具豁免**面为登记差异（原生照常先判防具） |

包数据：`shahai-swd-upstream-noname` 中 `swd_sikongyu.linyun` 由 `mapping:null` 改为 `mapping:"host:linyun"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 26 → 25**（映射 165 → 166）。
- 台账口径（登记表）：**157/184 → 158/184 映射，deferred 25 → 24**。

## 验证

```
cargo test --offline -p engine-native --test swd_linyun_two_cards_sha
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（林韵） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：两张手牌当杀命中（目标无闪受 1 伤、素材离手）；手牌+装备混合素材（装备离场、装备技撤销、照常命中）；两对照（不足两张不可发动、非出牌阶段不可发动） |
| engine-native 全量 | **956 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v312.log](replay-golden-v312.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 479（tag 482 追加） |

## 过程说明

- 转化杀经 `CardUsed` 子帧结算（丈八蛇矛同型）。发射后**置终态 stage**（复用既有 `ShuchuiTargetLoop` 作终态位）：子帧结算完本机再次恢复时直接收束，避免读取已被消费的应答（首版未置终态 → `answer present on resume` panic 实证）。
- 机器发射的转化杀**不计入每回合【杀】次数**（丈八/乱剑同款既有登记差异）。
