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

- 复用 [`room_native`](../../../../apps/game-server/src/room_native.rs) 的 `NativeRoomSession`
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

## 5 追加：入口按后端选择（2026-10-18 同轮）

原生路径此前**不可达**（`lobby`/`main` 只起旧线房间）。本批接线：

| 面 | 实现 |
|---|---|
| 配置 | `ServerConfig` 新增 `native_rooms: bool`、`native_room_packages: Vec<String>`、`native_room_namespace: String`；env `NONAME_NATIVE_ROOMS` / `NONAME_NATIVE_ROOM_PACKAGES` / `NONAME_NATIVE_ROOM_NAMESPACE`。**默认关闭**（切换需显式启用并逐项验收） |
| 选择 | `room_native_actor::spawn_native_room_if_configured(...)`：未启用 → `Ok(None)`（调用方走旧线）；启用 → 装载配置包并起原生 actor；**装载失败明确报错，不静默回退旧线**（回退会让「已启用」成假象） |
| lobby | 建房处按配置选择后端；原生失败以 `ErrorCode::Internal` 返回 |

新增用例 `nc09_backend_selection_is_explicit`（第 2 项）：

| 判据 | 结果 |
|---|---|
| `native=false` → 不接管（`Ok(None)`） | 通过 |
| `native=true` + 包可装载 → 起原生房间且可入座（真实 tokio） | 通过 |
| `native=true` + 包缺失 → **明确报错**（错误含包名，不静默回退） | 通过 |

实测：定向 **4 passed / 0 failed**（含 `persist` 既有 2 项；[targeted-2.log](targeted-2.log)）；
game-server 全量 **76 passed / 0 failed**（[game-server-tests-2.log](game-server-tests-2.log)）；
行数门禁 8253 文件（[source-lines-2.log](source-lines-2.log)）。

### 仍未完成（更新）

- [ ] **旧线房间测试跑在原生后端**：`room_flow.rs`/`multi_room_stress.rs` 仍走旧线；
      需要「同测试双后端」或为原生后端补等价用例（含断线重连、满座、密码、限流）
- [ ] 旧线能力对齐：锦标赛 WAL 分块签名、`pin_manifest`、`Recover` 消息
      （原生 actor 当前明确返回未接线错误）
- [ ] 命令后**个性化视图推送**（`fanout_after_command` 等价物）、actor 回包
      `legal_actions`/`choice` 接线
- [ ] 原生后端的**真实客户端/HTTP/WS 端到端**与平台档位
