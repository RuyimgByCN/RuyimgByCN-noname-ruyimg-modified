# L27 选将美化 批 3：每步（`new_meibu`）+ 距离视为 1（`new_meibu_range`）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L27 选将美化**（`extension/选将美化/extension.js`）
- 本批：`new_meibu`（tag 519）+ `new_meibu_range`（tag 520）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化——`vm_aux.marks` 复用 + 临时技能实例）
- 台账：L27「部分重构（批 3/N）」

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {global:'phaseUseBegin'}`，direct（可选） | `NewMeibu`（tag 519）：`PhaseBegin(Play)` + 可选确认门；`machine()` 从事件取**阶段开始者**为目标 |
| 过滤 | 目标≠自己、存活、自己有 `he` 牌、且 `event.player.inRange(player)`（**目标能及自己**） | 同口径：`computed_distance(目标, 自己) <= attack_range(目标)` |
| 代价 | `chooseToDiscard('he')` | `SelectCards`（自己手牌+装备，min1/max1，可取消）→ 手牌 `discard_from_hand`；装备撤技后入弃堆 |
| 效果① | 目标 `addTempSkill('new_zhixi','phaseUseEnd')`（止息） | 授予目标**临时止息实例**（`expires_end_of_turn`）+ 计数标记 = 目标体力（→ `blocks_all_cards_when` 立即封锁） |
| 效果② | 弃牌非【杀】非锦囊且非黑 → 目标 `addTempSkill('new_meibu_range','phaseUseEnd')` + `storage.meibu = 自己` | 授予目标临时 `new_meibu_range` 实例 + 标记 = 自己座位 → `computed_distance` 中**到该座位距离视为 1** |
| 展示 | `new_zhixi_mark` 文本 | 不迁移（登记差异） |

包数据：`select-beautify-upstream-liuli` 增加 `new_meibu`、`new_meibu_range` 两条声明（生成器同步）。

## 验证

```
cargo test --offline -p engine-native --test l27_new_meibu_batch3
cargo test --offline -p engine-native --test l27_new_zhixi_batch2
cargo test --offline -p engine-native --test l27_xiefang_batch1
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（每步） | **2 passed / 0 failed**（[targeted.log](targeted.log)）：1 号出牌阶段开始时 0 号确认并弃一张红色基本牌（非杀）→ 牌离手、1 号随后使用【杀】**被拒**（止息封锁）；对照（未持技无每步窗） |
| 定向（批 2 / 批 1 回归） | **2/2 + 2/2**（[targeted-batch2.log](targeted-batch2.log)、[targeted-batch1.log](targeted-batch1.log)） |
| engine-native 全量 | **1072 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **88/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 516（tag 519/520 追加） |

## 登记差异

- 止息临时技能以 `expires_end_of_turn` 承载（上游为 `phaseUseEnd`）→ 封锁可能延续到目标**回合结束**（弃牌阶段无出牌/响应面，实际等价）。
- 上游 `checkx`（AI 是否发动）与 `new_zhixi_mark` 文本未迁移。
- 上游「每步」的 `subSkill.range` 用 `storage.meibu`；原生以标记存座位号（等价）。
- 其余 38 个在用技能待批次实施；选将 UI 主面为混淆代码。
