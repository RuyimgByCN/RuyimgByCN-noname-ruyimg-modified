# ADR 0709：段检查点中止、规则版本 3 与纯命令身份局

- 状态：已接受（2026-09-17）。
- 关联：`crates/engine-native/`（`src/engine.rs`、`src/content/standard.rs`）；
  规则基准见 [native-rules-baseline](../architecture/native-rules-baseline.md)；
  前序：[ADR 0707](0707-native-structure-slice-matrix.md)、
  [ADR 0708](0708-native-rule-source-alignment.md)。

## 第十三批后续补注（2026-09-17）

N07-b 已在 [ADR 0710](0710-native-restore-checkpoint-abort.md) 修复，独立探针确认耗尽前/后及非空日志的恢复中止通过；
try_new_identity 2/3/9 人工程负向断言也已补入既有矩阵。下文保留第十二批时点的复核与限制，
不作为当前未完成项；最新状态见 [重写进度](../architecture/native-progress.md)。

## 第十二批独立复核记录

103 项工程测试全部通过；同实例 abort、版本递增、try_new_identity 和双种子纯命令局均已落地。
**N07 仍为部分完成**：可信暂停快照恢复后没有 segment_checkpoint，abort 返回 Ok 却未回滚状态/去重水位。
耗尽前、耗尽后保存恢复两种路径均已实跑复现；具体步骤与验收见 [当前进度 N07-b](../architecture/native-progress.md)。
以下按当前源码更正“恢复件可中止”“提高预算可续行”“全程守恒”等超出实现/测试范围的描述。

## 背景

第十一批后复核确认两处未关闭项，本批据此实现：

1. **N07 失败处理**：暂停段遭遇 `BudgetExhausted` 后段保持开放——
   `resume()` 确定性重复耗尽、新命令一律 `SegmentSuspended`，引擎永久卡死，
   无中止手段。
2. **N06 版本与建局约定**：第十一批语义已变但 `STANDARD_RULESET_VERSION`
   仍为 2；`try_new_identity` 缺失（外部建局依赖 panic）；身份局验收使用
   `debug_*` 击杀而非纯命令对局。

## 决策

### 1. 同实例段失败契约：检查点 + abort_segment

- 段启动时（任意新命令进入 `submit`）保存**段检查点**
  （`StateSnapshot` + 宿主日志水位）；段开放期间保留。
- 新 API `abort_segment() -> Result<(), RejectReason>`：仅限开放段；
  从检查点恢复状态（含修订号与去重水位——中止段的全部命令视为未发生）、
  日志截断回段前水位、关闭段。
- 暂停段遭遇硬预算耗尽仍保持开放；同一引擎可 abort 回滚，再按下一段的预算重试。
  `debug_set_budget` 只影响新段，已耗尽段不能通过调高它再 resume 继续。
- `Pending`/`Done`/非开放段 `Rejected` 时检查点清除；segment_open 随快照延续，
  但检查点不随快照保存，restore 将其设为 None。恢复件可在预算足够时 resume；
  当前 abort 的无检查点分支只关闭段、返回 Ok，保留部分状态和去重键，不能称为回滚成功。
- 修订号：暂停不递增（ADR 0707），中止回滚到段前 revision——宿主观察到的
  revision 序列在失败恢复后仍然单调且无跳变。

### 2. 版本递增与建局接口（N06 子项落地）

- `STANDARD_RULESET_VERSION` 2 → **3**：ADR 0708 的三处语义变化补做
  版本递增；项目内部规则声明与版本约定记录在
  [玩法规则基准](../architecture/native-rules-baseline.md)
  （递增时机、旧快照拒绝、golden 重录登记）。本批 14 个 golden 随版本 3 重录；
  PlaceDelayed 替换 JudgeTrick 的机器编码变化发生在第十一批，段检查点本批尚未入快照。
- 新增 `Engine::try_new_identity(seed, seats) -> Result<Engine, String>`：
  外部建局不依赖 panic；`new_identity` 保留为断言便捷封装。

### 3. 纯命令身份局验收（N06）

`identity_pure_command_match_reaches_outcome`：不使用任何 `debug_*` 击杀，
仅以「出牌阶段对下一位存活玩家出杀、响应全放弃、段内链式选择（濒死链等）
全部排干」的确定性脚本推进；五人 seed 1/3 分别收敛至 **LordSide** 与 **Rebels**，
校验胜负、主公存活关系及最终牌区守恒。读取手牌使用 debug_hand，但未通过调试接口改状态。

当前没有逐命令守恒断言。驱动计数是外层循环次数，应答和部分阶段提交未逐条计入，
不能把原记录的 377/306 解释为准确命令总数。内奸终局、更多人数/种子及重放/恢复对照待补。

## 验证（2026-09-17，`cargo test --offline -p engine-native`，共 103 passed / 0 failed）

- N07 失败契约 2 项：卡住段（`SegmentSuspended` + 确定性重复耗尽）→
  `abort_segment` 恢复段前状态/修订号/去重水位/日志 → 同键重试被接受、
  契约可重复使用；未开放段 `abort` 返回 `NotSuspended`；非分片失败整段
  回滚语义回归。
- N06 新增纯命令身份局 1 项（内部执行两个种子）；原身份配置矩阵未增加 try 接口负向断言。
- 本批新增测试共 3 项（2 项 abort + 1 项纯命令局），旧 100 项继续通过，总计 103。
  14 个 golden 在一个场景测试内运行，不另外计为 14 项。
- 独立探针（不计入 103 项）：非法人数 0/2/3/9/usize::MAX 返回 Err、4–8 人可建局，
  版本 3 拒绝版本 2 快照；同实例中止恢复状态和去重，恢复件中止在耗尽前/后均复现 N07-b。

## 限制与后续

- N07-b：恢复件缺检查点时不能假报 abort 成功；补检查点保存/恢复或明确拒绝策略及两类回归。
- N06：补逐步守恒、内奸终局、规则差异取舍/断言和 try 接口负向工程回归；N05 补足量桃救回、多目标抵消结合洗牌。
- 每条进入结算的命令新增段前完整快照，N09 需测量该开销，不能沿用“每命令只有一次工作副本”的旧成本估计。
- N08（首组官方技能与改判接入）、N09（新线性能基线）仍为后续批次。
