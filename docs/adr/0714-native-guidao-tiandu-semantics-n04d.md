# ADR 0714：空改判队列拒绝、天妒范围与鬼道旧牌归属修复

- 状态：第十六批切片已落地；2026-09-18 独立实跑 112 项通过，三项具体反例关闭；完整结构/技能验收未完成。
- 关联：[snap](../../crates/engine-native-kernel/src/snap.rs)、[kernel](../../crates/engine-native-kernel/src/kernel.rs)、
  [前序 ADR 0712](0712-native-first-official-skills.md)、[当前进度](../architecture/native-progress.md)。
- 规则参考：仓库上游与旧线内容，具体来源链接保留于 ADR 0712，不把上游参考或测试数当作官方等价证明。

## 后续复核（2026-09-20）

本文保留第十六批记录。N04-e 已在 ADR 0715 修复；事件已增加但次数/顺序有反例，装备改判也有合法恢复缺口。
最新 118 项与待办见 [当前进度](../architecture/native-progress.md)，不再把本文当时的 e/事件缺失描述当作现状。

## 背景与已落地修改

第十五批后复核发现 N04-d、天妒范围和鬼道旧牌归属问题，本批分别处理：

1. JudgePhase 快照校验拒绝 in_rewrite=true、asked=true、rewriters=[]，错误为
   rewrite_asked_without_rewriter；运行时消费分支也增加空队列防御。
   新增 restore_rejects_rewrite_asked_with_empty_rewriters，独立反例复跑同样被拒绝。
2. 天妒候选仅含持有技能的判定者本人。tiandu_gain_own_judge_card_only 和组合测试改为本人获得；
   原“他人天妒能取牌”反例已修复。
3. 鬼道替换时旧判定牌进入改判者手牌，新黑牌由判定流程持有，最终弃置或由判定者天妒获得。
   已补旧牌在手牌的断言；原“旧牌被弃置”差异关闭，装备区资格仍未接入。
4. STANDARD_RULESET_VERSION 4→5，原因是以上两项技能语义改变；14 个 golden 重录并通过，
   维护记录见 [覆盖清单](../native-coverage-manifest.md)。独立探针确认版本 5 拒绝版本 4 快照。

## 独立复核结果

全套 112 passed / 0 failed：源码单元 2、机制 64、边界快照 43、场景 2、调试 1；
相较上一轮只净增 1 项 N04-d 测试，技能机制测试为改写既有用例。

本轮主动分支探针沿用 seed 207、1 号判定乐、2 号鬼道、0 号为其他天妒持有者：

| 分支 | 结果 |
|---|---|
| 1 号无天妒，只有 0 号持有 | 主动改判后直接完成，无天妒选择；0 号不获得最终牌 |
| 1 号也有天妒并 Confirm | 仅询问 1 号，最终牌进入 1 号手牌，不入弃牌 |
| 1 号也有天妒并空应答放弃 | 最终牌进入弃牌，不被任何天妒持有者获得 |

三组均验证鬼道旧牌归属、skip_play_phase、各选择点直接/恢复摘要与事件增量、终态逐牌守恒。
这些是独立探针，不计入 112 项，也未固化为工程恢复测试。

## 新反例 N04-e：非空队列仍可能无法承接选择

在 gui_dao_rewrite 选择点只把 current.rewriters[0] 从 2 改为合法座位 3，
保留 observer=2、原黑牌选项及 2 号手牌。restore 仍返回 Ok；2 号提交原黑牌后，
take_hand_card(3, card) panic：`card 0 not in hand of 3`。

N04-d 仅补“非空”条件，未关联队首、开放选择、机器阶段、选项归属及资格。
后续须在 restore 拒绝不一致状态并补正反回归；不能仅在取牌处忽略错误，或把该快照视为合法续行。

## 覆盖声明更正与后续

- 原稿“改判与获得路径已入矩阵”不成立：pause_roundtrip_guidao_rewrite 的 builder 没有天妒，
  answer_by_prompt 实际答 Pass。将本轮三组主动探针固化后，才能计入工程矩阵覆盖。
- tiandu_gain_own_judge_card_only 只给判定者授予技能，未构造他人持有者；本轮独立排除验证通过，
  仍需工程负向断言。鬼道跳阶段、双方都改判等直接行为断言也未补齐。
- “最终判定/获得事件断言已补”不成立：新增的是手牌归属断言；kernel 仍只在初翻牌记 Judged，
  改判/天妒直接改手牌，最终判定与获得事件契约/日志断言尚缺。
- N04-e 与身份配比、N05 成功救活/主动恢复、N08 装备资格/事件、N06 其余规则仍须验收；
  N09 可准备 runner，现有 112 项和 golden 不代表阶段 2 或首组技能全部完成。
