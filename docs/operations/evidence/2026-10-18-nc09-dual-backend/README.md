# NC09 原生后端会话语义（双后端对照第一切片，2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**原生后端会话语义 2 项通过**；旧线测试套件跑在原生后端待续

## 1 口径

旧线房间（`room::spawn_room*`）与原生房间（`room_native_actor::spawn_room_native`）共用
`RoomMsg`/`RoomHandle`/`JoinReply` 与协议消息类型，故**同一批会话语义断言可在两后端复用**。
本切片先在原生后端把语义钉住（旧线侧由既有 `room_flow.rs` 覆盖）：

新增 `apps/game-server/tests/nc09_dual_backend.rs`（2 项）：

| 验收项 | 判据 | 结果 |
|---|---|---|
| 入座/满座/重连/观战/观战者提交 | 座位与角色正确；满座拒绝；重连**保留座位且令牌不变**；无效令牌失败；观战者无权威动作且提交失败 | 通过 |
| 非法命令明确拒绝 | 非当前回合座位推进 → `Rejected(InvalidActor)`（协议拒绝原因可见） | 通过 |

> 差异登记：旧线房间需先 `StartGame`（大厅前置），原生会话**建局即开局**（赛前大厅未建模）；
> 旧线房间另有锦标赛 WAL 分块签名等能力，原生 actor 尚未对齐（明确返回未接线错误）。

## 2 验证结果

```
cargo test --offline -p game-server --test nc09_dual_backend
cargo test --offline -p game-server --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 原生后端语义定向 | **2 passed / 0 failed**（含 `persist` 既有 2 项共 4 项） | [targeted.log](targeted.log) |
| game-server 全量 | **80 passed / 0 failed**（76 → +4） | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8254 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 3 未完成 / 待办（NC09）

- [ ] **旧线房间测试套件跑在原生后端**：`room_flow.rs`/`multi_room_stress.rs` 仍只跑旧线；
      需「同测试双后端」参数化（含密码、限流、断线重连、慢消费者）
- [ ] 旧线能力对齐：锦标赛 WAL 分块签名、`pin_manifest`、`Recover`
- [ ] 命令后个性化视图推送、actor 回包 `legal_actions`/`choice`
- [ ] 真实客户端/HTTP/WS 端到端、录像导出/跳转、平台档位
