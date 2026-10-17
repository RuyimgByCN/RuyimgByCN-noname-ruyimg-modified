# swd deferred 消解批 141：李香龙（`lxianglong`）+ `ShaDodged` 事件原语（v316/存档 v100 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **李香龙**（`character/swd.js`）；新增事件种 **`ShaDodged`（杀被闪避）**
- **规则/存档版本不变**（v316 / 存档 v100；无字段变化）——但**事件日志面**新增事件种，`sha_dodge_then_damage` 一场 golden 重录（见下）
- 上游回读：`character/swd.js`（`lxianglong` 26 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {target:'shaMiss'}`，`priority:5`，`direct:true`（可选） | `Lxianglong`（tag 499）：`ShaDodged` 触发（自己是该杀使用者、闪避者存活）+ 可选确认门 |
| 效果 | `chooseTarget(canUse('sha'))` → `useCard({name:'sha'}, trigger.cards, targets)`（虚拟杀） | `SelectPlayers`（攻击范围内其他存活，min1/max1，可取消）→ `use_virtual_card(Sha, [target])`（既有虚拟杀管线） |

**新事件种 `ShaDodged { card, source, target }`**（九处接线：event.rs 种类+载荷、archive.rs 种类码 21/判别 19、engine.rs 哈希判别 16、snap.rs 边界校验、sha 机器发射点）——发射点位于闪避分支末尾（青龙偃月刀/贯石斧等窗口之后，登记差异：窗口次序），上游 `shaMiss` 等价接入。

包数据：`shahai-swd-upstream-noname` 中该将的 `lxianglong` 由 `mapping:null` 改为 `mapping:"host:lxianglong"`，npack 已 rehash。

## golden 位移分析（AGENTS.md 要求：先查原因与语义再重录）

- 位移场景：**仅 `sha_dodge_then_damage` 一场**（actual `1524674f…`，原 `e6dc3757…`）；其余 **13 场零位移**。
- 原因：该场景包含【杀】被闪避路径，事件日志新增 `ShaDodged` 事件；golden 覆盖「状态+事件日志」摘要。
- 语义结论：**状态语义未变**（13 场零位移 + 该场景状态轨迹一致），位移仅在日志面；无字段/存档变化，规则版本保持 v316。
- 处理：重录该场并在 `replay_scenarios.rs` 表头登记重录依据。

## 计数

- 武将技能条目：191 总数，**deferred 12 → 11**（映射 179 → 180）。
- 台账口径（登记表）：**171/184 → 172/184 映射，deferred 11 → 10**。

## 验证

```
cargo test --offline -p engine-native --test swd_lxianglong_sha_miss
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（李香龙） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：0 号的杀被 1 号闪避 → 确认 → 选射程内 3 号 → 虚拟杀命中（3 号 -1）；三对照（杀命中不开窗、放弃不追加杀、未持技不开窗） |
| engine-native 全量 | **1011 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `sha_dodge_then_damage` 单场重录（原因/语义见上）+ 其余 13 场零位移——[replay-golden-v316.log](replay-golden-v316.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 496（tag 499 追加） |

## 登记差异

- `shaMiss` 窗口次序：原生在青龙偃月刀/贯石斧等窗口之后发射（上游按同点优先级排序）。
- 追加的杀为**虚拟杀**（无实体载体牌；上游以原杀之牌 `trigger.cards` 作载体——登记差异）。
- 上游 `priority:5` 的时序优先级未建模（原生按普通时机）。
