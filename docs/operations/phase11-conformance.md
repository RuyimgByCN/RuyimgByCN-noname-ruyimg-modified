# 阶段 11：一致性黄金用例、更新通道与扩展工作室

> **历史阶段记录**（T16④ 补注，2026-10-11）：本文为旧线阶段存档，「当前/已完成/待做」类陈述仅反映撰写时点；现行状态以[进度主稿](../architecture/native-progress.md)、[覆盖清单](../native-coverage-manifest.md)与[规则基准](../architecture/native-rules-baseline.md)为准。

阶段 10 解决「装得安全」；阶段 11 解决「装完能验」与「批量更新」。

## 组件

| 组件 | 路径 | 职责 |
|---|---|---|
| `conformance` | `crates/conformance` | 钉死 start/match 黄金哈希；CI 入口 `conformance` |
| `content-runtime::channel` | 更新目录 | ChannelManifest / plan / apply |
| `content-cli pack/channel-*` | 打包 DSL + 应用通道 |
| `extension-studio` | 无头工作室 | analyze → skeleton → publish/install |

## 黄金用例

```bash
cargo test -p conformance
cargo run -p conformance
```

当前钉死向量（`engine-cli hash` / `match` 对齐）：

- start: `(4,42)` `(2,1)` `(8,99)` — ruleset `bootstrap`
- identity match: `(4,42)` `(2,7)` `(8,99)` — 终局 hash，`max_steps=5000`
- package: `content/standard/jianxiong.skill` 可复现 logic_hash

**改哈希算法或规则语义时必须同步更新向量并写 ADR。**

## 更新通道

```text
channel/
  channel.json          # catalog + content hashes
  pkgs/foo-1.0.0.json
  pkgs/foo-1.0.0.sig.json
```

```bash
cargo run -p content-cli -- --store /tmp/npack channel-plan /path/to/channel.json
cargo run -p content-cli -- --store /tmp/npack channel-apply /path/to/channel \
  --key secret
# dry-run
cargo run -p content-cli -- --store /tmp/npack channel-apply /path/to/channel --dry-run
```

规则：

- 每个 slot 取 catalog 中**最后**出现的版本为 desired
- apply 时校验 content_hash；签名策略与 install 相同
- **live match 中**由 `DesktopHost::freeze_for_match` 拒绝变更

## 扩展工作室（无头）

```bash
cargo run -p extension-studio -- analyze path/to/skill.js --skeletons out/sk --md report.md
cargo run -p extension-studio -- grade path/to/skill.js
cargo run -p extension-studio -- publish content/standard/*.skill \
  --out dist/standard.pkg.json --key dev --store /tmp/npack
```

GUI 仍可后续挂在同一 CLI 协议上。

## CI 建议

```bash
cargo test -p conformance -p content-runtime -p diagnostics -p desktop-client
cargo run -p conformance
```
