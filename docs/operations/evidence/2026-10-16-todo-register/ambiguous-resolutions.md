# 待做登记表 13 条 ambiguous 消解决议（2026-10-16）

**性质**：只读源码/文档交叉核验后的归类决议；**不改** `crates/**`；本目录为权威写入面。  
**交叉参考**：[D4–D8 决定](../2026-10-16-d4-d8-decisions/README.md)、[D4 Boss ADR 证据](../2026-10-16-d4-boss-adr/README.md)、[ADR 0783](../../../adr/0783-boss-stage-flow-subsystem.md)、[T11 open-rows](../2026-10-16-t11-open-rows/README.md)、[T8 filter batches](../2026-10-16-t8-filter-batches/README.md)、[swd deferred b99](../2026-10-16-t12-swd-deferred-b99/README.md)。

归类取值：`keep_open`（边界已清，仍待做）/ `merge_into`（并入已有 ID）/ `split_new`（拆出新 ID）/ `permanent_diff`（永久差异/排除）/ `wait_d4` / `wait_d5` / `doc_lag_fix`（文档滞后可就地修正）。

---

## 1. `T8-FILTER-COMPOSITE`

| 项 | 内容 |
|---|---|
| **结论** | `keep_open` — 模糊性消除；工单边界已固化 |
| **依据** | [t8-filter-batches](../2026-10-16-t8-filter-batches/README.md)：词表 642、已承载 23、待办 **619**；60 批 `batches.json`；增量 17「18+624」为映射落地前快照 |
| **处置** | 保留本 ID 为 T8 组合 filter 总账；执行以 B01–B60 / `T8F-*` 为准，不以「约 624」为合同计数。依赖 `T8-EFFECT-QUERY-VARIANTS` 不变 |

## 2. `T11-BOSS-PANGTONG-EXTERNAL`

| 项 | 内容 |
|---|---|
| **结论** | `wait_d5` + `keep_open` — 与诸葛亮同属外部包/龙血依赖族，边界已清 |
| **依据** | [ADR 0783 §5](../../../adr/0783-boss-stage-flow-subsystem.md)；[d4-boss-adr](../2026-10-16-d4-boss-adr/README.md)「庞统涅槃/浴火 → 非 D4」；[庞统 v277](../2026-10-14-t11-boss-pangtong-v277/README.md)（天狱/栖梧已落；涅槃/浴火=shenhua `oldniepan` / offline `ylygniepan`+三外部技+`loopType`） |
| **处置** | 保持单列；`depends_on: [D5-LX-MIGRATION]`。完成边界=外部包技可装载且浴火后 loop 面可测，不阻塞 D4 关卡子系统 |

## 3. `T11-BOSS-DIZANG-DAYUAN`

| 项 | 内容 |
|---|---|
| **结论** | `keep_open`（**非 D4**）— 单列内容余项 |
| **依据** | ADR 0783 §5：「地藏王『大愿』AI 态度面……不纳入 D4」；[地藏王 v274](../2026-10-14-t11-boss-dizangwang-v274/README.md)（判定改写机两段 chooseControl + AI 态度；条目只列已转换技） |
| **处置** | 保持单列 open；挂判定改写/态度承载批，**不**依赖 `T11-BOSS-STAGE-SUBSYSTEM` |

## 4. `T11-BOSS-SHIKIEIKI-EXCLUDED`

| 项 | 内容 |
|---|---|
| **结论** | `permanent_diff` — **永久目录排除**（关闭） |
| **依据** | ADR 0783 §5 / d4-boss-adr：「映种……维持目录排除」；[秦广王 v276](../2026-10-14-t11-boss-qinguangwang-v276/README.md) Shikieiki 排除登记（`boss_yingzhong` 依赖 `get.character` 外部将池） |
| **处置** | `status=resolved`。还人终关「映姬候选」若实现：池未就绪则登记「候选恒不可达」，**不**借 D4 建外部将库。未来若立项外部将池基建，**另开新 ID**，不重开本排除项 |

