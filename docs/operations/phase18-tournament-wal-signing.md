# 阶段 18：比赛 WAL 块签名（§22.2）

> 对应架构计划 v2.0 §22.2（哈希链 / 签名 / 独立服务验证）与 §31.3（比赛房
> 需 WAL、哈希链、签名、长期制品保留）。

## 目标

为比赛 / 排名房提供防篡改终局结果验证：

1. WAL entries 分组成**哈希链块**（`WalBlock`）
2. 每个块用 **keyed-BLAKE3 MAC** 签名（`WalBlockSignature`）
3. 终局时生成 **MatchResultRecord**，携带 root hash + 终局签名
4. **独立验证服务**无需私钥即可校验链完整性，只需已知公钥 ID 即可验签

## 类型

```text
apps/game-server/src/persist.rs
├─ WalBlock               // 哈希链块：sequence + revision range + entry_hashes + chain link
├─ WalBlockSignature      // keyed-BLAKE3 签名 + key_id
└─ MatchResultRecord      // 终局记录：manifest_hash + outcome + root_hash + signatures
```

## 流程

```text
StartGame → pin_manifest
→ 接受 Command → append(entry)        // 每条带 manifest_hash
→ ... 对局进行 ...
→ match 结束时：
    build_blocks(block_size)           // 分块 + 计算哈希链
    seal_blocks(blocks, key, key_id)   // 签名每个块
    finalize_result(outcome, rev, key, key_id)  // 生成终局记录 + root 签名
```

## 独立验证路径（§22.2）

```rust
// 验证者只需 blocks + signatures + 已知 key（公钥 ID 对应的对称密钥）
RoomPersist::verify_block_chain(&blocks)?;           // 哈希链完整性
RoomPersist::verify_block_signatures(&blocks, &sigs, &key)?;  // 签名验证
// result_signature 覆盖 wal_root_hash
assert_eq!(record.result_signature.block_hash_hex, hex(&record.wal_root_hash));
```

哈希链验证**不需要**密钥；签名验证需要已知公钥对应的对称密钥。篡改任何
entry、block_hash、链链接或签名都会被检测到。

## 设计决策

- **Keyed-BLAKE3**：与 `content-runtime::sign` 包签名使用相同方案，避免引入
  第二套密钥基础设施
- **block_size 可配置**：默认 16 entries/block；小 block = 更细粒度验证，
  大 block = 更少签名开销
- **result_signature 覆盖 root**：终局签名只需验证一个哈希，而非整条链，
  简化独立验证者的工作
- **manifest_hash 贯穿**：终局记录携带 manifest_hash，验证者可拒绝跨对局
  混淆

## 测试

```bash
cargo test -p game-server --test tournament_wal_signing
```

| 测试 | 断言 |
|---|---|
| `block_chain_hashes_verify` | 哈希链端到端可验证 |
| `block_hashes_are_deterministic` | 相同 WAL → 相同块哈希 |
| `block_signatures_verify` | 正确密钥验签通过；错误密钥失败 |
| `block_chain_tamper_detected` | 篡改 block_hash 在 index 处检出 |
| `entry_tamper_breaks_block_hash` | compute_hash 与存储一致 |
| `result_record_carries_root_and_outcome` | 终局记录含 root + outcome |
| `independent_verifier_attestation` | 完整独立验证路径通过 |
| `empty_wal_produces_zero_root` | 空 WAL → 零 root |

## §31.3 持久化等级对照

| 等级 | WAL | 哈希链 | 签名 | 长期保留 |
|---|---|---|---|---|
| 临时 / 本地房 | ✗ | ✗ | ✗ | ✗ |
| 休闲房 | 异步快照 | ✗ | ✗ | ✗ |
| 排位房 | WAL 确认 | ✅ | ✗ | ✗ |
| **比赛房** | WAL | ✅ | ✅ | ✅ |

阶段 18 完成了比赛房所需的全部签名能力。实际接线到 room.rs 的比赛房路径
可后续添加（当前通过 `RoomPersist` API 直接调用）。
