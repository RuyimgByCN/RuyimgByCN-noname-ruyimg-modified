# 服务端房间写入失败与异常中断恢复（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加本轮未提交工作树；规则 **v322**、原生归档 **v105**。本轮改服务端房间保存/恢复，没有修改内核、WASM、前端源码或归档格式版本。当前结论见[进度主稿](../../../architecture/native-progress.md)。此前原局包副本能力保持，并纳入故障重启测试。

## 实现与失败语义

- 保存使用同目录、`create_new` 独占的 UUID 临时文件，`write_all`、文件 `sync_all`、原子替换；Unix 再同步房间目录，建立目录时也同步祖先，重试不会漏掉此前建立但未同步的目录。只清理本次拥有的失败临时文件，保留旧 final 与其他中断证据。
- 配置保存后，入座、重连、旁观、惰性回收、命令、同 revision 选将及 Recover 先取得 mutable session 检查点，保存成功后才更新 actor 成员/WAL/幂等结果及回覆、广播。失败返回错误并还原引擎/RNG、成员/令牌/座位和 dedup，不回成功、不发布候选状态；没有配置保存时保留内存房间路径。
- rename 后目录同步失败明确标为“替换已可见、耐久尚未确认”。内存仍回到最后确认状态，后续修改前必须强制重新保存该确认状态；修复再次失败仍拒绝操作，不能因状态签名相同而跳过。**若此时立即中断，重启可能读到完整但未确认的候选版本**；不会把这种错误宣称为磁盘已经回滚。
- 配置新房间先落盘再发布 `RoomCreated`。终局先保存终局再尝试删除，清理失败留下的终局记录在启动时不发布。恢复/迁移保存失败也不发布房间。
- 启动扫描隔离截断、checksum 错误、非普通文件和文件名/载荷房号不符的记录，健康房间继续恢复；问题文件与房号保留。旧式 `.json.tmp` 与 UUID 临时文件均不提升为 final，即使其中内容完整且较新。扫描目录失败时拒绝分配新房号；`u64::MAX` 不会溢出回用旧房号。
- Recover 故障回归还发现重放 dedup 高水位可能与下一 live command 碰撞；在共享 `replace_engine` 接口同步下一序号，检查点一并支持失败回滚。没有修改规则、随机流算法或黄金哈希。

Windows 保留临时文件 `sync_all`，关闭文件后使用同目录 `MoveFileExW(REPLACE_EXISTING | WRITE_THROUGH)`，不启用 copy/delete fallback；路径 canonicalize 后编码为经 NUL 检查的 UTF-16，保留 Unicode/长路径。失败且不能确认源文件仍存在时按提交不确定处理。移除不符合权限要求的只读目录 flush，依据 [FlushFileBuffers 权限要求](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-flushfilebuffers)与 [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)。**Windows mkdir/unlink 不承诺 Unix 目录 fsync 等价性，两项 Windows 专属测试本机未执行**；详见[平台查核记录](windows-storage-design.md)。

## 已完成的验证

