# PA07 缓解原型：SAB 镜像 + rAF 提前应用（2026-10-26）

- 日期：2026-10-26 20:00–21:30 +08:00
- 工作包：[台帐 PA07](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)
- 源码：`HEAD = 08466f60` 加未提交工作树。原型实现（`latency=sab` URL 门控，缺省零影响）：
  - `stepTimings.mirrorToSab`（worker）：回包 JSON 镜像写入主线程创建的 SharedArrayBuffer（[8..] 数据、[0] 序号、[4] 长度；seq 最后发布）；
  - `client.initSabMirror`（主线程）：创建 SAB 经 init 消息共享；rAF 轮询序号，命中即拷贝→解码→解析→提前 resolve 同一请求；请求在途挂 keepalive 动画（布局属性）保证空闲页也有 rAF；
  - 规则不迁移：只把 postMessage 会送的同一份信封提前反序列化；postMessage 路径兜底（pending 已删则跳过）。
- 环境：Chrome 156 有头 120 Hz（ProMotion）、CDP 真实输入（`--driver click`）、`--coop`（SAB 需跨源隔离）、4 座身份局 seed 43。

**一句话结论**：**机制完全打通**（early=145/145——每个回包都在 rAF 内提前解析；`transfer` p50 7.88→6.83 ms，-13%），但**端到端 input_to_paint 未改善**（16.35→16.28 ms，p50 目标 ≤10.3 ms@120Hz 未达）：提前解析省下的时间被 patch→frame 段（6.22→7.14 ms）等量吸收——提前的 DOM 变更仍未在输入后的第一个渲染更新内出帧，或帧标记测法偏悲观（见 §3 待查）。

## 1 A/B 数据（同负载同机同刷新率）

[runs/](runs/)（interactions.csv 为逐点击明细；frames.csv 体积原因未入档，summary 保留）。

| 指标（p50，n=145 真实点击） | 基线（无 SAB） | SAB 原型 | 变化 |
|---|---:|---:|---|
| transfer（worker 发出→主线程收到） | 7.88 ms | **6.83 ms** | **-13%** |
| roundtrip | 8.16 ms | 7.2 ms | -12% |
| patch_to_frame | 6.22 ms | 7.14 ms | +15%（吸收项） |
| frame_to_paint | 0.62 ms | 0.51 ms | ≈ |
| **input_to_paint（端到端）** | **16.35 ms** | **16.28 ms** | **无改善** |

诊断计数（sabDebug）：seq=148、early=145（145 个驱动点击全部提前解析）、parseErr=0。

## 2 本轮修掉的两个硬坑（对后续任何 SAB 方案有效）

1. **`TextDecoder.decode` 拒绝 SharedArrayBuffer 视图**（`The provided ArrayBufferView value must not be shared`）——首轮原型所有镜像解析静默失败（parseErr 逐次递增、early=0）。必须先拷贝到普通 `Uint8Array` 再解码。
2. **空闲页没有 rAF**：轮询依赖渲染机会；对局静置时（无动画）点击后无新帧 → rAF 沉默 → 推迟的回包任务先到（实测 seq 增长而 early=0）。对策：请求在途由输入处理器（不被推迟）挂 8ms 主线程 keepalive 动画（`margin-left` 布局属性——纯合成器动画不触发主线程 rAF）。

## 3 未决问题（下一步）

1. **端到端为何不吃提前解析的收益**：rAF 内 resolve→微任务 patch→当帧提交的理论链路未兑现（patch→frame 反而 +1ms）。两个候选解释待甄别：
   a. Chromium 输入后第一个渲染更新发生在下一 vsync，其中 rAF 已过样式阶段？→ 需 DevTools trace 证实补丁是否进入当帧；
   b. perfProbe 的帧标记（下一 rAF 时间戳）测法对当帧提交偏悲观 → 需对拍 `performance.measure`/trace 事件。
2. 若 a 成立，「rAF 轮询」路线对端到端 i2p 的上限受限，PA07 或需改判为「Chromium 行为固有、缓解仅省任务排队」；WKWebView/WebKit 无此问题（已达标，见 [PA08-2](../2026-10-26-pa08-2-wkwebview/README.md)）。
3. Firefox/Safari 真实点击复现数据仍缺（PA08-3 §4）。

## 4 本批验证

| 命令 | 结果 |
|---|---|
| `web-frame-latency.mjs --driver click --headed --coop [--latency sab]` ×2 | 均整局终局 rev 145（确定性一致）；sabDebug early=145/145 |
| 诊断页（MCP 可信点击）单点 | transfer 6.18→2.95 ms（修复 TextDecoder 坑前后） |
| `pnpm --dir apps/web-client test` / `vue-tsc --noEmit` | 80 passed / 0 failed；0 错误 |
