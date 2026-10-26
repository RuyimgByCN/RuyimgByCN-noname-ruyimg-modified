# 房间候选 rename 后的提交判定与重启恢复（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加本轮未提交工作树；规则 **v322**、引擎归档 **v105**。本轮只修改服务端存储与回归测试，未改内核、前端、WASM 或引擎归档格式。当前状态集中于[进度主稿](../../../architecture/native-progress.md#native-room-commit-2026-10-24)。

## 已修复的恢复判定

原实现将完整候选替换到 `<id>.json` 后，目录同步失败会回协议错误并恢复 RAM；此时若立即中断，重启却会按合法 checksum 接受候选。首次建房同样可能在返回错误后被重启恢复。两条真 WS 反例分别见[未确认命令红灯](ws-uncertain-rename-red.log)、[首次保存红灯](ws-uncertain-initial-red.log)，另有[store 红灯](store-red.log)。

现在先持久化 `<id>.json.journal` 的 `Prepared`，固定上一份确认 JSON 原始字节或原先不存在；然后写入、同步、rename 候选并同步目录，最后持久化 `Committed` 决定。日志包含版本、房号、UUID、完整候选 BLAKE3 及覆盖整份日志的校验和；正文增加 `commit_id`，公开 `StoredRoom` 接口和引擎归档格式不变。

- `Prepared` 只恢复经验证的旧确认内容或原先不存在；完整、较新且 checksum 正确的候选也不提升。扫描保持只读，候选／日志与房号保留。
- `Committed` 必须匹配候选 UUID 与全部字节，包含密码哈希、成员／令牌、随机状态和原局包副本；正文缺失或损坏时拒绝回退旧局。
- 新正文缺日志、日志截断／版本或身份错误／陈旧／symlink 均隔离该房间，不按旧格式猜测恢复。无日志且无 `commit_id` 的旧 JSON 保持兼容，迁移也有撤销依据。
- 再次保存前，先在旧 `Prepared` 仍有效时固定旧确认正文，再开始新交易；修复连续失败覆盖唯一回退依据的缺口。终局清理先移除日志，残留新正文无法复活旧进行中状态。
- 提交决定写入／同步失败时尝试持久化撤销；该补偿若也失败，错误明确标示 `decision_uncertain`，不把所有协议错误解释成确定未执行。

实现按职责分在 store 与 [journal 模块](../../../../apps/game-server/src/native_room_store/journal.rs)，单测在 [commit_tests](../../../../apps/game-server/src/native_room_store/commit_tests.rs)。本轮实际增量见[服务端差异](server-incremental.diff)。

## 已完成的验证

| 检查 | 当前结果与证据 |
| --- | --- |
| Store 故障／恢复定向 | **21/21**；[最终日志](store-final.log)，涵盖原四类候选故障、六类新日志故障、首次保存、两次只读扫描、连续失败、旧档迁移、完整绑定与终局清理 |
| 真 TCP/WS 定向 | **9 passed / 0 failed / 0 ignored**；[最终日志](ws-regressions-04.log)，包含 8 个实际场景测试与一个仅在子进程启用的测试服务入口 |
| 七 crate 完整离线回归 | **1904 passed / 0 failed / 71 ignored，exit 0**；[完整日志](cargo-regressions.log)、[起讫与命令](cargo-regressions.json)、[逐目标计数](cargo-counts.json) |
| 正式服务程序构建 | **exit 0**；[构建日志](server-build.log)、[起讫](server-build.json) |
| 前端完整测试／型别 | **70 passed / 0 failed / 0 skipped**，typecheck **exit 0**；[测试](frontend-test.log)、[型别](frontend-typecheck.log) |
| 既有原生 WASM / Worker | **44/44**；[日志](native-worker-smoke.log) |
| Chrome 正式程序 SIGKILL | **10 组通过、pageerror 0**；[日志](browser-crash-room.log)、[报告](browser-report.json)、[截图](browser-ws-restored.png) |
| 完整 preflight | **20/20，0 失败／0 跳过／0 豁免，exit 0**；[摘要](preflight-full-01.log)、[全部子命令输出](preflight-full-01-raw.log)、[逐项结果](preflight-full-01.json) |
| 写入后文档／行数门禁 | **7/7 通过，exit 0**；[最终日志](final-doc-gates.log)、[逐项结果](final-doc-gates.json)，包含行数单测、全量链接、文档行号、旧值、索引、源码行数及 diff 检查 |

完整 Cargo 于 **16:17:09–16:21:08 +08:00** 执行：`engine-native` **1547/0/71**、`game-server` **253/0/0**、`engine-native-wasm` **36/0/0**、`content-native-loader` **32/0/0**、`content-runtime` **33/0/0**、`engine-protocol` **1/0/0**、`rule-api` **2/0/0**（passed/failed/ignored）。保留现有 `#[path]` 重复入口，不把执行次数当唯一场景数；71 个忽略守卫不计通过，Windows 专属测试本机未编译运行。子进程的 exit 86 没有被另算一项通过。

