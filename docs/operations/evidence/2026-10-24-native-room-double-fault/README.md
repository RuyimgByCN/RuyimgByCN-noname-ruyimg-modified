# 最终决定与撤销补偿双重失败：停写、恢复与重试（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加本轮未提交工作树；规则 **v322**、引擎归档 **v105**。本轮修改服务端 actor、房间 JSON 与回归测试，没有修改内核、前端、WASM 或引擎归档格式。当前结论集中于[进度主稿](../../../architecture/native-progress.md#native-room-double-fault-2026-10-24)。

## 本轮修复

原实现只标示 `decision_uncertain`，actor 仍恢复旧 RAM，并在下一次修改前强制保存旧 RAM。若双重失败已留下合法 `Committed`，这会覆盖已提交候选；重启后原有网络幂等 HashMap 又会丢失，同 key、revision 0 的重试可能再次执行。[真 WS 红灯](ws-double-red.log)先证明双重失败后下一条请求仍得到 Accepted，而非停止变更；各残留阶段另由 store 矩阵核对。

- **停止不确定房间的后续写入**：最终决定与撤销补偿都失败时，actor 回到最后确认 RAM，但标记必须重启。提交、入座、旁观、选将、Recover 与修复保存都在写盘前拒绝；已有连接仍可查询旧确认投影。这份投影不代表磁盘决定已撤销。普通单故障仍沿用已验证的修复路径。
- **按持久化决定恢复**：新 bootstrap 验证 Prepared／Committed、房号、UUID、正文／日志完整性。Prepared 恢复旧确认状态或原先不存在；Committed 恢复候选，不能被旧 RAM 的后续重试覆写。恢复发布前仍经过正常保存步骤。
- **接受结果与候选一起落盘**：新增可选 `command_receipts`，记录重连令牌、玩家、非零 key、完整 `CommandKind` 和 Accepted 结果。收据纳入 checkpoint、回滚、状态导出、恢复和原有完整校验；空字段不序列化，保留旧 JSON 的 checksum 兼容。
- **同键重试按收据判定**：Committed 收据存在时只回原结果和当前投影，不再次执行、写盘或广播；Prepared 没有候选收据时可重试执行一次。相同身份／key 换命令内容明确拒绝，其他玩家或回收后的新座位身份不能继承收据。正常后续命令、重连、选将及现有最新快照／WAL Recover 保留收据。

实现见[停写判定](../../../../apps/game-server/src/room_native_actor/durability.rs)、[收据模块](../../../../apps/game-server/src/room_native/receipts.rs)和[本轮服务端增量](server-incremental.diff)。未增加依赖或生产故障开关。

## 双重故障与真 WS 重启

实例级 `fail_next_write_with_compensation` 组合三种决定故障与三种撤销故障。撤销故障使用独立的 Prepare 阶段参数，确实命中第二次写入，不是重复消费同一个单故障：

| 决定故障 | 撤销写入／文件同步失败 | 撤销替换后目录同步失败 |
| --- | --- | --- |
| DecisionWrite | Prepared：旧状态／不存在 | Prepared：旧状态／不存在 |
| DecisionSync | Prepared：旧状态／不存在 | Prepared：旧状态／不存在 |
| DecisionDirectorySync | Committed：完整候选 | Prepared：旧状态／不存在 |

Store 测试覆盖原房间／首次保存 **18 组**、两次 fresh store 只读扫描、字节与房号保留，以及故障计划只消费一次。[最终 store 定向](store-double-faults-final.log) **23/23**。

[真 TCP/WS 矩阵](../../../../apps/game-server/src/native_ws_e2e/double_failure.rs)使用正式 `shahai-swd` + `wudian`、两位玩家及旁观者，在私人选择处遍历 **9 组**故障。错误后没有 Accepted／新状态广播，同进程重试与新增旁观均拒绝，正文／日志 bytes 不变；销毁 runtime，停用并移除原安装包，再启动生产 bootstrap。Prepared 恢复 **3**，Committed 恢复 **4**，完整状态／RNG／选择／动作／原包副本与角色保持（仅允许既有授权代际 +1）。同 key **904** 只执行或重播一次，换内容拒绝、其他玩家不串用，下一命令到 **5** 后旧收据仍可重播。首次保存另遍历 **9 组**，分别恢复 **0／1** 房间，失败 ID 保留、新房号 **2**。

[真实进程中断回归](../../../../apps/game-server/src/native_ws_e2e/commit_crash.rs)的 test-only hook 在决定目录同步失败、撤销写入也失败后直接 **exit 87**，发生在 actor 回滚、错误响应和析构之前。磁盘留下 Committed revision **4**；新服务子进程按原副本恢复，同 key **1004** 返回原 Accepted 而不推进。再执行到 **5**、再启动另一服务子进程，旧 key 仍回原 revision **4** 结果和当前 revision **5** 投影。原 exit 86 的候选 rename 中断回归也保留执行。

完整 WS 定向[最终日志](ws-double-full-01.log)：**12 passed / 0 failed / 0 ignored**，其中 **11 个场景测试 + 1 个子进程入口**；矩阵循环不是独立 Cargo 测试数，子进程 exit 86／87 不另计通过。首次并行编译曾因新增单测误用 `StartGame.seed` 参数名失败，改用真实 `rng_seed/ruleset` 后重跑；[fixture 编译日志](ws-double-initial-01.log)与产品红灯分开保留，不改记为绿。

