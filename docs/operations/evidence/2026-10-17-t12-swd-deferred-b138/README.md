# swd deferred 消解批 138：天伦（`tianlun`）——判定区牌替换本次判定（v316 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **天伦**（`character/swd.js`）；扩展**判定机改判窗**（判定区来源）
- **规则/存档版本不变**（v316 / 存档 v100；无字段新增——判定机进行态承载）
- 上游回读：`character/swd.js`（`tianlun` 55 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {global:'judge'}`，`direct:true`（可选；`filter` 亦允许仅判定区有牌时发动） | 判定机改判队列：持有 `tianlun` 者入队（与鬼才/鬼道/天道/改天道并列），在改判窗询问 |
| 候选 | 本次判定牌 + 全场判定区牌 | 当前判定牌（`judge_held`）+ 各玩家 `judge` 区牌 + **判定机待结算队列**（原生在判定阶段开始即把判定区整体取入队列；排除正在结算的延时锦囊） |
| 效果 | 选中牌成为判定牌，**旧判定牌弃置**（`judging[0].discard()`） | 选中牌从所在判定区/队列移除 → 旧判定牌 `to_discard` → `judge_card = 选中牌` + `JudgementRewritten` 日志（**不**把旧牌交给改判者——与鬼道/鬼才路径不同） |
| 放弃 | `chooseButton` 可取消 | `ChoiceOption::Pass`（候选含 Pass） |

包数据：`shahai-swd-upstream-noname` 中该将的 `tianlun` 由 `mapping:null` 改为 `mapping:"host:tianlun"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 15 → 14**（映射 176 → 177）。
- 台账口径（登记表）：**168/184 → 169/184 映射，deferred 14 → 13**。

## 验证

```
cargo test --offline -p engine-native --test swd_tianlun_judge
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（天伦） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：判定窗候选含判定区牌（兵粮寸断）→ 选中后成为判定牌、**旧判定牌弃置**；对照（放弃改判不消费判定区牌；未持技不出现天伦窗） |
| engine-native 全量 | **999 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v316.log](replay-golden-v316.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 493（tag 496 追加） |

## 登记差异

- 原生判定阶段开始即把判定区整体取入判定机队列（上游判定区牌留在区内直到结算），故天伦候选同时纳入「队列中的待结算判定区牌」；非延时锦囊牌放入判定区会被当作延时锦囊结算（原生限制，登记）。
- 同持天伦与鬼道/鬼才时，改判窗按天伦候选集询问（上游为多次独立询问；登记差异）。
- 上游 `filter` 允许「仅判定区有牌、无判定牌」时发动；原生改判窗只在判定进行中出现（登记差异）。
