# 第 252 轮：跨 crate 验证汇总（2026-10-19）

- 日期：2026-10-19
- 规则版本 v322（工作树）/ 存档格式 v105
- 目的：在 NC08 标准面采样收敛后，做一次**跨 crate 的全量回归核对**，为后续接手提供基线数字

## 1 命令与结果

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | **1367 passed / 0 failed / 6 ignored** |
| `cargo test --offline -p engine-native-wasm` | **33 passed / 0 failed / 0 ignored** |
| `cargo test --offline -p game-server` | **96 passed / 0 failed / 0 ignored** |
| 客户端 node 测试（`apps/web-client/scripts/*.test.mjs`，**按退出码**） | **6 个文件全部通过 / 0 失败** |
| `python3 -B scripts/check-source-lines.py` | **8350 files ≤ 2000 lines** |

**说明**：
- 6 个 `ignored` 为**如实登记**的未验收项（结姻/裸衣/突袭/连弩等历史项 + 激将代打），
  各自在证据目录中留有诊断与待办，**不构成通过**；
- `npx vitest run` 在本机因 **npm 缓存目录权限**（`~/.npm` 含 root 属主文件）**无法运行** ✗
  → 改用**等价的 node 脚本测试**（`node scripts/*.test.mjs`）按**退出码**核对 ✓；
  该环境问题**与本仓库代码无关**（登记备查）。

## 2 本轮未做的事（避免误读）

- **未**修改任何生产源码（受并行拆分的 135 个未跟踪文件约束，见修复清单）；
- **未**新增验收项；本轮只是**核对既有数字**；
- 未验证：移动/桌面档位、真实网络、产品整机长程、未转换来源（14 个）。

## 3 与主稿的关系

数字与[进度主稿](../../../architecture/native-progress.md)第 250 轮里程碑一致；差异项与修复项以
[修复清单](../2026-10-18-blocked-fixes/README.md)（**28 项**）为准。
