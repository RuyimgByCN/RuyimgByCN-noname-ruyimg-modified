# NC05/NC09 房间级正式选将（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC05**（[方案 §NC05](../../../architecture/native-convergence-plan.md#nc05)）+ **NC09** 联机面
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**房间级选将入口已实现 + 回归通过**；协议 crate 未改动（避免与并行任务冲突）

## 1 缺口（前一批复核发现）

原生房间协议（`room_native::submit`）只映射 `PlayCard`/`EndPhase`/`ResolveChoice`/`StartGame`
——`Engine::assign_general*` **未进入房间协议** → 联机房间内玩家无法选将（客户端 WASM 路径
可用，属本地/单机）。

## 2 实现（**房间级消息**，不动共享协议 crate）

| 面 | 实现 |
|---|---|
| 消息 | `RoomMsg::ChooseGeneral { connection_id, name, reply }`（`room.rs`）——不新增 `CommandKind` 变体，避免改动 `engine-api`/`engine-protocol`（共享面，含旧线消费者） |
| 原生会话 | `NativeRoomSession::assign_general(connection_id, name)`：座位取**入座座位**（反冒充）；观战者拒绝；未知武将**明确报错**；返回 `(revision, state_hash_hex)` |
| 原生 actor | 处理该消息：成功后向**全体**推送新 `Snapshot`（视图/选择/权威动作），再回 `{"revision","state_hash"}` |
| 旧线房间 | 明确回「旧线房间后端不支持选将（请启用原生后端 NONAME_NATIVE_ROOMS=1）」——**不静默** |

> 注意（与前端切片一致的事实）：内核 `assign_general_named` **不推进 revision**（变化体现在
> `state_hash`），故推送/缓存失效判据不能用 revision 变化；actor 的推送判据是「操作成功」。

## 3 回归（`nc09_native_room_session.rs` 新增第 7 项）

| 判据 | 结果 |
|---|---|
| 旧线后端 → 明确「不支持」（错误文本含「不支持」） | 通过 |
| 原生后端入座者选将 → 成功返回 `state_hash`（或该武将不在目录时**明确报错**） | 通过 |
| 观战者选将 → 拒绝（`spectator`） | 通过 |

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向（含 `persist` 既有 2 项） | **9 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **87 passed / 0 failed**（84 → +3） | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8259 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办

- [ ] **协议层选将**（若产品需要走 HTTP/WS 标准命令面）：需在 `engine-api`/`engine-protocol`
      增加变体或消息，并处理旧线消费者的穷尽匹配——共享面，需与并行任务协调
- [ ] 候选**按座位门控**（赛前大厅、双将明置、选将信息隔离）
- [ ] 选将 UI 面板（客户端已有适配层/Worker/Client，缺界面）
- [ ] 真实浏览器端到端与平台档位
