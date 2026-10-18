# NC09 房间跨服务端重启：真实浏览器端到端（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**服务端被杀后重启，房间从磁盘恢复；客户端页内自动重连后回到原房间**（全链路打通）

## 1 本轮接线（承接会话级导出/恢复与落盘存储）

| 面 | 变更 |
|---|---|
| `config.rs` | 新增 `native_room_store`（默认 `data/native-rooms`，env `NONAME_NATIVE_ROOM_STORE`） |
| `room_native_actor.rs` | `spawn_native_room_if_configured` 增加 `store` 参数并以 `spawn_room_native_persisted` 启动 |
| `lobby.rs` | 新增 `native_room_store` 字段（空串 = 不落盘）；新建原生房间带存储；**新增 `restore_native_rooms()`**：`load_all` → 逐房 `restore_room_from_disk`（按 `pack_refs` 重新装载包）→ 以**原房间号**入表 → `next_room` 推进到最大号之后；单房失败**只告警不阻断启动** |
| `main.rs` | 启动时调用 `lobby.restore_native_rooms()`（0 恢复为空操作，失败仅告警） |

## 2 端到端实测（真实浏览器 + 真实服务端 + 磁盘持久化）

脚本：`apps/web-client/scripts/nc09-browser-disconnect.mjs`（**自身启停 game-server**，并把
`NONAME_NATIVE_ROOM_STORE` 指到临时目录；**全程不刷新页面**）。

| 阶段 | 结果 |
|---|---|
| 入座 | `房间 #1`；磁盘出现 `1.json` |
| 服务端 SIGKILL | **`连接已断开（服务端不可达）`** |
| 重启服务端（不刷新） | 客户端自动重连 → **`房间 #1`**（`hasRoom: true`）——房间**从磁盘恢复** |
| 控制台 | `console_errors: []` |

证据：[restart-e2e.log](restart-e2e.log)、[disconnect-report.json](disconnect-report.json)、
[disconnect-screenshot.png](disconnect-screenshot.png)。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| game-server 全量 | **92 passed / 0 failed** | [game-server-tests.log](game-server-tests.log) |
| 端到端（浏览器） | 见 §2 | [restart-e2e.log](restart-e2e.log) |
| 行数门禁 | 8274 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC09）

- [ ] 恢复后**名册广播**与在座客户端连接重建（当前客户端靠自动重连 + 令牌回座；服务端恢复后
      未主动向已连接者重播名册）
- [ ] 房间终局/关闭时删除落盘记录（当前只增不减）
- [ ] 落盘**节流**与 WAL 增量（当前每条接受命令全量写）、校验和
- [ ] 密码/令牌的落盘保护（当前明文，仅本地磁盘；生产需加密或外部存储）
- [ ] 限流与慢消费者反馈、联机存读档/录像、锦标赛面、平台与性能档位、服务端 SPA fallback

## 5 追加：座位恢复断言 + 终局清理（2026-10-18 同轮）

### 5.1 座位恢复（E2E 复跑，[restart-e2e-2.log](restart-e2e-2.log)）

重启后除房间号外**同时断言座位**：

```
STEP after_restart {"hasRoom":true,"roomLine":"房间 #1","statusLine":"联机房间 #1",
                    "seatLine":"► 第 1 轮 · P1"}
```

→ 客户端不仅回到房间，还回到**原座位 P1**（令牌重连命中原座位）；`console_errors: []`。

### 5.2 终局清理（落盘只增不减 → 已修）

`persist_room` 增加终局分支：`session.engine().outcome().is_some()` 时 **`store.remove(id)`** 并
停止写入（已结束房间无需跨重启恢复）。回归 `nc09_room_store_is_removed_on_match_end`：
写盘 → `remove` → 文件不存在、`load_all` 为空、重复 `remove` 幂等。

### 5.3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向（含 persist 既有 2 项） | **12 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **93 passed / 0 failed**（92 → +1） | [game-server-tests-2.log](game-server-tests-2.log) |
| E2E（浏览器，重启恢复房间+座位） | 见 §5.1 | [restart-e2e-2.log](restart-e2e-2.log) |

### 仍未完成（更新）

- [ ] 落盘**节流**与 WAL 增量/校验和（当前每条接受命令全量写）
- [ ] 密码/令牌落盘保护（当前明文，仅本地磁盘）
- [ ] 恢复后**主动**向已连接者重播名册（当前靠客户端重连 + 令牌回座）
- [ ] 限流与慢消费者反馈、联机存读档/录像、锦标赛面、平台与性能档位、服务端 SPA fallback
