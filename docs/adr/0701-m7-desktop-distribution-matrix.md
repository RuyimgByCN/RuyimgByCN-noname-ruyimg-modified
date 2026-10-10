# ADR 0701：M7 桌面分发矩阵——Windows/Linux 状态与延期理由

> 对应架构计划 §51（M7 多端与性能）。状态：**macOS/aarch64 已验证；Windows/Linux
> 分发 ADR 延期**（延期项保留在计划中并写明原因，符合 M7 验收门「可运行或书面 ADR」）。

## 状态

| 平台 | 壳 | 状态 | 依据 |
|---|---|---|---|
| macOS / aarch64 | Tauri 2 桌面壳 + Host Bridge | ✅ 已构建并运行标准局至终局 | `apps/desktop-client`，共享 Web UI 在 macOS `.app` 内跑到 revision 167 终局 |
| Windows x64 | Tauri 2 | ADR 延期 | 见下方阻塞 |
| Linux x64 | Tauri 2 | ADR 延期 | 见下方阻塞 |

## Windows / Linux 延期理由

1. **CI 构建矩阵未配置**：仓库的 `cargo test --workspace` 与 `cross-target-hash.sh`
   门禁覆盖 Native(macOS)/WASM 双后端，但尚未配置 GitHub Actions 的
   windows-latest / ubuntu-latest 构建与签名步骤。
2. **桌面壳分发未纳入本轮验收范围**：M7 验收门要求「三桌面平台可安装运行标准局」，
   当前 macOS 单平台已验证；Windows/Linux 构建与安装包（NSIS/AppImage）脚本未落地。
3. **确定性哈希依赖的浮点/平台差异已在架构层消除**（Native/WASM 哈希门禁跨平台一致），
   因此跨平台构建没有规则引擎层面的阻塞，仅有打包/CI 工程层面的待办。

## 后续（保留在计划中）

- 配置 3 平台 CI 构建矩阵 + 安装包产出（Tauri 2 `tauri build` 原生支持）。
- 在 Windows/Linux 上复跑 `cross-target-hash.sh` 验证规则哈希与 macOS 一致。
- 更新 M7 验收门：三平台安装运行 + 性能基线重测。
