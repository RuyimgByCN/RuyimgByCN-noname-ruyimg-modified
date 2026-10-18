# L20 标记补充 批 3：烈弓（`fz_liegong`）+ 狂骨（`fz_xinkuanggu`）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L20 标记补充**（`extension/标记补充/extension.js`）
- 本批：`fz_liegong`（tag 535）+ `fz_xinkuanggu`（tag 536）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- 台账：L20「部分重构（批 3/N）」

## 上游语义与原生接线

| 技能 | 上游 | 原生 |
|---|---|---|
| 烈弓 `fz_liegong`（2.0 KB） | `trigger {player:'useCardToPlayered'}`——对目标出【杀】且**目标手牌数 ≥ 自己体力 或 ≤ 自己攻击范围** → `directHit.push(target)` | `FzLiegong`（tag 535）：**杀机器内联**——满足条件时跳过防具判定与出闪询问（逐目标口径） |
| 狂骨 `fz_xinkuanggu`（2.3 KB） | `trigger {source:'damageSource'}` direct——距离 ≤1 且伤害 >0 → 回复 1 点体力**或**摸 1 张牌 | `FzXinKuanggu`（tag 536）：`Damage`（自己为来源、`amount>0`、`computed_distance ≤ 1`）可选确认门 → `PickTag`（0=回复 1 / 1=摸 1） |

## 验证

```
cargo test --offline -p engine-native --test l20_liegong_batch3
cargo test --offline -p engine-native --test l20_kurou_batch2 / l20_config_mods_batch1   # 批 1/2 回归
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
python3 -B scripts/check-source-lines.py
```

| 项 | 结果 |
|---|---|
| 定向（批 3） | **2 passed / 0 failed**（[targeted.log](targeted.log)）：烈弓——目标手牌 4 ≥ 出杀者体力 4 → **直伤命中**（未进入出闪）；狂骨——造成伤害后选「摸 1」→ 手牌净持平（出杀 -1 + 摸 1） |
| 定向（批 1/2 回归） | 3/3 + 3/3（[targeted-regression.log](targeted-regression.log)） |
| engine-native 全量 | **1095 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **94/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| **行数门禁（AGENTS.md 新增）** | `check-source-lines.py`：**8215 文件 ≤ 2000 行**（通过）；单测 `test_check_source_lines.py` OK |
| 键表守护 | `HostBehavior` 532（tag 535/536 追加） |

## 登记差异

- 烈弓的 `mod.attackRange`（`get.zhu(player,'shouyue')` 时 +1）**未迁移**（原生无「守约」主公技判定面）。
- 烈弓上游为**逐目标**窗口（`useCardToPlayered`）；原生在杀机器内按当前目标计算（等价）。
- 狂骨的 AI 选择（`isDamaged && recoverEffect>0 && …` 分支）未迁移：原生始终开确认门由玩家/策略选择。
- 其余 69 个技能待批次（中位 2.5 KB）。

## 提交状态说明（并行开发）

本批的**引擎侧臂**（`HostBehavior::FzLiegong`/`FzXinkuanggu`、杀机器烈弓直伤臂、键表守护 532）、**包声明**（`mark-supplement-upstream-liuli` 增至 6 条）与**用例**（`l20_liegong_batch3.rs`）位于工作树，与另一并行重构（`host.rs` 由 37 828 行拆分为 607 行等）处于**同一批文件**；为避免把他方未完成改动一并提交，本次只提交文档与证据（本目录、T9 登记表、进度主稿、台账、登记表）。上述代码/包/用例的验证已在工作树完成（见上表），待并行重构提交后随之下库。
