# 原生产品真实权威事件与 state hash（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；规则 **v322**、存档 **v105**。本批实现进度主稿 P2 第 5 项的一半：Worker 信封与原生 WS 不再返回空 `events`／`hash_hex`，改用同一内核的权威 hash 与公开事件码；**完整 VM 包与第三方免重编译安装仍未完成**。当前优先级见[进度主稿](../../../architecture/native-progress.md#delivery-2026-10-24)。

## 实现

- `crates/engine-native/src/view.rs`：新增 `public_event_code(&LogEvent) -> Option<u16>`，只按事件种类投影为既有协议码（2 摸牌、3 用牌/响应、4 伤害/防伤、5 回复、6 濒死、7 死亡、8 阶段/回合、10 选择），**不携带牌 id、选项内容或暗牌**。
- `Engine::public_event_codes_from(start)`：从日志游标投影公开码。
- `engine-native-wasm` 新增宿主/Worker 导出：`state_hash_hex()`（32 字节 hex）、`log_len()`、`public_event_codes_json(start)`；坏恢复不清空原日志，合法恢复后日志从空开始。
- `apps/web-client/src/worker/nativeBackend.ts`：`NativeHandle.eventCursor` 记录已消费日志长度；命令信封读取增量事件码并映射 `PublicEvent`，`hash_hex` 直接取内核 hash。observe/save/status 不消费事件，避免旁观操作吞掉命令事件。旧测试桩缺少新导出时保留空占位（真实 WASM 由 smoke 断言非空），生产路径不伪造。
- `apps/game-server/src/room_native.rs`：`NativeRoomSession.event_cursor` 随 checkpoint/rollback 恢复；`take_public_event_codes()` 供 actor 在命令回覆与广播中使用。恢复引擎时游标重置到新日志长度，避免重放。
- `room_native_actor.rs`：`CommandOutcome.public_event_codes` 由空数组改为真实增量；同一次命令的回覆与所有观察者广播共享同一组码。

## 验证

| 检查 | 结果 | 证据 |
|---|---|---|
| `engine-native` 完整 | **1547 / 0 / 71**（315 target） | [日志](engine-native-full.log)、[计数](engine-native-counts.json)、[退出码](engine-native-full.exit) |
| `engine-native-wasm` 完整 | **48 / 0 / 0**（8 target，含新增 3 项） | [日志](engine-native-wasm-full.log)、[计数](engine-native-wasm-counts.json)、[退出码](engine-native-wasm-full.exit) |
| `game-server` 完整 | **329 / 0 / 0**（23 target，含新增 1 项） | [最终日志](game-server-full-02.log)、[计数](game-server-counts.json)、[退出码](game-server-full-02.exit) |
| 前端 `pnpm test` | **72 passed / 0 failed** | [日志](frontend-test.log)、[退出码](frontend-test.exit) |
| 前端 `pnpm typecheck` | exit 0 | [日志](frontend-typecheck.log)、[退出码](frontend-typecheck.exit) |
| 重建原生 WASM + Worker 冒烟 | **47 / 47**，新增 hash/事件/旁观不消费 3 项 | [日志](native-worker-smoke.log)、[退出码](native-worker-smoke.exit) |

WASM 新测试 `tests/events_hash_export.rs` 覆盖：hash 为 64 hex 且随阶段推进变化；事件码非空且只在允许集合；恢复后日志为空、hash 保持；坏恢复不清空/不重放原日志。server 测试 `native_command_outcome_carries_public_event_codes` 覆盖真实 actor 回覆。Worker smoke 实测 `codes=[8,8,10]`、hash `12770595…55c1`，旁观信封事件为空且 hash 相同。

## 完整门禁

`python3 -u -B scripts/preflight.py` **20/20 通过、0 失败/跳过**（20:08–20:13 +08:00），脚本单测 180/180、行数门禁 8579 文件；见[preflight](preflight.log)/[退出码](preflight.exit)。`sync-original-content --verify-only` 仍 **exit 0**（[日志](sync-verify.log)/[退出码](sync-verify.exit)）。

## 失败记录（保留并修复）

1. **前端首跑 12 项失败**：既有 test double 的 `NativeEngine` 没有 `state_hash_hex/log_len/public_event_codes_json`，`nativeEnvelope` 直接调用导致 `TypeError`。修正为仅真实引擎读取权威面、测试桩退回空占位；真实 WASM smoke 断言三项非空。首跑日志随后被通过日志覆盖，原始错误文本保留在此：`handle.engine.state_hash_hex is not a function`（14 处适配层用例同因）。
2. **game-server 首跑 64/1 失败**：`commit_crash` 子进程把父进程“先创建空 `arm` 再写入”误读为单故障，exit 86 而测试期望 87。修正为轮询到非空内容后再选故障；[失败原文](game-server-full.log)保留，重跑[通过](game-server-full-02.log)。
3. Worker smoke 新增检查最初依赖 `authEnv` 的已有事件；改为在独立恢复句柄上提交一条 `endPhase` 后再断言，避免“循环未推进导致空事件”的假失败。

## 边界与未完成

- 事件只是**粗粒度码**，不含来源/目标/牌明细；与上游或旧线逐事件 payload 等价未证明。需要明细的 UI/回放仍读座位视图或后续事件契约。
- 同 key 幂等重放故意不重发历史事件；跨进程恢复后新日志从空开始，不承诺补齐重连前事件。
- 未做 native／WASM 同一命令的逐事件码与 hash 对照（P2 第 8 项），也未做长局事件内存增长测量。
- **完整 VM 包经正式安装进入 Worker／服务端、第三方技能免重编译、内容私有状态编码/hash 收敛仍未完成**；本批只关闭“空 events/hash 占位”。
- 未改规则/存档版本、golden 或包内容锁。
