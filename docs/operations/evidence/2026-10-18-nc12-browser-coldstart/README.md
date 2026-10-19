# NC12 真实浏览器档位：**「创建到可操作」**（headless Chrome，3 轮）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC12**（[方案 §NC12](../../../architecture/native-convergence-plan.md#nc12)）「浏览器档位 / 冷启动」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**真实浏览器（headless Chrome）下「点击本地开局 → 战斗界面可交互」p50 = 263 ms**（3/3 成功、控制台零错误）

## 1 方法（可复跑）

```bash
(cd apps/web-client/dist && python3 -m http.server 8123 &)
node apps/web-client/scripts/nc12-browser-coldstart.mjs http://127.0.0.1:8123/ \
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" <outdir> 3
```

`apps/web-client/scripts/nc12-browser-coldstart.mjs`（新增）：每轮**全新 Chrome profile**（磁盘缓存冷）
→ 导航 → 等大厅出现「本地开局」→ **点击** → 轮询「战斗界面出现（阶段/座位标记 + 正文 >200 字符）」
→ 记录墙钟，并采集 `PerformanceNavigationTiming` 与资源条目。

## 2 实测（[coldstart.log](coldstart.log)、[browser-coldstart.json](browser-coldstart.json)、[perf-summary.txt](perf-summary.txt)）

| 指标 | 值 |
|---|---|
| `to_interactive_ms`（3 轮） | **261 / 263 / 264** → **p50 = 263 ms**、max 264 ms |
| 可交互成功 | **3/3**（`all_interactive: true`） |
| 控制台错误 | **0**（`console_errors: []`） |
| 页面导航 `DOMContentLoaded` / `load` | 51.3/48.1/42.6 ms、52.9/48.7/43.3 ms |
| Worker 产物 | `engine.native.worker-JFW_PAlJ.js`（每轮均加载） |

**读法**：页面**加载到大厅**约 43–53 ms（本机静态服务）；**点击到可交互** 263 ms —— 后者包含
**worker 启动 + 原生 WASM 模块下载/实例化 + 首次建局 + 首帧渲染**（这些发生在首次本地开局时）。

## 3 口径边界（不得外推）

- **headless Chrome + 本机静态服务**：不含真实网络（CDN/首包 RTT）、不含浏览器扩展与 GPU 合成差异；
- **`wasm` 资源条目为空**：WASM 在 **worker 内**加载，**不进页面的 resource timing** → 本口径**未单独
  量化模块下载/编译耗时**（263 ms 中已包含，但无法拆分）→ 登记待办；
- 「可交互」的判据是**战斗界面出现**（阶段/座位标记 + 正文长度），**不是**完整交互验收；
- 单机 3 轮，未做多设备/多浏览器（Safari/Firefox）与移动端。

## 4 未完成 / 待办（NC12 浏览器档位）

- [ ] **拆分** 263 ms：worker 启动 / WASM 下载+编译 / 建局 / 首帧（需在 worker 内埋点或
      `PerformanceObserver` 于 worker 上下文）
- [ ] 多浏览器（Safari/Firefox）与移动端（Android/iOS）档位
- [ ] 真实网络（非本机）与慢网/丢包
- [ ] 浏览器内**完整对局**（本轮只到可交互）与动画帧/输入响应指标
- [ ] 与旧线前端（`VITE_ENGINE_BACKEND=legacy`）同口径对照

## 5 追加：旧线前端同口径对照（`VITE_ENGINE_BACKEND=legacy`）（2026-10-19 同轮）

```bash
VITE_ENGINE_BACKEND=legacy pnpm --dir apps/web-client exec vite build --outDir dist-legacy
(cd apps/web-client/dist-legacy && python3 -m http.server 8124 &)
node apps/web-client/scripts/nc12-browser-coldstart.mjs http://127.0.0.1:8124/ <chrome> <outdir> 3
```

| 档位 | `to_interactive_ms`（3 轮） | p50 | 可交互 | 控制台 |
|---|---|---:|---|---|
| **原生（默认后端）** | 261 / 263 / 264 | **263 ms** | 3/3 | 0 |
| **旧线（`legacy`）** | 260 / 260 / 260 | **260 ms** | 3/3 | 0 |

**读法（不夸大）**：两者在本机 headless 口径下**基本相同（差 3 ms，落在噪声内）**——
「点击到可交互」主要由 **worker 启动 + WASM 模块下载/实例化**决定，两条后端路径的**前端装配成本相当**；
**本对照不构成性能优劣结论**（差异未超出测量噪声，且两条后端的功能面不等价）。
日志：[legacy-coldstart.log](legacy-coldstart.log)、[legacy-coldstart.json](legacy-coldstart.json)。

### 仍未完成（更新）

- [ ] 拆分 263/260 ms（worker 内埋点或 worker 上下文 `PerformanceObserver`）
- [ ] 多浏览器（Safari/Firefox）与移动端（Android/iOS）
- [ ] 真实网络（非本机）与慢网/丢包
- [ ] 浏览器内**完整对局**与动画帧/输入响应指标
- [ ] 两档位的**功能等价**对照（本条只比冷启动耗时，不比功能面）