## 5. `T11-BOSS-MENGPO-WANGSHI-PLAY`

| 项 | 内容 |
|---|---|
| **结论** | `split_new` + `merge_into` + `permanent_diff` — 原抓包项拆清 |
| **依据** | [孟婆 v270](../2026-10-14-t11-boss-mengpo-v270/README.md)；[地藏王 v274](../2026-10-14-t11-boss-dizangwang-v274/README.md)；ADR 0783 §5（孟婆往事等零散差异不阻塞子系统） |

**拆分结果**：

| 子面 | 归类 | 落点 |
|---|---|---|
| 往事打出/响应窗（仅 PlayCard 使用位） | `permanent_diff` | 已在 v270 登记为与「封冻」同口径已知差异 → **本父项关闭面** |
| `_status.shidianyanluo_mengpodie` / `boss_mengpohuihun` 回魂牌堆 | `wait_d4` / `merge_into` | **并入** `T11-BOSS-HUANREN`（十殿关卡旗与还人同族） |
| 地藏王无量「建局摸 3」（gameDrawAfter/enterGame） | `split_new` | 新 ID **`T11-BOSS-DIZANG-WULIANG-GAMEDRAW`**（open） |

父项 `status=resolved`（拆分完成）；执行面见子 ID / 并入项。

## 6. `T11-EVENT-RNG-RESTORE-PARITY`

