# 阶段 7：权威服务器与联机协议

## 运行

```bash
cd noname-next
cargo run -p game-server
# 默认 ws://0.0.0.0:7700/ws
# 健康检查: GET /health → ok
# 环境变量: GAME_SERVER_ADDR=127.0.0.1:7700
```

## 架构

```text
WebSocket 连接
  → Lobby (建房 / 入座 / 观战)
  → Room Tokio 任务（串行邮箱）
       → Engine 权威执行
       → WAL 先写后广播
       → 按观察者 project_player / Spectator 投影
```

## 协议（JSON，`NETWORK_PROTOCOL_VERSION = 5`）

`ObserverSnapshot` 含 `round/current_player/phase`，已知牌携带权威
`zone/kind`。`JoinOk`、`CommandOutcome`、`Snapshot` 同时返回当前观察者的
`choice` 与逐条经引擎试提交验证的 `legal_actions`。因此刷新、重连和纯网络客户端
不再依赖瞬态 DomainEvent 推断阶段，也不在 UI 内复制卡牌/技能合法性规则。

Client → Server：`Hello` · `CreateRoom` · `JoinRoom` · `Spectate` · `SubmitCommand` · `RequestSnapshot` · `Heartbeat` · `LeaveRoom`

Server → Client：`Welcome` · `RoomCreated` · `JoinOk` · `CommandOutcome` · `RoomRoster` · `Snapshot` · `Pong` · `Error`

字段设计对齐未来 Protobuf；当前为 serde 带 tag 的 JSON。

## 安全要点

| 控制 | 行为 |
|---|---|
| 权威状态 | 仅服务端 `Engine` 持有 |
| 投影 | 每人只收自己的 `ObserverSnapshot` |
| 合法动作 | 仅玩家收到自己的 `legal_actions`；旁观者始终为空 |
| Choice 恢复 | Choice 合同随快照持久化，重连时恢复同一 option token |
| Actor 防伪 | 入座后强制 `command.actor = seated player` |
| 幂等 | `IdempotencyKey` 重复提交不二次执行 |
| 序号 | `conn_seq` 不得回退 |
| 错误 | `Error` 消息截断且不带状态转储 |
| WAL | 接受命令先持久化再向客户端确认/广播 |

## 重连

`JoinOk.reconnect_token` + 再次 `JoinRoom{ reconnect_token }` 恢复座位与出站通道。

## 测试

```bash
cargo test -p engine-protocol -p game-server
node scripts/websocket-smoke.mjs # 另启 game-server；跑完整双端局、旁观、普通/Choice 重连
```