## 6 工程注意：对照构建产物**不得留在门禁树内**（2026-10-19 同轮）

`VITE_ENGINE_BACKEND=legacy vite build --outDir dist-legacy` 会把**构建产物**写入
`apps/web-client/dist-legacy/`，而**行数门禁覆盖 `apps/`**（`scripts/check-source-lines.py`）→
实测被拦下：`apps/web-client/dist-legacy/assets/index-*.js: 5013 lines (limit 2000)`。

**处置**：对照测量完成后**删除该构建产物**（`rm -rf apps/web-client/dist-legacy`），
门禁恢复通过（8297 文件 ≤ 2000 行）。**后续复跑该对照时同样需在测量后清理**（或将产物放到门禁树外）。

## 7 追加：**263 ms 的拆分**（worker 内计时，2026-10-19 同轮）

**做法**：在原生 worker 内对初始化分段计时（`apps/web-client/src/worker/nativeBackend.ts`
的 `ensureNativeReady`）：①`fetch` 下载 `.wasm` 字节；②`init({module_or_path: bytes})` 编译+实例化；
结果随 `init` 回包以 `init_timings` 上报，客户端诊断钩子写入 `window.__nativeInitTimings`
（`src/engine/client.ts`，**不参与业务逻辑**），测量脚本读取。

**实测（3 轮，[coldstart-split.log](coldstart-split.log)）**：

| 轮 | `to_interactive_ms` | `fetch_ms` | `instantiate_ms` | worker 初始化合计 | `wasm_bytes` |
|---|---:|---:|---:|---:|---:|
| 0 | 260 | 7.3 | 1.8 | **9.1** | 2,644,903 |
| 1 | 260 | 6.5 | 1.7 | **8.2** | 2,644,903 |
| 2 | 260 | 5.9 | 1.9 | **7.8** | 2,644,903 |

**读法与更正（重要）**：WASM **下载 + 编译实例化合计仅 ~8–9 ms**（模块 **2.64 MB**），
而「点击到可交互」为 **260 ms** → **模块装载只占 ~3%**，**不是**主要成本。
因此 §2/§5 中「263 ms 主要由 worker 启动 + WASM 下载/实例化决定」的推断**不成立，予以更正**：
余下 **~251 ms** 属**应用侧**（worker 启动、建局、选将面板 93 项候选渲染、Vue 首帧等），
**尚未进一步拆分**（登记待办）。

### 仍未完成（更新）

- [ ] 应用侧 ~251 ms 的进一步拆分（建局 / 候选面板 / Vue 首帧；需在客户端埋点）
- [ ] 多浏览器（Safari/Firefox）与移动端（Android/iOS）、桌面（Tauri）
- [ ] 真实网络（非本机）与慢网/丢包
- [ ] 浏览器内**完整对局**（已在 [fullgame 证据](../2026-10-18-nc12-browser-fullgame/README.md)）与动画帧/输入响应

## 8 追加：**应用侧 ~251 ms 的拆分**（页面内埋点，2026-10-19 同轮）

**做法**：在 `BattleView.vue::connectAndStart` 的本地开局路径埋 `performance.now()` 标记
（诊断钩子 `window.__nativeStartMarks`，**不参与业务逻辑**）：
`t0_click`（进入开局流程）→ `t1_worker_ready`（worker 连接完成，含 WASM 初始化）
→ `t2_start_done`（建局往返完成）→ `t3_candidates_loaded`（选将候选拉取完成）
→ `t4_first_frame`（`nextTick` 后首帧）。

**实测（3 轮，[coldstart-appsplit.log](coldstart-appsplit.log)）**：

| 轮 | worker 连接 | 建局 | 候选拉取 + 首帧 | 合计（t0→t4） | 其中 WASM 初始化 |
|---|---:|---:|---:|---:|---:|
| 0 | 17.8 | 2.3 | **125.5** | **145.6** | 8.9 |
| 1 | 15.4 | 2.3 | **126.2** | **143.9** | 8.0 |
| 2 | 15.7 | 2.3 | **125.9** | **143.9** | 8.1 |

**结论与更正（重要）**：
1. **应用侧主成本是「选将候选拉取 + 首帧渲染」≈ 126 ms**（候选 93 项），而非 worker 启动或建局
   （两者合计 < 20 ms，建局仅 2.3 ms）；
2. **此前的 `to_interactive_ms` 260 ms 是「轮询口径」**（脚本每 **250 ms** 轮询一次可交互标志）→
   含最多 250 ms 的**测量延迟**；页面内埋点的精确值为 **≈ 144–146 ms**。
   → 后续结论应以**埋点值**为准，`to_interactive_ms` 保留为跨轮可比的历史口径（同一脚本、同一轮询间隔）。

### 仍未完成（更新）

- [ ] 选将候选 126 ms 的进一步优化/拆分（候选生成在引擎侧还是渲染侧；可否懒加载）
- [ ] 多浏览器（Safari/Firefox）与移动端（Android/iOS）、桌面（Tauri）
- [ ] 真实网络（非本机）与慢网/丢包
- [ ] 动画帧/输入响应指标；浏览器内完整对局见 [fullgame 证据](../2026-10-18-nc12-browser-fullgame/README.md)
