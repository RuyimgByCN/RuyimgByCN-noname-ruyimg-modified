# L27 选将美化 批 2：止息（`new_zhixi`）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L27 选将美化**（`extension/选将美化/extension.js`）
- 本批：技能 `new_zhixi`（止息）+ **状态相关出牌封锁**新原语（`blocks_all_cards_when`）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化——`vm_aux.marks` 复用 + 查询扩展）
- 台账：L27「部分重构（批 2/N）」

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 封锁 | `mod.cardEnabled`/`cardUsable`/`cardRespondable`：`storage.new_zhixi2` 或 计数标记 ≥ 体力 → 全部返回 false | **新原语** `SkillBehavior::blocks_all_cards_when(catalog, core, p)`（默认委托静态 `blocks_all_cards`）；`Catalog::blocks_all_cards` 改调该变体 → 4 个既有封锁消费点（出牌校验/应答窗×2/机器）自动生效 |
| 计数 | `trigger {player:'useCard'}` 锁定（`firstDo`）：`addMark('new_zhixi',1)` | `NewZhixi`（tag 518）：`CardUsed`（`Timing::First` = 上游 `firstDo` ✓）→ `vm_aux.marks[(座位, new_zhixi 定义)] += 1` |
| 锦囊记录 | `get.type2(card)=='trick'` → `storage.new_zhixi2 = true` | 判定「非基本且非装备」→ `vm_aux.marks[(座位, new_zhixi_mark2 定义)] = 1`（**标记键定义**无需实例 ✓） |
| 判定 | `count >= player.hp` | `count >= core.player(p).hp.max(1)` 或 锦囊记录非零 |
| 展示标记 | `storage.new_zhixi_mark = hp - count`（marktext 止息） | 保留**计数**语义（展示面登记差异） |

包数据：`select-beautify-upstream-liuli` 增加 `new_zhixi`、`new_zhixi_mark2` 两条声明（生成器同步；后者仅作标记键定义）。

## 验证

```
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
| 定向（止息） | **2 passed / 0 failed**（[targeted.log](targeted.log)）：使用【无中生有】后计数标记 = 1、锦囊记录 = 1，随后使用【杀】**被拒**（`Rejected`）；对照（未持技：锦囊后仍可出【杀】） |
| 定向（撷芳回归） | **2 passed / 0 failed**（[targeted-batch1.log](targeted-batch1.log)） |
| engine-native 全量 | **1070 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **88/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（标准场景不可达；`blocks_all_cards` 静态路径行为不变）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 514（tag 518 追加） |

## 登记差异

- 展示标记（上游 `hp - count` 与 `marktext`）不迁移：原生保留**计数**语义。
- 上游 `onremove`（移除技时清标记/storage）未建模。
- 上游 `mod.cardUsable` 的「次数上限」面在原生由封锁承载（等价于不可用）；`new_meibu`（每回合首名角色出牌阶段可弃牌令其本回合不能使用牌 ✓ 上游 `derivation`）**待后续批次**。
- 其余 40 个在用技能待批次实施；选将 UI 主面为混淆代码（不逐行重构）。
