# E03 事件帧生命周期缺陷定位与修复（TurnOverBefore 类帧）

主稿 E03 行登记的「事件帧生命周期缺陷」（TurnOverBefore 类帧未弹栈/派发时序，
写轮眼窃取状态膨胀与面具取消失效同源）定位到根因并修复。前置批次：
[mianju 实现](../2026-10-07-e03-mianju-impl/README.md)。

## 根因（三层）

1. **机器发射的子事件跳过前置窗口**（主因）：`Ctx::emit`
   （`src/machine.rs`）对 Damage 之外的子事件一律以 `Commit` 阶段起步。
   据守类机器 `emit(TurnOverBefore)` 后返回 `Flow::Done`，帧从 Commit 起
   步——`BeforeTriggers` 从未执行，取消窗口缺失，commit 以
   `cancelled=false` 先行翻转；随后 `AfterTriggers` 按 `f.kind`（仍为
   TurnOverBefore）再查候选，面具机器才入队并把 `cancelled=true` 写回
   **已提交的残留帧**（旧 DBG 实证 commit 先于写回、cur_frame=9 残留，
   与根因吻合）。
2. **after 窗口重复入队**：`plan_triggers` 的 after 阶段按 `f.kind` 查候选，
   TurnOverBefore 帧会让取消技在 before/after 各入队一次（重复写回）。
3. **`debug_flip` 未走事件流**：直接翻转状态（缺陷期临时通道），取消语义
   无从生效。

## 修复

| 文件 | 改动 |
|---|---|
| `src/machine.rs` | `emit`：Damage 与 **TurnOverBefore** 子事件从 `BeforeTriggers` 起步（前置窗口对机器发射的子事件同样生效） |
| `src/resolution.rs` | `plan_triggers`：TurnOverBefore 帧的 after 窗口静默（目录无「翻面后」触发种类；避免取消技重复入队） |
| `src/engine.rs` | `debug_flip` 事件驱动化：压入 TurnOverBefore 帧（BeforeTriggers 起步）→ `run` 驱动到弹栈，返回翻面是否发生；非 Done（待决/分片/预算）视为未闭环即 panic 并回滚日志 |
| `src/kernel.rs`、`src/content/host.rs` | 清理缺陷期 DBG `eprintln!`（`commit_flip` / `mianju 写回完成`） |

## 测试恢复（ignore 全部移除）

| 测试 | 状态 |
|---|---|
| `huanle_mianju_blocks_flip_while_equipped` | **通过并移除 ignore**（无面具翻面 → 面具取消 → 摘除后翻面） |
| `huanle_xuelunyang_steals_target_skill_until_turn_end` | **通过**（并行会话已移除 ignore；状态膨胀与帧残留同源，修复后消失） |
| `caoren_jushou_flips_and_mianju_cancels_flip` | **通过并移除 ignore**（据守翻面/翻回/面具取消+摸牌仍执行；旧「门应答循环自触发」实为帧缺陷症状） |
| `xly_flip_frame_lifecycle_probe` → **`flip_frame_lifecycle_no_residue`** | 升级为常规回归守卫：翻面后 stack/parks 必须清空 |
| `mianju_turn_over_before_cancel_writeback_semantics` → **`flip_primitive_toggles_facing_without_mask`** | 重写为无扩展基线（旧断言基于直接翻转语义已失效）；取消路径由面具测试覆盖 |
| `xly_state_growth_probe` | 保留为 ignore 诊断探针（修复后规模曲线已平坦：stack/parks 归零、skills 恒定） |

## 验收口径（核验于本批会话，命令与输出见 test-logs.txt）

- 分开陈述：**代码已实现**（emit 阶段修正 + after 静默 + 事件驱动 debug_flip）；
  **回归已加入**（帧无残留守卫）；**定向测试通过**（g1_huanle 13/0/1）；
  **全量通过**（engine-native 356/0/1、loader 全绿、content-runtime 全绿、
  engine-ai 29/0，合计 431 通过 / 0 失败 / 2 个统计行（g1_huanle 全量+定向
  各计一次，实际唯一 ignore = 规模探针））；`RUSTFLAGS=-D warnings` 通过；
  **golden 未动**（本批不涉及语义入哈希面的新增事件序列——据守/面具路径
  不在 golden 场景内，replay_scenarios 全绿即哈希一致证据）。
- 规则/格式版本不变（v61 / v4）：本批为缺陷修复，无语义面扩展。
- 未做：P2/N08 曹仁条目**装配授予路径**（`debug_set_general` 仍只装槽位，
  据守测试经 `debug_grant` 授予技能驱动）；E01 逐来源行为样本；其余主稿待做照旧。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（6 文件，相对 noname-next 根）。
