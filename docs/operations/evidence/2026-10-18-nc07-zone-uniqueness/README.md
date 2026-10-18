# NC07 牌区唯一性：弃牌堆重复卡缺陷修复（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC07**（[方案 §NC07](../../../architecture/native-convergence-plan.md#nc07)）+ **NC02**（读档可用性）
- 缺陷来源：**NC12 基线 §5.1**（[证据](../2026-10-18-nc12-baseline/README.md)）——swd 单扩展读档 **30/30 被拒**：
  `IdOutOfBounds{what:"card_in_multiple_zones", got:13}`
- 规则版本 v322（工作树；本批未再改版本）/ 存档格式 v105（无字段变化）
- 结论：**根因已定位 + 已修 + 回归已加 + 全量通过**

## 1 根因（本轮定位，最小复现测试化）

新增探针逐命令扫描「弃牌堆重复卡」，在 **step=17** 首次命中（swd 包、seed 42007、4 座、BoundedPlanner 64 节点）：

```
首次弃牌堆重复卡：step=17 cards=[13] 分布=discard×2
seat=2 action=Answer { choice_id: 3, selection: [Confirm] } 新增入弃牌堆=[177, 204, 13, 13]
日志尾部（时间顺序）：
  CardPlayed { card: 13, player: 2, targets: [1] }      ← 【杀】13 打出
  ChoiceOpened/Answered { id:3, prompt: trigger_confirm }
  SkillTriggered { shenmiguo_window } → CardPlayed { card: 177 }
  SkillTriggered { yuchan_swap } → CardsGiven { cards:[205] }
  CardPlayed { card: 13, player: 2, targets: [1] }      ← **同一张 13 再次被打出**
  DamageDealt { card: Some(13) } ×2
```

链条：**玉蝉换形（`SwdYuchanSwap`）把「正在被使用/结算」的手牌换出并 `to_discard`**，
随后在飞的【杀】结算再次把同一张牌入弃堆 → 弃牌堆出现同一张牌两次 → 引擎允许的活状态
被自己的所有权校验判非法 → 读档被拒（NC12 §5.1 的 30/30）。

## 2 修复（两处）

| # | 位置 | 修复 |
|---|---|---|
| 1 | `machine.rs::Ctx::to_discard` | 无条件的 `push` → **幂等入堆**：已在弃牌堆则直接返回（不重复入堆）。**不**改变调用方的区域语义（从手牌/牌堆移出仍由调用方负责），故既有 505 行为的语义不变 |
| 2 | `content/host/step/swd_equipment.rs`（玉蝉换形） | 换出牌显式 `take_hand_card` 后再 `to_discard`；换入牌按**牌 id** 定位写回（原先按索引 `hand[i]`，在牌区移动后索引失效 → 越界 panic） |

> 过程记录：第一版把「从其它区域移出」也放进 `to_discard`，导致 2 个既有用例失败
> （`g1_yunchou_generals_batch2_xiongzi_yaliang`、`hlss_gaiming_picks_judge_result_or_declines`）
> ——说明该原语被大量调用方按「只入堆」语义使用；因此收窄为**只保证不重复入堆**，
> 由真正需要移动牌区的调用方显式处理（最终全量通过，见 §3）。

## 3 验证结果

