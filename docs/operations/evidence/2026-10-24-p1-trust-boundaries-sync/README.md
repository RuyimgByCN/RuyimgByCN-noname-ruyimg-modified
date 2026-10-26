# P1 信任边界与上游内容同步（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加本轮未提交工作树；规则 **v322**、引擎归档 **v105**。本轮完成进度主稿 P1 的 4 项，以及 UP37-I01 的第一阶段（`df2682e6..1a548c95` 字符/技能/画像/目录实际同步）。当前优先级与剩余项只在[进度主稿](../../../architecture/native-progress.md#delivery-2026-10-24)维护；规则语义与 UP37 差异只在[规则基准](../../../architecture/native-rules-baseline.md#upstream-2026-10-24)维护；37 来源状态只在[扩展台账](../../../migration/upstream-extension-reconstruction.md#upstream-2026-10-24)维护。

## 1. P1：WASM 坏存档受控拒绝

- **根因**：`hex_decode` 用 `s[i..i+2]` 字符串下标切片；奇数长度、非 ASCII 或多字节 UTF-8 会 panic，WASM 表现为 `unreachable` 并使同句柄后续 `save_hex` 失效。
- **修正**：`hex_decode_bounded` 先检查上限、ASCII、偶数长度，再按字节解析；`restore_hex_checked` 在局部完成解码/恢复，全部成功后才替换 `self.inner`。新增上限 `MAX_ARCHIVE_HEX_CHARS = 64 MiB` 字符。
- **回归**：`crates/engine-native-wasm/tests/restore_boundary_handle.rs`（4 项：奇数/非 ASCII/非法字符、截断归档、拒绝后合法恢复、`engine_from_archive` 不 panic），加 lib 单元 5 项。**真生成包探针**用本轮重建的 `native-pkg` 验证 `"0"/"中"/"zz"/"0g"/" "` 受控拒绝、原存档字节与视图/合法动作保持、合法归档仍可恢复；[输出](wasm-restore-probe.log)、[脚本](wasm-restore-probe.mjs)。
- **边界**：证明的是 WASM 边界与同句柄继续可用；未证明 UI 对错误文案的呈现或 64 MiB 以上的业务政策。

## 2. P1：慢 WS 连接不阻塞房间 actor

- **修正**：`Outbound::try_send` 为非阻塞有界发送；队列满标记 `watch` 慢信号并返回 `Full`。房间 actor 的命令推送、选将快照与名册广播全部改走 `try_send`；只对 `Full` 移出广播（`Closed` 仍由 session 正常断开，避免测试/真实关闭被误判为慢）。session 读循环 `select!` 观察慢信号后退出；socket writer 每条消息 **5s** 超时，失败/超时同样标记。writer 只持有 `slow_marker()`（watch sender），不再持有 `Outbound` 克隆，避免 session 退出后出站队列无法关闭的死锁。
- **回归**：
  - `game-server` 单元 2 项（满载标记、Closed 不标慢）；
  - `nc09_native_actor::slow_observer_queue_full_does_not_block_room_actor`：容量 1、不读旁观者，其他玩家命令 3s 内完成且慢连接被移出；
  - 真 TCP/WS `native_extensions_ws_slow_spectator_does_not_block_commands`：300 条真实命令后当前玩家仍能推进，停读旁观者被服务端关闭。该用例用 `cfg(test)` 专用 writer 延迟/队列覆盖把饱和时间变得可控（生产构建不含开关，也不读环境变量）。
- **边界**：真实 WS 用例只在本机 macOS/loopback 跑；未做跨机/代理/TLS、带宽上限或长期慢连接统计。5s 写超时是策略值，未做尾延迟测量。

## 3. P1：浏览器重连保留待确认命令与原 key

- **修正**：`NetworkEngineClient` 的 pending 记录命令 `kind + key`；断线只失败欢迎/入房/快照等连接级 waiter，**不再清 pending**。重新 `JoinOk` 时先同步重放同 key/同内容命令（WebSocket 保序），再 resolve 入房；服务端按 reconnect token + key 返回原 Accepted 或只执行一次。自动重连 5 次耗尽后以“提交结果不确定”拒绝 pending，不伪装成已撤销；`dispose()` 才最终清理。
- **回归**：`apps/web-client/scripts/network-client.test.mjs` 新增 2 项——ACK 丢失后重连只以原 key 重放一次并由回执结算；自动重连耗尽后报“结果不确定”。既有“重连前禁止新提交”等 9 项保持通过。前端套件见[日志](frontend-test.log)。
- **追加（2026-10-24）**：跨浏览器刷新持久化已实现。`NetworkEngineClient` 在发送成功后把 `{roomId,key,kind}` 写入 `sessionStorage`；新实例启动时载入，`JoinOk` 后用**原 key/原命令**重放，服务端回执结算后清除。新增 `刷新后的新客户端用原 key 重放未确认命令`、`取消待确认命令会清除刷新持久化`；前端 **75/0**、tsc 0。新增 10 分钟 TTL：刷新持久化带 `createdAt`，超时不再重放，避免隔天错误重放陈旧命令。另增 `cancelPendingCommands()`（拒绝等待者并清除持久化），并在 BattleView 联机状态区接线「取消待确认」按钮（仅联机可用，取消后明确提示服务端可能已提交）。旧档无收据语义仍按主稿待办。

