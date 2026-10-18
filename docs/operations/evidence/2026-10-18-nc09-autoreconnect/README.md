# NC09 客户端自动重连（页内，无需刷新）（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）「重连」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**页内自动重连（指数退避）已实现并在真实浏览器验证**；服务端重启后房间不持久 → 客户端**准确报错**（登记）

## 1 实现

| 面 | 变更 |
|---|---|
| `src/engine/networkClient.ts` | `handleClose` 后调度 `scheduleReconnect()`：指数退避（1s→8s，最多 5 次），成功/失败通过新增 `onReconnect("reconnecting"｜"reconnected"｜"failed")` 广播；`dispose()` 清理计时器与计数 |
| `src/views/BattleView.vue` | 订阅 `onReconnect`：`正在重连…` → 重连成功后**带令牌自动回房**（`joinRoom(roomId, "", localStorage[reconnectKey])`）→ 成功写 `房间 #N · 已重连`，失败写 `重连后回房失败：<原因>` 并记日志 |

## 2 实测（脚本自身启停服务端，**不刷新页面**）

`apps/web-client/scripts/nc09-browser-disconnect.mjs` 扩展为页内重连验收（[autoreconnect-drive.log](autoreconnect-drive.log)、
[disconnect-report.json](disconnect-report.json)、[disconnect-screenshot.png](disconnect-screenshot.png)）：

| 阶段 | 结果 |
|---|---|
| 入座 | `房间 #1` |
| 服务端被 SIGKILL | **`连接已断开（服务端不可达）`**（可见） |
| 重启服务端（**不刷新**） | 客户端**自动重连** → 回房尝试 → **`重连后回房失败：room not found`** |
| 控制台 | `console_errors: []` |

**判读**：① WebSocket **自动重连成功**（无需用户刷新）；② 房间在服务端重启后**不持久** →
回房失败，客户端**如实报错**（不是静默保持旧状态）；③ 全程无未处理拒绝。

## 3 未完成 / 待办（NC09）

- [ ] **房间/对局跨服务端重启持久化**（本轮剩余缺口）：需把 `NativeRoomPersist` 的快照/WAL
      落到磁盘并在服务端启动时恢复房间（含名册与重连令牌），否则重连只能失败
- [ ] 重连**成功后**的状态一致性断言（当前依赖服务端仍持有房间）
- [ ] 限流（`max_cmds_per_sec`）与慢消费者的用户可见反馈
- [ ] 联机存读档/录像、锦标赛面、平台与性能档位、服务端 SPA fallback
