# NC12 联机往返时延（RTT，本机回环）（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC12**（[方案 §NC12](../../../architecture/native-convergence-plan.md#nc12)）「联机 RTT」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**已取得「成功命令」的往返时延**（n=30 全部 `Accepted`）；**本机回环、无真实网络**，不等于生产网络指标

## 1 方法（可复跑）

```bash
NONAME_ALLOW_UNSIGNED_EXTENSIONS=1 NONAME_NATIVE_ROOMS=1 NONAME_NATIVE_ROOM_PACKAGES=shahai-swd \
  GAME_SERVER_ADDR=127.0.0.1:7700 ./target/release/game-server
node apps/web-client/scripts/nc12-online-rtt.mjs 40
```

`apps/web-client/scripts/nc12-online-rtt.mjs`：Node 原生 WebSocket 直连——`Hello`（`network_protocol=5`）
→ `CreateRoom(2 人)` → 第二个连接 `JoinRoom` → 循环 `SubmitCommand(EndPhase)`，测「发出 → 收到
`CommandOutcome`」的墙钟时间；**只统计被接受（`Accepted`）的命令**（被拒命令走快速失败路径，不在
本指标口径内）；每 4 条命令换一个新房间以避免进入需要 `ResolveChoice` 的阶段。

## 2 实测（[rtt.log](rtt.log)；采样负载 [load.txt](load.txt)：20.01 / 17.38 / 15.51）

| 指标 | 实测 |
|---|---:|
| `n`（被接受样本） | **30** |
| **p50** | **546,542 ns ≈ 547 µs** |
| p95 | 825,000 ns ≈ 825 µs |
| max | 1,005,000 ns ≈ 1.01 ms |
| mean | 558,528 ns ≈ 559 µs |
| 被拒样本（未计入时延） | 10（对局进入选择阶段后 `InvalidChoice`，本探针不模拟选择应答） |

**口径与边界（不得外推）**：
- **本机回环**（127.0.0.1），无真实网络抖动/丢包/带宽限制 → **不是生产网络 RTT**；
- 采样时 `load average ≈ 20`（本机并行负载高）→ 只有 **p50** 可比较；
- 不含客户端渲染、动画与投影处理；不含 TLS/代理。

## 3 排障过程（本轮实际收益）

探针首版**全部被拒**（`Accepted=0`），逐层定位到 3 个真实问题（都已修复并记录）：

| # | 现象 | 根因 | 处置 |
|---|---|---|---|
| 1 | `Error: room error`（无细节） | `lobby.rs::map_room_err` 把未知错误统一成 `"room error"` | **改为 `room error: {原文}`**（联机排障可见原因；见下 §4） |
| 2 | `BadMessage: json: invalid type: map, expected unit` | `CommandKind::EndPhase {}` 是**单元变体**，serde 外部标签应为字符串 | 探针改发 `"EndPhase"` |
| 3 | 命令打到**旧房间**被拒 | 换房后未更新 `roomId` | 换房时更新 `roomId` |

**对照事实**：真实浏览器路径（[联机闭环证据](../2026-10-18-nc09-browser-online/README.md)）此前已
验证同样的「结束阶段」命令被接受（`rev 0→1`）——本轮探针问题在**手工报文**，非服务端缺陷。

## 4 顺带改进（产品可诊断性）

`apps/game-server/src/lobby.rs`：`map_room_err` 的兜底分支由固定文案 `"room error"` 改为
`"room error: <原文>"` —— 此前联机排障只能看到「room error」，看不到具体原因（本轮即由该改进
一次定位到 `StartGame 座位数不一致`）。

## 5 未完成 / 待办（NC12 联机面）

- [ ] **真实网络**（跨机/公网）RTT、丢包与重连抖动、弱网下的命令重试
- [ ] 慢消费者背压下的时延与丢弃率；`max_cmds_per_sec` 限流下的表现
- [ ] 客户端侧「提交到界面反映」的端到端时延（含投影与渲染）
- [ ] ≥30 分钟持续负载与温控；其余平台档位；旧线对照
