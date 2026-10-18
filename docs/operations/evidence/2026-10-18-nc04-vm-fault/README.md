# NC04 VM 故障与事务语义：故障传播 + 整段回滚（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC04**（[方案 §NC04](../../../architecture/native-convergence-plan.md#nc04)）
- 规则版本 v321 / 存档 v105（本次无规则与格式变化；`Resolution.fault` 为瞬态槽，不入快照/哈希）
- 结论：**根因已修 + 3 项回归通过（1 项挂起用例待补）**；跨目标/产品面待后续阶段

## 1 根因（已确证并修复）

`VmMachine::run` 的 `Err(_) => self.trap_done()`：VM 执行错误（fuel/栈/内存/能力/校验等）被**丢弃**，只清空排队效果后返回 `Flow::Done`，规则按正常路径继续——错误类型与来源全部丢失，违反 NC04「不得以 `Flow::Done` 吞掉故障后继续当作正常规则执行」。

## 2 修复

| 面 | 修复前 | 修复后 |
|---|---|---|
| 错误捕获 | `Err(_)` 丢弃 | `Err(e)` → `VmFault { skill, owner, event, kind, detail }`（技能/座位/事件/`VmError` 类别与详情） |
| 传播 | 无 | 调度主循环检测故障槽 → `StepOutcome::Fault` |
| 事务 | 仅丢弃排队效果 | **整段回滚**：工作副本丢弃、宿主日志截断、段检查点清除；开放段回滚到段前检查点 |
| 命令结果 | `Flow::Done`（表现为 Accepted） | `CommandOutcome::Rejected(RejectReason::VmFault { skill, detail })` |
| 诊断 | 无 | `Engine::resolution()` 暴露只读执行位置（含瞬态故障槽） |

## 3 回归（新增 `crates/engine-native/tests/nc04_vm_fault.rs`）

| 验收项 | 用例 | 结果 |
|---|---|---|
| 正常完成 | `nc04_normal_completion`（VM 阶段末回血/摸牌生效、无残留故障） | 通过 |
| fuel 耗尽（既有执行限额） | `nc04_fuel_exhaustion_rolls_back_and_rejects`（`loop 100000 { effect … }` → `OutOfFuel`）：**故障当次摘要与修订号不变**（整段回滚）；同触发点重复提交**确定性再现故障**且状态仍不变（不静默跳过） | 通过 |
| 故障命令的去重语义 | `nc04_duplicate_submission_after_fault`：被拒绝（含故障）的命令**不消费去重键** → 同 dedup 重放仍是确定性 `VmFault`（与既有 `Rejected` 语义一致） | 通过 |
| 选择挂起后恢复 | `nc04_pause_and_resume` | **ignored（待办）**：DSL 选择语句关键字未确认，不伪造覆盖 |
| 非法效果 / 无效 token / 恢复后故障 | 未覆盖 | 待办 |

## 4 验证结果

```
cargo test --offline -p engine-native --test nc04_vm_fault
cargo test --offline -p engine-native --no-fail-fast
CARGO_TARGET_DIR=<tmp> RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --all-targets
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC04 定向 | **3 passed / 0 failed / 1 ignored** | [targeted.log](targeted.log) |
| engine-native 全量 | **1104 passed / 0 failed / 2 ignored**（含新增 3 项 + 1 项 NC04 待补 ignored） | [engine-native-tests.log](engine-native-tests.log) |
| Gate 1b | exit 0 | [gate1b-all-targets.log](gate1b-all-targets.log) |
| 行数门禁 | 8223 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办（NC04）

- [ ] DSL 选择语句关键字确认后补「挂起 → 续跑 → 故障」覆盖（当前 1 项 ignored）
- [ ] 非法效果（capability/无效 host 请求）、无效 token（`ChoiceOption::Tag` 超域已回滚但未单测）、恢复后故障
- [ ] 预算/内存/栈限额的逐类触发用例（本轮只覆盖 `OutOfFuel`）
- [ ] 故障对**随机流**的影响核对（本轮只验证状态摘要与修订号；RNG 位点需专门用例）
- [ ] 产品侧展示：`RejectReason::VmFault` 经协议传到客户端 UI（NC03/NC09）
- [ ] 段内已分片提交（`Suspended`）后发生故障的回滚路径用例（代码已走检查点回滚，未单测）

## 6 追加：挂起→续跑 与 恢复后故障（手工字节码，2026-10-18）

DSL 目前只有 `effect`/`if`/`loop`/`let` 语句（`parse.rs::parse_stmt`），**没有选择语句**，故 VM 的 `HOST_CHOICE` 路径无法从 DSL 触发；本批用**手工构造的 `BytecodeModule`** 覆盖，新增 `crates/engine-native/tests/nc04_vm_resume.rs`（2 项）：

| 验收项 | 模块 | 用例 | 结果 |
|---|---|---|---|
| 选择挂起 → 续跑 → 段结算完成、无故障、VM 选择被消费 | push token 0/1 → `HOST_CHOICE 2` → `HALT` | `nc04_pause_resume_completes` | 通过 |
| **恢复后故障**：续跑进入宿主调用边界失败（无效效果 tag）→ `VmFault` + **整段回滚**（摘要/修订号不变）+ 故障槽清空 | push token 0/1 → `HOST_CHOICE 2` → `HOST_EFFECT_EXT 0xFFFF` | `nc04_fault_after_resume_rolls_back` | 通过 |

实测：定向 **2/2**；engine-native 全量 **1112/0/2**；Gate 1b exit 0；行数门禁 8228 文件 ≤2000 行。

说明：各效果 tag 的参数出栈契约不同，效果**落账**已由 DSL 版 `nc04_normal_completion` 覆盖，本批只覆盖挂起/续跑机制与恢复后故障的事务语义（不重复覆盖同一语义）。