| 项 | 内容 |
|---|---|
| **结论** | `keep_open` — 长期横切，但**可切片关闭**，不再模糊 |
| **依据** | [t11-open-rows 专节](../2026-10-16-t11-open-rows/README.md#专节同一命令下的事件随机流和恢复轨迹对照)：已有 ADR 0727 / `replay_and_restore_per_command_parity` 基线；缺夹具 `parity_chain_compound` / `parity_judge_rewrite_rng` / `parity_aoe_skip_trail` 等 |
| **处置** | 保持单列；完成判定改为：专节「仍缺场景」1–3 至少落地定向测+证据；4–5 随对应差异行立项。非「单一 PR 全关」也不再标 ambiguous |

## 7. `T12-SWD-DEFERRED-SKILLS`

| 项 | 内容 |
|---|---|
| **结论** | `keep_open` — 进度跟踪项；快照漂移不构成归类模糊 |
| **依据** | 台账当前段 + [b99](../2026-10-16-t12-swd-deferred-b99/README.md)：**133/184 映射，deferred 49**（登记表起草时约批 98：131/184、deferred≈51） |
| **处置** | 保持单列；计数以最新 swd deferred 证据 README 为准。卡牌多余 13 仍归 `T12-SWD-EXTRA-CARDS-DISPOSITION`，不并入本项 |

## 8. `T12-SRC-U06`

| 项 | 内容 |
|---|---|
| **结论** | 2026-10-16：`keep_open` — 通道验收不上调。2026-10-17：八项测试通过后 **resolved**。同日 L06 计入后曾为 **2/37**。其后 L07 计入曾为 **3/37**。其后 L23 计入曾为 **4/37**。其后 U03 计入曾为 **5/37**。其后 U04 计入曾为 **6/37**。其后 L04 计入曾为 **7/37**。其后 L05 计入曾为 **8/37**。其后 L22 计入曾为 **9/37**。其后 L02 计入曾为 **10/37**。其后 L15 计入曾为 **11/37**。其后 L17 计入曾为 **12/37**。其后 L18 计入曾为 **13/37**。其后 L26 计入曾为 **14/37**。其后 L21 计入，当前总账为 **15/37** |
| **依据** | 进度主稿：U06 为「首个 G0–G5 **通道**验收通过来源」；AGENTS.md：功能、配置、依赖、组合、保存恢复、安装升级、目标平台、规则差异齐才上调。2026-10-17 证据：[U06 八项](../2026-10-17-t12-u06-eight/README.md) |
| **处置** | 通道验收 ≠ 整包计数，该条仍然有效。八项正反例通过后 U06 计入整包；同日 L06、其后 L07、其后 L23 计入曾为 4/37。其后 U03 计入曾为 5/37。其后 U04 计入曾为 6/37。其后 L04 计入曾为 7/37。其后 L05 计入曾为 8/37。其后 L22 计入曾为 9/37。其后 L02 计入曾为 10/37。其后 L15 计入曾为 11/37。其后 L17 计入曾为 12/37。其后 L18 计入曾为 13/37。其后 L26 计入曾为 14/37。其后 L21 计入，当前总账为 15/37。苦无、牌堆随机点数，以及金币的画布、音效、提示、CSS 和粒子仍为登记差异。U04 不因同一经济再计一次。`T12-WHOLEPACK-0-37` 继续 open，其余 22 个来源未验收 |

## 9. `COV-TRIGGER-GETINDEX-BUCKETS`

| 项 | 内容 |
|---|---|
| **结论** | `keep_open`（条件立项）— 与「触发全序定案」分层 |
| **依据** | 覆盖清单「触发全序细节」定案保留确定性全序；[t11-open-rows](../2026-10-16-t11-open-rows/README.md) `T11-DIFF-TRIGGER-GETINDEX`（P2 内容驱动） |
| **处置** | 保持单列。关闭条件二选一：①有内容需求时写立项 ADR；②无需求时由 architect 在证据目录记「维持不立项」一句即关。无需求期间保持 open **不**算模糊 |

## 10. `COV-PARTIAL-MECHANISMS`

| 项 | 内容 |
|---|---|
| **结论** | `split_new` + `merge_into` — 伞项关闭，机制面单列 |
| **依据** | [覆盖清单](../../../native-coverage-manifest.md) 范围 A「已验证（部分）」多行 |

**拆分 / 并入**：

| 机制面 | 归类 | 落点 |
|---|---|---|
| `snapshot_restore` 新机器/选择逐点 | `merge_into` | `T11-EVENT-RNG-RESTORE-PARITY` |
| `identity_mode` 中间恢复续跑 | `merge_into` | `RULE-IDENTITY-VIEW-RESTORE` |
| `visible_views` 扩展/客户端视图隔离 | `split_new` | **`COV-VISIBLE-VIEWS-EXT`** |
| `session_auth` 权威网络接入 | `split_new` | **`COV-SESSION-AUTH-PRODUCT`** |
| `nullify_judge_window` 兵/闪电抵消组合 | `split_new` | **`COV-NULLIFY-JUDGE-EXPAND`** |
| `nullify_scope` 余量（轨迹夹具） | `merge_into` | 执行面见 t11-open-rows `T11-DIFF-AOE-EVENT-TRAIL`（本登记不重复开伞） |

父项 `status=resolved`。

## 11. `RULE-JUDGE-ORDER-NOTE`

| 项 | 内容 |
|---|---|
| **结论** | `doc_lag_fix` + `permanent_diff`（语义已定案）— **本登记项关闭** |
| **依据** | 覆盖清单行为差异「判定顺序」：**已定案保留原生** FIFO+闪电最后；基准「语义选择」表同行；[judge-order 证据](../2026-10-16-judge-order-upstream/README.md)；t11-open-rows 已列入口径定案关闭集 |
| **文档滞后点** | `native-rules-baseline.md`「当前声明」判定排序行验证边界曾写「是否改判上游待决策」（约 L3447） |
| **就地修正**（2026-10-16 docs-auditor 已改正文） | 验证边界改为：「**已定案保留原生**（非闪电 FIFO + 闪电最后）；与上游 LIFO 差异见语义选择表 / 覆盖清单；死亡剩余机器持牌路径不由终态测试完全覆盖」——证据 [rule-doc-lag-fix](../2026-10-16-rule-doc-lag-fix/README.md) |

## 12. `RULE-CONTENT-DIFF-CONVENTIONS`

| 项 | 内容 |
|---|---|
| **结论** | `permanent_diff` + `merge_into` — 惯例关闭为本登记项；举证并入整将验收 |
| **依据** | 规则基准「内容差异与后续验收」；ADR 0776（护驾转化牌实体响应、激将借牌直结算） |
| **处置** | `status=resolved`（非缺陷、已正式保留）。逐技能/复合场景上游等价举证 **并入** `COV-STANDARD-FULL-CONTENT`，不另挂「未定案缺陷」 |

## 13. `RULE-BUDGET-DOC-LAG`

| 项 | 内容 |
|---|---|
| **结论** | `doc_lag_fix` — **本登记项关闭**（正文已勘正） |
| **依据** | 进度主稿 T13：`PlanPool` 工作池 **2026-10-14 已完成**（[plan-pool 证据](../2026-10-14-t13-plan-pool/README.md)）；基准「预算/时间片」行曾写「独立工作池未实现」（约 L3456） |
| **就地修正**（2026-10-16 docs-auditor 已改正文） | 验证边界改为：「独立工作池（`PlanPool`）**已实现**（2026-10-14）；命令配额阻塞及更广策略质量仍待验收（见 `T13-AI-STRONGER-POLICY`）；令牌原代次反例已修」——证据 [rule-doc-lag-fix](../2026-10-16-rule-doc-lag-fix/README.md) |

---

## 汇总

| ID | 归类 | 登记 status |
|---|---|---|
| T8-FILTER-COMPOSITE | keep_open（边界清） | open |
| T11-BOSS-PANGTONG-EXTERNAL | wait_d5 | open |
| T11-BOSS-DIZANG-DAYUAN | keep_open（非 D4） | open |
| T11-BOSS-SHIKIEIKI-EXCLUDED | permanent_diff | **resolved** |
| T11-BOSS-MENGPO-WANGSHI-PLAY | split/merge/permanent | **resolved** |
| T11-EVENT-RNG-RESTORE-PARITY | keep_open（可切片） | open |
| T12-SWD-DEFERRED-SKILLS | keep_open | open |
| T12-SRC-U06 | 2026-10-17 八项验收后 resolved（通道不上调仍然有效；其后 L06、L07、L23 计入曾为 4/37。其后 U03 计入曾为 5/37。其后 U04 计入曾为 6/37。其后 L04 计入曾为 7/37。其后 L05 计入曾为 8/37。其后 L22 计入曾为 9/37。其后 L02 计入曾为 10/37。其后 L15 计入曾为 11/37。其后 L17 计入曾为 12/37。其后 L18 计入曾为 13/37。其后 L26 计入曾为 14/37。其后 L21 计入，当前总账 15/37） | resolved |
| COV-TRIGGER-GETINDEX-BUCKETS | keep_open（条件立项） | open |
| COV-PARTIAL-MECHANISMS | split/merge | **resolved** |
| RULE-JUDGE-ORDER-NOTE | doc_lag_fix | **resolved** |
| RULE-CONTENT-DIFF-CONVENTIONS | permanent_diff | **resolved** |
| RULE-BUDGET-DOC-LAG | doc_lag_fix | **resolved** |

**新增 open ID（4）**：`T11-BOSS-DIZANG-WULIANG-GAMEDRAW`、`COV-VISIBLE-VIEWS-EXT`、`COV-SESSION-AUTH-PRODUCT`、`COV-NULLIFY-JUDGE-EXPAND`。

**无法消解项**：无。13 条 ambiguous 标记全部清除。

**正文勘正**：`native-rules-baseline.md` L3447 / L3456 与 coverage `judge_zone` 同口径已于 2026-10-16 改正——见 [rule-doc-lag-fix](../2026-10-16-rule-doc-lag-fix/README.md)。**未改**：`native-progress.md`（并行他线）。
