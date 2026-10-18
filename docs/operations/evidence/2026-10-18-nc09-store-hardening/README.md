# NC09 落盘硬化：密码哈希 + 状态校验和（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**密码不落明文（加盐哈希）+ 状态校验和校验**；运行期亦不再持有明文

## 1 实现

| 面 | 内容 |
|---|---|
| `StoredRoom.password_hash`（替代 `password`） | `blake3("noname-native-room-pw-v1:" ‖ room_id_le ‖ ":" ‖ password)` 的 hex；空密码 → 空串（表示无密码） |
| `StoredRoom.checksum` | `blake3(state JSON)`；`save` **总是重算**（调用方无需维护），`load_all` **校验**（不匹配明确报错，不静默接受损坏/篡改） |
| 运行期 | `spawn_room_native_persisted` 把明文**立即转哈希**后交给 actor（`spawn_room_native_persisted_with_hash`）；入座校验改为 `password_hash(id, 输入) == 运行期哈希` —— **内存与磁盘均无明文** |
| 恢复路径 | `restore_room_from_disk` 直接把磁盘上的哈希带回运行期（无需也无法还原明文） |

**登记边界**：盐取房间号（本地磁盘口径，防「明文直读」而非防离线爆破）；生产环境应改为随机盐 +
外部密钥存储——已在待办登记。

## 2 回归（`nc09_native_room_session.rs` 新增第 10 项）

`nc09_room_store_hashes_password_and_verifies_checksum`：

| 判据 | 结果 |
|---|---|
| 哈希 ≠ 明文；同输入稳定；**盐=房间号**（不同房号哈希不同）；空密码 → 空串 | 通过 |
| 落盘文件**不含明文** `secret` 且含 `checksum` 字段 | 通过 |
| 读回哈希一致 | 通过 |
| **篡改状态但保留旧校验和** → `load_all` **明确报错**（含「校验和」） | 通过 |

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向（含 persist 既有 2 项） | **13 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **94 passed / 0 failed**（93 → +1） | [game-server-tests.log](game-server-tests.log) |
| 行数门禁 | 8274 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC09）

- [ ] **随机盐 + 外部密钥/凭据存储**（当前盐=房间号，仅本地磁盘口径）
- [ ] 落盘**节流**与 WAL 增量（当前每条接受命令全量写）
- [ ] 恢复后**主动**向已连接者重播名册（当前靠客户端重连 + 令牌回座）
- [ ] 限流与慢消费者反馈、联机存读档/录像、锦标赛面、平台与性能档位、服务端 SPA fallback

## 5 追加：落盘节流（2026-10-18 同轮）

`persist_room` 增加签名节流：签名 = `(引擎修订号, 成员数, 开局位, next_token)`；与上次**成功**
落盘一致则直接返回（不重复全量写）。actor 循环持有 `last_persist_sig`，仅在 `save` 成功后才更新
（失败会重试下一次落盘点，不静默丢状态）。

效果：同一回合内的多次落盘点（命令后、观战入座、重复触发）不再重复写盘；**状态真正变化**
（新命令、名册/令牌变化、开局）时仍立即落盘。

实测：定向 **13 passed / 0 failed**（[targeted-2.log](targeted-2.log)）；game-server 全量
**94 passed / 0 failed**（[game-server-tests-2.log](game-server-tests-2.log)）；行数门禁 8274 文件
（[source-lines-2.log](source-lines-2.log)）。

### 仍未完成（更新）

- [ ] **WAL 增量**（当前每次变化仍全量写状态；增量需按修订追加 + 定期压缩）
- [ ] 随机盐 + 外部密钥/凭据存储
- [ ] 恢复后**主动**名册重播；**缺席座位超时**（当前恢复出的座位长期保留，无超时回收）
- [ ] 限流与慢消费者反馈、联机存读档/录像、锦标赛面、平台与性能档位、服务端 SPA fallback
