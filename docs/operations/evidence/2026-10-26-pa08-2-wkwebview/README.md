# PA08-2 Tauri 桌面 WKWebView 基线（2026-10-26）

- 日期：2026-10-26 09:20–11:20 +08:00
- 工作包：[台帐 PA08](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) 扩面第二切片（Tauri 桌面）；同时给出 [PA07](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) 在 WKWebView 档的复现判定
- 源码：`HEAD = 32a40d90` 加未提交工作树；测量栈＝[headless 基线](../2026-10-26-web-frame-latency/README.md)同款（perfProbe/perfStats/stepTimings，`perf=1` URL 门控），外链驱动 `apps/web-client/scripts/web-frame-external.mjs`（本批入库）
- 环境：Mac15,10（M3 Max，内置 ProMotion 显示器）；Tauri 2 debug 构建 `noname-next-desktop`（`--features tauri-shell`，devUrl 模式加载本机静态服务），WKWebView 窗口 1280×820；负载与 Chrome 基线同构（4 座身份局、seed 42、steps 1500 上限）

**一句话结论**：WKWebView 固定 **60 Hz**（ProMotion 机器上仍 16.66 ms），帧节奏健康（p99 ≤ 33 ms、Long Task 0）；真实 HID 点击（cliclick）下 **transfer p50 仅 1.8–2.1 ms——不存在 Chrome 的输入后任务推迟**，`input_to_paint` p50 10.7–16.2 ms ≤ 60 Hz 目标 18.7 ms（1 刷新周期 + 2 ms）。**PA07 问题在 WKWebView 档不存在、目标已达标，无需缓解**；auto 档整局终局 rev 280 与 Chrome/headless 同 seed 完全一致。

## 1 原始数据

[runs/](runs/)：每目录 summary.json / frames.csv / interactions.csv / event-timings.csv / long-tasks.csv。驱动日志 RESULT 行已摘入下表。

| run（driver） | 帧间隔 p50 / p95 / p99 / max | >1.5×刷新 | transfer p50 | input_to_paint p50 | 样本 n |
|---|---:|---:|---:|---:|---:|
| tauri-p4-auto（页面自动） | 16.66 / 18.72 / 21.2 / 51.8 ms | 0.47% | —（无真实输入） | — | 6856 帧 |
| tauri-p4-manual（真实点击） | 16.68 / 18.10 / 19.4 / 37.7 ms | — | 1.80 ms | 12.38 ms | 2 |
| tauri-p4-manual2（真实点击） | 16.66 / 17.60 / 33.2 / 50.0 ms | — | 2.14 ms | 16.18 ms | 6 |
| tauri-p4-manual3（零样本†） | 16.66 / 17.62 / 33.3 / 49.9 ms | — | — | — | 0 |
| tauri-p4-manual4（真实点击） | 16.66 / 17.52 / 19.9 / 58.0 ms | — | 1.92 ms | 10.74 ms | 6 |

- roundtrip（点击→回包应用）p50 2.9–3.2 ms；Long Task 全部会话 0。
- 真实点击合计 **n = 14**：transfer p50 1.8–2.1 ms、input_to_paint p50 10.7–16.2 ms、p95 ≤ 22.6 ms（个别点击错过一次 vsync，p50 口径稳定达标）。
- † manual3 为坐标校准陷阱会话（见 §3），计入诚实记录不计样本。

## 2 与 Chrome 基线对照（PA07 判定）

| 指标（click/manual 档 p50） | Chrome headless 60 Hz | Chrome 有头 120 Hz | WKWebView 60 Hz（本批） |
|---|---:|---:|---:|
| worker transfer | 15.8 ms ≈ 1 帧 | 7.6–7.8 ms ≈ 1 帧 | **1.8–2.1 ms ≈ 1/8 帧** |
| input_to_paint | 33.1 ms ≈ 2 帧 | 16.4 ms ≈ 2 帧 | **10.7–16.2 ms ≤ 1 帧** |

- Chrome 的「输入后任务窗口把 worker 回包推迟到下一帧合成之后」在 WKWebView **不复现**：真实点击后回包几乎立即到达主线程。
- 60 Hz 目标＝16.67 + 2 = 18.7 ms：manual 各会话 p50（12.4 / 16.2 / 10.7 ms）全部达标；p95 22.6 ms 超出 4 ms（错过一次 vsync），按 PA07 验收口径（p50）判达标。
- 结论：PA07 缓解（rAF 轮询 SAB）只针对 Chromium 系；WKWebView/原生 WebKit 档无需改动。

## 3 方法注记（真实点击采集）

- 真实 HID 点击用 `cliclick`（CGEventPost）；窗口截图坐标 → 屏幕坐标按 `窗口原点(116,49) + 截图像素 / 1.6`（Retina dpr 2 下 1280 pt 窗口呈 2048 px 截图，实测比例 1.6）。
- **陷阱（manual3 零样本根因）**：kimi-cu AX 快照返回的按钮 bbox 是**陈旧 diff 树坐标**（给出 @277,127，视觉真值在 @509,225 附近）；按 AX 坐标换算的点击全部落空。修正方法＝窗口截图 → 视觉定位按钮中心 → 换算屏幕坐标；校准后同一坐标 12 次点击全部落点（期间按钮位置稳定，仅 ±2 px 抖动）。
- 每次点击间隔 ~3 s：自动步结算期间界面浮层会吞掉部分点击（manual4 12 次点击 6 次计入步进），样本按 autoDrive 实际计数为准。
- auto 档整局（1500 步上限内终局）final `rev 280 反贼胜`，与 [Chrome 有头基线](../2026-10-26-pa08-headed-baseline/README.md)同 seed 完全一致——跨引擎宿主确定性不变。

## 4 未覆盖

1. Windows WebView2 / Linux WebKitGTK（本机无环境，挂待办）；
2. WKWebView 120 Hz 探索（当前 Tauri/WKWebView 固定 60 Hz，无法产生 120 Hz 负载）；
3. 大扩展包/满手牌（PA08-4 统一做）、≥30 分钟长时（PA08-5 统一做）。

## 5 本批验证

| 命令 | 结果 |
|---|---|
| `node scripts/web-frame-external.mjs --app tauri --binary …/noname-next-desktop` ×5 会话 | auto 终局 + manual×4（n=14），console error 0，Long Task 0 |
| `pnpm --dir apps/web-client test` | 80 passed / 0 failed（含 perf-probe.test.mjs） |
| `pnpm --dir apps/web-client exec tsc --noEmit`（typecheck） | 通过 |
| `python3 -B scripts/check-source-lines.py` | 通过 |
