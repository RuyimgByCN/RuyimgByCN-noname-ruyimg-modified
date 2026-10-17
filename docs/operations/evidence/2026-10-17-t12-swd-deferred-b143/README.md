# swd deferred 消解批 143：改造（`gaizao`）+ 装备栏改写原语（字段批 v318/存档 v102，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **改造**（`character/swd.js`）；新增**装备栏改写原语**（`declared_slots`）+ **装备前置窗**（`Equipped` BeforeTriggers）
- **规则 v317→318、存档 v101→102**
- 上游回读：`character/swd.js`（`gaizao` 56 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:'useCardToBegin'}`，`direct:true`（可选）；`filterx`：标准装备牌、非 nomod/unique、有 subtype 且该 subtype 已被自己占用 | `Gaizao`（tag 501）：**`Equipped` 前置窗**（`DamageBefore` 类 meta + `extra_events:[Equipped]`；`resolution` 新增 `EventKind::Equipped => BeforeTriggers` 与阶段链）+ 可选确认门；过滤：自己装备、固有栏位已被占用且存在空栏位 |
| 选择 | `chooseControl`：5 栏中**未被占用**者 + 取消 | `PickTag`（空栏位，可取消） |
| 改写 | `trigger.card.name = name+'_gaizao_'+slot`（复制原牌面 + 改 subtype + `epic`，**保留装备技能**） | `GameStateCore.declared_slots[card] = 新栏位`——牌种不变故**装备技能保留** ✓；`Catalog::card_slot_of(core, card)` 统一查询（覆盖优先），46 处「实例派生栏位」调用点改走该查询 |
| 入槽 | 按新 subtype 入槽 | `commit_equip` 按 `card_slot_of` 入槽，且**同栏位**替换（原按牌种——具名/泛型同栏不互替的缺陷一并修复） |

包数据：`shahai-swd-upstream-noname` 中该将的 `gaizao` 由 `mapping:null` 改为 `mapping:"host:gaizao"`，npack 已 rehash。

## 连带修正与冻结量同步（AGENTS.md 要求：先查原因与语义）

- `commit_equip` 的替换判定由「同牌种」改为「**同栏位**」：修复「具名武器与泛型武器同栏却不互相替换」的缺陷。
- `ai_planner::policy_v2_drives_all_seats_without_answer_routing_panic` 冻结步数 **218 → 189**（轨迹随装备替换修复变化；**胜方不变**，仍为 Player(3)）；已在测试内注明原因并指向本证据目录。
- 14 golden 全部重录（版本位移）+ 版本回退探针（318→317 全 14 场 mismatch）。

## 计数

- 武将技能条目：191 总数，**deferred 10 → 9**（映射 181 → 182）。
- 台账口径（登记表）：**173/184 → 174/184 映射，deferred 9 → 8**。

## 验证

```
cargo test --offline -p engine-native --test swd_gaizao_slot
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（改造） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：武器栏已占用 → 改造窗含空栏位（防具）→ 选防具 → 新武器入防具栏、旧武器保留（`card_slot_of` 返回防具）；对照（栏位空着不开窗、放弃则正常替换）；`declared_slots` 存档往返 |
| engine-native 全量 | **1019 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)）——含快照/存档边界用例（初版解码顺序错位致 7 项失败，已修复并复跑） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（318→317 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v318.log](replay-golden-v318.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 498（tag 501 追加） |

## 登记差异

- 上游为「复制牌面 + 改名 + 改 subtype + `epic`」生成新卡定义；原生以**栏位覆盖表**承载（不新增牌种定义，装备技能保留；牌名不变）。
- 上游 `filterx` 的 `nomod`/`unique`/`lib.inpile` 判定未建模（原生按「装备牌 + 固有栏位已占用 + 存在空栏位」判定）。
- 装备前置窗为新增帧阶段（`Equipped` BeforeTriggers）：无注册触发时行为不变，仅帧生命周期多一阶段。