```bash
PATH=/Users/ruyimg/.cargo/bin:/usr/local/bin:$PATH cargo +stable test --offline \
  -p game-server -p engine-native -p engine-native-wasm -p content-native-loader \
  -p content-runtime -p engine-protocol -p rule-api --no-fail-fast
PATH=/Users/ruyimg/.cargo/bin:/usr/local/bin:$PATH cargo +stable build --offline -p game-server
```

环境为 macOS、Cargo/rustc **1.95.0**、Node **20.17.0**；preflight 默认 Python **3.10.10**，见[环境记录](environment.json)。完整 preflight 于 **16:08:34–16:15:22 +08:00** 运行未修改的 `preflight.main()`，仅透明记录原 `subprocess.run` 输出；脚本单测 **180 tests / OK**，行数门禁 **8574 文件 ≤2000 行**。原有编译警告与 ResourceWarning 保留，不声称零警告。最终补写文档后另跑上述文档／行数门禁。

## 真 WS 中断与续局

新增两项生产 `bootstrap/build_router` 真 WS 测试，每项遍历 `DirectorySync`、`DecisionWrite`、`DecisionSync`、`DecisionDirectorySync`：在私人弃牌选择处制造失败，确认磁盘仍有完整较新候选，不作重试／重连／修复保存便销毁 runtime。停用并移除原安装包后，新 bootstrap 只恢复旧确认 revision **3**、完整私人投影／动作／选择／令牌及原局双包副本，同 key **704** 重试接受并推进至 **4**。解码归档逐字段比较，只允许既有引擎恢复规则规定的 `auth_generation +1`。首次保存的四个故障均恢复 **0** 房间，失败 ID 仍保留，新房号 **2**。

[进程中断测试](../../../../apps/game-server/src/native_ws_e2e/commit_crash.rs) 使用独立测试服务子进程、真实 TCP/WS、两玩家及旁观者；`cfg(test)` 实例故障在候选 rename 返回后、目录同步和提交决定之前立即 `process::exit(86)`，不经过 actor 回滚、错误响应或析构。新子进程按原冻结包恢复 revision **3** 的私人选择与角色／令牌，同 key **804** 提交成功到 **4**。见[单项日志](ws-process-crash-01.log)；最终完整回归也执行同一用例。生产构建没有环境故障开关。

Chrome 的[既有脚本](browser-crash-room-smoke.mjs)另以新构建正式服务程序和前轮已验证生产 dist 外部副本执行：双玩家／旁观、隐私、同步、刷新，真实不可写目录失败后对服务进程 **SIGKILL**，重启自动回房并续局。坏 `999.json` 与两类 temp 保持，新房号 **1000**。本次执行 **16:17:51–16:18:01 +08:00**；[浏览器元资料](browser-crash-room.json)、[初次服务](browser-server-initial.log)、[重启服务](browser-server-restored.log)。这里的 SIGKILL 发生在错误响应后；精确命中 rename 后写入区间由上述测试子进程提供，二者均不等于真断电。

初轮 WS／服务端完整命令的新增 fixture 曾错误要求归档 bytes 完全一致，忽略既有授权代际轮换；[WS 首跑](ws-regressions-01.log)、[服务端首跑](server-compatibility-01.log)保留失败，不记全绿。修正为完整解码状态只允许预期授权代际递增后，才完成最终定向及七 crate 重跑。原有 8 类冻结内容负例改经 store 正常提交并断言存储校验通过，确保仍检验内容／撤销政策，而非因缺日志提前拒绝。

## 修改保护与边界

本轮开始备份 **46977 个既有路径**，初始 dirty/untracked **11357 条**；备份在 `/tmp/noname-next-room-commit-decision-20261024/`。逐路径保留原内容／模式／符号链接属性，仅 **6 个既有服务端／测试／进度文件**增量变化，其余 **46971 路径**原样；新增三个职责模块／测试与本证据目录，意外变更 **0**。受验 [1065 文件清单](source-manifest.json)最终摘要一致。三个生成包目录 **18 文件**与原 dirty dist **46 文件**均与本轮实体副本一致，未重新构建或覆盖 WASM／前端产物。**HEAD/index 不变，未暂存、未提交、未推送**；详见[逐项保护记录](preservation.json)。

- 候选 rename 后的窗口已有持久化回退判定；最终提交决定已经可见时，若决定同步和撤销补偿都发生真实 I/O 失败，或进程在最终决定期间中止，仍可能存在“决定已成立但调用方没收到成功”的结果不确定性。不能承诺所有错误／未回覆命令都必然回滚；跨重启请求幂等账本仍待补。
- 未验证真断电、存储控制器／网络文件系统、Windows 实机与跨平台 CI、跨机/TLS；保留原 Windows write-through 与目录建立／删除保证边界。本轮未重建 WASM，Worker 验证使用已保留产物。
- 备份／迁移新房间目录时需连同 `.json.journal`；缺失时明确拒绝恢复。旧伺服器不理解新判定，不能据本轮测试宣称可安全降级读取新目录。
- 沿用单进程、每房单 actor 写入假设；未加跨进程锁／后台队列／持久 WAL，未测 I/O 延迟收益。正式双包样本不代替 37 来源、完整规则等价或全平台验收。
