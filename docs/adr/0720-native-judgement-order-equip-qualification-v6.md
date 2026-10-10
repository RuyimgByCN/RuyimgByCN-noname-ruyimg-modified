# ADR 0720：判定事件全序落地、鬼道装备资格与规则版本 6

日期：2026-09-16
状态：已实施（engine-native）
关联：[0714](0714-native-guidao-tiandu-semantics-n04d.md)、[0716](0716-native-judgement-events-regressions.md)、[0719](0719-native-checkpoint-hierarchy-guidao-equip.md)、进度文档 N08/N04

## 背景与决定

[0719](0719-native-checkpoint-hierarchy-guidao-equip.md) 收尾时指出三项缺口：鬼道改判资格只查手牌、
`guidao_rewrite_with_black_equip` 名不副实（无真实装备、只答 Pass）、判定事件次数/顺序与
[0716](0716-native-judgement-events-regressions.md) 声明不符。本批逐项闭环。

### 1. N08-d：Finalized 恰好一次、全序对齐 0716

`JudgementProgress` 增加 `finalized`、`judge_held` 两个标志，判定阶段机器重排为：

`Judged → JudgementRewritten×N → JudgementFinalized（恰一次）→ 效果 → 天妒询问 → JudgementGained`

- Finalized 只在「改判队列耗尽、效果生效前」记录一次；旧实现的「无改判分支 + apply 各记一次」
  双记路径消除（seed 233 反例回归 `judgement_no_rewrite_single_finalized`）。
- 天妒询问移到效果生效之后，Gained 事件晚于 Finalized（seed 207 反例回归
  `judgement_event_order_rewrite_then_tiandu`，断言四事件全序）。
- 天妒待决时判定牌由机器持有（`judge_held`）：确认则入手牌、放弃则弃置；
  天妒者死亡等防御路径由完结分支统一弃置。死亡清队列 drain 按 `judge_held`
  决定是否把判定牌计入弃置——已随效果弃置或已获得的牌不再重复入堆。
- `apply_judgement` 改为关联函数（owner 显式传参），判定效果以 `machine_holds`
  决定判定牌是否随效果弃置。

### 2. N08-c：资格过滤含装备区 + 真实装备改判验收

- kernel 询问下一改判者的资格过滤从「手牌有黑」扩为「手牌 ∪ 装备区有黑」；
  仅装备区黑牌者不再被跳过（原反例一）。
- 选项构建（kernel）与 `active_ask` 恢复推导（snap）均为手牌在前、装备在后，顺序一致。
- 删除名不副实的 `guidao_rewrite_with_black_equip`，代之三组真实验收（boundary_snapshot）：
  - `guidao_rewrite_with_real_black_equip_only`：seed 217，2 号经**真实 PlayCard** 装备
    weapon2、清空手牌；改判询问到达、唯一牌选项为装备区武器；出武器后断言 Rewritten
    （card=武器、replaced=旧判定牌）、武器离开装备区并随乐弃置、旧判定牌入改判者手牌、
    Finalized 恰一次且在 Rewritten 后、逐牌区守恒。
  - `guidao_rewrite_mixed_hand_and_equip_options`：黑杀 + 黑武器并存，选项恰为
    `[手牌黑杀, 装备武器]`；出装备后未选手牌保留。
  - `restore_guidao_rewrite_equips_options`：改判窗口挂起 → save → restore → 含装备选项
    完整重建（原 OpenChoiceMismatch 反例关闭）→ 恢复件出武器与原实例同归。
  - 快照不携带恢复前日志：恢复件的 Judged 事件从原实例日志取 id，恢复后新事件正常入日志。

### 3. N04 补充：身份座位数恢复防御 + 中止状态断言

- `validate_core` 在 `roles_len` 之后新增 `identity_seats` 检查：身份局座位数必须在 4–8
  （建局入口 `try_new_identity` 已拒绝，恢复路径对直接伪造的非法人数快照拒绝，防御纵深）。
- `abort_segment_truncates_log` 补回中止后 `state_hash == 段前` 断言
  （前一版本只查日志长度与可继续性）。

### 4. 规则版本 6：哈希编码补全

`canonical_hash` 的 JudgePhase 编码此前不含 `finalized`/`judge_held`——翻转标志无法被哈希
察觉，属完整性缺口。两个标志入哈希后，状态编码变化，按约定递增
`STANDARD_RULESET_VERSION` 5 → 6，14 golden 全部重录并通过。旧版本 5 快照即被
`RuleVersionMismatch` 拒绝。

## 影响

- 测试：128 项（native_kernel 74、boundary_snapshot 49、场景 2、源码 2、调试 1），
  相比 0719 批次净增：−1（删伪装备测试）+3（真实装备三组）+2（事件次数/全序）= +4。
- 快照携带的 `JudgementProgressSnap` 新增两字段；恢复矩阵与逐点恢复用例全部通过。
- 无懈窗口：新布阵在摸牌后可能持有无懈，测试驱动对 `nullify_window` 一律 Pass。
- 剩余：N08 动态技能资格、完整武将授予；N05 主动改判/真实洗牌恢复扩展；N09 runner
  可靠性与计量；N06 距离/传导/奖惩差分；N10–N12 未开工。
