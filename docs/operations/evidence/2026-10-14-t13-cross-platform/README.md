# T13 跨平台运行核验——wasm 确定性与实例一致性（2026-10-14）

## 任务来源与口径

[进度主稿 T13](../../../architecture/native-progress.md) 余项「客户端 UI 与跨平台运行」的可执行切片：**wasm 目标的跨实例确定性**——同 seed 同命令序列在两次独立实例化后产出相同状态哈希（跨平台可重放的核心门），加 seed 敏感负例与多 seed 批量核对。

环境：wasm-pack 0.15.0 构建（O4 同款 --no-opt），Node v26.4.0，`loadEmbeddedStandard` 内嵌标准（O4 内容债①修复后路径）。

## 结果（[wasm-determinism.txt](wasm-determinism.txt)）

| 断言 | 结果 |
|---|---|
| 双实例确定性（seed 777、120 步） | **hash_match=true**（c56f51e7…） |
| seed 敏感负例（777 vs 778） | hash_differs=true ✓ |
| 多 seed 批量（5000–5004 × 双实例） | **全一致** |

## 登记边界

- wasm 面未暴露 save/restore 存档 API（仅 loadReplay/seekReplay 录像面）——存档跨实例接力在 wasm 面经录像承载；原生侧存档跨进程已由 T8 增量 7（自启子进程接力）覆盖。
- 浏览器端到端 UI 回归需真实浏览器 harness，仍归产品面批。
- wasm-opt 未安装，opt 配置对比待工具可用。
