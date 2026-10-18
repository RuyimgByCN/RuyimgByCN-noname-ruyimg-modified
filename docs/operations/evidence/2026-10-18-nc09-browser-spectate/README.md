# NC09 真实浏览器**旁观 + 多客户端信息隔离**（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）「观察者投影 / 跨座位信息隔离」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**两个真实浏览器页面同房间（入座 + 旁观）通过；旁观者只见公开投影与手牌数量，看不到手牌内容**

## 1 方法（可复跑）

```bash
NONAME_ALLOW_UNSIGNED_EXTENSIONS=1 NONAME_NATIVE_ROOMS=1 NONAME_NATIVE_ROOM_PACKAGES=shahai-swd \
  GAME_SERVER_ADDR=127.0.0.1:7700 ./target/release/game-server
(cd apps/web-client/dist && python3 -m http.server 8123 &)
node apps/web-client/scripts/nc09-browser-spectate.mjs http://127.0.0.1:8123/ \
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" <outdir> ws://127.0.0.1:7700/ws
```

新增脚本 `apps/web-client/scripts/nc09-browser-spectate.mjs`：CDP `Target.createTarget` 开**第二个页面**
→ 页面 A 创建并入座、页面 B 填房间号点「旁观」→ 分别截图与报告。

## 2 实测结果（[spectate-drive.log](spectate-drive.log)、[spectate-report.json](spectate-report.json)）

| 判据 | 结果 |
|---|---|
| 页面 A 创建并入座 | `{"clicked":true}`；房间号 **#1** |
| 页面 B 旁观同一房间 | `{"clicked":true}`；页面出现 **`公共旁观投影`** |
| 旁观者可见房间与投影 | `hasRoom: true`、`房间 #1 · rev 0 · 55a2f9c5a222… · native` |
| 旁观者可见名册 | `hasRoster: true`、玩家概况 P1–P4（**仅数量 `4🂠`**，身份未知 `?`） |
| **旁观者手牌** | **`手牌（0 张 · 点击或拖拽）`**、`hasOwnHandCards: false` → **看不到任何手牌内容** |
| 控制台 | **主页面与旁观页面均为 `[]`**（`console_errors_host` / `console_errors_spectator`） |

## 3 未完成 / 待办（NC09 联机面）

- [ ] **断线（服务端侧）重连**：关闭页面后凭令牌重连、过期/伪造令牌拒绝
- [ ] **命令在多客户端下的投影一致性**（一方出牌 → 另一方与旁观者同步更新且不泄漏隐藏信息）
- [ ] 联机**存读档/录像**（服务端 WAL/快照 + 客户端回放）
- [ ] 网络异常（服务端不可达、慢消费者、限流）与错误展示
- [ ] 锦标赛面（manifest 固定/分块签名，原生未实现）
- [ ] 平台档位（Windows/Linux/Android/iOS）与性能档位
- [ ] 服务端 SPA fallback（部署配置；本轮以根 URL 导航规避）
- [ ] 本轮临时服务端/静态服务已终止

## 4 追加：多客户端命令一致性（2026-10-18 同轮）

页面 A（入座者）点击「结束阶段」→ 断言**旁观页面 B** 同步前进且不泄漏：

| 判据 | 结果 |
|---|---|
| 页面 A rev | `0 → 1`（命令被权威服务端接受） |
| **旁观页面 B rev** | **`0 → 1`**（收到同一权威状态更新） |
| 旁观者更新后仍为公开面 | `手牌（0 张 · 点击或拖拽）`、`hasOwnHandCards: false` |
| 双页面控制台 | `console_errors_host: []`、`console_errors_spectator: []` |

证据：[spectate-drive-2.log](spectate-drive-2.log)、[spectate-report.json](spectate-report.json)、
[host-screenshot.png](host-screenshot.png)、[spectator-screenshot.png](spectator-screenshot.png)。

### 仍未完成（更新）

- [ ] 断线（服务端侧）重连与过期/伪造令牌拒绝
- [ ] 两名**入座玩家**之间的命令与投影一致性（本轮为「入座 + 旁观」）
- [ ] 联机存读档/录像、网络异常（不可达/慢消费者/限流）与错误展示
- [ ] 锦标赛面（manifest 固定/分块签名，原生未实现）、平台与性能档位、服务端 SPA fallback