| 验证 | 命令 / 范围 | 结果与原始证据 |
|---|---|---|
| store 故障 | `cargo +stable test --offline -p game-server --bin game-server native_room_store::tests -- --nocapture` | **10/10**；[最终日志](store-green-final.log) |
| actor 定向 | `cargo +stable test --offline -p game-server --test nc09_native_actor --test nc09_native_durability --test nc09_native_room_session -- --nocapture` | **62 passed / 0 failed / 0 ignored**；[最终日志](actor-targeted-final.log) |
| 真 TCP/WS | `cargo +stable test --offline -p game-server --bin game-server native_ws_e2e -- --nocapture` | **5 passed / 0 failed**，原两项内容锁与新增三项故障/扫描/房号回归；[日志](ws-durability-final.log)、[起讫](ws-durability-final.json) |
| 终局启动 | `cargo +stable test --offline -p game-server --bin game-server lobby::tests::finished_saved_room -- --nocapture` | **1/1**；终局不发布、健康局恢复、房号保留；[日志](lobby-terminal-test.log)、[起讫](lobby-terminal-test.json) |
| 七 crate 完整回归 | `cargo +stable test --offline -p game-server -p engine-native -p engine-native-wasm -p content-native-loader -p content-runtime -p engine-protocol -p rule-api --no-fail-fast` | **1845 passed / 0 failed / 71 ignored，exit 0**；[完整输出](cargo-regressions.log)、[起讫](cargo-regressions.json)、[逐 target 汇总](cargo-counts.json) |
| 实际服务程序 | `cargo +stable build --offline -p game-server` | **exit 0**；[日志](server-build.log)、[起讫](server-build.json) |
| 前端完整测试 | web-client 内 Node 20 `pnpm test` | **70 passed / 0 failed / 0 skipped**；[日志](frontend-test.log)、[起讫](frontend-test.json) |
| 前端型别 | web-client 内 `pnpm typecheck` | **exit 0**；[日志](frontend-typecheck.log)、[起讫](frontend-typecheck.json) |
| 既有原生 WASM / Worker | web-client 内 `/usr/local/bin/node --import ./scripts/ts-register.mjs scripts/native-worker-smoke.mjs` | **44 checks / 0 failed**；[日志](native-worker-smoke.log)、[起讫](native-worker-smoke.json) |
| Chrome 真程序 SIGKILL | [浏览器脚本](browser-crash-room-smoke.mjs)，新构建服务程序 + 前轮已验证生产 dist 外部副本 | **10 组通过，pageerror 0**；[日志](browser-crash-room.log)、[起讫与 dist 路径](browser-crash-room.json)、[报告](browser-report.json)、[续局截图](browser-ws-restored.png) |
| 行数与单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **单测 1/1，8571 文件 ≤2000 行**；[单测](source-lines-unit.log)、[门禁](source-lines.log)、[起讫](source-line-checks.json) |
| 完整 preflight | `python3 -u -B scripts/preflight.py` 完整模式 | **20/20 通过，0 失败 / 0 跳过 / 0 豁免，exit 0**；[摘要](preflight-full-01.log)、[全部子命令输出](preflight-full-01-raw.log)、[逐项结果与起讫](preflight-full-01.json) |

七 crate 完整命令于 **15:36:41–15:39:54 +08:00** 执行：`engine-native` **1547/0/71**、`game-server` **194/0/0**、`engine-native-wasm` **36/0/0**、`content-native-loader` **32/0/0**、`content-runtime` **33/0/0**、`engine-protocol` **1/0/0**、`rule-api` **2/0/0**（passed/failed/ignored）。包含现有 `#[path]` 重复执行与新增 store 单测的重复编译入口，不作唯一场景数；71 个忽略守卫未被算作通过。Windows 专属两项本机未编译运行。既有警告保留，不声称零警告。环境为 macOS、rustup Cargo/rustc **1.95.0**、Node **20.17.0**、pnpm **10.34.3**；preflight 使用默认 Python **3.10.10**，详见[环境记录](environment.json)。

## 先红后绿与真 WS 覆盖

- 旧 actor 遇真实不可写目录仍回 Accepted，分别由[actor 反例](actor-write-red.log)与[真 WS 反例](ws-write-red.log)证伪。旧 store 的固定临时路径会被残留目录阻塞，并放行文件名/房号错配，见[store 反例](store-red.log)。
- Recover 新反例在恢复成功后被下一条 live command 的 dedup 碰撞拒绝，见[修复前日志](actor-targeted-03.log)；修正共享高水位后完整定向与七 crate 回归通过。没有只改单项期望值。
- 首次壳层隔离测试使用了 writer 不会生成的任意 temp 后缀，修正为真实 UUID/旧式 temp 名称后通过；[该 fixture 失败](ws-crash-isolation-first.log)原样保留，与上述产品缺陷反例分开记录。

真 TCP/WS 使用生产 `bootstrap/build_router`、loopback 监听及 `tokio_tungstenite`，正式 `shahai-swd` + `wudian` 双包、两位玩家与旁观者在私人选择屏障落盘。真实目录不可写及 `PartialWrite/FileSync/Rename/DirectorySync` 四个实例故障逐一验证：无 Accepted、无未确认广播，完整私人投影/选择/动作、随机结果、原文件（替换前）、令牌及副本保持；rename 后不确定故障另测修复再失败仍拒绝。解除故障后同 idempotency key 重试成功，再销毁整个 runtime、停用/移除原包，以新 bootstrap 按冻结副本恢复并继续命令。另一用例验证两健康房与较新但未提交的完整 temp、截断 temp、截断 final/错 ID 记录共存，只恢复已提交 final，新局避让保留房号；checksum 损坏另由 store 定向测试覆盖。