## 4. P1：存档目录单写者

- **修正**：`StoreWriterLock` 在 Lobby 启动时取得目录排他锁并随 Lobby 存活。Unix 用 `flock(LOCK_EX|LOCK_NB)`；Windows 用 `share_mode(0)`（失败即报“已有写者”）；其他平台明确拒绝启动。锁文件 `.writer.lock` 不被 `scan` 当作房间记录。`bootstrap` 拆成可报告错误的 `try_bootstrap`，`main` 在第二写者/目录不可用时记录错误并以非零退出；测试仍用 `bootstrap` 便捷入口。
- **回归**：4 个锁单元测试（同目录第二写者拒绝、释放后可重取、不同目录独立、锁文件非房间记录、陈旧内容不阻塞）；`main` 级同进程双 bootstrap 拒绝与释放后重启；真**跨进程**测试：子进程持锁、父进程 `acquire` 失败、`SIGKILL` 子进程后父进程成功取锁（异常退出可安全重启）。
- **行为变化**：不可读/非目录的存档路径现在在启动阶段拒绝（此前能启动但扫描失败后继续运行）。原 `native_extensions_ws_refuses_room_creation_when_recovery_inventory_is_unsafe` 的 unreadable-directory 分支已改为断言启动拒绝；exhausted-id 分支保持运行期拒绝新房间。原始测试意图与改动原因保留在本页与提交历史。
- **边界**：仍是单进程单房 actor + 目录单写者；没有分布式 lease、跨网络文件系统保证、Windows 实机运行（两项 Windows 专属分支本机未编译执行）。

## 5. UP37-I01：内容同步到 `1a548c95`

范围固定为 `df2682e66af9930fe02861a25cb6f2cd2ef6ae36..1a548c95acdf8ac1ef44d74473403eda2ac86cf0`（37 提交、167 净变化路径）。本轮执行 `node scripts/sync-original-content.mjs` 完成字符树、画像、catalog 与能力矩阵同步；**未**修改任何 `content/original/ported/**` 人工 DSL，未改 standard nPack/扩展包，未提升规则或存档版本。

| 项 | 结果 |
|---|---|
| `legacy-source` | 366 文件、14.9 MiB；**49 个既有文件变化，0 新增、0 删除**；与 37 提交的字符面变更一致 |
| 画像 | 2707 文件；**+4**（周瑜/星董允/唐棠/许劭相关） |
| 目录角色 | 2576 → **2582**（+6：界朱然、界朱桓、唐棠、孙寒华、许劭、TW魔张梁） |
| 目录技能 | 6190 → **6209**（+19） |
| 目录 `def_id` | **既有条目 0 变化、0 删除**；新条目按稳定哈希分配（[差异](catalog-diff.json)） |
| 两个既有角色启用 | `collab:ol_chitu`/`collab:ol_le_menghuo` 上游新增技能列表；本地 ported nPack 仍为空技能，故状态由 `ported` 如实降为 `source_synced`，`def_id` 不变。对应 N05/N06 尚未实现，不把启用写成已可玩 |
| 人工 DSL | `ported/**` 16617 文件哈希逐一不变（[前后](ported-before.sha)/[后](ported-after.sha)，[diff 退出码](ported-diff.exit)） |
| 派生牌 | 上游新增 `kuratamashiro`、`mashiro`、`ol_cailian` 三张定义已在 `legacy-source` 对应 `card.js`；尚未进入原生目录/包 |
| 同步校验 | `--verify-only` **exit 0**（[输出](sync-verify.exit)/[日志](sync-verify.log)） |
| 内容身份 | catalog 既有 `def_id` 全保留；旧局内容锁不引用 catalog 文本，未改包字节。仍不把这次同步称为“规则等价” |

**没有做**：R01–R10 规则移植、N01–N07 可玩内容、模式 Step→Async 的等价对照、AI/资源表现验收。catalog 中的新技能目前是 `source_synced`（不是 `ported`），不得用目录存在代替实现或整包验收。

## 6. 验证汇总

