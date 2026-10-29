# Agent 规范二次审查与 v1.2 修订（2026-10-28）

本目录记录 `NONAME_NEXT_AGENT_SPEC.md` 从 v1.1 到 v1.2 的审查依据。**初批只改上层规范并保存审查结论；用户确认后的登记批只在剩余台账与进度主稿写入两项登记（§4）。两批都没有修改产品代码或规则基准，没有升级规则／格式／ABI，也没有提交或推送。** 第一次审查（v1.0→v1.1）见[同日审查证据](../2026-10-28-agent-spec-review/README.md)；当前状态仍以[进度主稿](../../../architecture/native-progress.md)和[剩余台账](../../../architecture/native-remaining-ledger.md)为准。

## 1. 基准

- HEAD：`ec672a1d3c413d39c9972ee02118ae92a21223ee`，与第一次审查相同；规则 v323、存档 v105、全局原生 ABI 753（按剩余台账核对）。已跟踪工作树无改动，已有未跟踪目录 `2026-10-28-agent-spec-review/`、`2026-10-28-nc07-lianyaohu-hash/` 原样保留。
- 规范位于子仓库上层：[NONAME_NEXT_AGENT_SPEC.md](../../../../../NONAME_NEXT_AGENT_SPEC.md)。v1.1 SHA-256 `4f65eba1c712f66a1a1002ad96a360acd9db9d20d19d74afa2261b8b3366d021`，v1.2 见 [checks.log](checks.log)。
- 结构查询使用 CodeGraph，文本与链接查询使用 rg；本批未运行 Rust 或前端测试。

## 2. 结论

v1.1 已纠正 WASM 插件后端、交付范围、重建工作区三类方向性问题，可以采纳为**实施约束与验收检查表**，但不能作为架构重设计。它的价值在于把收敛方案中分散的验收要求压缩为可检查的条款。保留的条款：§0 强制约束，§3.3 不变量，§3.4 单字段反例，§4.4 第 7 步恢复轨迹比对，§6.2 正式安装生命周期，§8.3 投影双向检查（不泄密，也不丢授权字段），§7.3“hash 相等不充分”，GT-011～014，§14 优先级。

v1.1 的其余问题是示意内容与现有源码不符，或与主稿重复。照字面执行会引出同义 API、新阶段模型或越权的规则改动。v1.2 按下表修订。

## 3. 审查项与 v1.2 修订

