# 阶段 16：自描述快照与录像文件（manifest hash 贯穿 §22.3 / §23）

> **历史阶段记录**（T16④ 补注，2026-10-11）：本文为旧线阶段存档，「当前/已完成/待做」类陈述仅反映撰写时点；现行状态以[进度主稿](../architecture/native-progress.md)、[覆盖清单](../native-coverage-manifest.md)与[规则基准](../architecture/native-rules-baseline.md)为准。

> 对应架构计划 v2.0 §22.3（录像头必须携带 MatchManifest）、§23（快照必须
> 携带 MatchManifestHash）。是阶段 14–15 之后 manifest 身份贯穿到持久化
> 制品的收尾工作。

## 目标

阶段 14 在引擎层引入 `MatchManifest`；阶段 15 将它接入服务端 WAL + 恢复。
阶段 16 让**快照**与**录像文件**本身也自描述其所属对局：

1. **快照自描述**（§23）：`Snapshot` 携带 `manifest_hash`；恢复时可校验
   快照属于正确的对局。
2. **录像文件自描述**（§22.3）：`ReplayLog` 头部已有 `manifest_hash`
   （阶段 14）；阶段 16 补齐 `to_bytes` / `from_bytes` 文件序列化 API 与
   `verify_manifest` 门，以及引擎级 `export_replay` / `import_replay`。

## 改动

### `crates/engine-core/src/snapshot.rs`

```rust
pub struct Snapshot {
    pub schema_version: u32,
    pub versions: VersionBundle,
    pub revision: u64,
    pub state_hash: [u8; 32],
    pub manifest_hash: MatchManifestHash,  // 新增（§23），#[serde(default)] 向后兼容
    pub state: GameState,
}
```

| 方法 | 行为 |
|---|---|
| `capture(state)` | manifest_hash = ZERO（pre-start / 旧路径） |
| `capture_with_manifest(state, mh)` | 盖戳 manifest hash |
| `restore()` | 仅校验 schema + state hash（不变） |
| `restore_with_manifest(expected)` | 额外校验 `manifest_hash == expected`；快照为零则跳过（向后兼容） |

新增错误：`SnapshotError::ManifestMismatch`。

### `crates/engine-core/src/replay/mod.rs`

| 方法 | 行为 |
|---|---|
| `ReplayLog::to_bytes()` | 序列化为 JSON（录像文件体） |
| `ReplayLog::from_bytes()` | 反序列化 |
| `ReplayLog::verify_manifest(expected)` | 校验头部 manifest hash；头部为零则跳过 |

新增 `ReplayFileError`（Serialize / Deserialize / ManifestMismatch）。

### `crates/engine-core/src/engine.rs`

| 方法 | 行为 |
|---|---|
| `Engine::snapshot()` | 现在用 `capture_with_manifest` 盖戳 pinned hash |
| `Engine::restore_snapshot_with_manifest()` | 用 `restore_with_manifest` 校验（§23） |
| `Engine::export_replay()` | 导出录像文件 JSON 字节 |
| `Engine::import_replay(bytes, expected)` | 校验 manifest → 重放 → 断言重建引擎 pinned hash 一致 |

### `apps/engine-cli`

```bash
cargo run -p engine-cli -- replay-export 4 42
# replay_bytes=1772
# export_manifest_hash=01f70fce…
# imported_state_hash=cf7dfcf4…
# imported_manifest_hash=01f70fce…
# replay_roundtrip=ok
```

## 设计：向后兼容

所有新增字段使用 `#[serde(default)]`：

- 旧快照（无 `manifest_hash`）反序列化为 ZERO，`restore_with_manifest` 跳过校验
- 旧录像（无 `manifest_hash`）头部为 ZERO，`verify_manifest` 跳过校验
- 黄金状态哈希不受影响（`manifest_hash` 在 GameState 之外）——由
  `phase16::golden_state_hash_unchanged_by_phase16` 钉死

## 测试

```bash
cargo test -p engine-core --test phase16_manifest_persistence
```

| 测试 | 断言 |
|---|---|
| `engine_snapshot_carries_manifest_hash` | 快照携带引擎 pinned hash；字节往返保留 |
| `snapshot_restore_with_correct_manifest_succeeds` | 正确 hash 恢复成功 |
| `snapshot_restore_with_wrong_manifest_rejected` | 错误 hash → `ManifestMismatch` |
| `replay_file_round_trip_preserves_manifest` | 导出→导入：state hash + manifest hash 一致；链可验证 |
| `replay_file_manifest_mismatch_rejected` | 期望错误 manifest → `ReplayFileError::ManifestMismatch` |
| `replay_file_without_manifest_expectation_imports` | `expected=None` 跳过校验（诊断路径） |
| `golden_state_hash_unchanged_by_phase16` | 黄金 start (4,42) 哈希不变 |

## 安全意义

- **录像完整性**：播放器导入录像时可校验其 manifest hash 与当前引擎兼容，
  拒绝跨语义版本的录像（§41）。
- **快照归属**：快照恢复时校验它属于正确的对局，防止把 A 对局的快照
  错误恢复到 B 对局（§23）。
- **隐私边界**：manifest hash 不泄露任何隐藏信息（它只描述引擎/规则/内容版本）。

## 后续接线（阶段 17+）

- Durable Outbox（战绩 / 排名）引用 manifest hash
- 比赛房对 WAL 块签名，根哈希写入结果记录（§22.2）
- Replay Capsule：高价值比赛录像打包引擎制品 + 内容包（§22.4）
- 隐私分级录像：Full / Player / Public 从授权视图构建（§22.5）
