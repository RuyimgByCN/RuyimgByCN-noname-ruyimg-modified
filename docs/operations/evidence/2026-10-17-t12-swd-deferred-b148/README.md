# swd deferred 消解批 148：神弓（`shengong`）——借用他人装备响应（v320/存档 v104 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **神弓**（`character/swd.js`）；新增**「借用响应」候选原语**（响应窗内联）
- **规则/存档版本不变**（v320 / 存档 v104；无字段变化——借用为响应窗候选 + 应答期取牌）
- 上游回读：`character/swd.js`（`shengong` 88 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:'chooseToRespondBegin'}`，`direct:true`；`filter`：需【闪】且他人有防具、或需【杀】且他人有武器 | 无触发时机（被动修饰）：响应窗构建时由 `shengong_borrow_options` 内联追加候选 |
| 候选 | 他人 `getEquip(2)`（防具，闪）/ `getEquip(1)`（武器，杀） | 其他存活角色装备区中**实际栏位**为防具/武器者（走 `card_slot_of`，兼容改造改写栏位） |
| 应答 | 借用牌作为响应打出（原牌离开持有者装备区） | 闪应答路径扩展：非本手牌/非本装备区 → `shengong_borrowed_owner` 定位所有者 → 从其装备区取走 + 撤技 + 入弃堆 + 日志 |
| 覆盖窗 | 全部响应窗（`chooseToRespondBegin`） | **闪响应窗**（杀闪避）、**AOE 响应窗**（按需求牌种）、**决斗响应杀窗**（3 处；其余响应窗登记差异） |

关键实现点：闪响应窗的「无闪直接命中」早退须**先纳入借用候选**（否则持有神弓但无闪者仍被判为无法响应）——已按此顺序构建。

包数据：`shahai-swd-upstream-noname` 中该将的 `shengong` 由 `mapping:null` 改为 `mapping:"host:shengong"`，npack 已 rehash。

## 计数

- 武将技能条目：**deferred 5 → 4**（映射 **177/184 → 178/184**）。

## 验证

```
cargo test --offline -p engine-native --test swd_shengong_borrow
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（神弓） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：被【杀】指定为目标且无闪 → 闪响应窗含**借用防具** → 选之闪避成功、防具从原持有者装备区取走并入弃牌堆；两对照（未持技无借用候选且照常命中、无借用来源照常命中） |
| engine-native 全量 | **1032 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（无版本变化）——[replay-golden-v320.log](replay-golden-v320.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 503（tag 506 追加） |

## 登记差异

- 覆盖窗为 3 处（闪/AOE/决斗）；濒死求桃等其余响应窗未纳入借用（上游 `chooseToRespondBegin` 为通用入口）。
- 上游以 `getEquip` 槽位判定；原生走 `card_slot_of`（含改造改写栏位）。
- 借用不校验原持有者意愿（上游同）；借用牌入弃牌堆（上游 `lose` 同口径）。
