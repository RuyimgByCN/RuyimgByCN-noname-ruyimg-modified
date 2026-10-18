# NC09 真实浏览器**联机闭环**（创建并入座，2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）「正式联机房间接入原生权威规则」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**真实浏览器 → 真实 game-server（原生后端）→ 创建并入座 → 房间/座位/手牌渲染，无错误文案**；重连/命令/旁观仍待验收

## 1 方法（可复跑）

```bash
# ① 服务端（原生后端）
NONAME_ALLOW_UNSIGNED_EXTENSIONS=1 NONAME_NATIVE_ROOMS=1 NONAME_NATIVE_ROOM_PACKAGES=shahai-swd \
  GAME_SERVER_ADDR=127.0.0.1:7700 ./target/release/game-server
# ② 静态服务
(cd apps/web-client/dist && python3 -m http.server 8123 &)
# ③ CDP 驱动（Node 内置依赖；headless Chrome）
node apps/web-client/scripts/nc09-browser-online.mjs http://127.0.0.1:8123/ \
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" <outdir> ws://127.0.0.1:7700/ws
```

新增脚本 `apps/web-client/scripts/nc09-browser-online.mjs`：填昵称/服务端 URL（Vue `v-model`
需派发 `input` 事件）→ 点击「创建并入座」→ 轮询断言房间/座位出现 → 截图 + JSON 报告。

## 2 实测结果（[online-drive.log](online-drive.log)、[online-report.json](online-report.json)、[online-screenshot.png](online-screenshot.png)）

| 判据 | 结果 |
|---|---|
| 表单填写（昵称/服务端 URL） | `{"nick":true,"url":true}` |
| 点击「创建并入座」 | `true` |
| 房间出现 | `联机房间 #1`、**`房间 #1 · rev 0 · 55a2f9c5a222… · native`** ← **服务端房间运行原生后端** |
| 座位/名册 | `对局进行中`、`1 · cdp-p1 在线`、玩家概况 P1–P4（4/4） |
| 手牌 | `手牌（4 张）` |
| 错误文案 | `online_has_error_text: false` |
| 服务端日志 | `room created room=1 players=4`；页面关闭后 `connection closed conn=1` |
| 控制台 | **`console_errors: []`**（全流程无异常/console.error） |

## 3 未完成 / 待办（NC09 联机面）

- [ ] **重连**（断开后凭令牌回到原座位）、**命令提交**（联机出牌/结束阶段）、**旁观**（公开投影）
- [ ] 联机**多客户端**（第二个浏览器上下文加入同一房间，校验各座位投影互不泄漏）
- [ ] 联机**存读档/录像**（服务端 WAL/快照 + 客户端回放）
- [ ] 锦标赛面（manifest 固定/分块签名）原生未实现（明确登记）
- [ ] 网络异常（服务端不可达、断线、慢消费者）与错误展示
- [ ] 平台档位（Windows/Linux/Android/iOS）与性能档位
- [ ] 临时服务端/静态服务仅用于本轮验收，已终止

## 4 追加：联机命令 + 自动重连（2026-10-18 同轮）

### 4.1 联机命令（浏览器 → 权威服务端）

点击「结束阶段」→ **`rev 0 → 1`**（按钮 `found:true/disabled:false`）——浏览器提交的命令经
WebSocket 到权威房间并被接受（[online-drive-4.log](online-drive-4.log)）。

### 4.2 重连：先暴露缺口，再补客户端自动恢复

- **首次实测（缺口）**：根 URL 重载后停在**大厅**（`hasRoom:false`）→ 客户端**不会自动回到房间**；
  进一步核对源码：令牌**已**落 `localStorage`（`reconnectKey(roomId)`），`joinRoom` 也会带令牌，
  但**没有启动时自动恢复会话**的入口 → 需用户手动重新进入房间。
- **修复**（`src/main.ts`）：启动时读取 `sessionStorage["noname-next.online-intent"]`，若非旁观意图
  则 `router.push("/battle?mode=online&action=join&room=…")` —— 结合已存的令牌，重连命中原座位。
- **复测**：根 URL 重载后 **自动回到房间** —— `hasRoom:true`、`hasSeat:true`、
  `房间 #N · rev 0 · 55a2f9c5a222… · native`、`1 · cdp-p1 在线`，**`console_errors: []`**。

> 说明：本轮每次运行都会新建房间（房间号递增，如 #4）；重连断言的是**回到该次运行的房间**。

### 4.3 仍未完成（更新）

- [ ] **旁观**（`Spectate`）在浏览器中的公开投影与信息隔离断言
- [ ] **多客户端**同房间（第二个浏览器上下文）与跨座位信息隔离
- [ ] 断线**服务端侧**重连（关闭页面后凭令牌重连）与过期令牌拒绝
- [ ] 联机存读档/录像、网络异常（服务端不可达/慢消费者）与错误展示
- [ ] 客户端路由需服务端 SPA fallback（部署配置；本轮以根 URL 导航规避）
