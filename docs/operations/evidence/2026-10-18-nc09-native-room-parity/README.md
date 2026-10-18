# NC09 旧线房间测试场景 → 原生后端覆盖对照（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：旧线房间测试场景**4 项中 3 项已在原生后端覆盖**（本批补齐多房间隔离与信箱有界）；余 1 项（锦标赛 manifest 固定）为原生未实现能力，明确登记

## 1 旧线场景 → 原生覆盖对照

| 旧线场景 | 原生覆盖 | 证据 |
|---|---|---|
| `room_flow.rs::two_players_start_and_idempotent_command` | ✅ 入座 + 命令 + **幂等重放** | `nc09_native_actor.rs::nc09_native_actor_join_command_snapshot` |
| `room_flow.rs::phase15_manifest_pinned_and_recovery_preserves_it` | ⚠️ **部分**：`Recover`（快照+WAL 重放）已覆盖；**锦标赛 manifest 固定/分块签名未实现** | `nc09_dual_backend.rs::nc09_native_backend_recover_matches_live_state`；缺口见「原生 actor 未接线」 |
| `multi_room_stress.rs::stress_10_concurrent_rooms` / `rooms_with_different_seeds_are_isolated` | ✅ **本批新增**：5 个原生房间摘要互不相同；一个房间的命令不影响其它房间 | `nc09_dual_backend.rs::nc09_native_rooms_are_isolated_and_mailbox_bounded` |
| `multi_room_stress.rs::room_mailbox_is_bounded` | ✅ **本批新增**：容量 1 的有界信箱在未消费时拒绝第二个（`try_send` 失败） | 同上 |

补充已覆盖项（旧线未单列但原生已验）：**房间密码**、**StartGame 开局确认**、**观战者拒绝提交/选将**、**命令后个性化推送**、**选将（房间级）**。

## 2 本批新增用例

`nc09_native_rooms_are_isolated_and_mailbox_bounded`：

| 判据 | 结果 |
|---|---|
| 5 个不同种子的原生房间 → 状态摘要互不相同 | 通过 |
| 房间 0 推进一条命令 → 其它房间摘要**不变** | 通过 |
| 容量 1 信箱：首个 `try_send` 成功、第二个失败（有界语义） | 通过 |

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向（含 `persist` 既有 2 项） | **9 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **88 passed / 0 failed**（87 → +1） | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8266 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC09）

- [ ] **锦标赛面**：manifest 固定（`pin_manifest`）与 WAL 分块签名——原生 actor 当前**未实现**
      （旧线 `Recover` 消息在原生侧明确返回未接线，不伪造）
- [ ] 协议层选将（`engine-api`/`engine-protocol` 共享面，需协调）
- [ ] 真实浏览器/桌面端到端与平台档位取证
- [ ] 慢消费者背压专项（当前依赖 `Outbound::send` 丢弃语义，未做专项用例）
