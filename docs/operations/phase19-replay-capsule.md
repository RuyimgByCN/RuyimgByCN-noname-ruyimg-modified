# 阶段 19：Replay Capsule（§22.4）

> **历史阶段记录**（T16④ 补注，2026-10-11）：本文为旧线阶段存档，「当前/已完成/待做」类陈述仅反映撰写时点；现行状态以[进度主稿](../architecture/native-progress.md)、[覆盖清单](../native-coverage-manifest.md)与[规则基准](../architecture/native-rules-baseline.md)为准。

> 对应架构计划 v2.0 §22.4（Replay Capsule：为高价值比赛录像打包引擎语义
> 制品 + 内容包 + 录像 + 快照，使长期重放不依赖外部制品）。

## 目标

高价值比赛的录像可能在多年后重放——届时引擎二进制、内容包和编译器/VM
版本可能已不再是默认版本。Replay Capsule 将重放所需的一切打包为单个
可序列化制品：

1. 引擎语义版本（`EngineSemanticsId`）
2. 可运行的匹配制品（Native + WASM `ArtifactBuildId`）
3. 内容包（嵌入或引用）
4. 编译器 / VM 兼容信息（`CapsuleCompat`）
5. 录像日志字节 + 可选快照字节

两种模式：
- **Embedded**：内容包字节内联，capsule 自包含，无需外部制品
- **Referenced**：内容包按哈希引用，capsule 更小，依赖外部制品仓库

## 类型

```text
crates/engine-core/src/capsule.rs
├─ ReplayCapsule          // 自包含/引用的录像包
├─ CapsuleMode            // Embedded | Referenced
├─ CapsuleContentEntry    // 内容包条目（id + version + hash + bytes?）
├─ CapsuleCompat          // Rule IR / VM ABI / schema 版本快照
└─ CapsuleError           // Schema / manifest / decode 错误
```

## API

```rust
// 构建 capsule（Embedded 模式 + 快照）
let capsule = engine.build_capsule(
    CapsuleMode::Embedded,
    true,  // include snapshot
    &[("standard", "1.0.0", content_hash, Some(pkg_json_bytes))],
)?;
let bytes = capsule.to_bytes()?;  // 序列化

// 打开 capsule 并重放（无需外部制品）
let (replayed_engine, capsule) = Engine::open_capsule(&bytes)?;
assert_eq!(replayed_engine.state_hash(), original_hash);

// 完整性验证
capsule.verify()?;  // schema + manifest hash + replay + snapshot decode
capsule.is_self_contained();  // 所有内容包是否内联
```

## 完整性验证

`ReplayCapsule::verify()` 检查：
1. Schema 版本匹配 `CAPSULE_SCHEMA_VERSION`
2. `manifest_hash` 等于 `manifest.manifest_hash()` 的重算值
3. `replay_bytes` 可成功反序列化为 `ReplayLog`
4. （如有）`snapshot_bytes` 可成功反序列化为 `Snapshot`

## 设计决策

- **JSON 序列化**：与 replay log / snapshot 一致，便于诊断和跨语言读取。
  未来可切换为二进制格式（不影响 schema 版本语义）
- **manifest_hash 贯穿**：capsule 携带 manifest + manifest_hash；打开时
  校验重放引擎的 pinned hash 与 capsule 一致
- **compat 快照**：记录 Rule IR / VM ABI / 各 schema 版本，使重放器可
  判断是否兼容当前引擎
- **两种 artifact build id**：Native + WASM 都记录，便于跨目标重放
- **outcome 可选**：未完成对局的 capsule 无 outcome；终局 capsule 携带

## 测试

```bash
cargo test -p engine-core --lib capsule                    # 单元
cargo test -p engine-core --test phase19_replay_capsule    # 集成
```

| 测试 | 断言 |
|---|---|
| `capsule_build_open_replay_round_trip` | 构建→序列化→打开→重放：state hash + manifest hash 一致 |
| `capsule_without_snapshot` | 无快照 capsule 仍可重放 |
| `capsule_embeds_content_packages` | Embedded 模式内联内容包字节 |
| `capsule_referenced_mode_no_inline_bytes` | Referenced 模式不内联字节 |
| `capsule_tampered_manifest_detected` | 篡改 manifest_hash → `ManifestHashMismatch` |
| `capsule_carries_compat_info` | 携带 Rule IR / VM ABI / artifact build ids |
| `capsule_outcome_only_when_finished` | 未完成对局无 outcome |
| `capsule_before_start_fails` | StartGame 前构建失败 |
| `capsule_long_match_replays` | 多命令长对局重放正确 |

## §22.5 隐私等级（后续）

Replay Capsule 当前携带完整（Full/Referee）replay。§22.5 的 Player / Public
Replay 隐私分级需从授权视图构建，不能先生成完整录像再删除字段——这条不变量
已在 `security_observer_leak.rs`（§37.9）中验证。
