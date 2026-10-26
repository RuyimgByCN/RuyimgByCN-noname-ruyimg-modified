# PA08-1 有头 Chrome 基线与 PA07 有头复现（2026-10-26）

- 日期：2026-10-26 06:30–07:40 +08:00
- 工作包：[台帐 PA08](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) 扩面第一切片（有头 Chrome）＋ [PA07](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) 的有头复现与缓解判别
- 源码：`HEAD = 7e28a937` 加未提交工作树；本批代码改动仅 `apps/web-client/scripts/web-frame-latency.mjs`（新增 `--headed` 旗标）；前端测量栈按[帧间隔基线](../2026-10-26-web-frame-latency/README.md)同法重建（wasm-pack 重建 WASM → scratchpad vite 构建，未覆盖仓库 dist）
- 环境：Mac15,10（M3 Max）内置 **ProMotion 显示器**；有头 Chrome 156.0.8078.48 窗口 1280×800、GPU 默认开启；负载与 headless 基线完全同构（同 seed、同驱动、同 max-steps 1500）

**一句话结论**：有头 120 Hz 下（帧 p50 **8.3 ms**），「真实点击后回包推迟约一刷新周期」**复现**：click 档 `transfer` p50 **7.6–7.8 ms ≈ 8.33 ms 一帧**（auto 档仅 1.5–1.7 ms），`input_to_paint` p50 **16.4–16.5 ms ≈ 两帧**——与 headless 60 Hz 的现象（16 ms/33 ms）完全同构，且**随刷新率缩放**。微基准进一步证明该推迟是**输入后任务窗口对所有主线程任务的一视同仁**（worker postMessage ≡ MessageChannel），任何任务型投递通道都无法突破；唯一非任务出口是 **rAF 帧回调内轮询 SAB**（缓解原型方向）。

## 1 有头 Chrome 整局（同 headless 基线负载）

原始数据：[runs/](runs/)（每档 summary.json / frames.csv / interactions.csv）。终局与 headless 一致（4 座 rev 280、8 座 rev 568 反贼胜）。

| 档位 | 帧间隔 p50 / p99 / max | >1.5×刷新周期 | 主路径 transfer p50 | input_to_paint p50 | vue_patch p50 |
|---|---|---:|---:|---:|---:|
| p4-click-headed | **8.3 / 10.2 / 208 ms**（120 Hz） | 0.05% | **7.6 ms** | **16.4 ms** | 1.3 ms |
| p4-auto-headed | 8.3 / 10.2 / 184 ms | 0.02% | 1.5 ms | 12.9 ms | 1.6 ms |
| p8-click-headed | 8.3 / 10.2 / 24 ms | 0.01% | **7.8 ms** | **16.5 ms** | 1.5 ms |
| p8-auto-headed | 8.3 / 10.3 / 34 ms | 0.13% | 1.7 ms | 14.1 ms | 2.1 ms |

- 有头（GPU 开）与 headless GPU 档结论一致：**GPU 开关不改变回包推迟**，推翻「headless 伪影」解释，PA07 的「有头 Chrome 复现」条件达成。
- 120 Hz 下掉帧几乎为零（p99 10.2–10.3 ms）；偶发 max（184–208 ms）为整局个别帧，比例 <0.15%。
- 与 headless 60 Hz 对照（[基线表 B](../2026-10-26-web-frame-latency/README.md)）：`transfer` 15.8→7.6 ms、`input_to_paint` 33.1→16.4 ms——**恰好按刷新周期等比缩放（1 帧/2 帧）**，确证「帧同步调度」根因而非固定开销。

## 2 PA07 判别微基准（三通道 + rAF 轮询设计验证）

程序：[microbench/](microbench/)（`index.html` + `drive.mjs`，有头 120 Hz + CDP 真实点击 ×60 + 连续动画使页面按帧合成 + COOP/COEP 跨源隔离）。原始输出 r1–r4。

| 通道 | run1 p50 / p95 | run2 p50 / p95 | 结论 |
|---|---:|---:|---|
| worker `postMessage`（任务） | 4.93 / 8.41 ms | 4.48 / 8.29 ms | 受输入后窗口推迟 |
| `MessageChannel` 主线程自发（任务） | 4.88 / 8.35 ms | 4.45 / 8.23 ms | **与 postMessage 完全一致** |
| 点击→下一 rAF（帧边界参考） | 4.56 / 7.99 ms | 4.25 / 7.98 ms | rAF 不被推迟 |

**判别结论**：
1. 输入后推迟是**任务级**的且与通道无关（MessageChannel 与 worker postMessage 逐点一致）——任何任务型投递（含 `Atomics.waitAsync` 回调，同为任务）都受同一窗口约束，「共享内存 + Atomics 通知」**按任务形式**不可缓解；
2. rAF 帧回调不被该窗口推迟（点击→rAF p50 ≈ 半周期）——**在 rAF 回调内轮询 SAB 并应用**是唯一结构出口：点击 t0 → worker ~1 ms 就绪写入 SAB → 下一帧 rAF（平均 t0+4.3 ms、最迟 t0+8.3 ms）读出并应用（其微任务在当帧提交前运行）→ 理论 input_to_paint p50 ≈ 6–7 ms、p99 ≤ 一帧+2 ms，满足 PA07 目标（≤1 刷新周期+2 ms）。此为 PA07 缓解原型的设计依据，游戏内实现待后续批。
3. 附注：微基准的 rAF 轮询通道样本为空——玩具页 drain 例程自身缺陷（r4 诊断：`reportCur=60` 证明 SAB 共享与 worker 写入完全正常，rAF 环在首次 drain 中断），不影响上述两条平台级结论；游戏内实现将以真实 perfProbe 覆盖并验证。

## 3 未覆盖

1. 有头 **60 Hz** 外接显示器档（本机仅内置 ProMotion；120 Hz 数据已足以判别缩放关系）；
2. Tauri WKWebView / Safari / Firefox / 移动端（后续切片）；
3. rAF 轮询 SAB 的游戏内端到端 A/B（PA07 缓解原型，待做）。

## 4 本批验证

| 命令 | 结果 |
|---|---|
| 冒烟 headless p4-click 30 步 | RESULT 正常（transfer 16.0 ms@60 Hz，与基线一致，证明测量栈重建等价） |
| 有头四档（1500 步上限，全部终局） | 4/4 正常终局，console error 0，Long Task 0 |
| 行数门禁（`check-source-lines.py`） | 通过 |
