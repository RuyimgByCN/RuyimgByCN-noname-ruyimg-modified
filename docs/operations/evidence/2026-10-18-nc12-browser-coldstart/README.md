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
