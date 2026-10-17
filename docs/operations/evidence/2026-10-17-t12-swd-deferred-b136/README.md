# swd deferred 消解批 136：极略（`jilve`）+ 通用目标规则表（字段批 v315/存档 v99，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **极略**（`character/swd.js`）+ 新原语**通用目标规则表** `GameStateCore::legal_targets`
- **规则 v314→315、存档 v98→99**（`PlayerState` 追加 `jilve_used`）
- 上游回读：`character/swd.js`（`jilve` 56 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:'phaseUse'`，`usable:3` | `Jilve`（tag 494）：出牌阶段主动面（`activate`；`jilve_used < 3` 且牌堆非空；出牌阶段开始清零） |
| ① 看牌 | 取牌堆顶 3 张（`ui.cardPile.childNodes`）建对话框 | 候选＝牌堆顶 3 张（`deck.iter().rev().take(3)`）中以 `Card` 选项呈现 |
| ② 过滤 | 非装备（基本牌/锦囊） | `is_basic_kind()` 或 `equip_slot().is_none()`，且**通用目标规则表**可给出目标（`min==0` 或目标非空） |
| ③ 使用 | `chooseButton` → `backup.viewAs = links[0]`（该牌本身）→ 目标按该牌规则 | 牌从牌堆移入手牌 → `CardUsed` 帧（按牌种路由既有机器）→ `jilve_used += 1` |
| ④ 目标 | 由该牌的 `filterTarget` 决定 | **新原语** `GameStateCore::legal_targets(catalog, kind, player) -> (min, max, 目标)`：杀族=射程内其他；桃=已受伤自己；酒/闪电=自己；无中=无目标；五谷/桃园=全体存活；南蛮/万箭=全体其他；决斗=任一其他；顺手/拆桥=距离内且有牌的其他；铁索=1–2 名存活；乐/兵=无同名牌的其他；借刀=有武器的其他 |

包数据：`shahai-swd-upstream-noname` 中该将的 `jilve` 由 `mapping:null` 改为 `mapping:"host:jilve"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 17 → 16**（映射 174 → 175）。
- 台账口径（登记表）：**166/184 → 167/184 映射，deferred 16 → 15**。

## 验证

```
cargo test --offline -p engine-native --test swd_jilve_top3
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（极略） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：牌堆顶的【杀】入选 → 目标窗（min=max=1）→ 命中掉血、牌离牌堆、`jilve_used=1`；对照（非出牌阶段拒绝、满 3 次拒绝、未持技拒绝、候选不含装备牌）；通用目标规则表 5 组断言 |
| engine-native 全量 | **990 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（315→314 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v315.log](replay-golden-v315.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 491（tag 494 追加） |

## 登记差异

- 通用目标规则表是 PlayCard 校验的**近似镜像**：忽略逐技豁免（谦逊/帷幕/守翼/逐月/疾风等范围与目标免疫）与护甲/装备细节；命令路径仍走原校验，仅机器发起「非手牌用牌」时使用。
- 极略候选为牌堆顶 3 张中**目标规则可满足**者（上游按对话框逐张 `filterTarget` 判定，等价但候选集可能更小）；使用后牌从牌堆入手再结算（上游同经 `viewAs` 用牌）。
