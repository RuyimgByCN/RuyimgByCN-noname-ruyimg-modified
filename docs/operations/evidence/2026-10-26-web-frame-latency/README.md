# 表现层帧间隔与输入延迟基线：headless Chrome，4／8 座身份局整局（2026-10-26）

- 日期：2026-10-26 00:05–00:17 +08:00 最终重跑（[run.log](run.log)）；2026-10-25 23:49 起的首轮与之结论一致，未入本目录
- 工作包：T13 / NC09 / NC12 的**表现层测量切片**；只测量，不优化；规则 **v322**、存档 **v105** 未变
- 源码：`HEAD = 7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；本批文件摘要见 [source-hashes.txt](source-hashes.txt)
- 后续任务登记：[剩余台帐 PA07–PA11](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)

**一句话结论**：同一局里，Worker 内每步处理 p50 只有 0.4–0.6 ms，Vue 刷新 DOM 约 2 ms；但**真实鼠标点击之后**，Worker 回包要约 16 ms（一帧）才送到主线程，点击到绘制 p50 约 33 ms（60 Hz 下两帧）。同一局改由页面计时器推进（没有输入事件）时，回包只要约 1 ms，发送到绘制约 15 ms。帧间隔在本环境无掉帧，整局没有 Long Task，控制台零错误。

> headless Chrome、本机静态服务、60 Hz 合成节奏；**不是**真实显示器呈现，也不是 Tauri WebView。数字只用于定位表现层各段的量级，不得当作产品 SLA，也不与旧线或内核基准相除。

## 1 设备与工具链

| 项 | 值 |
|---|---|
| 机型 / 系统 | Mac15,10（Apple M3 Max，14 核，36 GiB）/ macOS 27.0.1（26A434） |
| 浏览器 | Google Chrome 156.0.8078.48，`--headless=new --no-sandbox`，窗口 1280×800（视口 1280×657，DPR 1）；默认 `--disable-gpu`，GPU 档去掉该参数 |
| 前端构建 | Node v26.4.0，vite 6.4.3；以 [vite.perf.config.mjs](vite.perf.config.mjs) 复用仓库 `vite.config.ts`，输出到 scratchpad（**未覆盖**仓库内被跟踪的 `apps/web-client/dist/`） |
| 原生 WASM | rustc 1.95.0 + wasm-pack 0.15.0，`wasm-pack build crates/engine-native-wasm --target web --release --no-opt`，输出到 scratchpad；仓库内 `src/worker/native-pkg/` 产物早于 `ai.rs` 最新改动，未使用也未覆盖 |
| 并发负载 | load average 8.7–12.2（同机其他 agent） |

## 2 本批代码（默认关闭，只做测量）

| 文件 | 作用 |
|---|---|
| `apps/web-client/src/engine/perfProbe.ts`（新） | 主线程探针：rAF 帧时间戳、Long Task、Event Timing（≥16 ms）、逐请求分段；结果挂 `window.__nonamePerf` |
| `apps/web-client/src/engine/perfStats.ts`（新） | 纯统计（nearest-rank 分位，与 NC12 口径一致）；探针与驱动共用 |
| `apps/web-client/src/worker/stepTimings.ts`（新） | Worker 侧分段；开启时在回包之后追加 `{type:"step_timings"}` |
| `apps/web-client/scripts/web-frame-latency.mjs`（新） | headless 驱动：CDP 派发真实鼠标事件，导出 CSV 与 summary |
| `apps/web-client/scripts/perf-probe.test.mjs`（新） | 统计与 Worker 分段的单测（4 项） |
| `src/engine/client.ts`、`src/types/engine.ts`、`src/engine/viewStore.ts`、`src/worker/engine.native.worker.ts`、`src/worker/nativeBackend.ts` | 每处 1–15 行挂钩：发送/回包/分段消息、init 的 `perf` 开关、应用信封前后、信封合成内部分段点 |

开启方式：战斗页 URL 带 `perf=1`，或预置 `localStorage["noname-next-perf"]="1"`。关闭时各入口立即返回，Worker 回包与原先完全相同（单测覆盖）。

首版把「应用信封」挂钩写在 `BattleView.vue` 的 `handleEnvelope` 中，导致 `scripts/network-client.test.mjs`（该测试抽取 `handleEnvelope` 源码在只注入既定变量的环境里执行）报 `release is not a function`；已改为挂在 `viewStore.applyEnvelope` 内部，`BattleView.vue` 不含本批改动。本目录全部数据来自改正后的源码。

## 3 测量口径

驱动命令（在 `apps/web-client` 下）：

```bash
node --import ./scripts/ts-register.mjs scripts/web-frame-latency.mjs \
  --dist <构建目录> --out <输出目录> --mode identity --max-steps 1500 \
  --players 4|8 --seed 42|43 --driver click|auto [--gpu]
