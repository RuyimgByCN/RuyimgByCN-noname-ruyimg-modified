# ADR 0710：N07-b 恢复后中止——段检查点随快照延续与严格失败契约

- 状态：已接受（2026-09-17）。
- 关联：`crates/engine-native/`（`src/snap.rs`、`src/engine.rs`）；
  前序：[ADR 0707](0707-native-structure-slice-matrix.md)、
  [ADR 0709](0709-native-segment-abort-versioning.md)；
  缺口编号见 [重写进度](../architecture/native-progress.md)。

## 同日独立复核

105 项工程测试全部通过；另以独立探针覆盖真正耗尽前/后 × 空/非空历史日志四种组合，
原实例与恢复件均能中止回滚、退还去重键并同键重试完成，N07-b 原反例关闭。
新增工程测试的 helper 已先耗尽预算，因此其中“耗尽前保存”的注释与实际布阵不符；
本页据源码区分工程回归和补充探针，不把它们重复计数。剩余结构校验见 [N04/N05](../architecture/native-progress.md)。

## 背景

第十二批复核探针复现 **N07-b**：暂停段的快照经 `restore` 后，恢复件
`segment_open == true` 但没有段前检查点——`abort_segment()` 走「无检查点」
防御分支，仅关闭段标志并返回成功，既不回滚部分状态也不恢复去重水位。
原因：`Engine::save/restore` 不保存 `segment_checkpoint`，恢复时置 `None`。
该快照由可信引擎自身生成即可触发，不能以「不接受外部篡改」排除。

## 决策

### 1. 段检查点随快照延续

- `StateSnapshot` 新增 `segment_checkpoint: Option<Box<StateSnapshot>>`：
  `save()` 把当前段检查点的**完整快照**嵌套保存（日志水位不保存——恢复件
  的宿主日志从空开始，中止时截断到 0）；`restore()` 以水位 0 重建检查点。
- restore 检查直接内嵌检查点的规则版本、核心状态和执行位置结构；本轮探针确认
  该检查点版本不符会被拒绝。这不等于完整递归校验任意嵌套及外层/检查点一致性，后者仍归 N04。

### 2. 严格失败契约

`abort_segment` 的「无检查点」分支不再静默关闭段并返回成功——返回
`Err(NotSuspended)` 且不改状态或开放标志。正常 submit 创建的开放段携带检查点；
人为去除检查点的开放快照目前仍可 restore，但 abort 会报错，不再假报回滚成功。
开放标志、检查点存在性和层级的一致性校验仍归 N04。

### 3. 中止后的重试路径

中止后宿主可在**下一段**调整预算（`debug_set_budget` 覆盖）并以同键重试
命令使其完成；不宣称「提高预算后可直接 `resume`」（开放段预算已确定性地
耗尽）。时间片仍是引擎侧会话设置，重试前由宿主自行清除或放大。

## 验证（2026-09-17，`cargo test --offline -p engine-native`，共 105 passed / 0 failed）

- `abort_after_restore_restores_segment_start`：helper 已先耗尽预算；保存恢复后中止，
  断言段前摘要/revision、恢复件空日志及同键重试完成。
- `abort_contract_direct_vs_restored_identical`：两条线也均来自已耗尽 helper；
  比较直接/恢复后中止的摘要、revision、栈/挂起数，重试完成后摘要一致。
- `try_new_identity` 2/3/9 人返回 Err 的断言补在既有身份矩阵内，未新增独立测试计数；
  new_identity 的 panic 契约仍保留。
- 本批新增 2 项中止测试，原 103 项继续通过，总计 105。14 个 golden 在一个场景测试内运行，
  本批规则版本仍为 3、golden 未重录；已有编译警告仍在。
- 独立探针（不计入 105）：真正首次 Suspended 时预算尚未耗尽，与耗尽后分别保存，
  再组合已有/没有历史日志。四组均恢复段前摘要、revision、去重、栈/挂起数；
  原实例保留历史日志，恢复件为空；同键重试完成且两线终态相同。
- 异常快照探针：直接检查点版本不符被拒绝；去掉检查点后可 restore，但 abort 报错且状态不变。

## 限制与后续

- 检查点会克隆段前快照，具体内存/分配成本尚未测量，不填写倍数；计入 N09 新线基准。
- 中止只回滚一个开放命令段，不跨已经完成的 Accepted/Pending 边界。
- N04 补恢复身份精确配比、机器/选择关联及检查点一致性；N05 固化真正耗尽前和非空日志用例，
  补足量桃救回/洗牌组合；N06 补规则差异取舍与身份局逐步守恒、内奸终局。
- N08 官方技能、N09 性能基线、N11 字节存档继续按进度文档推进。
