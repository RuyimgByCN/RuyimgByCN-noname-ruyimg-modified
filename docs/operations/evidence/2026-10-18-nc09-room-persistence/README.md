# NC09 房间跨重启持久化（会话级导出/恢复）（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）「重连 / 房间恢复」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**会话级导出/恢复已实现并回归通过**（状态、成员、令牌、计数器延续；令牌命中原座位）；**服务端启动自动恢复（接线）仍待做**

## 1 实现（`apps/game-server/src/room_native.rs`）

| 面 | 内容 |
|---|---|
| `NativeRoomState`（新，`serde` 可序列化） | `snapshot_hex`（引擎归档字节 hex）+ `max_seats`/`started` + `next_token`/`next_dedup`（**延续，避免令牌复用与命令重放**）+ `pack_refs`（`包名@版本`）+ `members`（含**重连令牌**） |
| `NativeRoomSession::snapshot_hex()` | 当前引擎归档字节 hex（落盘用） |
| `NativeRoomSession::export_state(hex)` | 导出可落盘状态（**不含**连接句柄/IO） |
| `NativeRoomSession::from_saved(state, packs)` | 按归档字节重建引擎（有包走 `restore_with_extensions`）+ 重建成员与**令牌→连接**映射 + 延续计数器 |
| `NativeMember` | 加 `Serialize/Deserialize` |

## 2 回归（`nc09_native_room_session.rs` 第 8 项）

`nc09_session_export_and_restore_roundtrip`：

| 判据 | 结果 |
|---|---|
| 两人入座 + 一条命令后导出 → JSON 落盘 → 读回 | 通过 |
| 成员随状态落盘（2 人）、包引用落盘（`shahai-swd…`） | 通过 |
| 恢复后**修订号与状态摘要**与导出时一致 | 通过 |
| 恢复路径按 `pack_refs` **重新装载**同内容包（与产品启动同路径） | 通过 |
| 导出前的**重连令牌**在恢复后仍命中**原座位**（座位 0、玩家 id 不变） | 通过 |

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向（含 `persist` 既有 2 项） | **10 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **91 passed / 0 failed**（90 → +1） | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8273 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |
| Gate 1b（`-D warnings`，bins+tests） | **exit 101，失败点在 `crates/engine-core`（并行重构的未提交改动，非本批文件）** | [gate1b.log](gate1b.log) |

> Gate 1b 说明：`game-server` 依赖 `engine-core`，后者当前工作区有**他人未提交**的未使用导入告警
> （`effect/mod.rs` 等）——与本批改动无关，但使 `-D warnings` 整体失败；本批文件未新增告警。

## 4 未完成 / 待办（NC09）

- [ ] **服务端启动自动恢复（接线）**：扫描落盘房间 → 按 `pack_refs` 装载包 → `from_saved` →
      以同房间号重新 spawn actor（含密码/邮箱容量）——本批只做**会话级**导出/恢复
- [ ] 落盘时机与原子性（每条接受命令后 / 定时快照 + 临时文件改名）、WAL 增量与校验和
- [ ] 恢复后的名册广播与在座客户端的连接重建（客户端自动重连已实现，见
      [autoreconnect 证据](../2026-10-18-nc09-autoreconnect/README.md)）
- [ ] 限流与慢消费者反馈、联机存读档/录像、锦标赛面、平台与性能档位、服务端 SPA fallback