| # | v1.1 位置 | 源码或主稿依据 | v1.2 处理 |
|---|---|---|---|
| 1 | §3.2 `EngineOutcome`／`RulesEngine` 示意 | [engine.rs](../../../../crates/engine-native/src/engine.rs) 实际 `CommandOutcome` 为 Accepted／Pending／Rejected／**Suspended**；NC03 要求显式 **Fault**，示意两者都缺；[room_native.rs](../../../../apps/game-server/src/room_native.rs) 把 Pending／Suspended 映射为协议 Accepted | 换成现状节选，并按 NC03／NC04 列出补齐要求；禁止并建同义 trait |
| 2 | §4.1 `ResolutionFrame` 字段表、§4.2 七阶段“新架构的基础模型” | [event.rs](../../../../crates/engine-native-kernel/src/event.rs) 已有 `Frame` 与 `Stage::{BeforeTriggers, Commit, AfterTriggers, Intrinsic, Done}`；v1.1 的 `Commit` 在末尾，与现有中间阶段 `Commit` 同名异义 | 映射到现有载体；七阶段降为审查用语，并去掉 `Commit` 歧义 |
| 3 | §4.3 玩家自选发动顺序、GT-002 | [resolution.rs](../../../../crates/engine-native-kernel/src/resolution.rs) `plan_triggers` 按 `(Timing, −priority, optional, 顺座步数, SkillDefId)` 固定排序；上游 [arrangeTrigger](../../../../../archive/upstream/noname/apps/core/noname/library/element/content.ts)（约 4017–4058 行）对同角色同优先级的多个非 silent 技能调用 `chooseControl`。规则基准“当前声明与验证边界”的“触发全序”行和进度主稿“触发全序对照”行已于 2026-10-16 登记这项差异，并定案保留确定性全序（[对照证据](../2026-10-16-trigger-order-upstream/README.md)）。**本页初稿误称“未登记”**：当时只检索了“arrangeTrigger／选择下一个触发／发动顺序”，没有命中该行“同优先级交互选择”的写法，同日已更正。定案理由“上游交互并列不可无交互复现”正是收敛方案 NC08 要求复核的类型，这项复核此前没有登记 | 保留要求；写明现行定案与 NC08 复核路径，改判须立 ADR 并取得升号授权，禁止顺手修改 |
| 4 | §3.1 强类型 ID 清单 | [ids.rs](../../../../crates/engine-native-kernel/src/ids.rs)：`PlayerId = u8`，`CardId = u32` 兼作定义与实例编号，二者已进入摘要与编码；没有混用反例 | 不整体 newtype 化；只保留边界规则（隐藏牌不暴露可反查牌面的 `CardId`） |
| 5 | §8.2 `gameId/commandId/requestId` JSON 信封、§3.3 去重 | [messages.rs](../../../../crates/engine-protocol/src/messages.rs) 已有 `SubmitCommand{header, command, idempotency_key, replay}`；[receipts.rs](../../../../apps/game-server/src/room_native/receipts.rs) 按 `(reconnect_token, idempotency_key)` 保存回执；内核 `dedup_id` 为高水位；服务端用成员座位提交 | 指向现有协议与两层去重；不新增同义字段 |
| 6 | §4.4 第 6 步 `TimeoutDecision` | 旧线 `engine_api::CommandKind::TimeoutAction` 已存在；原生房间 rg `TimeoutAction／超时` 只找到离线座位惰性回收和 AI 50 ms 期限，**没有选择超时处理**；剩余台账 NC09 行未登记此项 | 改为引用现有形态，并列为 §1.3 未决参数，归 NC09 |
| 7 | §0 第 3 条“核心不得依赖系统时间” | [ai.rs](../../../../crates/engine-native/src/ai.rs) 原生分支用 `Instant` 计 AI 期限，WASM 为 `None`（PA09） | 范围限定为规则结算路径；§7.2 写明 AI 结果靠命令记录回放 |
| 8 | §5.2 YAML 概念语法 | 实际 DSL 为 `.skill`，例如 [stdkuangfu.skill](../../../../content/standard/stdkuangfu.skill) | 换成原样真实样例 |
| 9 | §5.4“每个标准技能正常／非法／边界三测” | PA12 盘点 7,614 技能；逐技能套模板不可行，测试会流于凑数 | 按原语、谓词族、触发／限次族分层；含选择与续行的技能必须验恢复；不缩减 37 来源验收 |
| 10 | §8.4“授权增量或私有快照” | PA10 已裁决维持整份视图（剩余台账 PA10 行） | 改为私有快照，增量待 PA08 触发条件 |
| 11 | §11.3“架构+安全双评审” | 仓库没有这两类评审人；实际门槛是用户对 ADR、协议、升号的授权（剩余台账 2026-10-27 21:50 裁决、[裁决登记](../2026-10-28-user-decisions/README.md)） | §0 第 10 条明确授权门槛与版本号占用；§11.2 改为单列验证证据 |
| 12 | §10 六阶段表、§11.1 角色、§11.4 提示词 | 与[收敛方案](../../../architecture/native-convergence-plan.md)第 4 节和[执行提示词](../../native-convergence-agent-prompt.md)重复，措辞已出现偏差 | §10 只保留与阶段的关系；删除角色表；提示词限定为单工作包使用，并加入裁决登记和授权门槛 |
| 13 | §13 GT-001～010 无测试映射 | 已有 `nc08_save_restore_matrix.rs`、`nc09_projection_and_errors.rs`、`nc08_cross_target_real.rs`、`nc04_vm_fault.rs`、`native_ws_e2e/` 等入口 | 改成表格，加入“候选入口”，并注明尚未核对覆盖；要求先登记映射再补断言 |
| 14 | §7.3 RNG 与浮点 | [rng.rs](../../../../crates/engine-native-kernel/src/rng.rs) 为 xorshift64*；`content/pile.rs` 等规则路径使用 f64 基本运算与 `round` | 写明 `RuleRng`，规则路径浮点只允许基本运算与舍入 |
| 15 | GT-011～013 与已有授权 | 裁决登记第 4、6、7、8 项已授权对应修复 | 在 §8.3 与 GT 表中链接授权条目，避免重复请示或越界 |

## 4. 主稿登记（同日，用户确认后）

用户确认登记第 3、6 项后，本批只写入登记，不改代码、规则版本或已有定案：

- [剩余台账](../../../architecture/native-remaining-ledger.md) NC08 行：登记“触发全序”定案的复核待办。差异本身已在规则基准登记，规则基准未改。
- 剩余台账 NC09 行：登记选择窗超时缺失及完成条件（时限与到时动作、显式命令进入命令流与 WAL、断线／重连／恢复／重复提交覆盖、新增命令或协议字段前先请示）。补充核实：旧线 `engine-core` 收到 `TimeoutAction` 时按取消处理选择；上游联机按 `choose_timeout` 倒计时，到时调用 `ui.click.auto()`（`archive/upstream/noname/apps/core/noname/game/index.js` 约 2020–2036 行）；`apps/game-server/src/native.rs` 中的 50 ms 是自动对局接口的 AI 规划期限，与房间超时无关。
- 剩余台账“2026-10-28 进展登记”和[进度主稿](../../../architecture/native-progress.md)各追加一行摘要。
- 规范 §1.3、§4.3、§4.4、§4.5、§10 与 GT-002 已改为指向上述登记。

## 5. 仍待用户决定

- **规范的位置**：规范在子仓库外，不受 Git 版本管理，[执行提示词](../../native-convergence-agent-prompt.md)和 AGENTS.md 的文档表也没有引用它。只打开 `noname-next` 的 Agent 看不到它。迁入仓库（如 `docs/architecture/`）并更新引用，属于路径与导航变更，留待用户决定。

## 6. 核验

命令与输出见 [checks.log](checks.log)；主稿登记批的检查追加在同一文件末尾。
