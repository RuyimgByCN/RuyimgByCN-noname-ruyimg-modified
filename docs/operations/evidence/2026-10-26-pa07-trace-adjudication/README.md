# PA07 trace 甄别：SAB 提前解析为何不进输入后第一帧——判定为 Chromium 固有两帧下限，缓解上限已到（2026-10-26）

- 执行时间：2026-10-26 **22:51:10 → 23:03 +08:00**（`HEAD = 69eff8ef` 加未提交的驱动 `--trace` 改动）；执行者 ZCode A 道
- 工作包：[台帐 PA07](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26)；承接[缓解原型](../2026-10-26-pa07-mitigation-prototype/README.md) §3 的两个候选解释（a 输入后第一个渲染更新在下一 vsync、rAF 已过样式阶段；(b) 页面帧标记测法偏悲观）
- 方法：给 `apps/web-client/scripts/web-frame-latency.mjs` 新增 `--trace <file>`（CDP `Tracing.start` ReportEvents，类目 `devtools.timeline` + `disabled-by-default-devtools.timeline.frame` + `benchmark`，只包住驱动段），有头 Chrome **156.0.8078.48**（ProMotion 120 Hz、CDP 真实输入、同局 seed 43、4 座身份局、`--coop`），A/B 各 **40 次真实点击**（A=`--latency sab`，early=39/40；B=基线）。逐点击按帧链 `BeginMainThreadFrame → AnimationFrame::Render → FireAnimationFrame → AnimationFrame::StyleAndLayout → Commit → DrawFrame → AnimationFrame::Presentation` 取真值（[分析器](analyze-trace.py)、[A 明细](analysis.json)）。

**一句话结论**：**候选 (a) 成立，(b) 不成立（对 SAB 而言）**。每次点击都落在当前帧的 rAF **之后**（输入相位距上一 `BeginMainThreadFrame` p50 1.60 ms，而该帧 rAF 在 BeginMainFrame+0.26 ms 已执行）——点击引发的任何 DOM 变更最早只能进入**下一帧**（input→firstRAF p50 7.73 ms），该帧于 p50 **16.63 ms** 呈现 ≈ 2×8.33 ms。SAB 的 rAF 内提前解析恰好把这 16.6 ms 打满（真 i2p−页面口径 i2p = **+0.95 ms**，页面测法并不悲观）；基线的页面口径反而**低估** 7.6 ms（帧标记是下一 rAF 时间戳，不是呈现）。**判定：Chrome 端 PA07 目标（点击到绘制 p50 ≤ 1 个刷新周期 + 2 ms = 10.3 ms@120Hz）不可达，属 Chromium 输入/帧管线固有；缓解上限 = 已实现（transfer −13% 的任务排队收益），端到端不再有 rAF 路线空间；WKWebView 维持达标。**

## 1 逐点击数据（p50，n=40/组）

| 段（ms） | SAB | 基线 | 解读 |
|---|---:|---:|---|
| 输入相位（距上一 BeginMainThreadFrame） | 1.597 | 0.866 | 点击落在当前帧 rAF 之后 |
| input→nextBeginMainFrame | 7.548 | 7.570 | 当前帧早已开始，下一帧在 ~1 个刷新周期后 |
| input→firstRAF | 7.726 | 7.646 | 最早可承载 DOM 变更的 rAF 在下一帧开头 |
| RAF→该帧 StyleAndLayout | **1.929** | 0.017 | SAB 在该 rAF 内完成镜像解析+patch（基线该帧无 DOM 变更） |
| **input→该帧 Presentation（真 i2p）** | **16.631**（p95 24.317） | 24.176* | SAB ≈ 2×8.33 ms；基线 *为其首 rAF 帧的呈现，patch 在更晚的 postMessage 任务里，真实更晚 |
| 页面口径 i2p | 15.755 | 16.395 | SAB 与真值差 +0.95 ms（测法准确）；基线低估 ~7.6 ms |
| 该帧 Presentation 距上一呈现 | 17.479 | 24.909 | SAB 固定 2 帧；基线 3 帧 |

## 2 判定依据与边界

1. **为什么进不了当帧**：`FireAnimationFrame` 紧随 `BeginMainThreadFrame`（+0.26 ms）执行；点击（`EventDispatch`）在该帧 rAF 之后 ~1.3–1.6 ms 才到主线程，属帧间任务。Chromium 把点击后的首个渲染更新排在下一 `BeginMainFrame`，其呈现再下一个 vsync——**任何**点击触发的 DOM 变更（无论 rAF 提前解析还是同步 patch）都至早在第 2 个 vsync 呈现；要突破需平台级方案（如 compositor 即时反馈/本地预测反馈），超出本项「规则不迁移、不加主线程规则」的范围。
2. **SAB 已到上限**：真 i2p p50 16.63 ms 与固有 2 帧下限一致；p95 24.3 ms 为个别帧错过 commit 时限多等一帧（`DroppedFrame` 25 次/组内），非 SAB 造成。缓解原型保留为 `latency=sab` 实验开关（缺省关闭），收益如实登记为 transfer/roundtrip −12~13%，端到端无收益。
3. **测量口径勘误**：perfProbe 的 input_to_paint 以「下一 rAF 时间戳」为 paint 标记，在 SAB 档准确（+0.95 ms），在基线档低估 ~7.6 ms（真实呈现更晚）；PA08 系列引用基线 i2p 时按此口径读。
4. **未做**：Windows WebView2/Safari/Firefox 的 trace（WKWebView 已达标无需缓解；其余平台 PA08-3 已有传输面数据，i2p 固有下限论证基于 Chromium 主线行为，Safari/WebKit 不同源不外推）。完整 trace 各 19–20 MB 未入库（可由 README 命令重现），入库为过滤后事件集（A 11,138 / B 10,061 条，含全部帧链与输入事件）。

## 3 复现

```bash
cd apps/web-client
pnpm exec vite build --outDir /tmp/pa07-dist --emptyOutDir
node --import ./scripts/ts-register.mjs scripts/web-frame-latency.mjs --dist /tmp/pa07-dist \
  --out /tmp/pa07/out-sab --mode identity --players 4 --seed 43 --driver click --max-steps 40 \
  --headed --coop --latency sab --trace /tmp/pa07/trace-sab.json     # A
# 同命令去掉 --latency sab → 基线；python3 analyze-trace.py（本目录）
```

## 4 本批验证

| 命令 | 结果 |
|---|---|
| 上述 A/B 各一次 | 均整局至 rev 40（状态摘要一致 `f9aaf848…`）；A `sabDebug {seq:43, early:39}`；两轮 console_errors 0 |
| `pnpm --dir apps/web-client test` / `typecheck` | 80/0/0；vue-tsc 0 错（驱动改动只增测量旗标，不影响页面） |
| `check-source-lines.py` / `check_local_links.py --all` / `check-adr-index.py` | 通过 / 坏链 0 / 通过 |