| 检查 | 结果 | 原始记录 |
|---|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | **1547 passed / 0 failed / 71 ignored**，315 个 target，exit 0 | [日志](engine-native-full.log)、[计数](engine-native-counts.json)、[退出码](engine-native-full.exit) |
| `cargo test --offline -p game-server --no-fail-fast` | **328 passed / 0 failed / 0 ignored**，23 个 target，exit 0 | [日志](game-server-full.log)、[计数](game-server-counts.json)、[退出码](game-server-full.exit) |
| 前端 `pnpm test` | 通过（见日志末尾 `fail 0`），exit 0 | [日志](frontend-test.log)、[退出码](frontend-test.exit) |
| 前端 `pnpm typecheck` | exit 0 | [日志](frontend-typecheck.log)、[退出码](frontend-typecheck.exit) |
| 重建原生 WASM + 既有 Worker 冒烟 | `{"total":44,"failed":0}`，exit 0 | [日志](native-worker-smoke.log)、[退出码](native-worker-smoke.exit) |
| 真生成包坏存档探针 | 5 类坏输入全部拒绝、原存档/视图/动作保持、合法恢复 ok | [日志](wasm-restore-probe.log) |
| 源同步 `--verify-only` | exit 0 | [日志](sync-verify.log) |
| 人工 DSL 不变 | 16617 文件 diff 为空，exit 0 | [diff 退出码](ported-diff.exit) |

测试计数按 Cargo target 汇总；既有 `#[path]` 重复测试入口与 Windows 专属测试未计为新增场景。写入文档与 R01 后完整 preflight **20/20 通过、0 失败/跳过**（19:18–19:21 与 19:45 后两次，脚本单测 **180/180**、行数门禁 **8578 文件 ≤2000 行**），最终见[preflight](preflight-final-02.log)/[退出码](preflight-final-02.exit)（此前两次分别见 [19:21](preflight-final.log) 与 [19:45 后](preflight-after-r01-docs.log)）；R01 后再跑 game-server **328/0/0**（[日志](game-server-after-r01.log)/[计数](game-server-after-r01-counts.json)），engine-core **3780/0/1**，见[R01 证据](../2026-10-24-up37-r01-dichou/README.md)。

## 7. 失败记录（保留原始原因，不用单项补跑冒充全绿）

1. 首次完整 `game-server` 回归有 2 个 target 失败：`nc09_failed_lazy_reclaim_keeps_original_seat_and_token`（把出站 `Closed` 也当作慢连接移出，导致测试座位被当作离线回收）与 `native_extensions_ws_refuses_room_creation_when_recovery_inventory_is_unsafe`（不可读目录现在按设计在启动阶段拒绝）。前者修为只对 `Full` 降级；后者更新测试期望并保留“exhausted-id 仍启动但拒绝新房间”分支。此后完整目标重跑通过，首次失败不删除。
2. 真 WS 慢连接用例最初 300 条命令未使 OS 缓冲/队列饱和，只证明不阻塞、未证明断开；加入 `cfg(test)` writer 延迟与小队列后复现并验证关闭。第一次 10s 等待超时的输出保留在本地 shell 历史；修复后的日志为本页最终记录。
3. 真 WS 用例第一次在断开后 panic：读端把连接重置当错误 `unwrap`。改为 `Err` 也视为已关闭后通过。
4. 定位慢连接死锁时发现 writer 持有 `Outbound` 克隆，session 退出后 `rx.recv()` 永远不返回；改为只持 `slow_marker()`。这是本轮的真实实现缺陷，已修复。
5. 首版 WASM 集成测试把 seat 0 与 seat 1 的视图混用而失败（测试夹具错误，不是产品失败）；修正 seat 后通过，未改产品断言。

## 8. 保护与边界

- 开始前备份 HEAD/status 与完整工作树到 `.worktree-backups/noname-next-2026-10-24-round/`（工作区外；HEAD `7e28a937…`）。本轮新增/修改仅服务端 4 模块、1 个新锁模块、2 个测试文件、WASM 代码/测试、前端客户端/测试、内容同步文件与本证据目录；**未删除既有文件**（`shahai` 拆分为本轮前已有工作）。HEAD 与 index 未变，未暂存、未提交、未推送。详见[保护记录](preservation.json)与[源码摘要](source-manifest.json)。
- 未验证：真断电/存储控制器缓存/网络文件系统、Windows 实机、跨机/TLS/代理、64 MiB 以上输入政策、R01–R10/N01–N07 规则与内容、37 来源整包八项、平台表现/性能。
- 环境：macOS、rustc/cargo 1.95.0、Node 20.17.0（Worker/探针）、pnpm 10.34.3、Python 3.10.10；WASM 由本轮 `build-native.mjs` 重建。见[environment.json](environment.json)。
