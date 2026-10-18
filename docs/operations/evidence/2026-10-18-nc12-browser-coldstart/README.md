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
