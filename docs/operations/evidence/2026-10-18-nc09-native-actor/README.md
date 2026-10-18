# NC09 原生权威房间 actor（IO 接线，2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**原生 actor（真实 tokio 任务）已实现 + 端到端用例通过**；入口按后端选择与旧线能力对齐待续

## 1 设计取舍（登记，不隐藏）

`room.rs` 的 actor 绑定旧线 `engine_core::Engine` 与旧线 `RoomPersist`
（`engine_api::Snapshot` 是**结构化旧线状态**，不能直接装原生归档），且旧线房间还承载
锦标赛 WAL 分块签名等**已测试**能力。因此本切片**并行**实现原生 actor
（`apps/game-server/src/room_native_actor.rs`）：

- 复用 [`room_native`](apps/game-server/src/room_native.rs) 的 `NativeRoomSession`
  （入座/重连/观战/命令/快照）与 `NativeRoomPersist`（原生归档快照 + WAL 重放恢复）；
- 复用 `RoomMsg`/`RoomHandle`/`JoinReply`/`Outbound` 与协议消息类型（`RoomHandle::new`
  为复用新增的构造函数）；
- 覆盖：入座/观战/命令（**幂等重放**、**反冒充**以入座座位覆盖协议 actor、提交前落
  WAL）、快照请求、断开、名册广播（`RoomMessage::RoomRoster`）、按 `snapshot_every`
  落原生归档快照；
- **未改**旧线房间（其能力与测试保持原样）。

## 2 回归（新增 `apps/game-server/tests/nc09_native_actor.rs`）

| 判据 | 结果 |
|---|---|
| 真实 tokio 任务：入座（座位/玩家 id 正确）→ 名册广播发出 → 快照返回 `current_player` | 通过 |
| 命令：以**入座座位**提交（协议 actor 被覆盖）→ `Accepted` | 通过 |
| **幂等重放**：同 `idempotency_key` 再提交 → 返回原结果（不重复执行） | 通过 |

## 3 验证结果

```
cargo test --offline -p game-server --test nc09_native_actor
cargo test --offline -p game-server --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 原生 actor 定向 | **1 passed / 0 failed**（含 `persist` 既有 2 项共 3 项） | [targeted.log](targeted.log) |
| game-server 全量 | **75 passed / 0 failed**（72 → +3） | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8253 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC09）

- [ ] **入口按后端选择**：`lobby.rs`/`main.rs` 仍只起旧线房间；需配置项选择原生 actor，
      并让既有 `room_flow.rs`/`multi_room_stress.rs` 跑在原生后端（或并行保留双后端）
- [ ] **旧线能力对齐**：锦标赛 WAL 分块签名（`tournament_wal_signing.rs`）、`pin_manifest`、
      `Recover` 消息（当前原生 actor 明确返回「未接线」错误，不伪造）
- [ ] 广播面：命令后的**个性化视图推送**（当前只广播名册；旧线有 `fanout_after_command`）
- [ ] `legal_actions`/`choice` 在 actor 回包中暂为空（会话已能给出，接线待补）
- [ ] 录像导出/跳转、WAL 落盘与跨进程恢复、客户端渲染、平台档位
