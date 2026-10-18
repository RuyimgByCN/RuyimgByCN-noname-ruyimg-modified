# NC09 真实浏览器端到端（首切片，2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）「真实客户端」层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**真实浏览器（headless Chrome 155）加载构建产物并渲染出原生后端大厅，控制台零错误**；**未驱动实际对局流程**（登记）

## 1 环境与方法（可复跑）

| 项 | 值 |
|---|---|
| 浏览器 | `Google Chrome 155.0.8059.82`（`/Applications/Google Chrome.app`），`--headless=new` |
| 构建 | `pnpm --dir apps/web-client build` → exit 0（`dist/`，679 ms） |
| 静态服务 | `python3 -m http.server 8123`（`apps/web-client/dist/`） |
| 服务端（可选） | `NONAME_ALLOW_UNSIGNED_EXTENSIONS=1 NONAME_NATIVE_ROOMS=1 NONAME_NATIVE_ROOM_PACKAGES=shahai-swd ./target/release/game-server` → 日志 `native content ready packages=41`、`listening addr=127.0.0.1:7700` |
| 命令 | `chrome --headless=new --user-data-dir=<tmp> --window-size=1280,800 --virtual-time-budget=12000 --screenshot=... --dump-dom <url>` |

> `--user-data-dir` 必需：不带时 Chrome 报 `Failed to create a unique user data directory for headless`（首次尝试记录在 [chrome-stderr.log](chrome-stderr.log)）。

## 2 实测结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 页面加载与控制台 | **零 `SEVERE`/`Uncaught`/`Failed to load`** | [chrome-console-root.log](chrome-console-root.log) |
| 根路由渲染 | `<main>` 有内容：2243 字节 / 368 字符文本 | [dom_.html](dom_.html) |
| 渲染内容（节选） | 「确定性身份局 · 默认原生引擎后端（engine-native-wasm 窄壳，Worker 内运行）…人数 2–8、种子、本地开局、联机大厅（通过 game-server WebSocket 加入权威房间）…创建并入座/加入房间/旁观…」 | [dom_.html](dom_.html) |
| 截图 | 顶部导航（大厅/对局/扩展/设置/录像/武将库）渲染正常 | [screenshot-root.png](screenshot-root.png) |

**说明**：先前的两张截图（`screenshot.png`、`screenshot-with-server.png`）内容为空，原因是访问的是
`/index.html` —— 该 SPA 的路由按路径匹配，`/index.html` 不匹配任何路由（`<main>` 仅 `<!---->`）；
访问 `/` 即正常渲染（本轮已复现并区分，属**测试 URL 选择问题**，不是应用缺陷）。

## 3 边界与未验收项（不得外推）

- [ ] **未驱动实际对局**：未在浏览器内点击「本地开局」跑完整流程，也未走联机加入/命令/重连；
      本轮只证明**构建产物在真实浏览器中加载、渲染大厅、无控制台错误**
- [ ] 未做**视觉/交互验收**（点击、键盘、触摸、窄屏、中英文、可访问性）
- [ ] 静态服务用 `python3 -m http.server`（**无 SPA fallback**）→ `/extensions`、`/collection`
      直接 404（[dom_extensions.html](dom_extensions.html)、[dom_collection.html](dom_collection.html) 为空）
      → 客户端路由需配合服务端 fallback 或 hash 路由，属**部署配置**待办
- [ ] 未验证浏览器内的 **Worker + WASM 原生路径**是否真正进入（大厅文案如此声明，但未断言 Worker 已起、
      未读 Worker 内的引擎状态）
- [ ] 平台档位（Windows/Linux/Android/iOS）与性能档位（动画帧/输入响应）未验收
- [ ] 本轮启动的服务端为**本地临时进程**，已在收尾时终止（非产品部署）

## 4 复跑命令

```bash
cd noname-next
pnpm --dir apps/web-client build
(cd apps/web-client/dist && python3 -m http.server 8123 &)
UDD=$(mktemp -d)
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --headless=new --disable-gpu \
  --user-data-dir="$UDD" --window-size=1280,800 --virtual-time-budget=12000 \
  --screenshot=shot.png --dump-dom http://127.0.0.1:8123/ > dom.html 2> console.log
```

## 5 CDP 驱动：本地开局 → 发现并修复**生产构建缺陷**（2026-10-18 同轮）

### 5.1 驱动方式

