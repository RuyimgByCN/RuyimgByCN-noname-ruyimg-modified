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
