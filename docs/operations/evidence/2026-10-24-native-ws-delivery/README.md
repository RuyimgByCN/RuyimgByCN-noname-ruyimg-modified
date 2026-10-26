# 原生正式扩展房间的真实 WS 验证（2026-10-24）

基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加本轮未提交工作树；规则 **v322**、原生存档 **v105**。当前结论见[进度主稿](../../../architecture/native-progress.md#native-ws-2026-10-24)。本轮使用真正的 loopback TCP/WebSocket、正式安装包和生产浏览器页面，没有以直调 `RoomMsg` 代替端到端证据。

## 接通的路径

- `CreateRoom` 沿用现有协议，由服务端配置启用原生后端和已安装包。正式 `upstream-noname/shahai-swd`、`upstream-noname/wudian` 的实体牌进入目录、牌堆与手牌。服务端装载还补齐现有 character-only 和 pile-only 资产。
- 房间保留原局包集合和密码；包引用绑定 namespace、完整版本、logic/content 指纹，读取安装资产时重算 bytes hash。恢复不再按短包名或 `ends_with` 猜测。缺包、同版本内容变化、安装副本被替换时拒绝恢复并保留存档。
- 统一协议选择 token 为候选下标，提交时还原真实 Card/Player/Tag/Confirm 等选项；解读私人选项前检查选择者身份。JoinOk、快照和广播都带个人 choice/合法动作；修复视图座位偏移、旧 revision 检查和玩家间幂等键串用。
- 新入座玩家使用 UUID 重连令牌，恢复延续原令牌；旧连接失权、重启后连接号避让与 roster 恢复；在线席位不会被缺席超时回收。选将虽然不递增 revision，状态变化仍保存并更新内存恢复点。
- 前端保留密码、角色和最新令牌回房；旁观者继续旁观。页面刷新回原房间，回房成功前禁止提交；命令被拒后也应用最新投影，旧信封延迟不会盖掉断线提示。

## 最终验证

| 验证 | 命令 | 结果 / 原始输出 |
|---|---|---|
| 六 crate 回归 | `cargo +stable test --offline -p game-server -p engine-native -p engine-native-wasm -p content-native-loader -p engine-protocol -p rule-api --no-fail-fast` | **1741 passed / 0 failed / 71 ignored，exit 0**；[最终完整日志](cargo-regressions-final.log)、[命令起讫](cargo-regressions-final.json)、[按 crate 汇总](cargo-summary-final.json) |
| 真 WS 原生房间 | 上述 `game-server` binary tests 自动执行 `native_ws_e2e` | 正式双包开局、双玩家/旁观隔离、密码/冒充/过期 revision、保存待选择状态、新 runtime 恢复与续局；缺包/同版本内容改变拒绝；**2/2 passed**；[最终定向日志](ws-e2e-targeted-final.log) |
| 前端完整回归 | web-client 内 Node 20 执行 `pnpm test` | **70 passed / 0 failed / 0 skipped**；[原始日志](frontend-test.log)，其中新增网络/提示时序回归 9 项 |
| 型别与生产构建 | `pnpm --dir apps/web-client build` | **exit 0**；[日志](frontend-build.log) |
| 原生 WASM / Worker | web-client 内 `/usr/local/bin/node --import ./scripts/ts-register.mjs scripts/native-worker-smoke.mjs` | **44 checks / 0 failed**；[日志](native-worker-smoke.log) |
| Chrome 生产页面 | 运行[浏览器脚本](browser-ws-smoke.mjs) | **7 checks / 0 failed，pageerror 0**；[最终输出](browser-ws-current-binary.log)、[命令与 dist 路径](browser-ws-current-binary.json)、[报告](browser-report.json)、[恢复后截图](browser-ws-restored.png) |
| 完整 preflight | `python3 -u -B scripts/preflight.py` | **20/20 通过，0 失败 / 0 跳过 / 0 豁免，exit 0**；[最终摘要](preflight-full-final.log)、[全部子命令原始输出](preflight-full-final-raw.log)、[命令起讫与每项结果](preflight-full-final.json) |
| 行数门禁及单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **单测 1/1；8565 个受维护文件均 ≤2000 行**；[单测](source-lines-unit.log)、[门禁](source-lines.log) |

六 crate 最终统计：`engine-native` **1547/0/71**、`game-server` **123/0/0**、`engine-native-wasm` **36/0/0**、`content-native-loader` **32/0/0**、`engine-protocol` **1/0/0**、`rule-api` **2/0/0**（passed/failed/ignored；包含现有 `#[path]` 重复执行，不作唯一场景数）。最终命令于 **14:23:38–14:25:13 +08:00** 完整重跑通过。

首次同一六 crate 命令为 **1739 passed / 2 failed / 71 ignored，exit 101**，保留[首跑日志](cargo-regressions.log)及[统计](cargo-summary.json)。两项均为本轮新增测试的 fixture 问题：pile-only 用短名撞到两个 namespace，改用明确 namespace 并保留二义性拒绝断言；choice fixture 先在裸 Engine 使用 dedup 后再新建 session，实际拒绝为 `DuplicateIdempotency`（[诊断](backend-choice-diagnostic.log)），改由 fresh session 经正式 submit 推进，未削弱生产校验。两套定向 **24/24** 通过后（[日志](backend-fixture-fixes.log)），才重跑上述完整命令。

工具链使用 rustup Cargo/rustc **1.95.0**，Node **20.17.0**、pnpm **10.34.3**；Cargo 离线执行。完整环境见[环境记录](environment.json)。preflight 的 Python 是默认 **3.10.10**；加前缀 PATH 的构建环境另有 Python 3.12.5。Cargo 日志保留既有警告，不声称 workspace 零警告或通过所有 `-D warnings` 门禁。

[首次完整 preflight](preflight-full.log)（[命令起讫](preflight-full.json)）于 **14:09:28–14:21:04 +08:00** 为 **19/20，exit 1**，仅脚本单测报 `FAILED (errors=1)`；因现有 preflight 只输出子命令末行，该次没有 traceback，**无法确认原因**。随后用同一默认 Python 3.10 不改源码重跑脚本 discovery，**180 tests / OK / exit 0**（[完整诊断输出](scripts-unit-diagnostic.log)）；保留首次失败，不据此覆盖为首次全绿。

最终完整 preflight 于 **14:30:34–14:36:55 +08:00** 执行原脚本未修改的 `preflight.main()`（完整模式），只在内存透明记录 `subprocess.run` 的原 stdout/stderr；20 项均执行，没有改变 argv/结果或添加豁免。脚本单测实际 **180 tests / OK**，保留完整原文；其中现有 ResourceWarning 不算作测试失败，不声称零警告。完成后补写汇总，再对最终文档执行本地链接、文档行号、旧值、索引、行数与 `git diff --check`，见[最终文档检查](final-doc-gates.log)。

## 真实网络覆盖与复现

Rust 测试启动真实 `TcpListener(127.0.0.1:0)`、生产 `bootstrap/build_router` 和 `tokio_tungstenite` 客户端。正式包加载后的 reference engine 与 WS 初始 canonical hash、本人手牌逐项一致，并以无扩展引擎作反例；验证的是实际实体牌而非只发现包元数据。三次阶段推进进入必须弃牌的选择屏障，只有选择者看到候选和合法答案；Snapshot 与 CommandOutcome 的完整状态相同。保存后销毁原 Tokio runtime，再以新 runtime/bootstrap 从磁盘重建房间，令牌重入保留完整 choice、动作、投影和摘要，提交真实 ResolveChoice 后继续同步且磁盘快照更新。

浏览器使用本次 Vite 生产 dist、三个独立浏览器 context 和真正的 `target/debug/game-server` 子进程。经大厅按钮创建受密码保护的房间、另一玩家加入及旁观，推进到私人选择屏障，再执行页面 reload 与服务端 **SIGTERM/重新启动**；三端自动按原角色回房，玩家座位/令牌及完整投影、choice、动作保持，点击渲染出来的答案后修订 **3→4**、三端摘要一致。报告的 7 项是断言组，不是 7 个来源验收。首次浏览器 6 组通过后增强到待选择恢复覆盖，首次输出和截图保留在[首次报告](initial-browser/browser-report.json)。最终 **14:28:19–14:28:23 +08:00** 再用当前 Cargo 构建的服务程序与已验证 dist 外部副本完整重跑 **7/7**（[前一次待选择报告](first-pending-choice-browser/browser-report.json)也保留）；临时根仅链接仓库 target/packages 和外部 dist，不覆盖已恢复的原 dirty dist。以上浏览器运行都通过，不是失败重试。

```bash
PATH=/Users/ruyimg/.cargo/bin:/usr/local/bin:$PATH cargo +stable build --offline -p game-server
PATH=/Users/ruyimg/.cargo/bin:/usr/local/bin:$PATH pnpm --dir apps/web-client build
PLAYWRIGHT_MODULE=/absolute/path/to/playwright/index.mjs /usr/local/bin/node docs/operations/evidence/2026-10-24-native-ws-delivery/browser-ws-smoke.mjs
```

浏览器脚本使用已安装的 Playwright（运行期路径见环境记录），没有新增仓库依赖。默认只监听 `127.0.0.1:5483`（页面）和 `:5484`（WS），可用 `NONAME_WEB_PORT` / `NONAME_WS_PORT` 覆盖；存储使用独立临时目录。脚本自己关闭浏览器、静态服务与 game-server，不连接用户原房间或修改真实保存目录。

手动运行原生 WS 服务时使用已有配置：

```bash
NONAME_NATIVE_ROOMS=1 \
NONAME_NATIVE_ROOM_NAMESPACE=upstream-noname \
NONAME_NATIVE_ROOM_PACKAGES=shahai-swd,wudian \
NONAME_NATIVE_ROOM_STORE=/absolute/path/to/room-saves \
cargo +stable run --offline -p game-server
```

`native_rooms` 继续显式启用；未启用时保留旧线后端，原生包装载失败不会静默回退。原生建局即开局，赛前大厅及联机选将 UI 仍按已有待办登记。

另一次证据整理期间的 preflight 重跑因本页先引用尚未落盘的 `final-doc-gates.log` 而在全量链接检查失败，随后主动停止，**12 项通过 / 1 项失败 / 1 项中断 / 6 项未执行**，不是完整检查结果；[中止日志](preflight-full-rerun.log)、[状态明细](preflight-full-rerun.json)、[已执行子命令原始输出](preflight-full-rerun-raw.log)保留。补齐真实文档检查输出后再开始最终完整重跑，未添加检查豁免。

## 修改保护

开始备份 HEAD、index diff、worktree binary diff，并记录 **46796** 个既有 tracked/untracked 文件的内容与模式摘要，其中初始 dirty/untracked status 为 **11196** 路径；备份位于 `/tmp/noname-next-ws-delivery-20261024/`。原三个生成包目录和 dirty dist 另有完整副本。

本轮没有改 WASM/Rust 内核、规则或归档版本；两条 WASM 的三个生成包目录共 **18 个文件**与开始时逐一 SHA-256/模式一致。Worker 44 项使用该已验证的原生 WASM，本轮没有再次重建 WASM。新生产 dist **45 文件**在浏览器通过后存到临时目录的 `validated-generated/`，随后恢复原 dirty dist **46 文件**，见[产物保护](artifact-restoration.json)。

BattleView 原有本地扩展开局/保存/恢复函数片段与开始时逐字相同；通过备份 patch 在临时目录重建其原文件并校验 SHA-256，再输出本次增量 diff，见[本地路径保护](battle-preservation.json)、[增量 diff](battle-incremental.diff)。最终仅允许本轮列出的服务端、网络前端、测试与进度主稿增量变更，其余基准路径保持；核对 **46796** 个基准路径，其中 **46783** 完全一致、**13** 个为本轮明确增量，意外变更 **0**；完整清单见[修改保护核对](preservation.json)、[1045 个受验源码/配置/包数据摘要](source-manifest.json)。HEAD/index 不变，未暂存、未提交、未推送。

## 边界与后续

- 这是本机真实 TCP/WS 和 Chrome 的原生房间切片；未验证跨机/TLS/代理、Firefox/Safari、移动真机、Tauri 安装包或远端 GitHub Actions。现有 CI 的 `cargo test --workspace` 会包含新增 binary tests，本轮没有运行 Actions。
- **服务端内容恢复边界**：`load_reference` 依次经 `load_scoped` 与当前激活内容锁比对；原包停用或升级后，即使旧安装副本仍在也拒绝恢复。本轮验证原身份仍启用的重启恢复及不匹配拒绝，与本地 Worker 冻结载荷、卸载后仍可恢复的能力分别记录。
- **旧存档兼容边界**：此前没有完整安装身份的原生扩展包引用明确拒绝恢复并保留文件，需要原版本程序处理或明确迁移；不以猜包方式继续运行。旧客户端未按服务端绑定的 room-only 本地令牌键也不再读取。
- 尚未验收断电/fsync、磁盘写入故障、限流/慢消费者/拥塞、跨重启协议幂等账本、原生签名 WAL、仅 debug/tests 可达的 `RoomMsg::Recover` WAL replay 后 dedup 续行、录像导出/跳转、完整整局/长程和联机选将/明将界面。磁盘写失败仍按现有行为记录警告，不能据本轮健康磁盘恢复证明故障时不会丢失进度。
- 本地原生 Worker 的 `events`/`hash_hex` 表现占位及现有自定义牌名/资源表现映射、全部特殊模式、37 来源 G0–G5、规则完全等价和性能收益不在本轮验收结论中。后续依次继续 NC06/NC07 状态迁移与同源性、逐来源和跨平台验收。
