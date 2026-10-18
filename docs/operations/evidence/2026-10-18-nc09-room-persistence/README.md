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

## 5 追加：落盘存储 + 启动恢复入口（2026-10-18 同轮）

新增 `apps/game-server/src/native_room_store.rs`（`NativeRoomStore`）：

| 能力 | 实现 |
|---|---|
| 落盘 | `save(&StoredRoom{room_id, password, state})`——先写 `<id>.json.tmp` 再 **rename**（同目录原子） |
| 读回 | `load_all()` 按房间号升序；**单文件损坏明确报错**（不跳过、不静默）；跳过 `.tmp` |
| 删除 | `remove(room_id)` |
| 启动恢复 | `room_native_actor::restore_room_from_disk(record, load_pack, …)`——按 `pack_refs` 重新装载包 → `from_saved` → 带存储重新 spawn |
| actor 落盘点 | ① 命令被接受（`new_revision > before`）；② 入座/重连（名册与令牌变化）；③ 房间级选将成功。`None` 存储 = 不落盘（测试/临时房间） |

回归 `nc09_room_store_roundtrip_and_restore`（第 9 项）：

| 判据 | 结果 |
|---|---|
| 落盘后 `42.json` 存在且 `.tmp` 已 rename | 通过 |
| `load_all` → `restore_room_from_disk` → 房间可用 | 通过 |
| 恢复后**凭导出前令牌入座命中座位 0** | 通过 |
| 损坏文件（`{ not json`）→ `load_all` **明确报错**（含「解析」） | 通过 |

实测：定向 **11 passed / 0 failed**（[targeted-2.log](targeted-2.log)）；game-server 全量
**92 passed / 0 failed**（[game-server-tests-2.log](game-server-tests-2.log)）；行数门禁 8274 文件
（[source-lines-2.log](source-lines-2.log)）。

### 仍未完成（更新）

- [ ] **服务端启动自动扫描**：`main.rs`/`Lobby` 尚未在启动时调用 `load_all` + `restore_room_from_disk`
      （存储与恢复入口已就绪并被测试覆盖，接线待做）
- [ ] 房间关闭/终局时 `remove` 落盘记录（当前只增不减）
- [ ] WAL 增量与校验和、落盘节流（当前每条接受命令全量写）
- [ ] 恢复后名册广播与在座客户端连接重建；限流/慢消费者反馈；联机存读档/录像；锦标赛面；
      平台与性能档位；服务端 SPA fallback
