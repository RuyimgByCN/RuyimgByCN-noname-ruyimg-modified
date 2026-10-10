# 治理设施真内容操作文档（M8 验收门）

> 历史参考（2026-09-16 标注）：本文描述旧线及所注明日期的状态，文中的“当前”、阶段验收和统计不代表新原生线完成度。最新入口见 [旧实现参考](../architecture/legacy-reference.md)。

> 更新：2026-08。对应架构计划 §52.2 M8 总验收门第 3 项：
> 「治理设施均有『真内容』操作文档」。本页给出在真内容（26 包 / 6068 技能 /
> 2526 武将）上操作各治理设施的路径与命令。

## 1. 内容同步与来源清单（M3 基线）

```bash
node scripts/sync-original-content.mjs --core-source /path/to/noname   # 同步
node scripts/sync-original-content.mjs --verify-only                    # 门禁：可复现
```

- `content/original/catalog.json`：6068 技能，每条含 `source`（上游行号）与 `migration_status`。
- `content/original/legacy-source/`：上游源码保留（364 文件），同步门禁校验。
- 许可状态由 `content/original/README.md` 记录（未审来源不标记可再分发）。

## 2. 内容打包与内容锁（content-cli）

```bash
cargo run -p content-cli -- pack content/community/hero.skill \
  --out content/community/dist/hero.npack.json \
  --package-id hero --namespace community --version 0.1.0 \
  --characters content/community/characters.json --metadata content/community/metadata.json
cargo run -p content-cli -- --store /tmp/npack install pkg.json --allow-unsigned  # 开发期
```

- 每个 ported 包 `metadata.json` 含 `license` 块（`code_license`/`default_asset_license`/
  `asset_sources`），32 包全部覆盖（`m8_governance` 测试持续校验）。
- 内容锁（ContentLock）随 lobby 每个房间钉死；滚动升级不影响进行中对局
  （`apps/game-server/tests/m8_rolling_upgrade.rs` 演练）。

## 3. 规则哈希与双后端一致性

```bash
bash scripts/cross-target-hash.sh    # Native vs WASM 全包哈希一致性门禁
cargo run -p engine-cli -- hash 4 42 # 单点 StartGame 哈希
```

- 包级 `logic_hash` 由 `rebuild-pack.sh <pkg>` 重建后随 npack 提交；哈希冻结测试
  （如 `original_sp_dingpan_buzhi`）在内容变更时需同步更新 `PACKAGE_LOGIC_HASH` 与
  `modules.len()`。

## 4. 录像 / 快照 / 对局审计（match manifest）

```bash
cargo run -p engine-cli -- manifest 4 42            # 钉死 MatchManifest 哈希 + 链状态
cargo run -p engine-cli -- replay-export 4 42       # 录像导出/导入 + manifest 校验
cargo test -p game-server --test tournament_wal_signing  # 比赛级 WAL 签名
cargo test -p game-server --test fault_injection         # 故障注入
```

## 5. 模式与内容包兼容

- `docs/migration/mode-content-compatibility.md`：模式 × 内容包矩阵（身份/国战/单挑已实现）。
- `engine-cli match 2 <seed> <steps> --ruleset single`：单挑可玩路径。

## 6. devtools 全套（真内容路径）

```bash
cargo test -p devtools
cargo run -p legacy-analyzer -- analyze content/original/legacy-source/character/sp/skill.js \
  --md report.md --skeletons out/      # 上游技能分析与分级画像
```

- `content/original/skill-capability-matrix.md`：5988 技能分级画像（A/B/C/D 仪表）。
- `apps/extension-studio`：清单 / 骨架 / 本地 Golden / 签名 / 发布到 channel。
