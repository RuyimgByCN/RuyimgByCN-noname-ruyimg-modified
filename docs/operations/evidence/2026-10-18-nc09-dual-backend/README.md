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

## 4 追加：旧线协议兼容 `StartGame`（2026-10-18 同轮）

旧线房间协议要求先发 `StartGame`；原生会话**建局即开局**（赛前大厅未建模）。为让旧线
房间测试可参数化跑在原生后端，`NativeRoomSession::submit` 现将 `StartGame` 作为
**开局确认**处理：

- 参数（座位数）与已建局一致 → `Accepted`（不重复建局，修订/摘要不变）；
- 参数不一致 → **明确报错**（`StartGame 座位数 N 与本局 M 不一致`），**不静默吞掉**。

新增用例 `nc09_native_backend_startgame_is_confirmation_only`（第 3 项）：

| 判据 | 结果 |
|---|---|
| 匹配的开局确认 → `Accepted` | 通过 |
| 座位数不一致 → 明确报错 | 通过 |

实测：定向 **3 passed / 0 failed**（含 `persist` 既有 2 项共 5 项；[targeted-2.log](targeted-2.log)）；
game-server 全量 **81 passed / 0 failed**（[game-server-tests-2.log](game-server-tests-2.log)）；
行数门禁 8254 文件（[source-lines-2.log](source-lines-2.log)）。

### 仍未完成（更新）

- [ ] **旧线房间测试套件参数化跑在原生后端**：`room_flow.rs`/`multi_room_stress.rs` 仍只跑
      旧线；`StartGame` 兼容已具备（本批），仍需处理：密码校验、限流、慢消费者、
      `Recover` 消息（原生 actor 明确返回未接线）
- [ ] 旧线能力对齐：锦标赛 WAL 分块签名、`pin_manifest`
- [ ] 命令后个性化视图推送、actor 回包 `legal_actions`/`choice`
- [ ] 真实客户端/HTTP/WS 端到端、录像导出/跳转、平台档位

## 5 追加：房间密码（2026-10-18 同轮）

原生 actor 原先**忽略** `Join.password`（旧线房间会校验）——这是双后端参数化的阻塞项之一。
本批补齐：`spawn_room_native_with_password`（空串 = 无密码）在 Join 处按旧线同口径校验，
不匹配返回 `unauthorized`；`spawn_room_native` 保持无密码默认。

新增用例 `nc09_native_backend_enforces_room_password`（第 4 项）：

| 判据 | 结果 |
|---|---|
| 非空房间密码 + 错误密码 → 拒绝（`unauthorized`） | 通过 |
| 正确密码 → 入座成功 | 通过 |

实测：定向 **4 passed / 0 failed**（含 `persist` 既有 2 项共 6 项；[targeted-3.log](targeted-3.log)）；
game-server 全量 **82 passed / 0 failed**（[game-server-tests-3.log](game-server-tests-3.log)）；
行数门禁 8254 文件（[source-lines-3.log](source-lines-3.log)）。

### 仍未完成（更新）

- [ ] 旧线房间测试套件参数化跑在原生后端：`StartGame` 兼容与**房间密码**已具备；仍需
      限流（`max_cmds_per_sec`）、慢消费者背压、`Recover` 消息（原生 actor 明确未接线）
- [ ] 旧线能力对齐：锦标赛 WAL 分块签名、`pin_manifest`
- [ ] 命令后个性化视图推送、actor 回包 `legal_actions`/`choice`
- [ ] 真实客户端/HTTP/WS 端到端、录像导出/跳转、平台档位
