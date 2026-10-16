# 2026-10-16 延时锦囊无懈时点核对 + 窗口/顺序口径「语义选择」定案

核验日期：2026-10-16。**只读核对 + 定案登记**（无代码/版本/golden 变化；规则 v290 / 存档 v77）。
采用玩法版本 = 本地 `archive/upstream/noname`（v1.11.7，HEAD `df2682e6`）。

## 1. 延时锦囊无懈时点（覆盖清单「延时锦囊无懈时点」行）

| 线 | 行为 | 源码 |
|---|---|---|
| 上游 | **两处开窗**：①使用牌时（`useCardToBegin`）；②判定阶段该锦囊进入判定时（`phaseJudge` 事件，锦囊已离判定区后开窗，被抵消则该次**不翻判定牌**、锦囊效果取消） | `_wuxie.trigger: {player:["useCardToBegin","phaseJudge"]}`（`card/standard.js:4225`）；`phaseJudge` 触发点 `content.ts:4953`；cancelled 分支 `content.ts:4962-4966`；无懈本体 phaseJudge 分支 `card/standard.js:3134-3137` |
| 原生 | **仅使用时开窗**（ADR 0708：对齐旧线），判定阶段不再开窗；被抵消则锦囊不入判定区 | 覆盖清单 `delayed_trick`/`nullify_scope` 行；`delayed_trick_play_time_nullify_window` 断言 |

⇒ 差异成立（另一次窗口面差异）。处置：并入下方「窗口/顺序口径」语义选择定案。

源码摘录：[upstream-delayed-nullify.txt](upstream-delayed-nullify.txt)。

## 2. 窗口/顺序口径「语义选择」定案（覆盖清单行为差异表 6 行收口）

2026-10-16 逐项对照（本批及本轮前 5 个证据目录）后**定案：保留原生（沿用旧线）口径，
上游对应行为登记为已知差异；改判需另立 ADR**（语义变化：递增 `STANDARD_RULESET_VERSION`、
重录 14 golden、改造相关测试）。

| 口径 | 项目采用（原生，源自旧线） | 上游 v1.11.7 | 对照证据 |
|---|---|---|---|
| 无懈应答名单起点 | 自**牌来源**起（`seat_order_from(source)`，ADR 0708） | `list.sortBySeat(_status.currentPhase)`（`card/standard.js:4487`） | [AOE/决斗/无懈](../2026-10-16-aoe-duel-nullify-ask-semantics/README.md) |
| 无懈对无懈 | 单窗口取反 + 自出无懈者下家重建环 | 嵌套 `_wuxie` 窗口（无懈本体重走 `useCardToBegin`） | 同上 |
| 铁索传导目标顺序 | `seat_order_from(受害者)` | `sort(lib.sort.seat)`（起点 `_status.currentPhase`，`library/skill.js:2037-2039`） | [铁索传导/座位序](../2026-10-16-chain-propagation-seat-order/README.md) |
| 解链时点 | 发传导帧**前**一次解除全部连环 | **逐受害者**结算时解链 + `notLink()` 事后防级联 | 同上 |
| 多目标非 multitarget 锦囊目标定序 | 命令提交序（使用者下家起） | `targets.sortBySeat(_status.currentPhase)`（`content.ts:10143`） | 同上 |
| 判定阶段顺序 | 非闪电放置序 FIFO + 闪电恒最后（ADR 0713） | 判定区数组 **LIFO**、无闪电分组（`content.ts:4931`+`player.js:15791`） | [判定顺序](../2026-10-16-judge-order-upstream/README.md) |
| 濒死轮次结构 | 按轮重建名单（重建起点=濒死者） | `acted` **单趟**、救回者原位复问 | [濒死名单](../2026-10-16-dying-save-roster-upstream/README.md) |
| 延时锦囊无懈时点 | 仅**使用时**开窗 | 使用时 **+ 判定时（`phaseJudge`）** 均开窗 | 本页 §1 |
| 临时技能存续期 | 至**拥有者**回合结束（上界近似） | 逐站点 `addTempSkill` expire 规格（默认≈当前阶段结束） | [临时技能存续期](../2026-10-16-temp-skill-expiry-audit/README.md) |

**定案依据**（逐条可复核）：
1. 这些口径在原生线已有成批回归与 14 golden 基线；翻转属语义变化，成本高于收益且会打破既有证据链。
2. 其中数处是原生/旧线为**确定性重放与恢复契约**选定的结构（判定顺序确定性、窗口状态可编码），
   上游对应处依赖运行时交互或 UI 约定（如 `chooseControl` 并列选择、`phaseJudge` 窗口在 UI 层呈现）。
3. 上游对应行为**未被证明**与官方规则一致（判定区 LIFO 与"按放置顺序"的官方口径相反；无懈名单起点
   属上游实现约定）。→ 因此不构成改判的充分依据。

**改判入口**：任一条目如后续获得官方规则依据或用户裁决，按 ADR 流程改判（版本递增 + 14 golden 重录 +
测试改造 + 证据目录），届时本表相应行改为"已改判"。

## 3. 影响

覆盖清单 6 行（AOE 询问顺序、无懈询问顺序与重开、逐目标窗口重开细节、解链时点、判定顺序、
濒死应答队列静态、延时锦囊无懈时点、临时技能存续期）处置由"待决策/待核对"改为
"**已定案：保留原生口径（见规则基准语义选择）**"；规则基准新增定案小节。无代码变化。
