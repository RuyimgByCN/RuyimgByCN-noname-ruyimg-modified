# ADR 0702：M7 移动端阻塞说明与替代方案

> **历史 ADR**：原 M7 移动壳决策已被 [性能架构](../../noname-performance-architecture.md) 的原生平台路线取代；保留当时阻塞原因。配套阶段快照已清理，见 [旧实现参考](../architecture/legacy-reference.md)。
> 状态：**移动端书面 ADR 说明阻塞与替代方案**（M7 验收门第 2 项）。

## 阻塞点

1. **渲染器适配**：战斗渲染器（`packages/renderer-runtime`）面向桌面 DOM/Web 布局，
   移动端触控热区、竖屏/横屏布局与性能预算（§38 设备档位）未做适配测量。
2. **壳可行性**：当时的阶段 28 快照登记了 Tauri 2 移动壳（iOS/Android）的
   可行性方向，但 WebView/WASM 单机路径在移动端的内存预算（当前标准包 ~14MB 内容 +
   WASM 运行期）未做真机验证。
3. **输入与延迟**：移动端断线恢复 / 观战 / Choice 重连（M2 已验证的桌面路径）
   在移动网络下的超时策略未测。

## 替代方案（plan B）

- **首选**：Tauri 2 移动壳复用现有 Web/WASM 单机路径 + WebSocket 远程入口（与桌面壳同构），
  用同一 `cross-target-hash` 门禁保证规则一致性。
- **备选**：纯 Web 响应式壳（PWA）作为移动端入口，先行验证渲染器移动适配，
  再决定是否引入 Tauri 原生壳。

## 验收路径（保留在计划中）

- 在 iOS Simulator / Android Emulator 跑通标准身份局 headless + 一轮 UI 交互。
- 记录移动端内存/启动 P50，纳入 §38 多档设备预算。
- 更新 M7 验收门：移动端可运行或书面 ADR（本文档即书面依据）。