Chrome 于 **15:40:39–15:40:44 +08:00** 经真实大厅/按钮/WS 推进到私人选择。把独立临时 rooms 目录移走、原路径换成普通文件，点击答案得到协议错误，未产生成功回覆/广播，原文件保持。随后对实际服务子进程执行 **SIGKILL**，恢复目录并加入部分/完整未提交 temp 和坏 `999.json`；新程序启动后三端按原角色/座位/令牌自动回房，完整私人状态保持，回答后 revision **3→4** 且原包副本再次落盘。新局房号 **1000**，坏档与 temp bytes 未变。SIGKILL 发生在故障响应后；写入阶段由受控故障注入与残留 fixture 覆盖，未把它当成命中某条写指令或真断电的证明。见[初始服务日志](browser-server-initial.log)、[重启服务日志](browser-server-restored.log)。

```bash
PATH=/Users/ruyimg/.cargo/bin:/usr/local/bin:$PATH cargo +stable build --offline -p game-server
NONAME_WEB_DIST=/absolute/path/to/validated/production/dist \
PLAYWRIGHT_MODULE=/absolute/path/to/playwright/index.mjs \
/usr/local/bin/node docs/operations/evidence/2026-10-24-native-room-durability/browser-crash-room-smoke.mjs
```

脚本默认 loopback 页面 **5483**、WS **5484**，可用 `NONAME_WEB_PORT` / `NONAME_WS_PORT` 覆盖。复制正式包、使用独立临时存储、关闭自建浏览器/服务；不访问用户真实房间数据或改原 dirty dist。故障控制仅在 Rust `cfg(test)` 实例存在，生产构建没有环境开关。

## 完整门禁与修改保护

完整 preflight 于 **15:36:41–15:43:02 +08:00** 首跑通过，**20 项全执行、0 失败、0 跳过、0 豁免**；脚本单测实际 **180 tests / OK**，行数门禁 **8571 文件 ≤2000 行**。运行未修改的 `preflight.main()` 完整模式，仅透明记录 `subprocess.run` 的原 stdout/stderr，没有改变命令、检查或返回值；现有 ResourceWarning 原样保留，不称零警告。补写本页与进度主稿后，再检查全量本地链接、严格文档行号、旧值、索引、行数及 `git diff --check`，见[最终文档门禁](final-doc-gates.log)。

开始备份 HEAD、index/worktree binary diff、服务端原始文件和 **46927 个既有路径**的内容/模式/符号链接属性；初始 dirty/untracked status **11307 条**，备份在 `/tmp/noname-next-room-durability-20261024/`。本次基准显式纳入三个生成目录与原 dirty dist，并有完整实体副本。最终仅 **6 个既有服务端/进度文件**增量变化，其余 **46921 路径**原样，新增四个模块/测试及本证据目录；意外变更 **0**。

受验 **1058 文件**源码/测试/配置/两正式包指纹见[来源清单](source-manifest.json)，最终摘要未变；本轮服务端增量见[差异](server-incremental.diff)，不把此前混合修改算作本轮新增。三个生成目录 **18 文件**与原 dirty dist **46 文件**和本轮实体备份逐内容/模式一致、无增删；本轮未重建或覆盖这些产物。完整结果见[修改保护](preservation.json)。**HEAD/index 不变，未暂存、未提交、未推送**。

## 边界与后续

- 没有验证真实断电、存储控制器缓存、网络文件系统、磁盘硬件损坏、Windows 实机/跨平台 CI、跨机/TLS/代理；Unix 同步失败明确报错，Windows 的目录建立/删除耐久保证较弱，不能作统一平台断电承诺。
- rename 后报错且马上中断，原 final 可能是旧确认版或完整未确认版；客户端不能把协议错误解读成“肯定没发生”。完整跨重启幂等账本、签名 WAL 和录像不在本轮。
- 保持单进程房间写入假设，没有新增多服务进程锁、持久 WAL 或后台队列。每次已接受修改同步保存，I/O/延迟未作前后测量，不声称性能收益。
- 无原副本且原安装版本已丢失的旧档仍无法恢复；坏 final 不自动拿 temp 猜测修复。完整表现、联机选将/明将、长程整局、37 来源 G0–G5 与上游等价仍按主稿待办，不由本轮本机样本代替。
