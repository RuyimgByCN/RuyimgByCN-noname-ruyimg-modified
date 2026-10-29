# ADR 0713：判定稳定分组与版本 4（N06-a 原反例关闭）

- 状态：已落地；2026-09-17 独立复核通过，原排序反例关闭。
- **2026-10-16 补注（勘正依据）**：本 ADR 的分组依据是当时参照的**旧线** `collect_delayed_queue`，不是规则来源。项目采用玩法版本已于 2026-10-16 明确为本地 `archive/upstream/noname`（v1.11.7），而上游 `phaseJudge` 为判定区数组 **LIFO（最后放置的先判）、无闪电分组**（`content.ts:4931-4960` + `player.js:15791`）。因此本 ADR 的排序规则与采用版本**不一致**，是否改判上游（属语义变化：版本次＋14 golden 重录＋既有排序测试改造）另立 ADR 决策；本批只做核对与现状固化——见[证据](../operations/evidence/2026-10-16-judge-order-upstream/README.md)与覆盖清单行为差异表「判定顺序」。
- 关联：[原生 kernel](../../crates/engine-native-kernel/src/kernel.rs)、[版本定义](../../crates/engine-native-content/src/content/standard.rs)、
  [前序 ADR 0711](0711-native-judge-order-alignment.md)、[首组技能切片 ADR 0712](0712-native-first-official-skills.md)。

## 背景与实现

第十四批只改为队首消费，反向放置闪电→乐仍会先闪电；规则版本仍为 3。
本批在创建 JudgePhase 队列时先收集非闪电，再追加闪电，两组内保持原相对顺序，之后队首先消费。
旧线参考函数实际为 collect_delayed_queue / resolve_judge_queue（前稿误写为 pending_delayed_cards）。

STANDARD_RULESET_VERSION 3→4，14 个 golden 全量重录。版本登记还须包含同轮 ADR 0712 的
技能目录追加与 JudgePhase.current 编码扩展；golden 一致本身不证明技能正确。

## 独立验证

- cargo test --offline -p engine-native：**111 passed / 0 failed**，已有编译警告仍在。
- judge_order_delayed_first_then_lightning_hit_kills 覆盖两种放置序：乐→闪电、闪电→乐；
  判定牌均红桃→黑桃 2–9，最终闪电致死及牌区守恒断言通过。
- 原 N06-a 独立探针保持 seed 179、3 血、黑桃 2–9→红桃：两种放置序现均存活，均 2 次 Judged，
  与声明的先乐后闪电一致。原反向放置的死亡反例不再复现。
- 独立探针把顶层/核心规则版本改为 3，restore 拒绝；14 个版本 4 golden 通过。
- 无新增测试数量：扩展既有排序测试并更新 golden；111 相对 106 的 5 项新增来自 ADR 0712。
  try_new_identity 的 2/3/9 人负向断言早在第十三批已存在，不重复计本批交付。

## 验收边界与后续

- N06-a 原反例关闭；多张非闪电的相对顺序、闪电在中间及排序/改判/保存恢复组合仍需扩展断言。
- 旧 N02 闪电→乐布阵经分组后不再测试死亡清剩余 queue；不能继续沿用上轮“本轮清理探针通过”的表述。
  防御性机器持牌清理仍保留专用测试待办；不得把当前简单布阵不可达推广到所有后续技能组合。
- N04-d 改判快照非法阶段、N08 天妒范围/鬼道旧牌归属仍待修复，N05 单桃布阵仍未救活。
  首组技能和阶段 2 均未整体验收，后续顺序以 [当前进度](../architecture/native-progress.md) 为准。