新增 `apps/web-client/scripts/nc09-browser-drive.mjs`：headless Chrome + **CDP**
（`--remote-debugging-port` + Node 内置 `WebSocket`，**无新增依赖**）——导航 → 断言大厅 →
点击「本地开局」→ 等待战斗界面 → 截图 + 报告（`drive-report.json`）。

### 5.2 首次驱动：暴露真实缺陷（修复前）

```
STEP lobby_ready ok
STEP clicked_local_start ok
STEP battle_view {"len":180,"hasPhase":true,"hasSeat":false, ... "未连接 ... 手牌（0 张）"}
RESULT {"console_errors":["console.error: engine worker error Event"]}
```

- 现象：战斗页出现但 **worker 未连接**（「未连接」、手牌 0）、控制台 `engine worker error Event`；
- 根因（构建产物核对）：`dist/assets/` **只有 `index.js` + css，没有 worker chunk**，
  `index.js` 中 `engine_native_wasm` 出现 **0 次** → `new Worker(new URL("../worker/engine.native.worker.ts", import.meta.url))`
  在**生产构建**下未产出 worker chunk → 浏览器加载 worker 失败。

### 5.3 修复与复测（修复后）

- 修复（`src/engine/client.ts`）：改用 Vite 的 **`?worker` 导入**
  （`import NativeEngineWorker from "../worker/engine.native.worker.ts?worker"`，旧线同理），
  并新增 `src/vite-worker.d.ts` 类型声明；
- 构建产物（`pnpm build` exit 0）：`engine.native.worker-*.js`、`engine.worker-*.js`、
  **`engine_native_wasm_bg-*.wasm`**（原生 WASM）均产出；
- 复测（[drive-after-fix.log](drive-after-fix.log)、[battle-screenshot.png](battle-screenshot.png)）：

```
STEP battle_view {"len":2383,"hasPhase":true,"hasSeat":true, "rev 0 · native ... 选将
   NC05 · 目标座位 P1 · 93 项候选 · 点击候选即按名装配（原生正式入口，非 debug）"}
RESULT {"lobby_has_local_start":true,"battle_text_len":2383,"battle_has_phase":true,"battle_has_seat":true,"console_errors":[]}
```

→ **真实浏览器中本地开局成功、原生 worker 运行（`rev 0 · native`）、选将面板显示 93 项候选、控制台零错误**。

### 5.4 仍未完成（更新）

- [ ] 未在浏览器内**完成一整局**（未点选将、未出牌、未存档/读档、未点自动一步）
- [ ] 未走**联机大厅**（WebSocket 加入/命令/重连）
- [ ] 未做视觉/交互/可访问性与平台档位验收
- [ ] 客户端路由需服务端 SPA fallback（部署配置）
- [ ] 本轮临时 `http.server` 已终止；CDP 驱动脚本仅 Node 内置依赖

## 6 浏览器内完整会话驱动：选将 → 自动推进 → 存档/读档（2026-10-18 同轮）

在 §5 修复后，把 CDP 驱动脚本扩展到**真实会话**（[drive-full-session.log](drive-full-session.log)、
[drive-report.json](drive-report.json)、[battle-after-actions.png](battle-after-actions.png)）：

| 步骤 | 实测 |
|---|---|
| ① 大厅 | `lobby_has_local_start: true` |
| ② 本地开局 | 战斗页渲染，`rev 0 · native`（**原生后端在浏览器中运行**） |
| ③ **选将**（点击候选「样例·张角组合」） | UI 显示 `✅ 已为 P1 装配 样例·张角组合 · state_hash 79543a31f338f687…` + `刷新后视图技能：天妒 · 鬼道` —— UI→Worker→WASM→内核→投影→UI 全链路 |
| ④ **自动一步 ×3** | **`rev 0 → 3`**（按钮 `found:true/disabled:false`；对局在浏览器中真实推进） |
| ⑤ **存档 / 读档** 点击 | `saveRes: true`、`loadRes: true`，**控制台零错误** |
| ⑥ 控制台 | `console_errors: []`（全流程无 `SEVERE`/`Uncaught`/`console.error`） |

### 仍未完成（更新）

- [ ] **联机大厅**未走通（WebSocket 加入房间/提交命令/重连/旁观）——本轮只验证本地原生路径
- [ ] 未验证**整局终局**（只推进 3 步）、未验证读档后状态一致性（只点击按钮未断言恢复结果）
- [ ] 视觉/交互（点击、键盘、触摸、窄屏、中英文、可访问性）与平台档位未验收
- [ ] 客户端路由需服务端 SPA fallback（部署配置）
