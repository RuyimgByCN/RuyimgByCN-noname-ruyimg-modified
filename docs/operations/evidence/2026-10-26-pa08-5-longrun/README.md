# PA08-5 ≥30 分钟长时与内存（2026-10-26）

- 日期：2026-10-26；数据文件落盘 14:08–14:33、首次提交 14:33（d77e4cd4）+08:00（原记「2026-10-26 16:20–18:20 +08:00」晚于提交或与文件时间矛盾，2026-10-26 17:30 扫描按数据文件时间与提交历史更正）
- 工作包：[台帐 PA08](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) 扩面第五切片（长时稳定性与内存）
- 源码：`HEAD = cf987ab0` 加未提交工作树（本切片新增：autodrive `loop=1`+`duration=<ms>` 连续多局、内存趋势采样、外链驱动透传与 summary 落盘——见 §3）
- 环境：同 [PA08-3](../2026-10-26-pa08-3-browsers/README.md)；Chrome 有头 120 Hz、`--coop`（跨源隔离，供内存 API）、4 座身份局 seed 42 连续多局（终局自动点「重新开局」）

**一句话结论**：**32.0 分钟连续 16.6 局**（4634 个自动步），帧节奏全程稳定（p50 8.33 / p95 9.77 / p99 10.26 ms，max 19.7 ms），掉帧率（>1.5×刷新）0.09%，**Long Task 全程 0**——长时无衰退。内存趋势见 §2（10 分钟采样档）。

## 1 32 分钟连续多局（chrome-32min）

原始数据：[runs/chrome-32min/](runs/chrome-32min/)（frames.csv 230,738 帧；interactions.csv 4,683 步）。

| 指标 | 值 |
|---|---|
| 时长 / 局数 | 1921 s / ≈16.6 局（每局终局 rev 280） |
| 帧间隔 p50 / p95 / p99 / max | 8.33 / 9.77 / 10.26 / 19.7 ms |
| >1.5×刷新周期占比 | 0.093% |
| Long Task | **0** |
| input_to_paint p50 / p95 / p99 | 12.4 / 17.0 / 18.3 ms |
| transfer p50 / p95 | 1.5 / 7.8 ms |
| roundtrip p50 / p95 | 2.1 / 8.7 ms |

注：summary.json 由原始 CSV 重算（见 §3 驱动 bug 注记）；帧合计时间 1921 s 与墙钟一致（无掉帧空洞）。

## 2 内存（chrome-memrun2，11 分钟采样档）

Chrome 156 非隔离页已移除 legacy `performance.memory`，但跨源隔离（`--coop`）下两者皆可用——周期采样用同步 legacy 读数（无强制 GC），收尾一次完整采样。每 30 s 一点，随报告带回序列（[runs/chrome-memrun2/summary.json](runs/chrome-memrun2/summary.json)）。

| 指标 | 值 |
|---|---|
| 采样点 | 23 点（t=1…639 s，覆盖 ~3.5 局） |
| usedJSHeapSize | **全程恒定 2.8 MB（23/23 点同值，零增长）** |
| 收尾快照 | used 7.4 MB / total 37.8 MB / limit 4.2 GB |
| Long Task / 帧 p99 | 0 / 10.2 ms |

（首轮 memrun 采样钩子缺陷：异步新 API 在此环境静默抛错导致 0 采样点，终值 17.4 MB 仍有效——见 runs 外的 memrun1 数据未入档；修正为 legacy 同步读数后 memrun2 完整。）**结论：长时无内存泄漏迹象。**

> 注：`usedJSHeapSize` 只覆盖主线程 JS 堆；worker 堆与 wasm 线性内存不在其中。wasm 引擎内存（实例级）随局重建释放——16.6 局循环下帧与任务指标无衰退（§1）是该路径稳定性的主证据。

## 3 本切片工具改动与 bug 注记

1. `autoDrive.driveAuto`：`loop=1`+`duration=<ms>` 支持连续多局（终局点「重新开局」）；缺省行为不变。周期内存采样（`memory_series`）与收尾 `sampleMemory()`（旧 `performance.memory` 优先，新 API 次之）。
2. 外链驱动：`--loop/--duration/--coop/--latency` 透传；summary 落盘 `run.memory`/`run.memory_series`。
3. bug 注记：32 分钟档的 driver 在收到报告后、写 summary 前因 `memory.usedJSHeapSize` 字段名写错（新采样结构字段为 `used`）抛错退出，报告原文（含内存值）随进程丢失——帧/交互 CSV 完整，summary 由 CSV 重算（公式同 perfStats nearest-rank）。字段名已修复，10 分钟档由修复后驱动完整落盘。

## 4 未覆盖

1. WKWebView/Safari/Firefox 的长时档（Chrome 已证无衰退；WebKit 家族帧节奏与单局口径见前批）；
2. 移动端（PA08-6 单独做）。

## 5 本批验证

| 命令 | 结果 |
|---|---|
| `node scripts/web-frame-external.mjs --loop 1 --duration 1920000 --coop`（chrome） | 32.0 min / 16.6 局正常循环，Long Task 0 |
| `pnpm --dir apps/web-client test` / `vue-tsc --noEmit` | 80 passed / 0 failed；0 错误 |
