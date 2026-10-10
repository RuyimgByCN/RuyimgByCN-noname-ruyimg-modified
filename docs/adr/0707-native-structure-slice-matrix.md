# ADR 0707：结构反例修复、九案例恢复矩阵与宿主时间片原型

- 状态：已接受（2026-09-16）。
- 关联：`crates/engine-native/`（`src/snap.rs`、`src/resolution.rs`、`src/engine.rs`）；
  前序决策见 [ADR 0704](0704-native-snapshot-command-boundary.md)、
  [ADR 0705](0705-native-semantics-nullify-identity.md)、
  [ADR 0706](0706-native-zone-conservation-restore-matrix.md)；
  缺口编号见 [重写进度](../architecture/native-progress.md)。

## 第十二批后续补注（2026-09-17）

第十二批新增同实例 abort，原实例“没有中止手段”的限制已解除；保存恢复后仍丢检查点，N07-b 未关闭。
见 [ADR 0709](0709-native-segment-abort-versioning.md) 和 [当前进度](../architecture/native-progress.md)。
下节保留第十一批后的复核范围，不再作为当前待办。

## 第十一批后核验记录（2026-09-17）

本页记录第十批决策；后续状态以 [重写进度](../architecture/native-progress.md) 为准。
N04 三个反例已修复，N05 九案例基线完成；不能由这些案例推定完整结构校验或所有恢复分支均通过。
N07 仅成功路径通过：09-16 独立探针以 seed 7、4 人、硬预算 2、时间片 1 推进 NextPhase，
续行耗尽后摘要已变化，segment_open 仍为 true，再 resume 持续耗尽，新命令返回 SegmentSuspended。
当前没有 abort；**失败处理属于阶段 2 待验收，不能整体延后到 N11 后将 N07 标记完成。**

## 背景

第九批复核把 N04 拆出三个可复现反例并给出 N05 剩余验收与 N07 待做：

- **N04-a**：`OptionalGate` 快照携带的计划事件载荷（`event`）未做 ID 校验，
  篡改 `Damage.target = 255` 仍可恢复。
- **N04-b**：`validate_stack` 先登记当前帧号再查父帧，父帧自引用
  （`parent == self`）被错误接受。
- **N04-c**：`open_choice` 与续执行机器之间没有关联校验——清空全部
  parks 的快照（应答永远无人承接）仍可恢复。
- **N05**：矩阵只覆盖固定答 Pass 的分支与散财链；未覆盖主动打出无懈/
  反无懈、判定→闪电→濒死嵌套链；未逐步对照事件增量、未断言终止、
  守恒检查未计入机器持牌。
- **N07**：宿主执行时间片与 `BudgetExhausted` 硬上限未区分。

## 决策

### 1. N04-a/b/c：三个结构反例修复

- `MachineSnapshot::validate` 的 OptionalGate 臂递归调用
  `validate_event_data(event)`——闸门计划载荷中的玩家/牌 ID 越界即拒绝。
- `validate_stack` 调整顺序：**先查父帧再登记当前帧号**——自引用与
  向前引用都会被 `frame_parent` 拒绝。
- `validate_resolution` 新增：`open_choice.is_some()` 要求 parks 非空
  （拒绝完全无机器的选择，`choice_without_machine`；尚未验证具体机器是否承接此选择）。**不**要求
  深度相等：AOE 段间伤害帧会停留在机器之上先行结算，挂起深度低于当前
  栈深是合法形态（有负向/正向测试共同锁定）。
- 身份局快照补充主公位置约束：`roles[0]` 必须为主公（首位玩家即主公，
  与 `Engine::new_identity` 的发放规则一致；`lord_seat`）。

### 2. N05：九案例恢复基线

`assert_pause_roundtrip` 升级为四重比对：每点保存/恢复摘要、终态摘要、
**恢复点之后的事件增量**（恢复件宿主日志为空，与基准线对应段逐一对照）、
**牌区+机器持牌守恒**（判定队列/JudgeTrick 持牌计入；闪电命中后
`lightning_wait=true` 的锦囊已弃置、不计入）。终态断言两线均无开放选择。
新增两个案例：主动无懈/反无懈链（`answer_plays_nullify`，覆盖窗口
asked 翻转与队列重建）、判定→闪电→濒死救援嵌套（`answer_saves_dying`
打出桃参与救援；第十批机器树为 JudgePhase→JudgeTrick→Damage→DyingSave，
第十一批已移除 JudgeTrick）。矩阵共 9 案例，未逐种枚举所有同步机器的嵌套恢复形态。

核验更正：该濒死布阵为 1 血受 3 雷伤且只提供 1 桃，没有成功救活断言；
不能写作“用桃救回”。补足桃、断言存活并继续剩余判定队列仍归 N05。

### 3. N07：宿主时间片

- `run()` 增加 `slice: Option<u32>` 参数：每完成一个调度步计一次数，
  片内步数用尽即在**步边界**返回新结果 `StepOutcome::Yielded`——此时
  状态自洽（机器均已正确挂起），可无损续行。
- `Engine` 新增会话数据 `time_slice`（默认 None = 不限）与 `segment_open`；
  API：`set_time_slice`、`is_suspended`、`resume()`。
- `CommandOutcome::Suspended`：状态已提交、无开放选择；`resume()` 以新片
  继续同一 `Resolution`，直至 Accepted/Pending/Rejected。暂停中的段拒绝
  一切命令（`SegmentSuspended`）；未暂停时 `resume()` 返回 `NotSuspended`。
- **暂停不递增 revision**：暂停是同一段执行的延续，完成时才增加 revision。
  所测成功路径中，分片与连续运行的终态摘要、事件序列一致；这不涵盖预算失败路径。
  暂停态的内容（预算/执行位置）变化仍使其哈希可区分。
- 硬预算失败行为随执行方式不同：不限片时 `BudgetExhausted` 整段回滚、状态逐字节不变。
  已分片提交的段无法整段回滚——开放段上的硬预算耗尽提交当前进度并返回
  `Rejected(BudgetExhausted)`，段确定性地重复耗尽。原决策将中止留到服务器阶段；
  2026-09-17 复核调整：宿主目前无法通过 API 放弃该段，N07 必须先补回滚/中止及失败回归。
- `segment_open` 随快照延续：暂停态保存→恢复后仍可 `resume()`。

## 第十批原执行记录（2026-09-16，共 98 passed / 0 failed）

- N04 负向回归 7 项：闸门载荷篡改、父帧自环/向前引用、选择无机器、
  主公位置篡改等全部拒绝；`restore_accepts_park_depth_below_stack`
  锁定「深度低于栈深」的合法形态。
- N05 矩阵 9 案例全部通过（新增主动无懈链、判定→闪电→濒死嵌套）。
- N07 测试 4 项：片 ∈ {1,3,7,64} 与不限片的终态摘要、事件序列逐条一致
  （片=1 暂停 ≥ 10 次）；暂停中命令拒绝且不改状态；非暂停 resume 拒绝；
  暂停点快照往返后继续与连续运行同终态。
- 其余 78 项（含 14 个 golden）不变全绿；无新增编译警告。

## 限制与后续

- N07 开放段失败后的回滚/中止未实现，优先补失败后可恢复操作、revision/dedup/日志/快照契约。
- N04 完整语义结构校验、N05 扩展分支、N06 独立规则依据与完整身份验收仍未全部完成。
- 本页 98 项是第十批历史记录；第十一批实跑结果统一见重写进度。