## 完整回归与门禁

| 检查 | 当前结果与证据 |
| --- | --- |
| 收据定向 | **3/3**，恢复／重连／回滚／换座位身份／错误收据／旧空字段兼容；[日志](receipt-targeted-01.log) |
| 七 crate 完整离线回归 | **1932 passed / 0 failed / 71 ignored，exit 0**；[原始输出](cargo-regressions.log)、[命令与起讫](cargo-regressions.json)、[逐目标计数](cargo-counts.json) |
| 正式服务程序构建 | **exit 0**；[日志](server-build.log)、[起讫](server-build.json) |
| 前端／型别 | **70 passed / 0 failed / 0 skipped**，typecheck **exit 0**；[测试](frontend-test.log)、[型别](frontend-typecheck.log) |
| 既有原生 WASM / Worker | **44/44**；[日志](native-worker-smoke.log) |
| Chrome 正式服务 SIGKILL | **10 组通过、pageerror 0**；[日志](browser-crash-room.log)、[报告](browser-report.json)、[截图](browser-ws-restored.png) |
| 完整 preflight | **20/20，0 失败／0 跳过／0 豁免，exit 0**；[摘要](preflight-full-01.log)、[全部子命令输出](preflight-full-01-raw.log)、[逐项结果](preflight-full-01.json) |
| 写入后文档／源码门禁 | **7/7 通过**；[最终日志](final-doc-gates.log)，包含行数单测、全量链接、严格文档行号、旧值、索引、行数与 diff 检查 |

完整 Cargo 于 **16:52:39–16:56:18 +08:00** 执行：`engine-native` **1547/0/71**、`game-server` **281/0/0**、`engine-native-wasm` **36/0/0**、`content-native-loader` **32/0/0**、`content-runtime` **33/0/0**、`engine-protocol` **1/0/0**、`rule-api` **2/0/0**（passed/failed/ignored）。保留现有 `#[path]` 重复入口，不作唯一场景数；71 个忽略守卫不计通过，Windows 专属测试本机未编译运行。

```bash
PATH=/Users/ruyimg/.cargo/bin:/usr/local/bin:$PATH cargo +stable test --offline \
  -p game-server -p engine-native -p engine-native-wasm -p content-native-loader \
  -p content-runtime -p engine-protocol -p rule-api --no-fail-fast
PATH=/Users/ruyimg/.cargo/bin:/usr/local/bin:$PATH cargo +stable build --offline -p game-server
```

完整 preflight 于 **16:44:58–16:50:25 +08:00** 调用未修改的 `preflight.main()`，仅透明记录原 subprocess 输出；脚本单测 **180 tests / OK**，行数门禁 **8576 文件 ≤2000 行**。环境为 macOS、Cargo/rustc **1.95.0**、Node **20.17.0**、默认 Python **3.10.10**，见[环境记录](environment.json)。原有编译警告与 ResourceWarning 保留，不称零警告。

[既有浏览器脚本](browser-crash-room-smoke.mjs)另以新正式服务程序和前轮已验证 dist 外部副本实跑普通写入失败、实际 SIGKILL、自动回房及继续游戏；[执行起讫](browser-crash-room.json)、[初次服务](browser-server-initial.log)、[重启服务](browser-server-restored.log)。该 SIGKILL 发生在错误响应后；双重故障的精确中断由 exit 87 测试进程覆盖，均不等于真断电。未重新构建 WASM／前端产物。

## 修改保护与边界

开始备份 **47024 个既有路径**、dirty/untracked **11404 条**，实体与差异备份在 `/tmp/noname-next-room-double-fault-20261024/`。最终仅 **11 个既有服务端／测试／进度文件**增量变化，其余 **47013 路径**内容／模式／符号链接属性一致，意外变更 **0**；新增两个职责模块／测试与本证据目录。[1067 文件受验清单](source-manifest.json)最终摘要不变，三个生成包目录 **18 文件**与原 dirty dist **46 文件**均和本轮实体备份一致。**HEAD/index 不变，未暂存、未提交、未推送**；详见[保护核对](preservation.json)。

- 双重失败时需要重启服务，错误不等于确定已撤销；有效磁盘决定可能是 Prepared 或 Committed。恢复后以收据判定非零 key 的 `SubmitCommand`，不把网络回覆与磁盘提交说成原子操作。
- 收据覆盖可恢复的进行中房间。key **0** 仍是明确不启用幂等；CreateRoom／ChooseGeneral 没有请求 key，不提供同键重播。旧档无收据时无法补回历史回覆，终局清理后不会为已结束房间提供收据查询。
- 保留会话内全部已接受非零 key，未设可使旧请求重新执行的淘汰规则；收据容量、每次完整保存的 I/O 成本和延迟尚未压测，不声称性能收益或有界成本。
- 未验证真断电、存储控制器／网络文件系统、Windows 实机／跨平台 CI、跨机/TLS；沿用单进程单房 actor 假设。完整原局包校验、撤销政策与新正文／日志一起备份的要求仍有效；本机双包样本不代表 37 来源或全平台验收。