```
cargo test --offline -p engine-native --test nc07_discard_duplicate_probe
cargo test --offline -p engine-native --test replay_scenarios
cargo test --offline -p engine-native --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 缺陷回归（探针→回归） | **1 passed / 0 failed**（60 步无重复卡 + **swd 读档成功** + 读档摘要一致） | [targeted.log](targeted.log) |
| 14 golden | **2/2 通过（零位移）** | [replay-golden.log](replay-golden.log) |
| engine-native 全量（含 examples） | **1129 passed / 0 failed / 2 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8238 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办

- [ ] 修复入库：`machine.rs` 为跟踪文件（可提交）；`swd_equipment.rs` 属并行重构新建的未跟踪目录，随其提交入库
- [ ] **NC12 §5.2/§5.3 两处 panic 未修**：① 规划内 panic（`take_hand_card`「card 90 not in hand of 0」、`monkey.rs:73` 被拦截的桃）；② **swd 权威结算 panic**（`apply_plan→submit→commit_frame`，非模拟内）——本轮只修了 §5.1
- [ ] 更一般的不变量：牌区唯一性目前只在 `to_discard` 幂等化 + 恢复期校验兜底，未在**每次移动**统一强制（NC07 结构化收敛待办）
- [ ] 其余牌区移动路径（`to_deck`/`to_hand`/装备进出）是否有同类重复入区路径，需逐路径审计

## 5 追加：NC12 §5.2/§5.3 两处 panic 的处置（2026-10-18 同轮）

### 5.1 §5.3 权威结算 panic —— 未再复现（判定为 §5.1 同根因）

新增 `crates/engine-native/tests/nc12_swd_authoritative_stability.rs`：
- `swd_authoritative_path_is_panic_free`：swd + variant1，seeds 42009/42010/42011 × 200 步，
  **直接 `submit` 真实引擎**（不经模拟），并逐步校验牌区唯一性 → **通过**；
- NC12 §5.3 记录的场景（seed 42009 step 68 `card 204 not in hand of 0`）**未再复现**。

结论：§5.3 与 §5.1 同根因（一张牌在飞结算期间被换区 → 后续对不在手牌的牌 `take_hand_card`），
已由本轮 `to_discard` 幂等化 + 玉蝉换形按牌 id 回写修复。

### 5.2 §5.2 规划内 panic —— 已定位并修复（AI 信息遮蔽破坏状态一致性）

- 复现（NC12 基线官方场景复跑，`nc12-bench-combo-variant1-rerun.log`）：
  `plan_panics=2 degraded=true`，两条 panic：
  `machine.rs:408 card 90 not in hand of 0`（step 5 seat 1）、
  `kernel/monkey.rs:73 被拦截的桃应在使用者手牌`（step 236 seat 0）。
- **根因**：`ai.rs::redact_opponent_hands` 把对手手牌与牌库合并后**重新分配实体牌 id**；
  挂起机器/开放选择/在飞帧记录着这些 id → 重分配后它们指向别的区域 → 模拟内
  `take_hand_card` panic。
- **修复**：重分配只在**静止态**执行（栈空 + 无挂起机器 + 无开放选择）；非静止态跳过。
  对手手牌仍不进入 AI 输入视图（`PolicyInput`），模拟过程中的信息泄漏面登记为 NC10 待办。
- **复跑证据**（同一命令、同机、同负载档）：修复后 `plan_panics=0 degraded=false
  result=OK games=3 steps=800 finished_games=3`（`nc12-bench-combo-variant1-after-redact-fix.log`）。

### 5.3 本批验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| §5.3 稳定性定向（含组合档） | **2 passed / 0 failed** | [targeted.log](targeted.log) |
| NC12 官方场景复跑（修复前） | `plan_panics=2 degraded=true` | [nc12-bench-combo-variant1-rerun.log](nc12-bench-combo-variant1-rerun.log) |
| NC12 官方场景复跑（修复后） | **`plan_panics=0 degraded=false result=OK`** | [nc12-bench-combo-variant1-after-redact-fix.log](nc12-bench-combo-variant1-after-redact-fix.log) |
| engine-native 全量（含 examples） | **1131 passed / 0 failed / 2 ignored** | [engine-native-tests-after-redact-fix.log](engine-native-tests-after-redact-fix.log) |
| 14 golden | **2/2 零位移** | [replay-golden.log](replay-golden.log) |
| 行数门禁 | 8239 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

### 5.4 仍未完成

- [ ] AI 模拟过程中的信息泄漏面（非静止态跳过重分配 → 模拟可能经结果侧信道泄漏对手手牌信息）→ NC10 待办
- [ ] `ai.rs` 为跟踪文件（可提交）；`swd_equipment.rs` 属并行重构未跟踪目录，随其提交入库
- [ ] NC12 基线中 huanle variant1「3 局仅 1 局终局」的分组耗时基线仍不成立（需在 panic 修复后重跑该组）
