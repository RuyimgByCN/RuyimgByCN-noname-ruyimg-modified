# NC09 原生权威房间会话（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**原生权威会话（入座/命令/快照/重连/观战）已实现 + 5 项回归通过**；actor 循环与 IO 接线待续

## 1 动机与边界

`room.rs` 的 actor 循环绑定旧线 `engine_core::Engine`。NC09 要求「本地、WASM Worker 和
服务端使用**同一原生会话契约**」。本切片实现**原生权威会话**（`apps/game-server/src/room_native.rs`）：

- 使用原生引擎 + [`native_view`](apps/game-server/src/native_view.rs) 的协议映射（唯一映射点）；
- 覆盖：入座（座位分配/满座拒绝）、重连（令牌换绑、座位与令牌不变、无效令牌失败）、
  观战（公开投影、无权威动作、不能提交）、命令（协议 `CommandKind` → 内核动作 +
  结果/拒绝原因映射）、快照（`revision`/`state_hash` 与内核一致）；
- **不含** tokio actor、广播、持久化/WAL、HTTP/WS 接线（下一批；本模块无 IO，便于测试）。

## 2 回归（新增 `apps/game-server/tests/nc09_native_room_session.rs`，5 项）

| 验收项 | 判据 | 结果 |
|---|---|---|
| 入座与投影 | 首选座位命中、自动分配不重复、满座拒绝；投影含全部座位 | 通过 |
| 命令与拒绝 | 非当前回合座位推进 → `InvalidActor`；当前回合座位推进 → `Accepted` | 通过 |
| 重连 | 令牌换绑连接、座位与令牌不变；无效令牌明确失败 | 通过 |
| 观战 | 公开投影、无权威动作、提交被拒（"spectator cannot submit"） | 通过 |
| 快照 | `revision`/`state_hash` 与内核一致 | 通过 |

## 3 验证结果

```
cargo test --offline -p game-server --test nc09_native_room_session
cargo test --offline -p game-server --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 原生房间会话定向 | **5 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **69 passed / 0 failed**（64 → +5） | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8249 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC09）

- [ ] **actor 循环接线**：把 `room.rs` 的 `engine_core::Engine` 换成本会话（含广播/Outbound、
      幂等缓存、WAL 持久化、快照周期）——现有 `room_flow.rs`/`multi_room_stress.rs` 是其回归面
- [ ] 命令面只映射 `PlayCard`/`EndPhase`/`ResolveChoice`；`ActivateSkill`/`RevealGeneral`/
      `StartGame`（原生建局即开局，赛前大厅未建模）等形态明确返回 `Unsupported`
- [ ] `MatchOutcome`/`SkillView` 字段/`known_cards` 真实映射（见视图桥接证据）
- [ ] 录像与跳转、真实客户端渲染、平台档位取证
- [ ] HTTP/WS 层端到端（本切片无 IO，未覆盖网络边界）
