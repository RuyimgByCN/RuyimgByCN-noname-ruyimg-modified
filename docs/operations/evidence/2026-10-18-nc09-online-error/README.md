# NC09 网络异常可见性（服务端不可达，2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）「错误展示 / 网络异常」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**服务端不可达时 UI 显示可见错误（`WebSocket 连接失败`），并修复了 2 处未处理 promise 拒绝**

## 1 方法（可复跑）

```bash
(cd apps/web-client/dist && python3 -m http.server 8123 &)   # 只起静态服务，**不起** game-server
node apps/web-client/scripts/nc09-browser-online-error.mjs http://127.0.0.1:8123/ \
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" <outdir> ws://127.0.0.1:7799/ws
```

新增脚本 `apps/web-client/scripts/nc09-browser-online-error.mjs`：填昵称 + **无人监听的 ws 地址**
→ 点「创建并入座」→ 断言界面出现可见错误文本 → 截图 + JSON 报告（含控制台错误）。

## 2 实测结果

| 判据 | 修复前 | 修复后 |
|---|---|---|
| 可见错误文本 | `WebSocket 连接失败`（界面） | `WebSocket 连接失败`（界面，保持） |
| 控制台 | **`exception: Uncaught (in promise)`** | **`console_errors: []`** |

证据：[error-drive.log](error-drive.log)（修复前）、[error-drive-after-fix2.log](error-drive-after-fix2.log)、
[online-error-report.json](online-error-report.json)、[online-error-screenshot.png](online-error-screenshot.png)。

## 3 修复的两处未处理拒绝（真实缺陷）

1. `src/views/BattleView.vue`：`void connectAndStart()` 未捕获 → 改为
   `connectAndStart().catch(...)`，把失败写入状态行与战斗日志（并置 `store.connected = false`）；
2. `src/engine/networkClient.ts`：`connect()` 中 `opened` 失败时，`welcome` 仍会在稍后 reject
   → 预先 `void welcome.catch(() => undefined)`，避免未处理拒绝（UI 已显示错误文本，但控制台仍报错）。

## 4 未完成 / 待办（NC09）

- [ ] 断线**中途**（已连接后服务端崩溃）的错误展示与自动重连策略
- [ ] 限流（`max_cmds_per_sec`）与慢消费者的用户可见反馈
- [ ] 联机存读档/录像、锦标赛面、平台与性能档位、服务端 SPA fallback
