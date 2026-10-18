# NC02 安装包身份与执行内容一致：以已验证安装副本为权威（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC02**（[方案 §NC02](../../../architecture/native-convergence-plan.md#nc02)）
- 反例来源：[2026-10-18 架构审查](../2026-10-18-architecture-audit/README.md)「安装包载入与混合包」（`loader-probe.log`）
- 规则版本 v321 / 存档 v105（本次无规则与格式变化）
- 结论：**根因已修 + 7 项验收回归通过 + game-server 全量通过**；跨目标/平台面待后续阶段

## 1 根因（已确证并修复）

`apps/game-server/src/native.rs::NativeContent::load` 以 `InstalledMeta.extension_root`
（**drop-in 原始来源目录**）为执行内容来源：从该目录读 `assets/cards.json` 与
`package.npack.json`。后果与审计反例一致：

1. VM 包安装后**移除原始来源目录** → 已安装记录与 store 仍在，装载却因回读来源路径失败；
2. 同一已安装包**只改原始来源**（新增有效 `assets/cards.json`）→ 装载结果由 1 个 VM 技能变为 0 个并载入 7 张卡牌。

## 2 修复

| 面 | 修复前 | 修复后 |
|---|---|---|
| 执行内容来源 | `meta.extension_root`（源目录） | **内容锁 + 安装副本**：`current_lock()` → `load_locked_packages()`（逐包校验 `logic_hash`/`content_hash`） |
| 包身份 | 仅按 `package_id` 在安装列表里找 | 命名空间 + 包 ID + 版本 + 内容指纹；**同 id 多命名空间明确失败**，`load_scoped(ns, id)` 精确选择 |
| 卡牌资源 | 源目录 `assets/cards.json` | `asset_paths` 相对路径 → `resolve_active_asset`（安装目录不可变路径） |
| 混合包 | 有 `cards.json` 时**丢弃** VM 声明 | VM 声明 `extend` 并入桥接产物（卡牌与 VM 技能同时保留） |
| `extension_root` | 执行内容权威 | 仅保留**来源信息**（不再参与装载） |

## 3 验收回归（新增 `apps/game-server/tests/nc02_install_identity.rs`，7 项）

| 验收项 | 用例 | 结果 |
|---|---|---|
| 删除原始来源后仍加载安装版本 | `nc02_loads_installed_copy_after_source_removed` | 通过（VM 技能来自安装副本） |
| 修改原始来源不改执行内容 | `nc02_source_mutation_does_not_change_execution_content` | 通过（源目录新增 7 张卡 → 装载仍 0 卡 + 1 VM 技能） |
| 混合卡牌与 VM 包不丢技能 | `nc02_mixed_package_keeps_cards_and_vm_skills` | 通过（npack 追加卡牌资产登记 → 1 卡 + 1 VM 技能） |
| 同 id 不同命名空间准确选择 | `nc02_same_id_different_namespace_requires_explicit_selection` | 通过（默认报错提示命名空间；`load_scoped` 精确装载） |
| 安装内容损坏明确失败 | `nc02_corrupt_installed_copy_fails_clearly` | 通过（破坏安装副本 `package.json` → 明确报错，不静默降级） |
| 重复装载稳定 | `nc02_repeated_loads_are_stable` | 通过（卡牌/VM 技能/包身份一致） |
| 升级不改写已固定内容锁的局 | `nc02_upgrade_does_not_rewrite_pinned_lock` | 通过（旧锁仍装载旧版本；活动集取到 2.0.0） |

## 4 验证结果

```
cargo test --offline -p game-server --test nc02_install_identity
cargo test --offline -p game-server --no-fail-fast
cargo test --offline -p content-native-loader -p content-runtime -p engine-native-wasm --no-fail-fast
CARGO_TARGET_DIR=<tmp> RUSTFLAGS="-D warnings" cargo check --offline -p game-server --all-targets
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| NC02 定向 | **7 passed / 0 failed** | [targeted.log](targeted.log) |
| game-server 全量 | **56 passed / 0 failed / 0 ignored** | [game-server-tests.log](game-server-tests.log) |
| 相关包 | 85/0/0 | [deps-tests.log](deps-tests.log) |
| `-D warnings` 检查 | **被并行重构阻断**：`engine-core` 106 处 unused-import 错误（**无一条位于 `apps/game-server`**） | [gate-game-server-all-targets.log](gate-game-server-all-targets.log) |
| 行数门禁 | 8220 文件 ≤ 2000 行；单测 OK | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办（NC02）

- [ ] `-D warnings` 门禁复跑：待 `engine-core` 并行重构收敛（当前其 106 处未使用导入使依赖编译失败；game-server 自身无告警）
- [ ] 卡牌之外的资源面（配置、立绘、音频）装载同源化：本次只覆盖 `assets/cards.json`；`assets/config.json` 等仍由各功能面自行读取，需按同一 `resolve_active_asset` 口径迁移
- [ ] 玩家侧「安装/启用/升级/回退/卸载」的**产品入口**端到端（NC03/NC05）：本次覆盖服务端装载路径，未覆盖客户端 UI 与 Worker
- [ ] 已安装包被 GC/回收后的重新取得与缺失拒绝策略（`gc_staging` 之外的内容回收）
