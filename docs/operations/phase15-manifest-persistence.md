# 阶段 15：MatchManifest 接入持久化层（WAL + 恢复）

> 对应架构计划 v2.0 §5.3（MatchManifestHash）、§24（服务端持久化顺序）、
> §31.5（滚动发布 / live match 固定版本）。是阶段 14 文档中标注的
> 「后续接线（阶段 15+）」第一条。

## 目标

阶段 14 在引擎层引入了 `MatchManifest` 与哈希链 `ResolutionRecord`，但
服务端的 WAL 与恢复路径仍是无 manifest 感知的。阶段 15 将 manifest hash
贯穿到持久化层，使：

1. **WAL 自描述**：每条 `WalEntry` 携带它所属对局的 `MatchManifestHash`。
2. **恢复保留 manifest**：从快照 + WAL 尾部重建的引擎仍持有原始 pinned
   manifest hash（重连 / 节点重启不变）。
3. **篡改检测**：恢复时校验每条 WAL entry 的 manifest hash 与 pinned 一致；
   不一致即拒绝（该 WAL 属于另一局对局）。

## 改动

### `apps/game-server/src/persist.rs`

```rust
pub struct WalEntry {
    pub revision: u64,
    pub state_hash: [u8; 32],
    pub command: Command,
    pub idempotency_key: u64,
    pub manifest_hash: MatchManifestHash, // 新增（§5.3）
}

pub struct RoomPersist {
    pub wal: Vec<WalEntry>,
    pub snapshots: BTreeMap<u64, Snapshot>,
    pub durable_revision: u64,
    manifest: Option<MatchManifest>,       // 新增：StartGame 时钉死
}
```

| 方法 | 行为 |
|---|---|
| `pin_manifest(m)` | StartGame 后钉死 manifest |
| `manifest_hash()` | 返回 pinned hash（未开始则全零） |
| `append(entry)` | 若 entry 未带 hash，则用 pinned hash 盖戳 |
| `recover_engine(persist)` | 见下 |

### `recover_engine` 恢复契约（§24, §31.5）

```text
1. 若 pinned manifest 存在且有快照：
   → restore_snapshot_with_manifest(snap, manifest)  // 保留 pinned hash
2. 重放 WAL 尾部，逐条校验 entry.manifest_hash == pinned_hash
   → 不一致 → Err("wal manifest mismatch at rev …")
3. 终局不变量：recovered.manifest_hash() == pinned_hash
   → 不一致 → Err
```

### `apps/game-server/src/room.rs`

- `StartGame` 自动开始成功后：`persist.pin_manifest(engine.manifest().clone())`
  在任何 `append` 之前调用
- 所有 `WalEntry` 构造点显式带 `manifest_hash: MatchManifestHash::ZERO`
  （由 `append` 用 pinned hash 盖戳）

## 持久化顺序（架构 §24，更新）

```text
接收并校验 Command
→ 内存中计算 Resolution Segment
→ 生成 Resolution Record（哈希链）
→ 持久化 WAL（每条带 manifest_hash）
→ 持久化成功
→ 发布 ObserverDelta / CommandResult
→ 异步生成 Snapshot 和外部结果
```

## 测试

```bash
cargo test -p game-server
```

覆盖：

| 测试 | 断言 |
|---|---|
| `wal_and_snapshot_recover_preserves_manifest` | 恢复后 `manifest_hash()` 等于 live；每条 WAL entry 带 hash |
| `wal_manifest_mismatch_is_rejected` | 注入伪造 manifest hash 的 entry → `recover_engine` 报错 |
| `phase15_manifest_pinned_and_recovery_preserves_it` | 房间级：自动开始 → 提交 → `Recover` 消息 → 成功 |

## 安全意义

- **重连一致性**：断线重连的客户端拿到的视图由恢复后的引擎投影，该引擎
  的 manifest hash 与对局开始时一致，防止版本漂移。
- **跨房间隔离**：WAL manifest mismatch 检测防止把 A 房间的日志错误地
  重放到 B 房间（架构 §31.2 资源隔离的语义保证）。
- **滚动发布**（§31.5）：旧房间恢复时 manifest hash 不变；新房间用新版本
  的 manifest。房间中途不会静默升级。

## 后续接线（阶段 16+）

- Durable Outbox（战绩 / 排名）引用 manifest hash
- 录像文件头强制校验 manifest hash（播放器拒绝不匹配）
- 快照增加可选 `manifest_hash` 字段（独立于 `restore_snapshot_with_manifest`）
- 签名：比赛房对 WAL 块签名，根哈希写入结果记录（§22.2）