```

- **click**：每步用 CDP `Input.dispatchMouseEvent` 真实点击「自动一步」，等按钮重新可用（MutationObserver）后再等 50 ms 进入下一步。
- **auto（无输入对照）**：真实点击一次「连续自动」，之后由页面计时器推进；这些请求没有关联输入事件，分段从「发送」起算。
- 开局后先空闲采样 3 s 帧间隔，再重置样本开始驱动；动画与 120 ms 表现节流保持默认开启。

逐请求分段（ms）：

| 字段 | 含义 |
|---|---|
| `input_to_send` | 输入事件 `timeStamp` → `postMessage` 前 |
| `roundtrip` | 发送 → 主线程 `onmessage` |
| `worker_total` | Worker 收到请求（就绪后）→ 回包 `postMessage` 前；`dispatch`（引擎命令）+ `view_json` + `view_parse` + `adapt`（视图适配/选择/合法动作/hash/公开事件） |
| `post` | 回包 `postMessage` 调用本身 |
| `transfer` | Worker 发出 → 主线程收到（按 `performance.timeOrigin + now()` 跨线程对齐，含序列化、排队与反序列化） |
| `apply` / `vue_patch` | 应用信封到 reactive store / `nextTick` 完成 DOM 刷新 |
| `patch_to_frame` / `frame_to_paint` | DOM 刷新 → 下一个 rAF 回调 → 该 rAF 之后第一个宏任务（近似该帧绘制完成，**不是**像素呈现） |
| `input_to_paint` | 输入（无输入时为发送）→ 上一项：端到端近似 |

## 4 结果

原始数据：`runs/<档位>/` 下的 `interactions.csv`（逐请求）、`frames.csv`、`idle-frames.csv`、`long-tasks.csv`、`event-timings.csv` 与 `summary.json`。

### 表 A 整局概况

| 档位 | 驱动 | GPU | 步数 | 终局 | 帧间隔 p50 / p99 / max | >1.5×刷新周期 | Long Task | 控制台错误 |
|---|---|---|---:|---|---|---:|---:|---:|
| `p4-click` 4 座 seed 42 | click | 关 | 280 | 反贼胜 · rev 280 | 16.7 / 16.8 / 33.4 ms | 0.03% | 0 | 0 |
| `p4-auto` 4 座 seed 42 | auto | 关 | 279 | 反贼胜 · rev 280 | 16.7 / 16.8 / 33.5 ms | 0.03% | 0 | 0 |
| `p8-click` 8 座 seed 42 | click | 关 | 568 | 反贼胜 · rev 568 | 16.7 / 16.8 / 33.4 ms | 0.01% | 0 | 0 |
| `p8-auto` 8 座 seed 42 | auto | 关 | 567 | 反贼胜 · rev 568 | 16.7 / 16.8 / 33.3 ms | 0.01% | 0 | 0 |
| `p4-click-gpu` 4 座 seed 42 | click | 开 | 280 | 反贼胜 · rev 280 | 16.7 / 16.8 / 16.8 ms | 0% | 0 | 0 |
| `p4-click-s43` 4 座 seed 43 | click | 关 | 145 | 反贼胜 · rev 145 | 16.7 / 16.8 / 33.3 ms | 0.06% | 0 | 0 |
| `p4-auto-s43` 4 座 seed 43 | auto | 关 | 144 | 反贼胜 · rev 145 | 16.7 / 16.8 / 33.3 ms | 0.03% | 0 | 0 |

同种子的 click 与 auto 走到同一终局与修订号（auto 少 1 步是首个请求来自点击「连续自动」，不计入无输入路径）。空闲 3 s 帧间隔各档均为 p50 16.7、max 16.8 ms。rAF 时间戳在非跨源隔离页面按 0.1 ms 取整，60 Hz 帧记为 16.7–16.8 ms，因此 `summary.json` 中「>8.33 ms」「>16.67 ms」两个绝对阈值比例在本环境分别是 100% 与约 62%，**只反映刷新率与取整，不代表掉帧**；本环境以「>1.5×刷新周期」判断掉帧。

### 表 B 逐请求分段（主路径：click 档为点击触发的「自动一步」，auto 档为计时器触发的「自动一步」；p50 / p95 / p99 / max，ms；字符数为 UTF-16 码元）

| 分段 | p4-click | p4-auto | p8-click | p8-auto | p4-click-gpu |
|---|---|---|---|---|---|
| `input_to_send` | 0.2 / 0.4 / 0.5 / 0.6 | — | 0.2 / 0.4 / 0.5 / 0.8 | — | 0.2 / 0.4 / 0.4 / 0.5 |
| `roundtrip` | 16.4 / 18.1 / 18.6 / 48.1 | 2.1 / 7.1 / 9.1 / 9.6 | 16.4 / 17.6 / 18.7 / 49.5 | 2.1 / 5.1 / 9.6 / 11.4 | 16.7 / 17.7 / 18.2 / 38.7 |
| `worker_total` | 0.4 / 1.1 / 2.9 / 7.0 | 0.6 / 1.7 / 2.6 / 3.2 | 0.5 / 1.1 / 2.8 / 5.5 | 0.6 / 1.4 / 3.3 / 5.9 | 0.4 / 1.1 / 2.8 / 3.4 |
| ├ `dispatch` | 0.2 / 0.5 / 1.2 / 5.7 | 0.3 / 1.1 / 1.9 / 2.5 | 0.3 / 0.7 / 1.1 / 4.8 | 0.3 / 0.9 / 1.5 / 5.6 | 0.2 / 0.6 / 1.5 / 3.0 |
| ├ `view_json` | 0 / 0.1 / 0.2 / 0.2 | 0.1 / 0.2 / 0.4 / 1.0 | 0.1 / 0.2 / 0.2 / 0.4 | 0.1 / 0.2 / 0.3 / 0.7 | 0 / 0.1 / 0.2 / 0.8 |
| ├ `view_parse` | 0 / 0.1 / 0.1 / 0.1 | 0 / 0.1 / 0.1 / 0.4 | 0 / 0.1 / 0.1 / 0.2 | 0 / 0.1 / 0.1 / 1.7 | 0 / 0.1 / 0.1 / 0.3 |
| └ `adapt` | 0.1 / 0.5 / 2.2 / 6.6 | 0.1 / 0.7 / 2.0 / 3.0 | 0.1 / 0.3 / 2.1 / 4.4 | 0.1 / 0.4 / 2.3 / 3.9 | 0.1 / 0.4 / 1.4 / 3.3 |
| `post` | 0 / 0.1 / 0.1 / 0.2 | 0 / 0.1 / 0.2 / 0.5 | 0 / 0.1 / 0.1 / 0.2 | 0 / 0.1 / 0.2 / 2.4 | 0 / 0.1 / 0.1 / 0.1 |
| `transfer` | **15.8** / 17.7 / 18.2 / 41.7 | **1.0** / 6.1 / 7.9 / 8.9 | **15.7** / 17.0 / 17.9 / 43.7 | **1.1** / 4.0 / 8.2 / 8.9 | **16.1** / 17.2 / 17.6 / 35.7 |
| `apply` | 0.3 / 0.4 / 0.5 / 0.5 | 0.3 / 0.5 / 0.6 / 0.7 | 0.3 / 0.5 / 0.5 / 0.6 | 0.3 / 0.5 / 0.6 / 0.6 | 0.2 / 0.4 / 0.5 / 0.5 |
| `vue_patch` | 2.1 / 2.7 / 4.3 / 19.5 | 1.4 / 2.1 / 2.3 / 2.4 | 2.4 / 3.1 / 3.7 / 24.1 | 1.6 / 2.5 / 2.8 / 3.5 | 1.9 / 2.6 / 3.5 / 17.4 |
| `patch_to_frame` | 13.1 / 15.3 / 16.3 / 16.8 | 6.7 / 13.8 / 14.7 / 15.0 | 12.8 / 14.7 / 15.5 / 16.5 | 6.2 / 13.5 / 14.9 / 15.9 | 13.5 / 15.0 / 15.6 / 15.9 |
| `frame_to_paint` | 1.1 / 1.7 / 2.2 / 2.2 | 4.8 / 8.3 / 9.1 / 35.6 | 1.2 / 1.9 / 2.5 / 3.4 | 4.9 / 9.1 / 10.2 / 24.7 | 1.0 / 1.7 / 2.6 / 3.0 |
| `input_to_paint` | **33.1** / 35.0 / 36.0 / 64.5 | **15.8** / 22.3 / 24.1 / 48.2 | **33.3** / 34.8 / 35.9 / 64.7 | **15.3** / 22.3 / 25.7 / 40.0 | **33.5** / 34.8 / 36.0 / 44.0 |
| `view_json` 字符 | 946 / 1073 / 1083 / 1102 | 946 / 1075 / 1083 / 1102 | 1618 / 1760 / 1783 / 1791 | 1619 / 1760 / 1783 / 1791 | 946 / 1073 / 1083 / 1102 |
| 信封字符 | 1253 / 1629 / 1982 / 2132 | 1253 / 1629 / 1982 / 2132 | 1760 / 2028 / 2527 / 2681 | 1760 / 2028 / 2527 / 2681 | 1253 / 1629 / 1982 / 2132 |

稳定性（seed 43，4 座）：click `transfer` 16.0 / 17.3 / 18.7 / 36.7、`input_to_paint` 33.4 / 34.7 / 51.4 / 60.0；auto `transfer` 1.0 / 4.8 / 6.1 / 6.4、`input_to_paint` 15.0 / 21.9 / 26.7 / 44.0，与 seed 42 一致。

Event Timing：click 档每次点击产生多条 ≥16 ms 的输入事件条目（p50 24 ms、max 40–56 ms；浏览器按 8 ms 取整），auto 档仅 12–29 条。

## 5 观察（待在目标环境确认，不是结论）

1. **Worker 内计算与 JSON 不是当前瓶颈**：8 座整局 `worker_total` p50 0.5 ms，视图 JSON 约 1.6 K 字符、解析 ≤0.2 ms（p99）。
2. **点击后回包推迟约一帧**：click 与 auto 的 `worker_total`、`post` 相同，差异全在 `transfer`（≈16 ms 对 ≈1 ms），GPU 开关不改变该现象。表现与 Chrome 在离散输入后优先安排下一帧渲染的调度策略一致，但**尚未在有头 Chrome、Tauri WebView（macOS WKWebView / Windows WebView2）、Safari、Firefox 验证**，原因未定论。
3. **端到端约两帧**：点击 → 回包（约一帧）→ DOM 刷新（约 2 ms）→ 等下一帧（约 13 ms）→ 绘制。若回包能在点击当帧送达，理论上可省约一帧；需按 PA07 实测。
4. 帧节奏：本环境 60 Hz、空闲与驱动期间均无明显掉帧，也没有 Long Task；无法据此判断 120 Hz 目标（性能架构 §11）。

## 6 未覆盖（不得外推）

1. 真实显示器呈现时间、GPU 合成、120 Hz 或可变刷新率显示器；
2. 有头 Chrome、Tauri 桌面 WebView、Safari、Firefox、Android/iOS；
3. 人工交互路径：出牌、选择目标、应答/取消、选将（本批只测「自动一步」与「连续自动」）；
4. 大扩展包、满手牌、长日志等较大视图；≥30 分钟长时与内存曲线；
5. 驱动扰动：每步一次 CDP `Runtime.evaluate` 与 MutationObserver 回调在主线程执行，量级很小但未单独扣除；`transfer` 依赖 Worker 与主线程 `timeOrigin` 对齐，存在亚毫秒级误差；
6. 旧线前端同口径对照。

## 7 本批验证

| 命令（`apps/web-client` 下，除注明外） | 结果 |
|---|---|
| `node --import ./scripts/ts-register.mjs --test scripts/perf-probe.test.mjs` | 4/0/0 |
| `pnpm test` | 首次有 1 项失败（上文 `network-client.test.mjs`，挂钩位置引起）；改正后 **80/0/0** |
| `pnpm typecheck` | 通过 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`（仓库根） | OK |
| `python3 -B scripts/check-source-lines.py`（仓库根） | 8608 个文件 ≤ 2000 行，通过 |
