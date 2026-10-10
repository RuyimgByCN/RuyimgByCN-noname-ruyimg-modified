# 阶段 14：MatchManifest 冻结 + 哈希链 ResolutionRecord

> 对应架构计划 v2.0 §5（MatchManifest）、§22.1–22.2（ResolutionRecord + 哈希链）、
> §41（live match 版本固定）。详见 ADR 0003。

## 目标

补齐 v2.0 复审指出的两项基础缺口，使架构可以冻结为基线候选：

1. **MatchManifest**：每局对局在 `StartGame` 时钉死一个不可变契约，涵盖
   引擎语义、Rule IR / VM ABI、内容锁、RNG 规格、预算与初始座次。
2. **哈希链 ResolutionRecord**：规范日志只记录成功提交的结算段，每条
   记录携带前后 Revision / State Hash、RNG 检查点、`ResolutionOutcome` 与
   `previous_record_hash → record_hash` 链。

## 三套身份严格分离

| 轴 | 类型 | 用途 |
|---|---|---|
| 规则语义 | `EngineSemanticsId` | 确定性契约；跨目标共享 |
| 二进制制品 | `ArtifactBuildId` | 部署 / 回滚 / 精确复现 |
| 对局契约 | `MatchManifestHash` | 录像 / WAL / 快照引用 |

Native 与 WASM 永远不可能拥有相同的二进制哈希；它们通过跨目标一致性测试
共享同一个 `EngineSemanticsId`。

## 新增类型

```text
engine-api::manifest
├─ MatchManifest          // 不可变对局契约
├─ MatchManifestHash      // 32B 规范哈希
├─ EngineSemanticsId      // 规则语义（跨目标共享）
├─ ArtifactBuildId        // 具体制品
├─ RuleIrVersion          // Rule IR 版本
├─ VmAbiVersion           // VM ABI 版本
├─ ContentLockHash        // 内容依赖锁
├─ RuleRngSpec            // RNG 算法 + 种子
├─ BudgetProfile          // 预算上限
├─ MatchConfig            // 模式 / 人数 / 变体
├─ InitialPlayer          // 初始座次
├─ ResolvedRulesetRef     // 规则集引用
├─ ProtocolProfile        // 协议档
└─ ManifestRejectionReason// 稳定错误码

engine-core::replay
├─ ResolutionRecord       // 规范结算记录
├─ ResolutionOutcome      // Committed / ChoiceBarrier / MatchCompleted / ...
├─ RuleRngCheckpoint      // RNG 检查点
└─ RecordHash             // 32B 记录哈希
```

## 行为

- `Engine::submit(StartGame)` 首次成功后：
  - 由命令参数 + 引导后的状态构造 `MatchManifest`
  - 计算 `MatchManifestHash` 并重建 `ReplayLog` 头部
  - 此后 `manifest()` 返回 `Some`，对局结束前不变
- 每个**成功提交**的段同时写入：
  - `ResolutionRecord`（哈希链）
  - `AcceptedCommand`（紧凑形式，保持黄金向量兼容）
- 拒绝的 Command 不进入规范日志（架构 §12、§22.1）
- `ReplayLog::verify_chain()` 在 O(n) 内检出第一条断链

## 与 GameState 的独立性

`MatchManifest` 不属于 `GameState`，不进入规范状态哈希。这保证了所有现有
黄金向量（start / identity match）的哈希不变——由
`phase14_manifest::manifest_independent_of_state_hash_keeps_golden_vectors`
钉死。

## CLI / 测试

```bash
cargo test -p engine-api              # manifest 哈希稳定性
cargo test -p engine-core --lib replay# 哈希链自洽
cargo test -p engine-core --test phase14_manifest   # 阶段 14 集成
cargo test --workspace                # 全量（含黄金向量）
```

phase14 覆盖的验收门：

- `StartGame` 后 manifest 钉死且非零
- 相同 Start 参数钉死相同 manifest hash
- 不同 seed 产生不同 manifest hash
- 每个成功 Command 都产生一条 ResolutionRecord
- `ResolutionOutcome` 正确分类（Committed / ChoiceBarrier）
- 哈希链端到端可验证
- `previous_record_hash` 正确链接
- 重放重建相同链 + 相同 manifest
- 快照 + manifest 恢复保留 pinned hash
- 黄金 start (4,42) 哈希不受影响

## 确定性契约（更新）

```text
相同 EngineSemanticsId
+ 相同 Rule IR / VM ABI
+ 相同 MatchManifestHash
+ 相同初始规范状态
+ 相同已提交 Command 序列
= 相同 Revision、State Hash、DomainEvent 和最终结果
```

## 后续接线（阶段 15+）

- 服务端房间：在 WAL 记录中写入 `MatchManifestHash`，重连时校验
- 录像文件：头部强制携带 manifest hash；播放器拒绝不匹配的录像
- 快照：增加可选 `manifest_hash` 字段（当前通过 `restore_snapshot_with_manifest` 传入）
- 跨目标：Native / WASM conformance 通过后才能 bump `EngineSemanticsId`
