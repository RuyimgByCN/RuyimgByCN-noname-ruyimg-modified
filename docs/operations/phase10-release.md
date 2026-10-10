# 阶段 10：桌面 / 移动发布与内容运维

## 目标

- 内容包**原子安装**、升级、回滚；中断不留下半安装 active 指针
- 扩展**撤销**（hash / package slot）
- **Content Lock** 固定对局内容集合（live match 不热更新）
- **制品保留**（build id + platform + golden hash）
- **脱敏诊断包**（无隐藏手牌 / 私有身份 / 密钥）
- 桌面 / 移动共用 **Host Bridge** 契约（Tauri 2 为首选壳）

## 组件

| 组件 | 路径 | 职责 |
|---|---|---|
| `content-runtime` | `crates/content-runtime` | 包存储、staging、active 原子写、revoke、lock |
| `diagnostics` | `crates/diagnostics` | 红acted DiagnosticPack |
| `content-cli` | `apps/content-cli` | 运维 CLI |
| `desktop-client` | `apps/desktop-client` | Tauri 2 壳 + Host Bridge |

## 存储布局

```text
{store}/
  packages/{namespace}__{package_id}/{version}/
    package.json
    signature.json? 
    meta.json
  staging/{install_id}/     # 崩溃后可 GC，不进入 active
  state/
    active.json             # 原子 tmp+rename
    history.json            # 回滚栈
    revoke.json
    artifacts.json
```

## CLI

```bash
cargo run -p content-cli -- --store /tmp/npack sign pkg.json --key secret --out sig.json
cargo run -p content-cli -- --store /tmp/npack install pkg.json --sig sig.json --key secret
cargo run -p content-cli -- --store /tmp/npack active
cargo run -p content-cli -- --store /tmp/npack rollback
cargo run -p content-cli -- --store /tmp/npack revoke-hash <hex>
cargo run -p content-cli -- --store /tmp/npack gc-staging
cargo run -p content-cli -- --store /tmp/npack retain-artifact \
  --build-id noname-next-0.1.0-m2 --platform aarch64-apple-darwin \
  --golden-hash <engine-cli-hash>
cargo run -p content-cli -- --store /tmp/npack diag-sample
```

## 验收对照

| 验收项 | 实现 |
|---|---|
| 更新中断不半安装 | staging + active 原子 rename；`gc_staging` |
| live match 不受滚动升级影响 | `DesktopHost::freeze_for_match` |
| 包依赖/签名/兼容范围可验证 | sign + engine API / VM ABI 检查 |
| 崩溃报告无隐藏信息泄露 | `diagnostics` banned patterns + hand_count only |
| 各平台同一 Golden Case | `engine-cli hash` + artifact `golden_hash` 记录 |

## Tauri 桌面构建

```bash
pnpm --dir apps/web-client desktop         # 开发窗口
pnpm --dir apps/web-client desktop:build   # 当前平台安装制品
```

桌面壳复用 Web/WASM 单机路径与 WebSocket 远程入口。Tauri capability 仅开放 core
窗口能力和本项目注册的 Host Bridge 命令；没有安装任意文件系统、Shell 或 HTTP 插件。

### M2 桌面验收记录（2026-07-13）

- 制品：`target/release/bundle/macos/无名杀 Next.app`
- 平台：macOS / aarch64，Tauri 2
- 对局：4 人、seed 42、共享 Web/WASM 标准身份局
- 结果：忠臣/主公胜利，revision 167，终局 hash 前缀 `fec37a40beafc0bf`
- UI 复核：手牌、装备、弃牌按 `CardView.zone` 分区；终局横幅与身份揭示正常

> 该终局 hash 是修正标准武将键 `xuchu` → `xuzhu` 之前的历史验收记录；键修正后的
> 当前确定性向量由 `cargo run -p conformance` 与 `scripts/cross-target-hash.sh` 维护，
> 不用本条旧 hash 作为现行门禁。

这证明当前平台壳可完成一局；Windows/Linux 安装、签名与自动更新仍属于 M7。

## 非目标（本阶段）

- Xcode / Android Studio 工程与三平台签名流水线
- 应用商店上架与代码签名证书轮换 UI
- 自动 CDN 差分更新协议（可在 store + lock 之上扩展）

## 安全提醒

签名只证明**完整性 / 来源标签**，不替代 VM 沙箱、能力清单与资源配额。
